<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/desktop/src-tauri/tests/first_folder.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The File Menu Preserves The Originating Window](../../../../../contract.md)

Captured SHA-256: `e7bdd2cc53e9057d0b629e58593dd05ef8c395911e682d95728ce3feac54c1ae`.

<book:entry path="apps/desktop/src-tauri/tests/first_folder.rs">
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
</book:entry>

</book:doc>
