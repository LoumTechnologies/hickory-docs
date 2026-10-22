#![cfg(unix)]
//! Synthetic authorization uses the same private output/debugger gate as retry.
use hick_literate::prediction::combined;
use hick_observe::prediction::*;
use hickory_executor::LocalExecutor;
use std::{path::PathBuf, sync::Arc};

#[path = "support/authorization.rs"]
mod fixture;
use fixture::*;

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

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn source_question_rebinds_a_repair_but_refuses_retargeted_capture_metadata_or_statement() {
    use hick_literate::prediction::{OutputEncoding, SourceQuestion};
    use sha2::{Digest, Sha256};
    let root = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(root.path()).unwrap();
    let original = request(&ledger, VULNERABLE, "question", 0);
    let question = SourceQuestion::capture(&original, VULNERABLE, OutputEncoding::Json).unwrap();
    let mut identity = original.identity.clone();
    identity.source = format!("{:x}", Sha256::digest(PATCHED.as_bytes()));
    let repaired = question.rebind(&identity, PATCHED).unwrap();
    let expected = request(&ledger, PATCHED, "expected-binding", 0);
    assert_eq!(repaired, expected.target);
    for changed in [
        PATCHED.replace("authorization.py:4", "authorization.py:3"),
        PATCHED.replace("of=\"allowed\"", "of=\"cross_tenant\""),
        PATCHED.replace("    return allowed\n", "    return allowed + 0\n"),
    ] {
        identity.source = format!("{:x}", Sha256::digest(changed.as_bytes()));
        assert!(question.rebind(&identity, &changed).is_err());
    }
    // Unrelated prose may move source byte spans while retaining the exact
    // generated capture location and source statement.
    let moved = format!("Additional review context\n{PATCHED}");
    identity.source = format!("{:x}", Sha256::digest(moved.as_bytes()));
    let rebound = question.rebind(&identity, &moved).unwrap();
    assert_ne!(rebound, expected.target);
}
