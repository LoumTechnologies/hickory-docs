//! The in-app agent, driven the way the desktop app's chat dock drives it:
//! `POST /api/docs/:id/agent`, a WebSocket on the run channel, and
//! `GET /api/docs/:id/agent/turns`.
//!
//! Protects docs/guarantees/agent/a-missing-key-degrades-to-a-note.md.
//!
//! No test here spends a token: the full-turn tests inject a
//! [`hickory_agent::ScriptedLlmClient`] through the serve state's test seam,
//! and the no-key test scrubs the provider variables from this test binary's
//! environment. The other tests never read those variables (the scripted
//! client bypasses provider resolution), so the scrub cannot race them.

use std::sync::Arc;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

struct Session {
    base: String,
    doc_id: String,
    state: hickory_cli::serve::LocalState,
    _dir: tempfile::TempDir,
}

async fn start() -> Session {
    start_with_folder(false).await
}

async fn start_with_folder(folder: bool) -> Session {
    start_session(folder, false).await
}

async fn start_session(folder: bool, editor_only: bool) -> Session {
    let dir = tempfile::tempdir().unwrap();
    if !editor_only {
        std::fs::write(dir.path().join("demo.md"), DOC).unwrap();
    }
    let root = dir.path().canonicalize().unwrap();

    let opts = ServeOptions {
        target: if folder || editor_only {
            root.clone()
        } else {
            root.join("demo.md")
        },
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    };
    let prepared = if editor_only {
        hickory_cli::serve::prepare_without_folder(opts).await
    } else {
        prepare(opts).await
    }
    .expect("session prepares");

    let state = prepared.state.clone();
    let doc_id = if editor_only {
        assert!(state.index.entries().is_empty());
        "workspace".into()
    } else {
        state.index.sole().expect("one document").0
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        doc_id,
        state,
        _dir: dir,
    }
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    (
        resp.status().as_u16(),
        resp.json().await.unwrap_or(Value::Null),
    )
}

async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    (
        resp.status().as_u16(),
        resp.json().await.unwrap_or(Value::Null),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn no_key_degrades_to_the_note_never_a_crash() {
    for var in [
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "DEEPSEEK_API_KEY",
        "XAI_API_KEY",
        "OPENROUTER_API_KEY",
        "HICKORY_LLM_PROVIDER",
    ] {
        unsafe { std::env::remove_var(var) };
    }

    let session = start().await;
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({ "prompt": "hello", "parent_id": null }),
    )
    .await;
    assert_eq!(status, 503, "{body}");
    let error = body["error"].as_str().expect("an {{error}} body");
    assert!(
        error.starts_with("agent not available"),
        "the client contract is the leading phrase: {error}"
    );
    // Degraded, not broken: the rest of the session still answers.
    let (status, _) = get(&session, "/api/health").await;
    assert_eq!(status, 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn an_untouched_document_lists_resolved_defaults_and_zero_totals() {
    let session = start().await;
    session
        .state
        .agent
        .set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new(["unused"])));

    let (status, listing) = get(
        &session,
        &format!("/api/docs/{}/agent/turns", session.doc_id),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(listing["provider"], "anthropic");
    assert_eq!(listing["model"], "claude-sonnet-5");
    assert_eq!(listing["totals"]["usd"], 0.0);
    assert_eq!(listing["totals"]["input"], 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn stopping_an_idle_document_names_the_situation() {
    let session = start().await;
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent/stop", session.doc_id),
        json!({}),
    )
    .await;
    assert_eq!(status, 409, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("no agent turn is running"),
        "{body}"
    );
}

// Guarantee: the Rust runtime has no fallback or compatibility mode.
#[tokio::test]
async fn removed_rust_backend_is_refused() {
    let session = start().await;
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({"prompt":"hello", "backend":"builtin"}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(body["error"].as_str().unwrap().contains("removed"));
}
