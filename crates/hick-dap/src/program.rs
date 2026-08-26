//! Turning a document into a program a debug adapter can launch.
//!
//! Both surfaces that start an adapter — an interactive session and a
//! document's `<hick:capture>` — need the same three things: the document's
//! files written somewhere, which of them is the program, and what language
//! it is. They need it identically, so it lives here rather than twice.
//!
//! The files are always written to a **scratch** directory. That is the whole
//! isolation story: the debuggee runs against a copy, so nothing it does
//! reaches the repository.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Write every file the document generates into `dir`, returning them in
/// document order.
pub fn weave_into(source: &str, dir: &Path) -> Result<Vec<PathBuf>> {
    let state = hick_lsp::document::HickDocumentState::from_source(source)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut written = Vec::new();
    for file in &state.virtual_files {
        let path = dir.join(&file.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&path, file.content())
            .with_context(|| format!("writing {}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

/// The language a generated file would be debugged as, or `None` when no
/// adapter exists for it.
///
/// Narrower than "what language is this file", deliberately: naming a
/// language hick cannot debug would start an adapter that can only fail.
pub fn language_of(path: &Path) -> Option<&'static str> {
    let name = path.to_string_lossy();
    let language = hick_lsp::lang_detect::language_id(&name)?;
    crate::known_languages()
        .into_iter()
        .find(|known| *known == language)
}

/// The first generated file in a debuggable language — the program.
pub fn entry_point(files: &[PathBuf]) -> Result<PathBuf> {
    files
        .iter()
        .find(|path| language_of(path).is_some())
        .cloned()
        .context(
            "this document generates no file in a language with a debug adapter, so there is \
             nothing to debug. Add a `hick:file` block whose path ends in a language hick can \
             debug (`hick dap list` names them).",
        )
}

/// The adapter command for a program, with the message a person can act on
/// when there is none.
pub fn adapter_for(program: &Path, project: &Path) -> Result<crate::Discovered> {
    let language = language_of(program).with_context(|| {
        format!(
            "{} is not in a language hick can debug",
            program.file_name().unwrap_or_default().to_string_lossy()
        )
    })?;
    crate::discover(language, project).with_context(|| {
        format!(
            "no debug adapter for {language} on this machine.\n{}",
            crate::discovery::how_to_get(language)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_file_routes_to_its_language() {
        assert_eq!(language_of(Path::new("app.py")), Some("python"));
        assert_eq!(language_of(Path::new("src/main.go")), Some("go"));
        // A language with no adapter is not debuggable, and must not be
        // offered as if it were.
        assert_eq!(language_of(Path::new("notes.md")), None);
    }

    #[test]
    fn the_entry_point_skips_files_nothing_can_debug() {
        let files = vec![
            PathBuf::from("README.md"),
            PathBuf::from("app.py"),
            PathBuf::from("other.py"),
        ];
        assert_eq!(entry_point(&files).unwrap(), PathBuf::from("app.py"));
    }

    #[test]
    fn a_document_with_nothing_debuggable_says_so() {
        let error = entry_point(&[PathBuf::from("notes.md")]).unwrap_err();
        assert!(format!("{error}").contains("hick dap list"), "{error}");
    }
}
