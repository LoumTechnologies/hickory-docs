<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep a prediction separate from the answer it is tested against

For: Reviewers deciding whether an agent demonstrated an independent prediction rather than copying an observed answer.

Predicting 160 and observing 160 still fails a requirement of at most 100. Changing the prediction after observing the result cannot earn first-attempt credit.

## What must happen

A managed attempt is admitted against a sealed prediction with exact source, input, dependency, environment and target identities. Matching a prediction and satisfying the original requirement are separate checks. Observed, stale, failed, replayed or unknown attempts cannot receive current certification. Exposure history remains attached to the scenario; changing source, names or sessions does not silently erase it. Human inspection records exposure before returning private answers. Interrupted attempts remain Unknown; corrupt history is refused.


## Limits of the evidence

This is a concise review contract. Adapter-specific clauses from the 990-line original remain in historical evidence and the coverage-debt ledger until individually reconciled. Ledger success does not certify host isolation, browser freshness, publication or authenticated evidence against a same-user attacker.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/predictions-precede-managed-observation.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Extract the core observable promise; separate historical checkpoints from current contract and explicitly retain adapter review debt.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

## Test the behavior: `private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr`

This is the maintained test fragment from `crates/hick-dap/tests/private_adapter.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-b2669b67dd81-private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr"></a>
### `private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr`

<storycode:copy id="f-b2669b67dd81-private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn private_adapter_process_diagnostics_do_not_reach_parent_stdout_or_stderr() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "private_adapter_child", "--nocapture"])
        .env("HICK_TEST_PRIVATE_ADAPTER_CHILD", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(SECRET));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(SECRET));
}

</storycode:copy>

## Test the behavior: `vulnerable_and_patched_authorization_keep_one_requirement_and_prediction`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_authorization.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-91952cf1f2e5-vulnerable_and_patched_authorization_keep_one_requirement_and_prediction"></a>
### `vulnerable_and_patched_authorization_keep_one_requirement_and_prediction`

<storycode:copy id="f-91952cf1f2e5-vulnerable_and_patched_authorization_keep_one_requirement_and_prediction">
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

</storycode:copy>

## Test the behavior: `correctly_predicting_the_vulnerability_does_not_accept_the_security_requirement`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_authorization.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-91952cf1f2e5-correctly_predicting_the_vulnerability_does_not_accept_the_security_requirement"></a>
### `correctly_predicting_the_vulnerability_does_not_accept_the_security_requirement`

<storycode:copy id="f-91952cf1f2e5-correctly_predicting_the_vulnerability_does_not_accept_the_security_requirement">
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

</storycode:copy>

## Test the behavior: `source_question_rebinds_a_repair_but_refuses_retargeted_capture_metadata_or_statement`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_authorization.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-91952cf1f2e5-source_question_rebinds_a_repair_but_refuses_retargeted_capture_metadata_or_statement"></a>
### `source_question_rebinds_a_repair_but_refuses_retargeted_capture_metadata_or_statement`

<storycode:copy id="f-91952cf1f2e5-source_question_rebinds_a_repair_but_refuses_retargeted_capture_metadata_or_statement">
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
</storycode:copy>

## Test the behavior: `same_revision_output_and_debugger_must_both_pass_before_promotion`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_combined.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-736f0187c52d-same_revision_output_and_debugger_must_both_pass_before_promotion"></a>
### `same_revision_output_and_debugger_must_both_pass_before_promotion`

<storycode:copy id="f-736f0187c52d-same_revision_output_and_debugger_must_both_pass_before_promotion">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a configured Python debug adapter; run the focused witness explicitly"]
async fn same_revision_output_and_debugger_must_both_pass_before_promotion() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(hick_dap::discover("python", &project).is_some());
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let mut prior = None;
    for (corrected, cap_output, key) in [
        (false, false, "fault"),
        (false, true, "output-only-fix"),
        (true, false, "repair"),
    ] {
        let text = source(corrected, cap_output);
        let request = request(&ledger, &text, key);
        let result = combined::run(
            &ledger,
            &request,
            &text,
            &project,
            Arc::new(LocalExecutor::new().unwrap()),
        )
        .await
        .unwrap();
        let feedback = serde_json::to_string(&result).unwrap();
        privacy::assert_withheld(&result, json!(160));
        assert!(!feedback.contains("captures"));
        assert!(!feedback.contains("stdout"));
        if corrected {
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
            assert_eq!(result.summary.prediction_matched, Some(false));
            assert_eq!(result.summary.original_requirement_satisfied, Some(false));
            assert!(
                ledger
                    .promote(&result.summary.attempt, &request.identity, actor())
                    .is_err()
            );
            assert_eq!(
                result.summary.classification,
                if cap_output {
                    Classification::Mismatch
                } else {
                    Classification::ExecutionRejected
                }
            );
        }
        if let Some(old_identity) = prior {
            assert_ne!(old_identity, request.identity.source);
        }
        prior = Some(request.identity.source.clone());
        let retry = Arc::new(LocalExecutor::new().unwrap());
        assert!(
            !combined::run(&ledger, &request, &text, &project, retry.clone())
                .await
                .unwrap()
                .dispatched
        );
        assert!(retry.transcripts().is_empty());
    }
    assert_eq!(ledger.history(None).unwrap().attempts.len(), 3);
    assert!(!private.path().join("deadline.py").exists());
}

</storycode:copy>

## Test the behavior: `supported_combination_reaches_the_shared_seal_gate_without_an_adapter`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_combined.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-736f0187c52d-supported_combination_reaches_the_shared_seal_gate_without_an_adapter"></a>
### `supported_combination_reaches_the_shared_seal_gate_without_an_adapter`

<storycode:copy id="f-736f0187c52d-supported_combination_reaches_the_shared_seal_gate_without_an_adapter">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn supported_combination_reaches_the_shared_seal_gate_without_an_adapter() {
    let directory = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(directory.path()).unwrap();
    let text = source(true, false);
    let mut request = request(&ledger, &text, "missing-seal");
    request.seal = "not-a-seal".into();
    let executor = Arc::new(LocalExecutor::new().unwrap());
    let error = combined::run(&ledger, &request, &text, directory.path(), executor.clone())
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Managed prediction admission refused")
    );
    assert!(executor.transcripts().is_empty());
    assert!(ledger.history(None).unwrap().attempts.is_empty());
}

</storycode:copy>

## Test the behavior: `unrelated_stdout_program_and_external_input_volume_refuse_before_execution`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_combined.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-736f0187c52d-unrelated_stdout_program_and_external_input_volume_refuse_before_execution"></a>
### `unrelated_stdout_program_and_external_input_volume_refuse_before_execution`

<storycode:copy id="f-736f0187c52d-unrelated_stdout_program_and_external_input_volume_refuse_before_execution">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn unrelated_stdout_program_and_external_input_volume_refuse_before_execution() {
    let directory = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(directory.path()).unwrap();
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (text, key) in [
        (
            source(true, false).replace("python3 work/deadline.py", "printf 100"),
            "unrelated-output",
        ),
        (
            source(true, false).replace("input=\".\"", "input=\"/tmp\""),
            "external-input",
        ),
    ] {
        let request = request(&ledger, &text, key);
        let executor = Arc::new(LocalExecutor::new().unwrap());
        assert!(
            combined::run(&ledger, &request, &text, &project, executor.clone())
                .await
                .is_err()
        );
        assert!(executor.transcripts().is_empty());
    }
    assert!(ledger.history(None).unwrap().attempts.is_empty());
}
</storycode:copy>

## Test the behavior: `seal`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_environment.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-b078fc2a53bf-seal"></a>
### `seal`

<storycode:copy id="f-b078fc2a53bf-seal">
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
</storycode:copy>

## Test the behavior: `real_fault_correction_and_novel_inputs_retain_private_evidence`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_output.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-1730eb6852af-real_fault_correction_and_novel_inputs_retain_private_evidence"></a>
### `real_fault_correction_and_novel_inputs_retain_private_evidence`

<storycode:copy id="f-1730eb6852af-real_fault_correction_and_novel_inputs_retain_private_evidence">
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

</storycode:copy>

## Test the behavior: `missing_stale_wrong_target_and_unsupported_paths_do_not_execute`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_output.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-1730eb6852af-missing_stale_wrong_target_and_unsupported_paths_do_not_execute"></a>
### `missing_stale_wrong_target_and_unsupported_paths_do_not_execute`

<storycode:copy id="f-1730eb6852af-missing_stale_wrong_target_and_unsupported_paths_do_not_execute">
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

</storycode:copy>

## Test the behavior: `runner_expectation_failure_cannot_be_overridden_by_a_matching_prediction`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_output.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-1730eb6852af-runner_expectation_failure_cannot_be_overridden_by_a_matching_prediction"></a>
### `runner_expectation_failure_cannot_be_overridden_by_a_matching_prediction`

<storycode:copy id="f-1730eb6852af-runner_expectation_failure_cannot_be_overridden_by_a_matching_prediction">
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

</storycode:copy>

## Test the behavior: `execution_errors_and_unparseable_answers_are_nonpassing_private_states`

This is the maintained test fragment from `crates/hick-literate/tests/prediction_output.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-1730eb6852af-execution_errors_and_unparseable_answers_are_nonpassing_private_states"></a>
### `execution_errors_and_unparseable_answers_are_nonpassing_private_states`

<storycode:copy id="f-1730eb6852af-execution_errors_and_unparseable_answers_are_nonpassing_private_states">
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
</storycode:copy>

## Test the behavior: `a_predictor_cannot_reuse_a_foreign_seal_or_read_foreign_prediction_text`

This is the maintained test fragment from `crates/hick-observe/src/prediction_scope_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a71053c6d40a-a_predictor_cannot_reuse_a_foreign_seal_or_read_foreign_prediction_text"></a>
### `a_predictor_cannot_reuse_a_foreign_seal_or_read_foreign_prediction_text`

<storycode:copy id="f-a71053c6d40a-a_predictor_cannot_reuse_a_foreign_seal_or_read_foreign_prediction_text">
use super::*;

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn a_predictor_cannot_reuse_a_foreign_seal_or_read_foreign_prediction_text() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let own = actor("agent");
    let foreign = Actor {
        kind: "agent".into(),
        id: "foreign-predictor".into(),
    };
    let prediction = seal(100, 'a');
    let scenario = prediction.scenario.key()?;
    let mut unrelated = prediction.clone();
    unrelated.scenario.family = "unrelated-private-question".into();
    ledger.observe(
        &unrelated.scenario,
        unrelated.identity.clone(),
        actor("human"),
        "PRIVATE-UNRELATED-SURFACE",
    )?;
    let sealed = ledger.seal(
        ledger.history(None)?.generation,
        prediction.clone(),
        own.clone(),
    )?;
    // A fresh request id must not defeat predictor ownership before any dispatch.
    assert!(
        ledger
            .admit_target(
                &sealed.id,
                "stolen",
                &prediction.identity,
                false,
                foreign.clone(),
                &prediction.target
            )
            .is_err()
    );
    assert!(ledger.history(None)?.attempts.is_empty());
    let mut copied = prediction.clone();
    copied.predicted = Predicate::Equals(json!("PRIVATE-HUMAN-COPIED-ANSWER"));
    copied.rationale = "PRIVATE-FOREIGN-RATIONALE".into();
    ledger.seal(ledger.history(None)?.generation, copied, foreign)?;
    let own_attempt = ledger.begin(&sealed.id, "own", &prediction.identity, false, own.clone())?;
    finish(&ledger, &own_attempt.id, json!(160), json!({}))?;
    let history = ledger.predictor_history(None, &own, &[scenario.clone()])?;
    assert_eq!(history.attempts.len(), 1);
    for entry in &history.ordering {
        if entry["event"]["kind"] != "sealed" {
            assert!(
                entry["event"].get("context").is_none(),
                "Projection must not add schema fields to non-seal events"
            );
        }
    }
    let encoded = serde_json::to_string(&history)?;
    assert!(!encoded.contains("PRIVATE-HUMAN-COPIED-ANSWER"));
    assert!(!encoded.contains("PRIVATE-FOREIGN-RATIONALE"));
    assert!(!encoded.contains("PRIVATE-UNRELATED-SURFACE"));
    let selected = ledger.select_predictor_history(&[scenario.clone()], &own)?;
    assert!(!serde_json::to_string(&selected)?.contains("PRIVATE-HUMAN-COPIED-ANSWER"));
    assert_eq!(selected.omitted_context_observations, 1);
    // Host/human audit retains the original account; projection never erases it.
    assert!(serde_json::to_string(&ledger.history(None)?)?.contains("PRIVATE-FOREIGN-RATIONALE"));
    Ok(())
}

</storycode:copy>

## Test the behavior: `legacy_cross_predictor_attempts_remain_historical_and_cannot_certify`

This is the maintained test fragment from `crates/hick-observe/src/prediction_scope_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a71053c6d40a-legacy_cross_predictor_attempts_remain_historical_and_cannot_certify"></a>
### `legacy_cross_predictor_attempts_remain_historical_and_cannot_certify`

<storycode:copy id="f-a71053c6d40a-legacy_cross_predictor_attempts_remain_historical_and_cannot_certify">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn legacy_cross_predictor_attempts_remain_historical_and_cannot_certify() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let prediction = seal(100, 'a');
    let sealed = ledger.seal(0, prediction.clone(), actor("agent"))?;
    let id = Uuid::new_v4().to_string();
    // Construct the old, previously admissible account through the journal's
    // existing commit writer; normal admission above now refuses it.
    ledger.transaction(|journal| {
        append(
            journal,
            Event::Started {
                id: id.clone(),
                seal: sealed.id,
                key: "legacy-owner".into(),
                replay: false,
                actor: Actor {
                    kind: "agent".into(),
                    id: "different-predictor".into(),
                },
            },
        )
    })?;
    let result = finish(&ledger, &id, json!(100), json!({}))?;
    assert_eq!(result.classification, Classification::Unknown);
    assert!(
        ledger
            .promote(&id, &prediction.identity, actor("host"))
            .is_err()
    );
    assert_eq!(
        ledger.history(Some(&prediction.identity))?.attempts[0].classification,
        Classification::Unknown
    );
    Ok(())
}
</storycode:copy>

## Test the behavior: `combined_predicate_requires_every_target_and_rejects_missing_extra_or_unbounded_fields`

This is the maintained test fragment from `crates/hick-observe/src/prediction_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a797b872866a-combined_predicate_requires_every_target_and_rejects_missing_extra_or_unbounded_fields"></a>
### `combined_predicate_requires_every_target_and_rejects_missing_extra_or_unbounded_fields`

<storycode:copy id="f-a797b872866a-combined_predicate_requires_every_target_and_rejects_missing_extra_or_unbounded_fields">
//! Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
use super::*;
use std::collections::BTreeMap;

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn combined_predicate_requires_every_target_and_rejects_missing_extra_or_unbounded_fields()
-> Result<()> {
    let predicate = Predicate::Fields(BTreeMap::from([
        ("output".into(), Predicate::NumberAtMost(100.0)),
        ("debugger".into(), Predicate::Equals(json!(20))),
    ]));
    assert!(predicate.matches(&json!({"output":100,"debugger":20})));
    for invalid in [
        json!({"output":160,"debugger":20}),
        json!({"output":100,"debugger":100}),
        json!({"output":100}),
        json!({"output":100,"debugger":20,"extra":true}),
        json!(null),
    ] {
        assert!(!predicate.matches(&invalid));
    }
    assert!(Predicate::Fields(BTreeMap::new()).validate().is_err());
    let mut nested = predicate;
    for _ in 0..10 {
        nested = Predicate::Fields(BTreeMap::from([("nested".into(), nested)]));
    }
    assert!(nested.validate().is_err());
    Ok(())
}
</storycode:copy>

## Test the behavior: `selected_history_preserves_failure_and_repair_without_exporting_private_answers`

This is the maintained test fragment from `crates/hick-observe/src/prediction_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a797b872866a-selected_history_preserves_failure_and_repair_without_exporting_private_answers"></a>
### `selected_history_preserves_failure_and_repair_without_exporting_private_answers`

<storycode:copy id="f-a797b872866a-selected_history_preserves_failure_and_repair_without_exporting_private_answers">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn selected_history_preserves_failure_and_repair_without_exporting_private_answers() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let mut unrelated = seal(100, 'c');
    unrelated.scenario.family = "private-unselected-family".into();
    ledger.observe(
        &unrelated.scenario,
        unrelated.identity.clone(),
        actor("human"),
        "private-human-view",
    )?;
    ledger.seal(ledger.history(None)?.generation, unrelated, actor("agent"))?;
    let original = seal(100, 'a');
    let scenario = original.scenario.key()?;
    let sealed = ledger.seal(
        ledger.history(None)?.generation,
        original.clone(),
        actor("agent"),
    )?;
    let first = ledger.begin(
        &sealed.id,
        "fault",
        &original.identity,
        false,
        actor("agent"),
    )?;
    finish(
        &ledger,
        &first.id,
        json!(160),
        json!({"stderr":"private-held-out-diagnostic", "adapter":"combined_live_v1", "captures":[{
            "spec":{"file":"fixture.py","line":5},"truncated":true,"problem":"private-capture-problem","moved":null,
            "hits":[{"index":1,"values":[{"value":"private-target-value"}],"context":{"run":"recorded-run","thread":1,"frame":2,"function":"retry","document_line":7,"source":"fixture.py","source_line":4,"breakpoint_bound":true,"private":"private-context-value"}}]
        }]}),
    )?;
    let repaired = seal(100, 'b');
    let sealed = ledger.seal(
        ledger.history(None)?.generation,
        repaired.clone(),
        actor("agent"),
    )?;
    let second = ledger.begin(
        &sealed.id,
        "repair",
        &repaired.identity,
        false,
        actor("agent"),
    )?;
    finish(
        &ledger,
        &second.id,
        json!(100),
        json!({"capture":"private-debugger-state"}),
    )?;
    ledger.promote(&second.id, &repaired.identity, actor("agent"))?;
    let before = ledger.history(None)?.generation;
    let selected = ledger.select_history(&[scenario.clone()])?;
    assert_eq!(selected.history.attempts.len(), 2);
    assert_eq!(
        selected.history.attempts[0].classification,
        Classification::Mismatch
    );
    assert_eq!(
        selected.history.attempts[1].classification,
        Classification::VerifiedAfterCorrection
    );
    assert!(!selected.history.attempts[1].current_certification);
    assert_eq!(selected.omitted_events, 2);
    assert_eq!(selected.debugger_contexts.len(), 1);
    assert_eq!(
        selected.debugger_contexts[0]["hits"][0]["context"]["run"],
        "recorded-run"
    );
    assert_eq!(selected.omitted_context_observations, 2);
    assert_eq!(selected.history.ordering[0]["sequence"], 3);
    assert!(
        selected
            .history
            .ordering
            .iter()
            .filter(|entry| entry["event"]["kind"] != "sealed")
            .all(|entry| entry["event"].get("context").is_none())
    );
    let text = serde_json::to_string(&selected)?;
    for private in [
        "private-held-out-diagnostic",
        "private-debugger-state",
        "private-unselected-family",
        "private-human-view",
        "private-capture-problem",
        "private-target-value",
        "private-context-value",
    ] {
        assert!(!text.contains(private));
    }
    assert_eq!(
        serde_json::from_str::<SelectedPredictionHistory>(&text)?.selected_scenarios,
        [scenario]
    );
    assert_eq!(ledger.history(None)?.generation, before);
    assert!(ledger.select_history(&[]).is_err());
    assert!(ledger.select_history(&["missing".into()]).is_err());
    assert!(
        ledger
            .reveal(&first.id, actor("agent"), "publication")
            .is_err()
    );
    Ok(())
}

</storycode:copy>

## Test the behavior: `admission_has_one_dispatch_owner_across_independent_hosts`

This is the maintained test fragment from `crates/hick-observe/src/prediction_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a797b872866a-admission_has_one_dispatch_owner_across_independent_hosts"></a>
### `admission_has_one_dispatch_owner_across_independent_hosts`

<storycode:copy id="f-a797b872866a-admission_has_one_dispatch_owner_across_independent_hosts">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn admission_has_one_dispatch_owner_across_independent_hosts() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let ledger = Ledger::open(directory.path())?;
    let prediction = seal(100, 'a');
    let sealed = ledger.seal(0, prediction.clone(), actor("agent"))?;
    let mut hosts = Vec::new();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
    for _ in 0..8 {
        let root = directory.path().to_owned();
        let id = sealed.id.clone();
        let identity = prediction.identity.clone();
        let target = prediction.target.clone();
        let barrier = barrier.clone();
        hosts.push(std::thread::spawn(move || {
            barrier.wait();
            Ledger::open(root)?.admit_target(
                &id,
                "dispatch-once",
                &identity,
                false,
                actor("agent"),
                &target,
            )
        }));
    }
    let admissions = hosts
        .into_iter()
        .map(|host| host.join().unwrap())
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(
        admissions
            .iter()
            .filter(|admission| admission.dispatch)
            .count(),
        1
    );
    assert!(
        admissions
            .iter()
            .all(|admission| admission.receipt.id == admissions[0].receipt.id)
    );
    assert!(
        !Ledger::open(directory.path())?
            .admit_target(
                &sealed.id,
                "dispatch-once",
                &prediction.identity,
                false,
                actor("agent"),
                &prediction.target
            )?
            .dispatch
    );
    assert!(
        ledger
            .admit_target(
                &sealed.id,
                "wrong-target",
                &prediction.identity,
                false,
                actor("agent"),
                &Target::Output {
                    execution: "different".into()
                }
            )
            .is_err()
    );
    let unknown = ledger.mark_unavailable(&admissions[0].receipt.id, actor("host"))?;
    assert_eq!(unknown.classification, Classification::Unknown);
    let reopened = Ledger::open(directory.path())?;
    assert!(
        reopened
            .admit(
                &sealed.id,
                "new-execution",
                &prediction.identity,
                false,
                actor("agent")
            )
            .is_err()
    );
    assert!(
        !reopened
            .admit(
                &sealed.id,
                "dispatch-once",
                &prediction.identity,
                false,
                actor("agent")
            )?
            .dispatch
    );
    reopened.interrupt(
        &admissions[0].receipt.id,
        actor("host"),
        Interruption::HostInterrupted,
    )?;
    assert!(
        reopened
            .admit(
                &sealed.id,
                "after-confirmed-stop",
                &prediction.identity,
                false,
                actor("agent")
            )?
            .dispatch
    );
    Ok(())
}

</storycode:copy>

## Test the behavior: `independent_execution_check_is_durable_and_cannot_be_replaced`

This is the maintained test fragment from `crates/hick-observe/src/prediction_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a797b872866a-independent_execution_check_is_durable_and_cannot_be_replaced"></a>
### `independent_execution_check_is_durable_and_cannot_be_replaced`

<storycode:copy id="f-a797b872866a-independent_execution_check_is_durable_and_cannot_be_replaced">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn independent_execution_check_is_durable_and_cannot_be_replaced() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let ledger = Ledger::open(directory.path())?;
    let prediction = seal(100, 'a');
    let sealed = ledger.seal(0, prediction.clone(), actor("agent"))?;
    let attempt = ledger.begin(
        &sealed.id,
        "check",
        &prediction.identity,
        false,
        actor("agent"),
    )?;
    let result = ObservedResult {
        identity: prediction.identity.clone(),
        target: prediction.target.clone(),
        actual: json!(100),
        evidence: json!({ "check": "failed" }),
    };
    let summary = ledger.finish_checked(&attempt.id, result.clone(), false)?;
    assert_eq!(summary.classification, Classification::ExecutionRejected);
    assert_eq!(summary.prediction_matched, Some(true));
    assert_eq!(summary.original_requirement_satisfied, Some(true));
    assert!(
        ledger
            .finish_checked(&attempt.id, result.clone(), true)
            .is_err()
    );
    assert!(
        Ledger::open(directory.path())?
            .promote(&attempt.id, &prediction.identity, actor("agent"))
            .is_err()
    );
    assert_eq!(
        ledger
            .finish_checked(&attempt.id, result, false)?
            .classification,
        Classification::ExecutionRejected
    );
    Ok(())
}
</storycode:copy>

## Test the behavior: `future_seals_bind_prior_stop_and_context_and_never_claim_independent_first_success`

This is the maintained test fragment from `crates/hick-observe/src/prediction_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a797b872866a-future_seals_bind_prior_stop_and_context_and_never_claim_independent_first_success"></a>
### `future_seals_bind_prior_stop_and_context_and_never_claim_independent_first_success`

<storycode:copy id="f-a797b872866a-future_seals_bind_prior_stop_and_context_and_never_claim_independent_first_success">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn future_seals_bind_prior_stop_and_context_and_never_claim_independent_first_success() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let mut question = seal(20, 'a');
    question.target = Target::Debugger {
        source: "root".into(),
        source_revision: question.identity.source.clone(),
        start_byte: 0,
        end_byte: 10,
        expression: "allowance".into(),
        hit: 2,
        frame: Some("retry".into()),
    };
    let visible = json!({"prior_visible":"private previous discussion"});
    let stop = FutureStop {
        run: "retained-run".into(),
        thread: 1,
        frame: 2,
        hit: 1,
        prior_visible_digest: digest(&visible)?,
    };
    let receipt = ledger.seal_future(
        0,
        question.clone(),
        actor("agent"),
        stop.clone(),
        visible.clone(),
    )?;
    assert!(
        ledger
            .begin(
                &receipt.id,
                "initial",
                &question.identity,
                false,
                actor("agent")
            )
            .is_err()
    );
    let mut wrong = stop.clone();
    wrong.run = "another-process".into();
    assert!(
        ledger
            .admit_future_target(
                &receipt.id,
                "wrong",
                &question.identity,
                actor("agent"),
                &question.target,
                &wrong
            )
            .is_err()
    );
    let admission = ledger.admit_future_target(
        &receipt.id,
        "future",
        &question.identity,
        actor("agent"),
        &question.target,
        &stop,
    )?;
    assert!(admission.dispatch);
    assert!(
        !ledger
            .admit_future_target(
                &receipt.id,
                "future",
                &question.identity,
                actor("agent"),
                &question.target,
                &stop
            )?
            .dispatch
    );
    let observation = ObservedResult {
        identity: question.identity.clone(),
        target: question.target.clone(),
        actual: json!(20),
        evidence: json!({}),
    };
    assert!(
        ledger
            .finish(&admission.receipt.id, observation.clone())
            .is_err()
    );
    assert!(
        ledger
            .finish_future_checked(&admission.receipt.id, observation.clone(), true, &wrong)
            .is_err()
    );
    let summary = ledger.finish_future_checked(&admission.receipt.id, observation, true, &stop)?;
    assert_eq!(
        summary.classification,
        Classification::InformedPredictionVerified
    );
    assert!(
        ledger
            .promote(&summary.attempt, &question.identity, actor("agent"))?
            .current_certification
    );
    let public = serde_json::to_string(&ledger.select_history(&[question.scenario.key()?])?)?;
    assert!(public.contains("future_stop"));
    assert!(!public.contains("private previous discussion"));
    // Context digest and next-hit claims are checked before writing any seal.
    assert!(
        ledger
            .seal_future(
                ledger.history(None)?.generation,
                question,
                actor("agent"),
                stop,
                json!({"changed":true})
            )
            .is_err()
    );
    Ok(())
}

</storycode:copy>

## Test the behavior: `request`

This is the maintained test fragment from `crates/hickory-agent/tests/prediction_verification.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-575d4e4459ff-request"></a>
### `request`

<storycode:copy id="f-575d4e4459ff-request">
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
</storycode:copy>

## Test the behavior: `options`

This is the maintained test fragment from `crates/hickory-cli/src/engine/client_prediction_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-014c7e3acc59-options"></a>
### `options`

<storycode:copy id="f-014c7e3acc59-options">
//! Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
use super::*;
use axum::{
    Json, Router,
    routing::{get, post},
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
fn options() -> Attach {
    let mut opts = Attach::new("/host-captured/doc.md".into(), crate::ExecutorChoice::Local);
    opts.prediction = Some(crate::serve::PrivatePredictionStartup {
        ledger_root: "/host-private/ledger".into(),
        seal: "sealed-question".into(),
        request_id: "request".into(),
        identity: hick_observe::prediction::Identity {
            source: "a".repeat(64),
            inputs: "b".repeat(64),
            dependencies: BTreeMap::new(),
            environment: BTreeMap::from([("runtime".into(), "c".repeat(64))]),
        },
        target: hick_observe::prediction::Target::Output {
            execution: "captured-cell".into(),
        },
        actor: hick_observe::prediction::Actor {
            kind: "agent".into(),
            id: "host-bound-predictor".into(),
        },
        encoding: hick_literate::prediction::OutputEncoding::Json,
        question: None,
        runtime: None,
        inputs: None,
        retained: None,
    });
    opts
}
</storycode:copy>

## Test the behavior: `advertised_private_whole_preview_never_writes_or_executes_and_refuses_stale_source`

This is the maintained test fragment from `crates/hickory-cli/src/serve/private_prediction_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-7cb981639695-advertised_private_whole_preview_never_writes_or_executes_and_refuses_stale_source"></a>
### `advertised_private_whole_preview_never_writes_or_executes_and_refuses_stale_source`

<storycode:copy id="f-7cb981639695-advertised_private_whole_preview_never_writes_or_executes_and_refuses_stale_source">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn advertised_private_whole_preview_never_writes_or_executes_and_refuses_stale_source() {
    let (_root, _private, state, ledger, marker, _) = fixture().await;
    let bound = state.private_prediction.clone().unwrap();
    let source = std::fs::read_to_string(&bound.path).unwrap();
    let snapshot = hickory_agent::hashline::LineIndex::new(&source).snapshot;
    let tools = catalogue(false);
    let edit = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "edit_doc")
        .unwrap();
    let whole = edit["inputSchema"]["properties"]["whole"]["enum"][0].clone();
    assert_eq!(whole, "true");
    let proposed = format!("{source}\n\nA proposed explanation.\n");
    let arguments = json!({"whole":whole,"snapshot":snapshot,"input":proposed});
    let Json(result) = preview(
        State(state.clone()),
        Json(json!({"name":"edit_doc","arguments":arguments})),
    )
    .await
    .unwrap();
    assert_eq!(result["before"], source);
    assert_eq!(result["after"], proposed);
    assert_eq!(std::fs::read_to_string(&bound.path).unwrap(), source);
    assert!(!marker.exists());
    assert!(ledger.history(None).unwrap().attempts.is_empty());
    assert!(preview(State(state.clone()), Json(json!({"name":"edit_doc","arguments":{"snapshot":snapshot,"run":"literal source text","input":"replacement"}}))).await.is_err());
    std::fs::write(
        &bound.path,
        format!("{source}\nChanged by another owner.\n"),
    )
    .unwrap();
    assert!(
        preview(
            State(state),
            Json(json!({"name":"edit_doc","arguments":arguments}))
        )
        .await
        .is_err()
    );
    assert!(!marker.exists());
}
</storycode:copy>

## Test the behavior: `hash`

This is the maintained test fragment from `crates/hickory-cli/tests/prediction_http.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-8a0b4d28c345-hash"></a>
### `hash`

<storycode:copy id="f-8a0b4d28c345-hash">
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
</storycode:copy>

## Test the behavior: `dormant_question_requires_a_submitted_prediction_and_reseals_repair_without_exposing_answers`

This is the maintained test fragment from `crates/hickory-cli/tests/prediction_http.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-8a0b4d28c345-dormant_question_requires_a_submitted_prediction_and_reseals_repair_without_exposing_answe"></a>
### `dormant_question_requires_a_submitted_prediction_and_reseals_repair_without_exposing_answers`

<storycode:copy id="f-8a0b4d28c345-dormant_question_requires_a_submitted_prediction_and_reseals_repair_without_exposing_answe">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test]
async fn dormant_question_requires_a_submitted_prediction_and_reseals_repair_without_exposing_answers()
 {
    let root = tempfile::tempdir().unwrap();
    let private = tempfile::tempdir().unwrap();
    let template_root = tempfile::tempdir().unwrap();
    let template_ledger = Ledger::open(template_root.path()).unwrap();
    let marker = root.path().join("dispatched.marker");
    let marker_path = marker.to_string_lossy().replace('\'', "'\"'\"'");
    let faulty = source(160).replace(
        "printf 160",
        &format!("printf dispatched > '{marker_path}'; printf 160"),
    );
    let mut initial = request(&template_ledger, &faulty, "host-question-template");
    let question: Seal = serde_json::from_value(
        template_ledger.history(None).unwrap().ordering[0]["event"]["seal"].clone(),
    )
    .unwrap();
    initial.seal.clear();
    let ledger = Arc::new(Ledger::open(private.path()).unwrap());
    let path = root.path().join("doc.md");
    std::fs::write(&path, &faulty).unwrap();
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
            request: initial,
            encoding: OutputEncoding::Json,
            question: Some(question.scenario),
            runtime: None,
            inputs: None,
            retained: None,
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
    let state: Value = client
        .get(format!("{base}/prediction/state"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let doc = state["document"].as_str().unwrap();
    let unsealed: Value = client
        .post(format!("{base}/docs/{doc}/run"))
        .json(&json!({"expected_source_hash":hash(faulty.as_bytes())}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let unsealed_run = unsealed["run_id"].as_str().unwrap();
    let mut before = Value::Null;
    for _ in 0..100 {
        before = client
            .get(format!("{base}/runs/{unsealed_run}"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if before["status"] != "running" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(before["status"], "failed");
    assert!(ledger.history(None).unwrap().attempts.is_empty());
    assert!(!marker.exists());
    for (index, text) in [faulty.clone(), source(100)].into_iter().enumerate() {
        if index == 1 {
            let edit:Value=client.post(format!("{base}/prediction/tool")).json(&json!({"name":"edit_doc","args":[["whole","true"],["snapshot",hash(faulty.as_bytes())]],"input":text})).send().await.unwrap().json().await.unwrap();
            assert_eq!(edit["ok"], true);
        }
        // Every seal binds the captured source. Requirements/identity/actors are
        // absent from the tool schema and cannot be replaced by the caller.
        assert_eq!(client.post(format!("{base}/prediction/seal")).json(&json!({"expected_source_hash":hash(text.as_bytes()),"predicted":{"kind":"equals","value":100},"rationale":"The same total budget still applies","actor":{"kind":"human","id":"forged"}})).send().await.unwrap().status().as_u16(),422);
        let sealed:Value=client.post(format!("{base}/prediction/seal")).json(&json!({"expected_source_hash":hash(text.as_bytes()),"predicted":{"kind":"equals","value":100},"rationale":"The same total budget still applies"})).send().await.unwrap().json().await.unwrap();
        assert_eq!(sealed["executed"], false);
        let accepted: Value = client
            .post(format!("{base}/docs/{doc}/check"))
            .json(&json!({"expected_source_hash":hash(text.as_bytes())}))
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
        assert_eq!(result["status"], if index == 0 { "failed" } else { "ok" });
        assert_eq!(
            result["blocks"][0]["summary"]["classification"],
            if index == 0 {
                "execution_rejected"
            } else {
                "verified_after_correction"
            }
        );
        assert!(!root.path().join("answer.txt").exists());
        assert!(
            marker.exists(),
            "Only sealed dispatch may execute the command"
        );
    }
    let history = ledger.history(None).unwrap();
    assert_eq!(history.attempts.len(), 2);
    assert_eq!(
        history
            .ordering
            .iter()
            .filter(|entry| entry["event"]["kind"] == "sealed")
            .count(),
        2
    );
    assert!(
        history
            .attempts
            .iter()
            .all(|attempt| !attempt.current_certification)
    );
    server.abort();
}
</storycode:copy>

## Implementation chapters

These chapters keep this behavior's larger implementation readable without duplicating shared code.

- [`apps/web/src/components/AgentPicker.tsx`](implementation/apps/web/src/components/AgentPicker.tsx.md#output-2cb1fb2fe5c8)
- [`apps/web/src/components/FeatureSettings.tsx`](implementation/apps/web/src/components/FeatureSettings.tsx.md#output-c9651595b18d)
- [`apps/web/src/components/NativeWorkflowTour.tsx`](implementation/apps/web/src/components/NativeWorkflowTour.tsx.md#output-9785a32c4177)
- [`apps/web/src/components/privatePredictionCoach.css`](implementation/apps/web/src/components/privatePredictionCoach.css.md#output-8905b58855c2)
- [`apps/web/src/main.tsx`](implementation/apps/web/src/main.tsx.md#output-44decff65943)
- [`crates/hick-dap/tests/private_adapter.rs`](implementation/crates/hick-dap/tests/private_adapter.rs.md#output-b2669b67dd81)
- [`crates/hick-literate/examples/authorization_evidence.rs`](implementation/crates/hick-literate/examples/authorization_evidence.rs.md#output-f34cc4987708)
- [`crates/hick-literate/src/prediction.rs`](implementation/crates/hick-literate/src/prediction.rs.md#output-72f0d919b421)
- [`crates/hick-literate/src/prediction/combined.rs`](implementation/crates/hick-literate/src/prediction/combined.rs.md#output-9b7bf060a506)
- [`crates/hick-literate/src/prediction/debugger.rs`](implementation/crates/hick-literate/src/prediction/debugger.rs.md#output-34958664eedc)
- [`crates/hick-literate/src/prediction/debugger/tests.rs`](implementation/crates/hick-literate/src/prediction/debugger/tests.rs.md#output-486f17fcecc4)
- [`crates/hick-literate/src/prediction/environment.rs`](implementation/crates/hick-literate/src/prediction/environment.rs.md#output-a7fe04771e89)
- [`crates/hick-literate/src/prediction/interactive.rs`](implementation/crates/hick-literate/src/prediction/interactive.rs.md#output-6cb50a5a2e5e)
- [`crates/hick-literate/src/prediction/python_identity.py`](implementation/crates/hick-literate/src/prediction/python_identity.py.md#output-aafa574a1866)
- [`crates/hick-literate/src/prediction/source_question.rs`](implementation/crates/hick-literate/src/prediction/source_question.rs.md#output-188d542b7351)
- [`crates/hick-literate/tests/prediction_authorization.rs`](implementation/crates/hick-literate/tests/prediction_authorization.rs.md#output-91952cf1f2e5)
- [`crates/hick-literate/tests/prediction_combined.rs`](implementation/crates/hick-literate/tests/prediction_combined.rs.md#output-736f0187c52d)
- [`crates/hick-literate/tests/prediction_environment.rs`](implementation/crates/hick-literate/tests/prediction_environment.rs.md#output-b078fc2a53bf)
- [`crates/hick-literate/tests/prediction_output.rs`](implementation/crates/hick-literate/tests/prediction_output.rs.md#output-1730eb6852af)
- [`crates/hick-observe/src/prediction_publication.rs`](implementation/crates/hick-observe/src/prediction_publication.rs.md#output-e7bf0b628579)
- [`crates/hick-observe/src/prediction_scope.rs`](implementation/crates/hick-observe/src/prediction_scope.rs.md#output-a3362438d4c1)
- [`crates/hick-observe/src/prediction_scope_tests.rs`](implementation/crates/hick-observe/src/prediction_scope_tests.rs.md#output-a71053c6d40a)
- [`crates/hick-observe/src/prediction_types.rs`](implementation/crates/hick-observe/src/prediction_types.rs.md#output-ca9436bd58e3)
- [`crates/hickory-agent/src/lib.rs`](implementation/crates/hickory-agent/src/lib.rs.md#output-a448ddea64c9)
- [`crates/hickory-agent/src/tools/dispatch.rs`](implementation/crates/hickory-agent/src/tools/dispatch.rs.md#output-7798872b38ba)
- [`crates/hickory-agent/src/tools/prediction.rs`](implementation/crates/hickory-agent/src/tools/prediction.rs.md#output-fe56bb99d8b7)
- [`crates/hickory-agent/tests/prediction_verification.rs`](implementation/crates/hickory-agent/tests/prediction_verification.rs.md#output-575d4e4459ff)
- [`crates/hickory-cli/src/engine/private_policy.rs`](implementation/crates/hickory-cli/src/engine/private_policy.rs.md#output-6a8af15fa213)
- [`crates/hickory-cli/src/mcp_stdio.rs`](implementation/crates/hickory-cli/src/mcp_stdio.rs.md#output-14b6d9cbf226)
- [`crates/hickory-cli/src/python_agent/host.rs`](implementation/crates/hickory-cli/src/python_agent/host.rs.md#output-d88916f0dc97)
- [`crates/hickory-cli/src/python_agent/private.rs`](implementation/crates/hickory-cli/src/python_agent/private.rs.md#output-40fff9d31e20)
- [`crates/hickory-cli/src/serve/private_practice.rs`](implementation/crates/hickory-cli/src/serve/private_practice.rs.md#output-391b915a64d3)
- [`crates/hickory-cli/src/serve/private_prediction_acp.rs`](implementation/crates/hickory-cli/src/serve/private_prediction_acp.rs.md#output-80eb5d06ac66)
- [`crates/hickory-cli/src/serve/private_prediction_acp_tests.rs`](implementation/crates/hickory-cli/src/serve/private_prediction_acp_tests.rs.md#output-bbc7e4e93ac2)
- [`crates/hickory-cli/src/serve/private_prediction_agent.rs`](implementation/crates/hickory-cli/src/serve/private_prediction_agent.rs.md#output-f90967235577)
- [`crates/hickory-cli/src/serve/private_prediction_debugger.rs`](implementation/crates/hickory-cli/src/serve/private_prediction_debugger.rs.md#output-ad517bf17a52)
- [`crates/hickory-cli/src/serve/private_prediction_recovery.rs`](implementation/crates/hickory-cli/src/serve/private_prediction_recovery.rs.md#output-ed1889813c82)
- [`crates/hickory-cli/src/serve/private_prediction_startup.rs`](implementation/crates/hickory-cli/src/serve/private_prediction_startup.rs.md#output-31f27f45f135)
- [`crates/hickory-cli/tests/prediction_engine.rs`](implementation/crates/hickory-cli/tests/prediction_engine.rs.md#output-f733448ba92e)
- [`crates/hickory-cli/tests/prediction_http.rs`](implementation/crates/hickory-cli/tests/prediction_http.rs.md#output-8a0b4d28c345)
<storycode:include file="implementation/apps/web/src/components/AgentPicker.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/FeatureSettings.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/NativeWorkflowTour.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/privatePredictionCoach.css.md" />
<storycode:include file="implementation/apps/web/src/main.tsx.md" />
<storycode:include file="implementation/crates/hick-dap/tests/private_adapter.rs.md" />
<storycode:include file="implementation/crates/hick-literate/examples/authorization_evidence.rs.md" />
<storycode:include file="implementation/crates/hick-literate/src/prediction.rs.md" />
<storycode:include file="implementation/crates/hick-literate/src/prediction/combined.rs.md" />
<storycode:include file="implementation/crates/hick-literate/src/prediction/debugger.rs.md" />
<storycode:include file="implementation/crates/hick-literate/src/prediction/debugger/tests.rs.md" />
<storycode:include file="implementation/crates/hick-literate/src/prediction/environment.rs.md" />
<storycode:include file="implementation/crates/hick-literate/src/prediction/interactive.rs.md" />
<storycode:include file="implementation/crates/hick-literate/src/prediction/python_identity.py.md" />
<storycode:include file="implementation/crates/hick-literate/src/prediction/source_question.rs.md" />
<storycode:include file="implementation/crates/hick-literate/tests/prediction_authorization.rs.md" />
<storycode:include file="implementation/crates/hick-literate/tests/prediction_combined.rs.md" />
<storycode:include file="implementation/crates/hick-literate/tests/prediction_environment.rs.md" />
<storycode:include file="implementation/crates/hick-literate/tests/prediction_output.rs.md" />
<storycode:include file="implementation/crates/hick-observe/src/prediction_publication.rs.md" />
<storycode:include file="implementation/crates/hick-observe/src/prediction_scope.rs.md" />
<storycode:include file="implementation/crates/hick-observe/src/prediction_scope_tests.rs.md" />
<storycode:include file="implementation/crates/hick-observe/src/prediction_types.rs.md" />
<storycode:include file="implementation/crates/hickory-agent/src/lib.rs.md" />
<storycode:include file="implementation/crates/hickory-agent/src/tools/dispatch.rs.md" />
<storycode:include file="implementation/crates/hickory-agent/src/tools/prediction.rs.md" />
<storycode:include file="implementation/crates/hickory-agent/tests/prediction_verification.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/engine/private_policy.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/mcp_stdio.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/python_agent/host.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/python_agent/private.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/private_practice.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/private_prediction_acp.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/private_prediction_acp_tests.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/private_prediction_agent.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/private_prediction_debugger.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/private_prediction_recovery.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/private_prediction_startup.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/tests/prediction_engine.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/tests/prediction_http.rs.md" />

## Implementation review

The shared prediction ledger records sealing, admission, execution outcome, exposure and promotion as distinct operations. Host adapters determine identities and keep private answers outside model-facing context. Native and browser adapters therefore need their own boundary witnesses in addition to ledger tests.

### Implementation

- [apps/agent/src/hickory_agent_worker/instructions.py](../../../mechanisms/apps/agent/src/hickory_agent_worker/instructions.py.md#output-4915d386d720)
- [apps/desktop/src-tauri/src/lib.rs](../../../mechanisms/apps/desktop/src-tauri/src/lib.rs.md#output-42caba993099)
- [apps/desktop/src-tauri/src/server.rs](../../../mechanisms/apps/desktop/src-tauri/src/server.rs.md#output-821f8d0a36be)
- [apps/web/src/components/AgentChanges.tsx](../../../mechanisms/apps/web/src/components/AgentChanges.tsx.md#output-d82ea3dcdcf9)
- [apps/web/src/components/AgentPicker.tsx](implementation/apps/web/src/components/AgentPicker.tsx.md#output-2cb1fb2fe5c8)
- [apps/web/src/components/ChatDock.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-57557cac78c8)
- [apps/web/src/components/FeatureSettings.tsx](implementation/apps/web/src/components/FeatureSettings.tsx.md#output-c9651595b18d)
- [apps/web/src/components/NativeWorkflowTour.tsx](implementation/apps/web/src/components/NativeWorkflowTour.tsx.md#output-9785a32c4177)
- [apps/web/src/components/PrivatePredictionCoach.tsx](../../../mechanisms/apps/web/src/components/PrivatePredictionCoach.tsx.md#output-f32ded7fd60b)
- [apps/web/src/components/privatePredictionCoach.css](implementation/apps/web/src/components/privatePredictionCoach.css.md#output-8905b58855c2)
- [apps/web/src/lib/nativeWindowActions.ts](../../../mechanisms/apps/web/src/lib/nativeWindowActions.ts.md#output-14de373a5fd7)
- [apps/web/src/lib/uiActionBridge.tsx](../../../mechanisms/apps/web/src/lib/uiActionBridge.tsx.md#output-5571d9f178fc)
- [apps/web/src/lib/uiActions.ts](../../../mechanisms/apps/web/src/lib/uiActions.ts.md#output-0cc430d9eb03)
- [apps/web/src/main.tsx](implementation/apps/web/src/main.tsx.md#output-44decff65943)
- [crates/hick-context-store/src/host.rs](../../../mechanisms/crates/hick-context-store/src/host.rs.md#output-c4e1ee6d700e)
- [crates/hick-dap/src/interactive_capture.rs](../../../mechanisms/crates/hick-dap/src/interactive_capture.rs.md#output-c26b35fd706d)
- [crates/hick-lang-wasm/src/prediction.rs](../../../mechanisms/crates/hick-lang-wasm/src/prediction.rs.md#output-d27c09b74da1)
- [crates/hick-literate/examples/authorization_evidence.rs](implementation/crates/hick-literate/examples/authorization_evidence.rs.md#output-f34cc4987708)
- [crates/hick-literate/src/prediction.rs](implementation/crates/hick-literate/src/prediction.rs.md#output-72f0d919b421)
- [crates/hick-literate/src/prediction/combined.rs](implementation/crates/hick-literate/src/prediction/combined.rs.md#output-9b7bf060a506)
- [crates/hick-literate/src/prediction/debugger.rs](implementation/crates/hick-literate/src/prediction/debugger.rs.md#output-34958664eedc)
- [crates/hick-literate/src/prediction/debugger/tests.rs](implementation/crates/hick-literate/src/prediction/debugger/tests.rs.md#output-486f17fcecc4)
- [crates/hick-literate/src/prediction/environment.rs](implementation/crates/hick-literate/src/prediction/environment.rs.md#output-a7fe04771e89)
- [crates/hick-literate/src/prediction/gate.rs](../../../mechanisms/crates/hick-literate/src/prediction/gate.rs.md#output-bcac6949feb6)
- [crates/hick-literate/src/prediction/interactive.rs](implementation/crates/hick-literate/src/prediction/interactive.rs.md#output-6cb50a5a2e5e)
- [crates/hick-literate/src/prediction/python_identity.py](implementation/crates/hick-literate/src/prediction/python_identity.py.md#output-aafa574a1866)
- [crates/hick-literate/src/prediction/source_question.rs](implementation/crates/hick-literate/src/prediction/source_question.rs.md#output-188d542b7351)
- [crates/hick-observe/src/prediction.rs](../../../mechanisms/crates/hick-observe/src/prediction.rs.md#output-f5f25bde8505)
- [crates/hick-observe/src/prediction_publication.rs](implementation/crates/hick-observe/src/prediction_publication.rs.md#output-e7bf0b628579)
- [crates/hick-observe/src/prediction_scope.rs](implementation/crates/hick-observe/src/prediction_scope.rs.md#output-a3362438d4c1)
- [crates/hick-observe/src/prediction_types.rs](implementation/crates/hick-observe/src/prediction_types.rs.md#output-ca9436bd58e3)
- [crates/hickory-agent/src/lib.rs](implementation/crates/hickory-agent/src/lib.rs.md#output-a448ddea64c9)
- [crates/hickory-agent/src/llm_openai.rs](../../../mechanisms/crates/hickory-agent/src/llm_openai.rs.md#output-0150d11c8f60)
- [crates/hickory-agent/src/provider.rs](../../../mechanisms/crates/hickory-agent/src/provider.rs.md#output-2be2d28e151b)
- [crates/hickory-agent/src/tools/dispatch.rs](implementation/crates/hickory-agent/src/tools/dispatch.rs.md#output-7798872b38ba)
- [crates/hickory-agent/src/tools/prediction.rs](implementation/crates/hickory-agent/src/tools/prediction.rs.md#output-fe56bb99d8b7)
- [crates/hickory-agent/src/tools/preview.rs](../../../mechanisms/crates/hickory-agent/src/tools/preview.rs.md#output-256c54fefd54)
- [crates/hickory-cli/examples/engine_client.rs](../../../mechanisms/crates/hickory-cli/examples/engine_client.rs.md#output-2ffdd2bc63e0)
- [crates/hickory-cli/src/engine/client.rs](../../../mechanisms/crates/hickory-cli/src/engine/client.rs.md#output-5c9c7cadce49)
- [crates/hickory-cli/src/engine/daemon.rs](../../../mechanisms/crates/hickory-cli/src/engine/daemon.rs.md#output-49126839fdb7)
- [crates/hickory-cli/src/engine/mcp.rs](../../../mechanisms/crates/hickory-cli/src/engine/mcp.rs.md#output-d68d35ce1f7a)
- [crates/hickory-cli/src/engine/mod.rs](../../../mechanisms/crates/hickory-cli/src/engine/mod.rs.md#output-e55203129482)
- [crates/hickory-cli/src/engine/private_policy.rs](implementation/crates/hickory-cli/src/engine/private_policy.rs.md#output-6a8af15fa213)
- [crates/hickory-cli/src/engine/proxy.rs](../../../mechanisms/crates/hickory-cli/src/engine/proxy.rs.md#output-b24aba9d1334)
- [crates/hickory-cli/src/mcp_stdio.rs](implementation/crates/hickory-cli/src/mcp_stdio.rs.md#output-14b6d9cbf226)
- [crates/hickory-cli/src/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/python_agent.rs.md#output-609d98dd1fad)
- [crates/hickory-cli/src/python_agent/host.rs](implementation/crates/hickory-cli/src/python_agent/host.rs.md#output-d88916f0dc97)
- [crates/hickory-cli/src/python_agent/private.rs](implementation/crates/hickory-cli/src/python_agent/private.rs.md#output-40fff9d31e20)
- [crates/hickory-cli/src/serve/acp/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/acp/mod.rs.md#output-fa77fc3f176f)
- [crates/hickory-cli/src/serve/agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/agent.rs.md#output-2ce16640df92)
- [crates/hickory-cli/src/serve/bootstrap.rs](../../../mechanisms/crates/hickory-cli/src/serve/bootstrap.rs.md#output-f374ffa94b18)
- [crates/hickory-cli/src/serve/engineer_tour.rs](../../../mechanisms/crates/hickory-cli/src/serve/engineer_tour.rs.md#output-f4de6cb618b8)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)
- [crates/hickory-cli/src/serve/private_practice.rs](implementation/crates/hickory-cli/src/serve/private_practice.rs.md#output-391b915a64d3)
- [crates/hickory-cli/src/serve/private_prediction_acp.rs](implementation/crates/hickory-cli/src/serve/private_prediction_acp.rs.md#output-80eb5d06ac66)
- [crates/hickory-cli/src/serve/private_prediction_agent.rs](implementation/crates/hickory-cli/src/serve/private_prediction_agent.rs.md#output-f90967235577)
- [crates/hickory-cli/src/serve/private_prediction_debugger.rs](implementation/crates/hickory-cli/src/serve/private_prediction_debugger.rs.md#output-ad517bf17a52)
- [crates/hickory-cli/src/serve/private_prediction_recovery.rs](implementation/crates/hickory-cli/src/serve/private_prediction_recovery.rs.md#output-ed1889813c82)
- [crates/hickory-cli/src/serve/private_prediction_startup.rs](implementation/crates/hickory-cli/src/serve/private_prediction_startup.rs.md#output-31f27f45f135)
- [crates/hickory-cli/src/serve/private_predictions.rs](../../../mechanisms/crates/hickory-cli/src/serve/private_predictions.rs.md#output-5eb3e4172fd6)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../mechanisms/crates/hickory-cli/src/serve/python_agent.rs.md#output-89d47ddc390d)
- [crates/hickory-cli/src/serve/shell.rs](../../../mechanisms/crates/hickory-cli/src/serve/shell.rs.md#output-16984801199b)
- [crates/hickory-cli/src/serve/socket.rs](../../../mechanisms/crates/hickory-cli/src/serve/socket.rs.md#output-ce0f0e119d3c)
- [scripts/test-private-prediction-ui.sh](../../../mechanisms/scripts/test-private-prediction-ui.sh.md#output-d93b734fd0b3)

### Tests and fixtures

- [apps/agent/tests/test_worker.py](../../../mechanisms/apps/agent/tests/test_worker.py.md#output-9f33e52791d8)
- [apps/web/e2e/privateCoachLayout.spec.ts](../../../mechanisms/apps/web/e2e/privateCoachLayout.spec.ts.md#output-3f8363e8ca8f)
- [apps/web/e2e/privatePredictionCoach.spec.ts](../../../mechanisms/apps/web/e2e/privatePredictionCoach.spec.ts.md#output-c4a491bb0b6d)
- [apps/web/e2e/staticUiParity.spec.ts](../../../mechanisms/apps/web/e2e/staticUiParity.spec.ts.md#output-b95b6714a2a7)
- [apps/web/src/components/AgentChanges.test.tsx](../../../mechanisms/apps/web/src/components/AgentChanges.tsx.md#output-7ab5cdb69360)
- [apps/web/src/components/BrowserPredictionPractice.test.tsx](../../../mechanisms/apps/web/src/components/BrowserPredictionPractice.test.tsx.md#output-32e2b9b729a8)
- [apps/web/src/components/FeatureSettings.test.tsx](../../../mechanisms/apps/web/src/components/FeatureSettings.test.tsx.md#output-637fcce1408f)
- [apps/web/src/components/NativeWorkflowTour.test.tsx](../../../mechanisms/apps/web/src/components/NativeWorkflowTour.test.tsx.md#output-8a1a5703c15d)
- [apps/web/src/components/PrivatePredictionCoach.test.tsx](../../../mechanisms/apps/web/src/components/PrivatePredictionCoach.tsx.md#output-4b3612758c98)
- [apps/web/src/lib/nativeWindowActions.test.ts](../../../mechanisms/apps/web/src/lib/nativeWindowActions.ts.md#output-fa3030dd95b0)
- [apps/web/src/lib/uiActionBridge.test.tsx](../../../mechanisms/apps/web/src/lib/uiActionBridge.tsx.md#output-2cc5832c66b7)
- [apps/web/src/tour/browserPrediction.test.ts](../../../mechanisms/apps/web/src/tour/browserPrediction.test.ts.md#output-5eb82555b921)
- [crates/hick-context-store/tests/lineage.rs](../../../mechanisms/crates/hick-context-store/tests/lineage.rs.md#output-335caf8febec)
- [crates/hick-dap/tests/private_adapter.rs](implementation/crates/hick-dap/tests/private_adapter.rs.md#output-b2669b67dd81)
- [crates/hick-literate/tests/prediction_authorization.rs](implementation/crates/hick-literate/tests/prediction_authorization.rs.md#output-91952cf1f2e5)
- [crates/hick-literate/tests/prediction_combined.rs](implementation/crates/hick-literate/tests/prediction_combined.rs.md#output-736f0187c52d)
- [crates/hick-literate/tests/prediction_environment.rs](implementation/crates/hick-literate/tests/prediction_environment.rs.md#output-b078fc2a53bf)
- [crates/hick-literate/tests/prediction_output.rs](implementation/crates/hick-literate/tests/prediction_output.rs.md#output-1730eb6852af)
- [crates/hick-observe/src/prediction_scope_tests.rs](implementation/crates/hick-observe/src/prediction_scope_tests.rs.md#output-a71053c6d40a)
- [crates/hick-observe/src/prediction_tests.rs](../../../mechanisms/crates/hick-observe/src/prediction_tests.rs.md#output-a797b872866a)
- [crates/hickory-agent/tests/prediction_verification.rs](implementation/crates/hickory-agent/tests/prediction_verification.rs.md#output-575d4e4459ff)
- [crates/hickory-cli/src/engine/client_prediction_tests.rs](../../../mechanisms/crates/hickory-cli/src/engine/client_prediction_tests.rs.md#output-014c7e3acc59)
- [crates/hickory-cli/src/serve/private_prediction_acp_tests.rs](implementation/crates/hickory-cli/src/serve/private_prediction_acp_tests.rs.md#output-bbc7e4e93ac2)
- [crates/hickory-cli/src/serve/private_prediction_tests.rs](../../../mechanisms/crates/hickory-cli/src/serve/private_prediction_tests.rs.md#output-7cb981639695)
- [crates/hickory-cli/tests/prediction_engine.rs](implementation/crates/hickory-cli/tests/prediction_engine.rs.md#output-f733448ba92e)
- [crates/hickory-cli/tests/prediction_http.rs](implementation/crates/hickory-cli/tests/prediction_http.rs.md#output-8a0b4d28c345)

### Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
