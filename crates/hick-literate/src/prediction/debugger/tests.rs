use super::*;
use hick_dap::capture::{CaptureContext, Hit, Recorded};
use hick_observe::prediction::{
    Actor, Classification, Identity, Predicate, Scenario, Seal, digest,
};
use serde_json::json;
use std::collections::BTreeMap;

fn target() -> Target {
    Target::Debugger {
        source: "root".into(),
        source_revision: "a".repeat(64),
        start_byte: 0,
        end_byte: 10,
        expression: "allowance".into(),
        hit: 2,
        frame: Some("retry".into()),
    }
}
fn captured() -> Captured {
    Captured {
        spec: CaptureSpec {
            file: "deadline.py".into(),
            line: 5,
            expressions: vec!["allowance".into()],
            condition: None,
            max: 2,
        },
        hits: (1..=2)
            .map(|index| Hit {
                index,
                context: Some(CaptureContext {
                    thread: 1,
                    frame: index as i64,
                    function: "retry".into(),
                    document_line: Some(7),
                    source: Some("deadline.py".into()),
                    source_line: Some(4),
                    breakpoint_bound: true,
                }),
                values: vec![Recorded {
                    expression: "allowance".into(),
                    value: "20".into(),
                    ok: true,
                }],
            })
            .collect(),
        truncated: true,
        problem: None,
        moved: None,
    }
}

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[test]
fn selected_capture_requires_exact_frame_source_hit_binding_and_successful_evaluation() {
    let good = captured();
    assert_eq!(
        selected(&[good.clone()], &good.spec, &target(), 7).unwrap(),
        json!(20)
    );
    let mut invalid = Vec::new();
    let mut value = good.clone();
    value.hits.pop();
    invalid.push(value);
    let mut value = good.clone();
    value.moved = Some(8);
    invalid.push(value);
    let mut value = good.clone();
    value.problem = Some("unbound".into());
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].context = None;
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].context.as_mut().unwrap().function = "other".into();
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].context.as_mut().unwrap().document_line = Some(8);
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].context.as_mut().unwrap().breakpoint_bound = false;
    invalid.push(value);
    let mut value = good.clone();
    value.hits[0].context.as_mut().unwrap().thread = 2;
    invalid.push(value);
    let mut value = good.clone();
    value.hits[0].context.as_mut().unwrap().source = Some("other.py".into());
    invalid.push(value);
    let mut value = good.clone();
    value.hits[0].context.as_mut().unwrap().source_line = Some(8);
    invalid.push(value);
    let mut value = good.clone();
    value.hits[0].context.as_mut().unwrap().frame = 0;
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].context.as_mut().unwrap().source = Some("other.py".into());
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].context.as_mut().unwrap().source_line = Some(8);
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].context.as_mut().unwrap().frame = 0;
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].values[0].ok = false;
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].values[0].expression = "elapsed".into();
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].values[0].value = "evaluation failed: secret".into();
    invalid.push(value);
    let mut value = good.clone();
    value.hits[1].index = 3;
    invalid.push(value);
    for value in invalid {
        assert!(selected(&[value], &good.spec, &target(), 7).is_err());
    }
}

fn source(corrected: bool) -> String {
    let allowance = if corrected {
        "budget - elapsed"
    } else {
        "budget"
    };
    format!(
        r#"<hick:container name="c" image="python" />
<hick:file path="deadline.py">
def retry(attempts, budget):
    elapsed = 0
    for duration in attempts:
        allowance = {allowance}
        elapsed += min(duration, allowance)
    return elapsed
print(retry([80, 80], 100))
</hick:file>
<hick:exec container="c">python3 deadline.py
<hick:capture at="deadline.py:5" of="allowance" max="2" />
</hick:exec>"#
    )
}
fn request(ledger: &Ledger, source: &str, key: &str) -> DebuggerRequest {
    use sha2::{Digest, Sha256};
    let identity = Identity {
        source: format!("{:x}", Sha256::digest(source.as_bytes())),
        inputs: digest(&json!([80, 80])).unwrap(),
        dependencies: BTreeMap::new(),
        environment: BTreeMap::from([(
            "debugpy-test-host".into(),
            digest(&"configured-adapter").unwrap(),
        )]),
    };
    let start = source.find("        elapsed +=").unwrap();
    let end = start + source[start..].find('\n').unwrap() + 1;
    let target = Target::Debugger {
        source: "root".into(),
        source_revision: identity.source.clone(),
        start_byte: start as u64,
        end_byte: end as u64,
        expression: "allowance".into(),
        hit: 2,
        frame: Some("retry".into()),
    };
    let seal = Seal {
        identity: identity.clone(),
        target: target.clone(),
        predicted: Predicate::Equals(json!(20)),
        rationale: "Only twenty milliseconds remain before the second attempt".into(),
        scenario: Scenario {
            family: "retry-budget/second-allowance".into(),
            inputs: json!([80, 80]),
            requirement_reference: "one total 100ms deadline; 80ms elapsed".into(),
            requirement: Predicate::Equals(json!(20)),
            identity_known: true,
        },
    };
    let receipt = ledger
        .seal(
            ledger.history(None).unwrap().generation,
            seal,
            Actor {
                kind: "agent".into(),
                id: "predictor".into(),
            },
        )
        .unwrap();
    DebuggerRequest {
        seal: receipt.id,
        request_id: key.into(),
        actor: Actor {
            kind: "agent".into(),
            id: "predictor".into(),
        },
        identity,
        target,
    }
}

// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a configured Python debug adapter; run this focused witness explicitly"]
async fn real_python_second_hit_fault_and_repair_use_the_existing_private_capture_evaluator() {
    let project = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(
        hick_dap::discover("python", &project).is_some(),
        "This focused witness requires a configured Python debug adapter"
    );
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let faulty = source(false);
    let first = request(&ledger, &faulty, "fault");
    let failed = run(&ledger, &first, &faulty, &project).await.unwrap();
    assert_eq!(failed.summary.classification, Classification::Mismatch);
    assert_eq!(failed.summary.prediction_matched, Some(false));
    assert!(!serde_json::to_string(&failed).unwrap().contains("captures"));
    assert!(
        ledger
            .promote(
                &failed.summary.attempt,
                &first.identity,
                first.actor.clone()
            )
            .is_err()
    );
    assert!(
        !run(&ledger, &first, &faulty, &project)
            .await
            .unwrap()
            .dispatched
    );
    let corrected = source(true);
    let second = request(&ledger, &corrected, "repair");
    let passed = run(&ledger, &second, &corrected, &project).await.unwrap();
    assert_eq!(
        passed.summary.classification,
        Classification::VerifiedAfterCorrection
    );
    assert_eq!(passed.summary.prediction_matched, Some(true));
    assert!(
        ledger
            .promote(
                &passed.summary.attempt,
                &second.identity,
                second.actor.clone()
            )
            .unwrap()
            .current_certification
    );
    assert_eq!(ledger.history(None).unwrap().attempts.len(), 2);
}
