<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A protected workspace save is validated against the revision it read

For: Authors using an agent to change documents while retaining review control.

Given an agent workspace frontend using `workspace_fs::Engine`, when a buffered
save replaces a source document or a generated text product, then the engine
checks the source revision and original bytes, maps product edits through the
existing lineage implementation, validates the candidate weave, and refuses a
save that cannot reproduce the proposed bytes before publishing the source.
The durable write and live-room update occur
under one room lock with a final source comparison; debounced room persistence
uses that lock too, so an older snapshot cannot finish saving over it.
A reverse save spanning several source documents is refused. Generated reads
are derived from the source view; they do not trust stale backing products.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-mounted-workspace-keeps-source-and-output-together.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-51472b1d3b69"></a>
## `crates/hickory-cli/tests/workspace_fs.rs`

Related contracts: [A protected workspace save is validated against the revision it read](contract.md).

<a id="f-51472b1d3b69-SOURCE"></a>
### `SOURCE`

<storycode:copy id="f-51472b1d3b69-SOURCE">
//! Guarantees: docs/guarantees/agent/the-mounted-workspace-keeps-source-and-output-together.md
use base64::Engine as _;
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare, workspace_fs::Engine},
};
use serde_json::{Value, json};

const SOURCE: &str = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
<hick:container name="local" image="alpine" />
<hick:file path="synthetic.txt"><hick:exec container="local">printf x</hick:exec></hick:file>
</hick:doc>
"##;
</storycode:copy>
<a id="f-51472b1d3b69-setup"></a>
### `setup`

<storycode:copy id="f-51472b1d3b69-setup">
async fn setup() -> (tempfile::TempDir, Engine, hickory_cli::serve::LocalState) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.md"), SOURCE).unwrap();
    let prepared = prepare(ServeOptions {
        target: dir.path().into(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let state = prepared.state;
    let engine = Engine::open(state.clone(), dir.path().join("access.md")).unwrap();
    (dir, engine, state)
}
</storycode:copy>
<a id="f-51472b1d3b69-open"></a>
### `open`

<storycode:copy id="f-51472b1d3b69-open">
async fn open(engine: &mut Engine, path: &str, write: bool) -> String {
    engine
        .request(&json!({"op":"open","path":path,"write":write}))
        .await
        .unwrap()["handle"]
        .as_str()
        .unwrap()
        .into()
}
</storycode:copy>
<a id="f-51472b1d3b69-read"></a>
### `read`

<storycode:copy id="f-51472b1d3b69-read">
async fn read(engine: &mut Engine, handle: &str) -> String {
    let r = engine
        .request(&json!({"op":"read","handle":handle,"offset":0,"length":99999}))
        .await
        .unwrap();
    String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(r["data"].as_str().unwrap())
            .unwrap(),
    )
    .unwrap()
}
</storycode:copy>
<a id="f-51472b1d3b69-replace"></a>
### `replace`

<storycode:copy id="f-51472b1d3b69-replace">
async fn replace(engine: &mut Engine, handle: &str, text: &str) {
    engine
        .request(&json!({"op":"truncate","handle":handle,"length":0}))
        .await
        .unwrap();
    engine.request(&json!({"op":"write","handle":handle,"offset":0,"data":base64::engine::general_purpose::STANDARD.encode(text)})).await.unwrap();
}
</storycode:copy>
<a id="f-51472b1d3b69-buffered_output_save_reverse_edits_source_and_live_room"></a>
### `buffered_output_save_reverse_edits_source_and_live_room`

<storycode:copy id="f-51472b1d3b69-buffered_output_save_reverse_edits_source_and_live_room">
#[tokio::test]
async fn buffered_output_save_reverse_edits_source_and_live_room() {
    let (dir, mut engine, state) = setup().await;
    let id = state.index.sole().unwrap().0;
    let room = state.rooms.get_or_create(&id).await.unwrap();
    let h = open(&mut engine, "greet.rs", true).await;
    let old = read(&mut engine, &h).await;
    assert!(old.contains("hello"));
    let after = old.replace("hello", "goodbye");
    replace(&mut engine, &h, &after).await;
    assert_eq!(
        std::fs::read_to_string(dir.path().join("demo.md")).unwrap(),
        SOURCE
    );
    engine
        .request(&json!({"op":"close","handle":h}))
        .await
        .unwrap();
    assert_eq!(room.text().await, SOURCE.replace("hello", "goodbye"));
    let next = open(&mut engine, "greet.rs", false).await;
    assert_eq!(read(&mut engine, &next).await, after);
    let session = std::fs::read_to_string(dir.path().join("access.md")).unwrap();
    assert!(session.contains("filesystem-access") && session.contains("filesystem-write"));
    assert!(
        !session.contains("<hick:read"),
        "filesystem accesses must not claim model context"
    );
}
</storycode:copy>
<a id="f-51472b1d3b69-invalid_source_and_stale_output_never_publish"></a>
### `invalid_source_and_stale_output_never_publish`

<storycode:copy id="f-51472b1d3b69-invalid_source_and_stale_output_never_publish">
#[tokio::test]
async fn invalid_source_and_stale_output_never_publish() {
    let (dir, mut engine, state) = setup().await;
    let source = open(&mut engine, "demo.md", true).await;
    replace(&mut engine, &source, "<hick:doc><hick:copy>").await;
    assert!(
        engine
            .request(&json!({"op":"close","handle":source}))
            .await
            .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("demo.md")).unwrap(),
        SOURCE
    );
    let output = open(&mut engine, "greet.rs", true).await;
    let old = read(&mut engine, &output).await;
    replace(&mut engine, &output, &old.replace("hello", "agent")).await;
    let id = state.index.sole().unwrap().0;
    state.rooms.get_or_create(&id).await.unwrap();
    state
        .rooms
        .apply_external_source(&id, &SOURCE.replace("hello", "person"))
        .await;
    assert!(
        engine
            .request(&json!({"op":"close","handle":output}))
            .await
            .unwrap_err()
            .to_string()
            .contains("changed")
    );
    assert!(
        state
            .rooms
            .get(&id)
            .await
            .unwrap()
            .text()
            .await
            .contains("person")
    );
}
</storycode:copy>
<a id="f-51472b1d3b69-temp_file_rename_uses_lineage_and_refuses_synthetic_bytes"></a>
### `temp_file_rename_uses_lineage_and_refuses_synthetic_bytes`

<storycode:copy id="f-51472b1d3b69-temp_file_rename_uses_lineage_and_refuses_synthetic_bytes">
#[tokio::test]
async fn temp_file_rename_uses_lineage_and_refuses_synthetic_bytes() {
    let (dir, mut engine, _) = setup().await;
    let out = open(&mut engine, "greet.rs", false).await;
    let text = read(&mut engine, &out).await.replace("hello", "renamed");
    engine
        .request(&json!({"op":"create","path":"edit.tmp","directory":false}))
        .await
        .unwrap();
    let temp = open(&mut engine, "edit.tmp", true).await;
    replace(&mut engine, &temp, &text).await;
    engine
        .request(&json!({"op":"close","handle":temp}))
        .await
        .unwrap();
    engine
        .request(&json!({"op":"rename","path":"edit.tmp","to":"greet.rs"}))
        .await
        .unwrap();
    assert!(!dir.path().join("edit.tmp").exists());
    assert!(
        std::fs::read_to_string(dir.path().join("demo.md"))
            .unwrap()
            .contains("renamed")
    );
    let reading = open(&mut engine, "synthetic.txt", true).await;
    replace(&mut engine, &reading, "invented replacement\n").await;
    assert!(
        engine
            .request(&json!({"op":"close","handle":reading}))
            .await
            .is_err()
    );
}
</storycode:copy>
<a id="f-51472b1d3b69-confines_paths_links_and_repository_history"></a>
### `confines_paths_links_and_repository_history`

<storycode:copy id="f-51472b1d3b69-confines_paths_links_and_repository_history">
#[tokio::test]
async fn confines_paths_links_and_repository_history() {
    let (dir, mut engine, _) = setup().await;
    for path in ["../outside", "/etc/passwd"] {
        assert!(
            engine
                .request(&json!({"op":"stat","path":path}))
                .await
                .is_err()
        );
    }
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(dir.path().join(".git/config"), "private config").unwrap();
    assert!(
        engine
            .request(&json!({"op":"open","path":".git/config","write":true}))
            .await
            .is_err()
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc/passwd", dir.path().join("escape")).unwrap();
        assert!(
            engine
                .request(&json!({"op":"open","path":"escape"}))
                .await
                .is_err()
        );
    }
    let entries: Value = engine
        .request(&json!({"op":"list","path":""}))
        .await
        .unwrap();
    assert!(
        entries
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["path"] == "greet.rs")
    );
}

</storycode:copy>
<a id="f-51472b1d3b69-text_frontend_shares_reverse_edits_and_records_actual_context"></a>
### `text_frontend_shares_reverse_edits_and_records_actual_context`

<storycode:copy id="f-51472b1d3b69-text_frontend_shares_reverse_edits_and_records_actual_context">
#[tokio::test]
async fn text_frontend_shares_reverse_edits_and_records_actual_context() {
    let (dir, mut engine, _) = setup().await;
    let shown = engine
        .request(&json!({"op":"read_text","path":"greet.rs"}))
        .await
        .unwrap();
    let after = shown["content"]
        .as_str()
        .unwrap()
        .replace("hello", "protocol");
    engine
        .request(&json!({"op":"write_text","path":"greet.rs","content":after}))
        .await
        .unwrap();
    let session = std::fs::read_to_string(dir.path().join("access.md")).unwrap();
    assert!(session.contains("<hick:read") && session.contains("<hick:wrote"));
    assert!(
        std::fs::read_to_string(dir.path().join("demo.md"))
            .unwrap()
            .contains("protocol")
    );
}

</storycode:copy>
<a id="f-51472b1d3b69-replacing_a_read_output_with_a_temp_file_refuses_concurrent_changes"></a>
### `replacing_a_read_output_with_a_temp_file_refuses_concurrent_changes`

<storycode:copy id="f-51472b1d3b69-replacing_a_read_output_with_a_temp_file_refuses_concurrent_changes">
#[tokio::test]
async fn replacing_a_read_output_with_a_temp_file_refuses_concurrent_changes() {
    let (dir, mut engine, state) = setup().await;
    let out = open(&mut engine, "greet.rs", false).await;
    let text = read(&mut engine, &out).await.replace("hello", "agent");
    std::fs::write(dir.path().join("save.tmp"), text).unwrap();
    let id = state.index.sole().unwrap().0;
    state.rooms.get_or_create(&id).await.unwrap();
    state
        .rooms
        .apply_external_source(&id, &SOURCE.replace("hello", "person"))
        .await;
    assert!(
        engine
            .request(&json!({"op":"rename","path":"save.tmp","to":"greet.rs"}))
            .await
            .is_err()
    );
    assert!(dir.path().join("save.tmp").exists());
    assert!(
        state
            .rooms
            .get(&id)
            .await
            .unwrap()
            .text()
            .await
            .contains("person")
    );
}

</storycode:copy>
<a id="f-51472b1d3b69-new_markdown_is_validated_before_publication_and_private_host_returns_errno"></a>
### `new_markdown_is_validated_before_publication_and_private_host_returns_errno`

<storycode:copy id="f-51472b1d3b69-new_markdown_is_validated_before_publication_and_private_host_returns_errno">
#[tokio::test]
async fn new_markdown_is_validated_before_publication_and_private_host_returns_errno() {
    let (dir, mut engine, _) = setup().await;
    engine
        .request(&json!({"op":"create","path":"new.md"}))
        .await
        .unwrap();
    let handle = open(&mut engine, "new.md", true).await;
    replace(&mut engine, &handle, "<hick:doc><hick:copy>").await;
    assert!(
        engine
            .request(&json!({"op":"close","handle":handle}))
            .await
            .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("new.md")).unwrap(),
        ""
    );
    assert!(
        engine
            .request(&json!({"op":"remove","path":""}))
            .await
            .is_err()
    );
    let host = hickory_cli::serve::workspace_fs::Host::start(engine)
        .await
        .unwrap();
    let reply: Value = reqwest::Client::new()
        .post(&host.url)
        .json(&json!({"op":"stat","path":"absent"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(reply["errno"], 2);
    assert!(reply["error"].as_str().unwrap().contains("No such file"));
}

</storycode:copy>
<a id="f-51472b1d3b69-final_publication_guard_refuses_a_newer_room_and_failed_persistence"></a>
### `final_publication_guard_refuses_a_newer_room_and_failed_persistence`

<storycode:copy id="f-51472b1d3b69-final_publication_guard_refuses_a_newer_room_and_failed_persistence">
#[tokio::test]
async fn final_publication_guard_refuses_a_newer_room_and_failed_persistence() {
    let (_dir, _engine, state) = setup().await;
    let id = state.index.sole().unwrap().0;
    let room = state.rooms.get_or_create(&id).await.unwrap();
    state
        .rooms
        .apply_external_source(&id, &SOURCE.replace("hello", "person"))
        .await;
    let called = std::sync::atomic::AtomicBool::new(false);
    let published = state
        .rooms
        .replace_source_if_current(&id, SOURCE, "agent", || {
            called.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })
        .await
        .unwrap();
    assert!(!published && !called.load(std::sync::atomic::Ordering::Relaxed));
    let current = room.text().await;
    assert!(
        state
            .rooms
            .replace_source_if_current(&id, &current, "agent", || anyhow::bail!("disk is full"))
            .await
            .is_err()
    );
    assert_eq!(room.text().await, current);
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/workspace_fs.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/workspace_fs.rs"><storycode:paste select="#f-51472b1d3b69-SOURCE"/><storycode:paste select="#f-51472b1d3b69-setup"/><storycode:paste select="#f-51472b1d3b69-open"/><storycode:paste select="#f-51472b1d3b69-read"/><storycode:paste select="#f-51472b1d3b69-replace"/><storycode:paste select="#f-51472b1d3b69-buffered_output_save_reverse_edits_source_and_live_room"/><storycode:paste select="#f-51472b1d3b69-invalid_source_and_stale_output_never_publish"/><storycode:paste select="#f-51472b1d3b69-temp_file_rename_uses_lineage_and_refuses_synthetic_bytes"/><storycode:paste select="#f-51472b1d3b69-confines_paths_links_and_repository_history"/><storycode:paste select="#f-51472b1d3b69-text_frontend_shares_reverse_edits_and_records_actual_context"/><storycode:paste select="#f-51472b1d3b69-replacing_a_read_output_with_a_temp_file_refuses_concurrent_changes"/><storycode:paste select="#f-51472b1d3b69-new_markdown_is_validated_before_publication_and_private_host_returns_errno"/><storycode:paste select="#f-51472b1d3b69-final_publication_guard_refuses_a_newer_room_and_failed_persistence"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/views/AgentSettings.tsx](../a-desktop-app-bundles-its-python-agent/contract.md#output-5b4b3eeb9113)
- [crates/hickory-cli/examples/fskit_bundle.rs](../../../mechanisms/crates/hickory-cli/examples/fskit_bundle.rs.md#output-fb890d5279a3)
- [crates/hickory-collab/src/lib.rs](../../../mechanisms/crates/hickory-collab/src/lib.rs.md#output-08bf73f4dffb)
- [scripts/dist-desktop.sh](../../../mechanisms/scripts/dist-desktop.sh.md#output-928dfd6fa282)

### Tests and fixtures

- [crates/hickory-cli/tests/workspace_fs.rs](contract.md#output-51472b1d3b69)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
