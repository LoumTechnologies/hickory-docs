<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/private_prediction_acp.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `42adcc54f9d290f82adb5e6ac83fe91b16153b378efdc1bdd5941ffbdc85e6de`.

<book:entry path="crates/hickory-cli/src/serve/private_prediction_acp.rs">
//! Fresh ACP context with private-only managed tools. External harness internals
//! are not an OS sandbox: no ordinary history, client filesystem or terminal is supplied.
use super::{LocalState, acp, private_predictions};
use anyhow::{Context, Result, ensure};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Failure {
    Unsupported,
    NonPassing,
    Stopped,
    Unavailable,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unsupported => "Private ACP requires protocol v1, HTTP MCP and the configured advertised authentication method. Check the adapter profile before retrying.",
            Self::NonPassing => "Private ACP completion gate is non-passing. Inspect the retained comparison history and review state.",
            Self::Stopped => "Private ACP stop was requested. Inspect retained history before retrying; dispatched operations may have effects.",
            Self::Unavailable => "Private ACP did not complete. Check its private command profile and environment API credentials; adapter diagnostics remain withheld.",
        })
    }
}
impl std::error::Error for Failure {}

#[derive(Clone)]
struct Bridge {
    state: LocalState,
    window: Option<String>,
    actor: String,
    operation: Arc<Mutex<()>>,
    cancel: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}
struct Host {
    url: String,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Host {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Bridge {
    async fn call(&self, params: &Value) -> Result<Value> {
        ensure!(
            !self.cancel.load(Ordering::SeqCst),
            "Private investigation stopped"
        );
        ensure!(
            self.calls.fetch_add(1, Ordering::SeqCst) < 1920,
            "Private tool budget exhausted"
        );
        let bound = self
            .state
            .private_prediction
            .as_ref()
            .context("Private binding missing")?;
        ensure!(!bound.is_retired(), "Private session ended");
        let name = params["name"].as_str().context("Tool name required")?;
        let mut args = params.get("arguments").cloned().unwrap_or(json!({}));
        ensure!(args.is_object(), "Tool arguments must be an object");
        if matches!(name, "ui_observe" | "ui_act" | "ui_cancel") {
            let window = self
                .window
                .as_deref()
                .context("No originating private window")?;
            return Ok(super::ui_actions::tool_result(
                self.state
                    .ui_actions
                    .call(window, &self.actor, name, args)
                    .await?,
            ));
        }
        let _operation = self.operation.lock().await;
        ensure!(
            !self.cancel.load(Ordering::SeqCst),
            "Private investigation stopped"
        );
        let mut accepted = None;
        if name == "edit_doc" {
            let preview = |arguments: Value| {
                super::private_predictions::preview(
                    State(self.state.clone()),
                    Json(json!({"name":"edit_doc","arguments":arguments})),
                )
            };
            let before = preview(args.clone())
                .await
                .map_err(|_| anyhow::anyhow!("Captured edit unavailable or stale"))?
                .0;
            let id = bound
                .edits
                .submit_review(
                    "edit_doc".into(),
                    before["surface"].as_str().map(str::to_owned),
                    before["before"]
                        .as_str()
                        .context("Missing captured source")?
                        .into(),
                    before["after"]
                        .as_str()
                        .context("Missing proposed source")?
                        .into(),
                )
                .await?;
            if self.cancel.load(Ordering::SeqCst) || bound.is_retired() {
                bound.edits.finished(&id, false);
                anyhow::bail!("Private investigation stopped");
            }
            let current = preview(args.clone()).await.map(|v| v.0);
            if current.as_ref().ok() != Some(&before) {
                bound.edits.finished(&id, false);
                anyhow::bail!("Captured source changed during review; no edit ran");
            }
            if let Some(replacement) = bound.edits.replacement(&id) {
                args["input"] = json!(replacement);
                args["whole"] = json!("true");
                args["snapshot"] = json!(
                    hickory_agent::hashline::LineIndex::new(before["before"].as_str().unwrap())
                        .snapshot
                );
                if preview(args.clone()).await.is_err() {
                    bound.edits.finished(&id, false);
                    anyhow::bail!("Reviewed replacement unavailable; no edit ran");
                }
            }
            accepted = Some(id);
        }
        let result = bound
            .mcp_tool(
                self.state.executor.clone(),
                &json!({"name":name,"arguments":args}),
            )
            .await;
        if let Some(id) = accepted {
            bound.edits.finished(&id, result["isError"] != true);
        }
        Ok(result)
    }
    async fn start(self) -> Result<Host> {
        let token = format!("{:016x}{:016x}", super::rand_id(), super::rand_id());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/{token}", listener.local_addr()?);
        let router = Router::new()
            .route(&format!("/{token}"), post(exchange))
            .with_state(self);
        let task = tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        Ok(Host { url, task })
    }
}
async fn exchange(
    State(bridge): State<Bridge>,
    Json(message): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let Some(id) = message.get("id") else {
        return (StatusCode::ACCEPTED, Json(Value::Null));
    };
    let result = match message["method"].as_str() {
        Some("initialize") => Ok(
            json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"hickory-private-prediction","version":env!("CARGO_PKG_VERSION")}}),
        ),
        Some("tools/list") => {
            let bound = bridge.state.private_prediction.as_ref().unwrap();
            let mut catalogue = private_predictions::catalogue(bound.retained_plan.is_some());
            if bridge.window.is_some() {
                catalogue["tools"]
                    .as_array_mut()
                    .unwrap()
                    .extend(super::ui_actions::catalogue());
            }
            Ok(catalogue)
        }
        Some("tools/call") => bridge.call(&message["params"]).await,
        _ => Err(anyhow::anyhow!("Private MCP method unavailable")),
    };
    // Never forward adapter/runner diagnostics through this protocol error path.
    let value = match result {
        Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
        Err(_) => {
            json!({"jsonrpc":"2.0","id":id,"result":{"isError":true,"content":[{"type":"text","text":"Private tool refused or cancelled. Inspect current captured source, review state and seal ordering; target values stay withheld."}]}})
        }
    };
    (StatusCode::OK, Json(value))
}

pub(super) struct Prepared {
    command: tokio::process::Command,
    directory: tempfile::TempDir,
    auth_method: Option<String>,
}

// This authorizes only the adapter's envelope around our already guarded MCP
// call. It is neither edit approval nor a filesystem/terminal permission.
#[derive(Default)]
struct McpPermissions {
    session: Option<String>,
    pending: HashMap<String, String>,
    allowed: HashSet<String>,
}
impl McpPermissions {
    fn observe(&mut self, message: &Value) {
        let params = &message["params"];
        if message["method"] != "session/update"
            || params["sessionId"].as_str() != self.session.as_deref()
            || self.session.is_none()
        {
            return;
        }
        let update = &params["update"];
        let Some(id) = update["toolCallId"].as_str() else {
            return;
        };
        if matches!(update["status"].as_str(), Some("completed" | "failed")) {
            self.pending.remove(id);
            return;
        }
        if update["sessionUpdate"] != "tool_call" || self.pending.len() >= 128 {
            return;
        }
        // A replacement notification cannot retain the previous call's scope.
        self.pending.remove(id);
        let input = &update["rawInput"];
        if input["server"] == "hickory-private-prediction"
            && input["tool"]
                .as_str()
                .is_some_and(|tool| self.allowed.contains(tool))
            && input["arguments"].is_object()
        {
            self.pending
                .insert(id.into(), params["sessionId"].as_str().unwrap().into());
        }
    }

    fn outcome(&mut self, params: &Value) -> Value {
        let cancelled = json!({"outcome":"cancelled"});
        let Some(id) = params["toolCall"]["toolCallId"].as_str() else {
            return cancelled;
        };
        let Some(session) = self.pending.remove(id) else {
            return cancelled;
        };
        if self.session.as_deref() != Some(&session)
            || params["sessionId"] != session
            || params["_meta"]["is_mcp_tool_approval"] != true
            || params["toolCall"]["kind"] != "execute"
        {
            return cancelled;
        }
        let Some(option) = params["options"].as_array().and_then(|options| {
            options
                .iter()
                .find(|option| option["kind"] == "allow_once" && option["optionId"].is_string())
        }) else {
            return cancelled;
        };
        json!({"outcome":"selected","optionId":option["optionId"]})
    }
}
pub(super) fn prepare(state: &LocalState, id: &str) -> Result<Prepared> {
    let agent = acp::config::commands(state)?
        .into_iter()
        .find(|agent| agent.id == id)
        .context(
            "Configure this ACP command in ordinary Settings before entering the private practice",
        )?;
    prepare_command(agent, &acp::config::directory(state))
}
fn prepare_command(
    agent: acp::config::AgentCommand,
    config_dir: &std::path::Path,
) -> Result<Prepared> {
    ensure!(
        !agent.workspace_filesystem,
        "Private ACP cannot mount the workspace filesystem"
    );
    let private_args = agent.private_prediction_args.context("Configure explicit private_prediction_args for this adapter; ordinary resume/configuration arguments cannot enter a private context")?;
    let executable = acp::config::executable(&agent.command, config_dir)
        .context("Private ACP adapter is not installed")?;
    ensure!(
        executable
            .extension()
            .is_none_or(|extension| extension != "cmd"),
        "Use a direct executable and script arguments for private ACP"
    );
    let directory = tempfile::tempdir()?;
    let mut command = tokio::process::Command::new(executable);
    command
        .args(private_args)
        .current_dir(directory.path())
        .env_clear();
    // Explicit credentials only, never inherited engine policy/tokens, HOME,
    // workspace, Python injection, provider base URLs, startup files or history.
    for key in [
        "PATH",
        "SYSTEMROOT",
        "WINDIR",
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENROUTER_API_KEY",
        "GOOGLE_API_KEY",
        "GEMINI_API_KEY",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    command
        .env("HOME", directory.path())
        .env("USERPROFILE", directory.path())
        .env("XDG_CONFIG_HOME", directory.path().join("config"))
        .env("XDG_DATA_HOME", directory.path().join("data"))
        .env("XDG_CACHE_HOME", directory.path().join("cache"));
    Ok(Prepared {
        command,
        directory,
        auth_method: agent.private_prediction_auth_method,
    })
}

pub(super) async fn run(
    state: LocalState,
    prepared: Prepared,
    prompt: String,
    window: Option<String>,
    actor: String,
    cancel: Arc<AtomicBool>,
) -> Result<String> {
    let catalogue = private_predictions::catalogue(
        state
            .private_prediction
            .as_ref()
            .is_some_and(|bound| bound.retained_plan.is_some()),
    );
    let mut allowed: HashSet<String> = catalogue["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|tool| tool["name"].as_str().map(str::to_owned))
        .collect();
    if window.is_some() {
        allowed.extend(
            super::ui_actions::catalogue()
                .iter()
                .filter_map(|tool| tool["name"].as_str().map(str::to_owned)),
        );
    }
    let permissions = Arc::new(Mutex::new(McpPermissions {
        allowed,
        ..Default::default()
    }));
    let host = Bridge {
        state: state.clone(),
        window,
        actor,
        operation: Arc::default(),
        cancel: cancel.clone(),
        calls: Arc::default(),
    }
    .start()
    .await?;
    let (rpc, mut events) = acp::transport::Rpc::spawn(prepared.command)?;
    let event_rpc = rpc.clone();
    let event_permissions = permissions.clone();
    let events_task = tokio::spawn(async move {
        while let Some(message) = events.recv().await {
            event_permissions.lock().await.observe(&message);
            // Client-side filesystem/terminal/extension calls remain refused.
            // Only a correlated private MCP envelope can receive allow_once;
            // Bridge::call still enforces sealing and independent edit review.
            if let Some(id) = message.get("id") {
                let reply = if message["method"] == "session/request_permission" {
                    let outcome = event_permissions.lock().await.outcome(&message["params"]);
                    json!({"jsonrpc":"2.0","id":id,"result":{"outcome":outcome}})
                } else {
                    json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Unavailable in private prediction mode; use the private MCP tools"}})
                };
                let _ = event_rpc.send(reply).await;
            }
            // Model notifications are not verification receipts. Do not hydrate
            // ordinary transcripts or return unbounded provider diagnostics.
        }
    });
    let conversation = async {
        let initialized = rpc.request("initialize", json!({"protocolVersion":1,"clientInfo":{"name":"hickory-private-prediction","version":env!("CARGO_PKG_VERSION")},"clientCapabilities":{"fs":{"readTextFile":false,"writeTextFile":false},"terminal":false}}), Duration::from_secs(30)).await?;
        if initialized["protocolVersion"] != 1
            || initialized["agentCapabilities"]["mcpCapabilities"]["http"] != true
        {
            return Err(Failure::Unsupported.into());
        }
        if let Some(method) = &prepared.auth_method {
            if !initialized["authMethods"]
                .as_array()
                .is_some_and(|methods| {
                    methods
                        .iter()
                        .any(|entry| entry["id"].as_str() == Some(method))
                })
            {
                return Err(Failure::Unsupported.into());
            }
            rpc.request(
                "authenticate",
                json!({"methodId":method}),
                Duration::from_secs(60),
            )
            .await?;
        }
        let session = rpc.request("session/new", json!({"cwd":prepared.directory.path(),"mcpServers":[{"type":"http","name":"hickory-private-prediction","url":host.url,"headers":[]}]}), Duration::from_secs(60)).await?;
        let session_id = session["sessionId"]
            .as_str()
            .context("Missing private ACP session identity")?;
        permissions.lock().await.session = Some(session_id.into());
        let text = format!(
            "Private prediction practice: use only the supplied private MCP tools. No terminal or filesystem client is available. Read approved source, seal before execution, keep target answers withheld, and obtain human review before edits. A model completion is not verification.\n\n{prompt}"
        );
        let result = rpc
            .request(
                "session/prompt",
                json!({"sessionId":session_id,"prompt":[{"type":"text","text":text}]}),
                Duration::from_secs(600),
            )
            .await?;
        ensure!(
            result["stopReason"] == "end_turn",
            "Private ACP did not finish its turn"
        );
        let completion = private_predictions::completion(State(state.clone()))
            .await
            .map_err(|_| anyhow::anyhow!("Private completion unavailable"))?
            .0;
        if completion["ready"] != true {
            return Err(Failure::NonPassing.into());
        }
        Ok("Private ACP investigation finished; the host repair-and-verify gate passed. No current certification or ordinary changeset acceptance is claimed.".into())
    };
    let result = tokio::select! { result = conversation => result, _ = async { while !cancel.load(Ordering::SeqCst) { tokio::time::sleep(Duration::from_millis(50)).await; } } => Err(Failure::Stopped.into()) };
    cancel.store(true, Ordering::SeqCst);
    if let Some(bound) = &state.private_prediction {
        bound.edits.cancel();
    }
    rpc.kill().await;
    events_task.abort();
    result.map_err(|error: anyhow::Error| {
        if error.is::<Failure>() {
            error
        } else {
            Failure::Unavailable.into()
        }
    })
}

#[cfg(test)]
#[path = "private_prediction_acp_tests.rs"]
mod tests;
#[cfg(test)]
pub(super) use tests::managed_boundary as test_managed_boundary;

#[cfg(test)]
pub(super) use tests::protocol_boundary as test_protocol_boundary;
</book:entry>

</book:doc>
