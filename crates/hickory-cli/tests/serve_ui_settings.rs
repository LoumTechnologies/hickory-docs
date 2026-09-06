//! The Settings UI routes, driven the way the desktop app's Settings page
//! drives them: `GET /api/settings/ui`, `PUT /api/settings/ui`, and the
//! persisted `ui.json` a fresh session — i.e. the desktop shell at its next
//! launch — reads back. Mirrors `serve_settings.rs`, which does the same for
//! the provider-key routes.

use std::path::PathBuf;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, UiStore, prepare};
use serde_json::{Value, json};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

struct Session {
    base: String,
    ui_path: PathBuf,
    _dir: tempfile::TempDir,
}

/// Start a session persisting UI settings to `<tempdir>/config/ui.json`, the
/// same shape the desktop app passes (`<app config dir>/ui.json`).
async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let root = dir.path().canonicalize().unwrap();
    let ui_path = root.join("config").join("ui.json");

    let prepared = prepare(ServeOptions {
        target: root.join("demo.hick"),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: Some(ui_path.clone()),
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
        ui_path,
        _dir: dir,
    }
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    (status, serde_json::from_str(&body).unwrap_or(Value::Null))
}

async fn put(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .put(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    (status, serde_json::from_str(&body).unwrap_or(Value::Null))
}

/// The full round trip: a fresh session answers the default, a PUT sets the
/// title with no restart, the file lands on disk, and a NEW session over the
/// same path — the desktop shell's next launch — reads the title back.
#[tokio::test(flavor = "multi_thread")]
async fn a_saved_window_title_is_live_and_survives_a_restart() {
    let session = start().await;

    let (status, body) = get(&session, "/api/settings/ui").await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "window_title": null, "format_on_save": false, "keymap": null, "native_accelerators": {} })
    );

    let (status, body) = put(
        &session,
        "/api/settings/ui",
        json!({ "window_title": "My Lab Notebook" }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "window_title": "My Lab Notebook", "format_on_save": false, "keymap": null, "native_accelerators": {} })
    );

    // Live immediately, no restart.
    let (status, body) = get(&session, "/api/settings/ui").await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "window_title": "My Lab Notebook", "format_on_save": false, "keymap": null, "native_accelerators": {} })
    );

    // Persisted where the desktop shell reads it at launch.
    let on_disk = UiStore::load(&session.ui_path).expect("ui.json parses");
    assert_eq!(on_disk.window_title.as_deref(), Some("My Lab Notebook"));

    // A fresh session over the same file — the next launch — sees it too.
    let reloaded = prepare(ServeOptions {
        target: session._dir.path().join("demo.hick"),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: Some(session.ui_path.clone()),
    })
    .await
    .expect("session reloads");
    let store = reloaded.state.ui.store.read().unwrap();
    assert_eq!(store.window_title.as_deref(), Some("My Lab Notebook"));
}

/// Clearing: an explicit null clears, and a blank string means "no custom
/// title" rather than a title of nothing.
#[tokio::test(flavor = "multi_thread")]
async fn null_or_blank_clears_the_custom_title() {
    let session = start().await;

    let (_, _) = put(
        &session,
        "/api/settings/ui",
        json!({ "window_title": "Custom" }),
    )
    .await;
    let (status, body) = put(
        &session,
        "/api/settings/ui",
        json!({ "window_title": null }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "window_title": null, "format_on_save": false, "keymap": null, "native_accelerators": {} })
    );

    let (_, _) = put(
        &session,
        "/api/settings/ui",
        json!({ "window_title": "Custom" }),
    )
    .await;
    let (status, body) = put(
        &session,
        "/api/settings/ui",
        json!({ "window_title": "   " }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "window_title": null, "format_on_save": false, "keymap": null, "native_accelerators": {} })
    );

    let on_disk = UiStore::load(&session.ui_path).expect("ui.json parses");
    assert_eq!(on_disk.window_title, None);
}

/// Bad bodies are refused whole, and refuse to change anything: a non-object
/// body, a wrong-typed title, and an unknown field all 400 while the stored
/// setting stays exactly as it was.
#[tokio::test(flavor = "multi_thread")]
async fn a_bad_put_changes_nothing() {
    let session = start().await;
    let (_, _) = put(
        &session,
        "/api/settings/ui",
        json!({ "window_title": "Keep me" }),
    )
    .await;

    for bad in [
        json!("just a string"),
        json!({ "window_title": 42 }),
        json!({ "font_size": 12 }),
    ] {
        let (status, _) = put(&session, "/api/settings/ui", bad).await;
        assert_eq!(status, 400);
        let (_, body) = get(&session, "/api/settings/ui").await;
        assert_eq!(
            body,
            json!({ "window_title": "Keep me", "format_on_save": false, "keymap": null, "native_accelerators": {} })
        );
    }
}

/// The CLI's mode: no path means the routes still answer, in memory only —
/// a PUT works for the session and no file appears anywhere.
#[tokio::test(flavor = "multi_thread")]
async fn without_a_path_the_routes_answer_in_memory() {
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
    let session = Session {
        base: format!("http://127.0.0.1:{port}"),
        ui_path: root.join("config").join("ui.json"),
        _dir: dir,
    };

    let (status, body) = put(
        &session,
        "/api/settings/ui",
        json!({ "window_title": "Ephemeral" }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "window_title": "Ephemeral", "format_on_save": false, "keymap": null, "native_accelerators": {} })
    );
    assert!(
        !session.ui_path.exists(),
        "no file may appear without a path"
    );
}

// docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md: the
// keymap and the resolved menu accelerators ride the same file as the title.
#[tokio::test]
async fn the_keymap_and_native_accelerators_round_trip() {
    let session = start().await;
    let (status, body) = put(
        &session,
        "/api/settings/ui",
        json!({
            "keymap": { "profile": "jetbrains", "overrides": { "editor.format": "Ctrl+Alt+L" } },
            "native_accelerators": { "save": "CmdOrCtrl+S", "save-all": null, "settings": "CmdOrCtrl+Alt+S" }
        }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["keymap"]["profile"], "jetbrains");
    assert_eq!(body["keymap"]["overrides"]["editor.format"], "Ctrl+Alt+L");
    assert_eq!(body["native_accelerators"]["save"], "CmdOrCtrl+S");
    assert!(
        body["native_accelerators"].get("save-all").is_none(),
        "null drops the entry: {body}"
    );

    let (_, again) = get(&session, "/api/settings/ui").await;
    assert_eq!(again["keymap"]["profile"], "jetbrains");
    assert_eq!(again["native_accelerators"]["settings"], "CmdOrCtrl+Alt+S");
    let stored = UiStore::load(&session.ui_path).unwrap();
    assert_eq!(
        stored.native_accelerators.get("save").map(String::as_str),
        Some("CmdOrCtrl+S")
    );

    let (status, body) = put(&session, "/api/settings/ui", json!({ "keymap": "vscode" })).await;
    assert_eq!(status, 400, "{body}");
    let (status, _) = put(&session, "/api/settings/ui", json!({ "keymap": null })).await;
    assert_eq!(status, 200);
    let (_, cleared) = get(&session, "/api/settings/ui").await;
    assert!(cleared["keymap"].is_null());
}
