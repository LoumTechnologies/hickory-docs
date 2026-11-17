use std::path::{Path, PathBuf};
/// Shared native file-selection adapter for source Run and retained failed-only coverage.
pub const JAVASCRIPT_TEST_DRIVER:&str=include_str!("testing/javascript.mjs");
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
        Some(crate::Interface::TestRunner { plan, .. }) => plan(root, file, name),
        _ => Err(format!("hick does not know how to run {language} tests")),
    }
}
pub mod candidates;
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    File,
    FileFilter,
    Crate,
    Package,
    Project,
}
pub fn scope(language: &str) -> Result<Scope, String> {
    let language = crate::routing::language_id(language).unwrap_or(language);
    match crate::registry().interface(language, crate::InterfaceKind::TestRunner) {
        Some(crate::Interface::TestRunner { scope, .. }) => Ok(*scope),
        _ => Err(format!(
            "No test runner scope for {language}. Choose a configured test language."
        )),
    }
}
pub type DebugPlanner = fn(&Path, &Path, &str) -> Result<TestDebugCommand, String>;
/// A language-owned launch configuration, never a client-supplied command.
#[derive(Debug)]
pub struct TestDebugCommand {
    pub title: String,
    pub cwd: PathBuf,
    pub program: PathBuf,
    pub launch: serde_json::Value,
    pub framework: &'static str,
}
pub fn test_debug_command(
    root: &Path,
    file: &Path,
    name: &str,
    language: &str,
) -> Result<TestDebugCommand, String> {
    let language = crate::routing::language_id(language).unwrap_or(language);
    if !file.starts_with(root) {
        return Err("Test source is outside the runtime workspace".into());
    }
    match crate::registry().interface(language, crate::InterfaceKind::TestRunner) {
        Some(crate::Interface::TestRunner {
            debug: Some(plan), ..
        }) => plan(root, file, name),
        _ => Err(format!(
            "The {language} test provider does not supply a debugger launch. Run is available separately; no ordinary-file Debug fallback was performed."
        )),
    }
}
#[cfg(test)]
mod debug_plan_tests {
    use super::*;
    #[test]
    fn python_debug_uses_the_same_qualified_pytest_family_without_shell_parsing() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("-literal %2F #.py");
        std::fs::write(
            &file,
            "class TestChoice:\n    def test_selected(self): pass\n",
        )
        .unwrap();
        let plan = test_debug_command(
            directory.path(),
            &file,
            "TestChoice::test_selected",
            "python",
        )
        .unwrap();
        assert_eq!(plan.framework, "pytest");
        assert_eq!(plan.program, file);
        assert_eq!(plan.launch["module"], "pytest");
        assert!(plan.launch["program"].is_null());
        assert_eq!(
            plan.launch["args"][0],
            "./-literal %2F #.py::TestChoice::test_selected"
        );
    }
    #[test]
    fn absent_debug_provider_never_falls_back_to_a_run_command() {
        let directory = tempfile::tempdir().unwrap();
        let error = test_debug_command(
            directory.path(),
            &directory.path().join("test.rs"),
            "test_selected",
            "rust",
        )
        .unwrap_err();
        assert!(error.contains("does not supply a debugger launch"));
    }
}
