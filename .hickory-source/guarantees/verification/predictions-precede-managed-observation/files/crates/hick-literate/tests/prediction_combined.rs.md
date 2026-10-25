<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/tests/prediction_combined.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../contract.md)

Captured SHA-256: `284239bca84942ed7c0e73aa2c23543fc37fd8199dfdd2393ded0e619be6f6df`.

<book:entry path="crates/hick-literate/tests/prediction_combined.rs">
#![cfg(unix)]
//! Same-revision ordinary exec and private DAP capture under one ledger gate.
#[path = "support/prediction_privacy.rs"]
mod privacy;
use hick_literate::prediction::{
    OutputEncoding,
    combined::{self, Request},
    output_target,
};
use hick_observe::prediction::*;
use hickory_executor::{Executor, LocalExecutor};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

fn actor() -> Actor {
    Actor {
        kind: "agent".into(),
        id: "predictor".into(),
    }
}
fn source(corrected: bool, cap_output: bool) -> String {
    let allowance = if corrected {
        "budget - elapsed"
    } else {
        "budget"
    };
    let final_value = if cap_output {
        "min(100, retry([80, 80], 100))"
    } else {
        "retry([80, 80], 100)"
    };
    format!(
        r#"<hick:container name="c" image="python" />
<hick:volume name="code" input="." />
<hick:file path="deadline.py">
def retry(attempts, budget):
    elapsed = 0
    for duration in attempts:
        allowance = {allowance}
        elapsed += min(duration, allowance)
    return elapsed
print({final_value})
</hick:file>
<hick:exec container="c" mount="code:/work">
python3 work/deadline.py
<hick:capture at="deadline.py:5" of="allowance" max="2" />
<hick:expect>100
</hick:expect>
</hick:exec>"#
    )
}
fn request(ledger: &Ledger, source: &str, key: &str) -> Request {
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&json!([80, 80])).unwrap(),
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
    let start = source.find("        elapsed +=").unwrap();
    let end = start + source[start..].find('\n').unwrap() + 1;
    let target = Target::Combined {
        output: Box::new(output_target("c", line, OutputEncoding::Json)),
        debugger: Box::new(Target::Debugger {
            source: "root".into(),
            source_revision: identity.source.clone(),
            start_byte: start as u64,
            end_byte: end as u64,
            expression: "allowance".into(),
            hit: 2,
            frame: Some("retry".into()),
        }),
    };
    let seal = Seal {
        identity: identity.clone(),
        target: target.clone(),
        rationale: "One total budget leaves twenty before the second attempt".into(),
        predicted: Predicate::Fields(BTreeMap::from([
            ("output".into(), Predicate::Equals(json!(100))),
            ("debugger".into(), Predicate::Equals(json!(20))),
        ])),
        scenario: Scenario {
            family: "retry-budget/combined".into(),
            inputs: json!([80, 80]),
            requirement_reference: "one total 100ms allowance".into(),
            identity_known: true,
            requirement: Predicate::Fields(BTreeMap::from([
                ("output".into(), Predicate::NumberAtMost(100.0)),
                ("debugger".into(), Predicate::Equals(json!(20))),
            ])),
        },
    };
    let receipt = ledger
        .seal(ledger.history(None).unwrap().generation, seal, actor())
        .unwrap();
    Request {
        seal: receipt.id,
        request_id: key.into(),
        actor: actor(),
        identity,
        target,
    }
}

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
</book:entry>

</book:doc>
