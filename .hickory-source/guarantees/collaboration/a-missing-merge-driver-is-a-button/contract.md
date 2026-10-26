<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Missing Merge Driver Is A Button, Not A Command To Go And Type

For: Engineers using local sessions or pairing their own machines.

Given a clone whose `.gitattributes` routes `*.md` at the hick merge
driver but whose git config does not define it, when the app opens that
folder and shows the banner saying so, then the banner carries a **Run
hick init** button that runs `hick init` on the folder inside the engine
— the same `run_init` the CLI runs: hook, `.gitignore`, `.gitattributes`,
the driver definition, the editor and agent files — and then says what it
wrote and that `.md` documents now merge through hick in this clone. If
it cannot (the folder is not a git work tree, git is missing), the warning
stays and the reason is beside it, in the engine's words.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-0e42ffa82b4e"></a>
## `crates/hickory-cli/tests/merge_driver_button.rs`

Related contracts: [A Missing Merge Driver Is A Button, Not A Command To Go And Type](contract.md).

<a id="f-0e42ffa82b4e-ENV_LOCK"></a>
### `ENV_LOCK`

The banner's "Run hick init" button, driven over real HTTP:
`POST /api/git/merge-driver` runs `hick init` in the engine and the
status the banner asked about comes back fixed.
Protects docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md.

<storycode:copy id="f-0e42ffa82b4e-ENV_LOCK">
//! The banner's "Run hick init" button, driven over real HTTP:
//! `POST /api/git/merge-driver` runs `hick init` in the engine and the
//! status the banner asked about comes back fixed.
//!
//! Protects docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;
use std::process::Command;
use std::sync::Arc;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

</storycode:copy>
<a id="f-0e42ffa82b4e-git"></a>
### `git`

<storycode:copy id="f-0e42ffa82b4e-git">
fn git(dir: &std::path::Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

</storycode:copy>
<a id="f-0e42ffa82b4e-the_button_runs_hick_init_and_the_driver_is_then_defined"></a>
### `the_button_runs_hick_init_and_the_driver_is_then_defined`

<storycode:copy id="f-0e42ffa82b4e-the_button_runs_hick_init_and_the_driver_is_then_defined">
#[tokio::test]
async fn the_button_runs_hick_init_and_the_driver_is_then_defined() {
    let _env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("note.md"), "# A note\n").unwrap();
    // The committed half without the per-clone half: routing, no driver.
    std::fs::write(root.join(".gitattributes"), "*.md merge=hick\n").unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-qm", "routing only"]);
    unsafe { std::env::set_var("HICKORY_STATE_DIR", state_dir.path()) };

    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let base = format!("http://127.0.0.1:{port}");

    let before: Value = reqwest::get(format!("{base}/api/git/merge-driver"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(before["ok"], false, "{before}");
    assert_eq!(before["status"]["configured"], false);

    let res = reqwest::Client::new()
        .post(format!("{base}/api/git/merge-driver"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let after: Value = res.json().await.unwrap();
    assert_eq!(after["ok"], true, "{after}");
    assert_eq!(after["changed"]["merge_driver"], true, "{after}");
    assert_eq!(after["status"]["configured"], true);

    // And the banner's own question now answers yes.
    let again: Value = reqwest::get(format!("{base}/api/git/merge-driver"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(again["ok"], true, "{again}");
}

</storycode:copy>
<a id="f-0e42ffa82b4e-a_folderless_session_neither_checks_nor_initializes_its_storage_repository"></a>
### `a_folderless_session_neither_checks_nor_initializes_its_storage_repository`

Internal storage can sit inside a repository without being an open folder.

<storycode:copy id="f-0e42ffa82b4e-a_folderless_session_neither_checks_nor_initializes_its_storage_repository">
// Internal storage can sit inside a repository without being an open folder.
// Guarantee: docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md
#[tokio::test]
async fn a_folderless_session_neither_checks_nor_initializes_its_storage_repository() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(
        dir.path(),
        &["config", "merge.hick.driver", "unused-driver"],
    );
    let storage = dir.path().join("unfiled");
    std::fs::create_dir(&storage).unwrap();
    let prepared = hickory_cli::serve::prepare_without_folder(ServeOptions {
        target: storage.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let http = reqwest::Client::new();
    let status: Value = http
        .get(format!("{base}/api/git/merge-driver"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["ok"], true, "{status}");
    assert_eq!(status["status"]["repository"], false, "{status}");
    let response = http
        .post(format!("{base}/api/git/merge-driver"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);
    assert!(response.text().await.unwrap().contains("Open a folder"));
    assert!(std::fs::read_dir(storage).unwrap().next().is_none());
    assert!(!dir.path().join(".gitattributes").exists());
    server.abort();
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/merge_driver_button.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/merge_driver_button.rs"><storycode:paste select="#f-0e42ffa82b4e-ENV_LOCK"/><storycode:paste select="#f-0e42ffa82b4e-git"/><storycode:paste select="#f-0e42ffa82b4e-the_button_runs_hick_init_and_the_driver_is_then_defined"/><storycode:paste select="#f-0e42ffa82b4e-a_folderless_session_neither_checks_nor_initializes_its_storage_repository"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/main.rs](../../../mechanisms/apps/desktop/src-tauri/src/main.rs.md#output-981e94b5aacd)
- [apps/web/src/api/client.ts](../../../mechanisms/apps/web/src/api/client.ts.md#output-1e909407d4cd)
- [apps/web/src/api/types.ts](../../../mechanisms/apps/web/src/api/types.ts.md#output-8a4c3188ff6c)
- [apps/web/src/components/MergeDriverNotice.tsx](../../../mechanisms/apps/web/src/components/MergeDriverNotice.tsx.md#output-c97d597c05fd)
- [crates/hickory-cli/src/merge_driver.rs](../../../mechanisms/crates/hickory-cli/src/merge_driver.rs.md#output-75494bcb5e9d)
- [crates/hickory-cli/src/serve/history.rs](../../../mechanisms/crates/hickory-cli/src/serve/history.rs.md#output-be949b7ea526)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)

### Tests and fixtures

- [apps/web/src/App.test.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-65ac24c24624)
- [apps/web/src/components/MergeDriverNotice.test.tsx](../../../mechanisms/apps/web/src/components/MergeDriverNotice.tsx.md#output-38c5ddfe6997)
- [crates/hickory-cli/tests/merge_driver_button.rs](contract.md#output-0e42ffa82b4e)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
