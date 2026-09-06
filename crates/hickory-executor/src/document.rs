//! What a cell is told about the document it belongs to.
//!
//! A cell's working directory is a scratch directory — that is what makes a
//! run reproducible and what stops a document from quietly writing into the
//! folder it lives in. The cost was that a document had no way to say "run
//! the tests of the project I am part of" except by hard-coding an absolute
//! path, which is the one thing a file people send each other must not
//! contain. Two environment variables close that without giving the scratch
//! directory up. See
//! `docs/guarantees/execution/a-cell-knows-where-its-document-is.md`.
//!
//! Beside `lib.rs` rather than in it because that file is long past the
//! file-length ratchet's line and may only shrink
//! (`scripts/check-file-length.sh`).

use std::path::{Path, PathBuf};

use crate::LocalExecutor;

impl LocalExecutor {
    /// Name the document whose cells this executor runs, so every cell can
    /// find the repository it belongs to.
    ///
    /// A cell's cwd is a scratch directory — that is what makes a run
    /// reproducible and what stops a document from quietly writing into the
    /// folder it lives in. The cost was that a document had NO way to say
    /// "run the tests of the project I am part of" except by hard-coding an
    /// absolute path, which is the one thing a document people send each
    /// other must not contain. Two variables fix that without giving the
    /// scratch directory up:
    ///
    /// - `HICK_DOC_DIR` — the absolute directory the document is in.
    /// - `HICK_DOC_PATH` — the absolute path of the document file.
    ///
    /// Both are canonicalised, so a cell that prints one gets the same
    /// bytes whether the document was named `d.hick`, `./d.hick` or an
    /// absolute path — the same reasoning as the derived scratch directory
    /// above, and for the same reason: a value that differs per invocation
    /// would report drift forever.
    ///
    /// **Under the sandbox the directory is usually not readable.** That is
    /// correct and deliberate — confinement is what makes running someone
    /// else's document safe — and the variable is still true: it says where
    /// the document is, not that this cell may read it. A cell that needs
    /// the project reachable is a cell for `HICKORY_EXECUTOR=local`.
    pub fn for_document(mut self, document: &Path) -> Self {
        self.document = Some(std::fs::canonicalize(document).unwrap_or_else(|_| {
            match std::env::current_dir() {
                Ok(cwd) => cwd.join(document),
                Err(_) => document.to_path_buf(),
            }
        }));
        self
    }

    /// The document variables every cell is given, empty when no document
    /// was named (`hick up` serving a folder, the MCP server, a test).
    pub(crate) fn document_env(&self) -> Vec<(&'static str, PathBuf)> {
        let Some(document) = &self.document else {
            return Vec::new();
        };
        let mut env = Vec::new();
        if let Some(dir) = document.parent() {
            env.push(("HICK_DOC_DIR", dir.to_path_buf()));
        }
        env.push(("HICK_DOC_PATH", document.clone()));
        env
    }
}
