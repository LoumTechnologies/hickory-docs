<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/serve_meeting_import.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Meeting import retains original evidence without invoking a model](../../../../contract.md)

Captured SHA-256: `1ad2d3a699714037c362069d8fcfc84aa28424550ae4d0e145695cba4e41042d`.

<book:entry path="crates/hickory-cli/tests/serve_meeting_import.rs">
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
    assert!(first["path"].as_str().unwrap().ends_with(".md"));
    let document: Value = document.json().await.unwrap();
    assert!(document["source"].as_str().unwrap().contains(text));
    let render = client
        .get(format!(
            "{base}/api/docs/{}/render",
            first["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(render.status(), 200, "{}", render.text().await.unwrap());
    let restored_index = hickory_cli::serve::store::DocIndex::scan(&root).unwrap();
    assert!(
        restored_index
            .entries()
            .iter()
            .any(|(_, path)| path == first["path"].as_str().unwrap())
    );
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
    // An explicit reimport repairs only its matching old-format note.
    let current_path = root.join(first["path"].as_str().unwrap());
    let legacy_path = current_path.with_extension("hick");
    let annotated = format!(
        "{}\nHuman correction retained.\n",
        std::fs::read_to_string(&current_path).unwrap()
    );
    std::fs::rename(&current_path, &legacy_path).unwrap();
    std::fs::write(&legacy_path, &annotated).unwrap();
    let upgraded: Value = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(upgraded["duplicate"], true);
    assert!(upgraded["path"].as_str().unwrap().ends_with(".md"));
    assert_eq!(std::fs::read_to_string(&legacy_path).unwrap(), annotated);
    let upgraded_doc = client
        .get(format!(
            "{base}/api/docs/{}",
            upgraded["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(upgraded_doc.status(), 200);
    assert_eq!(
        upgraded_doc.json::<Value>().await.unwrap()["source"],
        annotated
    );
    let upgraded_render = client
        .get(format!(
            "{base}/api/docs/{}/render",
            upgraded["doc_id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(
        upgraded_render.status(),
        200,
        "{}",
        upgraded_render.text().await.unwrap()
    );
    assert!(
        hickory_cli::serve::store::DocIndex::scan(&root)
            .unwrap()
            .entries()
            .iter()
            .any(|(_, path)| path == upgraded["path"].as_str().unwrap())
    );
    let rejected = client
        .post(&url)
        .json(&json!({"name":"recording.wav", "text":"audio"}))
        .send()
        .await
        .unwrap();
    assert_eq!(rejected.status(), 400);
    server.abort();
}
</book:entry>

</book:doc>
