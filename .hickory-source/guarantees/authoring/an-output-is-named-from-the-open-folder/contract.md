<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Output Is Named From The Open Folder

For: Authors editing executable documents and the files those documents produce.

Given a document anywhere under the open folder — `tools/app.md` writing
`<hick:file path="app.py">` — when the app lists its outputs, opens one in a
generated-file tab, carries an edit back through it, or reads it at a
commit, then the file is named the way the tree names it, relative to the
open folder (`tools/app.py`), on every route and in every answer; and a
request that spells it the document's way (`app.py`) is answered too,
under the folder-relative name.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-output-is-named-from-the-open-folder.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-6ec61b78f51d"></a>
## `crates/hickory-cli/tests/outputs_in_a_subfolder.rs`

Related contracts: [An Output Is Named From The Open Folder](contract.md).

<a id="f-6ec61b78f51d-ENV_LOCK"></a>
### `ENV_LOCK`

A document in a subfolder names its outputs the way the tree does.
Protects docs/guarantees/authoring/an-output-is-named-from-the-open-folder.md.

<storycode:copy id="f-6ec61b78f51d-ENV_LOCK">
//! A document in a subfolder names its outputs the way the tree does.
//!
//! Protects docs/guarantees/authoring/an-output-is-named-from-the-open-folder.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;
use std::sync::Arc;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

</storycode:copy>
<a id="f-6ec61b78f51d-a_generated_file_in_a_subfolder_is_found_by_its_folder_relative_path"></a>
### `a_generated_file_in_a_subfolder_is_found_by_its_folder_relative_path`

<storycode:copy id="f-6ec61b78f51d-a_generated_file_in_a_subfolder_is_found_by_its_folder_relative_path">
#[tokio::test]
async fn a_generated_file_in_a_subfolder_is_found_by_its_folder_relative_path() {
    let _env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("tools")).unwrap();
    std::fs::write(
        root.join("tools/app.md"),
        "# A tool\n\n<hick:file path=\"app.py\">\nprint(1)\n</hick:file>\n",
    )
    .unwrap();
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
    let doc_id = prepared.state.index.sole().expect("one document").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let base = format!("http://127.0.0.1:{port}");
    let get = |path: String| {
        let base = base.clone();
        async move {
            let res = reqwest::get(format!("{base}{path}")).await.unwrap();
            (
                res.status().as_u16(),
                res.json::<Value>().await.unwrap_or(Value::Null),
            )
        }
    };

    // Listed the way the tree names it.
    let (_, listed) = get(format!("/api/docs/{doc_id}/outputs")).await;
    let paths: Vec<&str> = listed["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"tools/app.py"), "{listed}");
    assert!(!paths.contains(&"tools/app.md"), "{listed}");

    // Found by that name, and answered under it.
    let (status, file) = get(format!("/api/docs/{doc_id}/outputs/file?path=tools/app.py")).await;
    assert_eq!(status, 200, "{file}");
    assert_eq!(file["path"], "tools/app.py");
    assert!(file["content"].as_str().unwrap().contains("print(1)"));

    // The document's own spelling still works — a `<hick:file path>` is
    // relative to the document, and that is not wrong, only local.
    let (status, same) = get(format!("/api/docs/{doc_id}/outputs/file?path=app.py")).await;
    assert_eq!(status, 200, "{same}");
    assert_eq!(same["path"], "tools/app.py");

    // And a miss lists what exists in the tree's spelling.
    let (status, miss) = get(format!(
        "/api/docs/{doc_id}/outputs/file?path=tools/nope.py"
    ))
    .await;
    assert_eq!(status, 404);
    assert!(
        miss["error"].as_str().unwrap().contains("tools/app.py"),
        "{miss}"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/outputs_in_a_subfolder.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/outputs_in_a_subfolder.rs"><storycode:paste select="#f-6ec61b78f51d-ENV_LOCK"/><storycode:paste select="#f-6ec61b78f51d-a_generated_file_in_a_subfolder_is_found_by_its_folder_relative_path"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)
- [crates/hickory-cli/src/serve/outputs.rs](../../../mechanisms/crates/hickory-cli/src/serve/outputs.rs.md#output-c3e5a56f4fd1)

### Tests and fixtures

- [crates/hickory-cli/tests/outputs_in_a_subfolder.rs](contract.md#output-6ec61b78f51d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
