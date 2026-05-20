//! hick-relay: cloud WebSocket relay for hick-agent remote UI sessions.
//!
//! Agents connect to `ws://<host>/agent/<session_id>` and browsers connect to
//! `ws://<host>/session/<session_id>`. The relay forwards bytes in both
//! directions without parsing them — it is content-agnostic.
//!
//! A session is created on the first connection (agent or browser) and
//! destroyed when both sides have disconnected.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

#[derive(Parser)]
#[command(about = "hick-relay: cloud WebSocket relay for hick-agent sessions")]
struct Cli {
    #[arg(long, env = "RELAY_BIND", default_value = "0.0.0.0:8080")]
    bind: SocketAddr,
}

// ---------------------------------------------------------------------------
// Session registry
// ---------------------------------------------------------------------------

/// A relay session: one agent ↔ many browser viewers.
struct Session {
    /// Broadcast channel carrying raw text frames from the agent to all browsers.
    agent_tx: broadcast::Sender<String>,
    /// Sender for browser→agent messages (relayed back to agent via its WS).
    browser_tx: broadcast::Sender<String>,
}

impl Session {
    fn new() -> Self {
        let (agent_tx, _) = broadcast::channel(512);
        let (browser_tx, _) = broadcast::channel(128);
        Self { agent_tx, browser_tx }
    }
}

type Registry = Arc<Mutex<HashMap<String, Arc<Session>>>>;

fn get_or_create(reg: &mut HashMap<String, Arc<Session>>, id: &str) -> Arc<Session> {
    reg.entry(id.to_string()).or_insert_with(|| Arc::new(Session::new())).clone()
}

// ---------------------------------------------------------------------------
// Agent WebSocket handler
// ---------------------------------------------------------------------------

async fn agent_ws(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(registry): State<Registry>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_agent(socket, session_id, registry))
}

async fn handle_agent(socket: WebSocket, session_id: String, registry: Registry) {
    let session = {
        let mut reg = registry.lock().await;
        get_or_create(&mut reg, &session_id)
    };

    let agent_tx = session.agent_tx.clone();
    let mut browser_rx = session.browser_tx.subscribe();
    let (mut sink, mut stream) = socket.split();

    loop {
        tokio::select! {
            msg = stream.next() => {
                match msg {
                    None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Text(t))) => { agent_tx.send(t.to_string()).ok(); }
                    Some(Ok(Message::Binary(b))) => {
                        if let Ok(s) = String::from_utf8(b.to_vec()) {
                            agent_tx.send(s).ok();
                        }
                    }
                    _ => {}
                }
            }
            cmd = browser_rx.recv() => {
                match cmd {
                    Ok(text) => {
                        if sink.send(Message::Text(text.into())).await.is_err() { break; }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    // Clean up if both sides are gone.
    let mut reg = registry.lock().await;
    let empty = reg.get(&session_id).is_some_and(|s| {
        s.agent_tx.receiver_count() == 0 && s.browser_tx.receiver_count() == 0
    });
    if empty {
        reg.remove(&session_id);
    }
}

// ---------------------------------------------------------------------------
// Browser WebSocket handler
// ---------------------------------------------------------------------------

async fn session_ws(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(registry): State<Registry>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_browser(socket, session_id, registry))
}

async fn handle_browser(socket: WebSocket, session_id: String, registry: Registry) {
    let session = {
        let mut reg = registry.lock().await;
        get_or_create(&mut reg, &session_id)
    };

    let browser_tx = session.browser_tx.clone();
    let mut agent_rx = session.agent_tx.subscribe();
    let (mut sink, mut stream) = socket.split();

    loop {
        tokio::select! {
            ev = agent_rx.recv() => {
                match ev {
                    Ok(text) => {
                        if sink.send(Message::Text(text.into())).await.is_err() { break; }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = stream.next() => {
                match msg {
                    None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Text(t))) => { browser_tx.send(t.to_string()).ok(); }
                    Some(Ok(Message::Binary(b))) => {
                        if let Ok(s) = String::from_utf8(b.to_vec()) {
                            browser_tx.send(s).ok();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// HTTP: session redirect (browser convenience endpoint)
// ---------------------------------------------------------------------------

async fn new_session() -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    axum::response::Redirect::to(&format!("/session/{id}"))
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let registry: Registry = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/agent/{session_id}", axum::routing::get(agent_ws))
        .route("/session/{session_id}", axum::routing::get(session_ws))
        .route("/new", axum::routing::get(new_session))
        .with_state(registry);

    log::info!("hick-relay listening on {}", cli.bind);
    let listener = tokio::net::TcpListener::bind(cli.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
