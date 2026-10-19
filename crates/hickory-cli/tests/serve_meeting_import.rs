//! Guarantee: docs/guarantees/authoring/meeting-import-preserves-source.md
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn local_http_import_preserves_original_deduplicates_and_navigates_exact_utterances() {
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
    let text = "WEBVTT\r\n\r\n00:00:01.000 --> 00:00:02.000\r\n<v Ada>Approve before deleting\r\n";
    let body = json!({"name":"meeting.vtt", "text":text});
    let url = format!("{base}/api/meetings/import");
    let first = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(first.status(), 200);
    let first: Value = first.json().await.unwrap();
    let document = client
        .get(format!(
            "{base}/api/docs/{}",
            first["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(document.status(), 200);
    assert_eq!(first["source_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(first["utterances"][0]["speaker"], "Ada");
    let at = first["utterances"][0]["start_utf8"].as_u64().unwrap() as usize;
    let end = first["utterances"][0]["end_utf8"].as_u64().unwrap() as usize;
    assert!(text[at..end].contains("Approve before deleting"));
    assert_eq!(
        std::fs::read(root.join(first["source"].as_str().unwrap())).unwrap(),
        text.as_bytes()
    );
    let second: Value = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(second["duplicate"], true);
    assert_eq!(first["path"], second["path"]);
    let rejected = client
        .post(&url)
        .json(&json!({"name":"recording.wav", "text":"audio"}))
        .send()
        .await
        .unwrap();
    assert_eq!(rejected.status(), 400);
    server.abort();
}
