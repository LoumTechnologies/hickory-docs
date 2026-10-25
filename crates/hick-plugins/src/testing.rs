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
