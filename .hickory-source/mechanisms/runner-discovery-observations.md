<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Bind runner discovery to saved-source observations

For maintainers changing saved-source runner discovery.

Runner discovery can build or import a project and can change files. Capture the saved snapshot before starting it, compare after it exits, and retain the earlier hashes rather than silently assigning its listing to new bytes. These are bounded workspace snapshots, not frozen inputs or proof of all runtime dependencies. Normalize reported file paths through the selected project and workspace; a runner name without a source remains unlocated.

<ui:copy id="runner-discovery-observation-0">
//! Observe a runner listing against saved source before and after it executes.
use super::*;
use sha2::{Digest, Sha256};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn source_state(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> &'static str {
    if before != after {
        "historical"
    } else if before.is_empty() {
        "unknown"
    } else {
        "current"
    }
}
fn source_paths(root: &Path, project: &Path, tests: &mut [TestResult]) {
    for test in tests {
        let Some(path) = test.path.take() else {
            test.line = None;
            continue;
        };
        let absolute = if Path::new(&path).is_absolute() {
            PathBuf::from(&path)
        } else {
            project.join(&path)
        };
        test.path = absolute
            .canonicalize()
            .ok()
            .filter(|path| path.starts_with(root) && path.is_file())
            .map(|path| store::relative(root, &path));
        if test.path.is_none() || test.line == Some(0) {
            test.line = None;
        }
    }
}
</ui:copy>

Plan through the existing provider, then observe the real runner between two saved snapshots. The fingerprint binds workspace, project, language, captured bytes and reported names/locations. The listing does not become execution authority or a test result.

<ui:copy id="runner-discovery-observation-1">
pub(super) async fn inspect(root: &Path, project: &Path, language: &str) -> Result<Value> {
    let metadata = config(project)?;
    let command = commands::discovery(project, language)?;
    let before = snapshot(root)?;
    let output = runner::execute(&command, project, None).await?;
    ensure!(output.code == Some(0), "Discovery failed: {}", output.text);
    let after = snapshot(root)?;
    let mut tests = results::discover(language, &output.text, &metadata);
    source_paths(root, project, &mut tests);
    let state = source_state(&before, &after);
    let workspace = hash(root.as_os_str().as_encoded_bytes());
    let project = store::relative(root, project);
    let fingerprint = hash(&serde_json::to_vec(
        &json!({"version":1,"workspace":workspace,"project":project,"language":language,"sources":before,"tests":tests.iter().map(|test|(&test.id,&test.path,&test.line)).collect::<Vec<_>>()}),
    )?);
    Ok(
        json!({"version":1,"tests":tests,"output":output.text,"metadata":metadata,"project":project,"language":language,"discovery":"runner","source_hashes":before,"source_state":state,"source_changed_during_discovery":before!=after,"source_frozen":false,"workspace_id":workspace,"fingerprint":fingerprint}),
    )
}
</ui:copy>

Focused model witnesses preserve pre-run bytes, distinguish empty snapshots and reject escaped/unresolvable paths. They do not certify provider output.

<ui:copy id="runner-discovery-observation-2">
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovery_observation_does_not_relabel_old_listing_with_new_source() {
        let before = BTreeMap::from([("test.py".into(), "before".into())]);
        assert_eq!(source_state(&before, &before), "current");
        let after = BTreeMap::from([("test.py".into(), "after".into())]);
        assert_eq!(source_state(&before, &after), "historical");
        assert_eq!(before["test.py"], "before");
        assert_eq!(source_state(&BTreeMap::new(), &BTreeMap::new()), "unknown");
    }
    #[test]
    fn discovery_observation_normalizes_project_relative_paths_without_escape() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let project = root.join("nested");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(project.join("test.py"), "def test_a(): pass").unwrap();
        let row = |path: &str, line| TestResult {
            id: "case".into(),
            status: "discovered".into(),
            output: "".into(),
            path: Some(path.into()),
            line,
            labels: vec![],
        };
        let mut tests = vec![
            row("test.py", Some(3)),
            row("../missing.py", Some(1)),
            row("test.py", Some(0)),
        ];
        source_paths(&root, &project, &mut tests);
        assert_eq!(tests[0].path.as_deref(), Some("nested/test.py"));
        assert_eq!(tests[0].line, Some(3));
        assert!(tests[1].path.is_none());
        assert!(tests[1].line.is_none());
        assert!(tests[2].line.is_none());
        let outside = tempfile::NamedTempFile::new().unwrap();
        let mut escaped = vec![row(&outside.path().to_string_lossy(), Some(1))];
        source_paths(&root, &project, &mut escaped);
        assert!(escaped[0].path.is_none());
        assert!(escaped[0].line.is_none());
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/coverage/discovery.rs"><ui:paste select="#runner-discovery-observation-0"/><ui:paste select="#runner-discovery-observation-1"/><ui:paste select="#runner-discovery-observation-2"/></ui:file>
</ui:doc>
