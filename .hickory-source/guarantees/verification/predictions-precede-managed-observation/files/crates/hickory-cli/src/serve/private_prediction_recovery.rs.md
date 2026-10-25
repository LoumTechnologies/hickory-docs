<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/private_prediction_recovery.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `ea4e77588e033124fd06029efa150606c46061fc2d87533a41d6adf6af4c0fc2`.

<book:entry path="crates/hickory-cli/src/serve/private_prediction_recovery.rs">
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
</book:entry>

</book:doc>
