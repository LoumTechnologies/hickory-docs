//! Where a document's code is staged on disk for the child language servers.
//!
//! A child language server is a real program reading real files: pyright and
//! rust-analyzer both open the path in the URI they are handed, and a project
//! -aware one walks upwards from it looking for a manifest. So the code inside
//! a `.hick` document has to exist somewhere on disk before any of them can
//! answer a question about it. That somewhere is a **staging directory**, one
//! per running server, with one subdirectory per open document.
//!
//! Two things about the old spelling of this — a hardcoded
//! `file:///tmp/hick-lsp-vfiles/<hash>/…` — were wrong, and they are separate
//! faults that happen to live on one line:
//!
//! * **It does not exist on Windows.** `Url::to_file_path` refuses a rooted
//!   path that names no drive, so the write it guarded never happened, no
//!   virtual file was ever staged, and every child was asked about files that
//!   were not there. Nothing failed; everything came back empty.
//! * **It is shared, on Unix.** `/tmp` is world-writable and the directory
//!   name was derived from the document URI by a hash anyone can compute, so
//!   two users on one machine collide, and on a shared host a third party can
//!   pre-create `/tmp/hick-lsp-vfiles/<hash>` and read — or replace — the code
//!   of a document they cannot open.
//!
//! [`std::env::temp_dir`] answers the first (it is `%TEMP%`, per-user, on
//! Windows and honours `TMPDIR` on Unix). A fresh randomly-named directory
//! inside it, created `0700`, answers the second: a name nobody can predict
//! and a directory nobody else can enter.
//!
//! ## Lifetime
//!
//! The staging directory belongs to the [`crate::HickBackend`] that made it
//! and is deleted when that backend drops — which is when the stdio server's
//! `serve` returns, or when the desktop app's WebSocket session ends. A run
//! that is *killed* leaves its directory behind; that is the honest cost of a
//! process that gets no chance to run any code. It is left rather than swept
//! by the next run, because a directory belonging to a live sibling session —
//! a second editor window, the app and a terminal `hick-lsp` at once — is
//! indistinguishable from a dead one's, and deleting a live one would break
//! language intelligence in a window nobody was touching. What is left is
//! under the temp directory the operating system already sweeps, and its name
//! is never reused.

use std::path::{Component, Path, PathBuf};

use tower_lsp::lsp_types::Url;

/// The name every staging directory starts with.
///
/// Public because it is a contract, not an implementation detail: the desktop
/// app's LSP bridge has to recognise a staged file coming back from a child
/// and name it by the output path the document gave it, and it recognises one
/// by this prefix. See [`staged_output_path`].
pub const STAGING_PREFIX: &str = "hick-lsp-vfiles-";

/// The environment variable a user would set to move the temp directory.
///
/// Named in the error text, because "somewhere under your temp directory"
/// is not something a user can act on and `TMPDIR=/some/path` is.
const TEMP_VAR: &str = if cfg!(windows) { "TEMP" } else { "TMPDIR" };

/// Why a document's code could not be staged for a language server.
///
/// Every variant says what failed, where, and what the user can do — these
/// are read by a person whose editor has just stopped answering, and the
/// whole point of the issue this type exists for is that they were previously
/// told nothing at all.
#[derive(Debug, Clone, thiserror::Error)]
pub enum StagingError {
    /// There is no staging directory for this run at all.
    #[error(
        "hick-lsp could not create a directory to stage code in, under the temporary \
         directory {temp}: {reason}.\n\
         Language intelligence (hover, completion, go to definition) will be empty until \
         this works, because a language server can only read code that exists as a file.\n\
         Check that {temp} exists and is writable, or set {var} to a directory you can write to."
    )]
    Unavailable {
        /// The temp directory that was tried.
        temp: String,
        /// The operating system's reason.
        reason: String,
        /// `TMPDIR`, or `TEMP` on Windows.
        var: &'static str,
    },

    /// A file could not be written into the staging directory.
    #[error(
        "hick-lsp could not write {path}, the staged copy of {block} that the language \
         server reads: {reason}.\n\
         Answers about that block will be empty until this works.\n\
         Check the free space and permissions on {path}, or set {var} to a directory you \
         can write to."
    )]
    Write {
        /// The staged path that could not be written.
        path: String,
        /// The `path=` of the `hick:file` block it came from.
        block: String,
        /// The operating system's reason.
        reason: String,
        /// `TMPDIR`, or `TEMP` on Windows.
        var: &'static str,
    },

    /// The document names an output path that would escape the staging area.
    #[error(
        "hick-lsp refused to stage the block <hick:file path=\"{block}\">: that path leaves \
         the directory the document's code is staged in, so writing it would touch a file \
         outside this document.\n\
         Give the block a path relative to the document, with no `..` in it."
    )]
    UnsafePath {
        /// The `path=` of the offending `hick:file` block.
        block: String,
    },
}

/// One running server's staging directory.
///
/// Created once per [`crate::HickBackend`]; deleted when it drops.
pub struct StagingArea {
    /// The directory, or the reason there is not one.
    ///
    /// A failure is kept rather than retried per write: the reason a temp
    /// directory cannot be created is a property of the machine, so retrying
    /// on every keystroke would produce one identical error per keystroke.
    area: Result<Area, StagingError>,
}

struct Area {
    /// Deletes the directory on drop. Held for that alone.
    _dir: tempfile::TempDir,
    /// The same path, canonicalised.
    root: PathBuf,
}

impl Default for StagingArea {
    fn default() -> Self {
        Self::new()
    }
}

impl StagingArea {
    /// Make this run's staging directory.
    ///
    /// Never fails: a machine with no writable temp directory still gets a
    /// working parser, structural navigation and diagnostics from `hick-lsp`
    /// itself — it is only the child servers that go quiet — so the failure is
    /// recorded and reported to the editor rather than taking the server down.
    /// `docs/guarantees/editor-intelligence/lsp-channel-degrades-never-errors.md`
    pub fn new() -> Self {
        let temp = std::env::temp_dir();
        let area = tempfile::Builder::new()
            .prefix(STAGING_PREFIX)
            .tempdir()
            .map(|dir| {
                let root = canonical(dir.path());
                Area { _dir: dir, root }
            })
            .map_err(|error| StagingError::Unavailable {
                temp: temp.display().to_string(),
                reason: error.to_string(),
                var: TEMP_VAR,
            });
        Self { area }
    }

    /// The directory this document's code is staged in.
    pub fn document_dir(&self, hick_uri: &Url) -> Result<PathBuf, StagingError> {
        let area = self.area.as_ref().map_err(StagingError::clone)?;
        Ok(area.root.join(document_key(hick_uri)))
    }

    /// The root a child language server is initialised against.
    ///
    /// The document's own staging directory rather than the project it lives
    /// in, so a project-aware child answers about the document's code and does
    /// not go discovering unrelated crates and packages beside the `.hick`
    /// file. A directory URL, so it ends in a slash the way a root URI does.
    pub fn document_root_uri(&self, hick_uri: &Url) -> Result<String, StagingError> {
        let dir = self.document_dir(hick_uri)?;
        // `from_directory_path` only fails for a relative path, and the temp
        // directory is absolute on every platform.
        Ok(Url::from_directory_path(&dir)
            .map(|url| url.to_string())
            .unwrap_or_else(|()| format!("{}/", dir.display())))
    }

    /// Where a `hick:file` block's staged copy lives.
    pub fn vfile_path(&self, hick_uri: &Url, block_path: &str) -> Result<PathBuf, StagingError> {
        let dir = self.document_dir(hick_uri)?;
        join_within(&dir, block_path).ok_or_else(|| StagingError::UnsafePath {
            block: block_path.to_string(),
        })
    }

    /// The URI a child language server knows a block's code by.
    ///
    /// Total, because it is also the key this server maps answers back
    /// through: a block whose code could not be staged still needs a stable
    /// name, and the document's own URI is the one name that always exists.
    /// Nothing is written under it — the write reports its own failure.
    pub fn vfile_uri(&self, hick_uri: &Url, block_path: &str) -> Url {
        self.vfile_path(hick_uri, block_path)
            .ok()
            .and_then(|path| Url::from_file_path(path).ok())
            .unwrap_or_else(|| hick_uri.clone())
    }

    /// Stage one block's code, creating the directories it needs.
    pub fn write_vfile(
        &self,
        hick_uri: &Url,
        block_path: &str,
        content: &str,
    ) -> Result<(), StagingError> {
        let path = self.vfile_path(hick_uri, block_path)?;
        let write_error = |error: std::io::Error| StagingError::Write {
            path: path.display().to_string(),
            block: block_path.to_string(),
            reason: error.to_string(),
            var: TEMP_VAR,
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(write_error)?;
        }
        std::fs::write(&path, content).map_err(write_error)
    }

    /// Copy one of the document's project manifests in beside the staged code.
    ///
    /// Separate from [`Self::write_vfile`] because it is genuinely
    /// best-effort: a manifest that cannot be copied leaves the child exactly
    /// as badly off as it was before, which is a weaker answer rather than a
    /// broken one.
    pub fn copy_into(&self, hick_uri: &Url, source: &Path, name: &str) -> std::io::Result<()> {
        let dir = self
            .document_dir(hick_uri)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        std::fs::create_dir_all(&dir)?;
        std::fs::copy(source, dir.join(name)).map(|_| ())
    }

    /// Forget a document's staged code, when its editor closes it.
    ///
    /// Best-effort: the whole staging directory goes when this run ends, so a
    /// failure here costs disk until then and nothing else.
    pub fn forget(&self, hick_uri: &Url) {
        if let Ok(dir) = self.document_dir(hick_uri) {
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    /// The reason nothing can be staged this run, if that is the situation.
    pub fn unavailable(&self) -> Option<&StagingError> {
        self.area.as_ref().err()
    }
}

/// If `path` is a staged file, the output path the document gave that block.
///
/// `…/hick-lsp-vfiles-XXXX/<document>/src/stats.py` → `src/stats.py`. The
/// desktop app's LSP bridge uses this to name a location a child returned:
/// those files are not files the user has, so they are shown as
/// `hick-output:///src/stats.py` rather than as a path under a temp directory
/// they never made. Matching on the directory *name* rather than on the
/// staging area's own path keeps it working when a child answers with a path
/// that resolved a symlink on the way (macOS's `/var` → `/private/var`).
pub fn staged_output_path(path: &Path) -> Option<String> {
    let mut components = path
        .components()
        .skip_while(|component| !is_staging_dir(component));
    // The staging directory, then the per-document one.
    components.next()?;
    components.next()?;
    let rest: Vec<String> = components
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    (!rest.is_empty()).then(|| rest.join("/"))
}

fn is_staging_dir(component: &Component<'_>) -> bool {
    matches!(component, Component::Normal(name)
        if name.to_string_lossy().starts_with(STAGING_PREFIX))
}

/// A directory name for a document, stable for as long as it is open.
fn document_key(hick_uri: &Url) -> String {
    let mut hash: u64 = 5381;
    for byte in hick_uri.as_str().bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    format!("{hash:x}")
}

/// Join a document-supplied relative path, or refuse it.
///
/// `path=` on a `hick:file` block is the document's text, and a document can
/// come from anywhere — a repository someone cloned, a note someone was sent.
/// `..`, an absolute path, and a Windows drive or UNC prefix are all refused,
/// because each of them turns "write inside this document's staging
/// directory" into "write wherever this string says".
fn join_within(root: &Path, relative: &str) -> Option<PathBuf> {
    // A leading separator has to be refused BEFORE the loop, not skipped by
    // it. `"/etc/passwd".split('/')` is `["", "etc", "passwd"]`, and treating
    // that empty first segment as "nothing to do" silently reinterprets a
    // rooted path as a relative one: the document asked for `/etc/passwd` and
    // would get `<staging>/etc/passwd`, a different file, staged without
    // complaint. It does not escape the staging area, which is why this is a
    // wrong-answer bug rather than a hole — but a block whose path cannot mean
    // what it says should be refused rather than quietly redirected.
    if relative.starts_with('/') || relative.starts_with('\\') {
        return None;
    }
    let mut out = root.to_path_buf();
    for part in relative.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        match Path::new(part).components().next() {
            Some(Component::Normal(name)) => out.push(name),
            // `..`, `/`, `C:` — anything that is not a plain name.
            _ => return None,
        }
    }
    (out != root).then_some(out)
}

/// The path as a child language server will report it back.
///
/// macOS's temp directory is `/var/folders/…`, and `/var` is a symlink to
/// `/private/var`; a server that resolves it answers with the resolved form,
/// which would then match nothing we had recorded. Canonicalising once here
/// means both sides use the resolved spelling.
///
/// Windows' canonical form is a `\\?\C:\…` verbatim path, which is not a
/// spelling `Url` handles, so the prefix comes back off.
fn canonical(path: &Path) -> PathBuf {
    let Ok(resolved) = std::fs::canonicalize(path) else {
        return path.to_path_buf();
    };
    if cfg!(windows) {
        let text = resolved.to_string_lossy();
        if let Some(stripped) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc_uri() -> Url {
        Url::parse("file:///home/u/project/notes.hick").unwrap()
    }

    #[test]
    fn staging_lives_under_the_users_own_temp_directory() {
        // Not `/tmp`: it does not exist on Windows, and where it does it is
        // shared with every other account on the machine.
        let area = StagingArea::new();
        let dir = area.document_dir(&doc_uri()).unwrap();
        assert!(
            dir.starts_with(canonical(&std::env::temp_dir())),
            "{} is not under {}",
            dir.display(),
            std::env::temp_dir().display()
        );
    }

    #[test]
    fn two_runs_never_share_a_directory() {
        // The old layout hashed the document URI into a fixed path, so two
        // users staging the same document collided — and on a shared machine
        // either could read or replace the other's code.
        let one = StagingArea::new();
        let two = StagingArea::new();
        assert_ne!(
            one.document_dir(&doc_uri()).unwrap(),
            two.document_dir(&doc_uri()).unwrap()
        );
    }

    #[test]
    fn a_staged_file_is_written_and_reads_back() {
        let area = StagingArea::new();
        area.write_vfile(&doc_uri(), "src/stats.py", "x = 1\n")
            .expect("a staged file is written");
        let path = area.vfile_path(&doc_uri(), "src/stats.py").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "x = 1\n");
    }

    #[test]
    fn the_uri_a_child_is_given_names_the_file_that_was_written() {
        // The bug this whole module exists for: the URI said one thing and the
        // filesystem had another, on Windows nothing at all.
        let area = StagingArea::new();
        area.write_vfile(&doc_uri(), "src/stats.py", "x = 1\n")
            .unwrap();
        let uri = area.vfile_uri(&doc_uri(), "src/stats.py");
        let path = uri.to_file_path().expect("the URI is a usable file path");
        assert!(path.is_file(), "{} was not written", path.display());
    }

    #[test]
    fn a_path_that_escapes_the_document_is_refused() {
        let area = StagingArea::new();
        for escape in ["../outside.py", "a/../../outside.py", "/etc/passwd"] {
            assert!(
                matches!(
                    area.write_vfile(&doc_uri(), escape, "danger"),
                    Err(StagingError::UnsafePath { .. })
                ),
                "{escape} was not refused"
            );
        }
    }

    #[test]
    fn forgetting_a_document_removes_its_code() {
        let area = StagingArea::new();
        area.write_vfile(&doc_uri(), "src/stats.py", "x = 1\n")
            .unwrap();
        area.forget(&doc_uri());
        assert!(!area.document_dir(&doc_uri()).unwrap().exists());
    }

    #[test]
    fn a_staged_path_is_recognised_by_its_output_path() {
        assert_eq!(
            staged_output_path(Path::new("/tmp/hick-lsp-vfiles-ab12/9f2a/src/stats.py")).as_deref(),
            Some("src/stats.py")
        );
        // A real one, wherever this machine's temp directory is.
        let area = StagingArea::new();
        let path = area.vfile_path(&doc_uri(), "src/stats.py").unwrap();
        assert_eq!(staged_output_path(&path).as_deref(), Some("src/stats.py"));
    }

    #[test]
    fn a_path_outside_the_staging_area_is_not_a_staged_file() {
        assert_eq!(
            staged_output_path(Path::new("/usr/lib/python3/typing.py")),
            None
        );
        // The staging directory itself, and a document's directory, are not
        // files a document named.
        assert_eq!(
            staged_output_path(Path::new("/tmp/hick-lsp-vfiles-ab12/9f2a")),
            None
        );
    }

    #[test]
    fn the_error_says_what_to_do_about_it() {
        let error = StagingError::Unavailable {
            temp: "/nowhere".into(),
            reason: "Permission denied (os error 13)".into(),
            var: TEMP_VAR,
        };
        let text = error.to_string();
        assert!(text.contains("/nowhere"), "{text}");
        assert!(text.contains("Permission denied"), "{text}");
        assert!(text.contains(TEMP_VAR), "{text}");
    }
}
