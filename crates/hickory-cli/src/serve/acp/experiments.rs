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
