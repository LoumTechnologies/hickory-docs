<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/tools/prediction.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fe56bb99d8b7"></a>
## `crates/hickory-agent/src/tools/prediction.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../contract.md).

<a id="f-fe56bb99d8b7-PredictionVerification"></a>
### `PredictionVerification`

Host-selected private verification for shared Rust document tools.

<storycode:copy id="f-fe56bb99d8b7-PredictionVerification">
//! Host-selected private verification for shared Rust document tools.
use super::*;
use hick_literate::prediction::{OutputEncoding, OutputRequest, SealedRequest, combined, debugger};
use hick_observe::prediction::{Classification, Ledger, Target};

pub(super) struct PredictionVerification {
    ledger: Arc<Ledger>,
    request: SealedRequest,
    encoding: OutputEncoding,
    environment: Option<hick_literate::prediction::environment::Profile>,
}
</storycode:copy>
<a id="f-fe56bb99d8b7-impl-EditSession"></a>
### `impl-EditSession`

<storycode:copy id="f-fe56bb99d8b7-impl-EditSession">
impl EditSession {
    /// Open directly under private policy: never load the ordinary transcript
    /// cache, import source closure, or weave runtime-derived material first.
    pub async fn open_prediction(
        doc_path: &Path,
        ledger: Arc<Ledger>,
        request: SealedRequest,
        encoding: OutputEncoding,
    ) -> Result<Self> {
        let source = std::fs::read_to_string(doc_path)
            .map_err(|_| anyhow::anyhow!("Private source unavailable"))?;
        private_source(&source)?;
        let session = Self {
            doc_path: doc_path.to_path_buf(),
            upstream: Default::default(),
            doc_name: doc_path.display().to_string(),
            params: vec![],
            source,
            weave: WeaveState {
                files: HashMap::new(),
                provenance: HashMap::new(),
            },
            verified_publication: false,
            prediction: None,
            root: project_root(doc_path),
        };
        session.require_prediction(ledger, request, encoding)
    }
    /// Host-only policy, retained across edits. Tool arguments cannot disable it
    /// or replace its seal. This scopes these document tools; attached UI and
    /// external processes require their own host quarantine before app enablement.
    pub fn require_prediction(
        mut self,
        ledger: Arc<Ledger>,
        request: SealedRequest,
        encoding: OutputEncoding,
    ) -> Result<Self> {
        anyhow::ensure!(
            self.upstream.is_empty() && self.params.is_empty(),
            "Private verification requires a standalone captured source"
        );
        anyhow::ensure!(
            sha256_hex(self.source.as_bytes()) == request.identity.source,
            "Private verification source differs from its captured identity"
        );
        private_source(&self.source)?;
        self.weave = WeaveState {
            files: HashMap::new(),
            provenance: HashMap::new(),
        };
        self.prediction = Some(PredictionVerification {
            ledger,
            request,
            encoding,
            environment: None,
        });
        Ok(self)
    }
    /// Trusted host runtime recipe, never a document tool argument.
    pub fn bind_prediction_environment(
        &mut self,
        profile: hick_literate::prediction::environment::Profile,
    ) -> Result<()> {
        self.prediction
            .as_mut()
            .context("Private policy required")?
            .environment = Some(profile);
        Ok(())
    }
    /// Only the host can install the next source-bound seal after a repair.
    pub fn replace_prediction(&mut self, request: SealedRequest) -> Result<()> {
        let policy = self
            .prediction
            .as_mut()
            .context("Private verification policy is not active")?;
        anyhow::ensure!(
            request.actor == policy.request.actor,
            "Prediction owner cannot change"
        );
        anyhow::ensure!(
            sha256_hex(self.source.as_bytes()) == request.identity.source,
            "Seal is not bound to the repaired source"
        );
        policy.request = request;
        Ok(())
    }
    pub(super) async fn verify_prediction(&self, executor: Arc<dyn Executor>) -> ToolOutcome {
        let policy = self.prediction.as_ref().expect("private policy checked");
        let project = self.doc_path.parent().unwrap_or(Path::new("."));
        let request = &policy.request;
        let result = match &request.target {
            Target::Output { .. } => {
                hick_literate::prediction::run_output_with_environment(
                    &policy.ledger,
                    &OutputRequest {
                        seal: request.seal.clone(),
                        request_id: request.request_id.clone(),
                        actor: request.actor.clone(),
                        identity: request.identity.clone(),
                        encoding: policy.encoding,
                    },
                    &self.source,
                    executor,
                    policy.environment.as_ref(),
                )
                .await
            }
            Target::Debugger { .. } => {
                debugger::run_with_environment(
                    &policy.ledger,
                    request,
                    &self.source,
                    project,
                    policy.environment.as_ref(),
                )
                .await
            }
            Target::Combined { .. } => {
                combined::run_with_environment(
                    &policy.ledger,
                    request,
                    &self.source,
                    project,
                    executor,
                    policy.environment.as_ref(),
                )
                .await
            }
        };
        let result = match result {
            Ok(result) => result,
            Err(_) => return ToolOutcome::err("verify", "Private prediction verification refused: seal, source, inputs, target or environment is unavailable".into()),
        };
        if std::fs::read_to_string(&self.doc_path).ok().as_ref() != Some(&self.source) {
            return ToolOutcome::err("verify", "Source changed during private verification; no current certification or output publication".into());
        }
        let passed = matches!(
            result.summary.classification,
            Classification::FirstAttemptVerified
                | Classification::InformedPredictionVerified
                | Classification::VerifiedAfterCorrection
                | Classification::VerifiedAfterPriorAttempt
        );
        let text = serde_json::to_string(&result).expect("bounded feedback serializes");
        // No generated files, captures or diagnostics are published even on pass.
        // Promotion is a distinct host action against the exact current identity.
        if passed {
            ToolOutcome::ok("verify", text)
        } else {
            ToolOutcome::err("verify", text)
        }
    }
}

</storycode:copy>
<a id="f-fe56bb99d8b7-private_source"></a>
### `private_source`

Reject source formats that can pull runtime/cached/imported material into reads.
This is source admission, never a value evaluator.

<storycode:copy id="f-fe56bb99d8b7-private_source">
/// Reject source formats that can pull runtime/cached/imported material into reads.
/// This is source admission, never a value evaluator.
pub(super) fn private_source(source: &str) -> Result<()> {
    let document =
        hick_lang::parse(source).map_err(|_| anyhow::anyhow!("Private source is unavailable"))?;
    anyhow::ensure!(
        document.all_tags().iter().all(|tag| {
            [
                "doc",
                "container",
                "file",
                "exec",
                "cmd",
                "expect",
                "capture",
                "volume",
            ]
            .contains(&tag.name.as_str())
        }),
        "Private source contains unsupported context or execution constructs"
    );
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/prediction.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/prediction.rs"><storycode:paste select="#f-fe56bb99d8b7-PredictionVerification"/><storycode:paste select="#f-fe56bb99d8b7-impl-EditSession"/><storycode:paste select="#f-fe56bb99d8b7-private_source"/></storycode:file>

</storycode:doc>
