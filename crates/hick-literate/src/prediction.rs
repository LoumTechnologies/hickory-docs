//! Host-only output admission around the existing live pipeline. No application
//! enable flag: debugger and cross-interface isolation are still prerequisites.
//!
//! The caller must own a fresh, private executor and durable ledger inaccessible
//! to the predictor. The adapter never returns pipeline output, files, errors,
//! recordings, or diagnostics, including on success. Execution permissions are
//! those of the supplied executor; prediction admission grants no capabilities.
use crate::{PipelineConfig, expect::ExpectationOutcome};
use anyhow::{Result, ensure};
use hick_observe::prediction::{Actor, Identity, Ledger, Summary, Target};
use hickory_executor::Executor;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod combined;
pub mod debugger;
pub mod environment;
mod gate;
mod source_question;
pub use source_question::{SourceInputs, SourceQuestion};

pub(crate) fn log_expectation(outcome: &ExpectationOutcome, withheld: bool) {
    if !outcome.passed {
        if withheld {
            log::warn!("Managed expectation mismatch; actual values withheld");
        } else {
            log::warn!(
                "expectation failed in '{}' at {}:{}: {}",
                outcome.container.as_deref().unwrap_or("agent cell"),
                outcome.doc,
                outcome.line,
                outcome.detail
            );
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputEncoding {
    Text,
    Json,
}

/// Host-captured target request shared by debugger and combined observations.
/// Adapters validate source/target correspondence before the common gate admits it.
#[derive(Clone)]
pub struct SealedRequest {
    pub seal: String,
    pub request_id: String,
    pub actor: Actor,
    pub identity: Identity,
    pub target: Target,
}

/// Host-captured admission; none of these identities come from tool arguments.
/// Source identity is independently recomputed here. The host must capture the
/// concrete inputs and executor/toolchain environment and recognize the scenario
/// family. Unestablished correspondence must have identity_known=false in the seal.
pub struct OutputRequest {
    pub seal: String,
    pub request_id: String,
    pub actor: Actor,
    pub identity: Identity,
    pub encoding: OutputEncoding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedOutput {
    pub summary: Summary,
    /// False for an idempotent retry, whether running, settled or interrupted.
    pub dispatched: bool,
}

/// A bounded output adapter, not universal application strict mode. Only one
/// standalone exec is supported. Unsupported capture/project/volume/agent/cache
/// paths fail before admission or execution; they cannot silently fall back.
/// No hooks or caller-selected pipeline configuration are accepted.
pub async fn run_output(
    ledger: &Ledger,
    request: &OutputRequest,
    source: &str,
    executor: Arc<dyn Executor>,
) -> Result<ManagedOutput> {
    run_output_with_environment(ledger, request, source, executor, None).await
}
/// Revalidate a host-selected runtime before dispatch and before private settlement.
pub async fn run_output_with_environment(
    ledger: &Ledger,
    request: &OutputRequest,
    source: &str,
    executor: Arc<dyn Executor>,
    profile: Option<&environment::Profile>,
) -> Result<ManagedOutput> {
    let environment = environment::Profile::begin(profile, &request.identity).await?;
    // Parse failures can quote source text. Keep all refusal feedback bounded.
    let target = validate(request, source).map_err(|_| {
        anyhow::anyhow!(
            "Managed output admission unavailable: unsupported document or changed source identity"
        )
    })?;
    let gate = match gate::admit(
        ledger,
        &request.seal,
        &request.request_id,
        &request.identity,
        &request.actor,
        &target,
    )? {
        gate::Admission::Existing(result) => return Ok(result),
        gate::Admission::Fresh(gate) => gate,
    };
    // An executor previously used for another run may retain answer-bearing
    // transcripts/state. Never mix it with this target's evidence.
    if !executor.transcripts().is_empty() {
        return gate.unavailable();
    }
    let scratch = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(_) => return gate.unavailable(),
    };
    let config = PipelineConfig {
        working_dir: Some(scratch.path().to_owned()),
        withhold_actual_diagnostics: true,
        max_rounds: 1,
        ..Default::default()
    };
    let result =
        match crate::run_pipeline_live(&[("held-out.md", source)], &config, &[], None, executor)
            .await
        {
            Ok(result) => result,
            Err(_) => return gate.unavailable(),
        };
    let actual = match gate::decode_output(&result, &target, request.encoding) {
        Ok(actual) => actual,
        Err(_) => return gate.unavailable(),
    };
    // Store diagnostics only inside the private ledger. In particular there is
    // no PipelineResult or publishable output capability in the return value.
    let unchanged = match environment::Profile::unchanged(&environment, &request.identity).await {
        Ok(value) => value,
        Err(_) => return gate.unavailable(),
    };
    let execution_accepted = unchanged && result.expectations.iter().all(|outcome| outcome.passed);
    gate.finish(
        actual,
        serde_json::json!({ "transcripts": result.transcripts,
        "expectations": result.expectations, "adapter": "live_output_v1" }),
        execution_accepted,
    )
}

/// Captured multi-file observations reuse the private ledger. This is a host API,
/// not an agent-controlled command runner or an OS isolation guarantee.
pub mod project_observation {
    use super::*;
    use hick_observe::prediction::digest;
    use std::{collections::BTreeMap, time::Duration};

    pub struct Capture {
        archive: Vec<u8>,
        fingerprints: BTreeMap<String, String>,
        source: String,
        plan: String,
        image: String,
        command: String,
        timeout: Duration,
    }

    impl Capture {
        /// Evaluate only supplied source bytes; never read a live working tree.
        /// The trusted host owns the command and its result normalization policy.
        pub fn new(
            entry: &str,
            sources: &BTreeMap<String, String>,
            image: &str,
            command: &str,
            timeout: Duration,
        ) -> Result<Self> {
            ensure!(
                !image.is_empty()
                    && image.len() <= 4096
                    && !command.is_empty()
                    && command.len() <= 65536
            );
            ensure!(timeout > Duration::ZERO && timeout <= Duration::from_secs(600));
            ensure!(
                sources.len() <= 1024
                    && sources.values().map(String::len).sum::<usize>() <= 16 * 1024 * 1024
            );
            let evaluated =
                hick_lang::evaluate_project(entry, sources, &hick_lang::ProjectTree::default())
                    .map_err(|_| anyhow::anyhow!("Captured project construction unavailable"))?;
            // Cached exec/agent results cannot masquerade as fresh construction.
            ensure!(
                evaluated.recordings.is_empty(),
                "Recorded construction is not admitted"
            );
            let mut archive = tar::Builder::new(Vec::new());
            let mut total = 0usize;
            let mut fingerprints = BTreeMap::new();
            ensure!(evaluated.new.entries().len() <= 4096);
            for (path, file) in evaluated.new.entries() {
                ensure!(
                    file.kind == hick_lang::EntryKind::File,
                    "Only regular files are admitted"
                );
                ensure!(!path.starts_with(hick_lang::EDITOR_BUFFER_PREFIX));
                ensure!(
                    !path.is_empty()
                        && !path.contains('\\')
                        && !path.contains(':')
                        && path
                            .split('/')
                            .all(|p| !p.is_empty() && p != "." && p != "..")
                );
                total = total
                    .checked_add(file.bytes.len())
                    .ok_or_else(|| anyhow::anyhow!("Project too large"))?;
                ensure!(total <= 32 * 1024 * 1024);
                fingerprints.insert(
                    path.clone(),
                    digest(&serde_json::json!({
                    "bytes":file.bytes.as_ref(), "executable":file.executable}))?,
                );
                let mut header = tar::Header::new_gnu();
                header.set_size(file.bytes.len() as u64);
                header.set_mode(if file.executable { 0o755 } else { 0o644 });
                header.set_cksum();
                archive.append_data(&mut header, path, file.bytes.as_ref())?;
            }
            let source = digest(&serde_json::json!({"entry":entry,"sources":sources}))?;
            let plan = digest(&serde_json::json!({"image":image,"command":command,
                "timeout_ns":timeout.as_nanos().to_string(),"adapter":"captured-project-json-v1"}))?;
            Ok(Self {
                archive: archive.into_inner()?,
                fingerprints,
                source,
                plan,
                image: image.into(),
                command: command.into(),
                timeout,
            })
        }
        /// Fingerprints bind bytes and executable metadata, not just filenames.
        pub fn fingerprints(&self) -> &BTreeMap<String, String> {
            &self.fingerprints
        }
        /// Pin host-owned observers/configuration through candidate repairs.
        /// A missing or modified protected output refuses before execution.
        pub fn protect(mut self, protected: &BTreeMap<String, String>) -> Result<Self> {
            ensure!(
                protected
                    .iter()
                    .all(|(path, hash)| self.fingerprints.get(path) == Some(hash)),
                "Protected observation artifact changed"
            );
            self.plan = digest(&serde_json::json!({"plan":self.plan,"protected":protected}))?;
            Ok(self)
        }
        pub fn identity(&self, inputs: String, environment: BTreeMap<String, String>) -> Identity {
            Identity {
                source: self.source.clone(),
                inputs,
                dependencies: BTreeMap::from([("observation-plan".into(), self.plan.clone())]),
                environment,
            }
        }
        pub fn target(&self) -> Target {
            Target::Output {
                execution: format!("captured-project-json-v1:{}", self.plan),
            }
        }
        /// `probe` must measure the actual executor runtime, not caller claims.
        /// The host must isolate all answer-bearing channels and own the executor
        /// exclusively. LocalExecutor is useful for trusted tests only. Probes
        /// before/after execution cannot detect transient runtime replacement.
        pub async fn run<F>(
            &self,
            ledger: &Ledger,
            request: &SealedRequest,
            executor: Arc<dyn Executor>,
            probe: F,
        ) -> Result<ManagedOutput>
        where
            F: Fn() -> Result<BTreeMap<String, String>>,
        {
            ensure!(
                request.target == self.target()
                    && request.identity.source == self.source
                    && request.identity.dependencies
                        == BTreeMap::from([("observation-plan".into(), self.plan.clone())]),
                "Captured project identity changed"
            );
            ensure!(
                matches!(
                    executor.script_platform(),
                    hickory_executor::ScriptPlatform::Posix
                ),
                "Captured project command requires a POSIX executor"
            );
            ensure!(
                !request.identity.environment.is_empty(),
                "Runtime identity is required"
            );
            let before = probe().map_err(|_| anyhow::anyhow!("Private runtime unavailable"))?;
            ensure!(
                before == request.identity.environment,
                "Private runtime changed"
            );
            let gate = match gate::admit(
                ledger,
                &request.seal,
                &request.request_id,
                &request.identity,
                &request.actor,
                &request.target,
            )? {
                gate::Admission::Existing(result) => return Ok(result),
                gate::Admission::Fresh(gate) => gate,
            };
            if !executor.transcripts().is_empty() {
                return gate.unavailable();
            }
            let observation = async {
                executor
                    .ensure_started("private-project", &self.image)
                    .await?;
                executor
                    .inject_volume("private-project", "project", &self.archive)
                    .await?;
                let command = format!("cd project && (\n{}\n)", self.command);
                let output = executor
                    .execute_with_options(
                        "private-project",
                        &command,
                        None,
                        hickory_executor::ExecOptions {
                            timeout: Some(self.timeout),
                        },
                    )
                    .await?;
                ensure!(output.len() <= 1024 * 1024);
                let actual: serde_json::Value = serde_json::from_str(&output)?;
                ensure!(probe()? == before);
                Ok::<_, anyhow::Error>(actual)
            }
            .await;
            match observation {
                Ok(actual) => gate.finish(
                    actual,
                    serde_json::json!({
                    "adapter":"captured-project-json-v1","source":self.source,"plan":self.plan,
                    "transcripts":executor.transcripts()}),
                    true,
                ),
                Err(_) => gate.unavailable(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use hick_observe::prediction::*;
        use hickory_executor::LocalExecutor;
        fn capture(value: u8) -> Capture {
            let source = format!(
                r#"<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:project />
<h:file path="value.py">VALUE = {value}
</h:file>
<h:file path="main.py">from value import VALUE
print(VALUE)
</h:file>
</h:doc>"#
            );
            Capture::new(
                "project.md",
                &BTreeMap::from([("project.md".into(), source)]),
                "host-python",
                "python3 main.py",
                Duration::from_secs(10),
            )
            .unwrap()
        }
        fn env() -> Result<BTreeMap<String, String>> {
            Ok(BTreeMap::from([(
                "test-host".into(),
                digest(&"trusted-local-fixture")?,
            )]))
        }
        fn request(c: &Capture, ledger: &Ledger) -> SealedRequest {
            let actor = Actor {
                kind: "agent".into(),
                id: "fixture".into(),
            };
            let identity = c.identity(digest(&serde_json::json!([])).unwrap(), env().unwrap());
            let target = c.target();
            let seal = ledger
                .seal(
                    ledger.history(None).unwrap().generation,
                    Seal {
                        scenario: Scenario {
                            family: "project-observation-test".into(),
                            inputs: serde_json::json!([]),
                            requirement_reference: "value equals 7".into(),
                            requirement: Predicate::Equals(serde_json::json!(7)),
                            identity_known: false,
                        },
                        identity: identity.clone(),
                        target: target.clone(),
                        predicted: Predicate::Equals(serde_json::json!(7)),
                        rationale: "module imports captured value".into(),
                    },
                    actor.clone(),
                )
                .unwrap();
            SealedRequest {
                seal: seal.id,
                request_id: "first".into(),
                actor,
                identity,
                target,
            }
        }
        #[tokio::test]
        async fn multi_file_private_observation_and_retry() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let c = capture(7);
            let r = request(&c, &ledger);
            let result = c
                .run(&ledger, &r, Arc::new(LocalExecutor::new().unwrap()), env)
                .await
                .unwrap();
            assert_eq!(result.summary.prediction_matched, Some(true));
            assert_eq!(result.summary.original_requirement_satisfied, Some(true));
            let json = serde_json::to_value(&result).unwrap();
            assert!(json.get("actual").is_none());
            assert!(json.get("transcripts").is_none());
            let fresh = Arc::new(LocalExecutor::new().unwrap());
            let retry = c.run(&ledger, &r, fresh.clone(), env).await.unwrap();
            assert!(!retry.dispatched);
            assert!(fresh.transcripts().is_empty());
        }
        #[tokio::test]
        async fn changed_project_or_runtime_is_refused_before_dispatch() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let c = capture(7);
            let r = request(&c, &ledger);
            for changed in [false, true] {
                let e = Arc::new(LocalExecutor::new().unwrap());
                let other = capture(if changed { 8 } else { 7 });
                assert!(
                    other
                        .run(&ledger, &r, e.clone(), || if changed {
                            env()
                        } else {
                            Ok(BTreeMap::new())
                        })
                        .await
                        .is_err()
                );
                assert!(e.transcripts().is_empty());
            }
        }
        #[tokio::test]
        async fn timeout_cannot_become_prediction_credit() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let mut c = capture(7);
            c.command = "sleep 2; printf 7".into();
            c.timeout = Duration::from_millis(20);
            c.plan = digest(&"timeout-fixture").unwrap();
            let r = request(&c, &ledger);
            let result = c
                .run(&ledger, &r, Arc::new(LocalExecutor::new().unwrap()), env)
                .await
                .unwrap();
            assert_ne!(result.summary.prediction_matched, Some(true));
            assert_ne!(result.summary.original_requirement_satisfied, Some(true));
        }
        #[tokio::test]
        async fn changed_observation_plan_is_refused() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let c = capture(7);
            let r = request(&c, &ledger);
            let mut changed = capture(7);
            changed.plan = digest(&"other-plan").unwrap();
            let e = Arc::new(LocalExecutor::new().unwrap());
            assert!(changed.run(&ledger, &r, e.clone(), env).await.is_err());
            assert!(e.transcripts().is_empty());
        }
        #[tokio::test]
        async fn runtime_drift_after_execution_is_unknown() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let c = capture(7);
            let r = request(&c, &ledger);
            let calls = std::cell::Cell::new(0);
            let result = c
                .run(&ledger, &r, Arc::new(LocalExecutor::new().unwrap()), || {
                    calls.set(calls.get() + 1);
                    if calls.get() == 1 {
                        env()
                    } else {
                        Ok(BTreeMap::new())
                    }
                })
                .await
                .unwrap();
            assert_ne!(result.summary.original_requirement_satisfied, Some(true));
        }
        #[tokio::test]
        async fn failed_command_with_valid_json_is_unknown() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let mut c = capture(7);
            c.command = "printf 7; exit 1".into();
            c.plan = digest(&"nonzero-fixture").unwrap();
            let r = request(&c, &ledger);
            let result = c
                .run(&ledger, &r, Arc::new(LocalExecutor::new().unwrap()), env)
                .await
                .unwrap();
            assert_ne!(result.summary.original_requirement_satisfied, Some(true));
        }
        #[tokio::test]
        async fn correctly_predicted_defect_still_fails_requirement() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let c = capture(8);
            let actor = Actor {
                kind: "agent".into(),
                id: "fixture".into(),
            };
            let identity = c.identity(digest(&serde_json::json!([])).unwrap(), env().unwrap());
            let target = c.target();
            let seal = ledger
                .seal(
                    ledger.history(None).unwrap().generation,
                    Seal {
                        scenario: Scenario {
                            family: "predicted-defect".into(),
                            inputs: serde_json::json!([]),
                            requirement_reference: "value equals 7".into(),
                            requirement: Predicate::Equals(serde_json::json!(7)),
                            identity_known: false,
                        },
                        identity: identity.clone(),
                        target: target.clone(),
                        predicted: Predicate::Equals(serde_json::json!(8)),
                        rationale: "captured defect returns eight".into(),
                    },
                    actor.clone(),
                )
                .unwrap();
            let r = SealedRequest {
                seal: seal.id,
                request_id: "defect".into(),
                actor,
                identity,
                target,
            };
            let result = c
                .run(&ledger, &r, Arc::new(LocalExecutor::new().unwrap()), env)
                .await
                .unwrap();
            assert_eq!(result.summary.prediction_matched, Some(true));
            assert_eq!(result.summary.original_requirement_satisfied, Some(false));
        }
        // An exposed fixture exercises integration, never first-attempt cognition.
        #[tokio::test]
        #[ignore = "requires HICKORY_PROOFS_PLAYGROUND, Lean, Java and TLC_JAR"]
        async fn real_proofs_playground_through_private_ledger() {
            let root =
                std::path::PathBuf::from(std::env::var("HICKORY_PROOFS_PLAYGROUND").unwrap());
            let sources = BTreeMap::from([(
                "project.md".into(),
                std::fs::read_to_string(root.join("project.md")).unwrap(),
            )]);
            let command = "python3 explore.py > observation.json && python3 -c 'import json; print(json.dumps(json.load(open(\"observation.json\"))[\"accepted\"]))'";
            let c = Capture::new(
                "project.md",
                &sources,
                "trusted-host",
                command,
                Duration::from_secs(120),
            )
            .unwrap();
            let probe = || -> Result<BTreeMap<String, String>> {
                let mut out = BTreeMap::new();
                for (name, program, args) in [
                    ("lean", "lean", vec!["--version"]),
                    ("java", "java", vec!["-version"]),
                    ("python", "python3", vec!["--version"]),
                ] {
                    let result = std::process::Command::new(program)
                        .args(args)
                        .current_dir(&root)
                        .output()?;
                    ensure!(result.status.success());
                    out.insert(
                        name.into(),
                        digest(&serde_json::json!([result.stdout, result.stderr]))?,
                    );
                }
                let jar = std::fs::read(std::env::var("TLC_JAR")?)?;
                use sha2::{Digest, Sha256};
                out.insert("tlc-jar".into(), format!("{:x}", Sha256::digest(jar)));
                Ok(out)
            };
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let actor = Actor {
                kind: "agent".into(),
                id: "exposed-fixture".into(),
            };
            let identity = c.identity(digest(&serde_json::json!([])).unwrap(), probe().unwrap());
            let target = c.target();
            let seal = ledger
                .seal(
                    ledger.history(None).unwrap().generation,
                    Seal {
                        scenario: Scenario {
                            family: "public-vault-witnesses".into(),
                            inputs: serde_json::json!([]),
                            requirement_reference: "all public witnesses accepted".into(),
                            requirement: Predicate::Equals(serde_json::json!(true)),
                            identity_known: false,
                        },
                        identity: identity.clone(),
                        target: target.clone(),
                        predicted: Predicate::Equals(serde_json::json!(true)),
                        rationale: "previously observed fixture; no independent prediction credit"
                            .into(),
                    },
                    actor.clone(),
                )
                .unwrap();
            let r = SealedRequest {
                seal: seal.id,
                request_id: "public-fixture".into(),
                actor,
                identity,
                target,
            };
            let started = std::time::Instant::now();
            let result = c
                .run(&ledger, &r, Arc::new(LocalExecutor::new().unwrap()), probe)
                .await
                .unwrap();
            println!(
                "playground private observation: {} seconds; {}",
                started.elapsed().as_secs_f64(),
                serde_json::to_string(&result).unwrap()
            );
            assert_eq!(result.summary.prediction_matched, Some(true));
            assert_eq!(result.summary.original_requirement_satisfied, Some(true));
        }
        #[tokio::test]
        async fn invalid_json_is_unknown_not_a_successful_prediction() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let mut c = capture(7);
            c.command = "printf 'not-json'".into();
            // Host fixture mutation before sealing; production fields are private.
            c.plan = digest(&"invalid-json-fixture").unwrap();
            let r = request(&c, &ledger);
            let result = c
                .run(&ledger, &r, Arc::new(LocalExecutor::new().unwrap()), env)
                .await
                .unwrap();
            assert_ne!(result.summary.prediction_matched, Some(true));
            assert_ne!(result.summary.original_requirement_satisfied, Some(true));
        }
    }
}
/// Exact target ID includes the output decoding semantics. Renaming the cell
/// cannot reset the scenario's ledger history.
pub fn output_target(container: &str, line: usize, encoding: OutputEncoding) -> Target {
    let encoding = match encoding {
        OutputEncoding::Text => "text",
        OutputEncoding::Json => "json",
    };
    Target::Output {
        execution: format!("{container}:{line}:{encoding}"),
    }
}

fn validate(request: &OutputRequest, source: &str) -> Result<Target> {
    use sha2::{Digest, Sha256};
    ensure!(request.identity.source == format!("{:x}", Sha256::digest(source.as_bytes())));
    ensure!(request.identity.dependencies.is_empty());
    let document = hick_lang::parse(source)?;
    fn supported(nodes: &[hick_lang::HickNode]) -> bool {
        nodes.iter().all(|node| match node {
            hick_lang::HickNode::Tag(tag) => {
                ["doc", "container", "exec", "expect", "cmd", "file"].contains(&tag.name.as_str())
                    && supported(&tag.children)
            }
            _ => true,
        })
    }
    ensure!(supported(&document.nodes));
    let cells: Vec<_> = document
        .all_tags()
        .into_iter()
        .filter(|tag| tag.name == "exec")
        .collect();
    ensure!(cells.len() == 1);
    let cell = cells[0];
    // Even frozen-without-cache execution is unsupported: admission never
    // presents a replay declaration as an independent fresh runtime witness.
    ensure!(crate::tag_attr(cell, "freeze").is_none());
    let container = crate::tag_attr(cell, "container").unwrap_or_default();
    ensure!(!container.is_empty());
    Ok(output_target(
        &container,
        cell.source_line,
        request.encoding,
    ))
}
