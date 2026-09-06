//! Where a run's container workdirs live, and how one run claims a name
//! without stepping on another's.
//!
//! Extracted from `lib.rs`, which is far past the file-length ratchet's line
//! and may only shrink (`scripts/check-file-length.sh`). Nothing here
//! changed in the move: the derived name, the lock, the takeover rule and
//! the sweep are the same code, and `LocalExecutor::new`'s doc comment is
//! still where the reasoning for a DERIVED name is written down.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use crate::LocalExecutor;

/// Where a run's container workdirs live.
///
/// Stable when this run could claim the name for its project, ephemeral when
/// it could not. See [`LocalExecutor::new`] for why the stable one matters.
pub(crate) enum ScratchRoot {
    /// A path derived from the project, held under a lock file and removed
    /// when the run ends.
    Stable { path: PathBuf, lock: PathBuf },
    /// A fresh random directory, removed on drop by `tempfile`.
    Ephemeral(tempfile::TempDir),
}

impl ScratchRoot {
    pub(crate) fn path(&self) -> &Path {
        match self {
            ScratchRoot::Stable { path, .. } => path,
            ScratchRoot::Ephemeral(dir) => dir.path(),
        }
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        if let ScratchRoot::Stable { path, lock } = self {
            let _ = std::fs::remove_dir_all(path);
            let _ = std::fs::remove_file(lock);
        }
    }
}

impl LocalExecutor {
    pub(crate) fn scratch_root(project: Option<&Path>) -> Result<ScratchRoot> {
        // Once per process, before this run takes a lock of its own: a run
        // that crashed leaves its directory behind, and nothing else ever
        // removes it.
        static SWEEP: std::sync::Once = std::sync::Once::new();
        SWEEP.call_once(|| sweep_stale_roots(&std::env::temp_dir()));

        let ephemeral = || -> Result<ScratchRoot> {
            Ok(ScratchRoot::Ephemeral(
                tempfile::Builder::new()
                    .prefix("hickory-local-")
                    .tempdir()
                    .context("failed to create LocalExecutor temp root")?,
            ))
        };
        // Canonicalised so `.`, `../proj` and an absolute path all name one
        // root. A path that cannot be canonicalised is used as given rather
        // than rejected: a worse name is still better than the wrong one.
        //
        // The EMPTY path is the exception, and it was a real bug rather than a
        // theoretical one. `Path::new("d.hick").parent()` is `Some("")`, not
        // `None`, so a caller writing `parent().unwrap_or(".")` handed an
        // empty path straight through — and `short_key("")` mixes in no bytes
        // and returns the FNV basis, the same twelve characters every time. So
        // every `hick run <bare-filename>` on the machine, in any project,
        // shared ONE scratch root: they raced for the lock, and a run that
        // took over a lock whose holder looked gone wiped a live run's
        // workdir, which surfaced as `failed to spawn sh -c: No such file or
        // directory`. Falling back to the working directory gives each project
        // its own key again.
        let key = match project.filter(|dir| !dir.as_os_str().is_empty()) {
            Some(dir) => std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf()),
            None => match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(_) => return ephemeral(),
            },
        };
        if key.as_os_str().is_empty() {
            return ephemeral();
        }
        let path = std::env::temp_dir().join(format!("hickory-local-{}", short_key(&key)));
        let lock = path.with_extension("lock");

        // Take the lock, or take over one whose process is gone: a run killed
        // with SIGKILL never removes its own, and a scratch directory nobody
        // can ever use again would be a worse bug than the one this fixes.
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(mut file) => {
                use std::io::Write as _;
                let _ = writeln!(file, "{}", std::process::id());
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if holder_is_alive(&lock) {
                    log::info!(
                        "another run holds {}; this one uses a temporary directory instead, \
                         so paths in its output will not match a previous run's",
                        path.display()
                    );
                    return ephemeral();
                }
                let _ = std::fs::write(&lock, format!("{}\n", std::process::id()));
            }
            Err(_) => return ephemeral(),
        }

        // A previous run's leftovers must not be visible to this one.
        let _ = std::fs::remove_dir_all(&path);
        if std::fs::create_dir_all(&path).is_err() {
            let _ = std::fs::remove_file(&lock);
            return ephemeral();
        }
        Ok(ScratchRoot::Stable { path, lock })
    }
}

/// A short, stable key for a path: FNV-1a/64 folded to 12 hex characters.
///
/// Not a cryptographic hash and does not need to be — this names a temp
/// directory, and the only property required is that the same project gets
/// the same name on every run of the same machine.
pub(crate) fn short_key(path: &Path) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in path.as_os_str().as_encoded_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    // Masked to 48 bits so the result is EXACTLY 12 hex characters. `{:012x}`
    // pads a short value but does not truncate a long one, and a caller that
    // recognises our directories by their shape needs a fixed width.
    format!("{:012x}", hash & 0xffff_ffff_ffff)
}

/// Whether the process named in a lock file still exists.
///
/// Unix only in the strict sense; elsewhere the lock is treated as live,
/// which costs a run its stable path and never costs it correctness.
/// The project key in a derived scratch-root name, if the name is one of ours.
///
/// [`short_key`] makes exactly 12 hex characters so a derived root can be
/// recognised by shape. That matters because the ephemeral fallback asks
/// `tempfile` for the SAME `hickory-local-` prefix, so the prefix alone does
/// not tell a derived root from a random directory that a live run is using —
/// and anything that deletes by name has to know the difference.
pub fn derived_root_key(name: &str) -> Option<&str> {
    let key = name.strip_prefix("hickory-local-")?;
    (key.len() == 12 && key.chars().all(|c| c.is_ascii_hexdigit())).then_some(key)
}

/// Remove derived scratch roots whose owning run is gone.
///
/// These accumulate: a run that does not exit cleanly never drops its
/// `ScratchRoot`, so the directory and its lock stay behind. 42 were present
/// in the temp directory during one test run, and nothing had ever removed
/// one.
///
/// Only a root whose lock says its holder is gone is touched, which is the
/// same question the takeover path asks and is answered by the same function.
/// Two things are deliberately left alone:
///
/// * A directory with **no lock**, because the ephemeral fallback is a
///   `tempfile` with our own prefix and deleting one would pull the ground
///   out from under a live run.
/// * `hickory-home-*`, the persistent cell homes, which exist precisely to
///   outlive a run and are not leftovers.
pub(crate) fn sweep_stale_roots(temp: &Path) {
    let Ok(entries) = std::fs::read_dir(temp) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        // Walk the LOCKS, not the directories: a lock is the only evidence
        // that a directory was ever claimed, and a lockless one is either in
        // use or not ours to judge.
        let Some(stem) = name.strip_suffix(".lock") else {
            continue;
        };
        if derived_root_key(stem).is_none() {
            continue;
        }
        let lock = entry.path();
        if holder_is_alive(&lock) {
            continue;
        }
        let root = temp.join(stem);
        log::debug!(
            "sweeping scratch root {} of a run that is gone",
            root.display()
        );
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&lock);
    }
}

/// How long a lock with no readable pid is assumed to belong to whoever is
/// still writing it.
///
/// `create_new` makes the file and the pid lands a moment later, so the two
/// are not one operation and a reader can catch the gap.
pub(crate) const UNWRITTEN_LOCK_GRACE: std::time::Duration = std::time::Duration::from_secs(30);

/// Whether a lock is too young to be anything but a half-written one.
///
/// A lock that crashed between the two syscalls is indistinguishable from one
/// mid-write except by age, and guessing "held" forever would wedge a project's
/// scratch directory permanently. Age settles it: microseconds old is the gap,
/// half a minute old is a leftover.
fn lock_was_just_created(lock: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(lock) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        // No mtime to reason about — assume held, because deleting a live
        // run's workdirs is the worse of the two mistakes.
        return true;
    };
    modified
        .elapsed()
        .map(|age| age < UNWRITTEN_LOCK_GRACE)
        .unwrap_or(true)
}

pub(crate) fn holder_is_alive(lock: &Path) -> bool {
    let Ok(body) = std::fs::read_to_string(lock) else {
        return false;
    };
    let Ok(pid) = body.trim().parse::<u32>() else {
        // An EMPTY lock is the dangerous case, and it is the one that bit:
        // reading "no pid" as "nobody owns this" let a second run take over
        // and `remove_dir_all` a directory whose owner was a microsecond from
        // using it. That is exactly how a live run's workdir disappeared and
        // surfaced as `failed to spawn 'sh -c': No such file or directory`.
        return lock_was_just_created(lock);
    };
    if pid == std::process::id() {
        return true;
    }
    #[cfg(unix)]
    {
        Path::new(&format!("/proc/{pid}")).exists()
            || unsafe { libc::kill(pid as i32, 0) } == 0
            || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }
    #[cfg(not(unix))]
    {
        true
    }
}
