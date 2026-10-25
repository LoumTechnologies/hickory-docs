<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/private_prediction_startup.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `7ad28b3e9fd0bde75e64510c5d0aeabb8bb41231f5b8c0d1fa9b6e71b41bde4f`.

<book:entry path="crates/hickory-cli/src/serve/private_prediction_startup.rs">
//! Trusted process transport for a startup binding, never a model tool schema.
use super::private_predictions::Binding;
use anyhow::{Result, ensure};
use hick_literate::prediction::{OutputEncoding, SealedRequest};
use hick_observe::prediction::{Actor, Identity, Ledger, Target, digest};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Startup {
    pub ledger_root: PathBuf,
    pub seal: String,
    pub request_id: String,
    pub identity: Identity,
    pub target: Target,
    pub actor: Actor,
    pub encoding: OutputEncoding,
    #[serde(default)]
    pub question: Option<hick_observe::prediction::Scenario>,
    #[serde(default)]
    pub runtime: Option<hick_literate::prediction::environment::Profile>,
    #[serde(default)]
    pub inputs: Option<hick_literate::prediction::SourceInputs>,
    #[serde(default)]
    pub retained: Option<super::PrivatePredictionDebuggerPlan>,
}
impl Startup {
    pub(crate) fn fingerprint(&self) -> Result<String> {
        digest(self)
    }
    pub(crate) fn binding(&self, root: &std::path::Path) -> Result<Binding> {
        let private = self.ledger_root.canonicalize()?;
        ensure!(
            !private.starts_with(root),
            "Private ledger must be outside the predictor workspace"
        );
        let ledger = Arc::new(Ledger::open(&private)?);
        if self.seal.is_empty() {
            ensure!(
                self.question.is_some(),
                "An unsealed startup needs a host requirement and scenario"
            );
        } else {
            let history = ledger.history(None)?;
            let sealed = history
                .ordering
                .iter()
                .find(|entry| {
                    entry["event"]["kind"] == "sealed" && entry["event"]["id"] == self.seal
                })
                .ok_or_else(|| anyhow::anyhow!("Startup prediction seal unavailable"))?;
            let seal: hick_observe::prediction::Seal =
                serde_json::from_value(sealed["event"]["seal"].clone())?;
            ensure!(
                sealed["event"]["actor"] == serde_json::to_value(&self.actor)?,
                "Startup seal belongs to another predictor"
            );
            ensure!(
                seal.identity == self.identity && seal.target == self.target,
                "Startup prediction differs from its durable seal"
            );
        }
        Ok(Binding {
            ledger,
            request: SealedRequest {
                seal: self.seal.clone(),
                request_id: self.request_id.clone(),
                identity: self.identity.clone(),
                target: self.target.clone(),
                actor: self.actor.clone(),
            },
            encoding: self.encoding,
            question: self.question.clone(),
            runtime: self.runtime.clone(),
            inputs: self.inputs.clone(),
            retained: self.retained.clone(),
        })
    }
}
</book:entry>

</book:doc>
