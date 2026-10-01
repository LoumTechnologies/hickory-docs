// Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md
use hick_project_env::{
    Provider, Readiness, Registry,
    host::Host,
    providers::{node::Node, uv::Uv},
};
use std::{path::Path, time::Duration};

fn write(root: &Path, file: &str, text: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}
fn python(root: &Path) {
    write(
        root,
        "pyproject.toml",
        "[project]\nname='example'\nversion='0.1.0'\n",
    );
    write(root, "uv.lock", "version = 1\n");
}

#[test]
fn manifests_do_not_guess_a_package_manager_and_conflicts_refuse_actions() {
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "pyproject.toml", "[project]\nname='example'\n");
    assert!(Registry::default().discover(dir.path()).is_empty());
    write(dir.path(), "uv.lock", "version = 1\n");
    write(dir.path(), "poetry.lock", "");
    assert!(
        Registry::default().discover(dir.path())[0]
            .ambiguity
            .is_some()
    );
    write(
        dir.path(),
        "package.json",
        r#"{"packageManager":"pnpm@10.0.0"}"#,
    );
    write(dir.path(), "package-lock.json", "{}");
    assert!(Node.detects(dir.path()).unwrap().ambiguity.is_some());
}

#[test]
fn workspace_members_share_an_environment_but_independent_nested_projects_do_not() {
    let dir = tempfile::tempdir().unwrap();
    python(dir.path());
    write(
        dir.path(),
        "pyproject.toml",
        "[project]\nname='root'\nversion='0.1.0'\n[tool.uv.workspace]\nmembers=['packages/*']\nexclude=['packages/independent']\n",
    );
    python(&dir.path().join("packages/a"));
    python(&dir.path().join("packages/independent"));
    assert_eq!(
        hick_project_env::providers::uv::owner(&dir.path().join("packages/a"), dir.path()).unwrap(),
        dir.path()
    );
    let projects = Registry::default().discover(dir.path());
    assert_eq!(projects.len(), 2);
    write(
        dir.path(),
        "node/package.json",
        r#"{"packageManager":"pnpm@10.0.0"}"#,
    );
    write(
        dir.path(),
        "node/pnpm-workspace.yaml",
        "packages:\n  - 'packages/*'\n  - '!packages/separate'\n",
    );
    write(
        dir.path(),
        "node/packages/a/package.json",
        r#"{"packageManager":"pnpm@10.0.0"}"#,
    );
    write(
        dir.path(),
        "node/packages/separate/package.json",
        r#"{"packageManager":"pnpm@10.0.0"}"#,
    );
    assert!(hick_project_env::providers::node::is_member(
        &Node.detects(&dir.path().join("node/packages/a")).unwrap(),
        dir.path()
    ));
    assert!(!hick_project_env::providers::node::is_member(
        &Node
            .detects(&dir.path().join("node/packages/separate"))
            .unwrap(),
        dir.path()
    ));
}

#[tokio::test]
async fn a_missing_manager_is_a_typed_finding_with_no_install_command() {
    let dir = tempfile::tempdir().unwrap();
    python(dir.path());
    let host = Host {
        path: dir.path().as_os_str().into(),
        timeout: Duration::from_secs(1),
    };
    let f = Registry::default()
        .inspect(&Uv.detects(dir.path()).unwrap(), &host)
        .await;
    assert_eq!(f.state, Readiness::ManagerMissing);
    assert!(f.actions.is_empty());
    assert!(f.install_url.is_some());
}

#[cfg(unix)]
fn manager(root: &Path, body: &str) -> Host {
    use std::os::unix::fs::PermissionsExt;
    write(root, "tools/uv", &format!("#!/bin/sh\n{body}\n"));
    std::fs::set_permissions(
        root.join("tools/uv"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    Host {
        path: root.join("tools").as_os_str().into(),
        timeout: Duration::from_secs(2),
    }
}

#[cfg(unix)]
#[tokio::test]
async fn native_checks_distinguish_stale_lock_stale_environment_unknown_and_ready() {
    let dir = tempfile::tempdir().unwrap();
    python(dir.path());
    write(dir.path(), ".venv/pyvenv.cfg", "home=/python\n");
    let project = Uv.detects(dir.path()).unwrap();
    let registry = Registry::default();
    let ready = manager(
        dir.path(),
        "echo '{\"sync\":{\"action\":\"check\",\"changes\":[]}}'",
    );
    let before = std::fs::read(dir.path().join("uv.lock")).unwrap();
    assert_eq!(
        registry.inspect(&project, &ready).await.state,
        Readiness::Ready
    );
    let stale = manager(
        dir.path(),
        "echo '{\"sync\":{\"action\":\"check\",\"changes\":[{\"name\":\"missing\"}]}}'; exit 1",
    );
    assert_eq!(
        registry.inspect(&project, &stale).await.state,
        Readiness::EnvironmentStale
    );
    let broken = manager(dir.path(), "echo 'offline cache unavailable' >&2; exit 2");
    let f = registry.inspect(&project, &broken).await;
    assert_eq!(f.state, Readiness::Unknown);
    assert!(f.actions.is_empty());
    let lock = manager(
        dir.path(),
        "echo 'The lockfile needs to be updated, but --locked was provided' >&2; exit 1",
    );
    let f = registry.inspect(&project, &lock).await;
    assert_eq!(f.state, Readiness::LockStale);
    assert_eq!(f.actions[0].id, "resolve-sync");
    assert_eq!(std::fs::read(dir.path().join("uv.lock")).unwrap(), before);
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".venv/pyvenv.cfg")).unwrap(),
        "home=/python\n"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn unsupported_check_still_reports_a_missing_environment_and_diagnostics_do_not_churn_revisions()
 {
    let dir = tempfile::tempdir().unwrap();
    python(dir.path());
    let registry = Registry::default();
    let project = Uv.detects(dir.path()).unwrap();
    let host = manager(
        dir.path(),
        "echo 'unexpected argument --output-format' >&2; exit 2",
    );
    let f = registry.inspect(&project, &host).await;
    assert_eq!(f.state, Readiness::EnvironmentMissing);
    assert_eq!(f.actions[0].argv[1..], ["sync", "--locked"]);
    write(dir.path(), ".venv/pyvenv.cfg", "");
    let host = manager(
        dir.path(),
        "echo '{\"sync\":{\"action\":\"check\",\"changes\":[]}}'; echo 'Audited in 1ms' >&2",
    );
    let first = registry.inspect(&project, &host).await;
    let host = manager(
        dir.path(),
        "echo '{\"sync\":{\"action\":\"check\",\"changes\":[]}}'; echo 'Audited in 2ms' >&2",
    );
    let second = registry.inspect(&project, &host).await;
    assert_eq!(first.revision, second.revision);
    write(dir.path(), "pyproject.toml", "[project]\nname='changed'\n");
    assert_ne!(
        second.revision,
        registry.inspect(&project, &host).await.revision
    );
}

#[cfg(unix)]
#[tokio::test]
async fn a_check_that_never_returns_is_unknown_and_cannot_block_the_workspace() {
    let dir = tempfile::tempdir().unwrap();
    python(dir.path());
    let mut host = manager(dir.path(), "while :; do :; done");
    host.timeout = Duration::from_millis(100);
    let f = Registry::default()
        .inspect(&Uv.detects(dir.path()).unwrap(), &host)
        .await;
    assert_eq!(f.state, Readiness::Unknown);
    assert!(f.details.contains("timed out"));
}

#[cfg(unix)]
#[tokio::test]
async fn node_workspaces_offer_installation_for_member_dependencies_without_guessing_freshness() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    write(dir.path(), "package.json", r#"{"packageManager":"pnpm@10.0.0"}"#);
    write(dir.path(), "pnpm-workspace.yaml", "packages:\n - 'packages/*'\n");
    write(dir.path(), "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
    write(dir.path(), "packages/a/package.json", r#"{"dependencies":{"fixture":"1.0.0"}}"#);
    write(dir.path(), "tools/pnpm", "#!/bin/sh\nexit 0\n");
    std::fs::set_permissions(dir.path().join("tools/pnpm"), std::fs::Permissions::from_mode(0o755)).unwrap();
    let host = Host { path: dir.path().join("tools").as_os_str().into(), timeout: Duration::from_secs(1) };
    let registry = Registry::default();
    let projects = registry.discover(dir.path());
    assert_eq!(projects.len(), 1, "workspace member prompted as an independent project");
    let missing = registry.inspect(&projects[0], &host).await;
    assert_eq!(missing.state, Readiness::EnvironmentMissing);
    assert_eq!(missing.actions[0].argv[1..], ["install", "--frozen-lockfile"]);
    std::fs::create_dir(dir.path().join("node_modules")).unwrap();
    let existing = registry.inspect(&projects[0], &host).await;
    assert_eq!(existing.state, Readiness::Unknown);
    assert!(existing.actions.is_empty());
}
