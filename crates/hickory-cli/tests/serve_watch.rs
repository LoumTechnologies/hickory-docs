//! Read-only observation keeps external source saves visible without generating
//! outputs or automatically carrying external output edits into source.
//!
//! Protects docs/guarantees/collaboration/the-app-sees-external-edits.md

use std::path::PathBuf;
use std::time::Duration;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::watch;
use hickory_cli::serve::{LocalState, ServeOptions, prepare};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

async fn start(with_doc: bool) -> (LocalState, watch::WatchGuard, PathBuf, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    if with_doc {
        std::fs::write(dir.path().join("demo.md"), DOC).unwrap();
    }
    let root = dir.path().canonicalize().unwrap();

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

    let state = prepared.state.clone();
    let guard = watch::spawn(state.clone()).expect("watch starts");
    (state, guard, root, dir)
}

/// Poll until `check` holds or the deadline passes.
async fn eventually(what: &str, mut check: impl FnMut() -> bool) {
    for _ in 0..200 {
        if check() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for: {what}");
}

/// An empty folder is a session, not an error — the app's first-run state.
#[tokio::test]
async fn an_empty_folder_prepares_and_watches() {
    let (state, guard, root, _dir) = start(false).await;
    assert_eq!(state.index.entries().len(), 0);

    // A document that appears later — `touch`, git checkout, another program
    // — is picked up without a restart: indexed without generating.
    std::fs::write(root.join("late.md"), DOC).unwrap();
    eventually("the new document is indexed without generating", || {
        !state.index.entries().is_empty()
    })
    .await;

    assert!(!root.join("greet.rs").exists());
    guard.shutdown().await;
}

/// Observation preserves external output bytes and never amends its source.
#[tokio::test]
async fn a_generated_file_edit_is_preserved_without_automatic_amendment() {
    let (_state, guard, root, _dir) = start(true).await;
    let output = root.join("greet.rs");
    assert!(!output.exists());
    std::fs::write(&output, "external output edit").unwrap();
    std::fs::write(root.join("later.md"), DOC).unwrap();
    tokio::time::sleep(Duration::from_millis(450)).await;
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "external output edit"
    );
    assert_eq!(std::fs::read_to_string(root.join("demo.md")).unwrap(), DOC);
    guard.shutdown().await;
}
/// Disk observation must not replace a live buffer before the reader chooses.
#[tokio::test]
async fn an_external_doc_edit_reaches_the_live_room() {
    let (state, guard, root, _dir) = start(true).await;
    let (id, _) = state.index.sole().expect("one document");
    let room = state.rooms.get_or_create(&id).await.expect("room opens");
    let before = room.text().await;
    let updated = DOC.replace("# Demo", "# Demo, renamed outside the app");
    std::fs::write(root.join("demo.md"), &updated).unwrap();
    // Exercise the same observer directly: no arbitrary wait can prove that
    // a background replacement will never arrive.
    let woven = hickory_cli::up::state::WovenState::default();
    hickory_cli::serve::watch::reconcile_rooms(&state, &root, &woven).await;
    assert_eq!(room.text().await, before);
    assert_eq!(
        std::fs::read_to_string(root.join("demo.md")).unwrap(),
        updated
    );

    // The same conditional operation powers reload and merge: neither stale
    // disk observations nor stale live buffers authorize a replacement.
    use axum::{
        Json,
        extract::{Path, State},
    };
    use hickory_cli::serve::api::{SaveDoc, SaveDocExpected, put_doc};
    for (live, disk) in [
        (before.clone(), "stale disk".to_string()),
        ("stale buffer".to_string(), updated.clone()),
    ] {
        assert!(
            put_doc(
                State(state.clone()),
                Path(id.clone()),
                Json(SaveDoc {
                    source: "merged".to_string(),
                    expected: Some(SaveDocExpected { source: live, disk }),
                })
            )
            .await
            .is_err()
        );
        assert_eq!(room.text().await, before);
        assert_eq!(
            std::fs::read_to_string(root.join("demo.md")).unwrap(),
            updated
        );
    }
    let _ = put_doc(
        State(state.clone()),
        Path(id.clone()),
        Json(SaveDoc {
            source: "merged".to_string(),
            expected: Some(SaveDocExpected {
                source: before,
                disk: updated,
            }),
        }),
    )
    .await
    .expect("reviewed merge applies");
    assert_eq!(room.text().await, "merged");
    assert_eq!(
        std::fs::read_to_string(root.join("demo.md")).unwrap(),
        "merged"
    );
    guard.shutdown().await;
}
/// The rooms' own persist is not an external edit: reconciling it back in
/// would revert keystrokes typed since. The store's echo test is what the
/// watcher consults.
#[tokio::test]
async fn the_stores_own_persist_is_recognised_as_an_echo() {
    let (state, guard, _root, _dir) = start(true).await;
    let (id, _) = state.index.sole().expect("one document");

    assert!(!state.store.was_own_write(&id, "anything"));
    use hickory_collab::DocStore as _;
    state
        .store
        .save(&id, "persisted text", b"")
        .await
        .expect("save");
    assert!(state.store.was_own_write(&id, "persisted text"));
    assert!(
        !state
            .store
            .was_own_write(&id, "persisted text, then typed more")
    );

    guard.shutdown().await;
}
