<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The LSP Channel Degrades To Structural Or Null Answers, Never Errors

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a doc WebSocket with the 0x02 LSP bridge (api.md v0.3), when a child
language server (pyright, rust-analyzer, …) or the hick-lsp binary itself is
unavailable, then every client request still receives a JSON-RPC response
with a `result` (structural copy/paste answers where the parser can provide
them, `null` otherwise) — never an `error` frame, a dropped request, or a
closed channel. Definition/reference targets inside generated outputs are
translated through the last run's provenance to `hick:///` source
coordinates when byte-mappable, else returned as `hick-output:///` targets.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/lsp-channel-degrades-never-errors.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-76f75a35bcb3"></a>
## `crates/hick-lsp/src/staging.rs`

Related contracts: [The LSP Channel Degrades To Structural Or Null Answers, Never Errors](contract.md).

<a id="f-76f75a35bcb3-STAGING_PREFIX"></a>
### `STAGING_PREFIX`

Where a document's code is staged on disk for the child language servers.
A child language server is a real program reading real files: pyright and
rust-analyzer both open the path in the URI they are handed, and a project
-aware one walks upwards from it looking for a manifest. So the code inside
a `.md` document has to exist somewhere on disk before any of them can
answer a question about it. That somewhere is a **staging directory**, one
per running server, with one subdirectory per open document.
Two things about the old spelling of this — a hardcoded
`file:///tmp/hick-lsp-vfiles/<hash>/…` — were wrong, and they are separate
faults that happen to live on one line:
* **It does not exist on Windows.** `Url::to_file_path` refuses a rooted
path that names no drive, so the write it guarded never happened, no
virtual file was ever staged, and every child was asked about files that
were not there. Nothing failed; everything came back empty.
* **It is shared, on Unix.** `/tmp` is world-writable and the directory
name was derived from the document URI by a hash anyone can compute, so
two users on one machine collide, and on a shared host a third party can
pre-create `/tmp/hick-lsp-vfiles/<hash>` and read — or replace — the code
of a document they cannot open.
[`std::env::temp_dir`] answers the first (it is `%TEMP%`, per-user, on
Windows and honours `TMPDIR` on Unix). A fresh randomly-named directory
inside it, created `0700`, answers the second: a name nobody can predict
and a directory nobody else can enter.
## Lifetime
The staging directory belongs to the [`crate::HickBackend`] that made it
and is deleted when that backend drops — which is when the stdio server's
`serve` returns, or when the desktop app's WebSocket session ends. A run
that is *killed* leaves its directory behind; that is the honest cost of a
process that gets no chance to run any code. It is left rather than swept
by the next run, because a directory belonging to a live sibling session —
a second editor window, the app and a terminal `hick-lsp` at once — is
indistinguishable from a dead one's, and deleting a live one would break
language intelligence in a window nobody was touching. What is left is
under the temp directory the operating system already sweeps, and its name
is never reused.

<storycode:copy id="f-76f75a35bcb3-STAGING_PREFIX">
//! Where a document's code is staged on disk for the child language servers.
//!
//! A child language server is a real program reading real files: pyright and
//! rust-analyzer both open the path in the URI they are handed, and a project
//! -aware one walks upwards from it looking for a manifest. So the code inside
//! a `.md` document has to exist somewhere on disk before any of them can
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

</storycode:copy>
<a id="f-76f75a35bcb3-TEMP_VAR"></a>
### `TEMP_VAR`

The environment variable a user would set to move the temp directory.
Named in the error text, because "somewhere under your temp directory"
is not something a user can act on and `TMPDIR=/some/path` is.

<storycode:copy id="f-76f75a35bcb3-TEMP_VAR">
/// The environment variable a user would set to move the temp directory.
///
/// Named in the error text, because "somewhere under your temp directory"
/// is not something a user can act on and `TMPDIR=/some/path` is.
const TEMP_VAR: &str = if cfg!(windows) { "TEMP" } else { "TMPDIR" };

</storycode:copy>
<a id="f-76f75a35bcb3-StagingError"></a>
### `StagingError`

Why a document's code could not be staged for a language server.
Every variant says what failed, where, and what the user can do — these
are read by a person whose editor has just stopped answering, and the
whole point of the issue this type exists for is that they were previously
told nothing at all.

<storycode:copy id="f-76f75a35bcb3-StagingError">
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

</storycode:copy>
<a id="f-76f75a35bcb3-StagingArea"></a>
### `StagingArea`

One running server's staging directory.
Created once per [`crate::HickBackend`]; deleted when it drops.

<storycode:copy id="f-76f75a35bcb3-StagingArea">
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

</storycode:copy>
<a id="f-76f75a35bcb3-Area"></a>
### `Area`

<storycode:copy id="f-76f75a35bcb3-Area">
struct Area {
    /// Deletes the directory on drop. Held for that alone.
    _dir: tempfile::TempDir,
    /// The same path, canonicalised.
    root: PathBuf,
}

</storycode:copy>
<a id="f-76f75a35bcb3-impl-Default-for-StagingArea"></a>
### `impl-Default for StagingArea`

<storycode:copy id="f-76f75a35bcb3-impl-Default-for-StagingArea">
impl Default for StagingArea {
    fn default() -> Self {
        Self::new()
    }
}

</storycode:copy>
<a id="f-76f75a35bcb3-impl-StagingArea"></a>
### `impl-StagingArea`

<storycode:copy id="f-76f75a35bcb3-impl-StagingArea">
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
    /// not go discovering unrelated crates and packages beside the `.md`
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

</storycode:copy>
<a id="f-76f75a35bcb3-staged_output_path"></a>
### `staged_output_path`

If `path` is a staged file, the output path the document gave that block.
`…/hick-lsp-vfiles-XXXX/<document>/src/stats.py` → `src/stats.py`. The
desktop app's LSP bridge uses this to name a location a child returned:
those files are not files the user has, so they are shown as
`hick-output:///src/stats.py` rather than as a path under a temp directory
they never made. Matching on the directory *name* rather than on the
staging area's own path keeps it working when a child answers with a path
that resolved a symlink on the way (macOS's `/var` → `/private/var`).

<storycode:copy id="f-76f75a35bcb3-staged_output_path">
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

</storycode:copy>
<a id="f-76f75a35bcb3-is_staging_dir"></a>
### `is_staging_dir`

<storycode:copy id="f-76f75a35bcb3-is_staging_dir">
fn is_staging_dir(component: &Component<'_>) -> bool {
    matches!(component, Component::Normal(name)
        if name.to_string_lossy().starts_with(STAGING_PREFIX))
}

</storycode:copy>
<a id="f-76f75a35bcb3-document_key"></a>
### `document_key`

A directory name for a document, stable for as long as it is open.

<storycode:copy id="f-76f75a35bcb3-document_key">
/// A directory name for a document, stable for as long as it is open.
fn document_key(hick_uri: &Url) -> String {
    let mut hash: u64 = 5381;
    for byte in hick_uri.as_str().bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    format!("{hash:x}")
}

</storycode:copy>
<a id="f-76f75a35bcb3-join_within"></a>
### `join_within`

Join a document-supplied relative path, or refuse it.
`path=` on a `hick:file` block is the document's text, and a document can
come from anywhere — a repository someone cloned, a note someone was sent.
`..`, an absolute path, and a Windows drive or UNC prefix are all refused,
because each of them turns "write inside this document's staging
directory" into "write wherever this string says".

<storycode:copy id="f-76f75a35bcb3-join_within">
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

</storycode:copy>
<a id="f-76f75a35bcb3-canonical"></a>
### `canonical`

The path as a child language server will report it back.
macOS's temp directory is `/var/folders/…`, and `/var` is a symlink to
`/private/var`; a server that resolves it answers with the resolved form,
which would then match nothing we had recorded. Canonicalising once here
means both sides use the resolved spelling.
Windows' canonical form is a `\\?\C:\…` verbatim path, which is not a
spelling `Url` handles, so the prefix comes back off.

<storycode:copy id="f-76f75a35bcb3-canonical">
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

</storycode:copy>
<a id="f-76f75a35bcb3-tests"></a>
### `tests`

<storycode:copy id="f-76f75a35bcb3-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn doc_uri() -> Url {
        Url::parse("file:///home/u/project/notes.md").unwrap()
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
</storycode:copy>

### Assemble `crates/hick-lsp/src/staging.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/staging.rs"><storycode:paste select="#f-76f75a35bcb3-STAGING_PREFIX"/><storycode:paste select="#f-76f75a35bcb3-TEMP_VAR"/><storycode:paste select="#f-76f75a35bcb3-StagingError"/><storycode:paste select="#f-76f75a35bcb3-StagingArea"/><storycode:paste select="#f-76f75a35bcb3-Area"/><storycode:paste select="#f-76f75a35bcb3-impl-Default-for-StagingArea"/><storycode:paste select="#f-76f75a35bcb3-impl-StagingArea"/><storycode:paste select="#f-76f75a35bcb3-staged_output_path"/><storycode:paste select="#f-76f75a35bcb3-is_staging_dir"/><storycode:paste select="#f-76f75a35bcb3-document_key"/><storycode:paste select="#f-76f75a35bcb3-join_within"/><storycode:paste select="#f-76f75a35bcb3-canonical"/><storycode:paste select="#f-76f75a35bcb3-tests"/></storycode:file>

<a id="output-207b83769ac7"></a>
## `crates/hick-lsp/src/structural.rs`

Related contracts: [The LSP Channel Degrades To Structural Or Null Answers, Never Errors](contract.md).

<a id="f-207b83769ac7-position_to_byte"></a>
### `position_to_byte`

Structural (parser-level) navigation over a hick document.
These answers need no child language server: `hick:copy id="x"` defines a
named block, `hick:paste select="#x"` (or `hick:copy select="#x"`) weaves
it. Definition at a paste site jumps to the copy definition; references at
either end list every paste site (optionally plus the definition).
Also home to the LSP-position ↔ byte-offset helpers used by the proxy
handlers (LSP positions count UTF-16 code units per line).

<storycode:copy id="f-207b83769ac7-position_to_byte">
//! Structural (parser-level) navigation over a hick document.
//!
//! These answers need no child language server: `hick:copy id="x"` defines a
//! named block, `hick:paste select="#x"` (or `hick:copy select="#x"`) weaves
//! it. Definition at a paste site jumps to the copy definition; references at
//! either end list every paste site (optionally plus the definition).
//!
//! Also home to the LSP-position ↔ byte-offset helpers used by the proxy
//! handlers (LSP positions count UTF-16 code units per line).

use hick_lang::{HickDocument, HickNode, HickTag};

/// Convert an LSP position (0-based line, UTF-16 character) to a byte offset.
pub fn position_to_byte(source: &str, line: u32, character: u32) -> Option<usize> {
    let mut current_line = 0u32;
    let mut line_start = 0usize;
    if line > 0 {
        let mut found = false;
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                current_line += 1;
                if current_line == line {
                    line_start = i + 1;
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return None;
        }
    }
    let rest = &source[line_start..];
    let line_end = rest
        .find('\n')
        .map(|i| line_start + i)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];

    let mut units = 0u32;
    for (byte_idx, ch) in line_text.char_indices() {
        if units >= character {
            return Some(line_start + byte_idx);
        }
        units += ch.len_utf16() as u32;
    }
    // Position at (or past) end of line clamps to line end.
    Some(line_end)
}

</storycode:copy>
<a id="f-207b83769ac7-byte_to_position"></a>
### `byte_to_position`

Convert a byte offset to an LSP position (0-based line, UTF-16 character).
Offsets past the end clamp to the final position.

<storycode:copy id="f-207b83769ac7-byte_to_position">
/// Convert a byte offset to an LSP position (0-based line, UTF-16 character).
/// Offsets past the end clamp to the final position.
pub fn byte_to_position(source: &str, offset: usize) -> (u32, u32) {
    let offset = offset.min(source.len());
    let before = &source[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32;
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character: u32 = source[line_start..offset]
        .chars()
        .map(|c| c.len_utf16() as u32)
        .sum();
    (line, character)
}

</storycode:copy>
<a id="f-207b83769ac7-Span"></a>
### `Span`

<storycode:copy id="f-207b83769ac7-Span">
/// A byte span in the .md source.
pub type Span = (usize, usize);

</storycode:copy>
<a id="f-207b83769ac7-CopyDef"></a>
### `CopyDef`

<storycode:copy id="f-207b83769ac7-CopyDef">
#[derive(Debug, Clone)]
struct CopyDef {
    id: String,
    /// Span of the opening `<hick:copy …>` tag (the definition location).
    tag_span: Span,
    /// Extent of the whole element (opening tag through last child span).
    extent: Span,
}

</storycode:copy>
<a id="f-207b83769ac7-PasteSite"></a>
### `PasteSite`

<storycode:copy id="f-207b83769ac7-PasteSite">
#[derive(Debug, Clone)]
struct PasteSite {
    id: String,
    /// Span of the `<hick:paste select="#id" …>` tag.
    span: Span,
}

</storycode:copy>
<a id="f-207b83769ac7-CopyGraph"></a>
### `CopyGraph`

<storycode:copy id="f-207b83769ac7-CopyGraph">
#[derive(Debug, Default)]
struct CopyGraph {
    defs: Vec<CopyDef>,
    sites: Vec<PasteSite>,
}

</storycode:copy>
<a id="f-207b83769ac7-max_descendant_end"></a>
### `max_descendant_end`

<storycode:copy id="f-207b83769ac7-max_descendant_end">
fn max_descendant_end(tag: &HickTag, mut end: usize) -> usize {
    for child in &tag.children {
        match child {
            HickNode::Text(_, Some(span)) => end = end.max(span.end),
            HickNode::Text(_, None) => {}
            HickNode::Tag(t) => {
                if let Some(span) = t.source_span {
                    end = end.max(span.end);
                }
                end = max_descendant_end(t, end);
            }
        }
    }
    end
}

</storycode:copy>
<a id="f-207b83769ac7-walk"></a>
### `walk`

<storycode:copy id="f-207b83769ac7-walk">
fn walk(nodes: &[HickNode], graph: &mut CopyGraph) {
    for node in nodes {
        let HickNode::Tag(tag) = node else { continue };
        let span = tag.source_span.map(|s| (s.start, s.end));
        if tag.name == "copy" || tag.name == "paste" {
            if let Some(select) = tag.get_attribute("select") {
                if let Some(span) = span {
                    graph.sites.push(PasteSite {
                        id: select.strip_prefix('#').unwrap_or(select).to_string(),
                        span,
                    });
                }
            } else if tag.name == "copy"
                && let Some(id) = tag.get_attribute("id")
                && let Some(span) = span
            {
                graph.defs.push(CopyDef {
                    id: id.to_string(),
                    tag_span: span,
                    extent: (span.0, max_descendant_end(tag, span.1)),
                });
            }
        }
        walk(&tag.children, graph);
    }
}

</storycode:copy>
<a id="f-207b83769ac7-build_graph"></a>
### `build_graph`

<storycode:copy id="f-207b83769ac7-build_graph">
fn build_graph(doc: &HickDocument) -> CopyGraph {
    let mut graph = CopyGraph::default();
    walk(&doc.nodes, &mut graph);
    graph
}

</storycode:copy>
<a id="f-207b83769ac7-id_at"></a>
### `id_at`

Which copy id is under the cursor (a paste site tag, a copy definition tag,
or anywhere inside a copy definition's body)?

<storycode:copy id="f-207b83769ac7-id_at">
/// Which copy id is under the cursor (a paste site tag, a copy definition tag,
/// or anywhere inside a copy definition's body)?
fn id_at(graph: &CopyGraph, offset: usize) -> Option<String> {
    if let Some(site) = graph
        .sites
        .iter()
        .find(|s| s.span.0 <= offset && offset < s.span.1)
    {
        return Some(site.id.clone());
    }
    graph
        .defs
        .iter()
        .find(|d| d.extent.0 <= offset && offset < d.extent.1)
        .map(|d| d.id.clone())
}

</storycode:copy>
<a id="f-207b83769ac7-definition"></a>
### `definition`

Structural go-to-definition: the copy definition's opening-tag span for the
copy id under the cursor.

<storycode:copy id="f-207b83769ac7-definition">
/// Structural go-to-definition: the copy definition's opening-tag span for the
/// copy id under the cursor.
pub fn definition(doc: &HickDocument, offset: usize) -> Option<Span> {
    let graph = build_graph(doc);
    let id = id_at(&graph, offset)?;
    graph.defs.iter().find(|d| d.id == id).map(|d| d.tag_span)
}

</storycode:copy>
<a id="f-207b83769ac7-references"></a>
### `references`

Structural references: every paste site weaving the copy id under the
cursor, plus (when `include_declaration`) the definition's opening tag.

<storycode:copy id="f-207b83769ac7-references">
/// Structural references: every paste site weaving the copy id under the
/// cursor, plus (when `include_declaration`) the definition's opening tag.
pub fn references(doc: &HickDocument, offset: usize, include_declaration: bool) -> Vec<Span> {
    let graph = build_graph(doc);
    let Some(id) = id_at(&graph, offset) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if include_declaration && let Some(def) = graph.defs.iter().find(|d| d.id == id) {
        out.push(def.tag_span);
    }
    out.extend(graph.sites.iter().filter(|s| s.id == id).map(|s| s.span));
    out.sort_unstable();
    out.dedup();
    out
}

</storycode:copy>
<a id="f-207b83769ac7-tests"></a>
### `tests`

<storycode:copy id="f-207b83769ac7-tests">
#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="imports">
use std::io;
</hick:copy>
<hick:file path="main.rs">
<hick:paste select="#imports" />
fn main() {}
</hick:file>
<hick:file path="lib.rs">
<hick:paste select="#imports" />
</hick:file>
</hick:doc>"##;

    fn parse() -> HickDocument {
        hick_lang::parse(DOC).unwrap()
    }

    #[test]
    fn position_byte_round_trip_ascii() {
        let src = "abc\ndef\nghi";
        assert_eq!(position_to_byte(src, 0, 0), Some(0));
        assert_eq!(position_to_byte(src, 1, 2), Some(6));
        assert_eq!(byte_to_position(src, 6), (1, 2));
        assert_eq!(byte_to_position(src, 0), (0, 0));
    }

    #[test]
    fn position_byte_utf16_units() {
        // '€' is 3 UTF-8 bytes, 1 UTF-16 unit; '𝄞' is 4 UTF-8 bytes, 2 units.
        let src = "€x\n𝄞y";
        assert_eq!(position_to_byte(src, 0, 1), Some(3)); // after €
        assert_eq!(position_to_byte(src, 1, 2), Some(9)); // after 𝄞 (5 + 4)
        assert_eq!(byte_to_position(src, 3), (0, 1));
        assert_eq!(byte_to_position(src, 9), (1, 2));
    }

    #[test]
    fn position_past_line_end_clamps() {
        let src = "ab\ncd";
        assert_eq!(position_to_byte(src, 0, 99), Some(2));
        assert_eq!(position_to_byte(src, 9, 0), None);
    }

    #[test]
    fn definition_from_paste_site() {
        let doc = parse();
        let paste_off = DOC.find(r##"<hick:paste select="#imports" />"##).unwrap();
        let span = definition(&doc, paste_off + 5).unwrap();
        assert!(DOC[span.0..span.1].starts_with(r#"<hick:copy id="imports""#));
    }

    #[test]
    fn definition_from_inside_copy_body() {
        let doc = parse();
        let body_off = DOC.find("use std::io;").unwrap();
        let span = definition(&doc, body_off).unwrap();
        assert!(DOC[span.0..span.1].starts_with(r#"<hick:copy id="imports""#));
    }

    #[test]
    fn references_lists_both_paste_sites() {
        let doc = parse();
        let body_off = DOC.find("use std::io;").unwrap();
        let refs = references(&doc, body_off, false);
        assert_eq!(refs.len(), 2);
        for span in &refs {
            assert!(DOC[span.0..span.1].starts_with(r##"<hick:paste select="#imports""##));
        }
        let with_decl = references(&doc, body_off, true);
        assert_eq!(with_decl.len(), 3);
    }

    #[test]
    fn no_copy_under_cursor_is_empty() {
        let doc = parse();
        let off = DOC.find("fn main() {}").unwrap();
        assert_eq!(definition(&doc, off), None);
        assert!(references(&doc, off, true).is_empty());
    }
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/structural.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/structural.rs"><storycode:paste select="#f-207b83769ac7-position_to_byte"/><storycode:paste select="#f-207b83769ac7-byte_to_position"/><storycode:paste select="#f-207b83769ac7-Span"/><storycode:paste select="#f-207b83769ac7-CopyDef"/><storycode:paste select="#f-207b83769ac7-PasteSite"/><storycode:paste select="#f-207b83769ac7-CopyGraph"/><storycode:paste select="#f-207b83769ac7-max_descendant_end"/><storycode:paste select="#f-207b83769ac7-walk"/><storycode:paste select="#f-207b83769ac7-build_graph"/><storycode:paste select="#f-207b83769ac7-id_at"/><storycode:paste select="#f-207b83769ac7-definition"/><storycode:paste select="#f-207b83769ac7-references"/><storycode:paste select="#f-207b83769ac7-tests"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-lsp/src/backend.rs](../../../mechanisms/crates/hick-lsp/src/backend.rs.md#output-c936496a03c1)
- [crates/hick-lsp/src/server_config.rs](../../../mechanisms/crates/hick-lsp/src/server_config.rs.md#output-58bede440b7e)
- [crates/hick-lsp/src/staging.rs](contract.md#output-76f75a35bcb3)
- [crates/hick-lsp/src/structural.rs](contract.md#output-207b83769ac7)
- [crates/hickory-cli/src/serve/lsp_bridge.rs](../../../mechanisms/crates/hickory-cli/src/serve/lsp_bridge.rs.md#output-a429fedbabb9)

### Tests and fixtures

- [crates/hickory-cli/tests/lsp_desktop.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_desktop.rs.md#output-c14f2fec35ca)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
