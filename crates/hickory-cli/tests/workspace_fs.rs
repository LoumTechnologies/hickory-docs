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
    let engine = Engine::open(state.clone(), dir.path().join("access.hick")).unwrap();
    (dir, engine, state)
}
async fn open(engine: &mut Engine, path: &str, write: bool) -> String {
    engine
        .request(&json!({"op":"open","path":path,"write":write}))
        .await
        .unwrap()["handle"]
        .as_str()
        .unwrap()
        .into()
}
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
async fn replace(engine: &mut Engine, handle: &str, text: &str) {
    engine
        .request(&json!({"op":"truncate","handle":handle,"length":0}))
        .await
        .unwrap();
    engine.request(&json!({"op":"write","handle":handle,"offset":0,"data":base64::engine::general_purpose::STANDARD.encode(text)})).await.unwrap();
}
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
    let session = std::fs::read_to_string(dir.path().join("access.hick")).unwrap();
    assert!(session.contains("filesystem-access") && session.contains("filesystem-write"));
    assert!(
        !session.contains("<hick:read"),
        "filesystem accesses must not claim model context"
    );
}
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
    let session = std::fs::read_to_string(dir.path().join("access.hick")).unwrap();
    assert!(session.contains("<hick:read") && session.contains("<hick:wrote"));
    assert!(
        std::fs::read_to_string(dir.path().join("demo.md"))
            .unwrap()
            .contains("protocol")
    );
}

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
