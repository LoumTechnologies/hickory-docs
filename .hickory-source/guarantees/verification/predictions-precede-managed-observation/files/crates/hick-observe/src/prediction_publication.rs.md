<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-observe/src/prediction_publication.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../contract.md)

Captured SHA-256: `c26f0da40d4aeb566197dbd42df5d5e3e8a41e9acb7224c1c6f3803758667dcd`.

<book:entry path="crates/hick-observe/src/prediction_publication.rs">
//! Explicit scenario selection without releasing private actuals or diagnostics.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectedPredictionHistory {
    pub format: String,
    pub version: u32,
    pub selected_scenarios: Vec<String>,
    pub omitted_events: usize,
    pub omitted_attempts: usize,
    pub omitted_context_observations: usize,
    pub history: History,
    #[serde(default)]
    pub debugger_contexts: Vec<Value>,
    pub scope: String,
}

impl Ledger {
    /// Select complete scenario histories, never only their passing attempts.
    /// Actuals and private evidence remain withheld even from this export. A
    /// human-answer publication needs separate isolation; selection grants no
    /// disclosure or upload capability. There is no model-facing export tool.
    pub fn select_history(&self, scenarios: &[String]) -> Result<SelectedPredictionHistory> {
        self.select_history_inner(scenarios, None)
    }
    /// Actor-scoped metadata for an active predictor; human selections keep the
    /// separate complete-scenario publication path above.
    pub fn select_predictor_history(
        &self,
        scenarios: &[String],
        actor: &Actor,
    ) -> Result<SelectedPredictionHistory> {
        self.select_history_inner(scenarios, Some(actor))
    }
    fn select_history_inner(
        &self,
        scenarios: &[String],
        actor: Option<&Actor>,
    ) -> Result<SelectedPredictionHistory> {
        let selected: BTreeSet<_> = scenarios.iter().cloned().collect();
        ensure!(
            !selected.is_empty() && selected.len() <= 64,
            "Select one to 64 scenarios"
        );
        let mut history = self.history(None)?;
        let total_attempts = history.attempts.len();
        let total_events = history.ordering.len();
        let original_contexts: BTreeMap<_, _> = history
            .ordering
            .iter()
            .map(|entry| {
                (
                    entry["sequence"].as_u64().unwrap_or(0),
                    entry["event"]["context"].as_array().map_or(0, Vec::len),
                )
            })
            .collect();
        if let Some(actor) = actor {
            history = super::scope::project(history, actor, scenarios)?;
        }
        let mut seals = BTreeSet::new();
        let mut found = BTreeSet::new();
        for entry in &history.ordering {
            let event = &entry["event"];
            if event["kind"] == "sealed" {
                let seal: Seal = serde_json::from_value(event["seal"].clone())?;
                let scenario = seal.scenario.key()?;
                if selected.contains(&scenario) {
                    found.insert(scenario);
                    seals.insert(
                        event["id"]
                            .as_str()
                            .context("Missing seal identity")?
                            .to_owned(),
                    );
                }
            }
        }
        ensure!(
            found == selected,
            "Selected prediction scenario is unavailable"
        );
        history
            .attempts
            .retain(|attempt| selected.contains(&attempt.scenario));
        let attempts: BTreeSet<_> = history
            .attempts
            .iter()
            .map(|attempt| attempt.attempt.clone())
            .collect();
        let debugger_contexts = self.transaction(|journal| {
            generation(journal, history.generation)?;
            let mut contexts = Vec::new();
            for attempt in &attempts {
                let Some((_, evidence)) = finished(journal, attempt) else {
                    continue;
                };
                if ![Some("private_capture_v1"), Some("combined_live_v1")]
                    .contains(&evidence["adapter"].as_str())
                {
                    continue;
                }
                let Some(captures) = evidence["captures"].as_array() else {
                    continue;
                };
                for capture in captures {
                    let hits: Vec<_> = capture["hits"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|hit| {
                            let context =
                                hit.get("context").and_then(Value::as_object).map(|raw| {
                                    [
                                        "run",
                                        "thread",
                                        "frame",
                                        "function",
                                        "document_line",
                                        "source",
                                        "source_line",
                                        "breakpoint_bound",
                                    ]
                                    .into_iter()
                                    .filter_map(|key| {
                                        raw.get(key).map(|value| (key.to_owned(), value.clone()))
                                    })
                                    .collect::<serde_json::Map<_, _>>()
                                });
                            serde_json::json!({"index": hit["index"], "context":context})
                        })
                        .collect();
                    contexts.push(
                        serde_json::json!({"attempt":attempt,"adapter":evidence["adapter"],
                        "file":capture["spec"]["file"],"line":capture["spec"]["line"],
                        "problem":!capture["problem"].is_null(),"moved":!capture["moved"].is_null(),
                        "truncated":capture["truncated"],"hits":hits}),
                    );
                }
            }
            Ok(contexts)
        })?;
        // Retain original global sequence and hash references. Gaps are declared,
        // not renumbered into a fabricated complete journal. These commitments
        // alone are not an authentication or public journal-validation proof.
        history.ordering.retain(|entry| {
            let event = &entry["event"];
            match event["kind"].as_str() {
                Some("sealed") => event["id"].as_str().is_some_and(|id| seals.contains(id)),
                Some("started") => event["id"].as_str().is_some_and(|id| attempts.contains(id)),
                Some("observed") => event["scenario"]
                    .as_str()
                    .is_some_and(|id| selected.contains(id)),
                _ => event["attempt"]
                    .as_str()
                    .is_some_and(|id| attempts.contains(id)),
            }
        });
        let mut omitted_context_observations = 0;
        for entry in &mut history.ordering {
            let original_context = original_contexts
                .get(&entry["sequence"].as_u64().unwrap_or(0))
                .copied();
            if let Some(context) = entry
                .get_mut("event")
                .and_then(|event| event.get_mut("context"))
                .and_then(Value::as_array_mut)
            {
                let before = original_context.unwrap_or(context.len());
                context.retain(|observation| {
                    observation["scenario"]
                        .as_str()
                        .is_some_and(|id| selected.contains(id))
                });
                omitted_context_observations += before - context.len();
            }
        }
        Ok(SelectedPredictionHistory {
            format: "hickory-selected-prediction-history".into(), version: 1,
            selected_scenarios: selected.into_iter().collect(),
            omitted_events: total_events - history.ordering.len(),
            omitted_attempts: total_attempts - history.attempts.len(),
            omitted_context_observations,
            history,
            debugger_contexts,
            scope: if actor.is_some(){"Predictor-scoped historical declarations; foreign predictions and rationales are omitted. Global commitments have declared gaps; actuals remain withheld and no current certification or human judgment is established."}else{"Selected historical host declarations; actual values and private evidence withheld. No current-source certification, human review, upload or authenticity is established by serialization."}.into(),
        })
    }
}
</book:entry>

</book:doc>
