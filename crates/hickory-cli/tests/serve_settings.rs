//! The Settings key routes, driven the way the desktop app's Settings page
//! drives them: `GET /api/settings/keys`, `PUT /api/settings/keys`, and then
//! the agent route picking the stored key up with no restart.
//!
//! Protects docs/guarantees/agent/keys-are-stored-locally-and-never-leave.md.
//!
//! No test here spends a token or leaves the machine: the environment's
//! provider variables are scrubbed up front (this binary contains only this
//! module, so the scrub races nothing), and `ANTHROPIC_BASE_URL` is pointed
//! at a closed loopback port so the one turn that does start fails instantly
//! against 127.0.0.1 instead of carrying a fabricated key to a real vendor.

use std::path::PathBuf;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

const KEY: &str = "sk-ant-stored-in-settings-000042";

struct Session {
    base: String,
    doc_id: String,
    key_path: PathBuf,
    state: hickory_cli::serve::LocalState,
    _dir: tempfile::TempDir,
}

/// Start a session persisting keys to `<tempdir>/config/llm-keys.json`, the
/// same shape the desktop app passes (`<app config dir>/llm-keys.json`).
async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let root = dir.path().canonicalize().unwrap();
    let key_path = root.join("config").join("llm-keys.json");

    let prepared = prepare(ServeOptions {
        target: root.join("demo.hick"),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: Some(key_path.clone()),
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");

    let state = prepared.state.clone();
    let doc_id = state.index.sole().expect("one document").0;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        doc_id,
        key_path,
        state,
        _dir: dir,
    }
}

async fn get(session: &Session, path: &str) -> (u16, String) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    (
        resp.status().as_u16(),
        resp.text().await.unwrap_or_default(),
    )
}

async fn put(session: &Session, path: &str, body: Value) -> (u16, String) {
    let resp = reqwest::Client::new()
        .put(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    (
        resp.status().as_u16(),
        resp.text().await.unwrap_or_default(),
    )
}

fn scrub_provider_env() {
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
}

/// The full Settings round trip: PUT a key, GET shows it configured and
/// masked (never in full), the file lands on disk owner-only, the in-memory
/// store serves the very next agent turn without a restart — and none of it
/// depends on a single environment variable.
#[tokio::test(flavor = "multi_thread")]
async fn a_saved_key_is_masked_private_and_live_without_restart() {
    scrub_provider_env();
    // The one agent turn this test starts must fail on this machine, not
    // reach a vendor with a fabricated key. Nothing listens on port 9.
    unsafe { std::env::set_var("ANTHROPIC_BASE_URL", "http://127.0.0.1:9") };

    let session = start().await;

    // Before anything is saved (and with the environment scrubbed): all
    // five providers listed, none configured — and the agent degrades to
    // its configuration note.
    let (status, body) = get(&session, "/api/settings/keys").await;
    assert_eq!(status, 200, "{body}");
    let listing: Value = serde_json::from_str(&body).unwrap();
    let providers = listing["providers"].as_array().unwrap();
    let ids: Vec<&str> = providers
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        ["anthropic", "openai", "deepseek", "xai", "openrouter"]
    );
    assert!(
        providers.iter().all(|p| p["configured"] == false),
        "{listing}"
    );
    let resp = reqwest::Client::new()
        .post(format!(
            "{}/api/docs/{}/agent",
            session.base, session.doc_id
        ))
        .json(&json!({ "prompt": "hello", "parent_id": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 503, "no key anywhere yet");

    // PUT the key. The response — like every response — must not carry it.
    let (status, body) = put(&session, "/api/settings/keys", json!({ "anthropic": KEY })).await;
    assert_eq!(status, 200, "{body}");
    assert!(
        !body.contains(KEY),
        "the key must never cross the wire: {body}"
    );
    let listing: Value = serde_json::from_str(&body).unwrap();
    let anthropic = listing["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "anthropic")
        .unwrap()
        .clone();
    assert_eq!(anthropic["configured"], true);
    assert_eq!(anthropic["masked"], "sk-a…42");

    // GET agrees, and still never echoes the key.
    let (status, body) = get(&session, "/api/settings/keys").await;
    assert_eq!(status, 200);
    assert!(!body.contains(KEY), "{body}");
    assert!(body.contains("sk-a…42"), "{body}");

    // The file exists where the desktop app would look, owner-only.
    let on_disk = std::fs::read_to_string(&session.key_path).expect("the key file persists");
    assert!(on_disk.contains(KEY), "the file is where the key lives");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&session.key_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "keys must not be group- or world-readable");
    }

    // The in-memory store took the key too…
    assert_eq!(
        session
            .state
            .keys
            .store
            .read()
            .unwrap()
            .key_for("anthropic")
            .as_deref(),
        Some(KEY)
    );
    // …and the store-aware resolution accepts on it with an empty
    // environment, which is exactly what the agent route now runs.
    let store = session.state.keys.store.read().unwrap().clone();
    assert_eq!(
        hickory_agent::resolve_selector_with_store(None, &store).unwrap(),
        "anthropic"
    );

    // The agent route no longer answers "no key": the turn is accepted (202,
    // not 503) with no restart in between. It then fails against the dead
    // loopback endpoint, which is the point — the key goes nowhere real.
    let resp = reqwest::Client::new()
        .post(format!(
            "{}/api/docs/{}/agent",
            session.base, session.doc_id
        ))
        .json(&json!({ "prompt": "hello", "parent_id": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        202,
        "the stored key must satisfy provider resolution: {}",
        resp.text().await.unwrap_or_default()
    );

    // Clearing with null removes it from the listing and the file alike.
    let (status, body) = put(&session, "/api/settings/keys", json!({ "anthropic": null })).await;
    assert_eq!(status, 200, "{body}");
    let listing: Value = serde_json::from_str(&body).unwrap();
    assert!(
        listing["providers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["configured"] == false),
        "{listing}"
    );
    let on_disk = std::fs::read_to_string(&session.key_path).unwrap();
    assert!(!on_disk.contains(KEY), "a cleared key must leave the disk");
}

/// An unknown provider id is refused naming the valid ones, and nothing —
/// not the store, not the file — changes, even for the valid entries that
/// arrived in the same request.
#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_provider_id_is_refused_and_changes_nothing() {
    let session = start().await;

    let (status, body) = put(
        &session,
        "/api/settings/keys",
        json!({ "openai": "sk-would-have-landed", "gemini": "sk-x" }),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    for name in ["anthropic", "openai", "deepseek", "grok", "openrouter"] {
        assert!(body.contains(name), "the error must name {name}: {body}");
    }
    assert!(!body.contains("sk-x"), "never echo key material: {body}");

    // All-or-nothing: the valid sibling entry did not land either.
    assert!(
        session
            .state
            .keys
            .store
            .read()
            .unwrap()
            .key_for("openai")
            .is_none()
    );
    assert!(!session.key_path.exists(), "nothing was persisted");
}
