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
    context: super::client::RoomContext,
    server: Arc<Mutex<crate::mcp::Server>>,
    record: Arc<Mutex<Record>>,
    workspace_gate: Arc<Mutex<()>>,
}

pub struct Host {
    pub url: String,
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
        let server =
            crate::mcp::Server::embedded(state.index.root().to_path_buf(), doc, executor, session);
        let bridge = Bridge {
            context: super::client::RoomContext {
                index: state.index.clone(),
                rooms: state.rooms.clone(),
            },
            server: Arc::new(Mutex::new(server)),
            record,
            workspace_gate,
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
        Ok(Self { url, task })
    }
}

async fn exchange(State(bridge): State<Bridge>, Json(message): Json<Value>) -> impl IntoResponse {
    let Some(id) = message.get("id") else {
        return (StatusCode::ACCEPTED, Json(Value::Null));
    };
    let method = message["method"].as_str().unwrap_or("");
    // The same lock gates ACP writes and MCP evidence. Never two open file writers.
    let _workspace = bridge.workspace_gate.lock().await;
    let _record = bridge.record.lock().await;
    let mut server = bridge.server.lock().await;
    let params = message.get("params").cloned().unwrap_or(json!({}));
    let snapshots = if method == "tools/call" {
        match super::workspace::capture(&bridge.context).await {
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
    let mut result = server.handle(method, &params).await;
    if let Err(e) = super::workspace::finish(&bridge.context, snapshots).await {
        result = Err((
            -32603,
            format!("Could not merge the tool edit into the open document: {e:#}"),
        ));
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
