//! Which program evaluates a language's formulas, and getting it onto the
//! machine.
//!
//! # Why there is nothing to download
//!
//! `hick lsp install` and `hick dap install` fetch things, because a language
//! server is a large third-party program nobody would want us to reimplement.
//! A formula backend is the opposite: sixty lines that read a pipe and call
//! `eval`. So the backends are **embedded in this binary** and written into
//! the project's cache the first time they are needed.
//!
//! That makes "auto-installed" literally true, and true offline. There is no
//! network path here at all — no URL, no fallback, no "re-run when you are
//! online". What a machine needs is the *interpreter*, which it either has or
//! does not, and which we say plainly rather than trying to provide.
//!
//! # Where they go
//!
//! `.hick-cache/formula/`, which `hick init` already keeps out of git — the
//! same place `hick search --install-model` puts its model. A backend is
//! derived, reproducible from the binary that wrote it, and belongs in a
//! cache rather than in anybody's history.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

/// A language's backend: the script, and the interpreter that runs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backend {
    /// Canonical language id, as a document spells it.
    pub language: &'static str,
    /// The file the script is written to, under the formula cache.
    pub file_name: &'static str,
    /// Interpreters to try, best first. The first one that answers
    /// `--version` wins.
    pub interpreters: &'static [&'static str],
    /// The script itself, embedded in this binary.
    pub source: &'static str,
}

/// Every backend this build carries.
///
/// Adding a language is an entry here plus a script — deliberately, because
/// the host owns references, ordering and cycles (see `graph.rs`), so a
/// backend has nothing left to get wrong.
pub const BACKENDS: &[Backend] = &[
    Backend {
        language: "python",
        file_name: "python_backend.py",
        // `python3` first: on the machines where `python` still exists it is
        // as likely to be Python 2 as not, and a Python 2 backend fails in a
        // way that reads like a bug in the formula.
        interpreters: &["python3", "python"],
        source: include_str!("../backends/python_backend.py"),
    },
    Backend {
        language: "javascript",
        file_name: "javascript_backend.mjs",
        interpreters: &["node"],
        source: include_str!("../backends/javascript_backend.mjs"),
    },
];

/// Aliases a document might use for a language.
pub fn canonical_language(name: &str) -> Option<&'static str> {
    match name.trim().to_ascii_lowercase().as_str() {
        "python" | "py" | "python3" => Some("python"),
        "javascript" | "js" | "node" | "mjs" => Some("javascript"),
        _ => None,
    }
}

/// The backend for a language, whatever it was called.
pub fn backend_for(language: &str) -> Option<&'static Backend> {
    let canonical = canonical_language(language)?;
    BACKENDS.iter().find(|b| b.language == canonical)
}

/// Where backends live for a project.
pub fn backend_dir(root: &Path) -> PathBuf {
    root.join(".hick-cache").join("formula")
}

/// The first interpreter on this machine that answers `--version`.
///
/// Checked rather than assumed: a `node` on `PATH` that is a broken shim is a
/// far more confusing failure at evaluation time than at install time.
pub fn find_interpreter(backend: &Backend) -> Option<String> {
    backend
        .interpreters
        .iter()
        .find(|name| {
            std::process::Command::new(name)
                .arg("--version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
        .map(|name| name.to_string())
}

/// Write a language's backend into the project's cache, and check that
/// something can run it.
///
/// Idempotent, and it rewrites rather than skipping: the script is derived
/// from this binary, so an older copy left by an older version is exactly the
/// thing to replace. Nothing is downloaded — see the module header.
pub fn install(root: &Path, language: &str) -> Result<PathBuf> {
    let backend = backend_for(language).with_context(|| {
        let known: Vec<&str> = BACKENDS.iter().map(|b| b.language).collect();
        format!(
            "no formula backend for `{language}`.\n  \
             This build carries: {}.\n  \
             A backend is a small script that evaluates one expression at a \
             time; adding one is a contribution to hick-formula, not a \
             download.",
            known.join(", ")
        )
    })?;

    let dir = backend_dir(root);
    std::fs::create_dir_all(&dir).with_context(|| format!("could not create {}", dir.display()))?;
    let path = dir.join(backend.file_name);
    std::fs::write(&path, backend.source)
        .with_context(|| format!("could not write {}", path.display()))?;

    if find_interpreter(backend).is_none() {
        bail!(
            "wrote {} — but nothing on this machine can run it.\n  \
             Tried: {}.\n  \
             Install one and formulas in `{}` will work; documents using \
             other languages are unaffected.",
            path.display(),
            backend.interpreters.join(", "),
            backend.language,
        );
    }
    Ok(path)
}

/// Every language this project can evaluate right now: a backend this build
/// knows, an interpreter this machine has.
pub fn installed_languages(root: &Path) -> Vec<&'static str> {
    BACKENDS
        .iter()
        .filter(|b| backend_dir(root).join(b.file_name).is_file() && find_interpreter(b).is_some())
        .map(|b| b.language)
        .collect()
}

/// The command that runs a language's backend, installing it if it is not
/// there yet.
///
/// This is the "auto" in auto-installed: the first formula in a document puts
/// the script in place. Writing sixty lines into a cache is not an act worth
/// asking permission for, which is exactly why it can be automatic here and
/// could not be for `hick lsp install`.
pub fn ensure(root: &Path, language: &str) -> Result<(String, PathBuf)> {
    let backend = backend_for(language)
        .with_context(|| format!("no formula backend for `{language}` in this build"))?;
    let path = backend_dir(root).join(backend.file_name);
    // Rewritten when it is missing OR stale: the script is derived from this
    // binary, so a copy left by an older version is exactly what to replace.
    let current = std::fs::read_to_string(&path).ok();
    if current.as_deref() != Some(backend.source) {
        install(root, language)?;
    }
    let interpreter = find_interpreter(backend).with_context(|| {
        format!(
            "formulas in `{}` need one of: {}. None is on this machine.",
            backend.language,
            backend.interpreters.join(", ")
        )
    })?;
    Ok((interpreter, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_is_found_by_any_name_a_document_would_use() {
        assert_eq!(backend_for("py").map(|b| b.language), Some("python"));
        assert_eq!(backend_for("Python3").map(|b| b.language), Some("python"));
        assert_eq!(backend_for("js").map(|b| b.language), Some("javascript"));
        assert_eq!(backend_for("node").map(|b| b.language), Some("javascript"));
    }

    #[test]
    fn an_unknown_language_is_none_rather_than_a_guess() {
        assert!(backend_for("cobol").is_none());
        assert!(backend_for("").is_none());
    }

    #[test]
    fn installing_writes_the_script_into_the_cache_git_ignores() {
        let dir = tempfile::tempdir().unwrap();
        // May fail on a machine with no python; the FILE is written either
        // way, which is the part this asserts.
        let _ = install(dir.path(), "python");
        let written = dir.path().join(".hick-cache/formula/python_backend.py");
        assert!(written.is_file(), "the script is written");
        assert!(
            std::fs::read_to_string(&written)
                .unwrap()
                .contains("def evaluate"),
            "and it is the real script"
        );
    }

    #[test]
    fn installing_an_unknown_language_says_what_this_build_carries() {
        let dir = tempfile::tempdir().unwrap();
        let error = install(dir.path(), "cobol").unwrap_err().to_string();
        assert!(error.contains("python"), "{error}");
        assert!(error.contains("not a download"), "{error}");
    }

    #[test]
    fn installing_replaces_an_older_copy_rather_than_skipping_it() {
        // The script is derived from this binary, so a stale copy left by an
        // older version is exactly the thing to overwrite.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".hick-cache/formula/python_backend.py");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "# stale\n").unwrap();
        let _ = install(dir.path(), "python");
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("def evaluate")
        );
    }

    #[test]
    fn nothing_is_written_outside_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let _ = install(dir.path(), "python");
        let top: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(top, vec![".hick-cache".to_string()]);
    }
}
