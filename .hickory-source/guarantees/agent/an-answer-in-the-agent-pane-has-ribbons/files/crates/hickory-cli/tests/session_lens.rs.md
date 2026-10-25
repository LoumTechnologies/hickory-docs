<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/session_lens.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An Answer In The Agent Pane Has Ribbons](../../../../contract.md)

Captured SHA-256: `e77570212efe484c868e84da09060b8b4d25aaced31637e91e998f9de14bc785`.

<book:entry path="crates/hickory-cli/tests/session_lens.rs">
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

const SESSION: &str = r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-09-05T00:00:00Z" doc="notes/today.md">
<hick:user turn="t1">Why is checkout slow?</hick:user>
<hick:assistant>
<hick:tool name="read_doc">
<hick:arg name="doc">meetings/sync.md</hick:arg>
</hick:tool>
</hick:assistant>
<hick:tool-result name="read_doc" ok="true">
…
</hick:tool-result>
<hick:read file="data/latency.csv" sha256="ff" lines="1-8"/>
<hick:assistant>
It slowed after the pool change; see [the sync](meetings/sync.md#L12-L20).
</hick:assistant>
<hick:wrote file="notes/today.md" lines="72-75" hashes="a b c d"/>
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
    std::fs::write(root.join("notes/today.md"), "# today\n").unwrap();
    std::fs::write(root.join("sessions/s.md"), SESSION).unwrap();
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
        "http://127.0.0.1:{port}/api/sessions/view?path=sessions/s.md"
    ))
    .await
    .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["path"], "sessions/s.md");
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
            ("context".into(), "meetings/sync.md".into(), Value::Null, 4),
            (
                "context".into(),
                "data/latency.csv".into(),
                serde_json::json!([1, 8]),
                11
            ),
            (
                "declared".into(),
                "meetings/sync.md".into(),
                serde_json::json!([12, 20]),
                13
            ),
            (
                "lineage".into(),
                "notes/today.md".into(),
                serde_json::json!([72, 75]),
                15
            ),
        ]
    );
    // Retrospective corrections survive reopening and never erase the original.
    let client = reqwest::Client::new();
    let endpoint = format!("http://127.0.0.1:{port}/api/sessions/audit");
    let response = client.post(&endpoint).json(&serde_json::json!({
        "path":"sessions/s.md", "entry":{
            "statement":"Checkout delay was caused by the pool change",
            "reason":"This was the explanation originally used", "dependencies":["notes/today.md"]
        }
    })).send().await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let view_url = format!("http://127.0.0.1:{port}/api/sessions/view?path=sessions/s.md");
    let before: Value = client
        .get(&view_url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let original = before["audit"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["record_kind"] == "audit")
        .unwrap();
    assert_eq!(original["actor"], "user-declared");
    let id = original["id"].as_str().unwrap();
    std::fs::write(root.join("notes/today.md"), "# corrected context\n").unwrap();
    let response = client.post(&endpoint).json(&serde_json::json!({
        "path":"sessions/s.md", "entry":{
            "statement":"The cause is still unresolved", "reason":"The pool explanation lacks a controlled witness", "supersedes":id
        }
    })).send().await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let after: Value = client
        .get(&view_url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let declarations: Vec<_> = after["audit"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["record_kind"] == "audit")
        .collect();
    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0]["id"], id);
    assert_eq!(
        declarations[0]["dependency_snapshots"][0]["current_state"],
        "changed"
    );
    assert_eq!(declarations[1]["entry"]["supersedes"], id);
    assert!(
        after["source"]
            .as_str()
            .unwrap()
            .contains("Why is checkout slow?")
    );
    let invalid = client.post(&endpoint).json(&serde_json::json!({
        "path":"sessions/s.md", "entry":{"statement":"Correction", "reason":"Check", "supersedes":"nonexistent"}
    })).send().await.unwrap();
    assert_eq!(invalid.status().as_u16(), 422);
}

// Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
#[tokio::test]
async fn a_literal_acp_answer_gets_current_output_evidence_only_when_disk_bytes_match() {
    let _env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::create_dir(root.join("sessions")).unwrap();
    let document = "# Producer\n<hick:file path=\"test2.py\">\nprint('hello')\n</hick:file>\n";
    std::fs::write(root.join("test2.md"), document).unwrap();
    std::fs::write(root.join("test2.py"), "print('hello')\n").unwrap();
    let source = format!(
        r#"<hick:session><hick:user turn="one" provider="acp:codex">Which document?</hick:user>
<hick:assistant><hick:input>
[test2.md]({}:2) writes `test2.py` through `<hick:file path="test2.py">`.
</hick:input></hick:assistant></hick:session>"#,
        root.join("test2.md").display()
    );
    std::fs::write(root.join("sessions/s.md"), &source).unwrap();
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
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let url = format!("http://127.0.0.1:{port}/api/sessions/view?path=sessions/s.md");
    let body: Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
    assert_eq!(body["blocks"][1]["kind"], "session-assistant");
    assert!(
        body["blocks"][1]["body"]
            .as_str()
            .unwrap()
            .contains("<hick:file")
    );
    assert_eq!(body["links"][0]["family"], "declared");
    assert_eq!(body["links"][0]["to"]["path"], "test2.md");
    let proof = body["links"]
        .as_array()
        .unwrap()
        .iter()
        .find(|link| link.get("evidence").is_some())
        .unwrap_or_else(|| panic!("current output lineage: {body:#}"));
    assert_eq!(proof["family"], "lineage");
    assert_eq!(proof["to"]["path"], "test2.py");
    assert_eq!(proof["evidence"]["path"], "test2.md");
    assert_eq!(proof["evidence"]["lines"], serde_json::json!([3, 3]));
    std::fs::write(root.join("test2.py"), "changed outside document\n").unwrap();
    let changed: Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
    assert!(
        changed["links"]
            .as_array()
            .unwrap()
            .iter()
            .all(|link| link.get("evidence").is_none())
    );
    assert_eq!(
        std::fs::read_to_string(root.join("sessions/s.md")).unwrap(),
        source
    );
    server.abort();
}
</book:entry>

</book:doc>
