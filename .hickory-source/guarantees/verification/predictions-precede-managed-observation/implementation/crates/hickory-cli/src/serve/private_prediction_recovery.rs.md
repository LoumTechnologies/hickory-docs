<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/private_prediction_recovery.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ed1889813c82"></a>
## `crates/hickory-cli/src/serve/private_prediction_recovery.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../contract.md).

<a id="f-ed1889813c82-Recovery"></a>
### `Recovery`

Trusted restart metadata. No program output or predictor history is stored here.

<storycode:copy id="f-ed1889813c82-Recovery">
//! Trusted restart metadata. No program output or predictor history is stored here.
use super::{private_prediction_startup::Startup, private_predictions::Bound};
use anyhow::{Result, ensure};
use hick_literate::prediction::SourceQuestion;
use hick_observe::prediction::{Scenario, Seal};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Recovery {
    version: u32,
    path: PathBuf,
    profile: Startup,
    question: SourceQuestion,
    scenario: Scenario,
    approved_source: String,
    executor: crate::ExecutorChoice,
}
</storycode:copy>
<a id="f-ed1889813c82-impl-Recovery"></a>
### `impl-Recovery`

<storycode:copy id="f-ed1889813c82-impl-Recovery">
impl Recovery {
    pub(crate) fn capture(
        bound: &Bound,
        profile: Startup,
        executor: crate::ExecutorChoice,
    ) -> Self {
        Self {
            version: 1,
            path: bound.path.clone(),
            profile,
            question: bound.source_question.clone(),
            scenario: bound.question.clone(),
            approved_source: bound.identity.read().unwrap().source.clone(),
            executor,
        }
    }
    pub(crate) fn protected_store(&self) -> &Path {
        &self.profile.ledger_root
    }
    pub(crate) fn root(&self) -> &Path {
        self.path.parent().expect("captured absolute document")
    }
    pub(crate) fn approve(&self, source: &str) -> Self {
        Self {
            approved_source: format!("{:x}", Sha256::digest(source)),
            ..self.clone()
        }
    }
    pub(crate) fn save(&self) -> Result<()> {
        crate::engine::private_policy::store_binding(self.root(), self)
    }
    pub(crate) fn restore(
        &self,
        root: &Path,
        executor: crate::ExecutorChoice,
    ) -> Result<(PathBuf, Startup)> {
        ensure!(
            self.version == 1 && self.root() == root && self.path.canonicalize()? == self.path,
            "Private recovery document moved; trusted host review is required"
        );
        ensure!(
            self.executor == executor,
            "Private recovery requires its original executor; no execution-permission fallback is allowed"
        );
        let source = std::fs::read_to_string(&self.path)
            .map_err(|_| anyhow::anyhow!("Private recovery source unavailable"))?;
        let hash = format!("{:x}", Sha256::digest(source.as_bytes()));
        ensure!(
            hash == self.approved_source,
            "Private source changed outside the approved session; trusted host recovery is required"
        );
        let mut profile = self.profile.clone();
        profile.identity.source = hash;
        if let Some(inputs) = &profile.inputs {
            inputs.validate(&profile.identity, &source)?;
        }
        profile.target = self.question.rebind(&profile.identity, &source)?;
        profile.question = Some(self.scenario.clone());
        let ledger = hick_observe::prediction::Ledger::open(&profile.ledger_root)?;
        let history = ledger.history(None)?;
        let actor = serde_json::to_value(&profile.actor)?;
        profile.seal = history
            .ordering
            .iter()
            .rev()
            .find_map(|entry| {
                let event = &entry["event"];
                if event["kind"] != "sealed" || event["actor"] != actor {
                    return None;
                }
                let seal: Seal = serde_json::from_value(event["seal"].clone()).ok()?;
                (seal.identity == profile.identity
                    && seal.target == profile.target
                    && seal.scenario == self.scenario)
                    .then(|| event["id"].as_str().map(str::to_owned))
                    .flatten()
            })
            .unwrap_or_default();
        profile.request_id = format!("restored:{}", crate::engine::window_token());
        Ok((self.path.clone(), profile))
    }
}
</storycode:copy>
<a id="f-ed1889813c82-impl-Bound"></a>
### `impl-Bound`

<storycode:copy id="f-ed1889813c82-impl-Bound">
impl Bound {
    pub(crate) fn install_recovery(
        &self,
        profile: Startup,
        executor: crate::ExecutorChoice,
    ) -> Result<()> {
        let recovery = Recovery::capture(self, profile, executor);
        recovery.save()?;
        self.recovery
            .set(recovery)
            .map_err(|_| anyhow::anyhow!("Private recovery already installed"))
    }
    pub(super) fn retain_approved_source(&self, source: &str) -> Result<()> {
        if let Some(recovery) = self.recovery.get() {
            recovery.approve(source).save()?;
        }
        Ok(())
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/private_prediction_recovery.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/private_prediction_recovery.rs"><storycode:paste select="#f-ed1889813c82-Recovery"/><storycode:paste select="#f-ed1889813c82-impl-Recovery"/><storycode:paste select="#f-ed1889813c82-impl-Bound"/></storycode:file>

</storycode:doc>
