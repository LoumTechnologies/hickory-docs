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
