//! Guarantee: docs/guarantees/agent/ui-action-dispatch.md
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
    up::{
        UpCommand,
        state::{OutputState, WovenState, set_read_only},
    },
};
use serde_json::{Value, json};

#[tokio::test]
async fn output_resolution_http_waits_for_the_applied_or_refused_loop_reply() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let path = root.join("out.txt");
    let mut woven = WovenState::default();
    woven
        .write_output(
            &root,
            &path,
            OutputState {
                doc: root.join("doc.md"),
                rel_path: "out.txt".into(),
                content: "produced".into(),
                provenance: vec![],
            },
        )
        .unwrap();
    set_read_only(&path, false).unwrap();
    std::fs::write(&path, "disk").unwrap();
    woven.hold_output(&path, "Cannot carry back".into());
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let (sender, mut commands) = tokio::sync::mpsc::unbounded_channel();
    *prepared.state.up_commands.lock().unwrap() = Some(sender);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!(
        "http://{}/api/outputs/resolve",
        listener.local_addr().unwrap()
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let client = reqwest::Client::new();
    for (expected, status) in [("stale disk", 409), ("disk", 200)] {
        let body = json!({ "path": "out.txt", "content": "merged", "expected_current": expected, "expected_produced": "produced" });
        let client = client.clone();
        let endpoint = url.clone();
        let response =
            tokio::spawn(async move { client.post(endpoint).json(&body).send().await.unwrap() });
        let command = tokio::time::timeout(std::time::Duration::from_secs(5), commands.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(
            !response.is_finished(),
            "Queued work must not be acknowledged as applied"
        );
        match command {
            UpCommand::Resolve { request, reply } => {
                assert_eq!(request.path, path);
                let outcome = woven.resolve_output(&request).map_err(|e| e.to_string());
                reply.send(outcome).unwrap();
            }
            _ => panic!("Wrong host command"),
        }
        let response = response.await.unwrap();
        assert_eq!(response.status().as_u16(), status);
        let body: Value = response.json().await.unwrap();
        if status == 409 {
            assert!(body["error"].as_str().unwrap().contains("changed on disk"));
            assert!(woven.held().contains_key(&path));
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "disk");
        } else {
            assert_eq!(
                body,
                json!({ "ok": true, "path": "out.txt", "applied": true, "carried_back": false })
            );
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "merged");
        }
    }
    assert_eq!(client.post(&url).json(&json!({ "path": "../escape", "content": "x", "expected_current": "", "expected_produced": "" })).send().await.unwrap().status(), 400);
    assert!(commands.try_recv().is_err());
    server.abort();
}
