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
use serde_json::json;

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

fn actor(kind: &str) -> Actor {
    Actor {
        kind: kind.into(),
        id: format!("fixture-{kind}"),
    }
}
fn seal(predicted: u64, source: char) -> Seal {
    let inputs = json!({"attempts":[80,80],"budget":100});
    Seal {
        scenario: Scenario {
            family: "retry-total-budget-v1".into(),
            inputs: inputs.clone(),
            requirement_reference: "R1".into(),
            requirement: Predicate::NumberAtMost(100.0),
            identity_known: true,
        },
        identity: Identity {
            source: source.to_string().repeat(64),
            inputs: digest(&inputs).unwrap(),
            dependencies: BTreeMap::new(),
            environment: BTreeMap::from([("node".into(), "e".repeat(64))]),
        },
        target: Target::Output {
            execution: "deadline-witness".into(),
        },
        predicted: Predicate::Equals(json!(predicted)),
        rationale: "Two attempts share one total budget".into(),
    }
}
fn finish(ledger: &Ledger, attempt: &str, actual: Value, evidence: Value) -> Result<Summary> {
    let observation = ledger.transaction(|journal| {
        let (_, seal, _) = find_attempt(journal, attempt)?;
        Ok(ObservedResult {
            identity: seal.identity.clone(),
            target: seal.target.clone(),
            actual,
            evidence,
        })
    })?;
    ledger.finish(attempt, observation)
}

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
#[test]
fn missing_stale_and_concurrent_seals_refuse_execution_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let prediction = seal(100, 'a');
    assert!(
        ledger
            .begin(
                "missing",
                "one",
                &prediction.identity,
                false,
                actor("agent")
            )
            .is_err()
    );
    let sealed = ledger.seal(0, prediction.clone(), actor("agent"))?;
    assert!(ledger.seal(0, prediction.clone(), actor("agent")).is_err());
    assert!(
        ledger
            .begin(
                &sealed.id,
                "one",
                &seal(100, 'b').identity,
                false,
                actor("agent")
            )
            .is_err()
    );
    let first = ledger.begin(
        &sealed.id,
        "one",
        &prediction.identity,
        false,
        actor("agent"),
    )?;
    assert_eq!(
        ledger
            .begin(
                &sealed.id,
                "one",
                &prediction.identity,
                false,
                actor("agent")
            )?
            .id,
        first.id
    );
    assert!(
        ledger
            .begin(
                &sealed.id,
                "one",
                &prediction.identity,
                false,
                actor("human")
            )
            .is_err()
    );
    assert!(
        ledger
            .begin(
                &sealed.id,
                "two",
                &prediction.identity,
                false,
                actor("agent")
            )
            .is_err()
    );
    Ok(())
}
#[test]
fn mismatches_withhold_actual_values_and_reveal_records_actor_before_return() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let prediction = seal(100, 'a');
    let sealed = ledger.seal(0, prediction.clone(), actor("agent"))?;
    let attempt = ledger.begin(
        &sealed.id,
        "run",
        &prediction.identity,
        false,
        actor("agent"),
    )?;
    let result = finish(
        &ledger,
        &attempt.id,
        json!(160),
        json!({"stdout":"private actual 160"}),
    )?;
    assert_eq!(result.classification, Classification::Mismatch);
    let public = serde_json::to_string(&ledger.history(None)?)?;
    assert!(!public.contains("private actual 160"));
    assert!(!public.contains(":160"));
    assert!(
        ledger
            .promote(&attempt.id, &prediction.identity, actor("agent"))
            .is_err()
    );
    let revealed = ledger.reveal(&attempt.id, actor("human"), "native-debugger")?;
    assert_eq!(revealed["actual"], 160);
    let reopened = Ledger::open(root.path())?.history(None)?;
    assert_eq!(
        reopened.ordering.last().unwrap()["event"]["kind"],
        "observed"
    );
    assert_eq!(
        reopened.ordering.last().unwrap()["event"]["actor"]["kind"],
        "human"
    );
    let generation = ledger.history(None)?.generation;
    assert!(
        ledger
            .reveal(&attempt.id, actor("agent"), "context-delivery")
            .is_err()
    );
    assert!(
        ledger
            .reveal(&attempt.id, actor("host"), "tool-response")
            .is_err()
    );
    assert_eq!(ledger.history(None)?.generation, generation);
    // Record a pre-existing accidental exposure; recording it does not deliver data.
    ledger.observe(
        &prediction.scenario,
        prediction.identity.clone(),
        actor("agent"),
        "prior-context",
    )?;
    assert_eq!(
        ledger.history(None)?.ordering.last().unwrap()["event"]["actor"]["kind"],
        "agent"
    );
    assert!(finish(&ledger, &attempt.id, json!(100), json!({})).is_err());
    Ok(())
}
#[test]
fn correctly_predicting_the_bug_does_not_satisfy_the_requirement() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let prediction = seal(160, 'a');
    let sealed = ledger.seal(0, prediction.clone(), actor("agent"))?;
    let attempt = ledger.begin(
        &sealed.id,
        "run",
        &prediction.identity,
        false,
        actor("agent"),
    )?;
    let result = finish(&ledger, &attempt.id, json!(160), json!({}))?;
    assert_eq!(result.prediction_matched, Some(true));
    assert_eq!(result.original_requirement_satisfied, Some(false));
    assert_eq!(result.classification, Classification::RequirementFailed);
    assert!(
        ledger
            .promote(&attempt.id, &prediction.identity, actor("agent"))
            .is_err()
    );
    Ok(())
}
#[test]
fn code_correction_preserves_failed_history_and_requires_current_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let original = seal(100, 'a');
    let sealed = ledger.seal(0, original.clone(), actor("agent"))?;
    let failure = ledger.begin(&sealed.id, "bad", &original.identity, false, actor("agent"))?;
    finish(&ledger, &failure.id, json!(160), json!({}))?;
    ledger.observe(
        &original.scenario,
        original.identity.clone(),
        actor("agent"),
        "prior-debug-state",
    )?;
    let repaired = seal(100, 'b');
    let sealed = ledger.seal(
        ledger.history(None)?.generation,
        repaired.clone(),
        actor("agent"),
    )?;
    let attempt = ledger.begin(
        &sealed.id,
        "good",
        &repaired.identity,
        false,
        actor("agent"),
    )?;
    let result = finish(&ledger, &attempt.id, json!(100), json!({}))?;
    assert_eq!(
        result.classification,
        Classification::VerifiedAfterCorrection
    );
    assert!(
        ledger
            .promote(&attempt.id, &repaired.identity, actor("agent"))?
            .promoted
    );
    assert_eq!(
        Ledger::open(root.path())?.history(None)?.attempts[0].classification,
        Classification::Mismatch
    );
    assert!(
        ledger
            .promote(&attempt.id, &seal(100, 'c').identity, actor("agent"))
            .is_err()
    );
    Ok(())
}
#[test]
fn observed_expectation_adjustments_cannot_bless_the_bug_or_reset_via_source_identity() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let original = seal(100, 'a');
    let sealed = ledger.seal(0, original.clone(), actor("agent"))?;
    let failure = ledger.begin(
        &sealed.id,
        "first",
        &original.identity,
        false,
        actor("agent"),
    )?;
    finish(&ledger, &failure.id, json!(160), json!({}))?;
    ledger.observe(
        &original.scenario,
        original.identity.clone(),
        actor("agent"),
        "prior-cached-transcript",
    )?;
    let mut adjusted = seal(160, 'b');
    adjusted.scenario.requirement = Predicate::NumberAtMost(160.0);
    assert_eq!(adjusted.scenario.key()?, original.scenario.key()?);
    let sealed = ledger.seal(
        ledger.history(None)?.generation,
        adjusted.clone(),
        actor("agent"),
    )?;
    let next = ledger.begin(
        &sealed.id,
        "adjusted",
        &adjusted.identity,
        false,
        actor("agent"),
    )?;
    let summary = finish(&ledger, &next.id, json!(160), json!({}))?;
    assert_eq!(
        summary.classification,
        Classification::AdjustedAfterObservation
    );
    assert_eq!(summary.original_requirement_satisfied, Some(false));
    assert!(
        ledger
            .promote(&next.id, &adjusted.identity, actor("agent"))
            .is_err()
    );
    let mut novel = seal(50, 'b');
    novel.scenario.inputs = json!({"attempts":[20,30],"budget":100});
    novel.identity.inputs = digest(&novel.scenario.inputs)?;
    let sealed = ledger.seal(
        ledger.history(None)?.generation,
        novel.clone(),
        actor("agent"),
    )?;
    let next = ledger.begin(&sealed.id, "novel", &novel.identity, false, actor("agent"))?;
    assert_eq!(
        finish(&ledger, &next.id, json!(50), json!({}))?.classification,
        Classification::FirstAttemptVerified
    );
    Ok(())
}
#[test]
fn unknown_prior_exposure_and_replay_cannot_certify_pristine_success() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let prediction = seal(100, 'a');
    ledger.observe(
        &prediction.scenario,
        prediction.identity.clone(),
        actor("human"),
        "earlier-debugger",
    )?;
    let sealed = ledger.seal(
        ledger.history(None)?.generation,
        prediction.clone(),
        actor("agent"),
    )?;
    let attempt = ledger.begin(
        &sealed.id,
        "run",
        &prediction.identity,
        false,
        actor("agent"),
    )?;
    assert_eq!(
        finish(&ledger, &attempt.id, json!(100), json!({}))?.classification,
        Classification::Unknown
    );
    assert!(
        ledger
            .promote(&attempt.id, &prediction.identity, actor("agent"))
            .is_err()
    );
    let other = tempfile::tempdir()?;
    let replay_ledger = Ledger::open(other.path())?;
    let sealed = replay_ledger.seal(0, prediction.clone(), actor("agent"))?;
    let replay = replay_ledger.begin(
        &sealed.id,
        "replay",
        &prediction.identity,
        true,
        actor("agent"),
    )?;
    assert_eq!(
        finish(&replay_ledger, &replay.id, json!(100), json!({}))?.classification,
        Classification::Replayed
    );
    assert!(
        replay_ledger
            .promote(&replay.id, &prediction.identity, actor("agent"))
            .is_err()
    );
    Ok(())
}
#[test]
fn corrupt_history_is_refused_instead_of_becoming_an_empty_ledger() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    ledger.seal(0, seal(100, 'a'), actor("agent"))?;
    fs::write(
        root.path().join("prediction.json"),
        br#"{"version":1,"entries":["corrupt"]}"#,
    )?;
    assert!(ledger.history(None).is_err());
    assert!(ledger.seal(0, seal(100, 'b'), actor("agent")).is_err());
    Ok(())
}

#[test]
fn debugger_predictions_bind_expression_exact_source_span_and_hit_and_retain_context() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let mut prediction = seal(20, 'a');
    prediction.scenario.family = "retry-budget.remaining-before-attempt".into();
    prediction.target = Target::Debugger {
        frame: None,
        source: "root".into(),
        source_revision: prediction.identity.source.clone(),
        start_byte: 20,
        end_byte: 25,
        expression: "budget - total".into(),
        hit: 2,
    };
    let sealed = ledger.seal(0, prediction.clone(), actor("agent"))?;
    let attempt = ledger.begin(
        &sealed.id,
        "debug-run",
        &prediction.identity,
        false,
        actor("agent"),
    )?;
    let mut observed = ObservedResult {
        identity: prediction.identity.clone(),
        target: prediction.target.clone(),
        actual: json!(20),
        evidence: json!({"adapter":"existing-hick-capture", "stop":"stop-2", "frame":"frame-1"}),
    };
    if let Target::Debugger { hit, .. } = &mut observed.target {
        *hit = 1;
    }
    assert!(ledger.finish(&attempt.id, observed.clone()).is_err());
    observed.target = prediction.target.clone();
    observed.identity.source = "b".repeat(64);
    assert!(ledger.finish(&attempt.id, observed.clone()).is_err());
    observed.identity = prediction.identity.clone();
    assert_eq!(
        ledger.finish(&attempt.id, observed)?.classification,
        Classification::FirstAttemptVerified
    );
    ledger.reveal(&attempt.id, actor("human"), "debugger-variables")?;
    ledger.observe(
        &prediction.scenario,
        prediction.identity.clone(),
        actor("agent"),
        "prior-debug-context",
    )?;
    ledger.seal(ledger.history(None)?.generation, prediction, actor("agent"))?;
    let history = ledger.history(None)?;
    let context = &history.ordering.last().unwrap()["event"]["context"];
    assert_eq!(context[0]["delivered_to_predictor"], false);
    assert_eq!(context[1]["delivered_to_predictor"], true);
    assert_eq!(context[0]["surface"], "debugger-variables");
    Ok(())
}

#[test]
fn interrupted_attempts_survive_restart_and_are_never_presented_as_success() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let prediction = seal(100, 'a');
    let sealed = ledger.seal(0, prediction.clone(), actor("agent"))?;
    let pending = ledger.begin(
        &sealed.id,
        "before-crash",
        &prediction.identity,
        false,
        actor("agent"),
    )?;
    let reopened = Ledger::open(root.path())?;
    assert!(
        reopened
            .begin(
                &sealed.id,
                "after-crash",
                &prediction.identity,
                false,
                actor("agent")
            )
            .is_err()
    );
    assert!(
        reopened
            .interrupt(&pending.id, actor("agent"), Interruption::HostInterrupted)
            .is_err()
    );
    assert_eq!(
        reopened
            .interrupt(&pending.id, actor("host"), Interruption::HostInterrupted)?
            .classification,
        Classification::Unknown
    );
    assert!(finish(&reopened, &pending.id, json!(100), json!({})).is_err());
    let next = reopened.begin(
        &sealed.id,
        "after-crash",
        &prediction.identity,
        false,
        actor("agent"),
    )?;
    assert_eq!(
        finish(&reopened, &next.id, json!(100), json!({}))?.classification,
        Classification::VerifiedAfterPriorAttempt
    );
    assert_eq!(
        reopened.history(None)?.attempts[0].classification,
        Classification::Unknown
    );
    Ok(())
}

#[test]
fn independent_concurrent_writers_cannot_seal_against_the_same_generation() -> Result<()> {
    let root = tempfile::tempdir()?;
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let path = root.path().to_owned();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let ledger = Ledger::open(path).unwrap();
                barrier.wait();
                ledger.seal(0, seal(100, 'a'), actor("agent")).is_ok()
            })
        })
        .collect();
    assert_eq!(
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .filter(|success| *success)
            .count(),
        1
    );
    assert_eq!(Ledger::open(root.path())?.history(None)?.generation, 1);
    Ok(())
}

#[test]
fn guessing_a_replacement_expectation_cannot_earn_verification_credit() -> Result<()> {
    let root = tempfile::tempdir()?;
    let ledger = Ledger::open(root.path())?;
    let original = seal(100, 'a');
    let sealed = ledger.seal(0, original.clone(), actor("agent"))?;
    let first = ledger.begin(
        &sealed.id,
        "first",
        &original.identity,
        false,
        actor("agent"),
    )?;
    finish(
        &ledger,
        &first.id,
        json!(90),
        json!({"stderr":"secret answer 90"}),
    )?;
    assert!(ledger.reveal(&first.id, actor("agent"), "debug").is_err());
    // Even a lucky guess at the secret result, within the original requirement,
    // is not evidence that the original prediction or implementation was repaired.
    let replacement = seal(90, 'b');
    let sealed = ledger.seal(
        ledger.history(None)?.generation,
        replacement.clone(),
        actor("agent"),
    )?;
    let next = ledger.begin(
        &sealed.id,
        "replacement",
        &replacement.identity,
        false,
        actor("agent"),
    )?;
    let result = finish(&ledger, &next.id, json!(90), json!({}))?;
    assert_eq!(result.prediction_matched, Some(true));
    assert_eq!(result.original_requirement_satisfied, Some(true));
    assert_eq!(result.classification, Classification::PredictionChanged);
    assert!(
        ledger
            .promote(&next.id, &replacement.identity, actor("agent"))
            .is_err()
    );
    Ok(())
}

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

#[path = "prediction_scope_tests.rs"]
mod scoped;
