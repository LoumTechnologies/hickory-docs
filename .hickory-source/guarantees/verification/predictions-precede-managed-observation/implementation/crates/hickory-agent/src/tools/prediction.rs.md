<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/tools/prediction.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fe56bb99d8b7"></a>
## `crates/hickory-agent/src/tools/prediction.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../contract.md).

<a id="f-fe56bb99d8b7-PredictionVerification"></a>
### `PredictionVerification`

The host fixes the supporting source map and observation plan. It can protect
selected generated outputs by content and executable-mode fingerprints, so source
repairs cannot quietly rewrite the observer or model bounds. Runtime writes are
not prevented by these pre-execution checks: a reviewed executor must provide
that boundary. An entry edit rebuilds against these fixed inputs without running
commands or exposing cached output.

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
    project: Option<ProjectPolicy>,
}

struct ProjectPolicy {
    entry: String,
    protected: std::collections::BTreeMap<String, String>,
    sources: std::collections::BTreeMap<String, String>,
    image: String,
    command: String,
    timeout: std::time::Duration,
    probe: Arc<dyn Fn() -> Result<std::collections::BTreeMap<String, String>> + Send + Sync>,
}
impl ProjectPolicy {
    fn capture(
        &self,
        source: &str,
    ) -> Result<hick_literate::prediction::project_observation::Capture> {
        let mut sources = self.sources.clone();
        sources.insert(self.entry.clone(), source.into());
        hick_literate::prediction::project_observation::Capture::new(
            &self.entry,
            &sources,
            &self.image,
            &self.command,
            self.timeout,
        )?
        .protect(&self.protected)
    }
}
</storycode:copy>
<a id="f-fe56bb99d8b7-impl-EditSession"></a>
### `impl-EditSession`

The captured-project constructor keeps the existing read/edit/verify dispatch and
its private output refusals. Entry edits reconstruct against the fixed captured
support pages, and a replacement seal must bind that entire construction and the
host observation plan. No automatic language-specific proof tools are added.
Ordinary web/ACP startup still needs a reviewed project binding and quarantine;
this constructor alone does not enable it globally.

<storycode:copy id="f-fe56bb99d8b7-impl-EditSession">
impl EditSession {
    /// Trusted-host constructor for a captured project. The existing document
    /// tools edit only the entry; other captured source pages remain fixed.
    /// No disk outputs or live includes enter private context. UI/ACP startup
    /// and a fully isolated runtime remain separate host responsibilities.
    pub fn open_project_prediction(
        doc_path: &Path,
        entry: String,
        sources: std::collections::BTreeMap<String, String>,
        protected: std::collections::BTreeMap<String, String>,
        image: String,
        command: String,
        timeout: std::time::Duration,
        ledger: Arc<Ledger>,
        request: SealedRequest,
        probe: Arc<dyn Fn() -> Result<std::collections::BTreeMap<String, String>> + Send + Sync>,
    ) -> Result<Self> {
        let source = sources
            .get(&entry)
            .context("Captured entry unavailable")?
            .clone();
        anyhow::ensure!(
            std::fs::read_to_string(doc_path).ok().as_ref() == Some(&source),
            "Entry differs from captured source"
        );
        let project = ProjectPolicy {
            entry,
            protected,
            sources,
            image,
            command,
            timeout,
            probe,
        };
        let capture = project.capture(&source)?;
        anyhow::ensure!(
            capture.identity(
                request.identity.inputs.clone(),
                request.identity.environment.clone()
            ) == request.identity
                && capture.target() == request.target,
            "Captured project binding differs"
        );
        Ok(Self {
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
            root: project_root(doc_path),
            prediction: Some(PredictionVerification {
                ledger,
                request,
                encoding: OutputEncoding::Json,
                environment: None,
                project: Some(project),
            }),
        })
    }
    pub(super) fn validate_prediction_source(&self) -> Result<()> {
        match self
            .prediction
            .as_ref()
            .and_then(|policy| policy.project.as_ref())
        {
            Some(project) => {
                project.capture(&self.source)?;
                Ok(())
            }
            None => private_source(&self.source),
        }
    }

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
            project: None,
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
        let source_identity = match &policy.project {
            Some(project) => project.capture(&self.source)?.identity(
                request.identity.inputs.clone(),
                request.identity.environment.clone(),
            ),
            None => {
                let mut identity = request.identity.clone();
                identity.source = sha256_hex(self.source.as_bytes());
                identity
            }
        };
        anyhow::ensure!(
            source_identity == request.identity,
            "Seal is not bound to the repaired source and plan"
        );
        policy.request = request;
        Ok(())
    }
    pub(super) async fn verify_prediction(&self, executor: Arc<dyn Executor>) -> ToolOutcome {
        let policy = self.prediction.as_ref().expect("private policy checked");
        let project = self.doc_path.parent().unwrap_or(Path::new("."));
        let request = &policy.request;
        let result = if let Some(project) = &policy.project {
            match project.capture(&self.source) {
                Ok(capture) => {
                    capture
                        .run(&policy.ledger, request, executor, || (project.probe)())
                        .await
                }
                Err(_) => Err(anyhow::anyhow!("Captured project unavailable")),
            }
        } else {
            match &request.target {
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

#[cfg(test)]
mod captured_project_tests {
    use super::*;
    use hick_observe::prediction::{Actor, Predicate, Scenario, Seal, digest};
    use std::{collections::BTreeMap, time::Duration};
    fn environment() -> Result<BTreeMap<String, String>> {
        Ok(BTreeMap::from([(
            "trusted-test".into(),
            digest(&"local-python")?,
        )]))
    }
    #[tokio::test]
    async fn project_uses_existing_private_tools_and_stales_after_edit() {
        let temp = tempfile::tempdir().unwrap();
        let source = r##"<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:project />
<h:copy id="value">VALUE = 17071
</h:copy>
<h:file path="value.py"><h:paste select="#value"/></h:file>
<h:file path="main.py">from value import VALUE
print(VALUE)
</h:file>
</h:doc>"##;
        let path = temp.path().join("project.md");
        std::fs::write(&path, source).unwrap();
        let sources = BTreeMap::from([("project.md".into(), source.into())]);
        let capture = hick_literate::prediction::project_observation::Capture::new(
            "project.md",
            &sources,
            "host-python",
            "python3 main.py",
            Duration::from_secs(10),
        )
        .unwrap();
        let protected =
            BTreeMap::from([("main.py".into(), capture.fingerprints()["main.py"].clone())]);
        let capture = capture.protect(&protected).unwrap();
        let ledger = Arc::new(Ledger::open(temp.path().join("private-ledger")).unwrap());
        let identity = capture.identity(
            digest(&serde_json::json!([])).unwrap(),
            environment().unwrap(),
        );
        let target = capture.target();
        let actor = Actor {
            kind: "agent".into(),
            id: "fixture".into(),
        };
        let seal = ledger
            .seal(
                ledger.history(None).unwrap().generation,
                Seal {
                    scenario: Scenario {
                        family: "project-tools".into(),
                        inputs: serde_json::json!([]),
                        requirement_reference: "fixture value".into(),
                        requirement: Predicate::Equals(serde_json::json!(17071)),
                        identity_known: false,
                    },
                    identity: identity.clone(),
                    target: target.clone(),
                    predicted: Predicate::Equals(serde_json::json!(17071)),
                    rationale: "captured import".into(),
                },
                actor.clone(),
            )
            .unwrap();
        let request = SealedRequest {
            seal: seal.id,
            request_id: "first".into(),
            actor,
            identity,
            target,
        };
        let mut session = EditSession::open_project_prediction(
            &path,
            "project.md".into(),
            sources,
            protected,
            "host-python".into(),
            "python3 main.py".into(),
            Duration::from_secs(10),
            ledger,
            request.clone(),
            Arc::new(environment),
        )
        .unwrap();
        let executor =
            || Arc::new(hickory_executor::LocalExecutor::new().unwrap()) as Arc<dyn Executor>;
        for tool in ["read_output", "read_file", "run_doc"] {
            let inv = ToolInvocation::synthetic(tool, vec![], None).unwrap();
            let outcome = execute_tool(&mut session, executor(), &inv).await;
            assert!(!outcome.ok);
            assert!(!outcome.text.contains("17071"));
        }
        let inv = ToolInvocation::synthetic("verify", vec![], None).unwrap();
        let outcome = execute_tool(&mut session, executor(), &inv).await;
        assert!(!outcome.text.contains("17071"));
        let result: serde_json::Value = serde_json::from_str(&outcome.text).unwrap();
        assert_eq!(result["summary"]["prediction_matched"], true);
        assert_eq!(result["summary"]["original_requirement_satisfied"], true);
        assert_eq!(result["summary"]["current_certification"], false);
        let original = session.source.clone();
        session.source = session.source.replace("print(VALUE)", "print(17071)");
        assert!(session.reweave().await.is_err());
        session.source = original;
        session.source = session.source.replace("17071", "17072");
        session.reweave().await.unwrap();
        assert!(session.weave.files.is_empty());
        assert!(session.replace_prediction(request).is_err());
        std::fs::write(&path, &session.source).unwrap();
        let outcome = execute_tool(&mut session, executor(), &inv).await;
        assert!(!outcome.ok);
        assert!(!outcome.text.contains("17072"));
        assert!(!temp.path().join("main.py").exists());
    }
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/prediction.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/prediction.rs"><storycode:paste select="#f-fe56bb99d8b7-PredictionVerification"/><storycode:paste select="#f-fe56bb99d8b7-impl-EditSession"/><storycode:paste select="#f-fe56bb99d8b7-private_source"/></storycode:file>

</storycode:doc>
