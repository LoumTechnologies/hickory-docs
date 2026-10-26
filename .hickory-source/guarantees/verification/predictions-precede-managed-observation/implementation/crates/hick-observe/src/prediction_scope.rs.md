<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-observe/src/prediction_scope.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a3362438d4c1"></a>
## `crates/hick-observe/src/prediction_scope.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-a3362438d4c1-impl-Ledger"></a>
### `impl-Ledger`

Predictor-facing projection. Foreign predictions may themselves copy human-only
answers; their predicates/rationales are not a predictor reading capability.

<storycode:copy id="f-a3362438d4c1-impl-Ledger">
//! Predictor-facing projection. Foreign predictions may themselves copy human-only
//! answers; their predicates/rationales are not a predictor reading capability.
use super::*;
use std::collections::BTreeSet;
impl Ledger {
    /// Keep this predictor's selected question histories and relevant exposure
    /// metadata. Global ordering/commitments and classification remain unchanged.
    pub fn predictor_history(
        &self,
        current: Option<&Identity>,
        actor: &Actor,
        scenarios: &[String],
    ) -> Result<History> {
        project(self.history(current)?, actor, scenarios)
    }
}
</storycode:copy>
<a id="f-a3362438d4c1-project"></a>
### `project`

<storycode:copy id="f-a3362438d4c1-project">
pub(super) fn project(
    mut history: History,
    actor: &Actor,
    scenarios: &[String],
) -> Result<History> {
    actor.validate()?;
    let selected: BTreeSet<_> = scenarios.iter().collect();
    ensure!(
        !selected.is_empty() && selected.len() <= 64,
        "Select bounded predictor scenarios"
    );
    let actor = serde_json::to_value(actor)?;
    let mut seals = BTreeSet::new();
    for entry in &history.ordering {
        let event = &entry["event"];
        if event["kind"] == "sealed" && event["actor"] == actor {
            let seal: Seal = serde_json::from_value(event["seal"].clone())?;
            if selected.contains(&seal.scenario.key()?) {
                seals.insert(
                    event["id"]
                        .as_str()
                        .context("Seal identity missing")?
                        .to_owned(),
                );
            }
        }
    }
    let attempts: BTreeSet<_> = history
        .ordering
        .iter()
        .filter_map(|entry| {
            let event = &entry["event"];
            (event["kind"] == "started"
                && event["actor"] == actor
                && event["seal"].as_str().is_some_and(|id| seals.contains(id)))
            .then(|| event["id"].as_str().map(str::to_owned))
            .flatten()
        })
        .collect();
    history
        .attempts
        .retain(|attempt| attempts.contains(&attempt.attempt));
    history.ordering.retain(|entry| {
        let event = &entry["event"];
        match event["kind"].as_str() {
            Some("sealed") => event["id"].as_str().is_some_and(|id| seals.contains(id)),
            Some("started") => event["id"].as_str().is_some_and(|id| attempts.contains(id)),
            Some("observed") => event["scenario"]
                .as_str()
                .is_some_and(|id| selected.iter().any(|selected| selected.as_str() == id)),
            _ => event["attempt"]
                .as_str()
                .is_some_and(|id| attempts.contains(id)),
        }
    });
    for entry in &mut history.ordering {
        if let Some(context) = entry
            .get_mut("event")
            .and_then(|event| event.get_mut("context"))
            .and_then(Value::as_array_mut)
        {
            context.retain(|observation| {
                observation["scenario"]
                    .as_str()
                    .is_some_and(|id| selected.iter().any(|selected| selected.as_str() == id))
            });
        }
    }
    Ok(history)
}
</storycode:copy>

### Assemble `crates/hick-observe/src/prediction_scope.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-observe/src/prediction_scope.rs"><storycode:paste select="#f-a3362438d4c1-impl-Ledger"/><storycode:paste select="#f-a3362438d4c1-project"/></storycode:file>

</storycode:doc>
