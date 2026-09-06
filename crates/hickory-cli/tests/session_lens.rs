//! The session view carries what the lens draws — the conversation as
//! blocks — and what each element declared about where it came from.
//!
//! Protects docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;
use std::sync::Arc;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

const SESSION: &str = r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-09-05T00:00:00Z" doc="notes/today.hick">
<hick:user turn="t1">Why is checkout slow?</hick:user>
<hick:assistant>
<hick:tool name="read_doc">
<hick:arg name="doc">meetings/sync.hick</hick:arg>
</hick:tool>
</hick:assistant>
<hick:tool-result name="read_doc" ok="true">
…
</hick:tool-result>
<hick:read file="data/latency.csv" sha256="ff" lines="1-8"/>
<hick:assistant>
It slowed after the pool change; see [the sync](meetings/sync.hick#L12-L20).
</hick:assistant>
<hick:wrote file="notes/today.hick" lines="72-75" hashes="a b c d"/>
</hick:session>
"#;

#[tokio::test]
async fn the_session_view_carries_blocks_and_the_three_families_of_links() {
    let _env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("notes")).unwrap();
    std::fs::create_dir_all(root.join("sessions")).unwrap();
    std::fs::write(root.join("notes/today.hick"), "# today\n").unwrap();
    std::fs::write(root.join("sessions/s.hick"), SESSION).unwrap();
    unsafe { std::env::set_var("HICKORY_STATE_DIR", state_dir.path()) };
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    let res = reqwest::get(format!(
        "http://127.0.0.1:{port}/api/sessions/view?path=sessions/s.hick"
    ))
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["path"], "sessions/s.hick");
    assert!(body["source"].as_str().unwrap().contains("<hick:user"));

    let kinds: Vec<&str> = body["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        vec![
            "session-user",
            "session-assistant",
            "session-tool",
            "session-tool-result",
            "session-read",
            "session-assistant",
            "session-wrote"
        ]
    );

    let links: Vec<(String, String, Value, u64)> = body["links"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["family"].as_str().unwrap().to_string(),
                l["to"]["path"].as_str().unwrap().to_string(),
                l["to"]["lines"].clone(),
                l["lines"][0].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        links,
        vec![
            (
                "context".into(),
                "meetings/sync.hick".into(),
                Value::Null,
                4
            ),
            (
                "context".into(),
                "data/latency.csv".into(),
                serde_json::json!([1, 8]),
                11
            ),
            (
                "declared".into(),
                "meetings/sync.hick".into(),
                serde_json::json!([12, 20]),
                12
            ),
            (
                "lineage".into(),
                "notes/today.hick".into(),
                serde_json::json!([72, 75]),
                15
            ),
        ]
    );
}
