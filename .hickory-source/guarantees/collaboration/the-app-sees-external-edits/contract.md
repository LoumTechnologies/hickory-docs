<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The App Runs The Up-Loop: External Edits Are Seen, Never Clobbered

For: Engineers using local sessions or pairing their own machines.

Given the desktop app open on a folder, when any other program — vim, a
formatter, a coding agent, `git checkout` — changes a `.md` document on
disk, then the live editor buffer follows the file (the change reconciles
into the document's room); when such a program saves an edit in a generated
output file, then the edit is carried back into the document it came from,
exactly as headless `hick up` would carry it; and when a new `.md` file
appears, then it is indexed, woven, and openable without restarting the app.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/the-app-sees-external-edits.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-9de73ad7ae6f"></a>
## `crates/hickory-cli/tests/serve_watch.rs`

Related contracts: [The App Runs The Up-Loop: External Edits Are Seen, Never Clobbered](contract.md).

<a id="f-9de73ad7ae6f-DOC"></a>
### `DOC`

The in-app up-loop: the watcher the desktop app runs beside the rooms.
`local-only.md` names the design this protects: the app and the working
tree are two writers on the same files, so external edits must reconcile
into the live editor, and edits saved in generated files must land back
in their documents — while the app is open, without restarting anything.
Protects docs/guarantees/collaboration/the-app-sees-external-edits.md

<storycode:copy id="f-9de73ad7ae6f-DOC">
//! The in-app up-loop: the watcher the desktop app runs beside the rooms.
//!
//! `local-only.md` names the design this protects: the app and the working
//! tree are two writers on the same files, so external edits must reconcile
//! into the live editor, and edits saved in generated files must land back
//! in their documents — while the app is open, without restarting anything.
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

</storycode:copy>
<a id="f-9de73ad7ae6f-start"></a>
### `start`

<storycode:copy id="f-9de73ad7ae6f-start">
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

</storycode:copy>
<a id="f-9de73ad7ae6f-eventually"></a>
### `eventually`

Poll until `check` holds or the deadline passes.

<storycode:copy id="f-9de73ad7ae6f-eventually">
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

</storycode:copy>
<a id="f-9de73ad7ae6f-an_empty_folder_prepares_and_watches"></a>
### `an_empty_folder_prepares_and_watches`

An empty folder is a session, not an error — the app's first-run state.

<storycode:copy id="f-9de73ad7ae6f-an_empty_folder_prepares_and_watches">
/// An empty folder is a session, not an error — the app's first-run state.
#[tokio::test]
async fn an_empty_folder_prepares_and_watches() {
    let (state, guard, root, _dir) = start(false).await;
    assert_eq!(state.index.entries().len(), 0);

    // A document that appears later — `touch`, git checkout, another program
    // — is picked up without a restart: indexed and woven.
    std::fs::write(root.join("late.md"), DOC).unwrap();
    eventually("the new document is indexed and woven", || {
        !state.index.entries().is_empty() && root.join("greet.rs").is_file()
    })
    .await;

    guard.shutdown().await;
}

</storycode:copy>
<a id="f-9de73ad7ae6f-a_generated_file_edit_lands_back_in_the_document"></a>
### `a_generated_file_edit_lands_back_in_the_document`

The initial pass weaves what is on disk, and an edit saved in a generated
file lands back in the document — the up-loop, inside the app process.

<storycode:copy id="f-9de73ad7ae6f-a_generated_file_edit_lands_back_in_the_document">
/// The initial pass weaves what is on disk, and an edit saved in a generated
/// file lands back in the document — the up-loop, inside the app process.
#[tokio::test]
async fn a_generated_file_edit_lands_back_in_the_document() {
    let (_state, guard, root, _dir) = start(true).await;

    let output = root.join("greet.rs");
    eventually("the initial weave writes greet.rs", || output.is_file()).await;

    let text = std::fs::read_to_string(&output).unwrap();
    // The file can be read-only while the loop runs; write the way an editor
    // with `:w!` would.
    let edited = text.replace("hello", "howdy");
    let perms = std::fs::metadata(&output).unwrap().permissions();
    let mut writable = perms.clone();
    #[allow(clippy::permissions_set_readonly_false)]
    writable.set_readonly(false);
    std::fs::set_permissions(&output, writable).unwrap();
    std::fs::write(&output, &edited).unwrap();

    eventually("the edit reaches demo.md", || {
        std::fs::read_to_string(root.join("demo.md"))
            .unwrap_or_default()
            .contains("howdy")
    })
    .await;

    guard.shutdown().await;
}

</storycode:copy>
<a id="f-9de73ad7ae6f-an_external_doc_edit_reaches_the_live_room"></a>
### `an_external_doc_edit_reaches_the_live_room`

An external edit to a document reconciles into its live room, so the
editor buffer follows the file instead of clobbering it on the next
persist.

<storycode:copy id="f-9de73ad7ae6f-an_external_doc_edit_reaches_the_live_room">
/// An external edit to a document reconciles into its live room, so the
/// editor buffer follows the file instead of clobbering it on the next
/// persist.
#[tokio::test]
async fn an_external_doc_edit_reaches_the_live_room() {
    let (state, guard, root, _dir) = start(true).await;
    eventually("initial weave", || root.join("greet.rs").is_file()).await;

    let (id, _) = state.index.sole().expect("one document");
    let room = state.rooms.get_or_create(&id).await.expect("room opens");
    assert!(room.text().await.contains("hello"));

    let updated = DOC.replace("# Demo", "# Demo, renamed outside the app");
    std::fs::write(root.join("demo.md"), &updated).unwrap();

    eventually("the room follows the file", || {
        // `text()` is async; sample through a blocking handle each poll.
        let room = room.clone();
        futures::executor::block_on(async { room.text().await.contains("renamed outside") })
    })
    .await;

    guard.shutdown().await;
}

</storycode:copy>
<a id="f-9de73ad7ae6f-the_stores_own_persist_is_recognised_as_an_echo"></a>
### `the_stores_own_persist_is_recognised_as_an_echo`

The rooms' own persist is not an external edit: reconciling it back in
would revert keystrokes typed since. The store's echo test is what the
watcher consults.

<storycode:copy id="f-9de73ad7ae6f-the_stores_own_persist_is_recognised_as_an_echo">
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
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_watch.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_watch.rs"><storycode:paste select="#f-9de73ad7ae6f-DOC"/><storycode:paste select="#f-9de73ad7ae6f-start"/><storycode:paste select="#f-9de73ad7ae6f-eventually"/><storycode:paste select="#f-9de73ad7ae6f-an_empty_folder_prepares_and_watches"/><storycode:paste select="#f-9de73ad7ae6f-a_generated_file_edit_lands_back_in_the_document"/><storycode:paste select="#f-9de73ad7ae6f-an_external_doc_edit_reaches_the_live_room"/><storycode:paste select="#f-9de73ad7ae6f-the_stores_own_persist_is_recognised_as_an_echo"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/server.rs](../../../mechanisms/apps/desktop/src-tauri/src/server.rs.md#output-821f8d0a36be)
- [crates/hickory-cli/src/serve/store.rs](../../../mechanisms/crates/hickory-cli/src/serve/store.rs.md#output-60e6f71bfd5c)
- [crates/hickory-cli/src/serve/watch.rs](../../../mechanisms/crates/hickory-cli/src/serve/watch.rs.md#output-c9fa53b5e3bf)

### Tests and fixtures

- [crates/hickory-cli/tests/serve_local.rs](../../../mechanisms/crates/hickory-cli/tests/serve_local.rs.md#output-2b0b4fa19188)
- [crates/hickory-cli/tests/serve_watch.rs](contract.md#output-9de73ad7ae6f)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
