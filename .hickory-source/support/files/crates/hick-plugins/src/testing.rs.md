<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/testing.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `5827696319a116cafaddf5f333ba8dc1dbeb1f4bf12d9645d3031441154115fc`.

<book:entry path="crates/hick-plugins/src/testing.rs">
use std::path::{Path, PathBuf};
pub type Planner = fn(&Path, &Path, &str) -> Result<TestCommand, String>;
/// What will run, and where.
#[derive(Debug, PartialEq, Eq)]
pub struct TestCommand {
    pub title: String,
    pub cwd: PathBuf,
    pub argv: Vec<String>,
}

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
</book:entry>

</book:doc>
