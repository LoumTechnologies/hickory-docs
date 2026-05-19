//! Local HTTP + WebSocket server for agent UI prototyping (SPA + [`UiEvent`] stream).
//!
//! Enabled with the Cargo feature **`web`** (pulls in `axum`). Binaries attach the server's
//! WebSocket handlers to [`tokio::sync::broadcast::Sender`] for [`UiEvent`] fan-out.

use std::net::SocketAddr;
use std::sync::Mutex;

use anyhow::Context;
use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::routing::get;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

use crate::{UiCommand, UiEvent};

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
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_WEB_PORT,
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
        Self { port }
    }
}

#[derive(Clone)]
struct Attached {
    event_tx: broadcast::Sender<UiEvent>,
    cmd_tx: mpsc::UnboundedSender<UiCommand>,
}

/// Localhost HTTP service (static SPA + `/ws`).
pub struct WebServer {
    listener: Mutex<Option<TcpListener>>,
    port: u16,
    attached: Mutex<Option<Attached>>,
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
        })
    }

    /// Connect UI channels: outbound [`UiEvent`] broadcast and inbound [`UiCommand`] sink.
    pub fn attach(
        &self,
        event_tx: broadcast::Sender<UiEvent>,
        cmd_tx: mpsc::UnboundedSender<UiCommand>,
    ) {
        self.attached
            .lock()
            .expect("web mutex")
            .replace(Attached { event_tx, cmd_tx });
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

        let routes = Router::new()
            .route("/", get(serve_index))
            .route("/assets/{name}", get(serve_asset))
            .route("/ws", get(ws_upgrade))
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
    ([(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    )], INDEX_HTML)
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
    ([(axum::http::header::CONTENT_TYPE, hv)], body).into_response()
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(attached): State<Attached>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_connected(socket, attached))
}

async fn ws_connected(socket: WebSocket, attached: Attached) {
    let mut recv = attached.event_tx.subscribe();
    let cmd_tx = attached.cmd_tx;
    let (mut sink, mut stream) = socket.split();
    loop {
        tokio::select! {
            ev = recv.recv() => {
                match ev {
                    Ok(ev) => {
                        let Ok(text) = serde_json::to_string(&ev) else { continue };
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
