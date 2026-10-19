//! Guarantee: docs/guarantees/execution/runtime-history-keeps-runs-and-occurrences-separate.md
use hick_observe::{Run, Source, Status};
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn runtime_history_http_keeps_evidence_immutable_and_pins_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let client = reqwest::Client::new();
    let mut run = Run::new(
        vec![Source::capture(
            "document".into(),
            "example.md".into(),
            b"console.log('hello')".to_vec(),
        )],
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    );
    run.finish(Status::Failed).unwrap();
    let url = format!("{base}/api/runtime/runs");
    assert_eq!(
        client.post(&url).json(&run).send().await.unwrap().status(),
        200
    );
    let id_url = format!("{url}/{}", run.id);
    assert_eq!(
        client
            .patch(&id_url)
            .json(&json!({"pinned":true}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client.post(&url).json(&run).send().await.unwrap().status(),
        200
    );
    let listing: Value = client.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(listing["runs"][0]["run"]["pinned"], true);
    assert_eq!(listing["runs"][0]["run"]["status"], "failed");
    let mut corrupt = run.clone();
    corrupt.sources[0].bytes.push(b'!');
    assert_eq!(
        client
            .post(&url)
            .json(&corrupt)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert!(root.join(format!(".hick/runs/{}.json", run.id)).exists());
    assert_eq!(client.delete(&id_url).send().await.unwrap().status(), 200);
    let listing: Value = client.get(&url).send().await.unwrap().json().await.unwrap();
    assert!(listing["runs"].as_array().unwrap().is_empty());
    server.abort();
}
