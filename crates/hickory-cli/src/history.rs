//! Recording what every writer did, before it did it.
//!
//! `docs/specs/freeform/local-history.md`. The store itself is
//! [`hickory_workspace::history`]; this is the seam every writer in the
//! product goes through to leave a stop behind.
//!
//! Two rules shape the whole module.
//!
//! **Local history must never break a write.** Every function here swallows
//! its own failures and logs them. A person's run must not fail because a
//! cache on their own machine could not be written — this store exists to
//! make things recoverable, and a version of it that can take a run down
//! makes them less so.
//!
//! **The before-bytes are read from disk, here, immediately before the
//! write.** Not passed in, and not remembered from earlier: whatever is on
//! disk right now is what a person would lose, whoever put it there.

use std::path::{Path, PathBuf};

use hickory_workspace::WorkspaceStore;
use hickory_workspace::history::{Act, ActKind, History, PendingChange, Retention};

/// How much local history to keep, from the environment.
///
/// Read here rather than threaded through every caller because it is machine
/// configuration for a machine-local cache — and validated rather than
/// silently defaulted, so a typo says so instead of quietly halving somebody's
/// history.
pub fn retention() -> Retention {
    let mut retention = Retention::default();
    if let Some(bytes) = parsed("HICKORY_HISTORY_BYTES") {
        retention.max_bytes = bytes;
    }
    if let Some(days) = parsed("HICKORY_HISTORY_DAYS") {
        retention.max_days = days;
    }
    retention
}

fn parsed(name: &str) -> Option<u64> {
    let raw = std::env::var(name).ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    match raw.parse::<u64>() {
        Ok(value) => Some(value),
        Err(_) => {
            eprintln!(
                "warning: {name} is not a whole number ({raw:?}), so local history is using its \
                 default. Set it to a count, like {name}=7."
            );
            None
        }
    }
}

/// The store for a project, or `None` when there is nowhere to put one.
///
/// `None` is a normal outcome: a machine with no data directory, or one where
/// it cannot be created, still runs everything else.
pub fn open(root: &Path) -> Option<History> {
    let store = WorkspaceStore::for_project(&absolute(root)).ok()?;
    History::open(&store).ok()
}

/// The project key is the canonical path, so a relative root has to be made
/// absolute before it is used as one.
///
/// Not defensive tidying — this was a real bug, found by running it. A run
/// invoked as `hick run note.hick` has `doc_path.parent() == ""`, which
/// hashes to the key for the empty string, so every act landed in a store
/// nothing would ever look in while `hick history` read the one for the
/// working directory and reported an empty folder. Two stores, no error, and
/// a feature that silently did nothing.
fn absolute(root: &Path) -> PathBuf {
    if root.is_absolute() {
        return root.to_path_buf();
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if root.as_os_str().is_empty() {
        cwd
    } else {
        cwd.join(root)
    }
}

/// Record what `writes` is about to do to `root`, reading each file's current
/// bytes as the *before*.
///
/// Call immediately before writing. Returns the act when one was worth
/// keeping — an act whose files all end up with the bytes they already had is
/// not a stop anybody wants to scroll past.
pub fn record(
    root: &Path,
    kind: ActKind,
    detail: Option<String>,
    writes: &[(PathBuf, Vec<u8>)],
) -> Option<Act> {
    let history = open(root)?;
    let changes: Vec<PendingChange> = writes
        .iter()
        .map(|(path, after)| {
            (
                relative(&absolute(root), path),
                std::fs::read(path).ok(),
                Some(after.clone()),
            )
        })
        .collect();
    let act = match history.record(root, kind, detail, &now(), &changes) {
        Ok(act) => act,
        Err(error) => {
            // Never fatal: a run must not fail because a cache on this
            // machine could not be written.
            log::debug!("local history not recorded: {error:#}");
            return None;
        }
    };
    if act.is_some()
        && let Err(error) = history.prune(retention(), &now())
    {
        log::debug!("local history not pruned: {error:#}");
    }
    act
}

/// Record explicit before/after bytes, for a writer whose "before" is not
/// simply what is on disk at the path being written.
///
/// The merge driver is the case: git hands it a temp file to write the result
/// into, so the bytes that matter are the ones it was given rather than the
/// ones at the repo-relative path.
pub fn record_changes(
    root: &Path,
    kind: ActKind,
    detail: Option<String>,
    changes: &[PendingChange],
) -> Option<Act> {
    let history = open(root)?;
    match history.record(root, kind, detail, &now(), changes) {
        Ok(act) => act,
        Err(error) => {
            log::debug!("local history not recorded: {error:#}");
            None
        }
    }
}

/// Record a set of files that are about to be deleted.
pub fn record_removals(
    root: &Path,
    kind: ActKind,
    detail: Option<String>,
    paths: &[PathBuf],
) -> Option<Act> {
    let history = open(root)?;
    let changes: Vec<PendingChange> = paths
        .iter()
        .map(|path| {
            (
                relative(&absolute(root), path),
                std::fs::read(path).ok(),
                None,
            )
        })
        .collect();
    history.record(root, kind, detail, &now(), &changes).ok()?
}

/// Project-relative, with `/` separators.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn now() -> String {
    crate::serve::now_rfc3339_public()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_retention_value_warns_and_keeps_the_default() {
        // Silently halving somebody's history because they typed `7d` is
        // worse than telling them.
        unsafe { std::env::set_var("HICKORY_HISTORY_DAYS", "7d") };
        assert_eq!(retention().max_days, Retention::default().max_days);
        unsafe { std::env::set_var("HICKORY_HISTORY_DAYS", "7") };
        assert_eq!(retention().max_days, 7);
        unsafe { std::env::remove_var("HICKORY_HISTORY_DAYS") };
    }

    #[test]
    fn a_relative_root_resolves_to_the_folder_a_person_is_actually_in() {
        // Found by running it: `hick run note.hick` gives a parent of "",
        // which hashed to the key for the empty string. Every act landed in a
        // store nothing would ever look in, while `hick history` read the one
        // for the working directory and reported an empty folder. Two stores,
        // no error, and a feature that silently did nothing.
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(absolute(Path::new("")), cwd);
        assert_eq!(absolute(Path::new("sub")), cwd.join("sub"));
        assert_eq!(absolute(Path::new("/tmp")), PathBuf::from("/tmp"));
    }

    #[test]
    fn a_path_outside_the_root_keeps_its_own_name_rather_than_being_mangled() {
        let rel = relative(Path::new("/a/b"), Path::new("/a/b/c/d.hick"));
        assert_eq!(rel, "c/d.hick");
        // Not every writer's path is under the root (an out-dir run), and a
        // wrong relative path would revert to the wrong place.
        let outside = relative(Path::new("/a/b"), Path::new("/elsewhere/d.hick"));
        assert_eq!(outside, "/elsewhere/d.hick");
    }
}
