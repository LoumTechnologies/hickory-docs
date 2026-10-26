<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A project environment offers its own sync

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a host project identified as uv, npm or pnpm by its manifest,
configuration or lockfile, when Hickory checks its environment, then it
reports a typed finding and offers only a repair the selected provider can
plan. An ambiguous manager offers an explicit choice, saved in the user's
workspace state, before it offers installation.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-b84eb9a7e17c"></a>
## `apps/web/src/environments/useEnvironments.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A project environment offers its own sync](contract.md).

<a id="f-b84eb9a7e17c-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md

<storycode:copy id="f-b84eb9a7e17c-expression_statement">
// Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { environmentApi, type EnvironmentFinding } from "../api/environments";
import { useEnvironments } from "./useEnvironments";
import { onShowTerminal } from "../lib/revealLine";

vi.mock("../api/environments", () => ({ environmentApi: { inspect: vi.fn(), act: vi.fn(), choose: vi.fn() } }));
</storycode:copy>
<a id="f-b84eb9a7e17c-finding"></a>
### `finding`

<storycode:copy id="f-b84eb9a7e17c-finding">
const finding: EnvironmentFinding = {
  project: "/project", manager: "uv", target: "host", profile: "manager defaults", revision: "one",
  state: "environment-missing", message: "Missing dependencies", details: "", actions: [], install_url: null,
  interpreter: null, manager_choices: [],
};
</storycode:copy>
<a id="f-b84eb9a7e17c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b84eb9a7e17c-expression_statement-2">
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(environmentApi.inspect).mockResolvedValue({ findings: [finding], running: {} });
});
</storycode:copy>
<a id="f-b84eb9a7e17c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-b84eb9a7e17c-expression_statement-3">
afterEach(cleanup);

</storycode:copy>
<a id="f-b84eb9a7e17c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-b84eb9a7e17c-expression_statement-4">
it("dismisses only the observed evidence revision and clears a successful recheck", async () => {
  const { result } = renderHook(useEnvironments);
  await waitFor(() => expect(result.current.notices).toHaveLength(1));
  act(() => result.current.dismiss(finding));
  await act(() => result.current.refresh(true));
  expect(result.current.notices).toHaveLength(0);
  vi.mocked(environmentApi.inspect).mockResolvedValue({ findings: [{ ...finding, revision: "two" }], running: {} });
  await act(() => result.current.refresh(true));
  expect(result.current.notices).toHaveLength(1);
  vi.mocked(environmentApi.inspect).mockResolvedValue({ findings: [{ ...finding, state: "ready", revision: "three" }], running: {} });
  await act(() => result.current.refresh(true));
  expect(result.current.notices).toHaveLength(0);
});

</storycode:copy>
<a id="f-b84eb9a7e17c-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-b84eb9a7e17c-expression_statement-5">
it("opens the returned terminal and reports failed repairs without hiding the finding", async () => {
  const show = vi.fn(); const unsubscribe = onShowTerminal(show);
  const { result } = renderHook(useEnvironments);
  await waitFor(() => expect(result.current.notices).toHaveLength(1));
  vi.mocked(environmentApi.act).mockResolvedValue({ id: "session", title: "uv: sync" } as never);
  await act(() => result.current.act(finding, "sync"));
  expect(show).toHaveBeenCalledWith({ id: "session", title: "uv: sync" });
  vi.mocked(environmentApi.act).mockRejectedValue(new Error("The environment changed. Recheck."));
  await act(() => result.current.act(finding, "sync"));
  expect(result.current.error).toBe("The environment changed. Recheck.");
  expect(result.current.notices).toHaveLength(1);
  unsubscribe();
});
</storycode:copy>

### Assemble `apps/web/src/environments/useEnvironments.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/environments/useEnvironments.test.tsx"><storycode:paste select="#f-b84eb9a7e17c-expression_statement"/><storycode:paste select="#f-b84eb9a7e17c-finding"/><storycode:paste select="#f-b84eb9a7e17c-expression_statement-2"/><storycode:paste select="#f-b84eb9a7e17c-expression_statement-3"/><storycode:paste select="#f-b84eb9a7e17c-expression_statement-4"/><storycode:paste select="#f-b84eb9a7e17c-expression_statement-5"/></storycode:file>

<a id="output-5e0ba6cf199b"></a>
## `crates/hick-project-env/tests/providers.rs`

Discover workspace environments and publish evaluated project entries through the host filesystem boundary.

Related contracts: [A project environment offers its own sync](contract.md).

<a id="f-5e0ba6cf199b-write"></a>
### `write`

Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md

<storycode:copy id="f-5e0ba6cf199b-write">
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
</storycode:copy>
<a id="f-5e0ba6cf199b-python"></a>
### `python`

<storycode:copy id="f-5e0ba6cf199b-python">
fn python(root: &Path) {
    write(
        root,
        "pyproject.toml",
        "[project]\nname='example'\nversion='0.1.0'\n",
    );
    write(root, "uv.lock", "version = 1\n");
}

</storycode:copy>
<a id="f-5e0ba6cf199b-manifests_do_not_guess_a_package_manager_and_conflicts_refuse_actions"></a>
### `manifests_do_not_guess_a_package_manager_and_conflicts_refuse_actions`

<storycode:copy id="f-5e0ba6cf199b-manifests_do_not_guess_a_package_manager_and_conflicts_refuse_actions">
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

</storycode:copy>
<a id="f-5e0ba6cf199b-workspace_members_share_an_environment_but_independent_nested_projects_do_not"></a>
### `workspace_members_share_an_environment_but_independent_nested_projects_do_not`

<storycode:copy id="f-5e0ba6cf199b-workspace_members_share_an_environment_but_independent_nested_projects_do_not">
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

</storycode:copy>
<a id="f-5e0ba6cf199b-a_missing_manager_is_a_typed_finding_with_no_install_command"></a>
### `a_missing_manager_is_a_typed_finding_with_no_install_command`

<storycode:copy id="f-5e0ba6cf199b-a_missing_manager_is_a_typed_finding_with_no_install_command">
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

</storycode:copy>
<a id="f-5e0ba6cf199b-manager"></a>
### `manager`

<storycode:copy id="f-5e0ba6cf199b-manager">
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

</storycode:copy>
<a id="f-5e0ba6cf199b-native_checks_distinguish_stale_lock_stale_environment_unknown_and_ready"></a>
### `native_checks_distinguish_stale_lock_stale_environment_unknown_and_ready`

<storycode:copy id="f-5e0ba6cf199b-native_checks_distinguish_stale_lock_stale_environment_unknown_and_ready">
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

</storycode:copy>
<a id="f-5e0ba6cf199b-unsupported_check_still_reports_a_missing_environment_and_diagnostics_do_not_churn_revisio"></a>
### `unsupported_check_still_reports_a_missing_environment_and_diagnostics_do_not_churn_revisions`

<storycode:copy id="f-5e0ba6cf199b-unsupported_check_still_reports_a_missing_environment_and_diagnostics_do_not_churn_revisio">
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

</storycode:copy>
<a id="f-5e0ba6cf199b-a_check_that_never_returns_is_unknown_and_cannot_block_the_workspace"></a>
### `a_check_that_never_returns_is_unknown_and_cannot_block_the_workspace`

<storycode:copy id="f-5e0ba6cf199b-a_check_that_never_returns_is_unknown_and_cannot_block_the_workspace">
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

</storycode:copy>
<a id="f-5e0ba6cf199b-node_workspaces_offer_installation_for_member_dependencies_without_guessing_freshness"></a>
### `node_workspaces_offer_installation_for_member_dependencies_without_guessing_freshness`

<storycode:copy id="f-5e0ba6cf199b-node_workspaces_offer_installation_for_member_dependencies_without_guessing_freshness">
#[cfg(unix)]
#[tokio::test]
async fn node_workspaces_offer_installation_for_member_dependencies_without_guessing_freshness() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    write(
        dir.path(),
        "package.json",
        r#"{"packageManager":"pnpm@10.0.0"}"#,
    );
    write(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n - 'packages/*'\n",
    );
    write(dir.path(), "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
    write(
        dir.path(),
        "packages/a/package.json",
        r#"{"dependencies":{"fixture":"1.0.0"}}"#,
    );
    write(dir.path(), "tools/pnpm", "#!/bin/sh\nexit 0\n");
    std::fs::set_permissions(
        dir.path().join("tools/pnpm"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let host = Host {
        path: dir.path().join("tools").as_os_str().into(),
        timeout: Duration::from_secs(1),
    };
    let registry = Registry::default();
    let projects = registry.discover(dir.path());
    assert_eq!(
        projects.len(),
        1,
        "workspace member prompted as an independent project"
    );
    let missing = registry.inspect(&projects[0], &host).await;
    assert_eq!(missing.state, Readiness::EnvironmentMissing);
    assert_eq!(
        missing.actions[0].argv[1..],
        ["install", "--frozen-lockfile"]
    );
    std::fs::create_dir(dir.path().join("node_modules")).unwrap();
    let existing = registry.inspect(&projects[0], &host).await;
    assert_eq!(existing.state, Readiness::Unknown);
    assert!(existing.actions.is_empty());
}
</storycode:copy>

### Assemble `crates/hick-project-env/tests/providers.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-project-env/tests/providers.rs"><storycode:paste select="#f-5e0ba6cf199b-write"/><storycode:paste select="#f-5e0ba6cf199b-python"/><storycode:paste select="#f-5e0ba6cf199b-manifests_do_not_guess_a_package_manager_and_conflicts_refuse_actions"/><storycode:paste select="#f-5e0ba6cf199b-workspace_members_share_an_environment_but_independent_nested_projects_do_not"/><storycode:paste select="#f-5e0ba6cf199b-a_missing_manager_is_a_typed_finding_with_no_install_command"/><storycode:paste select="#f-5e0ba6cf199b-manager"/><storycode:paste select="#f-5e0ba6cf199b-native_checks_distinguish_stale_lock_stale_environment_unknown_and_ready"/><storycode:paste select="#f-5e0ba6cf199b-unsupported_check_still_reports_a_missing_environment_and_diagnostics_do_not_churn_revisio"/><storycode:paste select="#f-5e0ba6cf199b-a_check_that_never_returns_is_unknown_and_cannot_block_the_workspace"/><storycode:paste select="#f-5e0ba6cf199b-node_workspaces_offer_installation_for_member_dependencies_without_guessing_freshness"/></storycode:file>

<a id="output-61003cc8a070"></a>
## `crates/hickory-cli/tests/serve_environments.rs`

Related contracts: [A project environment offers its own sync](contract.md).

<a id="f-61003cc8a070-start"></a>
### `start`

Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md
Real package manager, real HTTP action, real terminal, selected interpreter.

<storycode:copy id="f-61003cc8a070-start">
// Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md
//! Real package manager, real HTTP action, real terminal, selected interpreter.
use hickory_cli::{
    ExecutorChoice,
    serve::{LocalState, ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::{path::Path, process::Command, time::Duration};

async fn start(root: &Path) -> (String, LocalState, tokio::task::JoinHandle<()>) {
    let prepared = prepare(ServeOptions {
        target: root.to_path_buf(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let state = prepared.state;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    (url, state, task)
}

</storycode:copy>
<a id="f-61003cc8a070-status"></a>
### `status`

<storycode:copy id="f-61003cc8a070-status">
async fn status(url: &str) -> Value {
    reqwest::get(format!("{url}/api/environments?refresh=true"))
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap()
}

</storycode:copy>
<a id="f-61003cc8a070-run"></a>
### `run`

<storycode:copy id="f-61003cc8a070-run">
fn run(root: &Path, program: &Path, args: &[&str]) {
    let out = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

</storycode:copy>
<a id="f-61003cc8a070-uv_sync_in_a_terminal_prepares_the_interpreter_used_by_project_tests"></a>
### `uv_sync_in_a_terminal_prepares_the_interpreter_used_by_project_tests`

<storycode:copy id="f-61003cc8a070-uv_sync_in_a_terminal_prepares_the_interpreter_used_by_project_tests">
#[tokio::test]
async fn uv_sync_in_a_terminal_prepares_the_interpreter_used_by_project_tests() {
    let Some(uv) = hick_project_env::host::Host::default().executable("uv") else {
        eprintln!("uv is unavailable; real-manager integration is exercised on the uv CI job");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let found = Command::new(&uv)
        .args(["python", "find", "--offline"])
        .output()
        .unwrap();
    assert!(found.status.success());
    let python = std::path::PathBuf::from(String::from_utf8(found.stdout).unwrap().trim());
    // A local wheel avoids network and caches. The dependency is only importable
    // from the environment prepared by the button, not from system Python.
    run(
        &root,
        &python,
        &[
            "-c",
            r#"
import zipfile
with zipfile.ZipFile('fixture_dep-0.1.0-py3-none-any.whl', 'w') as z:
    z.writestr('fixture_dep.py', 'VALUE = 42\n')
    z.writestr('fixture_dep-0.1.0.dist-info/METADATA', 'Metadata-Version: 2.1\nName: fixture-dep\nVersion: 0.1.0\n')
    z.writestr('fixture_dep-0.1.0.dist-info/WHEEL', 'Wheel-Version: 1.0\nGenerator: fixture\nRoot-Is-Purelib: true\nTag: py3-none-any\n')
    z.writestr('fixture_dep-0.1.0.dist-info/RECORD', '')
"#,
        ],
    );
    std::fs::write(root.join("pyproject.toml"), "[project]\nname='environment-fixture'\nversion='0.1.0'\nrequires-python='>=3.11'\ndependencies=['fixture-dep']\n[tool.uv.sources]\nfixture-dep={path='fixture_dep-0.1.0-py3-none-any.whl'}\n").unwrap();
    std::fs::write(
        root.join("test_fixture.py"),
        "def test_dependency():\n    import fixture_dep\n    assert fixture_dep.VALUE == 42\n",
    )
    .unwrap();
    run(&root, &uv, &["lock", "--offline", "--no-python-downloads"]);
    let lock = std::fs::read(root.join("uv.lock")).unwrap();
    let manifest = std::fs::read(root.join("pyproject.toml")).unwrap();
    let (url, state, server) = start(&root).await;
    let initial = status(&url).await;
    let finding = &initial["findings"][0];
    assert_eq!(finding["state"], "environment-missing", "{finding:#}");
    assert!(
        !root.join(".venv").exists(),
        "checking created an environment"
    );
    assert_eq!(std::fs::read(root.join("uv.lock")).unwrap(), lock);
    let body = json!({"project":finding["project"],"manager":finding["manager"],"revision":finding["revision"],"action":finding["actions"][0]["id"]});
    let response = reqwest::Client::new()
        .post(format!("{url}/api/environments/actions"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let session: Value = response.json().await.unwrap();
    let terminal = state
        .terminals
        .get(session["id"].as_str().unwrap())
        .unwrap();
    tokio::time::timeout(Duration::from_secs(20), async {
        while terminal.exit_code().is_none() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(terminal.exit_code(), Some(0), "{}", terminal.screen_text());
    assert!(
        !terminal.attach().0.is_empty(),
        "installation output was discarded"
    );
    let ready = status(&url).await;
    assert_eq!(ready["findings"][0]["state"], "ready", "{ready:#}");
    let command = hickory_cli::serve::test_run::test_command(
        &root,
        &root.join("test_fixture.py"),
        "test_dependency",
        "python",
    )
    .unwrap();
    assert!(command.argv[0].contains(".venv"), "{command:?}");
    run(
        &root,
        Path::new(&command.argv[0]),
        &["-c", "import test_fixture; test_fixture.test_dependency()"],
    );
    assert_eq!(std::fs::read(root.join("uv.lock")).unwrap(), lock);
    assert_eq!(
        std::fs::read(root.join("pyproject.toml")).unwrap(),
        manifest
    );

    // A stale action must not launch again after the environment changes.
    let response = reqwest::Client::new()
        .post(format!("{url}/api/environments/actions"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 409);
    // An externally changed manifest produces its own lockfile diagnosis.
    std::fs::write(
        root.join("pyproject.toml"),
        String::from_utf8(manifest)
            .unwrap()
            .replace("dependencies=['fixture-dep']", "dependencies=[]"),
    )
    .unwrap();
    let stale = status(&url).await;
    assert_eq!(stale["findings"][0]["state"], "lock-stale", "{stale:#}");
    server.abort();
}

</storycode:copy>
<a id="f-61003cc8a070-action_routes_reject_undiscovered_paths_and_arbitrary_commands"></a>
### `action_routes_reject_undiscovered_paths_and_arbitrary_commands`

<storycode:copy id="f-61003cc8a070-action_routes_reject_undiscovered_paths_and_arbitrary_commands">
#[tokio::test]
async fn action_routes_reject_undiscovered_paths_and_arbitrary_commands() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let (url, _, server) = start(&root).await;
    let response = reqwest::Client::new()
        .post(format!("{url}/api/environments/actions"))
        .json(&json!({"project":"/tmp","revision":"anything","action":"rm -rf","manager":"uv"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 404);
    server.abort();
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_environments.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_environments.rs"><storycode:paste select="#f-61003cc8a070-start"/><storycode:paste select="#f-61003cc8a070-status"/><storycode:paste select="#f-61003cc8a070-run"/><storycode:paste select="#f-61003cc8a070-uv_sync_in_a_terminal_prepares_the_interpreter_used_by_project_tests"/><storycode:paste select="#f-61003cc8a070-action_routes_reject_undiscovered_paths_and_arbitrary_commands"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/environments/EnvironmentPanel.tsx](../../../mechanisms/apps/web/src/environments/EnvironmentPanel.tsx.md#output-a428d5761b36)

### Tests and fixtures

- [apps/web/src/environments/EnvironmentPanel.test.tsx](../../../mechanisms/apps/web/src/environments/EnvironmentPanel.tsx.md#output-f41d08c2423e)
- [apps/web/src/environments/useEnvironments.test.tsx](contract.md#output-b84eb9a7e17c)
- [crates/hick-project-env/tests/providers.rs](contract.md#output-5e0ba6cf199b)
- [crates/hickory-cli/tests/serve_environments.rs](contract.md#output-61003cc8a070)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
