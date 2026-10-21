#![cfg(unix)]
//! Synthetic authorization uses the same private output/debugger gate as retry.
use hick_literate::prediction::{OutputEncoding, combined, output_target};
use hick_observe::prediction::*;
use hickory_executor::LocalExecutor;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

const VULNERABLE: &str = include_str!("../../../examples/security-authorization/vulnerable.md");
const PATCHED: &str = include_str!("../../../examples/security-authorization/patched.md");

fn actor() -> Actor {
    Actor {
        kind: "agent".into(),
        id: "synthetic-authorization-predictor".into(),
    }
}

fn request(ledger: &Ledger, source: &str, key: &str, prediction: i64) -> combined::Request {
    let inputs = json!({"tenant":"tenant-a", "record":"record-b",
        "records":{"record-a":"tenant-a", "record-b":"tenant-b"}});
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&inputs).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([(
            "fixture-runtime".into(),
            digest(&"local-python-and-configured-debugpy").unwrap(),
        )]),
    };
    let document = hick_lang::parse(source).unwrap();
    let line = document
        .all_tags()
        .into_iter()
        .find(|tag| tag.name == "exec")
        .unwrap()
        .source_line;
    let start = source.find("    return allowed\n").unwrap();
    let target = Target::Combined {
        output: Box::new(output_target("c", line, OutputEncoding::Json)),
        debugger: Box::new(Target::Debugger {
            source: "root".into(),
            source_revision: identity.source.clone(),
            start_byte: start as u64,
            end_byte: (start + "    return allowed\n".len()) as u64,
            expression: "allowed".into(),
            hit: 1,
            frame: Some("authorize".into()),
        }),
    };
    let fields = |value| {
        Predicate::Fields(BTreeMap::from([
            ("output".into(), Predicate::Equals(json!(value))),
            ("debugger".into(), Predicate::Equals(json!(value))),
        ]))
    };
    let receipt = ledger
        .seal(
            ledger.history(None).unwrap().generation,
            Seal {
                identity: identity.clone(),
                target: target.clone(),
                predicted: fields(prediction),
                rationale:
                    "Cross-tenant access must be denied while legitimate access stays available"
                        .into(),
                scenario: Scenario {
                    family: "synthetic-cross-tenant-authorization".into(),
                    inputs,
                    requirement_reference: "R-AUTH-1; R-LEGIT-1".into(),
                    requirement: fields(0),
                    identity_known: true,
                },
            },
            actor(),
        )
        .unwrap();
    combined::Request {
        seal: receipt.id,
        request_id: key.into(),
        actor: actor(),
        identity,
        target,
    }
}

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a configured Python debug adapter; run this focused witness explicitly"]
async fn vulnerable_and_patched_authorization_keep_one_requirement_and_prediction() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(hick_dap::discover("python", &project).is_some());
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let mut identities = Vec::new();
    for (source, key, repaired) in [
        (VULNERABLE, "vulnerable", false),
        (PATCHED, "patched", true),
    ] {
        let request = request(&ledger, source, key, 0);
        let result = combined::run(
            &ledger,
            &request,
            source,
            &project,
            Arc::new(LocalExecutor::new().unwrap()),
        )
        .await
        .unwrap();
        assert_eq!(result.summary.prediction_matched, Some(repaired));
        assert_eq!(
            result.summary.original_requirement_satisfied,
            Some(repaired)
        );
        if repaired {
            assert_eq!(
                result.summary.classification,
                Classification::VerifiedAfterCorrection
            );
            assert!(
                ledger
                    .promote(&result.summary.attempt, &request.identity, actor())
                    .unwrap()
                    .current_certification
            );
        } else {
            assert_eq!(
                result.summary.classification,
                Classification::ExecutionRejected
            );
            assert!(
                ledger
                    .promote(&result.summary.attempt, &request.identity, actor())
                    .is_err()
            );
        }
        identities.push(request.identity);
    }
    assert_ne!(identities[0].source, identities[1].source);
    assert_eq!(identities[0].inputs, identities[1].inputs);
    let history = ledger.history(Some(&identities[1])).unwrap();
    assert_eq!(history.attempts.len(), 2);
    assert_eq!(history.attempts[0].scenario, history.attempts[1].scenario);
    assert_eq!(history.attempts[0].classification, Classification::Stale);
    assert!(!history.attempts[0].current_certification);
    assert!(!private.path().join("authorization.py").exists());
}

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a configured Python debug adapter; run this focused witness explicitly"]
async fn correctly_predicting_the_vulnerability_does_not_accept_the_security_requirement() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let request = request(&ledger, VULNERABLE, "known-bug", 1);
    let result = combined::run(
        &ledger,
        &request,
        VULNERABLE,
        &project,
        Arc::new(LocalExecutor::new().unwrap()),
    )
    .await
    .unwrap();
    assert_eq!(result.summary.prediction_matched, Some(true));
    assert_eq!(result.summary.original_requirement_satisfied, Some(false));
    assert!(
        ledger
            .promote(&result.summary.attempt, &request.identity, actor())
            .is_err()
    );
}
