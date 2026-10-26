<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang-wasm/src/prediction.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d27c09b74da1"></a>
## `crates/hick-lang-wasm/src/prediction.rs`

Related contracts: [Browser-local prediction practice uses the shared gate](../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-d27c09b74da1-Operation"></a>
### `Operation`

Browser host adapter onto the existing prediction ledger, not a new gate.
Raw snapshots include private results: never deliver them through UI tools.

<storycode:copy id="f-d27c09b74da1-Operation">
//! Browser host adapter onto the existing prediction ledger, not a new gate.
//! Raw snapshots include private results: never deliver them through UI tools.
use hick_observe::prediction::{Actor, Identity, Interruption, Ledger, ObservedResult, Seal};
use serde::Deserialize;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Operation {
    Recover {
        actor: Actor,
        scenarios: Vec<String>,
    },
    Interrupt {
        attempt: String,
    },
    Seal {
        generation: u64,
        seal: Seal,
        actor: Actor,
    },
    Admit {
        seal: String,
        request: String,
        identity: Identity,
        actor: Actor,
    },
    Finish {
        attempt: String,
        result: ObservedResult,
        execution_accepted: bool,
    },
    History {
        current: Identity,
        actor: Actor,
        scenarios: Vec<String>,
    },
    Promote {
        attempt: String,
        current: Identity,
        actor: Actor,
    },
    Reveal {
        attempt: String,
        actor: Actor,
    },
}
</storycode:copy>
<a id="f-d27c09b74da1-browser_prediction"></a>
### `browser_prediction`

Private browser-host transition. Actor and actual result arguments must be
supplied by trusted adapters, never forwarded from model invocation fields.
Storage is user-controlled, not a native private directory or authenticated
evidence store. Snapshot persistence must succeed before admitted dispatch.

<storycode:copy id="f-d27c09b74da1-browser_prediction">
/// Private browser-host transition. Actor and actual result arguments must be
/// supplied by trusted adapters, never forwarded from model invocation fields.
/// Storage is user-controlled, not a native private directory or authenticated
/// evidence store. Snapshot persistence must succeed before admitted dispatch.
#[wasm_bindgen]
pub fn browser_prediction(snapshot: &str, command: &str) -> String {
    let result = (|| -> anyhow::Result<Value> {
        anyhow::ensure!(
            command.len() <= 128 * 1024,
            "Browser prediction command capacity exceeded"
        );
        let ledger = Ledger::portable(if snapshot.is_empty() {
            None
        } else {
            Some(snapshot)
        })?;
        let value = match serde_json::from_str::<Operation>(command)? {
            Operation::Recover { actor, scenarios } => {
                // Only the browser host holding the exclusive runtime lease
                // may call this after confirming no managed worker is alive.
                let history = ledger.predictor_history(None, &actor, &scenarios)?;
                let settled: std::collections::BTreeSet<_> = history
                    .ordering
                    .iter()
                    .filter_map(|entry| {
                        let event = &entry["event"];
                        if matches!(event["kind"].as_str(), Some("finished" | "interrupted")) {
                            event["attempt"].as_str().map(str::to_owned)
                        } else {
                            None
                        }
                    })
                    .collect();
                let pending: Vec<_> = history
                    .ordering
                    .iter()
                    .filter_map(|entry| {
                        let event = &entry["event"];
                        if event["kind"] == "started" {
                            event["id"]
                                .as_str()
                                .filter(|id| !settled.contains(*id))
                                .map(str::to_owned)
                        } else {
                            None
                        }
                    })
                    .collect();
                for attempt in &pending {
                    ledger.interrupt(
                        attempt,
                        Actor {
                            kind: "host".into(),
                            id: "browser-runtime-recovery".into(),
                        },
                        Interruption::HostInterrupted,
                    )?;
                }
                json!({"interrupted":pending.len(),"execution_replayed":false,"current_certification":false})
            }
            Operation::Interrupt { attempt } => serde_json::to_value(ledger.interrupt(
                &attempt,
                Actor {
                    kind: "host".into(),
                    id: "browser-runtime".into(),
                },
                Interruption::EvidenceUnavailable,
            )?)?,
            Operation::Seal {
                generation,
                seal,
                actor,
            } => {
                let scenario = seal.scenario.key()?;
                json!({"receipt":ledger.seal(generation, seal, actor)?, "scenario":scenario})
            }
            Operation::Admit {
                seal,
                request,
                identity,
                actor,
            } => serde_json::to_value(ledger.admit(&seal, &request, &identity, false, actor)?)?,
            Operation::Finish {
                attempt,
                result,
                execution_accepted,
            } => {
                let mut summary = serde_json::to_value(ledger.finish_checked(
                    &attempt,
                    result,
                    execution_accepted,
                )?)?;
                summary["original_expectation_unchanged"] =
                    json!(ledger.original_expectation_unchanged(&attempt)?);
                summary
            }
            Operation::History {
                current,
                actor,
                scenarios,
            } => {
                let mut history = serde_json::to_value(ledger.predictor_history(
                    Some(&current),
                    &actor,
                    &scenarios,
                )?)?;
                if let Some(attempts) = history["attempts"].as_array_mut() {
                    for attempt in attempts {
                        let id = attempt["attempt"]
                            .as_str()
                            .ok_or_else(|| anyhow::anyhow!("Attempt identity unavailable"))?;
                        attempt["original_expectation_unchanged"] =
                            json!(ledger.original_expectation_unchanged(id)?);
                    }
                }
                history
            }
            Operation::Promote {
                attempt,
                current,
                actor,
            } => serde_json::to_value(ledger.promote(&attempt, &current, actor)?)?,
            Operation::Reveal { attempt, actor } => {
                ledger.reveal(&attempt, actor, "browser-human-inspection")?
            }
        };
        Ok(json!({"snapshot":ledger.portable_snapshot()?,"value":value,"error":null}))
    })();
    match result {
        Ok(value) => value.to_string(),
        // Invalid/private values may occur in serde/runtime errors. The browser
        // UI receives no diagnostic text from raw result/snapshot processing.
        Err(_) => json!({"snapshot":null,"value":null,"error":"Browser prediction operation refused; history, identity or ordering is unavailable"}).to_string(),
    }
}

</storycode:copy>
<a id="f-d27c09b74da1-tests"></a>
### `tests`

<storycode:copy id="f-d27c09b74da1-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use hick_observe::prediction::{Predicate, Scenario, Target, digest};
    use std::collections::BTreeMap;
    // Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
    #[test]
    fn browser_recovery_preserves_interrupted_history_and_refuses_replay() -> anyhow::Result<()> {
        let ledger = Ledger::portable(None)?;
        let inputs = json!({"attempts":[70,65],"budget":90});
        let identity = Identity {
            source: "a".repeat(64),
            inputs: digest(&inputs)?,
            dependencies: BTreeMap::new(),
            environment: BTreeMap::from([("worker".into(), "b".repeat(64))]),
        };
        let actor = Actor {
            kind: "agent".into(),
            id: "browser-fixture".into(),
        };
        let scenario = Scenario {
            family: "browser-recovery-fixture".into(),
            inputs,
            requirement_reference: "R1".into(),
            requirement: Predicate::NumberAtMost(90.0),
            identity_known: false,
        };
        let key = scenario.key()?;
        let sealed = ledger.seal(
            0,
            Seal {
                scenario,
                identity: identity.clone(),
                target: Target::Output {
                    execution: "deadline-output".into(),
                },
                predicted: Predicate::Equals(json!(90)),
                rationale: "One total budget".into(),
            },
            actor.clone(),
        )?;
        let pending = ledger.admit(&sealed.id, "original", &identity, false, actor.clone())?;
        assert!(pending.dispatch);
        let reply: Value = serde_json::from_str(&browser_prediction(
            &ledger.portable_snapshot()?,
            &json!({"operation":"recover","actor":actor,"scenarios":[key]}).to_string(),
        ))?;
        assert!(reply["error"].is_null());
        assert_eq!(reply["value"]["interrupted"], 1);
        assert_eq!(reply["value"]["execution_replayed"], false);
        let after = Ledger::portable(Some(reply["snapshot"].as_str().unwrap()))?;
        assert!(
            !after
                .admit(&sealed.id, "original", &identity, false, actor.clone())?
                .dispatch
        );
        assert!(
            after
                .admit(&sealed.id, "intentional-retry", &identity, false, actor)?
                .dispatch
        );
        assert_eq!(after.history(Some(&identity))?.attempts.len(), 2);
        assert_eq!(
            after.history(Some(&identity))?.attempts[0].classification,
            hick_observe::prediction::Classification::Unknown
        );
        Ok(())
    }
}
</storycode:copy>

### Assemble `crates/hick-lang-wasm/src/prediction.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang-wasm/src/prediction.rs"><storycode:paste select="#f-d27c09b74da1-Operation"/><storycode:paste select="#f-d27c09b74da1-browser_prediction"/><storycode:paste select="#f-d27c09b74da1-tests"/></storycode:file>

</storycode:doc>
