//! Outbound relay client — connects the agent to `relay.hickory.dev` and bridges
//! [`UiEvent`] / [`UiCommand`] over a cloud WebSocket session.
//!
//! Enabled with the **`relay`** Cargo feature.

use anyhow::Context as _;
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::ui_event::{UiCommand, UiEvent, ui_event_wire_value};

/// Outbound WebSocket relay bridge.
///
/// The agent connects to `wss://relay.hickory.dev/agent/<session_id>` and forwards
/// all [`UiEvent`]s as JSON frames. Incoming frames are parsed as [`UiCommand`]s and
/// forwarded to the agent's command channel.
pub struct RelayClient {
    session_url: String,
}

impl RelayClient {
    /// Spawn the relay bridge and return a handle with [`session_url`](Self::session_url).
    ///
    /// `agent_ws_url` is `wss://relay.hickory.dev/agent/<session_id>`.
    /// `session_url` is the browser-facing URL (derived from `agent_ws_url`).
    pub async fn connect(
        agent_ws_url: &str,
        event_rx: broadcast::Receiver<UiEvent>,
        cmd_tx: mpsc::UnboundedSender<UiCommand>,
    ) -> anyhow::Result<Self> {
        let (ws, _resp) = connect_async(agent_ws_url)
            .await
            .with_context(|| format!("relay connect to {agent_ws_url}"))?;

        let session_url = agent_ws_url
            .replace("/agent/", "/session/")
            .replace("ws://", "http://")
            .replace("wss://", "https://");

        tokio::spawn(run_relay_loop(ws, event_rx, cmd_tx));

        Ok(Self { session_url })
    }

    /// Browser-facing session URL (share this with remote users).
    pub fn session_url(&self) -> &str {
        &self.session_url
    }
}

async fn run_relay_loop(
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    mut event_rx: broadcast::Receiver<UiEvent>,
    cmd_tx: mpsc::UnboundedSender<UiCommand>,
) {
    let (mut sink, mut stream) = ws.split();

    loop {
        tokio::select! {
            ev = event_rx.recv() => {
                match ev {
                    Ok(ev) => {
                        let Ok(text) = serde_json::to_string(&ui_event_wire_value(&ev)) else { continue };
                        if sink.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = stream.next() => {
                match msg {
                    None => break,
                    Some(Ok(Message::Text(t))) => {
                        if let Ok(cmd) = serde_json::from_str::<UiCommand>(&t) {
                            cmd_tx.send(cmd).ok();
                        }
                    }
                    Some(Ok(Message::Binary(b))) => {
                        if let Ok(cmd) = serde_json::from_slice::<UiCommand>(&b) {
                            cmd_tx.send(cmd).ok();
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
}
