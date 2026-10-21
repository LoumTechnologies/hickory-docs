#![cfg(unix)]
//! Guarantees: docs/guarantees/verification/predictions-precede-managed-observation.md
//! Real shared-tool dispatch; not a complete attached UI policy witness.
use hick_literate::prediction::{OutputEncoding, SealedRequest, output_target};
use hick_observe::prediction::*;
use hickory_agent::{EditSession, ToolInvocation, execute_tool};
use hickory_executor::LocalExecutor;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};

fn request(ledger: &Ledger, source: &str, key: &str) -> SealedRequest {
    let actor = Actor {
        kind: "agent".into(),
        id: "host-bound-predictor".into(),
    };
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&json!([80, 80])).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([("test-runtime".into(), digest(&"local-printf").unwrap())]),
    };
    let target = output_target("c", 2, OutputEncoding::Json);
    let receipt = ledger
        .seal(
            ledger.history(None).unwrap().generation,
            Seal {
                scenario: Scenario {
                    family: "shared-tool-total-budget".into(),
                    inputs: json!([80, 80]),
                    requirement_reference: "single total deadline".into(),
                    requirement: Predicate::NumberAtMost(100.0),
                    identity_known: true,
                },
                identity: identity.clone(),
                target: target.clone(),
                predicted: Predicate::Equals(json!(100)),
                rationale: "All attempts share one budget".into(),
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
fn source(value: u64) -> String {
    format!(
        "<hick:container name=\"c\" image=\"alpine\" />\n<hick:file path=\"answer.txt\"><hick:exec container=\"c\">printf {value}<hick:expect>100</hick:expect></hick:exec></hick:file>"
    )
}
async fn tool(
    session: &mut EditSession,
    name: &str,
    args: Vec<(String, String)>,
) -> hickory_agent::ToolOutcome {
    execute_tool(
        session,
        Arc::new(LocalExecutor::new().unwrap()),
        &ToolInvocation::synthetic(name, args, None).unwrap(),
    )
    .await
}
#[tokio::test]
async fn shared_verify_preserves_failure_then_repair_without_answers_or_output_publication() {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let ledger = Arc::new(Ledger::open(private.path()).unwrap());
    let path = root.path().join("doc.md");
    let faulty = source(160);
    std::fs::write(&path, &faulty).unwrap();
    std::fs::write(root.path().join("previous.txt"), "PRIVATE-CACHED-ANSWER").unwrap();
    let initial = request(&ledger, &faulty, "fault");
    let mut session =
        EditSession::open_prediction(&path, ledger.clone(), initial, OutputEncoding::Json)
            .await
            .unwrap();
    let failed = tool(&mut session, "verify", vec![]).await;
    assert!(!failed.ok);
    let feedback: serde_json::Value = serde_json::from_str(&failed.text).unwrap();
    assert_eq!(feedback["summary"]["classification"], "execution_rejected");
    assert!(
        !failed.text.contains("stdout")
            && !failed.text.contains("captures")
            && !failed.text.contains("actual")
    );
    assert!(!root.path().join("answer.txt").exists());
    for name in ["read_file", "read_output", "edit_output", "shell", "run"] {
        let result = tool(
            &mut session,
            name,
            vec![("path".into(), "previous.txt".into())],
        )
        .await;
        assert!(!result.ok && !result.text.contains("PRIVATE-CACHED-ANSWER"));
    }
    let preview = session
        .preview_edit(
            &ToolInvocation::synthetic(
                "edit_output",
                vec![("path".into(), "answer.txt".into())],
                None,
            )
            .unwrap(),
        )
        .await;
    assert!(preview.is_err());
    // A source edit through the captured tool still needs a new host seal.
    let corrected = source(100);
    let edit = EditSession::reviewed_invocation(
        &ToolInvocation::synthetic("edit_doc", vec![], None).unwrap(),
        &faulty,
        corrected.clone(),
    )
    .unwrap();
    let edited = execute_tool(&mut session, Arc::new(LocalExecutor::new().unwrap()), &edit).await;
    assert!(edited.ok, "{}", edited.text);

    assert!(!tool(&mut session, "verify", vec![]).await.ok);
    let next = request(&ledger, &corrected, "repair");
    session.replace_prediction(next).unwrap();
    let passed = tool(&mut session, "verify", vec![]).await;
    assert!(passed.ok, "{}", passed.text);
    let feedback: serde_json::Value = serde_json::from_str(&passed.text).unwrap();
    assert_eq!(
        feedback["summary"]["classification"],
        "verified_after_correction"
    );
    assert!(!root.path().join("answer.txt").exists());
    let history = ledger.history(None).unwrap();
    assert_eq!(history.attempts.len(), 2);
    assert!(history.attempts.iter().all(|entry| !entry.promoted));
    // Out-of-band writes cannot inject runtime answers through source reads.
    std::fs::write(&path, "PRIVATE-RUNTIME-ANSWER").unwrap();
    let refused = tool(&mut session, "read_doc", vec![]).await;
    assert!(!refused.ok && !refused.text.contains("PRIVATE-RUNTIME-ANSWER"));
    let snapshots = session.source_snapshots();
    assert_eq!(
        snapshots[0]["sha256"],
        format!("{:x}", Sha256::digest(corrected.as_bytes()))
    );
    assert_eq!(snapshots[0]["current"], false);
    assert!(
        !serde_json::to_string(&snapshots)
            .unwrap()
            .contains(&format!("{:x}", Sha256::digest(b"PRIVATE-RUNTIME-ANSWER")))
    );
}
