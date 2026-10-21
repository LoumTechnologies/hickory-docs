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
            if let Some(context) = entry["event"]["context"].as_array_mut() {
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
            scope: "Selected historical host declarations; actual values and private evidence withheld. No current-source certification, human review, upload or authenticity is established by serialization.".into(),
        })
    }
}
