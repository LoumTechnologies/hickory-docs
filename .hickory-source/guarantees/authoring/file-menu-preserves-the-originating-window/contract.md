<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The File Menu Preserves The Originating Window

For: Authors editing executable documents and the files those documents produce.

Given a Hickory Docs window with no folder open, File → Open Folder or a
recent folder attaches the selected folder to that same window. The native
window and editor origin stay in place; existing tabs, unsaved unnamed notes,
recovery drafts, and explicitly opened files stay available. Files appears
beside the editors. This also applies to a window launched on a single file.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/file-menu-preserves-the-originating-window.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-9298d77f02d5"></a>
## `apps/desktop/src-tauri/tests/first_folder.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

Related contracts: [The File Menu Preserves The Originating Window](contract.md).

<a id="f-9298d77f02d5-the_first_folder_keeps_the_editor_origin_and_existing_work"></a>
### `the_first_folder_keeps_the_editor_origin_and_existing_work`

<storycode:copy id="f-9298d77f02d5-the_first_folder_keeps_the_editor_origin_and_existing_work">
//! Guarantee: docs/guarantees/authoring/file-menu-preserves-the-originating-window.md
use hickory_desktop_lib::server;
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn the_first_folder_keeps_the_editor_origin_and_existing_work() {
    let config = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(folder.path().join("new.md"), "# Selected folder").unwrap();
    let markdown = outside.path().canonicalize().unwrap().join("existing.md");
    let text = outside.path().canonicalize().unwrap().join("existing.txt");
    std::fs::write(&markdown, "# Existing note").unwrap();
    std::fs::write(&text, "Existing text").unwrap();
    let session = server::start_blank_with_config(Some(config.path()), None)
        .await
        .unwrap();
    let http = reqwest::Client::new();
    let url = &session.url;
    for path in [&markdown, &text] {
        assert!(
            http.post(format!("{url}/api/open-file"))
                .json(&json!({"path":path}))
                .send()
                .await
                .unwrap()
                .status()
                .is_success()
        );
    }
    let opened: Value = http
        .post(format!("{url}/api/open-file"))
        .json(&json!({"path":markdown}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let old_id = opened["doc_id"].as_str().unwrap();
    let draft =
        json!({"path":"untitled:original", "contents":"Unsaved note", "base":"", "saved_at":1});
    assert!(
        http.put(format!("{url}/api/workspace/drafts"))
            .json(&draft)
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    let layout = json!({"layout":{"original_tabs":[old_id, "untitled:original"]}});
    assert!(
        http.put(format!("{url}/api/workspace/ui"))
            .json(&json!({"state":layout}))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );

    // A failed selection changes nothing and does not consume the blank window.
    assert!(
        session
            .attach_folder(
                &folder.path().join("missing"),
                Some(config.path()),
                None,
                None
            )
            .await
            .is_err()
    );
    assert!(
        session
            .attach_folder(folder.path(), Some(config.path()), None, None)
            .await
            .unwrap()
    );
    assert_eq!(&session.ui_url, url);
    let files: Value = http
        .get(format!("{url}/api/files"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(files["folder_open"], true);
    assert_eq!(
        files["root_path"],
        folder
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
    let ui: Value = http
        .get(format!("{url}/api/workspace/ui"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(ui["state"], layout);
    let drafts: Value = http
        .get(format!("{url}/api/workspace/drafts"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(drafts["drafts"], json!([draft]));
    let old_doc: Value = http
        .get(format!("{url}/api/docs/{old_id}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(old_doc["source"], "# Existing note");
    assert!(
        http.put(format!("{url}/api/docs/{old_id}"))
            .json(&json!({"source":"# Edited existing note"}))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    assert_eq!(
        std::fs::read_to_string(&markdown).unwrap(),
        "# Edited existing note"
    );
    let unselected = outside
        .path()
        .canonicalize()
        .unwrap()
        .join("unselected.txt");
    std::fs::write(&unselected, "Not selected").unwrap();
    assert!(
        !http
            .get(format!("{url}/api/file"))
            .query(&[("path", &unselected)])
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    let plain: Value = http
        .get(format!("{url}/api/file"))
        .query(&[("path", &text)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(plain["content"], "Existing text");
    assert!(
        http.put(format!("{url}/api/file"))
            .json(&json!({"path":text,"content":"Updated text","base_hash":plain["hash"]}))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    assert_eq!(std::fs::read_to_string(&text).unwrap(), "Updated text");
    let new_doc: Value = http
        .post(format!("{url}/api/open-file"))
        .json(&json!({"path":folder.path().join("new.md")}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        http.get(format!(
            "{url}/api/docs/{}",
            new_doc["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap()
        .status()
        .is_success()
    );
    // A second folder must be handed to another window by the native shell.
    assert!(
        !session
            .attach_folder(outside.path(), Some(config.path()), None, None)
            .await
            .unwrap()
    );
}

</storycode:copy>
<a id="f-9298d77f02d5-a_window_launched_on_a_file_can_also_accept_its_first_folder"></a>
### `a_window_launched_on_a_file_can_also_accept_its_first_folder`

<storycode:copy id="f-9298d77f02d5-a_window_launched_on_a_file_can_also_accept_its_first_folder">
#[tokio::test(flavor = "multi_thread")]
async fn a_window_launched_on_a_file_can_also_accept_its_first_folder() {
    let outside = tempfile::tempdir().unwrap();
    let folder = tempfile::tempdir().unwrap();
    let file = outside.path().join("note.md");
    std::fs::write(&file, "# Open file").unwrap();
    let session = server::start(&file, None, None).await.unwrap();
    assert!(
        session
            .attach_folder(folder.path(), None, None, None)
            .await
            .unwrap()
    );
    let http = reqwest::Client::new();
    let files: Value = http
        .get(format!("{}/api/files", session.url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(files["folder_open"], true);
}
</storycode:copy>

### Assemble `apps/desktop/src-tauri/tests/first_folder.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/tests/first_folder.rs"><storycode:paste select="#f-9298d77f02d5-the_first_folder_keeps_the_editor_origin_and_existing_work"/><storycode:paste select="#f-9298d77f02d5-a_window_launched_on_a_file_can_also_accept_its_first_folder"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/views/folderPane.ts](../../../mechanisms/apps/web/src/views/folderPane.ts.md#output-c66b482f3cc6)
- [crates/hickory-cli/src/serve/plain_file.rs](../../../mechanisms/crates/hickory-cli/src/serve/plain_file.rs.md#output-128423f88fa6)

### Tests and fixtures

- [apps/desktop/src-tauri/tests/first_folder.rs](contract.md#output-9298d77f02d5)
- [apps/web/src/App.test.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-65ac24c24624)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
