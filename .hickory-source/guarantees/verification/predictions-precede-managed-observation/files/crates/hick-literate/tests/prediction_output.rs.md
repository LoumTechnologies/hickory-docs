<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/tests/prediction_output.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../contract.md)

Captured SHA-256: `090de22caa640850b4f5300402d3cd15283286c8440947c837d07e5002419145`.

<book:entry path="crates/hick-literate/tests/prediction_output.rs">
#![cfg(unix)]
//! Actual host-process witnesses for the bounded private output adapter.
//! LocalExecutor is not a sandbox/native/provider witness.
#[path = "support/prediction_privacy.rs"]
mod privacy;
use hick_literate::prediction::{OutputEncoding, OutputRequest, output_target, run_output};
use hick_observe::prediction::*;
use hickory_executor::{Executor, LocalExecutor};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};

fn actor() -> Actor {
    Actor {
        kind: "agent".into(),
        id: "predictor".into(),
    }
}
fn source(attempts: &[u64], corrected: bool, expected: u64) -> String {
    let allowance = if corrected { "100 - elapsed" } else { "100" };
    format!(
        "<hick:container name=\"c\" image=\"python\" />\n<hick:file path=\"answer.txt\"><hick:exec container=\"c\" show=\"output\">python3 -c 'attempts={attempts:?}; elapsed=0\nfor duration in attempts:\n allowance={allowance}\n elapsed += min(duration, allowance)\nprint(elapsed)'<hick:expect>{expected}\n</hick:expect></hick:exec></hick:file>"
    )
}
fn seal_request(
    ledger: &Ledger,
    source: &str,
    inputs: &[u64],
    prediction: u64,
    key: &str,
) -> OutputRequest {
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&json!(inputs)).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([(
            "test-host-runtime".into(),
            digest(&"local-python3").unwrap(),
        )]),
    };
    let seal = Seal {
        scenario: Scenario {
            family: "retry-total-deadline".into(),
            inputs: json!(inputs),
            requirement_reference: "total allowance <= 100ms".into(),
            requirement: Predicate::NumberAtMost(100.0),
            identity_known: true,
        },
        identity: identity.clone(),
        target: output_target("c", 2, OutputEncoding::Json),
        predicted: Predicate::Equals(json!(prediction)),
        rationale: "The total budget bounds all attempts".into(),
    };
    let receipt = ledger
        .seal(ledger.history(None).unwrap().generation, seal, actor())
        .unwrap();
    OutputRequest {
        seal: receipt.id,
        request_id: key.into(),
        actor: actor(),
        identity,
        encoding: OutputEncoding::Json,
    }
}
fn executor() -> Arc<LocalExecutor> {
    Arc::new(LocalExecutor::new().unwrap())
}

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn real_fault_correction_and_novel_inputs_retain_private_evidence() {
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let faulty = source(&[80, 80], false, 100);
    let request = seal_request(&ledger, &faulty, &[80, 80], 100, "fault");
    let result = run_output(&ledger, &request, &faulty, executor())
        .await
        .unwrap();
    assert_eq!(
        result.summary.classification,
        Classification::ExecutionRejected
    );
    assert_eq!(result.summary.prediction_matched, Some(false));
    assert_eq!(result.summary.original_requirement_satisfied, Some(false));
    privacy::assert_withheld(&result, json!(160));
    assert!(
        ledger
            .promote(&result.summary.attempt, &request.identity, actor())
            .is_err()
    );
    let retry_executor = executor();
    let retry = run_output(&ledger, &request, &faulty, retry_executor.clone())
        .await
        .unwrap();
    assert!(!retry.dispatched);
    assert_eq!(retry.summary.attempt, result.summary.attempt);
    assert!(retry_executor.transcripts().is_empty());

    let corrected = source(&[80, 80], true, 100);
    let request = seal_request(&ledger, &corrected, &[80, 80], 100, "repair");
    let repaired = run_output(&ledger, &request, &corrected, executor())
        .await
        .unwrap();
    assert_eq!(
        repaired.summary.classification,
        Classification::VerifiedAfterCorrection
    );
    assert!(
        ledger
            .promote(&repaired.summary.attempt, &request.identity, actor())
            .unwrap()
            .current_certification
    );
    assert!(!private.path().join("answer.txt").exists());
    for (inputs, expected, key) in [(vec![20, 30], 50, "short"), (vec![], 0, "empty")] {
        let text = source(&inputs, true, expected);
        let request = seal_request(&ledger, &text, &inputs, expected, key);
        let result = run_output(&ledger, &request, &text, executor())
            .await
            .unwrap();
        assert_eq!(
            result.summary.classification,
            Classification::FirstAttemptVerified
        );
    }
    assert_eq!(ledger.history(None).unwrap().attempts.len(), 4);
}

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn missing_stale_wrong_target_and_unsupported_paths_do_not_execute() {
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let text = source(&[80, 80], true, 100);
    let mut request = seal_request(&ledger, &text, &[80, 80], 100, "one");
    let runner = executor();
    let seal_id = request.seal.clone();
    request.seal = "absent".into();
    assert!(
        run_output(&ledger, &request, &text, runner.clone())
            .await
            .is_err()
    );
    request.seal = seal_id;
    assert!(
        run_output(&ledger, &request, &(text.clone() + "\n"), runner.clone())
            .await
            .is_err()
    );
    request.encoding = OutputEncoding::Text;
    assert!(
        run_output(&ledger, &request, &text, runner.clone())
            .await
            .is_err()
    );
    assert!(runner.transcripts().is_empty());
    assert!(ledger.history(None).unwrap().attempts.is_empty());
    for forbidden in [
        "<hick:capture at=\"x.py:1\" of=\"value\" />",
        "<hick:project />",
        "<hick:volume name=\"x\" />",
        "<hick:agent>run</hick:agent>",
    ] {
        let text = text.replace("<hick:expect>", &format!("{forbidden}<hick:expect>"));
        let request = seal_request(&ledger, &text, &[80, 80], 100, "unsupported");
        assert!(
            run_output(&ledger, &request, &text, runner.clone())
                .await
                .is_err()
        );
    }
    assert!(runner.transcripts().is_empty());
}

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn runner_expectation_failure_cannot_be_overridden_by_a_matching_prediction() {
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let text = source(&[20, 30], true, 99);
    let request = seal_request(&ledger, &text, &[20, 30], 50, "independent-check");
    let result = run_output(&ledger, &request, &text, executor())
        .await
        .unwrap();
    assert_eq!(result.summary.prediction_matched, Some(true));
    assert_eq!(result.summary.original_requirement_satisfied, Some(true));
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

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn execution_errors_and_unparseable_answers_are_nonpassing_private_states() {
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    for (command, key) in [
        ("printf hidden_runtime_diagnostic >&2; exit 1", "exec-error"),
        ("printf not_json", "decode-error"),
    ] {
        let text = format!(
            "<hick:container name=\"c\" image=\"alpine\" />\n<hick:exec container=\"c\">{command}</hick:exec>"
        );
        let request = seal_request(&ledger, &text, &[80, 80], 100, key);
        let result = run_output(&ledger, &request, &text, executor())
            .await
            .unwrap();
        assert_eq!(result.summary.classification, Classification::Unknown);
        let feedback = serde_json::to_string(&result).unwrap();
        assert!(!feedback.contains("hidden_runtime_diagnostic"));
        assert!(!feedback.contains("not_json"));
        assert!(
            ledger
                .admit(
                    &request.seal,
                    "replacement",
                    &request.identity,
                    false,
                    actor()
                )
                .is_err()
        );
        // The test owns the local executor: the failed command has exited.
        // Production adapters must obtain equivalent authoritative confirmation.
        ledger
            .interrupt(
                &result.summary.attempt,
                Actor {
                    kind: "host".into(),
                    id: "test-executor".into(),
                },
                Interruption::EvidenceUnavailable,
            )
            .unwrap();
        assert!(
            ledger
                .promote(&result.summary.attempt, &request.identity, actor())
                .is_err()
        );
    }
}
</book:entry>

</book:doc>
