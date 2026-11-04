<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/prediction.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-72f0d919b421"></a>
## `crates/hick-literate/src/prediction.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-72f0d919b421-combined"></a>
### `combined`

Host-only output admission around the existing live pipeline. No application
enable flag: debugger and cross-interface isolation are still prerequisites.
The caller must own a fresh, private executor and durable ledger inaccessible
to the predictor. The adapter never returns pipeline output, files, errors,
recordings, or diagnostics, including on success. Execution permissions are
those of the supplied executor; prediction admission grants no capabilities.

<storycode:copy id="f-72f0d919b421-combined">
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
</storycode:copy>
<a id="f-72f0d919b421-debugger"></a>
### `debugger`

<storycode:copy id="f-72f0d919b421-debugger">
pub mod debugger;
</storycode:copy>
<a id="f-72f0d919b421-environment"></a>
### `environment`

<storycode:copy id="f-72f0d919b421-environment">
pub mod environment;
</storycode:copy>
<a id="f-72f0d919b421-gate"></a>
### `gate`

<storycode:copy id="f-72f0d919b421-gate">
mod gate;
</storycode:copy>
<a id="f-72f0d919b421-source_question"></a>
### `source_question`

<storycode:copy id="f-72f0d919b421-source_question">
mod source_question;
</storycode:copy>
<a id="f-72f0d919b421-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-72f0d919b421-use_declaration">
pub use source_question::{SourceInputs, SourceQuestion};

</storycode:copy>
<a id="f-72f0d919b421-log_expectation"></a>
### `log_expectation`

<storycode:copy id="f-72f0d919b421-log_expectation">
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

</storycode:copy>
<a id="f-72f0d919b421-OutputEncoding"></a>
### `OutputEncoding`

<storycode:copy id="f-72f0d919b421-OutputEncoding">
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputEncoding {
    Text,
    Json,
}

</storycode:copy>
<a id="f-72f0d919b421-SealedRequest"></a>
### `SealedRequest`

Host-captured target request shared by debugger and combined observations.
Adapters validate source/target correspondence before the common gate admits it.

<storycode:copy id="f-72f0d919b421-SealedRequest">
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

</storycode:copy>
<a id="f-72f0d919b421-OutputRequest"></a>
### `OutputRequest`

Host-captured admission; none of these identities come from tool arguments.
Source identity is independently recomputed here. The host must capture the
concrete inputs and executor/toolchain environment and recognize the scenario
family. Unestablished correspondence must have identity_known=false in the seal.

<storycode:copy id="f-72f0d919b421-OutputRequest">
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

</storycode:copy>
<a id="f-72f0d919b421-ManagedOutput"></a>
### `ManagedOutput`

<storycode:copy id="f-72f0d919b421-ManagedOutput">
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedOutput {
    pub summary: Summary,
    /// False for an idempotent retry, whether running, settled or interrupted.
    pub dispatched: bool,
}

</storycode:copy>
<a id="f-72f0d919b421-run_output"></a>
### `run_output`

A bounded output adapter, not universal application strict mode. Only one
standalone exec is supported. Unsupported capture/project/volume/agent/cache
paths fail before admission or execution; they cannot silently fall back.
No hooks or caller-selected pipeline configuration are accepted.

<storycode:copy id="f-72f0d919b421-run_output">
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
</storycode:copy>
<a id="f-72f0d919b421-run_output_with_environment"></a>
### `run_output_with_environment`

Revalidate a host-selected runtime before dispatch and before private settlement.

A captured project is evaluated from a supplied source map, archived as regular
files and observed through the existing admission/settlement gate. The capture
binds all Markdown bytes and the host-owned command, image and timeout. No live
working directory is consulted. The runtime probe must describe the actual
executor; the host owns isolation and observer integrity. This initial host API
is available to host-bound shared agent tools, while ordinary web/ACP startup remains incomplete. Its LocalExecutor witnesses do not
certify quarantine. It deliberately accepts JSON observations without knowing
Lean, TLC, or any particular proof language. The focused tests below exercise
multi-file imports, replay, changed identities and malformed output.

<storycode:copy id="f-72f0d919b421-run_output_with_environment">
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

</storycode:copy>
<a id="f-72f0d919b421-output_target"></a>
### `output_target`

Exact target ID includes the output decoding semantics. Renaming the cell
cannot reset the scenario's ledger history.

<storycode:copy id="f-72f0d919b421-output_target">
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

</storycode:copy>
<a id="f-72f0d919b421-validate"></a>
### `validate`

<storycode:copy id="f-72f0d919b421-validate">
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
</storycode:copy>


### Capture source instead of consulting the live tree

The host owns an archive, source identity and observation plan. Fields are private so a model tool cannot replace a command or file after sealing.

<storycode:copy id="captured-project-types">
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

</storycode:copy>

### Construct a bounded regular-file archive

Evaluate the supplied Markdown map with the ordinary project evaluator. Refuse retained runtime recordings, symlinks, editor buffers, unsafe paths and oversized outputs. Preserve executable metadata. This step is construction, not an execution or proof result.

<storycode:copy id="captured-project-construction">
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
</storycode:copy>

### Bind the observation and protect its checker

Hash source separately from the execution plan. A host may pin generated observers and configurations; missing or changed protected files refuse a repair, and their fingerprints become part of the plan. This is pre-execution integrity, not a read-only runtime mount.

<storycode:copy id="captured-project-identities">
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
</storycode:copy>

### Admit before observing and settle without revealing

Revalidate the actual host runtime, admit through the shared ledger, and supply the archive to a fresh executor. Replays return their existing summary. Command errors, timeouts, malformed JSON and runtime drift become Unknown. Only the ledger compares prediction and requirement; raw transcripts remain private. Executor isolation, effective timeouts and output resource limits still require a reviewed host implementation.

An ordinary-development host can explicitly request public feedback with
`observe_unsealed`. It records exposure before dispatch and shares the same
execution path as sealed observations; it never fabricates a prediction for the
control arm. Ordinary development uses an agent actor; a held-out evaluator uses a host
actor and keeps its returned values outside candidate context. A host releasing public feedback after a sealed observation
must likewise record exposure before delivery. Earlier forecast receipts remain
immutable; future predictions then carry their exposure history.

<storycode:copy id="captured-project-execution">
        /// Explicit unsealed observation for a host-selected audience. Record exposure
        /// before dispatch, including failed runs, so later seals cannot claim an
        /// unobserved first attempt. The host owns audience routing: held-out values
        /// must never be returned to an agent, and use a host actor.
        pub async fn observe_unsealed<F>(
            &self,
            ledger: &Ledger,
            scenario: &hick_observe::prediction::Scenario,
            actor: Actor,
            identity: &Identity,
            executor: Arc<dyn Executor>,
            probe: F,
        ) -> Result<serde_json::Value>
        where
            F: Fn() -> Result<BTreeMap<String, String>>,
        {
            let before = probe()?;
            ensure!(*identity == self.identity(digest(&scenario.inputs)?, before.clone()));
            ensure!(!before.is_empty() && executor.transcripts().is_empty());
            ensure!(matches!(
                executor.script_platform(),
                hickory_executor::ScriptPlatform::Posix
            ));
            ledger.observe(scenario, identity.clone(), actor, "captured-project-public")?;
            self.execute_observation(executor, &probe, before).await
        }

        async fn execute_observation<F>(
            &self,
            executor: Arc<dyn Executor>,
            probe: &F,
            before: BTreeMap<String, String>,
        ) -> Result<serde_json::Value>
        where
            F: Fn() -> Result<BTreeMap<String, String>>,
        {
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
            Ok(actual)
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
            let observation = self
                .execute_observation(executor.clone(), &probe, before)
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

</storycode:copy>

### Challenge the host boundary with real execution

These focused witnesses cover multi-file imports, retries, changed source/plan/runtime, malformed output, command failure, timeout and prediction of a defect. The optional real playground witness runs Lean and TLC. It deliberately uses unknown identity and cannot certify independent prediction or adversarial quarantine.

<storycode:copy id="captured-project-witnesses">
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
        async fn public_observation_records_exposure_without_a_synthetic_seal() {
            let dir = tempfile::tempdir().unwrap();
            let ledger = Ledger::open(dir.path()).unwrap();
            let c = capture(7);
            let actor = Actor {
                kind: "agent".into(),
                id: "fixture".into(),
            };
            let scenario = Scenario {
                family: "project-observation-test".into(),
                inputs: serde_json::json!([]),
                requirement_reference: "value equals 7".into(),
                requirement: Predicate::Equals(serde_json::json!(7)),
                identity_known: true,
            };
            let identity = c.identity(digest(&scenario.inputs).unwrap(), env().unwrap());
            let value = c
                .observe_unsealed(
                    &ledger,
                    &scenario,
                    actor.clone(),
                    &identity,
                    Arc::new(LocalExecutor::new().unwrap()),
                    env,
                )
                .await
                .unwrap();
            assert_eq!(value, serde_json::json!(7));
            let history = ledger.history(None).unwrap();
            assert_eq!(history.generation, 1);
            assert!(history.attempts.is_empty());
            let seal = ledger
                .seal(
                    history.generation,
                    Seal {
                        scenario,
                        identity: identity.clone(),
                        target: c.target(),
                        predicted: Predicate::Equals(serde_json::json!(7)),
                        rationale: "already seen".into(),
                    },
                    actor.clone(),
                )
                .unwrap();
            let output = c
                .run(
                    &ledger,
                    &SealedRequest {
                        seal: seal.id,
                        request_id: "after-public".into(),
                        actor,
                        identity,
                        target: c.target(),
                    },
                    Arc::new(LocalExecutor::new().unwrap()),
                    env,
                )
                .await
                .unwrap();
            // An unsealed exposure conservatively makes later origin unknown.
            assert_eq!(output.summary.classification, Classification::Unknown);
            assert!(!output.summary.current_certification);
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
            // The playground now includes the runner chapter and lockfile.
            // Capture the declared closure instead of assuming a single page.
            let sources = hick_lang::load_project_sources("project.md", &|name| {
                std::fs::read_to_string(root.join(name)).map_err(|e| e.to_string())
            })
            .unwrap();
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
</storycode:copy>

### Assemble `crates/hick-literate/src/prediction.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/prediction.rs"><storycode:paste select="#f-72f0d919b421-combined"/><storycode:paste select="#f-72f0d919b421-debugger"/><storycode:paste select="#f-72f0d919b421-environment"/><storycode:paste select="#f-72f0d919b421-gate"/><storycode:paste select="#f-72f0d919b421-source_question"/><storycode:paste select="#f-72f0d919b421-use_declaration"/><storycode:paste select="#f-72f0d919b421-log_expectation"/><storycode:paste select="#f-72f0d919b421-OutputEncoding"/><storycode:paste select="#f-72f0d919b421-SealedRequest"/><storycode:paste select="#f-72f0d919b421-OutputRequest"/><storycode:paste select="#f-72f0d919b421-ManagedOutput"/><storycode:paste select="#f-72f0d919b421-run_output"/><storycode:paste select="#f-72f0d919b421-run_output_with_environment"/><storycode:paste select="#captured-project-types"/><storycode:paste select="#captured-project-construction"/><storycode:paste select="#captured-project-identities"/><storycode:paste select="#captured-project-execution"/><storycode:paste select="#captured-project-witnesses"/><storycode:paste select="#f-72f0d919b421-output_target"/><storycode:paste select="#f-72f0d919b421-validate"/></storycode:file>

</storycode:doc>
