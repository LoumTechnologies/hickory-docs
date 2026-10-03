//! Private MCP bridge using the existing tool implementation, with one recorder.
use super::{super::LocalState, record::Record};
use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct Bridge {
    state: LocalState,
    context: super::client::RoomContext,
    server: Arc<Mutex<crate::mcp::Server>>,
    record: Arc<Mutex<Record>>,
    workspace_gate: Arc<Mutex<()>>,
    edits: Arc<super::edits::Edits>,
}

pub struct Host {
    pub url: String,
    pub edits: Arc<super::edits::Edits>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Host {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Host {
    pub async fn start(
        state: LocalState,
        doc: PathBuf,
        record: Arc<Mutex<Record>>,
        workspace_gate: Arc<Mutex<()>>,
    ) -> Result<Self> {
        let executor = state.executor.build().await?;
        let session = record.lock().await.path.clone();
        let server = crate::mcp::Server::embedded(
            state.index.root().to_path_buf(),
            doc,
            executor,
            session.clone(),
        );
        let edits = Arc::new(super::edits::Edits::default());
        if let Some(mode) = super::record::edit_mode(&session) {
            *edits.mode.lock().unwrap() = mode;
        }
        let bridge = Bridge {
            state: state.clone(),
            context: super::client::RoomContext {
                index: state.index.clone(),
                rooms: state.rooms.clone(),
            },
            server: Arc::new(Mutex::new(server)),
            record,
            workspace_gate,
            edits: edits.clone(),
        };
        let token = format!(
            "{:016x}{:016x}",
            super::super::rand_id(),
            super::super::rand_id()
        );
        let router = Router::new()
            .route(&format!("/{token}"), post(exchange))
            .with_state(bridge);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/{token}", listener.local_addr()?);
        let task = tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        Ok(Self { url, edits, task })
    }
}

async fn exchange(State(bridge): State<Bridge>, Json(message): Json<Value>) -> impl IntoResponse {
    let Some(id) = message.get("id") else {
        return (StatusCode::ACCEPTED, Json(Value::Null));
    };
    let method = message["method"].as_str().unwrap_or("");
    let params = message.get("params").cloned().unwrap_or(json!({}));
    let name = params["name"].as_str().unwrap_or("");
    let args = &params["arguments"];
    if method == "tools/call"
        && let Some(buffer) = bridge.edits.buffer(&json!({}))
        && let Some(path) = buffer.document.or(buffer.path)
    {
        let path = bridge.state.index.root().join(path);
        if path.exists() {
            bridge.server.lock().await.set_default_doc(path);
        }
    }
    if method == "tools/call"
        && matches!(
            name,
            "read_buffer" | "edit_buffer" | "read_doc" | "edit_doc"
        )
        && args.get("upstream").is_none()
        && let Some(buffer) = bridge.edits.buffer(args)
    {
        let outcome = buffer_call(&bridge, name, args, buffer).await;
        let result = match outcome {
            Ok(text) => json!({"content":[{"type":"text","text":text}],"isError":false}),
            Err(e) => json!({"content":[{"type":"text","text":format!("{e:#}")}],"isError":true}),
        };
        return (
            StatusCode::OK,
            Json(json!({"jsonrpc":"2.0","id":id,"result":result})),
        );
    }
    if method == "tools/call" && matches!(name, "read_buffer" | "edit_buffer") {
        return (
            StatusCode::OK,
            Json(
                json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":"No matching open editor. Use read_doc with an explicit document path."}],"isError":true}}),
            ),
        );
    }
    let reviewed =
        if method == "tools/call" && matches!(name, "edit_doc" | "edit_output" | "create_doc") {
            let preview = {
                let _workspace = bridge.workspace_gate.lock().await;
                if let Err(e) = super::workspace::capture(&bridge.context).await {
                    return tool_error(id, &format!("Could not flush the editor: {e:#}"));
                }
                bridge.server.lock().await.preview_edit(name, args).await
            };
            match preview {
                Ok((path, old, next)) => {
                    let change_id = match bridge
                        .edits
                        .submit(
                            path.clone(),
                            Some(path.clone()),
                            old.clone(),
                            next.clone(),
                            false,
                        )
                        .await
                    {
                        Ok(id) => id,
                        Err(e) => return tool_error(id, &format!("{e:#}")),
                    };
                    Some(((path, old, next), change_id))
                }
                Err(e) => return tool_error(id, &e),
            }
        } else {
            None
        };
    // The same lock gates ACP writes and MCP evidence. Never two open file writers.
    let _workspace = bridge.workspace_gate.lock().await;
    let _record = bridge.record.lock().await;
    let mut server = bridge.server.lock().await;
    let review_id = reviewed.as_ref().map(|(_, id)| id.clone());
    let captured = if method == "tools/call"
        && !super::super::representation_tools::catalogue()
            .iter()
            .any(|tool| tool["name"] == params["name"])
    {
        super::workspace::capture(&bridge.context).await
    } else {
        Ok(Default::default())
    };
    if let Some((preview, change_id)) = reviewed {
        match server.preview_edit(name, args).await {
            Ok(current) if current == preview => {}
            _ => {
                bridge.edits.finished(&change_id, false);
                return tool_error(
                    id,
                    "The document changed during review. Read it again before editing; nothing was changed.",
                );
            }
        }
    }
    let representation_call = method == "tools/call"
        && super::super::representation_tools::catalogue()
            .iter()
            .any(|tool| tool["name"] == params["name"]);
    let snapshots = if method == "tools/call" && !representation_call {
        match captured {
            Ok(snapshots) => snapshots,
            Err(e) => {
                return (
                    StatusCode::OK,
                    Json(
                        json!({"jsonrpc":"2.0","id":id,"error":{"code":-32603,"message":format!("Could not flush document edits: {e:#}")}}),
                    ),
                );
            }
        }
    } else {
        Default::default()
    };
    let mut result = if representation_call {
        super::super::representation_tools::call(
            &bridge.state,
            params["name"].as_str().unwrap_or(""),
            &params["arguments"],
        )
        .await
        .map_err(|e| (-32603, e.message().to_string()))
    } else {
        server.handle(method, &params).await
    };
    if representation_call {
        let _ = _record.context(
            "literate-view-tool",
            &json!({"tool":params["name"],"arguments":params["arguments"],"ok":result.is_ok(),"observation":result.as_ref().ok()}),
        );
    }
    if method == "tools/list"
        && let Ok(value) = &mut result
        && let Some(tools) = value["tools"].as_array_mut()
    {
        tools.extend(super::super::representation_tools::catalogue());
        tools.extend(super::edits::catalogue());
    }
    if let Err(e) = super::workspace::finish(&bridge.context, snapshots).await {
        result = Err((
            -32603,
            format!("Could not merge the tool edit into the open document: {e:#}"),
        ));
    }
    if let Some(id) = review_id {
        bridge
            .edits
            .finished(&id, result.as_ref().is_ok_and(|v| v["isError"] != true));
    }
    let reply = match result {
        Ok(value) => json!({"jsonrpc":"2.0", "id":id, "result":value}),
        Err((code, message)) => {
            json!({"jsonrpc":"2.0", "id":id, "error":{"code":code, "message":message}})
        }
    };
    (StatusCode::OK, Json(reply))
}

/// The CLI and the desktop executable can both serve a stdio MCP proxy, so a
/// downloaded app needs no separately installed hick CLI. No shell evaluation.
pub fn proxy_argv(args: &[String]) -> Option<Result<()>> {
    if args.first().map(String::as_str) != Some("--hickory-mcp-proxy") {
        return None;
    }
    Some((|| {
        let url = args
            .get(1)
            .context("the MCP proxy needs its private loopback URL")?;
        let parsed = reqwest::Url::parse(url)?;
        anyhow::ensure!(
            parsed.scheme() == "http" && parsed.host_str() == Some("127.0.0.1"),
            "MCP proxy only connects to its local engine"
        );
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(proxy(url))
    })())
}

async fn proxy(url: &str) -> Result<()> {
    use std::io::{BufRead, Write};
    let client = reqwest::Client::new();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let message: Value = serde_json::from_str(&line?)?;
        let response = client
            .post(url)
            .json(&message)
            .send()
            .await?
            .error_for_status()?;
        if message.get("id").is_some() {
            let reply: Value = response.json().await?;
            writeln!(stdout, "{reply}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

pub fn configuration(host: &Host, http: bool) -> Result<Value> {
    if http {
        return Ok(json!({"type":"http", "name":"hick", "url": host.url, "headers":[]}));
    }
    Ok(
        json!({"name":"hick", "command": std::env::current_exe()?.display().to_string(), "args":["--hickory-mcp-proxy", host.url], "env":[]}),
    )
}

pub fn confined(root: &Path, raw: &str) -> Result<PathBuf> {
    let path = PathBuf::from(raw);
    anyhow::ensure!(path.is_absolute(), "ACP file paths must be absolute");
    let canonical = if path.exists() {
        path.canonicalize()?
    } else {
        path.parent()
            .context("file has no parent")?
            .canonicalize()?
            .join(path.file_name().context("file has no name")?)
    };
    anyhow::ensure!(
        canonical.starts_with(root.canonicalize()?),
        "{} is outside this workspace; use an agent tool with an explicit permission instead",
        path.display()
    );
    Ok(canonical)
}

async fn buffer_call(
    bridge: &Bridge,
    name: &str,
    args: &Value,
    buffer: super::super::agent_context::EditorBuffer,
) -> Result<String> {
    use hickory_agent::hashline::LineIndex;
    if matches!(name, "read_doc" | "read_buffer") {
        bridge
            .record
            .lock()
            .await
            .context("acp-buffer-read", &json!({"buffer":buffer}))?;
        return Ok(format!(
            "buffer {} ({}):\n{}",
            buffer.id.as_deref().unwrap_or("focused"),
            buffer.name,
            LineIndex::new(&buffer.content).render()
        ));
    }
    anyhow::ensure!(
        buffer.kind.as_deref() != Some("generated"),
        "This is generated code. Use read_output and edit_output so the edit lands in its source document through lineage."
    );
    let next = super::edits::replacement(&buffer.content, args)?;
    hick_lang::parse(&next).context("This edit would break the document; nothing was changed")?;
    bridge
        .record
        .lock()
        .await
        .context("acp-buffer-edit", &json!({"buffer":buffer,"content":next}))?;
    bridge
        .edits
        .submit(
            buffer.id.clone().unwrap_or_else(|| buffer.name.clone()),
            buffer.path.clone(),
            buffer.content.clone(),
            next.clone(),
            true,
        )
        .await?;
    bridge.edits.updated(&buffer, next.clone());
    Ok(format!(
        "Edited the live buffer. Fresh hashes:\n{}",
        LineIndex::new(&next).render()
    ))
}

fn tool_error(id: &Value, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(
            json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":message}],"isError":true}}),
        ),
    )
}
