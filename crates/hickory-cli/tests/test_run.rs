//! One test, run from the gutter, through the endpoint, to a terminal that
//! finishes.
//!
//! Protects docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md

use std::time::{Duration, Instant};

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

#[tokio::test]
async fn a_rust_test_runs_in_a_terminal_named_after_it() {
    if std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        eprintln!("SKIPPED: no cargo on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(
        root.join("notes.hick"),
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"notes.md\">\n# Notes\n</hick:doc>\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("lib/src")).unwrap();
    std::fs::write(
        root.join("lib/Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("lib/src/lib.rs"),
        "pub fn add(a: u32, b: u32) -> u32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() { assert_eq!(super::add(1, 2), 3); }\n}\n",
    )
    .unwrap();

    let prepared = prepare(ServeOptions {
        target: root.join("notes.hick"),
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
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::new();

    // A path outside the folder, and a language with no runner, are refused.
    let refused = client
        .post(format!("{base}/api/tests/run"))
        .json(&json!({ "path": "../x.rs", "name": "adds", "language": "rust" }))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status().as_u16(), 400);
    let refused = client
        .post(format!("{base}/api/tests/run"))
        .json(&json!({ "path": "lib/src/lib.rs", "name": "adds", "language": "haskell" }))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status().as_u16(), 422);

    let response = client
        .post(format!("{base}/api/tests/run"))
        .json(&json!({ "path": "lib/src/lib.rs", "name": "adds", "language": "rust" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let session: Value = response.json().await.unwrap();
    assert_eq!(session["title"], "test: adds");
    assert!(
        session["cwd"].as_str().unwrap().ends_with("lib"),
        "ran in the wrong directory: {session}"
    );
    let id = session["id"].as_str().unwrap().to_string();

    // The session finishes: cargo compiled the crate and ran the test. Its
    // state comes from the terminal registry, the same one the dock reads.
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        let listing: Value = client
            .get(format!("{base}/api/terminals"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let found = listing["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == id)
            .cloned()
            .expect("the session is listed");
        let state = found["state"].as_str().unwrap_or("");
        if state == "finished" {
            // Finished because cargo ran the test, not because something
            // exited early: the runner's own last line says so.
            let preview = found["preview"].as_str().unwrap_or("");
            assert!(
                preview.contains("test result") || preview.contains("1 passed"),
                "the session finished without cargo's verdict: {found}"
            );
            break;
        }
        assert_ne!(state, "failed", "the test run failed: {found}");
        assert!(
            Instant::now() < deadline,
            "the test run did not finish: {found}"
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
