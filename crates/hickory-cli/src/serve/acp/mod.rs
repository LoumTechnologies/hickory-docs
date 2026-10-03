//! ACP is an optional agent extension. The language and model client stay independent.
mod client;
mod config;
mod mcp;
pub(crate) mod record;
mod transport;
mod workspace;
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

pub async fn catalogue(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let dir = config::directory(&state);
    let agents: Vec<Value> = config::commands(&state).map_err(error)?.into_iter().map(|agent| {
        let available = config::executable(&agent.command, &dir).is_some();
        let installable = matches!(agent.id.as_str(), "codex" | "claude") && config::executable("npm", &dir).is_some();
        let cli_available = matches!(agent.id.as_str(), "codex" | "claude") && config::executable(&agent.id, &dir).is_some();
        json!({"id":agent.id,"name":agent.name,"command":agent.command,"args":agent.args,"available":available,"installable":installable,"cli_available":cli_available,"workspace_filesystem":agent.workspace_filesystem})
    }).collect();
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
    let path = match body.session.as_deref() {
        Some(rel) => {
            let path = state.index.root().join(rel);
            let canonical = path.canonicalize().map_err(|e| error(e.into()))?;
            if !canonical.starts_with(
                state
                    .index
                    .root()
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
            state.index.root(),
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
    let _operation = client.operation.lock().await;
    let setup_error = client.setup().await.err().map(|e| format!("{e:#}"));
    let mut view = client.snapshot();
    view["error"] = json!(setup_error);
    view["session"] = json!(
        client
            .record
            .lock()
            .await
            .path
            .strip_prefix(state.index.root())
            .map(|p| p.display().to_string())
            .unwrap_or_default()
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
    state.agent.hydrate(state.index.root(), &doc_id, &doc);
    let prompt = body.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(ApiError::bad_request("Say what you want the agent to do."));
    }
    let session = client
        .record
        .lock()
        .await
        .path
        .strip_prefix(state.index.root())
        .map(|p| p.display().to_string())
        .unwrap_or_default();
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
    let context =
        if doc_id == super::agent_context::WORKSPACE_AGENT || !body.context.buffers.is_empty() {
            super::agent_context::describe(&state, &body.context)
        } else {
            String::new()
        };
    let run_id = turn_id.clone();
    tokio::spawn(async move {
        let outcome = client
            .prompt(
                &run_id,
                body.parent_id.as_deref(),
                &prompt,
                &context,
                cancel,
            )
            .await;
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
