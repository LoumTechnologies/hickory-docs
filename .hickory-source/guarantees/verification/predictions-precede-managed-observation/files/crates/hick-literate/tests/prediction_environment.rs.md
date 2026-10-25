<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/tests/prediction_environment.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../contract.md)

Captured SHA-256: `c27d8d2492964ee78d2959aa2bad779f699f9b10578e1bf2f0347d1644045058`.

<book:entry path="crates/hick-literate/tests/prediction_environment.rs">
#![cfg(unix)]
//! Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
use hick_literate::prediction::{
    OutputEncoding, OutputRequest, environment::Profile, output_target, run_output_with_environment,
};
use hick_observe::prediction::*;
use hickory_executor::LocalExecutor;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
fn seal(ledger: &Ledger, source: &str, environment: BTreeMap<String, String>) -> OutputRequest {
    let actor = Actor {
        kind: "agent".into(),
        id: "environment-predictor".into(),
    };
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source)),
        inputs: digest(&json!([])).unwrap(),
        dependencies: Default::default(),
        environment,
    };
    let receipt = ledger
        .seal(
            0,
            Seal {
                scenario: Scenario {
                    family: "runtime-revalidation".into(),
                    inputs: json!([]),
                    requirement_reference: "Keep the sealed runtime and return 100".into(),
                    requirement: Predicate::Equals(json!(100)),
                    identity_known: true,
                },
                identity: identity.clone(),
                target: output_target("c", 2, OutputEncoding::Json),
                predicted: Predicate::Equals(json!(100)),
                rationale: "The same pinned interpreter must execute the source".into(),
            },
            actor.clone(),
        )
        .unwrap();
    OutputRequest {
        seal: receipt.id,
        request_id: "run".into(),
        actor,
        identity,
        encoding: OutputEncoding::Json,
    }
}
#[tokio::test]
#[ignore = "requires installed Python/debugpy; explicitly run the focused runtime witness"]
async fn runtime_is_revalidated_before_dispatch_and_before_settlement() {
    let project = tempfile::tempdir().unwrap();
    let parent = project.path().join(".hick-cache/adapters");
    std::fs::create_dir_all(&parent).unwrap();
    let adapter = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.hick-cache/adapters/python")
        .canonicalize()
        .unwrap();
    let link = parent.join("python");
    std::os::unix::fs::symlink(adapter, &link).unwrap();
    let profile = Profile::PythonDebugpy {
        project: project.path().into(),
    };
    let captured = profile.capture().await.unwrap();
    let source = "<hick:container name=\"c\" image=\"python\" />\n<hick:exec container=\"c\">python3 -c 'print(100)'</hick:exec>";
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let mut wrong = captured.clone();
    wrong.insert("output_python".into(), "0".repeat(64));
    let request = seal(&ledger, source, wrong);
    assert!(
        run_output_with_environment(
            &ledger,
            &request,
            source,
            Arc::new(LocalExecutor::new().unwrap()),
            Some(&profile)
        )
        .await
        .is_err()
    );
    assert!(ledger.history(None).unwrap().attempts.is_empty());
    let stable = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(stable.path()).unwrap();
    let request = seal(&ledger, source, captured.clone());
    let result = run_output_with_environment(
        &ledger,
        &request,
        source,
        Arc::new(LocalExecutor::new().unwrap()),
        Some(&profile),
    )
    .await
    .unwrap();
    assert_eq!(
        result.summary.classification,
        Classification::FirstAttemptVerified
    );
    let source = format!(
        "<hick:container name=\"c\" image=\"python\" />\n<hick:exec container=\"c\">python3 -c \"import os; os.unlink('{}'); print(100)\"</hick:exec>",
        link.display()
    );
    let another = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(another.path()).unwrap();
    let request = seal(&ledger, &source, captured);
    let result = run_output_with_environment(
        &ledger,
        &request,
        &source,
        Arc::new(LocalExecutor::new().unwrap()),
        Some(&profile),
    )
    .await
    .unwrap();
    assert_eq!(result.summary.classification, Classification::Unknown);
    assert!(
        ledger
            .promote(&result.summary.attempt, &request.identity, request.actor)
            .is_err()
    );
    assert!(!link.exists());
}
</book:entry>

</book:doc>
