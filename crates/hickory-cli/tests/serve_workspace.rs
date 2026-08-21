//! What the window remembers: which tabs were open, and which buffers had
//! unsaved changes when it closed.
//!
//! Protects docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
//!
//! Driven the way the page drives it — real HTTP against the real router, no
//! in-process shortcut (`.instructions/framework-agnostic-system-tests.md`).
//! The state directory is named through `HICKORY_STATE_DIR` so this test can
//! prove a draft round-trips without writing into the developer's home.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo
</hick:doc>
"##;

struct Session {
    base: String,
    project: std::path::PathBuf,
    _dir: tempfile::TempDir,
}

/// One state directory for the whole binary, set once.
///
/// `HICKORY_STATE_DIR` is process-global, so a per-test directory would be a
/// race: these tests run concurrently, and whichever set the variable last
/// would own everybody's writes. Isolation comes from the store's own key
/// instead — each test gets its own project folder, and the store is keyed by
/// the project's canonical path, so two tests can never see each other's
/// layout even though they share a root.
fn state_dir() -> &'static std::path::Path {
    static STATE: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let dir = STATE.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        // SAFETY: written exactly once, inside OnceLock's initialisation, and
        // nothing else in this binary touches this variable.
        unsafe { std::env::set_var(hickory_workspace::STATE_DIR_VAR, dir.path()) };
        dir
    });
    dir.path()
}

async fn start() -> Session {
    let _ = state_dir();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let root = dir.path().canonicalize().unwrap();

    let prepared = prepare(ServeOptions {
        target: root.join("demo.hick"),
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

    Session {
        base: format!("http://127.0.0.1:{port}"),
        project: root,
        _dir: dir,
    }
}

async fn send(
    session: &Session,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> (u16, Value) {
    let mut req = reqwest::Client::new().request(method, format!("{}{path}", session.base));
    if let Some(body) = body {
        req = req.json(&body);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    send(session, reqwest::Method::GET, path, None).await
}

async fn put(session: &Session, path: &str, body: Value) -> (u16, Value) {
    send(session, reqwest::Method::PUT, path, Some(body)).await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_window_layout_round_trips() {
    let session = start().await;

    let (status, body) = get(&session, "/api/workspace/ui").await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "state": null }),
        "a fresh project remembers nothing"
    );

    let layout = json!({
        "panes": [{ "tabs": ["demo.hick"], "active": 0 }],
        "wrap": { "demo.hick": 72 },
    });
    let (status, _) = put(&session, "/api/workspace/ui", json!({ "state": layout })).await;
    assert_eq!(status, 200);

    let (_, body) = get(&session, "/api/workspace/ui").await;
    assert_eq!(body, json!({ "state": layout }));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_draft_comes_back_with_the_bytes_it_was_taken_from() {
    // The base is the whole reason a restored draft can be MERGED rather than
    // fought over: it is the common ancestor of the buffer and whatever the
    // file says now.
    let session = start().await;

    let (status, _) = put(
        &session,
        "/api/workspace/drafts",
        json!({
            "path": "demo.hick",
            "contents": "half a sentence",
            "base": "# Demo\n",
            "saved_at": 1_700_000_000_000u64,
        }),
    )
    .await;
    assert_eq!(status, 200);

    let (status, body) = get(&session, "/api/workspace/drafts").await;
    assert_eq!(status, 200);
    let drafts = body["drafts"].as_array().expect("a list of drafts");
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0]["path"], "demo.hick");
    assert_eq!(drafts[0]["contents"], "half a sentence");
    assert_eq!(drafts[0]["base"], "# Demo\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn saving_the_file_throws_its_draft_away() {
    let session = start().await;
    put(
        &session,
        "/api/workspace/drafts",
        json!({ "path": "demo.hick", "contents": "x", "base": "", "saved_at": 1 }),
    )
    .await;

    let (status, _) = send(
        &session,
        reqwest::Method::DELETE,
        "/api/workspace/drafts?path=demo.hick",
        None,
    )
    .await;
    assert_eq!(status, 200);

    let (_, body) = get(&session, "/api/workspace/drafts").await;
    assert_eq!(body["drafts"].as_array().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn discarding_a_draft_that_is_not_there_is_not_an_error() {
    // The page discards on every save, and most saves have nothing to discard.
    let session = start().await;
    let (status, _) = send(
        &session,
        reqwest::Method::DELETE,
        "/api/workspace/drafts?path=never-existed.hick",
        None,
    )
    .await;
    assert_eq!(status, 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn nothing_is_written_inside_the_project_folder() {
    // The guarantee this whole store exists for. A draft is unfinished work
    // its author has not decided to keep; a `.gitignore` entry is a promise
    // this tool cannot keep on somebody else's machine.
    let session = start().await;
    put(
        &session,
        "/api/workspace/ui",
        json!({ "state": { "panes": [] } }),
    )
    .await;
    put(
        &session,
        "/api/workspace/drafts",
        json!({ "path": "demo.hick", "contents": "unsaved", "base": "", "saved_at": 1 }),
    )
    .await;

    let names: Vec<String> = std::fs::read_dir(&session.project)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["demo.hick".to_string()],
        "the project folder grew something: {names:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_layout_carrying_a_document_is_refused_with_a_reason() {
    // The failure mode this guards against is somebody stuffing buffer
    // contents into the layout blob, where it would be reloaded on every
    // start and never merged against anything.
    let session = start().await;
    let huge = "x".repeat(1024 * 1024 + 1);
    let (status, body) = put(
        &session,
        "/api/workspace/ui",
        json!({ "state": { "oops": huge } }),
    )
    .await;
    assert_eq!(status, 422);
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("draft store"),
        "the message must say where this belongs instead: {body}"
    );
}
