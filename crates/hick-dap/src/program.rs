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
///
/// The bytes are the engine's: `written_content`, not `content`. The
/// difference is one leading newline, and it is the difference between a
/// `.csproj` MSBuild will load and one it refuses at line 2 position 1.
/// `Mapping::for_document` drops the same line from the other side.
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
        std::fs::write(&path, file.written_content())
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

/// No adapter for this language on this machine.
///
/// A **type** rather than a sentence, because it is the one debug failure a
/// person can fix without leaving the app: the app downcasts to it to offer
/// the install as a button, instead of printing a command and asking someone
/// to go and type it in a terminal. `installable` is asked of the same
/// function that writes the sentence, so a button is never offered for a
/// language `hick dap install` does not serve.
#[derive(Debug)]
pub struct MissingAdapter {
    pub language: String,
    /// What to do about it, in prose — the ecosystem's own way, for the
    /// languages hick cannot install.
    pub how: String,
    /// Whether `hick dap install <language>` is a real command.
    pub installable: bool,
}

impl std::fmt::Display for MissingAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "no debug adapter for {} on this machine.\n{}",
            self.language, self.how
        )
    }
}

impl std::error::Error for MissingAdapter {}

/// The adapter command for a program, with the message a person can act on
/// when there is none.
pub fn adapter_for(program: &Path, project: &Path) -> Result<crate::Discovered> {
    let language = language_of(program).with_context(|| {
        format!(
            "{} is not in a language hick can debug",
            program.file_name().unwrap_or_default().to_string_lossy()
        )
    })?;
    crate::discover(language, project).ok_or_else(|| {
        anyhow::Error::new(MissingAdapter {
            language: language.to_string(),
            how: crate::discovery::how_to_get(language),
            installable: crate::discovery::suggests_hick_install(language),
        })
    })
}

#[cfg(test)]
mod tests {
    /// A scaffold a document owns: `exec > ingested > file`, which is what
    /// `hick ingest --from '#cell'` writes. Nothing here is at the top
    /// level, which is the whole point.
    const INGESTED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
        # Scaffolded\n\n\
        <hick:exec container=\"sdk\">\n\
        <hick:copy id=\"scaffold\">\n\
        dotnet new console -n app -o out\n\
        </hick:copy>\n\
        <hick:ingested from=\"#scaffold\" sha256=\"dead\" at=\"2026-09-03\" files=\"2\" skipped=\"0\">\n\
        <hick:file path=\"app/Program.cs\">\n\
        \u{feff}Console.WriteLine(\"Hello, World!\");\n\
        </hick:file>\n\
        <hick:file path=\"app/app.csproj\">\n\
        \u{feff}<Project Sdk=\"Microsoft.NET.Sdk\" />\n\
        </hick:file>\n\
        </hick:ingested>\n\
        </hick:exec>\n\
        </hick:doc>\n";

    #[test]
    fn a_file_a_document_owns_is_generated_however_deeply_it_is_nested() {
        // The bug this exists for: `hick ingest` writes its files three tags
        // down, the engine's weave has always written them (`all_tags`), and
        // this weave asked only the top level — so Debug on a scaffolded
        // `Program.cs` answered "this document does not generate
        // app/Program.cs. It generates:" and then nothing at all.
        let dir = tempfile::tempdir().unwrap();
        let files = weave_into(INGESTED, dir.path()).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|p| p.strip_prefix(dir.path()).unwrap().display().to_string())
            .collect();
        assert_eq!(names, vec!["app/Program.cs", "app/app.csproj"]);
        assert!(entry_point(&files).unwrap().ends_with("Program.cs"));
    }

    #[test]
    fn what_has_to_be_at_byte_zero_is_at_byte_zero() {
        // `dotnet new` writes a BOM. A leading blank line puts it at line 2
        // position 1, where MSBuild refuses the project file outright
        // (MSB4025) — so the bytes written are the engine's, with no line in
        // front of them. The same would be true of a `#!` line and an XML
        // declaration; a BOM is only the one that was caught.
        let dir = tempfile::tempdir().unwrap();
        let files = weave_into(INGESTED, dir.path()).unwrap();
        for file in &files {
            let bytes = std::fs::read(file).unwrap();
            assert_eq!(
                &bytes[..3],
                &[0xEF, 0xBB, 0xBF],
                "{} does not start with the bytes the document holds",
                file.display()
            );
        }
    }

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
    fn a_missing_adapter_is_a_type_the_app_can_act_on() {
        // The whole point: the app must be able to tell "install this" from
        // "something else went wrong" without reading English — so the type
        // is asserted directly, on every machine, rather than through a
        // discovery call whose answer depends on what is installed here.
        let missing = MissingAdapter {
            language: "python".to_string(),
            how: crate::discovery::how_to_get("python"),
            installable: crate::discovery::suggests_hick_install("python"),
        };
        assert!(missing.installable, "hick dap install python exists");
        let error = anyhow::Error::new(missing);
        assert_eq!(
            error
                .downcast_ref::<MissingAdapter>()
                .expect("the failure is the typed one")
                .language,
            "python"
        );
        // The sentence a terminal shows is unchanged.
        assert!(
            format!("{error}").starts_with("no debug adapter for python on this machine."),
            "{error}"
        );
    }

    #[test]
    fn adapter_for_wraps_a_failed_discovery_in_that_type() {
        // The wiring half. Only meaningful on a machine with no python
        // adapter — this one may have debugpy, and a test that demanded its
        // absence failed here while passing in CI, which is the wrong way
        // round for a check meant to protect a promise.
        let found = adapter_for(Path::new("app.py"), Path::new("/nonexistent-project"));
        match found {
            Err(error) => assert!(
                error.downcast_ref::<MissingAdapter>().is_some(),
                "a failed discovery produced an untyped error: {error}"
            ),
            Ok(discovered) => assert!(
                !discovered.command.is_empty(),
                "discovery succeeded with no command to run"
            ),
        }
    }

    #[test]
    fn a_document_with_nothing_debuggable_says_so() {
        let error = entry_point(&[PathBuf::from("notes.md")]).unwrap_err();
        assert!(format!("{error}").contains("hick dap list"), "{error}");
    }
}
