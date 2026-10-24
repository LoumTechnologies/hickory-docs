//! Selected execution must not run unrelated cells, including through block actions.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};

const SOURCE: &str = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n<hick:file path=\"before.txt\"><hick:exec container=\"selected\" show=\"output\">printf 'dependency\\n'</hick:exec></hick:file>\n<hick:exec container=\"other\">printf 'selected-error-stderr\\n' > /dev/stderr; exit 91</hick:exec>\n<hick:file path=\"chosen.txt\"><hick:exec container=\"selected\" show=\"output\">printf 'chosen\\n'</hick:exec></hick:file>\n</hick:doc>\n";

async fn finish(client: &reqwest::Client, base: &str, value: Value) -> Value {
    let id = value["run_id"].as_str().unwrap();
    let until = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let value: Value = client
            .get(format!("{base}/api/runs/{id}"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if value["status"] != "running" {
            return value;
        }
        assert!(tokio::time::Instant::now() < until, "run timed out");
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn selected_run_and_block_action_execute_dependencies_but_not_unrelated_cells() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let path = root.join("selected.md");
    let doc_id = hickory_cli::serve::store::doc_id("selected.md");
    std::fs::write(&path, SOURCE).unwrap();
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
    let hash = hick_context_protocol::digest(SOURCE.as_bytes());
    let response = client
        .post(format!("{base}/api/docs/{doc_id}/run"))
        .json(&json!({"cells":["selected:4"],"expected_source_hash":hash}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    let run = finish(&client, &base, response.json().await.unwrap()).await;
    assert_eq!(run["status"], "ok", "{run}");
    assert_eq!(run["source_hash"], hash);
    assert_eq!(
        std::fs::read_to_string(root.join("before.txt"))
            .unwrap()
            .trim_end_matches('\n'),
        "dependency"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("chosen.txt"))
            .unwrap()
            .trim_end_matches('\n'),
        "chosen"
    );

    // Reset records, then exercise the element-dispatch route independently.
    std::fs::write(&path, SOURCE).unwrap();
    std::fs::remove_file(root.join("before.txt")).unwrap();
    std::fs::remove_file(root.join("chosen.txt")).unwrap();
    let receipt: Value = client
        .get(format!("{base}/api/docs/{doc_id}/render"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(receipt["document"], doc_id);
    assert_eq!(receipt["source_hash"], hash);
    let at = SOURCE.rfind("<hick:exec").unwrap();
    std::fs::write(&path, format!("\n{SOURCE}")).unwrap();
    let stale = client
        .post(format!("{base}/api/docs/{doc_id}/blocks/{at}/run"))
        .json(&json!({"expected_source_hash":hash}))
        .send()
        .await
        .unwrap();
    assert_eq!(stale.status(), 409);
    assert!(!root.join("before.txt").exists());
    assert!(!root.join("chosen.txt").exists());
    std::fs::write(&path, SOURCE).unwrap();
    let response = client
        .post(format!("{base}/api/docs/{doc_id}/blocks/{at}/run"))
        .json(&json!({"expected_source_hash":format!("sha256:{}", hash.trim_start_matches("sha256:"))}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    let run = finish(&client, &base, response.json().await.unwrap()).await;
    assert_eq!(run["status"], "ok", "{run}");
    assert!(root.join("before.txt").exists());
    assert!(root.join("chosen.txt").exists());

    // The queued execution guard also rejects a stale disclosed revision.
    std::fs::write(&path, SOURCE).unwrap();
    std::fs::remove_file(root.join("before.txt")).unwrap();
    std::fs::remove_file(root.join("chosen.txt")).unwrap();
    let response = client
        .post(format!("{base}/api/docs/{doc_id}/run"))
        .json(&json!({"cells":["selected:4"],"expected_source_hash":"sha256:stale"}))
        .send()
        .await
        .unwrap();
    let run = finish(&client, &base, response.json().await.unwrap()).await;
    assert_eq!(run["status"], "failed", "{run}");
    assert!(
        run["error"]
            .as_str()
            .unwrap()
            .contains("Source revision changed")
    );
    assert_eq!(run["requested_source_hash"], "sha256:stale");
    assert!(!root.join("before.txt").exists());
    assert!(!root.join("chosen.txt").exists());

    // An unknown id must fail before even a valid requested cell executes.
    std::fs::write(&path, SOURCE).unwrap();
    let response = client
        .post(format!("{base}/api/docs/{doc_id}/run"))
        .json(&json!({"cells":["selected:4","missing:99"]}))
        .send()
        .await
        .unwrap();
    let run = finish(&client, &base, response.json().await.unwrap()).await;
    assert_eq!(run["status"], "failed", "{run}");
    assert!(
        run["error"]
            .as_str()
            .unwrap()
            .contains("Unknown requested cell")
    );
    assert!(!root.join("before.txt").exists());
    assert!(!root.join("chosen.txt").exists());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), SOURCE);
    let partial_check = client
        .post(format!("{base}/api/docs/{doc_id}/check"))
        .json(&json!({"cells":["selected:4"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(partial_check.status(), 422);
    // A failed selected cell retains its exact source receipt without duplicating
    // the command failure or stderr through CapturedRootError's source chain.
    let failed = client
        .post(format!("{base}/api/docs/{doc_id}/run"))
        .json(&json!({"cells":["other:3"],"expected_source_hash":hash}))
        .send()
        .await
        .unwrap();
    assert_eq!(failed.status(), 202);
    let failed = finish(&client, &base, failed.json().await.unwrap()).await;
    assert_eq!(failed["status"], "failed", "{failed}");
    assert_eq!(failed["source_hash"], hash);
    assert_eq!(failed["requested_source_hash"], hash);
    let error = failed["error"].as_str().unwrap();
    assert_eq!(
        error
            .matches("command failed in container 'other' (exit 91)")
            .count(),
        1,
        "{error}"
    );
    assert_eq!(
        error.matches("\nselected-error-stderr").count(),
        1,
        "{error}"
    );

    // Negative control: the same source really fails when all cells execute.
    let whole = client
        .post(format!("{base}/api/docs/{doc_id}/run"))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    let run = finish(&client, &base, whole.json().await.unwrap()).await;
    assert_eq!(
        run["status"], "failed",
        "unrelated exit91 must fail whole run: {run}"
    );
    server.abort();
}
