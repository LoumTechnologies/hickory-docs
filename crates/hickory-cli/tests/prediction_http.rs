#![cfg(unix)]
//! Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
//! Startup-bound HTTP policy; no native/provider application flag is inferred.
use hick_literate::prediction::{OutputEncoding, SealedRequest, output_target};
use hick_observe::prediction::*;
use hickory_cli::{
    ExecutorChoice,
    serve::{PrivatePredictionBinding, ServeOptions, prepare_with_prediction},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn source(value: u64) -> String {
    format!(
        "<hick:container name=\"c\" image=\"alpine\" />\n<hick:file path=\"answer.txt\"><hick:exec container=\"c\">printf {value}<hick:expect>100</hick:expect></hick:exec></hick:file>"
    )
}
fn request(ledger: &Ledger, source: &str, key: &str) -> SealedRequest {
    let actor = Actor {
        kind: "agent".into(),
        id: "host-http-predictor".into(),
    };
    let identity = Identity {
        source: hash(source.as_bytes()),
        inputs: digest(&json!([80, 80])).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([("runtime".into(), digest(&"local-printf").unwrap())]),
    };
    let target = output_target("c", 2, OutputEncoding::Json);
    let receipt = ledger
        .seal(
            ledger.history(None).unwrap().generation,
            Seal {
                identity: identity.clone(),
                target: target.clone(),
                predicted: Predicate::Equals(json!(100)),
                rationale: "Attempts share one deadline".into(),
                scenario: Scenario {
                    family: "http-total-deadline".into(),
                    inputs: json!([80, 80]),
                    requirement_reference: "single total 100ms budget".into(),
                    requirement: Predicate::NumberAtMost(100.0),
                    identity_known: true,
                },
            },
            actor.clone(),
        )
        .unwrap();
    SealedRequest {
        seal: receipt.id,
        request_id: key.into(),
        actor,
        identity,
        target,
    }
}
#[tokio::test]
async fn native_run_route_uses_private_gate_and_alternate_http_surfaces_refuse_without_answers() {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let ledger = Arc::new(Ledger::open(private.path()).unwrap());
    let path = root.path().join("doc.md");
    let faulty = source(160);
    std::fs::write(&path, &faulty).unwrap();
    std::fs::write(root.path().join("cached.txt"), "PRIVATE-OLD-ANSWER").unwrap();
    let prepared = prepare_with_prediction(
        ServeOptions {
            target: path.clone(),
            port: 0,
            params: vec![],
            executor: ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        },
        PrivatePredictionBinding {
            ledger: ledger.clone(),
            request: request(&ledger, &faulty, "fault"),
            encoding: OutputEncoding::Json,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let base = format!("http://{address}/api");
    let client = reqwest::Client::new();
    let status: Value = client
        .get(format!("{base}/prediction/state"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let doc = status["document"].as_str().unwrap();
    for (method, route) in [
        ("GET", format!("/docs/{doc}/outputs")),
        ("GET", format!("/docs/{doc}/context")),
        ("GET", "/file?path=cached.txt".into()),
        ("GET", "/sessions/view?path=cached.txt".into()),
        ("GET", "/ws".into()),
        ("POST", "/launch-profiles/run".into()),
        ("POST", format!("/docs/{doc}/agent")),
        ("POST", format!("/docs/{doc}/agent/acp")),
        ("POST", format!("/docs/{doc}/agent/python")),
    ] {
        let response = client
            .request(method.parse().unwrap(), format!("{base}{route}"))
            .json(&json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 422, "{route}");
        assert!(
            !response
                .text()
                .await
                .unwrap()
                .contains("PRIVATE-OLD-ANSWER")
        );
    }
    let accepted: Value = client
        .post(format!("{base}/docs/{doc}/run"))
        .json(&json!({"expected_source_hash":hash(faulty.as_bytes())}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let run = accepted["run_id"].as_str().unwrap();
    let mut result = Value::Null;
    for _ in 0..100 {
        result = client
            .get(format!("{base}/runs/{run}"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if result["status"] != "running" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(result["status"], "failed");
    assert_eq!(
        result["blocks"][0]["summary"]["classification"],
        "execution_rejected"
    );
    assert!(result["blocks"][0]["summary"].get("actual").is_none());
    assert!(!root.path().join("answer.txt").exists());
    let read: Value = client
        .post(format!("{base}/prediction/tool"))
        .json(&json!({"name":"read_doc","args":[]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(read["ok"], true);
    let corrected = source(100);
    let edit:Value=client.post(format!("{base}/prediction/tool")).json(&json!({"name":"edit_doc","args":[["whole","true"],["snapshot",hash(faulty.as_bytes())]],"input":corrected})).send().await.unwrap().json().await.unwrap();
    assert_eq!(edit["ok"], true, "{edit}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source(100));
    // No runtime policy toggle or unsealed fallback; repair requires host reseal.
    assert_eq!(
        client
            .post(format!("{base}/docs/{doc}/check"))
            .json(&json!({"expected_source_hash":hash(source(100).as_bytes())}))
            .send()
            .await
            .unwrap()
            .status()
            .as_u16(),
        409
    );
    std::fs::write(&path, "PRIVATE-RUNTIME-REPLACEMENT").unwrap();
    let refused: Value = client
        .post(format!("{base}/prediction/tool"))
        .json(&json!({"name":"read_doc","args":[]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(refused["ok"], false);
    assert!(!refused.to_string().contains("PRIVATE-RUNTIME-REPLACEMENT"));
    let history: Value = client
        .get(format!("{base}/prediction/history"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(history["attempts"][0]["current_certification"], false);
    server.abort();
    // A reviewed host restart supplies the new source seal, retaining the same
    // durable scenario history. No HTTP argument can replace that policy.
    let corrected = source(100);
    std::fs::write(&path, &corrected).unwrap();
    let prepared = prepare_with_prediction(
        ServeOptions {
            target: path,
            port: 0,
            params: vec![],
            executor: ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        },
        PrivatePredictionBinding {
            ledger: ledger.clone(),
            request: request(&ledger, &corrected, "repair"),
            encoding: OutputEncoding::Json,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let base = format!("http://{address}/api");
    let accepted: Value = client
        .post(format!("{base}/docs/{doc}/check"))
        .json(&json!({"expected_source_hash":hash(corrected.as_bytes())}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let run = accepted["run_id"].as_str().unwrap();
    for _ in 0..100 {
        result = client
            .get(format!("{base}/runs/{run}"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if result["status"] != "running" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(result["status"], "ok");
    assert_eq!(
        result["blocks"][0]["summary"]["classification"],
        "verified_after_correction"
    );
    assert_eq!(ledger.history(None).unwrap().attempts.len(), 2);
    assert!(!root.path().join("answer.txt").exists());
    server.abort();
}
