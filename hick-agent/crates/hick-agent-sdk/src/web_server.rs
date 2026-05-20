//! Local HTTP + WebSocket server for agent UI prototyping (SPA + [`UiEvent`] stream).
//!
//! Enabled with the Cargo feature **`web`** (pulls in `axum`). Binaries attach the server's
//! WebSocket handlers to [`tokio::sync::broadcast::Sender`] for [`UiEvent`] fan-out.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use axum::Router;
use axum::extract::Query;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

use crate::{UiCommand, UiEvent};
use crate::ui_event::ui_event_wire_value;
use crate::session_bundle_reader::SessionBundleReader;

// ---------------------------------------------------------------------------
// Embedded React SPA (`hick-agent/web-ui/dist`, built via `just build-web-ui`)
// ---------------------------------------------------------------------------

static INDEX_HTML: &[u8] = include_bytes!("../../../web-ui/dist/index.html");
static ASSET_APP_JS: &[u8] = include_bytes!("../../../web-ui/dist/assets/app.js");
static ASSET_APP_CSS: &[u8] = include_bytes!("../../../web-ui/dist/assets/app.css");

/// Default TCP port (`7890`).
pub const DEFAULT_WEB_PORT: u16 = 7890;

/// Tunables for [`WebServer::bind`].
#[derive(Debug, Clone)]
pub struct WebServerConfig {
    /// Listening port (`HICK_WEB_PORT` overrides via [`WebServerConfig::from_env`]).
    pub port: u16,
    /// Project root directory (serves `/cwd/` and `/pane-files/` routes).
    pub project_dir: Option<PathBuf>,
    /// Allowed-host list used to generate CSP for HTML panes.
    pub allowed_hosts: Vec<crate::security_config::HostEntry>,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_WEB_PORT,
            project_dir: None,
            allowed_hosts: Vec::new(),
        }
    }
}

impl WebServerConfig {
    /// Read `HICK_WEB_PORT`; fall back to [`DEFAULT_WEB_PORT`] when unset or invalid.
    pub fn from_env() -> Self {
        let port = std::env::var("HICK_WEB_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_WEB_PORT);
        Self { port, project_dir: None, allowed_hosts: Vec::new() }
    }

    /// Attach project directory and security config to the web server configuration.
    pub fn with_project(mut self, project_dir: PathBuf, allowed_hosts: Vec<crate::security_config::HostEntry>) -> Self {
        self.project_dir = Some(project_dir);
        self.allowed_hosts = allowed_hosts;
        self
    }
}

#[derive(Clone)]
struct Attached {
    event_tx: broadcast::Sender<UiEvent>,
    cmd_tx: mpsc::UnboundedSender<UiCommand>,
    /// Pre-serialized event history for page-refresh replay (TASK-37.10).
    history: Arc<Mutex<Vec<String>>>,
    /// Project root for /cwd/ and /pane-files/ static routes.
    project_dir: Option<PathBuf>,
    /// Allowed-host list for HTML pane CSP.
    allowed_hosts: Arc<Vec<crate::security_config::HostEntry>>,
}

/// Localhost HTTP service (static SPA + `/ws`).
pub struct WebServer {
    listener: Mutex<Option<TcpListener>>,
    port: u16,
    attached: Mutex<Option<Attached>>,
    config: WebServerConfig,
}

impl WebServer {
    /// Listen on `127.0.0.1:{config.port}`.
    ///
    /// Call [`attach`](Self::attach) before [`start`](Self::start).
    pub async fn bind(config: WebServerConfig) -> anyhow::Result<Self> {
        let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("bind HTTP listener on {addr}"))?;
        let port = listener.local_addr()?.port();
        Ok(Self {
            listener: Mutex::new(Some(listener)),
            port,
            attached: Mutex::new(None),
            config,
        })
    }

    /// Connect UI channels: outbound [`UiEvent`] broadcast and inbound [`UiCommand`] sink.
    pub fn attach(
        &self,
        event_tx: broadcast::Sender<UiEvent>,
        cmd_tx: mpsc::UnboundedSender<UiCommand>,
    ) {
        let history = Arc::new(Mutex::new(Vec::<String>::new()));
        self.attached
            .lock()
            .expect("web mutex")
            .replace(Attached {
                event_tx,
                cmd_tx,
                history,
                project_dir: self.config.project_dir.clone(),
                allowed_hosts: Arc::new(self.config.allowed_hosts.clone()),
            });
    }

    /// Spawn `axum` on the bound listener (non-blocking). Returns `http://localhost:{port}`.
    pub async fn start(&self) -> anyhow::Result<String> {
        let listener = self
            .listener
            .lock()
            .expect("listener mutex poisoned")
            .take()
            .ok_or_else(|| anyhow::anyhow!("WebServer::start called more than once"))?;

        let attached = self
            .attached
            .lock()
            .expect("attached mutex poisoned")
            .clone()
            .ok_or_else(|| anyhow::anyhow!("call WebServer::attach before start"))?;

        // TASK-37.10: spawn history-capture task.
        // Subscribe while holding the history lock ensures no events are lost between the
        // snapshot and the live subscription in ws_connected.
        {
            let history = Arc::clone(&attached.history);
            let mut sub = attached.event_tx.subscribe();
            tokio::spawn(async move {
                while let Ok(ev) = sub.recv().await {
                    let Ok(text) = serde_json::to_string(&ui_event_wire_value(&ev)) else {
                        continue;
                    };
                    history.lock().unwrap().push(text);
                }
            });
        }

        let routes = Router::new()
            .route("/", get(serve_index))
            .route("/assets/{name}", get(serve_asset))
            .route("/ws", get(ws_upgrade))
            .route("/ws/replay", get(ws_replay_upgrade))
            .route("/pane-files/{*path}", get(serve_pane_file))
            .route("/cwd/{*path}", get(serve_cwd_file))
            .with_state(attached.clone());

        let addr = listener.local_addr()?;
        let url = format!("http://localhost:{}", addr.port());

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, routes.into_make_service()).await {
                log::warn!("web server exited: {e}");
            }
        });

        Ok(url)
    }

    /// Resolved listen port ([`TcpListener::local_addr`] after bind).
    pub fn configured_port(&self) -> u16 {
        self.port
    }
}

async fn serve_index() -> impl IntoResponse {
    ([
        (axum::http::header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
        (axum::http::header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
    ], INDEX_HTML)
}

async fn serve_asset(Path(name): Path<String>) -> impl IntoResponse {
    let body: &[u8] = match name.as_str() {
        "app.js" => ASSET_APP_JS,
        "app.css" => ASSET_APP_CSS,
        _ => &[],
    };
    if body.is_empty() {
        return (
            axum::http::StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "not found",
        )
            .into_response();
    }
    let ct = match name.rsplit_once('.').map(|(_, ext)| ext) {
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        _ => "application/octet-stream",
    };
    let hv = HeaderValue::from_str(ct).unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    ([
        (axum::http::header::CONTENT_TYPE, hv),
        (axum::http::header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
    ], body).into_response()
}

// ---------------------------------------------------------------------------
// WebSocket — live events + TASK-37.10 history replay on reconnect
// ---------------------------------------------------------------------------

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(attached): State<Attached>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_connected(socket, attached))
}

async fn ws_connected(socket: WebSocket, attached: Attached) {
    // TASK-37.10: atomically subscribe + snapshot so no events are missed or duplicated.
    let (recv, snapshot) = {
        let guard = attached.history.lock().unwrap();
        let recv = attached.event_tx.subscribe();
        let snap = guard.clone(); // clone while locked
        drop(guard);
        (recv, snap)
    };
    let mut recv = recv;
    let cmd_tx = attached.cmd_tx;
    let (mut sink, mut stream) = socket.split();

    // Send history first (page-refresh replay).
    for msg in &snapshot {
        if sink.send(Message::text(msg.as_str())).await.is_err() {
            let _ = sink.close().await;
            return;
        }
    }

    // Forward live events from broadcast; accept commands from the browser.
    loop {
        tokio::select! {
            ev = recv.recv() => {
                match ev {
                    Ok(ev) => {
                        let Ok(text) = serde_json::to_string(&ui_event_wire_value(&ev)) else { continue };
                        if sink.send(Message::text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            in_msg = stream.next() => {
                match in_msg {
                    None => break,
                    Some(Ok(Message::Text(t))) => {
                        let _ = parse_and_forward_command(&cmd_tx, t.as_bytes());
                    }
                    Some(Ok(Message::Binary(b))) => {
                        let _ = parse_and_forward_command(&cmd_tx, &b);
                    }
                    Some(Ok(Message::Ping(_))) => {}
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(_)) => break,
                }
            }
        }
    }
    let _ = sink.close().await;
}

fn parse_and_forward_command(cmd_tx: &mpsc::UnboundedSender<UiCommand>, bytes: &[u8]) -> bool {
    match serde_json::from_slice::<UiCommand>(bytes) {
        Ok(c) => cmd_tx.send(c).is_ok(),
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Replay WebSocket endpoint — streams a session bundle file over WS
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ReplayQuery {
    path: String,
}

async fn ws_replay_upgrade(
    ws: WebSocketUpgrade,
    Query(q): Query<ReplayQuery>,
    State(_attached): State<Attached>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_replay_connected(socket, q.path))
}

async fn ws_replay_connected(socket: WebSocket, bundle_path: String) {
    let (mut sink, _stream) = socket.split();

    let bundle_dir = std::path::Path::new(&bundle_path);
    let events = match SessionBundleReader::read(bundle_dir) {
        Ok(e) => e,
        Err(err) => {
            let msg = serde_json::json!({
                "type": "error",
                "node_id": 0,
                "text": format!("replay: {err}")
            });
            let _ = sink.send(Message::text(msg.to_string())).await;
            let _ = sink.close().await;
            return;
        }
    };

    for ev in &events {
        let Ok(text) = serde_json::to_string(&ui_event_wire_value(ev)) else {
            continue;
        };
        if sink.send(Message::text(text)).await.is_err() {
            break;
        }
    }
    let _ = sink.close().await;
}

// ---------------------------------------------------------------------------
// /pane-files/{path} — serves files from <project_dir>/.agents/panes/
// TASK-37.11
// ---------------------------------------------------------------------------

async fn serve_pane_file(
    Path(rel): Path<String>,
    State(attached): State<Attached>,
) -> impl IntoResponse {
    let Some(ref project_dir) = attached.project_dir else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let panes_root = project_dir.join(".agents").join("panes");
    serve_contained_file(&panes_root, &rel, None).await
}

// ---------------------------------------------------------------------------
// /cwd/{path} — serves files from the project CWD (TASK-37.12)
// ---------------------------------------------------------------------------

async fn serve_cwd_file(
    Path(rel): Path<String>,
    State(attached): State<Attached>,
) -> impl IntoResponse {
    let Some(ref project_dir) = attached.project_dir else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // Determine if this path is under an HTML pane directory so we can attach a CSP.
    let csp = html_pane_csp_for_path(&rel, project_dir, &attached.allowed_hosts);
    serve_contained_file(project_dir, &rel, csp.as_deref()).await
}

/// Returns Some(csp) if `rel` falls under `.agents/panes/<name>/` for an HTML pane.
fn html_pane_csp_for_path(
    rel: &str,
    project_dir: &std::path::Path,
    allowed_hosts: &[crate::security_config::HostEntry],
) -> Option<String> {
    let pane_prefix = ".agents/panes/";
    let after_panes = rel.strip_prefix(pane_prefix)?;
    let pane_name = after_panes.split('/').next()?;
    let pane_dir = project_dir.join(".agents").join("panes").join(pane_name);

    let yaml_path = pane_dir.join("pane.yaml");
    let text = std::fs::read_to_string(&yaml_path).ok()?;
    let parsed: serde_json::Value = serde_yaml::from_str(&text).ok()?;
    if parsed.get("type")?.as_str()? != "html" {
        return None;
    }
    let extra: Vec<String> = parsed
        .get("extra_connect_src")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    Some(crate::pane_csp::generate_pane_csp(allowed_hosts, &extra))
}

/// Serve a file under `root` at relative path `rel`, enforcing path containment.
async fn serve_contained_file(
    root: &std::path::Path,
    rel: &str,
    csp: Option<&str>,
) -> axum::response::Response {
    // Sanitize relative path — strip any leading slashes then canonicalize.
    let rel_clean = rel.trim_start_matches('/');
    let joined = root.join(rel_clean);

    // Path containment check (blocks traversal via `..` or symlinks).
    let Ok(canonical_root) = root.canonicalize() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(canonical_joined) = joined.canonicalize() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if !canonical_joined.starts_with(&canonical_root) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if canonical_joined.is_dir() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let body = match std::fs::read(&canonical_joined) {
        Ok(b) => b,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let ext = canonical_joined
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let content_type = ext_to_content_type(ext);

    let mut resp = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, content_type)
        .header(axum::http::header::CACHE_CONTROL, "no-store");

    if let Some(csp_val) = csp {
        resp = resp.header("Content-Security-Policy", csp_val);
    }

    resp.body(axum::body::Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn ext_to_content_type(ext: &str) -> &'static str {
    match ext {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "md" => "text/markdown; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}
