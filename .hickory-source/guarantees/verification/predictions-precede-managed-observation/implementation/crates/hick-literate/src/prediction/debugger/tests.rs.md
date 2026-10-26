<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/prediction/debugger/tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-486f17fcecc4"></a>
## `crates/hick-literate/src/prediction/debugger/tests.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../../contract.md).

<a id="f-486f17fcecc4-target"></a>
### `target`

<storycode:copy id="f-486f17fcecc4-target">
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
</storycode:copy>
<a id="f-486f17fcecc4-captured"></a>
### `captured`

<storycode:copy id="f-486f17fcecc4-captured">
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
                    run: "fixture-capture-run".into(),
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

</storycode:copy>
<a id="f-486f17fcecc4-selected_capture_requires_exact_frame_source_hit_binding_and_successful_evaluation"></a>
### `selected_capture_requires_exact_frame_source_hit_binding_and_successful_evaluation`

<storycode:copy id="f-486f17fcecc4-selected_capture_requires_exact_frame_source_hit_binding_and_successful_evaluation">
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
    value.hits[0].context.as_mut().unwrap().run = "different-adapter-run".into();
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

</storycode:copy>
<a id="f-486f17fcecc4-source"></a>
### `source`

<storycode:copy id="f-486f17fcecc4-source">
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
</storycode:copy>
<a id="f-486f17fcecc4-request"></a>
### `request`

<storycode:copy id="f-486f17fcecc4-request">
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

</storycode:copy>
<a id="f-486f17fcecc4-real_python_second_hit_fault_and_repair_use_the_existing_private_capture_evaluator"></a>
### `real_python_second_hit_fault_and_repair_use_the_existing_private_capture_evaluator`

<storycode:copy id="f-486f17fcecc4-real_python_second_hit_fault_and_repair_use_the_existing_private_capture_evaluator">
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

</storycode:copy>
<a id="f-486f17fcecc4-retained_session_requires_future_seal_before_resume_and_keeps_answers_private"></a>
### `retained_session_requires_future_seal_before_resume_and_keeps_answers_private`

<storycode:copy id="f-486f17fcecc4-retained_session_requires_future_seal_before_resume_and_keeps_answers_private">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires configured Python debugpy; explicitly run the focused witness"]
async fn retained_session_requires_future_seal_before_resume_and_keeps_answers_private() {
    use super::interactive::RetainedDebugger;
    let project = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(hick_dap::discover("python", &project).is_some());
    let profile = crate::prediction::environment::Profile::PythonDebugpy {
        project: project.clone(),
    };
    let environment = profile.capture().await.unwrap();
    let root = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(root.path()).unwrap();
    for corrected in [false, true] {
        let text = source(corrected);
        let template = request(
            &ledger,
            &text,
            if corrected {
                "repair-template"
            } else {
                "fault-template"
            },
        );
        let history = ledger.history(None).unwrap();
        let mut future: Seal = serde_json::from_value(
            history
                .ordering
                .iter()
                .find(|entry| entry["event"]["id"] == template.seal)
                .unwrap()["event"]["seal"]
                .clone(),
        )
        .unwrap();
        future.identity.environment = environment.clone();
        let mut initial = future.clone();
        initial.scenario.family = "retry-budget/initial-allowance".into();
        initial.scenario.requirement_reference =
            "initial allowance is the full 100ms budget".into();
        initial.predicted = Predicate::Equals(json!(100));
        initial.scenario.requirement = initial.predicted.clone();
        if let Target::Debugger { hit, .. } = &mut initial.target {
            *hit = 1;
        }
        let receipt = ledger
            .seal(history.generation, initial.clone(), template.actor.clone())
            .unwrap();
        let first = DebuggerRequest {
            seal: receipt.id,
            request_id: format!("initial-{corrected}"),
            actor: template.actor.clone(),
            identity: initial.identity,
            target: initial.target,
        };
        // The application can retain the paused owner after its temporary
        // ledger handle goes away; the journal still has one lock/authority.
        let temporary_handle = ledger.clone();
        let (session, result) = RetainedDebugger::start_with_environment(
            &temporary_handle,
            &first,
            &text,
            &project,
            Some(&profile),
        )
        .await
        .unwrap();
        drop(temporary_handle);
        assert!(result.summary.prediction_matched.unwrap());
        let mut session = session.unwrap();
        let visible = json!({"source_review":"two attempts with a single total deadline", "prior_verdict":"matched"});
        // An unbound initial seal cannot resume this retained process.
        assert!(session.resume(&template, &visible).await.is_err());
        let next = session
            .seal_next(future, format!("future-{corrected}"), visible.clone())
            .unwrap();
        // Neither an initial adapter nor changed context can consume the seal.
        assert!(run(&ledger, &next, &text, &project).await.is_err());
        assert!(
            session
                .resume(&next, &json!({"changed":true}))
                .await
                .is_err()
        );
        let result = session.resume(&next, &visible).await.unwrap();
        let retry = session.resume(&next, &visible).await.unwrap();
        assert!(!retry.dispatched);
        assert_eq!(retry.summary.attempt, result.summary.attempt);
        assert!(
            session
                .resume(&next, &json!({"changed":true}))
                .await
                .is_err()
        );

        assert_eq!(
            result.summary.classification,
            if corrected {
                Classification::VerifiedAfterCorrection
            } else {
                Classification::Mismatch
            }
        );
        let feedback = serde_json::to_string(&result).unwrap();
        assert!(!feedback.contains("values") && !feedback.contains("captures"));
        if corrected {
            assert!(
                ledger
                    .promote(&result.summary.attempt, &next.identity, next.actor)
                    .unwrap()
                    .current_certification
            );
        } else {
            assert!(
                ledger
                    .promote(&result.summary.attempt, &next.identity, next.actor)
                    .is_err()
            );
        }
        session.shutdown().await;
    }
    let public = serde_json::to_string(&ledger.history(None).unwrap()).unwrap();
    assert!(
        public.contains("future_stop")
            && !public.contains("two attempts with a single total deadline")
    );
    // Fresh correct future-stop success must never claim an independent first prediction.
    let fresh_root = tempfile::tempdir().unwrap();
    let fresh = Ledger::open(fresh_root.path()).unwrap();
    let text = source(true);
    let template = request(&fresh, &text, "informed-template");
    let history = fresh.history(None).unwrap();
    let seal: Seal = serde_json::from_value(history.ordering[0]["event"]["seal"].clone()).unwrap();
    let prior = json!({"previous_stop":"reviewed source"});
    let stop = hick_observe::prediction::FutureStop {
        run: "host-run".into(),
        thread: 1,
        frame: 1,
        hit: 1,
        prior_visible_digest: digest(&prior).unwrap(),
    };
    let receipt = fresh
        .seal_future(
            history.generation,
            seal,
            template.actor.clone(),
            stop.clone(),
            prior,
        )
        .unwrap();
    assert!(
        fresh
            .admit_target(
                &receipt.id,
                "wrong",
                &template.identity,
                false,
                template.actor.clone(),
                &template.target
            )
            .is_err()
    );
    let admission = fresh
        .admit_future_target(
            &receipt.id,
            "informed",
            &template.identity,
            template.actor.clone(),
            &template.target,
            &stop,
        )
        .unwrap();
    let observation = hick_observe::prediction::ObservedResult {
        identity: template.identity,
        target: template.target,
        actual: json!(20),
        evidence: json!({}),
    };
    assert!(
        fresh
            .finish(&admission.receipt.id, observation.clone())
            .is_err()
    );
    let result = fresh
        .finish_future_checked(&admission.receipt.id, observation, true, &stop)
        .unwrap();
    assert_eq!(
        result.classification,
        Classification::InformedPredictionVerified
    );
}

</storycode:copy>
<a id="f-486f17fcecc4-retained_session_missing_future_hit_is_unknown_and_never_promotable"></a>
### `retained_session_missing_future_hit_is_unknown_and_never_promotable`

<storycode:copy id="f-486f17fcecc4-retained_session_missing_future_hit_is_unknown_and_never_promotable">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires configured Python debugpy; explicitly run the focused witness"]
async fn retained_session_missing_future_hit_is_unknown_and_never_promotable() {
    use super::interactive::RetainedDebugger;
    let project = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(hick_dap::discover("python", &project).is_some());
    let root = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(root.path()).unwrap();
    let text = source(true).replace("    return elapsed", "        return elapsed");
    let template = request(&ledger, &text, "early-return-template");
    let history = ledger.history(None).unwrap();
    let future: Seal =
        serde_json::from_value(history.ordering[0]["event"]["seal"].clone()).unwrap();
    let mut initial = future.clone();
    initial.scenario.family = "retry-budget/initial-allowance".into();
    initial.scenario.requirement_reference = "initial allowance is 100ms".into();
    initial.predicted = Predicate::Equals(json!(100));
    initial.scenario.requirement = initial.predicted.clone();
    if let Target::Debugger { hit, .. } = &mut initial.target {
        *hit = 1;
    }
    let receipt = ledger
        .seal(history.generation, initial.clone(), template.actor.clone())
        .unwrap();
    let first = DebuggerRequest {
        seal: receipt.id,
        request_id: "first".into(),
        actor: template.actor.clone(),
        identity: initial.identity,
        target: initial.target,
    };
    let (session, result) = RetainedDebugger::start(&ledger, &first, &text, &project)
        .await
        .unwrap();
    assert_eq!(result.summary.prediction_matched, Some(true));
    let mut session = session.unwrap();
    let visible = json!({"previous_stop":"source reviewed"});
    let next = session
        .seal_next(future, "missing-second-hit".into(), visible.clone())
        .unwrap();
    let result = session.resume(&next, &visible).await.unwrap();
    assert_eq!(result.summary.classification, Classification::Unknown);
    assert_eq!(result.summary.prediction_matched, None);
    assert!(
        ledger
            .promote(&result.summary.attempt, &next.identity, next.actor)
            .is_err()
    );
    assert!(!serde_json::to_string(&result).unwrap().contains("captures"));
    session.shutdown().await;
}

</storycode:copy>
<a id="f-486f17fcecc4-retained_runtime_change_refuses_continue_without_dispatch"></a>
### `retained_runtime_change_refuses_continue_without_dispatch`

<storycode:copy id="f-486f17fcecc4-retained_runtime_change_refuses_continue_without_dispatch">
// Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
#[cfg(unix)]
#[tokio::test]
#[ignore = "requires configured Python debugpy; explicitly run the focused witness"]
async fn retained_runtime_change_refuses_continue_without_dispatch() {
    use super::interactive::RetainedDebugger;
    let project = tempfile::tempdir().unwrap();
    let parent = project.path().join(".hick-cache/adapters");
    std::fs::create_dir_all(&parent).unwrap();
    let adapter = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.hick-cache/adapters/python")
        .canonicalize()
        .unwrap();
    let link = parent.join("python");
    std::os::unix::fs::symlink(adapter, &link).unwrap();
    let profile = crate::prediction::environment::Profile::PythonDebugpy {
        project: project.path().into(),
    };
    let private = tempfile::tempdir().unwrap();
    let ledger = Ledger::open(private.path()).unwrap();
    let text = source(true);
    let template = request(&ledger, &text, "template");
    let mut future: Seal = serde_json::from_value(
        ledger
            .history(None)
            .unwrap()
            .ordering
            .iter()
            .find(|entry| entry["event"]["id"] == template.seal)
            .unwrap()["event"]["seal"]
            .clone(),
    )
    .unwrap();
    future.identity.environment = profile.capture().await.unwrap();
    let mut initial = future.clone();
    initial.scenario.family = "runtime-retained-initial".into();
    initial.predicted = Predicate::Equals(json!(100));
    initial.scenario.requirement = initial.predicted.clone();
    if let Target::Debugger { hit, .. } = &mut initial.target {
        *hit = 1;
    }
    let receipt = ledger
        .seal(
            ledger.history(None).unwrap().generation,
            initial.clone(),
            template.actor.clone(),
        )
        .unwrap();
    let first = DebuggerRequest {
        seal: receipt.id,
        request_id: "initial".into(),
        actor: template.actor,
        identity: initial.identity,
        target: initial.target,
    };
    let (session, result) = RetainedDebugger::start_with_environment(
        &ledger,
        &first,
        &text,
        project.path(),
        Some(&profile),
    )
    .await
    .unwrap();
    assert!(result.summary.prediction_matched.unwrap());
    let mut session = session.unwrap();
    let visible = json!({"prior_verdict":"matched"});
    let next = session
        .seal_next(future, "future".into(), visible.clone())
        .unwrap();
    let attempts = ledger.history(None).unwrap().attempts.len();
    std::fs::remove_file(link).unwrap();
    assert!(session.resume(&next, &visible).await.is_err());
    assert_eq!(
        ledger.history(None).unwrap().attempts.len(),
        attempts,
        "Runtime refusal must precede future dispatch"
    );
    session.shutdown().await;
}
</storycode:copy>

### Assemble `crates/hick-literate/src/prediction/debugger/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/prediction/debugger/tests.rs"><storycode:paste select="#f-486f17fcecc4-target"/><storycode:paste select="#f-486f17fcecc4-captured"/><storycode:paste select="#f-486f17fcecc4-selected_capture_requires_exact_frame_source_hit_binding_and_successful_evaluation"/><storycode:paste select="#f-486f17fcecc4-source"/><storycode:paste select="#f-486f17fcecc4-request"/><storycode:paste select="#f-486f17fcecc4-real_python_second_hit_fault_and_repair_use_the_existing_private_capture_evaluator"/><storycode:paste select="#f-486f17fcecc4-retained_session_requires_future_seal_before_resume_and_keeps_answers_private"/><storycode:paste select="#f-486f17fcecc4-retained_session_missing_future_hit_is_unknown_and_never_promotable"/><storycode:paste select="#f-486f17fcecc4-retained_runtime_change_refuses_continue_without_dispatch"/></storycode:file>

</storycode:doc>
