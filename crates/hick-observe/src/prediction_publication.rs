//! Explicit scenario selection without releasing private actuals or diagnostics.
use super::*;
use std::collections::BTreeSet;

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
        let selected: BTreeSet<_> = scenarios.iter().cloned().collect();
        ensure!(
            !selected.is_empty() && selected.len() <= 64,
            "Select one to 64 scenarios"
        );
        let mut history = self.history(None)?;
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
        let total_attempts = history.attempts.len();
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
        let total_events = history.ordering.len();
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
            if let Some(context) = entry
                .get_mut("event")
                .and_then(|event| event.get_mut("context"))
                .and_then(Value::as_array_mut)
            {
                let before = context.len();
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
            scope: "Selected historical host declarations; actual values and private evidence withheld. No current-source certification, human review, upload or authenticity is established by serialization.".into(),
        })
    }
}
