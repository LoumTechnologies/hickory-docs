//! Contract: .hickory-source/guarantees/agent/feature-experiments/contract.md
//! A deterministic ACP peer checks plumbing and isolation, never model benefit.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration;

struct TestEvidence(std::path::PathBuf);
impl Drop for TestEvidence {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn normal_codex_request_collects_isolated_ablations_and_reviews_without_inventing_savings() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::write(root.join("note.md"), "# Implement a request\n").unwrap();
    std::fs::write(root.join("project.md"), "# An index for the same source\n").unwrap();
    std::fs::write(root.join("app.txt"), "before\n").unwrap();
    let contract_path = ".hickory-source/guarantees/demo/contract.md";
    std::fs::create_dir_all(root.join(".hickory-source/guarantees/demo")).unwrap();
    std::fs::write(root.join(contract_path), "# A task contract\n").unwrap();
    let prepared = prepare(ServeOptions {
        target: root.join("note.md"),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: Some(root.join("ui.json")),
    })
    .await
    .unwrap();
    let doc = prepared
        .state
        .index
        .entries()
        .into_iter()
        .find(|(_, path)| path == "note.md")
        .unwrap()
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let http = reqwest::Client::new();
    let command = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!(
            "examples/acp_fixture{}",
            std::env::consts::EXE_SUFFIX
        ));
    assert!(
        command.is_file(),
        "Build the acp_fixture example for this focused test"
    );
    let response = http
        .put(format!("{base}/api/agents"))
        .json(&json!([{"id":"codex","name":"Codex protocol fixture","command":command,"args":[]}]))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let response: Value = http
        .post(format!("{base}/api/docs/{doc}/agent/acp"))
        .json(&json!({"backend":"codex"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["ready"], true, "{response}");
    let response: Value = http.post(format!("{base}/api/docs/{doc}/agent")).json(&json!({"backend":"codex","prompt":"measure-request","experiment":{"features":["literate-indexes","lineage-tools"],"timeout_seconds":30}})).send().await.unwrap().json().await.unwrap();
    let turn = response["session_id"].as_str().unwrap();
    tokio::time::timeout(Duration::from_secs(45), async {
        loop {
            let response: Value = http
                .get(format!("{base}/api/docs/{doc}/agent/turns"))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if let Some(result) = response["turns"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == turn && t["status"] != "running")
            {
                assert_eq!(result["status"], "ok", "{result}");
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("app.txt")).unwrap(),
        "primary or trial edit\n"
    );
    let data: Value = http
        .get(format!("{base}/api/feature-experiments"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let _evidence = TestEvidence(std::path::PathBuf::from(data["storage"].as_str().unwrap()));
    let record = data["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == turn)
        .unwrap();
    assert_eq!(record["pairs"].as_array().unwrap().len(), 2, "{record}");
    assert_eq!(record["primary"]["status"], "completed");
    let primary: Value =
        serde_json::from_str(record["primary"]["answer"].as_str().unwrap()).unwrap();
    assert_eq!(primary["has_index"], true);
    assert_eq!(primary["has_lineage"], true);
    assert_eq!(primary["starting_source"], "before\n");
    assert_eq!(
        record["measurement_implementation_sha256"],
        hex::encode(Sha256::digest(include_bytes!(
            "../src/serve/acp/experiments.rs"
        )))
    );
    let mut trial_paths = std::collections::HashSet::new();
    for pair in record["pairs"].as_array().unwrap() {
        assert!(pair["savings"].is_null());
        for arm in pair["arms"].as_array().unwrap() {
            assert_eq!(arm["status"], "completed", "{arm}");
            assert!(arm["correct"].is_null());
            assert!(
                arm["end_to_end_seconds"].as_f64().unwrap()
                    >= arm["wall_seconds"].as_f64().unwrap()
            );
            assert!(arm["setup_seconds"].as_f64().unwrap() >= 0.0);
            assert!(
                arm["total_tokens"].is_null(),
                "Fixture reported context occupancy only"
            );
            let answer: Value = serde_json::from_str(arm["answer"].as_str().unwrap()).unwrap();
            assert_eq!(
                answer["starting_source"], "before\n",
                "Each arm must start before the primary edit"
            );
            let enabled = arm["arm"] == "enabled";
            if pair["feature"] == "literate-indexes" {
                assert_eq!(answer["has_index"], enabled);
            }
            if pair["feature"] == "lineage-tools" {
                assert_eq!(answer["has_lineage"], enabled);
                assert_eq!(answer["direct_disabled_refusal"], !enabled);
            }
            let workspace = arm["workspace"].as_str().unwrap();
            assert_ne!(std::path::Path::new(workspace), root);
            assert!(trial_paths.insert(workspace.to_string()));
            assert!(std::path::Path::new(arm["transcript"].as_str().unwrap()).is_file());
        }
    }
    let mut last = Value::Null;
    for arm in ["enabled", "disabled"] {
        last = http.post(format!("{base}/api/feature-experiments/{turn}/review")).json(&json!({"feature":"literate-indexes","arm":arm,"correct":true,"contract":contract_path,"notes":"Protocol fixture observed the frozen starting bytes and isolated writes; not a model correctness claim."})).send().await.unwrap().json().await.unwrap();
    }
    let pair = last["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == turn)
        .unwrap()["pairs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["feature"] == "literate-indexes")
        .unwrap();
    assert!(pair["savings"]["wall_seconds"].is_number());
    assert!(pair["savings"]["total_tokens"].is_null());
    let covered = last["coverage"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["contract"] == contract_path)
        .unwrap();
    assert_eq!(covered["status"], "reviewed_observation");
    std::fs::write(root.join(contract_path), "# Changed requirement\n").unwrap();
    let stale: Value = http
        .get(format!("{base}/api/feature-experiments"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        stale["coverage"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["contract"] == contract_path)
            .unwrap()["status"],
        "unmeasured"
    );
    let corrected: Value = http.post(format!("{base}/api/feature-experiments/{turn}/review")).json(&json!({"feature":"literate-indexes","arm":"enabled","correct":false,"notes":"A later review retracts the earlier acceptance; the original review must remain retained."})).send().await.unwrap().json().await.unwrap();
    let corrected_pair = corrected["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == turn)
        .unwrap()["pairs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["feature"] == "literate-indexes")
        .unwrap();
    assert!(corrected_pair["savings"].is_null());
    let storage = std::path::PathBuf::from(data["storage"].as_str().unwrap());
    assert_eq!(
        std::fs::read_dir(storage.join(turn).join("reviews"))
            .unwrap()
            .count(),
        3
    );
    server.abort();
    // The fixture's personal dataset is not user research evidence.
    std::fs::remove_dir_all(storage).unwrap();
}
