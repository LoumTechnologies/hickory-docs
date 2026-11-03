<eval:doc xmlns:eval="http://www.hickorydocs.com/1.0">
# Measure features during ordinary Codex ACP requests

For: maintainers using Hickory to implement a change and collect evidence about
which authoring features help that work.

When a human checks **Measure this request with and without Hickory features**,
Send still performs the normal implementation. Before it starts, Hickory freezes
regular project files and named editor overlays. Afterwards, each selected
feature receives a fresh enabled and disabled Codex ACP session over separate
copies of those inputs. Trial edits are retained for review, never merged into
the primary turn's working tree by this mechanism.

The first interventions are literate source indexes and lineage output tools.
The index intervention covers `project.md`, `.hickory-source/` and current
source-backed views exported under `.hickory-indexes/` in the trial copy. It is
an availability ablation, not a claim that the agent actually read the index.
Lineage-tool ablation removes the named tools from discovery and refuses direct
calls. Other features need explicit interventions and independent witnesses;
the canonical contract inventory keeps that missing work visible.

## Read the evidence within its limits

A pair gets the same query and captured source, independent fresh sessions, and
the same selected model/configuration. It does not replay the primary agent's
conversation. Working-copy names are opaque and arm order is randomized; provider cache state and hidden agent memory
are not controlled. Working copies are not an OS sandbox for external adapters.
Inherited credentials, dependencies and host tools remain under their ordinary
policies; permission requests in trial sessions are refused rather than approved
without the person. Failures and unavailable measurements stay in the dataset.

Prompt wall time spans the ACP prompt call through completion, including its
tool work. Snapshot preparation, per-arm setup and end-to-end arm time are recorded
separately, so retaining an extra index does not masquerade as model latency. ACP's [usage updates](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/docs/protocol/v1/prompt-turn.mdx)
report context occupancy; that is not billed input. Only optional prompt-response
usage counters supply token totals. No cost is inferred from token counts.
The adapter remains the source of usage declarations. The install catalogue pins
Codex ACP 2.5.0. Its [prompt accumulator](https://raw.githubusercontent.com/agentclientprotocol/codex-acp/v2.5.0/src/CodexAcpServer.ts)
reports root-prompt totals; the earlier 2.0.1 pin exposes only the last model call.
Retain unqualified counters raw, but leave normalized prompt tokens null for
unaudited adapter identities/versions. Hidden child threads and provider internals
remain outside that evidence scope.

Completion is not correctness. Human reviews attach concrete observations or
focused test evidence to each arm. Only when both completed arms are accepted as
correct do we show disabled-minus-enabled savings, including negative values.
Optional contract attribution records the exact current contract hash with the review; changed contracts retain an unmeasured status until reviewed again.
One request is one observation, not a proof or a population estimate. Export raw
results, inspect failures, and accumulate different tasks and repetitions before
making broader claims. Every canonical contract starts as unmeasured rather than
inheriting correctness from unrelated trials.

Origin: the user's request to measure normal implementation work through Codex
ACP inside Hickory, including with/without literate indexes. This supersedes the
unimplemented standalone-runner approach. No paid evaluation is implied by
loading the page; Send with human opt-in dispatches the additional requests.

Implementation review (2026-11-03): partially verified. The inspected path reuses
Codex ACP, freezes the inputs before the primary turn, gates tool availability,
records qualified usage and keeps trial writes in separate working copies.

Executed checks: six focused Rust mechanism witnesses, seven UI/Send/export
witnesses, and one HTTP/ACP conformance flow passed. The conformance peer checks
frozen inputs, feature removal, retained artifacts, review retraction and contract
staleness; it is not a model-performance experiment. Type checking and final
source reconstruction are recorded separately during delivery. No live model
benefit, population-level saving, or coverage of unimplemented interventions is
claimed. Human judgment: not yet recorded; test review declarations are fixtures.
The code and focused witnesses below are the maintained implementation.

## Construct `crates/hickory-cli/src/serve/acp/experiments.rs`

Use the same ACP client and typed options so measurements cannot silently switch to a different agent or activate an unknown feature.

<eval:copy id="feature-experiments-experiments-0">
//! In-app ablations reuse Codex ACP; ordinary implementation remains the primary turn.
use super::super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use super::{client::Client, config::AgentCommand};
use anyhow::{Context, Result, ensure};
use axum::{
    Json,
    extract::{Path, State},
};
use hick_lang::{EntryKind, ProjectEntry, ProjectSnapshot, ProjectTree};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path as FsPath, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const FEATURES: &[(&str, &str)] = &[
    ("literate-indexes", "Literate source indexes"),
    (
        "lineage-tools",
        "Read and edit generated code through lineage",
    ),
];
const LIMITATIONS: &str = "Fresh working copies and ACP sessions; not an OS sandbox. Adapter-managed reads, caches and hidden context are opaque. Same request starts fresh rather than replaying the primary conversation. Human correctness reviews are declarations. Null tokens mean unavailable, not zero. Query savings exclude snapshot/setup cost.";

</eval:copy>

Bound the number of extra runs and the time per arm before any primary request starts. One request produces one enabled/disabled pair per selected feature.

<eval:copy id="feature-experiments-experiments-1">
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub features: Vec<String>,
    #[serde(default = "timeout_default")]
    pub timeout_seconds: u64,
}
fn timeout_default() -> u64 {
    300
}
impl Options {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.features.is_empty() && self.features.len() <= FEATURES.len(),
            "Select one or two measurable features"
        );
        let mut seen = std::collections::HashSet::new();
        for feature in &self.features {
            ensure!(
                FEATURES.iter().any(|(id, _)| id == feature) && seen.insert(feature),
                "Unknown or duplicate feature: {feature}"
            );
        }
        ensure!(
            (10..=600).contains(&self.timeout_seconds),
            "Comparison timeout must be 10–600 seconds per arm"
        );
        Ok(())
    }
}

</eval:copy>

Retain evidence in personal project storage, outside the repository. Hash exact bytes; validate route identities before resolving a result directory.

<eval:copy id="feature-experiments-experiments-2">
fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn directory(state: &LocalState) -> Result<PathBuf> {
    Ok(
        hickory_workspace::WorkspaceStore::for_project(state.index.root())?
            .dir()
            .join("feature-experiments"),
    )
}
fn save(path: &FsPath, value: &Value) -> Result<()> {
    std::fs::create_dir_all(path.parent().context("Missing parent")?)?;
    super::super::store::write_atomic(path, &serde_json::to_vec_pretty(value)?)
}
fn valid_id(id: &str) -> Result<()> {
    ensure!(
        id.len() == 16 && id.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid comparison identity"
    );
    Ok(())
}

</eval:copy>

Freeze Gitignore-visible regular files, preserving executable bits, and overlay current editor text without saving it. Refuse oversized inputs. Record omitted symlinks and runtime inputs rather than copying references into the live repository.

<eval:copy id="feature-experiments-experiments-3">
/// Freeze Gitignore-visible regular files plus live editor bytes without saving editors.
fn capture(
    root: &FsPath,
    overlays: BTreeMap<String, String>,
) -> Result<(ProjectTree, Vec<String>)> {
    let mut tree = ProjectTree::default();
    let mut omitted = Vec::new();
    let mut total = 0usize;
    for item in ignore::WalkBuilder::new(root)
        .hidden(false)
        .follow_links(false)
        .build()
    {
        let item = item?;
        if item.path() == root || item.file_type().is_some_and(|t| t.is_dir()) {
            continue;
        }
        let name = item
            .path()
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        if item.file_type().is_none_or(|t| !t.is_file()) {
            omitted.push(name);
            continue;
        }
        // Runtime evidence and secrets do not become task inputs, even outside Git.
        if name.starts_with("sessions/")
            || name.starts_with(".git/")
            || name == ".env"
            || name.starts_with(".env.")
        {
            omitted.push(name);
            continue;
        }
        ensure!(
            item.metadata()?.len() <= 16 * 1024 * 1024,
            "Snapshot file exceeds 16 MiB: {name}"
        );
        let bytes = std::fs::read(item.path())?;
        total += bytes.len();
        ensure!(
            total <= 256 * 1024 * 1024 && tree.entries().len() < 20000,
            "Comparison snapshot exceeds 256 MiB or 20000 files"
        );
        #[cfg(unix)]
        let executable = {
            use std::os::unix::fs::PermissionsExt;
            item.metadata()?.permissions().mode() & 0o111 != 0
        };
        #[cfg(not(unix))]
        let executable = false;
        tree.insert(
            &name,
            ProjectEntry {
                bytes: bytes.into(),
                kind: EntryKind::File,
                executable,
                origins: vec![],
            },
        )
        .map_err(anyhow::Error::msg)?;
    }
    for (name, text) in overlays {
        ensure!(
            tree.entries().contains_key(&name),
            "Save or include the named editor file before measuring: {name}"
        );
        let mut entry = tree.entries()[&name].as_ref().clone();
        entry.bytes = text.into_bytes().into();
        tree.insert(&name, entry).map_err(anyhow::Error::msg)?;
    }
    Ok((tree, omitted))
}

</eval:copy>

Capture once before the normal implementation turn. Include current source-backed readings only when their recorded code still matches the captured source. Snapshot bytes, command identity, selected settings and omissions make later comparisons auditable.

<eval:copy id="feature-experiments-experiments-4">
pub struct Plan {
    pub path: PathBuf,
    pub record: Value,
    tree: ProjectTree,
    command: AgentCommand,
    config: Value,
    options: Options,
    context: Value,
}

pub async fn prepare(
    state: &LocalState,
    doc: &str,
    turn: &str,
    prompt: &str,
    options: Options,
    context: &super::super::agent_context::EditorContext,
    client: &Client,
) -> Result<Plan> {
    options.validate()?;
    ensure!(
        client.agent == "codex",
        "Feature measurement currently requires Codex ACP"
    );
    let started = Instant::now();
    let mut overlays = BTreeMap::new();
    for (id, _) in state.index.entries() {
        if let Some(room) = state.rooms.get(&id).await
            && let Some(path) = state.index.absolute(&id)
        {
            if let Ok(name) = path.strip_prefix(state.index.root()) {
                overlays.insert(name.to_string_lossy().replace('\\', "/"), room.text().await);
            }
        }
    }
    for buffer in &context.buffers {
        let path = buffer
            .path
            .as_deref()
            .context("Save unnamed editor buffers before measuring")?;
        let path = FsPath::new(path);
        let name = if path.is_absolute() {
            path.strip_prefix(state.index.root())?.to_path_buf()
        } else {
            path.to_path_buf()
        };
        overlays.insert(
            name.to_string_lossy().replace('\\', "/"),
            buffer.content.clone(),
        );
    }
    let mut frozen_context = serde_json::to_value(context)?;
    frozen_context["ui_window"] = Value::Null;
    frozen_context["live"] = json!(false);
    for buffer in frozen_context["buffers"]
        .as_array_mut()
        .into_iter()
        .flatten()
    {
        if let Some(path) = buffer["path"].as_str() {
            let path = FsPath::new(path);
            let relative = path.strip_prefix(state.index.root()).unwrap_or(path);
            buffer["path"] = json!(relative.to_string_lossy().replace('\\', "/"));
        }
        buffer["provenance_uri"] = Value::Null;
        buffer["id"] = Value::Null;
    }
    let root = state.index.root().to_path_buf();
    let (mut tree, mut omitted) =
        tokio::task::spawn_blocking(move || capture(&root, overlays)).await??;
    super::super::representation_store::load(state)
        .await
        .map_err(|error| anyhow::anyhow!("{}", error.detail()))?;
    for view in state.representations.lock().await.values() {
        if !matches!(
            view.backing,
            super::super::representation::Backing::Files { .. }
        ) {
            continue;
        }
        if view.explanation_stale
            || !view.files.iter().all(|file| {
                tree.entries()
                    .get(&file.path)
                    .is_some_and(|entry| entry.bytes.as_ref() == file.content.as_bytes())
            })
        {
            omitted.push(format!("stale-view:{}", view.id));
            continue;
        }
        let name = format!(".hickory-indexes/{}.md", view.id);
        ensure!(
            !tree.entries().contains_key(&name),
            "Reserved comparison index path already exists: {name}"
        );
        tree.insert(
            &name,
            ProjectEntry {
                bytes: view.source.as_bytes().into(),
                kind: EntryKind::File,
                executable: false,
                origins: vec![],
            },
        )
        .map_err(anyhow::Error::msg)?;
    }
    if options.features.iter().any(|f| f == "literate-indexes") {
        ensure!(
            tree.entries().keys().any(|p| is_index(p)),
            "No project.md or .hickory-source index was captured; choose another feature"
        );
    }
    let command = super::config::commands(state)?
        .into_iter()
        .find(|c| c.id == "codex")
        .context("Codex ACP is not configured")?;
    ensure!(
        !command.workspace_filesystem,
        "Measurement needs ordinary ACP working copies; disable the native workspace mount for Codex first"
    );
    let config = client.snapshot();
    let manifest: BTreeMap<_, _> = tree
        .entries()
        .iter()
        .map(|(p, e)| {
            (
                p,
                json!({"sha256":sha(&e.bytes),"executable":e.executable,"bytes":e.bytes.len()}),
            )
        })
        .collect();
    let path = directory(state)?.join(turn);
    std::fs::create_dir_all(path.parent().context("Missing dataset directory")?)?;
    std::fs::create_dir(&path)?;
    let snapshot = serde_json::to_value(ProjectSnapshot::capture(&tree))?;
    save(&path.join("snapshot.json"), &snapshot)?;
    let record = json!({"version":1,"engine_version":env!("CARGO_PKG_VERSION"),"measurement_implementation_sha256":sha(include_bytes!("experiments.rs")),"id":turn,"document":doc,"prompt":prompt,"created_at":super::super::now_rfc3339(),"status":"queued","features":options.features,"snapshot_sha256":sha(&serde_json::to_vec(&snapshot)?),"source_manifest":manifest,"omitted":omitted,"agent":command,"agent_info":client.initialized,"config":config,"timeout_seconds":options.timeout_seconds,"preparation_seconds":started.elapsed().as_secs_f64(),"pairs":[],"limitations":LIMITATIONS});
    save(&path.join("record.json"), &record)?;
    Ok(Plan {
        path,
        record,
        tree,
        command,
        config,
        options,
        context: frozen_context,
    })
}

</eval:copy>

Keep each ablation narrow: omit only the declared literate index paths, or remove and refuse the named lineage tools. Qualify usage by the audited adapter implementation; field names alone do not establish whole-prompt accounting. The remaining files are exact clones of the same captured tree.

<eval:copy id="feature-experiments-experiments-5">
fn is_index(path: &str) -> bool {
    path == "project.md"
        || path.starts_with(".hickory-source/")
        || path.starts_with(".hickory-indexes/")
}
pub fn disabled_tool(feature: &str, name: &str) -> bool {
    feature == "lineage-tools"
        && matches!(name, "read_output" | "edit_output" | "trace" | "lineage")
}
fn arm_tree(tree: &ProjectTree, feature: &str, enabled: bool) -> Result<ProjectTree> {
    let mut result = ProjectTree::default();
    for (name, entry) in tree.entries() {
        if !enabled && feature == "literate-indexes" && is_index(name) {
            continue;
        }
        result
            .insert(name, entry.as_ref().clone())
            .map_err(anyhow::Error::msg)?;
    }
    Ok(result)
}

</eval:copy>

Each arm starts a new embedded Hickory state and Codex ACP process. Apply the same selected settings, automatically accept Hickory edits only inside the copy, and refuse unattended adapter permission requests. Keep transcripts, output snapshots and reported token spend; do not treat context occupancy as spend. A drop guard stops the adapter on timeout.

<eval:copy id="feature-experiments-experiments-6">
/// Audited adapter versions are evidence policy, not a guess based on field names.
fn whole_prompt_usage(agent: &Value, response: &Value) -> Value {
    let audited = agent["name"] == "@agentclientprotocol/codex-acp" && agent["version"] == "2.5.0";
    let mut result = json!({"input_tokens":null,"output_tokens":null,"total_tokens":null,"cached_read_tokens":null,"thought_tokens":null,"usage_scope":if audited { "adapter-reported root prompt totals; child threads and provider internals may be opaque" } else { "unqualified adapter counters; install audited Codex ACP 2.5.0 for prompt totals" }});
    if audited {
        for (field, key) in [
            ("input_tokens", "inputTokens"),
            ("output_tokens", "outputTokens"),
            ("total_tokens", "totalTokens"),
            ("cached_read_tokens", "cachedReadTokens"),
            ("thought_tokens", "thoughtTokens"),
        ] {
            if let Some(value) = response["usage"][key].as_u64() {
                result[field] = json!(value);
            }
        }
    }
    result
}

struct StopClient(Arc<Client>);
impl Drop for StopClient {
    fn drop(&mut self) {
        let client = self.0.clone();
        tokio::spawn(async move {
            client.shutdown().await;
        });
    }
}

/// A new embedded Hickory state owns all tools and rooms in each disposable copy.
async fn arm(
    state: &LocalState,
    plan: &Plan,
    feature: &str,
    enabled: bool,
    root: PathBuf,
    cancel: Arc<AtomicBool>,
) -> Value {
    let started = Instant::now();
    let name = if enabled { "enabled" } else { "disabled" };
    let mut observation = json!({"arm":name,"status":"error","correct":null,"review":null,"wall_seconds":null,"input_tokens":null,"output_tokens":null,"total_tokens":null,"cost_usd":null,"tools":null,"workspace":root});
    let outcome: Result<()> = async {
        ensure!(!cancel.load(Ordering::Relaxed), "Comparison stopped");
        std::fs::create_dir_all(&root)?;
        let tree = arm_tree(&plan.tree, feature, enabled)?;
        hick_lsp::write_tree(&root, &tree).map_err(anyhow::Error::msg)?;
        observation["input_manifest_sha256"] = json!(sha(&serde_json::to_vec(&ProjectSnapshot::capture(&tree))?));
        let prepared = super::super::prepare(super::super::ServeOptions { target:root.clone(), port:0, params:state.params.as_ref().clone(), executor:state.executor.clone(), key_store_path:None, ui_settings_path:None }).await?;
        let mut isolated = prepared.state;
        // Command lookup stays machine-local; the tool server's project root remains the copy.
        isolated.ui = state.ui.clone();
        let client = Client::start(&isolated, super::super::agent_context::WORKSPACE_AGENT, plan.command.clone(), root.join("sessions/comparison.md")).await?;
        let _stop = StopClient(client.clone());
        client.experiment.store(true, Ordering::Relaxed);
        if !enabled && feature == "lineage-tools" {
            *client.host.disabled_tools.lock().await = ["read_output", "edit_output", "trace", "lineage"].into_iter().filter(|name| disabled_tool(feature, name)).map(String::from).collect();
        }
        let job: Result<()> = async {
            client.setup().await?;
            // Reapply every selected session setting in both fresh contexts, never silently fall back.
            let session = client.session.lock().unwrap().clone().context("No session")?;
            if let Some(settings) = plan.config["configOptions"].as_array() {
                for setting in settings {
                    let value = client.rpc.request("session/set_config_option", json!({"sessionId":session,"configId":setting["id"],"value":setting["currentValue"]}), Duration::from_secs(30)).await?;
                    if let Some(options) = value.get("configOptions") { client.view.lock().unwrap()["configOptions"] = options.clone(); }
                }
            }
            if let Some(mode) = plan.config["modes"]["currentModeId"].as_str() {
                client.rpc.request("session/set_mode", json!({"sessionId":session,"modeId":mode}), Duration::from_secs(30)).await?;
            }
            ensure!(client.model() == plan.config["configOptions"].as_array().and_then(|opts| opts.iter().find(|o| o["category"] == "model" || o["id"] == "model")).and_then(|o| o["currentValue"].as_str()).or_else(|| plan.config["models"]["currentModelId"].as_str()).unwrap_or("agent default"), "Fresh ACP model differs from primary model");
            *client.host.edits.mode.lock().unwrap() = super::edits::Mode::AutoAccept;
            observation["model"] = json!(client.model());
            let mut context = plan.context.clone();
            if !enabled && feature == "literate-indexes" {
                if let Some(buffers) = context["buffers"].as_array_mut() { buffers.retain(|buffer| !buffer["path"].as_str().is_some_and(is_index)); }
            }
            let context: super::super::agent_context::EditorContext = serde_json::from_value(context)?;
            let described = super::super::agent_context::describe(&isolated, &context);
            observation["supplied_context"] = json!(described);
            observation["setup_seconds"] = json!(started.elapsed().as_secs_f64());
            let query_started = Instant::now();
            let result = client.prompt("comparison", None, plan.record["prompt"].as_str().unwrap_or_default(), &described, cancel.clone()).await;
            observation["wall_seconds"] = json!(query_started.elapsed().as_secs_f64());
            observation["observed"] = client.last_observation.lock().unwrap().clone();
            observation["tools"] = observation["observed"]["tool_calls"].clone();
            let response = observation["observed"]["prompt_response"].clone();
            // 2.0.1 exposes the last model call, while audited 2.5.0 accumulates the prompt.
            let measured = whole_prompt_usage(&client.initialized["agentInfo"], &response);
            for (key, value) in measured.as_object().context("Missing usage measurement")? { observation[key] = value.clone(); }
            observation["agent_info"] = client.initialized["agentInfo"].clone();
            let answer = result?;
            observation["answer"] = json!(answer);
            ensure!(!client.experiment_permission_denied.load(Ordering::Relaxed), "Adapter requested permission; comparison refused unattended authorization");
            observation["status"] = json!("completed");
            Ok(())
        }.await;
        client.shutdown().await;
        observation["transcript"] = json!(root.join("sessions/comparison.md"));
        job?;
        // Retain a portable output snapshot for later correctness review, including edits.
        let root_copy = root.clone();
        let (completed, _) = tokio::task::spawn_blocking(move || capture(&root_copy, BTreeMap::new())).await??;
        save(&plan.path.join(format!("{feature}-{name}-output.json")), &serde_json::to_value(ProjectSnapshot::capture(&completed))?)?;
        Ok(())
    }.await;
    observation["end_to_end_seconds"] = json!(started.elapsed().as_secs_f64());
    if let Err(error) = outcome {
        observation["status"] = json!("error");
        observation["error"] = json!(format!("{error:#}"));
    }
    observation
}

</eval:copy>

Run arms sequentially in randomized order and persist partial results before spending on another arm. Stop shares the primary turn cancellation flag; interrupted and failed arms remain evidence.

<eval:copy id="feature-experiments-experiments-7">
pub async fn execute(state: &LocalState, mut plan: Plan, cancel: Arc<AtomicBool>) -> Result<()> {
    plan.record["status"] = json!("running");
    save(&plan.path.join("record.json"), &plan.record)?;
    for feature in plan.options.features.clone() {
        let mut pair = json!({"feature":feature,"arms":[],"order":[],"savings":null});
        let first_enabled = super::super::rand_id() % 2 == 0;
        for enabled in [first_enabled, !first_enabled] {
            if cancel.load(Ordering::Relaxed) {
                plan.record["status"] = json!("stopped");
                break;
            }
            let name = if enabled { "enabled" } else { "disabled" };
            let root = plan
                .path
                .join(format!("arm-{:016x}", super::super::rand_id()));
            pair["order"].as_array_mut().unwrap().push(json!(name));
            pair["arms"].as_array_mut().unwrap().push(json!({"arm":name,"status":"running","correct":null,"wall_seconds":null,"input_tokens":null,"output_tokens":null,"total_tokens":null,"workspace":root,"transcript":root.join("sessions/comparison.md")}));
            let mut pending = plan.record.clone();
            pending["active_pair"] = pair.clone();
            save(&plan.path.join("record.json"), &pending)?;
            let arm_started = Instant::now();
            let operation = arm(
                state,
                &plan,
                &feature,
                enabled,
                root.clone(),
                cancel.clone(),
            );
            let value = match tokio::time::timeout(
                Duration::from_secs(plan.options.timeout_seconds),
                operation,
            )
            .await
            {
                Ok(value) => value,
                Err(_) => {
                    json!({"arm":name,"status":"timeout","error":"Comparison exceeded its time budget","correct":null,"input_tokens":null,"output_tokens":null,"total_tokens":null,"wall_seconds":null,"end_to_end_seconds":arm_started.elapsed().as_secs_f64(),"workspace":root,"transcript":root.join("sessions/comparison.md")})
                }
            };
            *pair["arms"]
                .as_array_mut()
                .unwrap()
                .last_mut()
                .context("Missing pending arm")? = value;
            // Save partial evidence before dispatching another paid request.
            let mut current = plan.record.clone();
            current["active_pair"] = pair.clone();
            save(&plan.path.join("record.json"), &current)?;
        }
        plan.record["pairs"].as_array_mut().unwrap().push(pair);
        save(&plan.path.join("record.json"), &plan.record)?;
        if cancel.load(Ordering::Relaxed) {
            break;
        }
    }
    if plan.record["status"] == "running" {
        plan.record["status"] = json!("completed");
    }
    save(&plan.path.join("record.json"), &plan.record)
}

</eval:copy>

Compute disabled-minus-enabled savings only for completed, model-matched arms separately reviewed as correct. Preserve null counters and negative savings.

<eval:copy id="feature-experiments-experiments-8">
fn savings(pair: &Value) -> Value {
    let arms = pair["arms"].as_array().cloned().unwrap_or_default();
    let enabled = arms.iter().find(|a| a["arm"] == "enabled");
    let disabled = arms.iter().find(|a| a["arm"] == "disabled");
    let (Some(a), Some(b)) = (enabled, disabled) else {
        return Value::Null;
    };
    if a["correct"] != true
        || b["correct"] != true
        || a["status"] != "completed"
        || b["status"] != "completed"
        || a["model"] != b["model"]
        || a["model"] == "agent default"
    {
        return Value::Null;
    }
    let mut result = json!({});
    for field in [
        "wall_seconds",
        "input_tokens",
        "output_tokens",
        "total_tokens",
    ] {
        result[field] = match (a[field].as_f64(), b[field].as_f64()) {
            (Some(a), Some(b)) => json!(b - a),
            _ => Value::Null,
        };
    }
    result
}

</eval:copy>

Export raw records with append-only human reviews layered on top. Inventory every canonical contract as unmeasured: a completed ablation does not establish the whole feature contract.

<eval:copy id="feature-experiments-experiments-9">
pub async fn list(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root = directory(&state).map_err(super::error)?;
    let mut records = Vec::new();
    if root.is_dir() {
        for path in std::fs::read_dir(&root)
            .map_err(|e| super::error(e.into()))?
            .flatten()
            .map(|e| e.path())
        {
            if let Ok(raw) = std::fs::read(path.join("record.json"))
                && let Ok(mut record) = serde_json::from_slice::<Value>(&raw)
            {
                let reviews = std::fs::read_dir(path.join("reviews"))
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter_map(|entry| std::fs::read(entry.path()).ok())
                    .filter_map(|raw| serde_json::from_slice::<Value>(&raw).ok())
                    .collect::<Vec<_>>();
                if let Some(active) = record.get_mut("active_pair") {
                    let partial = active.take();
                    if partial.is_object() {
                        record["pairs"]
                            .as_array_mut()
                            .context("Missing comparison pairs")
                            .map_err(super::error)?
                            .push(partial);
                    }
                }
                for pair in record["pairs"].as_array_mut().into_iter().flatten() {
                    let feature = pair["feature"].clone();
                    for arm in pair["arms"].as_array_mut().into_iter().flatten() {
                        let mut matching = reviews
                            .iter()
                            .filter(|r| r["feature"] == feature && r["arm"] == arm["arm"])
                            .collect::<Vec<_>>();
                        matching.sort_by_key(|r| {
                            (
                                r["created_at"].as_str().unwrap_or_default(),
                                r["recorded_ns"].as_str().unwrap_or_default(),
                            )
                        });
                        if let Some(review) = matching.last() {
                            arm["correct"] = review["correct"].clone();
                            arm["review"] = (*review).clone();
                        }
                    }
                    pair["savings"] = savings(pair);
                }
                records.push(record);
            }
        }
    }
    records.sort_by_key(|r| {
        std::cmp::Reverse(r["created_at"].as_str().unwrap_or_default().to_owned())
    });
    let features = FEATURES.iter().map(|(id, title)| {
        let pairs = records.iter().flat_map(|r| r["pairs"].as_array().into_iter().flatten()).filter(|p| p["feature"] == *id).collect::<Vec<_>>();
        let summary = |field: &str| { let mut values = pairs.iter().filter_map(|p| p["savings"][field].as_f64()).collect::<Vec<_>>(); values.sort_by(f64::total_cmp); let n = values.len(); json!({"n":n,"median":if n == 0 { None } else { Some((values[(n-1)/2] + values[n/2])/2.0) }}) };
        json!({"id":id,"title":title,"pairs":pairs.len(),"wall_seconds_saved":summary("wall_seconds"),"tokens_saved":summary("total_tokens"),"scope":"Descriptive across recorded tasks; heterogeneous tasks and model settings are not controlled; no population estimate"})
    }).collect::<Vec<_>>();
    // Every canonical contract remains in the denominator; no inferred performance claims.
    let mut coverage = Vec::new();
    let contracts = state.index.root().join(".hickory-source/guarantees");
    if contracts.is_dir() {
        for entry in ignore::WalkBuilder::new(&contracts)
            .hidden(false)
            .git_ignore(false)
            .build()
            .flatten()
        {
            if entry.file_name() == "contract.md"
                && let Ok(bytes) = std::fs::read(entry.path())
            {
                let relative = entry
                    .path()
                    .strip_prefix(state.index.root())
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .replace('\\', "/");
                coverage.push(json!({"contract":relative,"sha256":sha(&bytes),"status":if records.iter().flat_map(|r| r["pairs"].as_array().into_iter().flatten()).flat_map(|p| p["arms"].as_array().into_iter().flatten()).any(|arm| arm["review"]["contract"] == relative && arm["review"]["contract_sha256"] == sha(&bytes)) { "reviewed_observation" } else { "unmeasured" },"note":"Feature-specific witnesses must establish this contract; ablation completion alone does not."}));
            }
        }
    }
    Ok(Json(
        json!({"version":1,"records":records,"features":features,"coverage":coverage,"storage":root,"limitations":LIMITATIONS}),
    ))
}

</eval:copy>

Require a completed arm and concrete correctness evidence before appending a review. Keep the original measurements and earlier reviews intact.

<eval:copy id="feature-experiments-experiments-10">
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub feature: String,
    pub arm: String,
    pub correct: bool,
    pub notes: String,
    #[serde(default)]
    pub contract: Option<String>,
}
pub async fn review(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Review>,
) -> ApiResult<Json<Value>> {
    valid_id(&id).map_err(super::error)?;
    if body.notes.trim().is_empty() || body.notes.len() > 8192 {
        return Err(ApiError::bad_request(
            "Record correctness evidence in 1–8192 bytes",
        ));
    }
    let root = directory(&state).map_err(super::error)?.join(&id);
    let record: Value = serde_json::from_slice(
        &std::fs::read(root.join("record.json"))
            .map_err(|_| ApiError::not_found("Comparison not found"))?,
    )
    .map_err(|e| super::error(e.into()))?;
    let arm = record["pairs"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(record.get("active_pair"))
        .find(|p| p["feature"] == body.feature)
        .and_then(|p| p["arms"].as_array())
        .and_then(|arms| arms.iter().find(|a| a["arm"] == body.arm));
    if arm.is_none_or(|a| a["status"] != "completed") {
        return Err(ApiError::conflict(
            "Only a completed comparison arm can be reviewed",
        ));
    }
    let contract_hash = if let Some(contract) = &body.contract {
        let path = std::path::Path::new(contract);
        if !contract.starts_with(".hickory-source/guarantees/")
            || !contract.ends_with("/contract.md")
            || path
                .components()
                .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(ApiError::bad_request(
                "Choose a current contract from the coverage inventory",
            ));
        }
        let path = state
            .index
            .root()
            .join(path)
            .canonicalize()
            .map_err(|_| ApiError::not_found("Contract not found"))?;
        if !path.starts_with(state.index.root()) {
            return Err(ApiError::bad_request("Contract escapes the project"));
        }
        Some(
            record["source_manifest"][contract]["sha256"]
                .as_str()
                .ok_or_else(|| {
                    ApiError::conflict(
                        "This contract was not part of the frozen task; leave attribution empty",
                    )
                })?
                .to_string(),
        )
    } else {
        None
    };
    let evidence = json!({"contract":body.contract,"contract_sha256":contract_hash,"feature":body.feature,"arm":body.arm,"correct":body.correct,"notes":body.notes,"created_at":super::super::now_rfc3339(),"kind":"human-declared","recorded_ns":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos().to_string(),"snapshot_sha256":record["snapshot_sha256"]});
    save(
        &root
            .join("reviews")
            .join(format!("{:016x}.json", super::super::rand_id())),
        &evidence,
    )
    .map_err(super::error)?;
    list(State(state)).await
}

</eval:copy>

Focused witnesses challenge feature isolation, null usage, incorrect answers, negative savings and snapshot safety without invoking a paid provider.

<eval:copy id="feature-experiments-experiments-11">
#[cfg(test)]
mod tests {
    use super::*;

    // Contract: .hickory-source/guarantees/agent/feature-experiments/contract.md
    #[test]
    fn ablations_preserve_code_and_hide_only_the_selected_feature() {
        let mut tree = ProjectTree::default();
        for name in [
            "app.rs",
            "project.md",
            ".hickory-source/task.md",
            ".hickory-indexes/view.md",
            "README.md",
        ] {
            tree.insert(
                name,
                ProjectEntry::file(format!("contents of {name}").into_bytes()),
            )
            .unwrap();
        }
        let baseline = arm_tree(&tree, "literate-indexes", false).unwrap();
        assert_eq!(
            baseline
                .entries()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["README.md", "app.rs"]
        );
        assert_eq!(
            baseline.entries()["app.rs"].bytes,
            tree.entries()["app.rs"].bytes
        );
        assert_eq!(arm_tree(&tree, "lineage-tools", false).unwrap(), tree);
        assert_eq!(arm_tree(&tree, "literate-indexes", true).unwrap(), tree);
        assert!(disabled_tool("lineage-tools", "read_output"));
        assert!(!disabled_tool("lineage-tools", "read_doc"));
    }

    #[test]
    fn last_call_counts_and_context_occupancy_never_become_prompt_savings() {
        let response = json!({"usage":{"inputTokens":100,"outputTokens":20,"totalTokens":150,"cachedReadTokens":30},"used":9999});
        let older = whole_prompt_usage(
            &json!({"name":"@agentclientprotocol/codex-acp","version":"2.0.1"}),
            &response,
        );
        assert!(older["total_tokens"].is_null());
        let audited = whole_prompt_usage(
            &json!({"name":"@agentclientprotocol/codex-acp","version":"2.5.0"}),
            &response,
        );
        assert_eq!(audited["total_tokens"], 150);
        assert_eq!(audited["cached_read_tokens"], 30);
        let occupancy_only = whole_prompt_usage(
            &json!({"name":"@agentclientprotocol/codex-acp","version":"2.5.0"}),
            &json!({"used":9999}),
        );
        assert!(occupancy_only["total_tokens"].is_null());
    }

    #[test]
    fn savings_require_separate_correct_results_and_keep_regressions_and_nulls() {
        let mut pair = json!({"arms":[{"arm":"enabled","status":"completed","model":"same","correct":null,"wall_seconds":4.0,"input_tokens":120},{"arm":"disabled","status":"completed","model":"same","correct":true,"wall_seconds":3.0,"input_tokens":100}]});
        assert!(savings(&pair).is_null());
        pair["arms"][0]["correct"] = json!(true);
        let result = savings(&pair);
        assert_eq!(result["wall_seconds"], -1.0);
        assert_eq!(result["input_tokens"], -20.0);
        assert!(result["output_tokens"].is_null());
        pair["arms"][0]["model"] = json!("different");
        assert!(savings(&pair).is_null());
        pair["arms"][0]["model"] = json!("same");
        pair["arms"][1]["status"] = json!("error");
        assert!(savings(&pair).is_null());
    }

    #[test]
    fn frozen_overlays_do_not_save_primary_files_or_copy_symlinks_and_secrets() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("app.rs"), "disk").unwrap();
        std::fs::write(dir.path().join(".env"), "SECRET=private").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(dir.path().join("app.rs"), dir.path().join("linked.rs"))
            .unwrap();
        let (tree, omitted) = capture(
            dir.path(),
            BTreeMap::from([("app.rs".into(), "unsaved".into())]),
        )
        .unwrap();
        assert_eq!(&*tree.entries()["app.rs"].bytes, b"unsaved");
        assert_eq!(
            std::fs::read_to_string(dir.path().join("app.rs")).unwrap(),
            "disk"
        );
        assert!(!tree.entries().contains_key(".env"));
        assert!(omitted.iter().any(|p| p == ".env"));
        #[cfg(unix)]
        assert!(!tree.entries().contains_key("linked.rs"));
        assert!(
            capture(
                dir.path(),
                BTreeMap::from([("../outside".into(), "refuse".into())])
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn stop_before_comparisons_retains_incomplete_evidence_without_spawning() {
        let dir = tempfile::tempdir().unwrap();
        let prepared = super::super::super::prepare(super::super::super::ServeOptions {
            target: dir.path().to_path_buf(),
            port: 0,
            params: vec![],
            executor: crate::ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        })
        .await
        .unwrap();
        let path = dir.path().join("evidence");
        let command = AgentCommand {
            id: "codex".into(),
            name: "must not start".into(),
            command: "nonexistent-agent-for-stop-witness".into(),
            args: vec![],
            workspace_filesystem: false,
            private_prediction_args: None,
            private_prediction_auth_method: None,
        };
        let plan = Plan {
            path: path.clone(),
            record: json!({"status":"queued","pairs":[]}),
            tree: ProjectTree::default(),
            command,
            config: json!({}),
            options: Options {
                features: vec!["lineage-tools".into()],
                timeout_seconds: 10,
            },
            context: json!({"buffers":[]}),
        };
        execute(&prepared.state, plan, Arc::new(AtomicBool::new(true)))
            .await
            .unwrap();
        let record: Value =
            serde_json::from_slice(&std::fs::read(path.join("record.json")).unwrap()).unwrap();
        assert_eq!(record["status"], "stopped");
        assert_eq!(record["pairs"][0]["arms"], json!([]));
        assert!(!path.join("lineage-tools-enabled").exists());
        assert!(!path.join("lineage-tools-disabled").exists());
    }

    #[test]
    fn invalid_options_refuse_extra_requests() {
        for features in [
            vec![],
            vec!["unknown".into()],
            vec!["lineage-tools".into(), "lineage-tools".into()],
        ] {
            assert!(
                Options {
                    features,
                    timeout_seconds: 300
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            Options {
                features: vec!["lineage-tools".into()],
                timeout_seconds: 300
            }
            .validate()
            .is_ok()
        );
        assert!(
            Options {
                features: vec!["lineage-tools".into()],
                timeout_seconds: 0
            }
            .validate()
            .is_err()
        );
        assert!(valid_id("../escape").is_err());
    }
}
</eval:copy>

<eval:file path="crates/hickory-cli/src/serve/acp/experiments.rs"><eval:paste select="#feature-experiments-experiments-0"/><eval:paste select="#feature-experiments-experiments-1"/><eval:paste select="#feature-experiments-experiments-2"/><eval:paste select="#feature-experiments-experiments-3"/><eval:paste select="#feature-experiments-experiments-4"/><eval:paste select="#feature-experiments-experiments-5"/><eval:paste select="#feature-experiments-experiments-6"/><eval:paste select="#feature-experiments-experiments-7"/><eval:paste select="#feature-experiments-experiments-8"/><eval:paste select="#feature-experiments-experiments-9"/><eval:paste select="#feature-experiments-experiments-10"/><eval:paste select="#feature-experiments-experiments-11"/></eval:file>

## Construct `apps/web/src/components/FeatureExperiments.tsx`

Keep nullable usage and correctness explicit in the UI; an absent usage report never renders as zero savings.

<eval:copy id="feature-experiments-FeatureExperiments-0">
import { useEffect, useState } from "react";
import { request } from "../api/client";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";

type Arm = { arm: string; status: string; model?: string; answer?: string; error?: string; wall_seconds: number | null; input_tokens: number | null; output_tokens: number | null; total_tokens: number | null; usage_scope?: string; correct: boolean | null; workspace?: string; transcript?: string; review?: { notes: string } | null };
type Pair = { feature: string; arms: Arm[]; savings: Record<string, number | null> | null };
type TrialRecord = { id: string; document: string; prompt: string; status: string; created_at: string; pairs: Pair[]; active_pair?: Pair };
type Dataset = { records: TrialRecord[]; features: { id: string; title: string; pairs: number; wall_seconds_saved?: { n: number; median: number | null }; tokens_saved?: { n: number; median: number | null } }[]; coverage: { contract: string; status: string }[]; storage: string; limitations: string };
const choices = [{ id: "literate-indexes", title: "Literate source indexes" }, { id: "lineage-tools", title: "Lineage tools for generated code" }];
const number = (value: number | null | undefined) => value == null ? "unknown" : value.toLocaleString(undefined, { maximumFractionDigits: 2 });

</eval:copy>

Offer human-only opt-in beside the composer and pass selected features with Send. Show the extra-run count and time limit before dispatch. The same owned UI commands drive mouse and agent surfaces, but agents cannot authorize extra spending or certify their own results. Retained answers, workspace paths, evidence notes and export make results reviewable.

<eval:copy id="feature-experiments-FeatureExperiments-1">
/** The checkbox authorizes extra ACP requests, so only a human can turn it on. */
export function FeatureExperiments({ doc, backend, running, features, activeFeatures = [], onFeatures, onUpdate }: { doc: string; backend: string; running: boolean; features: string[]; activeFeatures?: string[]; onFeatures(value: string[]): void; onUpdate?: () => Promise<void> }) {
  const [open, setOpen] = useState(false);
  const [data, setData] = useState<Dataset | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [notes, setNotes] = useState("");
  const [contract, setContract] = useState("");
  const [busy, setBusy] = useState(false);
  const parent = `feature-experiments:${doc}`;
  const revision = useUiRevision(() => [backend, running, features, open, data, notes, contract, busy]);
  useUiReading("Feature comparison dataset", { document: doc, opted_in: backend === "codex" && features.length > 0, features, current_request_features: activeFeatures, open, data, error });
  useEffect(() => {
    if (!open) return;
    let active = true;
    const read = async () => { try { const next = await request<Dataset>("GET", "/api/feature-experiments"); if (active) { setData(next); setError(""); } } catch (reason) { if (active) setError(String(reason)); } };
    void read();
    const timer = running ? setInterval(() => void read(), 5000) : undefined;
    return () => { active = false; if (timer) clearInterval(timer); };
  }, [open, running, doc]);
  const unavailable = (reason: string): UiOutcome => ({ status: "unavailable", reason });
  const invoke = useUiCommands([
    { key: `${parent}:measure`, command: { label: "Measure this request with and without selected features", parent, subject: doc, parameters: { enabled: { type: "boolean" } }, prerequisites: ["Human opt-in authorizes two extra Codex ACP runs per selected feature"], read: () => ({ revision: revision(), enabled: !busy && !running && backend === "codex", value: features.length > 0 }), execute: async (args, actor) => {
      if (actor.kind !== "human") return { status: "requires_input", request: parent, reason: "A human must authorize additional agent runs" };
      onFeatures(args.enabled ? ["literate-indexes"] : []); return { status: "completed", value: { enabled: args.enabled } };
    } } },
    { key: `${parent}:feature`, command: { label: "Choose a feature to measure", parent, subject: doc, parameters: { id: { type: "string", maxLength: 128, choices: choices.map(c => c.id) }, enabled: { type: "boolean" } }, prerequisites: ["Human opt-in; one feature is changed per pair"], read: () => ({ revision: revision(), enabled: !busy && !running && backend === "codex" && features.length > 0, value: features }), execute: async (args, actor) => {
      if (actor.kind !== "human") return { status: "requires_input", request: parent, reason: "A human must choose extra comparison runs" };
      const id = args.id as string;
      onFeatures(args.enabled ? [...new Set([...features, id])] : features.filter(f => f !== id)); return { status: "completed", value: { selected: id, enabled: args.enabled } };
    } } },
    { key: `${parent}:open`, command: { label: "Show feature measurement dataset", parent, subject: doc, parameters: { open: { type: "boolean" } }, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: open }), execute: async args => { setOpen(args.open as boolean); return { status: "completed", value: { open: args.open } }; } } },
    { key: `${parent}:export`, command: { label:"Export feature measurement dataset", parent, subject:doc, parameters:{}, prerequisites:[], read:() => ({ revision:revision(), enabled:open && !busy }), execute:async (_args, _actor, signal): Promise<UiOutcome> => {
      setBusy(true); setError("");
      try {
        const latest = await request<Dataset>("GET", "/api/feature-experiments");
        setData(latest);
        const text = JSON.stringify(latest, null, 2);
        if (signal.aborted) return { status:"completed", value:{ browser_download_requested:false } };
        const url = URL.createObjectURL(new Blob([text], { type:"application/json" }));
        const revoke = URL.revokeObjectURL.bind(URL);
        try { const link = document.createElement("a"); link.href = url; link.download = "hickory-feature-measurements.json"; link.click(); }
        finally { setTimeout(() => revoke(url), 1000); }
        setNotice("Dataset download requested. Source snapshots and working copies remain in the listed local storage directory.");
        return { status:"completed", value:{ browser_download_requested:true, file_saved:"Browser-owned; not observed", bytes:new TextEncoder().encode(text).length } };
      } catch (reason) { setError(String(reason)); return unavailable(String(reason)); }
      finally { setBusy(false); }
    } } },
    { key: `${parent}:update`, command: { label: "Update Codex adapter for measured token totals", parent, subject: doc, parameters: {}, prerequisites: ["Downloads the audited adapter through Hickory's existing installer"], read: () => ({ revision: revision(), enabled: !!onUpdate && backend === "codex" && !running && !busy }), execute: async (_args, actor) => {
      if (actor.kind !== "human") return { status:"requires_input", request:parent, reason:"A human chooses whether to update the installed adapter" };
      if (!onUpdate) return unavailable("Adapter installation is unavailable in this view");
      setBusy(true); setError(""); setNotice("");
      try { await onUpdate(); setNotice("Codex adapter updated. New comparison sessions will use it."); return { status:"completed", value:{ adapter_updated:true } }; }
      catch (reason) { setError(String(reason)); return unavailable(String(reason)); } finally { setBusy(false); }
    } } },
    { key: `${parent}:notes`, command: { label: "Draft feature correctness evidence", parent, subject: doc, parameters: { notes: { type: "string", maxLength: 8192 }, contract: { type: "string", maxLength: 4096 } }, prerequisites: [], read: () => ({ revision: revision(), enabled: !busy, value: notes }), execute: async args => { setNotes(args.notes as string); setContract(args.contract as string); return { status: "completed", value: { drafted: true, reviewed: false } }; } } },
    { key: `${parent}:review`, command: { label: "Record correctness of a comparison result", parent, subject: doc, parameters: { id: { type: "string", maxLength: 16 }, feature: { type: "string", maxLength: 128, choices: choices.map(c => c.id) }, arm: { type: "string", maxLength: 128, choices: ["enabled", "disabled"] }, correct: { type: "boolean" } }, prerequisites: ["Inspect the retained working copy and tests; human judgment is separate from completion"], read: () => ({ revision: revision(), enabled: !busy && !!notes.trim() }), execute: async (args, actor) => {
      if (actor.kind !== "human") return { status: "requires_input", request: parent, reason: "An agent cannot certify its own comparison outcome" };
      if (!notes.trim()) return unavailable("Describe the correctness evidence first");
      setBusy(true);
      try { setData(await request<Dataset>("POST", `/api/feature-experiments/${args.id}/review`, { feature: args.feature, arm: args.arm, correct: args.correct, notes, ...(contract ? { contract } : {}) })); setNotes(""); return { status: "completed", value: { review_recorded: true, kind: "human-declared" } }; }
      catch (reason) { setError(String(reason)); return unavailable(String(reason)); } finally { setBusy(false); }
    } } },
  ]);
  const act = (key: string, args: Parameters<typeof invoke>[1]) => { void invoke(`${parent}:${key}`, args).catch(reason => setError(String(reason))); };
  const records = data?.records.filter(record => record.document === doc) ?? [];
  return <section className="feature-experiments" aria-label="Feature performance measurements">
    <label><input type="checkbox" checked={backend === "codex" && features.length > 0} disabled={running || busy || backend !== "codex"} onChange={event => act("measure", { enabled: event.target.checked })} /> Measure this request with and without Hickory features</label>
    {running && activeFeatures.length > 0 && <p role="status">This request is measuring {activeFeatures.join(", ")}. The next request starts with measurement off.</p>}
    {backend !== "codex" && <small>Choose Codex ACP to collect comparisons.</small>}
    {backend === "codex" && features.length > 0 && <div>
      {choices.map(choice => <label key={choice.id}><input type="checkbox" checked={features.includes(choice.id)} disabled={running || busy} onChange={event => act("feature", { id: choice.id, enabled: event.target.checked })} />{choice.title}</label>)}
      <p>{2 * features.length} extra Codex runs after your normal request, up to five minutes each. Fresh working copies keep trial edits separate; they use the adapter’s normal host permissions. Comparisons start fresh, without conversation history.</p>
    </div>}
    <button type="button" className="btn-link" aria-expanded={open} disabled={busy} onClick={() => act("open", { open: !open })}>{open ? "Hide measurements" : "Measurements and dataset"}</button>
    {error && <p role="alert">{error}</p>}
    {notice && <p role="status">{notice}</p>}
    {open && <div>
      <p>Results accumulate locally. Completion is separate from correctness; savings appear only after both results are reviewed as correct. Negative savings mean the feature cost more.</p>
      {onUpdate && backend === "codex" && <button disabled={running || busy} onClick={() => act("update", {})}>Update Codex adapter for token totals</button>}
      <button disabled={busy} onClick={() => act("export", {})}>Export dataset</button>
      <p>{data?.coverage.length ?? 0} contracts inventoried; feature-specific correctness and performance evidence still needs to be collected. Storage: {data?.storage ?? "loading"}</p>
      <table><thead><tr><th>Feature</th><th>Pairs</th><th>Median prompt seconds saved</th><th>Median tokens saved</th></tr></thead><tbody>{data?.features.map(feature => <tr key={feature.id}><td>{feature.title}</td><td>{feature.pairs}</td><td>{number(feature.wall_seconds_saved?.median)} (n={feature.wall_seconds_saved?.n ?? 0})</td><td>{number(feature.tokens_saved?.median)} (n={feature.tokens_saved?.n ?? 0})</td></tr>)}</tbody></table>
      <p>Medians describe reviewed pairs in this project across recorded tasks; they are not population estimates.</p>
      {!records.length && <p>No comparisons recorded for this conversation yet.</p>}
      {records.map(record => <article key={record.id}>
        <h4>{record.prompt.slice(0, 160)}</h4><p>Last recorded status: {record.status} · {record.created_at}</p>
        {[...record.pairs, ...(record.active_pair ? [record.active_pair] : [])].map((pair, i) => <div key={`${pair.feature}-${i}`}>
          <strong>{pair.feature}</strong>
          {pair.arms.map(arm => <details key={arm.arm}><summary>{arm.arm} · {arm.status} · {number(arm.wall_seconds)} seconds · {number(arm.total_tokens)} tokens · correctness {arm.correct === null ? "unreviewed" : arm.correct ? "accepted" : "rejected"}</summary>
            <pre>{arm.answer ?? arm.error ?? "No answer recorded"}</pre><p>{arm.usage_scope}</p><p>Working copy: {arm.workspace} · Record: {arm.transcript}</p>
            {arm.review?.notes && <p>Evidence: {arm.review.notes}</p>}
            {arm.status === "completed" && <><button disabled={busy || !notes.trim()} onClick={() => act("review", { id: record.id, feature: pair.feature, arm: arm.arm, correct: true })}>Correct</button><button disabled={busy || !notes.trim()} onClick={() => act("review", { id: record.id, feature: pair.feature, arm: arm.arm, correct: false })}>Incorrect</button></>}
          </details>)}
          {pair.savings && <p>Saved {number(pair.savings.wall_seconds)} seconds · {number(pair.savings.total_tokens)} tokens (disabled minus enabled).</p>}
        </div>)}
      </article>)}
      <label>Correctness evidence<textarea disabled={busy} value={notes} maxLength={8192} onChange={event => act("notes", { notes: event.target.value, contract })} placeholder="Tests or observations supporting the result you are about to review" /></label>
      <label>Feature contract checked (optional)<select disabled={busy} value={contract} onChange={event => act("notes", { notes, contract: event.target.value })}><option value="">No contract attribution</option>{data?.coverage.map(item => <option key={item.contract} value={item.contract}>{item.contract} · {item.status}</option>)}</select></label>
      <p>{data?.limitations}</p>
    </div>}
  </section>;
}
</eval:copy>

<eval:file path="apps/web/src/components/FeatureExperiments.tsx"><eval:paste select="#feature-experiments-FeatureExperiments-0"/><eval:paste select="#feature-experiments-FeatureExperiments-1"/></eval:file>

## Construct `crates/hickory-cli/tests/feature_experiments.rs`

Drive the real HTTP Send route and the ordinary ACP client with a deterministic protocol peer. Check that all four arms see the pre-primary source, run in different directories, enforce feature removal, retain records and require reviews before savings. This checks instrumentation, not model performance.

<eval:copy id="feature-experiments-feature_experiments-0">
//! Contract: .hickory-source/guarantees/agent/feature-experiments/contract.md
//! A deterministic ACP peer checks plumbing and isolation, never model benefit.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration;

struct TestEvidence(std::path::PathBuf);
impl Drop for TestEvidence {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
async fn normal_codex_request_collects_isolated_ablations_and_reviews_without_inventing_savings() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::write(root.join("note.md"), "# Implement a request\n").unwrap();
    std::fs::write(root.join("project.md"), "# An index for the same source\n").unwrap();
    std::fs::write(root.join("app.txt"), "before\n").unwrap();
    let contract_path = ".hickory-source/guarantees/demo/contract.md";
    std::fs::create_dir_all(root.join(".hickory-source/guarantees/demo")).unwrap();
    std::fs::write(root.join(contract_path), "# A task contract\n").unwrap();
    let prepared = prepare(ServeOptions {
        target: root.join("note.md"),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: Some(root.join("ui.json")),
    })
    .await
    .unwrap();
    let doc = prepared
        .state
        .index
        .entries()
        .into_iter()
        .find(|(_, path)| path == "note.md")
        .unwrap()
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let http = reqwest::Client::new();
    let command = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!(
            "examples/acp_fixture{}",
            std::env::consts::EXE_SUFFIX
        ));
    assert!(
        command.is_file(),
        "Build the acp_fixture example for this focused test"
    );
    let response = http
        .put(format!("{base}/api/agents"))
        .json(&json!([{"id":"codex","name":"Codex protocol fixture","command":command,"args":[]}]))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    let response: Value = http
        .post(format!("{base}/api/docs/{doc}/agent/acp"))
        .json(&json!({"backend":"codex"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["ready"], true, "{response}");
    let response: Value = http.post(format!("{base}/api/docs/{doc}/agent")).json(&json!({"backend":"codex","prompt":"measure-request","experiment":{"features":["literate-indexes","lineage-tools"],"timeout_seconds":30}})).send().await.unwrap().json().await.unwrap();
    let turn = response["session_id"].as_str().unwrap();
    tokio::time::timeout(Duration::from_secs(45), async {
        loop {
            let response: Value = http
                .get(format!("{base}/api/docs/{doc}/agent/turns"))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if let Some(result) = response["turns"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == turn && t["status"] != "running")
            {
                assert_eq!(result["status"], "ok", "{result}");
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("app.txt")).unwrap(),
        "primary or trial edit\n"
    );
    let data: Value = http
        .get(format!("{base}/api/feature-experiments"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let _evidence = TestEvidence(std::path::PathBuf::from(data["storage"].as_str().unwrap()));
    let record = data["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == turn)
        .unwrap();
    assert_eq!(record["pairs"].as_array().unwrap().len(), 2, "{record}");
    assert_eq!(record["primary"]["status"], "completed");
    let primary: Value =
        serde_json::from_str(record["primary"]["answer"].as_str().unwrap()).unwrap();
    assert_eq!(primary["has_index"], true);
    assert_eq!(primary["has_lineage"], true);
    assert_eq!(primary["starting_source"], "before\n");
    assert_eq!(
        record["measurement_implementation_sha256"],
        hex::encode(Sha256::digest(include_bytes!(
            "../src/serve/acp/experiments.rs"
        )))
    );
    let mut trial_paths = std::collections::HashSet::new();
    for pair in record["pairs"].as_array().unwrap() {
        assert!(pair["savings"].is_null());
        for arm in pair["arms"].as_array().unwrap() {
            assert_eq!(arm["status"], "completed", "{arm}");
            assert!(arm["correct"].is_null());
            assert!(
                arm["end_to_end_seconds"].as_f64().unwrap()
                    >= arm["wall_seconds"].as_f64().unwrap()
            );
            assert!(arm["setup_seconds"].as_f64().unwrap() >= 0.0);
            assert!(
                arm["total_tokens"].is_null(),
                "Fixture reported context occupancy only"
            );
            let answer: Value = serde_json::from_str(arm["answer"].as_str().unwrap()).unwrap();
            assert_eq!(
                answer["starting_source"], "before\n",
                "Each arm must start before the primary edit"
            );
            let enabled = arm["arm"] == "enabled";
            if pair["feature"] == "literate-indexes" {
                assert_eq!(answer["has_index"], enabled);
            }
            if pair["feature"] == "lineage-tools" {
                assert_eq!(answer["has_lineage"], enabled);
                assert_eq!(answer["direct_disabled_refusal"], !enabled);
            }
            let workspace = arm["workspace"].as_str().unwrap();
            assert_ne!(std::path::Path::new(workspace), root);
            assert!(trial_paths.insert(workspace.to_string()));
            assert!(std::path::Path::new(arm["transcript"].as_str().unwrap()).is_file());
        }
    }
    let mut last = Value::Null;
    for arm in ["enabled", "disabled"] {
        last = http.post(format!("{base}/api/feature-experiments/{turn}/review")).json(&json!({"feature":"literate-indexes","arm":arm,"correct":true,"contract":contract_path,"notes":"Protocol fixture observed the frozen starting bytes and isolated writes; not a model correctness claim."})).send().await.unwrap().json().await.unwrap();
    }
    let pair = last["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == turn)
        .unwrap()["pairs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["feature"] == "literate-indexes")
        .unwrap();
    assert!(pair["savings"]["wall_seconds"].is_number());
    assert!(pair["savings"]["total_tokens"].is_null());
    let covered = last["coverage"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["contract"] == contract_path)
        .unwrap();
    assert_eq!(covered["status"], "reviewed_observation");
    std::fs::write(root.join(contract_path), "# Changed requirement\n").unwrap();
    let stale: Value = http
        .get(format!("{base}/api/feature-experiments"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        stale["coverage"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["contract"] == contract_path)
            .unwrap()["status"],
        "unmeasured"
    );
    let corrected: Value = http.post(format!("{base}/api/feature-experiments/{turn}/review")).json(&json!({"feature":"literate-indexes","arm":"enabled","correct":false,"notes":"A later review retracts the earlier acceptance; the original review must remain retained."})).send().await.unwrap().json().await.unwrap();
    let corrected_pair = corrected["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == turn)
        .unwrap()["pairs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["feature"] == "literate-indexes")
        .unwrap();
    assert!(corrected_pair["savings"].is_null());
    let storage = std::path::PathBuf::from(data["storage"].as_str().unwrap());
    assert_eq!(
        std::fs::read_dir(storage.join(turn).join("reviews"))
            .unwrap()
            .count(),
        3
    );
    server.abort();
    // The fixture's personal dataset is not user research evidence.
    std::fs::remove_dir_all(storage).unwrap();
}
</eval:copy>

<eval:file path="crates/hickory-cli/tests/feature_experiments.rs"><eval:paste select="#feature-experiments-feature_experiments-0"/></eval:file>

## Construct `apps/web/src/components/FeatureExperiments.test.tsx`

Check that measurement starts unchecked, requires Codex, states extra-run costs, and does not turn missing tokens or unreviewed results into savings.

<eval:copy id="feature-experiments-FeatureExperiments-test-0">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useState } from "react";
import { FeatureExperiments } from "./FeatureExperiments";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { request } from "../api/client";
vi.mock("../api/client", () => ({ request: vi.fn() }));

afterEach(() => { cleanup(); vi.clearAllMocks(); });

function Composer({ backend = "codex", running = false }) {
  const [features, setFeatures] = useState<string[]>([]);
  return <FeatureExperiments doc="workspace" backend={backend} running={running} features={features} onFeatures={setFeatures} />;
}

describe("in-app feature measurement consent and evidence", () => {
  it("exports fresh recorded outcomes through the application transport", async () => {
    const initial = { records:[], features:[], coverage:[], storage:"local", limitations:"Recorded observations" };
    const latest = { ...initial, records:[{ id:"failed", document:"elsewhere", prompt:"Task", status:"error", created_at:"today", pairs:[] }] };
    vi.mocked(request).mockResolvedValueOnce(initial).mockResolvedValueOnce(latest);
    const create = vi.fn((_blob: Blob) => "blob:feature-dataset"), revoke = vi.fn();
    const OriginalURL = URL;
    vi.stubGlobal("URL", Object.assign(class extends OriginalURL {}, { createObjectURL:create, revokeObjectURL:revoke }));
    const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});
    try {
      render(<Composer />);
      await act(async () => fireEvent.click(screen.getByRole("button", { name:"Measurements and dataset" })));
      await waitFor(() => expect(request).toHaveBeenCalledTimes(1));
      await act(async () => fireEvent.click(screen.getByRole("button", { name:"Export dataset" })));
      expect(request).toHaveBeenCalledTimes(2);
      expect(click).toHaveBeenCalledOnce();
      const blob = create.mock.calls[0][0] as Blob;
      const text = await new Promise<string>((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(String(reader.result)); reader.onerror = reject; reader.readAsText(blob); });
      expect(JSON.parse(text)).toEqual(latest);
    } finally { click.mockRestore(); vi.unstubAllGlobals(); }
  });

  // Contract: .hickory-source/guarantees/agent/feature-experiments/contract.md
  it("starts unchecked, exposes extra run count, and lets the human select an ablation", async () => {
    render(<Composer />);
    const checkbox = screen.getByRole("checkbox", { name: "Measure this request with and without Hickory features" }) as HTMLInputElement;
    expect(checkbox.checked).toBe(false);
    await act(async () => fireEvent.click(checkbox));
    expect(checkbox.checked).toBe(true);
    expect(screen.getByText(/2 extra Codex runs/)).toBeTruthy();
    await act(async () => fireEvent.click(screen.getByRole("checkbox", { name: "Lineage tools for generated code" })));
    expect(screen.getByText(/4 extra Codex runs/)).toBeTruthy();
    expect(request).not.toHaveBeenCalled();
  });

  it("requires a human to authorize extra runs through the agent UI bridge", async () => {
    const registry = new UiActions({ workspace:"p", window:"w" }, () => true, async () => {});
    render(<UiActionContext.Provider value={registry}><Composer /></UiActionContext.Provider>);
    const action = registry.groups().groups.flatMap(group => registry.observe(group,64).actions).find(item => item.label === "Measure this request with and without selected features")!;
    let outcome: unknown;
    await act(async () => { outcome = await registry.act(registry.scope, { id:"codex:fixture", kind:"agent" }, { handle:action.handle, arguments:{ enabled:true }, request_id:crypto.randomUUID() }); });
    expect(outcome).toMatchObject({ status:"result", outcome:{ status:"requires_input" } });
    expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(false);
    expect(request).not.toHaveBeenCalled();
  });

  it("does not offer extra Codex runs through a different backend", () => {
    render(<Composer backend="python" />);
    expect((screen.getByRole("checkbox") as HTMLInputElement).disabled).toBe(true);
    expect(screen.getByText("Choose Codex ACP to collect comparisons.")).toBeTruthy();
  });

  it("keeps unreviewed correctness and unavailable usage visible", async () => {
    vi.mocked(request).mockResolvedValue({ records: [{ id:"abc", document:"workspace", prompt:"Implement quota", status:"completed", created_at:"today", pairs:[{feature:"literate-indexes", arms:[{arm:"enabled", status:"completed", correct:null, total_tokens:null, wall_seconds:1, answer:"Done"}], savings:null}] }], features:[], coverage:[{contract:"example",status:"unmeasured"}], storage:"local", limitations:"Not a proof" });
    render(<Composer />);
    await act(async () => fireEvent.click(screen.getByRole("button", { name:"Measurements and dataset" })));
    await waitFor(() => expect(screen.getByText(/unknown tokens · correctness unreviewed/)).toBeTruthy());
    expect(screen.getByRole("button", { name:"Export dataset" })).toBeTruthy();
    expect((screen.getByRole("button", { name:"Correct" }) as HTMLButtonElement).disabled).toBe(true);
  });
});
</eval:copy>

<eval:file path="apps/web/src/components/FeatureExperiments.test.tsx"><eval:paste select="#feature-experiments-FeatureExperiments-test-0"/></eval:file>

</eval:doc>
