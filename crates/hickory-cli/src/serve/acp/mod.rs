//! ACP is an optional agent extension. The language and model client stay independent.
mod client;
pub(super) mod config;
pub(crate) mod edits;
mod mcp;
pub(super) mod read_receipt;
pub(crate) mod record;
pub(super) mod transport;
pub(super) mod workspace;
pub(super) use client::RoomContext;
pub use mcp::proxy_argv;
pub(super) use record::partial_answer;

use super::{
    LocalState,
    agent::{AgentRequest, TurnRecord},
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub struct Hub {
    clients: tokio::sync::Mutex<HashMap<String, Arc<client::Client>>>,
    pub selected: Mutex<HashMap<String, String>>,
    pub installing: tokio::sync::Mutex<()>,
}

fn error(e: anyhow::Error) -> ApiError {
    ApiError::unprocessable(format!("{e:#}"))
}

/// Ordinary harness filesystem/history cannot enter a private predictor context.
fn ensure_ordinary(state: &LocalState) -> anyhow::Result<()> {
    anyhow::ensure!(
        state.private_prediction.is_none(),
        "Private ACP requires an isolated harness/tool/review adapter; ordinary ACP startup is refused"
    );
    anyhow::ensure!(
        !crate::engine::private_policy::active()?,
        "Ordinary ACP startup is paused while private prediction policy is active"
    );
    Ok(())
}

pub async fn catalogue(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let dir = config::directory(&state);
    let mut agents: Vec<Value> = config::commands(&state).map_err(error)?.into_iter().map(|agent| {
        let available = config::executable(&agent.command, &dir).is_some();
        let installable = matches!(agent.id.as_str(), "codex" | "claude") && config::executable("npm", &dir).is_some();
        let cli_available = matches!(agent.id.as_str(), "codex" | "claude") && config::executable(&agent.id, &dir).is_some();
        json!({"id":agent.id,"name":agent.name,"command":agent.command,"args":agent.args,"available":available,"installable":installable,"cli_available":cli_available,"workspace_filesystem":agent.workspace_filesystem,"private_prediction_args":agent.private_prediction_args,"private_prediction_auth_method":agent.private_prediction_auth_method})
    }).collect();
    agents.push(super::python_agent::catalogue(&state));
    Ok(Json(
        json!({"agents":agents,"workspace_filesystem":super::workspace_fs::availability()}),
    ))
}

pub async fn save_commands(
    State(state): State<LocalState>,
    Json(commands): Json<Vec<config::AgentCommand>>,
) -> ApiResult<Json<Value>> {
    config::validate(&commands).map_err(error)?;
    let dir = config::directory(&state);
    std::fs::create_dir_all(&dir).map_err(|e| error(e.into()))?;
    let raw = serde_json::to_vec_pretty(&commands).map_err(|e| error(e.into()))?;
    super::store::write_atomic(&dir.join("agents.json"), &raw).map_err(error)?;
    catalogue(State(state)).await
}

pub async fn install(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let package = match id.as_str() {
        "codex" => "@agentclientprotocol/codex-acp@2.0.1",
        "claude" => "@agentclientprotocol/claude-agent-acp@0.81.2",
        _ => {
            return Err(ApiError::bad_request(
                "This agent has no install catalogue. Set its executable in Settings → Agents.",
            ));
        }
    };
    let _gate = state.agent.acp.installing.lock().await;
    let dir = config::directory(&state);
    let npm = config::executable("npm", &dir).ok_or_else(|| ApiError::unavailable("Installing an agent adapter needs Node.js and npm. Install Node.js, or configure an existing adapter in Settings → Agents."))?;
    let mut cmd = config::npm_process(&npm, &dir).map_err(error)?;
    cmd.args(["install", "--no-audit", "--no-fund", "--prefix"])
        .arg(&dir)
        .arg(package)
        .kill_on_drop(true);
    if let Some(parent) = npm.parent() {
        let inherited = std::env::var_os("PATH").unwrap_or_default();
        let paths = std::iter::once(parent.to_path_buf()).chain(std::env::split_paths(&inherited));
        cmd.env(
            "PATH",
            std::env::join_paths(paths).map_err(|e| error(e.into()))?,
        );
    }
    let output = tokio::time::timeout(Duration::from_secs(180), cmd.output())
        .await
        .map_err(|_| {
            ApiError::unavailable(
                "The adapter download timed out. Check your connection and try Install again.",
            )
        })?
        .map_err(|e| error(e.into()))?;
    if !output.status.success() {
        return Err(ApiError::unprocessable(format!(
            "Installing {id} failed. Check your Node.js installation and network, then try again."
        )));
    }
    drop(_gate);
    catalogue(State(state)).await
}

#[derive(Deserialize)]
pub struct Connect {
    pub backend: String,
    #[serde(default)]
    pub session: Option<String>,
}

pub async fn connect(
    State(state): State<LocalState>,
    Path(doc_id): Path<String>,
    Json(body): Json<Connect>,
) -> ApiResult<Json<Value>> {
    let command = config::commands(&state)
        .map_err(error)?
        .into_iter()
        .find(|c| c.id == body.backend)
        .ok_or_else(|| {
            ApiError::bad_request("Unknown agent. Choose an agent from the catalogue.")
        })?;
    let doc = super::agent_context::subject(&state, &doc_id)?;
    let session_root = super::agent_context::session_root(&state, &doc_id)?;
    let path = match body.session.as_deref() {
        Some(rel) => {
            let path = session_root.join(rel);
            let canonical = path.canonicalize().map_err(|e| error(e.into()))?;
            if !canonical.starts_with(
                session_root
                    .join("sessions")
                    .canonicalize()
                    .map_err(|e| error(e.into()))?,
            ) {
                return Err(ApiError::bad_request(
                    "Choose a conversation under this workspace's sessions folder.",
                ));
            }
            let source = std::fs::read_to_string(&canonical).map_err(|e| error(e.into()))?;
            let view = hickory_agent::session_view::session_view(&source);
            if view.doc.as_deref().map(std::path::Path::new) != Some(doc.as_path()) {
                return Err(ApiError::conflict(
                    "That conversation belongs to another document. Open its document to resume it.",
                ));
            }
            if record::metadata(&canonical).is_none_or(|m| m["agent"] != body.backend) {
                return Err(ApiError::conflict(
                    "That conversation belongs to a different agent. Start a new thread to change agents.",
                ));
            }
            canonical
        }
        None => hickory_agent::session_file_path(
            &session_root,
            &format!("{}-{:016x}", body.backend, super::rand_id()),
        ),
    };
    // Serialise connection changes per workspace, before any subprocess starts.
    let mut clients = state.agent.acp.clients.lock().await;
    let reusable = match clients.get(&doc_id) {
        Some(c) => {
            c.agent == body.backend
                && !c.closed.load(std::sync::atomic::Ordering::Relaxed)
                && ((body.session.is_none() && c.last_turn.lock().unwrap().is_none())
                    || c.record.lock().await.path == path)
        }
        None => false,
    };
    if !reusable {
        if let Some(old) = clients.get(&doc_id)
            && old.active.lock().unwrap().is_some()
        {
            return Err(ApiError::conflict(
                "Stop the running turn before switching agents or threads.",
            ));
        }
        let client = client::Client::start(&state, &doc_id, command, path)
            .await
            .map_err(error)?;
        let source = std::fs::read_to_string(&client.record.lock().await.path).unwrap_or_default();
        *client.last_turn.lock().unwrap() = hickory_agent::session_view::session_view(&source)
            .turns
            .last()
            .map(|t| t.id.clone());
        if let Some(old) = clients.insert(doc_id.clone(), client) {
            old.shutdown().await;
        }
    }
    let client = clients.get(&doc_id).expect("inserted").clone();
    drop(clients);
    state
        .agent
        .acp
        .selected
        .lock()
        .unwrap()
        .insert(doc_id.clone(), body.backend);
    // Flush unsaved document text before the harness begins reading from disk.
    if let Some(room) = state.rooms.get(&doc_id).await {
        super::store::write_atomic(&doc, room.text().await.as_bytes()).map_err(error)?;
    }
    // A pane remount may reconnect to a turn waiting for review. Returning its
    // existing connection must never wait for that same turn to finish.
    let setup_error = if client.active.lock().unwrap().is_some() {
        None
    } else {
        let _operation = client.operation.lock().await;
        client.setup().await.err().map(|e| format!("{e:#}"))
    };
    let mut view = client.snapshot();
    view["error"] = json!(setup_error);
    let record_path = client.record.lock().await.path.clone();
    view["session"] = json!(
        record_path
            .strip_prefix(state.index.root())
            .unwrap_or(&record_path)
            .display()
            .to_string()
    );
    Ok(Json(view))
}

async fn connection(state: &LocalState, doc: &str) -> ApiResult<Arc<client::Client>> {
    state
        .agent
        .acp
        .clients
        .lock()
        .await
        .get(doc)
        .cloned()
        .ok_or_else(|| ApiError::conflict("Connect an agent before using its controls."))
}

pub async fn snapshot(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(connection(&state, &doc).await?.snapshot()))
}

#[derive(Deserialize)]
pub struct Authenticate {
    pub method_id: String,
}
pub async fn authenticate(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
    Json(body): Json<Authenticate>,
) -> ApiResult<Json<Value>> {
    let client = connection(&state, &doc).await?;
    let _operation = client.operation.lock().await;
    let method = client.initialized["authMethods"]
        .as_array()
        .and_then(|a| a.iter().find(|m| m["id"] == body.method_id))
        .ok_or_else(|| ApiError::bad_request("Choose a sign-in method offered by this agent."))?;
    if method["type"].as_str().is_some_and(|t| t != "agent") {
        return Err(ApiError::unprocessable(
            "This adapter requires terminal sign-in. Sign in with its own CLI, then reconnect here.",
        ));
    }
    client
        .rpc
        .request(
            "authenticate",
            json!({"methodId":body.method_id}),
            Duration::from_secs(300),
        )
        .await
        .map_err(error)?;
    client.setup().await.map_err(error)?;
    Ok(Json(client.snapshot()))
}

#[derive(Deserialize)]
pub struct Setting {
    pub config_id: String,
    pub value: Value,
}
pub async fn configure(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
    Json(body): Json<Setting>,
) -> ApiResult<Json<Value>> {
    let client = connection(&state, &doc).await?;
    let _operation = client.operation.try_lock().map_err(|_| {
        ApiError::conflict("Wait for this turn to finish before changing its settings.")
    })?;
    let session = client
        .session
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| ApiError::conflict("Sign in before choosing an agent model."))?;
    let (method, params) = if body.config_id == "__mode" {
        (
            "session/set_mode",
            json!({"sessionId":session,"modeId":body.value}),
        )
    } else {
        (
            "session/set_config_option",
            json!({"sessionId":session,"configId":body.config_id,"value":body.value}),
        )
    };
    let result = client
        .rpc
        .request(method, params, Duration::from_secs(30))
        .await
        .map_err(error)?;
    let mut view = client.view.lock().unwrap();
    if body.config_id == "__mode" {
        view["modes"]["currentModeId"] = body.value;
    }
    if let Some(options) = result.get("configOptions") {
        view["configOptions"] = options.clone();
    }
    drop(view);
    Ok(Json(client.snapshot()))
}

#[derive(Deserialize)]
pub struct Permission {
    pub request_id: String,
    pub option_id: String,
}
pub async fn permission(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
    Json(body): Json<Permission>,
) -> ApiResult<Json<Value>> {
    let client = connection(&state, &doc).await?;
    let mut permissions = client.permissions.lock().unwrap();
    let pending = permissions.get(&body.request_id).ok_or_else(|| {
        ApiError::conflict("This permission request has already ended. Refresh the conversation.")
    })?;
    if pending.view["options"]
        .as_array()
        .is_none_or(|options| !options.iter().any(|o| o["optionId"] == body.option_id))
    {
        return Err(ApiError::bad_request(
            "Choose one of the permission options the agent offered.",
        ));
    }
    let pending = permissions.remove(&body.request_id).expect("checked");
    let _ = pending
        .reply
        .send(json!({"outcome":{"outcome":"selected","optionId":body.option_id}}));
    Ok(Json(json!({"answered":true})))
}

pub async fn start_turn(
    state: LocalState,
    doc_id: String,
    body: AgentRequest,
    backend: String,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let client = connection(&state, &doc_id).await?;
    if client.agent != backend || !client.snapshot()["ready"].as_bool().unwrap_or(false) {
        return Err(ApiError::conflict(
            "Connect and sign in to the selected agent before sending a message.",
        ));
    }
    let doc = super::agent_context::subject(&state, &doc_id)?;
    state.agent.hydrate(
        &super::agent_context::session_root(&state, &doc_id)?,
        &doc_id,
        &doc,
    );
    let prompt = body.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(ApiError::bad_request("Say what you want the agent to do."));
    }
    let record_path = client.record.lock().await.path.clone();
    let session = record_path
        .strip_prefix(state.index.root())
        .unwrap_or(&record_path)
        .display()
        .to_string();
    let turn_id = format!("{:016x}", super::rand_id());
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        let mut map = state.agent.turns.lock().unwrap();
        let turns = map.entry(doc_id.clone()).or_default();
        if turns.iter().any(|t| t.status == "running") {
            return Err(ApiError::conflict(
                "An agent turn is already running. Stop it or wait for it to finish.",
            ));
        }
        if body.parent_id != *client.last_turn.lock().unwrap() && !client.can_rewind() {
            return Err(ApiError::conflict(
                "This ACP agent continues from the latest turn. Start a new thread to branch; arbitrary rewind is not supported by this adapter.",
            ));
        }
        turns.push(TurnRecord {
            backend: backend.clone(),
            id: turn_id.clone(),
            parent_id: body.parent_id.clone(),
            prompt: prompt.clone(),
            answer: None,
            status: "running".into(),
            error: None,
            created_at: super::now_rfc3339(),
            provider: format!("acp:{backend}"),
            model: client.model(),
            usage: None,
            session,
        });
        state
            .agent
            .cancels
            .lock()
            .unwrap()
            .insert(turn_id.clone(), cancel.clone());
    }
    state
        .agent
        .acp
        .selected
        .lock()
        .unwrap()
        .insert(doc_id.clone(), backend);
    client
        .host
        .edits
        .live_context
        .store(body.context.live, std::sync::atomic::Ordering::Relaxed);
    *client.host.edits.buffers.lock().unwrap() = body.context.buffers.clone();
    *client.host.ui_window.lock().await = body.context.ui_window.clone();
    let context =
        if doc_id == super::agent_context::WORKSPACE_AGENT || !body.context.buffers.is_empty() {
            super::agent_context::describe(&state, &body.context).replace(
                "Offer suggested edits in your answer for unsaved buffers.",
                "Use read_buffer and edit_buffer for requested edits, including unsaved buffers.",
            )
        } else {
            String::new()
        };
    let context = format!(
        "{}\n{context}\nFor changes to the current open editor, call read_buffer then edit_buffer using its hashes. Do not return a replacement document as your answer. The client handles approval. For generated code use read_output and edit_output so lineage carries the edit into its source.",
        hickory_agent::audit::INSTRUCTIONS
    );
    let project_capture = super::agent_projects::begin_editors(
        &state,
        &doc_id,
        record_path.clone(),
        client.host.edits.clone(),
    )
    .await;
    if let Err(error) = &project_capture
        && client.supports_changesets()
    {
        client
            .record
            .lock()
            .await
            .context(
                "literate-project-capture-error",
                &json!({"turn":turn_id,"error":format!("{error:#}")}),
            )
            .map_err(self::error)?;
        state.agent.cancels.lock().unwrap().remove(&turn_id);
        if let Some(turn) = state
            .agent
            .turns
            .lock()
            .unwrap()
            .get_mut(&doc_id)
            .and_then(|ts| ts.iter_mut().find(|t| t.id == turn_id))
        {
            turn.status = "error".into();
            turn.error = Some(format!("Cannot freeze changeset: {error:#}"));
        }
        return Err(ApiError::unprocessable(format!(
            "Cannot freeze changeset before agent writes: {error:#}"
        )));
    }
    *client.host.changeset.lock().await = match &project_capture {
        Ok(Some(capture)) => {
            let mut contract = crate::changeset::Contract::new(
                capture.root.clone(),
                capture.inventory.clone(),
                capture
                    .starting
                    .restore()
                    .map_err(ApiError::unprocessable)?,
                turn_id.clone(),
            );
            contract.workspace = capture
                .workspace
                .clone()
                .map(|scope| (scope, client.host.edits.clone()));
            Some(contract)
        }
        _ => None,
    };
    let run_id = turn_id.clone();
    tokio::spawn(async move {
        let _annotation_gate = state.agent.annotations_gate.read().await;
        let mut outcome = client
            .prompt(
                &run_id,
                body.parent_id.as_deref(),
                &prompt,
                &context,
                cancel,
            )
            .await;
        match project_capture {
            Ok(Some(mut capture)) => {
                if outcome.is_ok()
                    && let Ok(artifact) = client.host.completion().await
                {
                    capture.response = artifact.sources.get(&artifact.entry).cloned();
                    capture.artifact = Some(artifact);
                }
                super::agent_projects::finish(
                    &state,
                    &run_id,
                    capture,
                    outcome.as_ref().ok().map(String::as_str),
                )
                .await
            }
            Ok(None) => (),
            Err(e) => {
                let _ = client.record.lock().await.context(
                    "literate-project-capture-error",
                    &json!({"turn":run_id,"error":format!("{e:#}")}),
                );
            }
        }
        if super::agent_projects::cancelled(&state, &run_id) {
            outcome = Err(anyhow::anyhow!(hickory_agent::STOPPED_BY_USER));
        }
        if outcome.is_ok()
            && client.supports_changesets()
            && client.host.changeset.lock().await.is_some()
        {
            let accepted = state
                .agent
                .projects
                .lock()
                .unwrap()
                .get(&run_id)
                .is_some_and(|capture| capture.artifact.is_some() && capture.error.is_none());
            if !accepted {
                outcome = Err(anyhow::anyhow!(
                    "The final artifact became stale or could not be recorded at turn completion. Its writes remain captured."
                ));
            }
        }
        // Late MCP requests must not submit against a completed turn.
        client.host.changeset.lock().await.take();
        state.agent.cancels.lock().unwrap().remove(&run_id);
        let status = {
            let mut map = state.agent.turns.lock().unwrap();
            let turn = map
                .get_mut(&doc_id)
                .and_then(|ts| ts.iter_mut().find(|t| t.id == run_id))
                .expect("registered");
            match outcome {
                Ok(answer) => {
                    turn.answer = Some(answer);
                    turn.status = "ok".into();
                }
                Err(e) => {
                    let message = format!("{e:#}");
                    turn.status = if message == hickory_agent::STOPPED_BY_USER {
                        "stopped"
                    } else {
                        "error"
                    }
                    .into();
                    turn.error = Some(message);
                }
            }
            turn.status.clone()
        };
        state
            .rooms
            .publish_run_event(&doc_id, &json!({"run_id":run_id,"status":status}))
            .await;
    });
    Ok((StatusCode::ACCEPTED, Json(json!({"session_id":turn_id}))))
}

pub fn recovered_status(path: &std::path::Path, turn: &str) -> (String, Option<String>) {
    match record::status(path, turn) {
        Some(s) => (s["status"].as_str().unwrap_or("error").into(), s["error"].as_str().map(str::to_string)),
        None => ("stopped".into(), Some("The app closed before this agent turn finished. Its recorded activity is preserved.".into())),
    }
}

#[derive(Deserialize)]
pub struct EditDecision {
    pub mode: Option<edits::Mode>,
    pub id: Option<String>,
    pub accepted: Option<bool>,
    pub error: Option<String>,
    pub current_text: Option<String>,
    pub new_text: Option<String>,
    pub capture_request: Option<String>,
    pub context: Option<super::agent_context::EditorContext>,
}
pub async fn edit_decision(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
    Json(body): Json<EditDecision>,
) -> ApiResult<Json<Value>> {
    let client = connection(&state, &doc).await?;
    if let (Some(id), Some(context)) = (body.capture_request, body.context) {
        client
            .host
            .edits
            .refreshed(&id, context.buffers)
            .map_err(error)?;
    }
    if let Some(mode) = body.mode {
        let pending = client.host.edits.snapshot()["changes"]
            .as_array()
            .is_some_and(|changes| changes.iter().any(|c| c["status"] == "pending"));
        if !pending
            && state
                .agent
                .turns
                .lock()
                .unwrap()
                .get(&doc)
                .is_some_and(|turns| turns.iter().any(|t| t.status == "running"))
        {
            return Err(ApiError::conflict(
                "Wait for this turn to finish before changing edit approval.",
            ));
        }
        // Changing modes never silently accepts an already pending edit.
        let _operation = if pending {
            None
        } else {
            Some(client.operation.try_lock().map_err(|_| {
                ApiError::conflict("Wait for an edit proposal or for this turn to finish before changing edit approval.")
            })?)
        };
        client
            .record
            .lock()
            .await
            .context("acp-edit-mode", &json!({"mode":mode}))
            .map_err(error)?;
        *client.host.edits.mode.lock().unwrap() = mode;
    }
    if let Some(id) = body.id {
        if let Some(text) = body.new_text {
            client.host.edits.revise(&id, text).map_err(error)?;
        }
        if body.accepted != Some(true)
            && let Some(content) = body.current_text
        {
            client.host.edits.refresh_rejected_buffer(&id, content);
        }
        let result = if body.accepted == Some(true) {
            Ok(())
        } else {
            Err(body.error.unwrap_or_else(|| "The user rejected this edit. Read the buffer again before trying a different change.".into()))
        };
        client
            .record
            .lock()
            .await
            .context(
                "acp-edit-decision",
                &json!({"id":id,"accepted":result.is_ok(),"error":result.as_ref().err()}),
            )
            .map_err(error)?;
        client.host.edits.resolve(&id, result).map_err(error)?;
    }
    Ok(Json(client.snapshot()))
}
pub(super) async fn project_review(state: &LocalState, doc: &str) -> ApiResult<Arc<edits::Edits>> {
    Ok(connection(state, doc).await?.host.edits.clone())
}

impl Hub {
    pub(crate) async fn connected(&self) -> bool {
        self.clients
            .lock()
            .await
            .values()
            .any(|client| !client.closed.load(std::sync::atomic::Ordering::Relaxed))
    }
}

#[cfg(test)]
pub(super) async fn assert_private_startup_refused(state: &LocalState) {
    // Guarantee: docs/guarantees/verification/predictions-precede-managed-observation.md
    // Called from the real prepared-private-session safety fixture. Neither the
    // executable lookup nor ordinary MCP recorder/executor may be reached.
    let record_path = state.index.root().join("must-not-create-acp-record.md");
    let command = config::AgentCommand {
        id: "private-refusal".into(),
        name: "Private refusal challenge".into(),
        command: "must-not-look-up-private-acp-executable".into(),
        args: vec![],
        workspace_filesystem: true,
        private_prediction_args: None,
        private_prediction_auth_method: None,
    };
    let error = client::Client::start(state, "private-doc", command, record_path.clone())
        .await
        .err()
        .expect("private ACP must refuse before spawning");
    assert!(error.to_string().contains("Private ACP"), "{error:#}");
    let record = Arc::new(tokio::sync::Mutex::new(record::Record::existing(
        record_path.clone(),
    )));
    let error = mcp::Host::start(
        state.clone(),
        record_path.clone(),
        record,
        Arc::new(tokio::sync::Mutex::new(())),
    )
    .await
    .err()
    .expect("ordinary ACP MCP must refuse private state");
    assert!(error.to_string().contains("Private ACP"), "{error:#}");
    assert!(!record_path.exists());
}
