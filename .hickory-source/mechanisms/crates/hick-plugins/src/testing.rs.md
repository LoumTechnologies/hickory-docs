<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/testing.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9a1aec043cf8"></a>
## `crates/hick-plugins/src/testing.rs`

<a id="f-9a1aec043cf8-Planner"></a>
### `Planner`

<storycode:copy id="f-9a1aec043cf8-Planner">
use std::path::{Path, PathBuf};
pub type Planner = fn(&Path, &Path, &str) -> Result<TestCommand, String>;
</storycode:copy>
<a id="f-9a1aec043cf8-TestCommand"></a>
### `TestCommand`

<storycode:copy id="f-9a1aec043cf8-TestCommand">
/// What will run, and where.
#[derive(Debug, PartialEq, Eq)]
pub struct TestCommand {
    pub title: String,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
}

</storycode:copy>
<a id="f-9a1aec043cf8-nearest"></a>
### `nearest`

The nearest ancestor of `file` (itself included) holding `manifest`,
stopping at `root`.

<storycode:copy id="f-9a1aec043cf8-nearest">
/// The nearest ancestor of `file` (itself included) holding `manifest`,
/// stopping at `root`.
pub(crate) fn nearest(root: &Path, file: &Path, manifest: &str) -> Option<PathBuf> {
    let mut dir = file.parent()?;
    loop {
        if dir.join(manifest).is_file() {
            return Some(dir.to_path_buf());
        }
        if dir == root {
            return None;
        }
        dir = dir.parent()?;
    }
}

</storycode:copy>
<a id="f-9a1aec043cf8-nearest_glob"></a>
### `nearest_glob`

The nearest ancestor holding a file with `extension`.

<storycode:copy id="f-9a1aec043cf8-nearest_glob">
/// The nearest ancestor holding a file with `extension`.
pub(crate) fn nearest_glob(root: &Path, file: &Path, extension: &str) -> Option<PathBuf> {
    let mut dir = file.parent()?;
    loop {
        if let Ok(entries) = std::fs::read_dir(dir)
            && entries.flatten().any(|e| {
                e.path()
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with(extension))
            })
        {
            return Some(dir.to_path_buf());
        }
        if dir == root {
            return None;
        }
        dir = dir.parent()?;
    }
}

</storycode:copy>
<a id="f-9a1aec043cf8-test_command"></a>
### `test_command`

<storycode:copy id="f-9a1aec043cf8-test_command">
pub fn test_command(
    root: &Path,
    file: &Path,
    name: &str,
    language: &str,
) -> Result<TestCommand, String> {
    let language = crate::routing::language_id(language).unwrap_or(language);
    if !file.starts_with(root) {
        return Err("test file is outside the workspace".into());
    }
    match crate::registry().interface(language, crate::InterfaceKind::TestRunner) {
        Some(crate::Interface::TestRunner { plan }) => plan(root, file, name),
        _ => Err(format!("hick does not know how to run {language} tests")),
    }
}
</storycode:copy>

### Assemble `crates/hick-plugins/src/testing.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/testing.rs"><storycode:paste select="#f-9a1aec043cf8-Planner"/><storycode:paste select="#f-9a1aec043cf8-TestCommand"/><storycode:paste select="#f-9a1aec043cf8-nearest"/><storycode:paste select="#f-9a1aec043cf8-nearest_glob"/><storycode:paste select="#f-9a1aec043cf8-test_command"/></storycode:file>

</storycode:doc>
