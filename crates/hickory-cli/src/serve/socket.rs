//! The collaboration socket, local edition.
//!
//! The frame protocol is the hosted server's, because it is the same client:
//! `0x00` Yjs sync/awareness, `0x01` run events, `0x02` the LSP bridge.
//!
//! `0x02` used to go unanswered here, on the reasoning that a local session
//! had no per-project checkout to start child language servers against. That
//! stopped being true when the desktop app began opening a folder: the served
//! directory IS the checkout. It is now bridged to an in-process `hick-lsp`
//! (see [`super::lsp_bridge`]), started on the connection's first `0x02`
//! frame so a session that never asks a language question never spawns a
//! language server.
//!
//! The room machinery is `hickory-collab`, shared with the hosted server. What
//! differs is only the gate in front of it.

use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use futures::{SinkExt as _, StreamExt as _};
use hickory_collab::{CHANNEL_DEBUG, CHANNEL_LSP, CHANNEL_RUN, CHANNEL_YJS};
use serde::Deserialize;
use tokio::sync::mpsc;

use super::LocalState;
use super::api::ApiError;
use super::lsp_bridge::LspBridge;

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize)]
pub struct WsParams {
    doc: String,
}

/// `WS /api/ws?doc=doc:<id>`.
pub async fn ws_handler(
    State(state): State<LocalState>,
    Query(params): Query<WsParams>,
    upgrade: WebSocketUpgrade,
) -> Response {
    // Output rooms (`output:<id>:<path>`) are a hosted feature: locally the
    // generated files are on disk, and the editor edits them through
    // `/outputs/edit`, which resolves into the document.
    let Some(doc_id) = params.doc.strip_prefix("doc:") else {
        return ApiError::bad_request(
            "doc must be doc:<id> — live output rooms are not part of a local session",
        )
        .into_response();
    };
    let doc_id = doc_id.to_string();

    if state.index.path_of(&doc_id).is_none() {
        return ApiError::not_found(format!("no document {doc_id} in this session"))
            .into_response();
    }

    upgrade.on_upgrade(move |socket| async move {
        if let Err(e) = run_socket(state, doc_id, socket).await {
            log::debug!("local ws session ended: {e:#}");
        }
    })
}

async fn run_socket(state: LocalState, key: String, socket: WebSocket) -> anyhow::Result<()> {
    let room = state.rooms.get_or_create(&key).await?;
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    room.attach(client_id, tx.clone());

    let (mut sink, mut stream) = socket.split();
    let writer = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(WsMessage::Binary(frame.into())).await.is_err() {
                break;
            }
        }
    });

    room.handshake(&tx).await;

    // Started on demand and dropped with the connection, which takes the
    // child language servers with it.
    let mut lsp: Option<LspBridge> = None;
    // The same rule for debuggers, and it matters more: a debug session holds
    // a running program and a scratch directory, so one that outlived the
    // window would leak both.
    let mut debuggers: Option<std::sync::Arc<crate::debug_sessions::Registry>> = None;

    while let Some(msg) = stream.next().await {
        let data = match msg {
            Ok(WsMessage::Binary(b)) => b.to_vec(),
            Ok(WsMessage::Text(t)) => t.as_bytes().to_vec(),
            Ok(WsMessage::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        };
        if data.is_empty() {
            continue;
        }
        match data[0] {
            CHANNEL_YJS => {
                // A read-only link still reads: the same channel carries the
                // client's request for the document. The room drops only the
                // messages that would change it.
                if let Err(e) = state
                    .rooms
                    .handle_yjs_payload(&room, client_id, &tx, &data[1..], true)
                    .await
                {
                    log::debug!("yjs message error on {key}: {e:#}");
                    break;
                }
            }
            CHANNEL_LSP => {
                let bridge = match lsp.as_ref() {
                    Some(bridge) => bridge,
                    None => match LspBridge::start(state.index.root(), tx.clone()) {
                        Ok(started) => lsp.insert(started),
                        Err(e) => {
                            // The editor degrades without the bridge, so a
                            // failure here ends the language channel, never
                            // the session that carries the user's edits.
                            log::warn!("no language server session for {key}: {e:#}");
                            continue;
                        }
                    },
                };
                match serde_json::from_slice(&data[1..]) {
                    Ok(message) => {
                        if let Err(e) = bridge.send(message) {
                            log::debug!("language server session ended on {key}: {e:#}");
                            lsp = None;
                        }
                    }
                    Err(e) => log::debug!("unparseable lsp frame on {key}: {e}"),
                }
            }
            CHANNEL_DEBUG => {
                // One registry per connection: a debug session belongs to the
                // window that started it, and every one of them ends when
                // that window goes away.
                let registry = debuggers.get_or_insert_with(|| {
                    std::sync::Arc::new(crate::debug_sessions::Registry::new())
                });
                match super::debug_bridge::request_of(&data) {
                    Ok(request) => {
                        // Spawned rather than awaited: a `continue` can take
                        // as long as the program does, and blocking here
                        // would freeze this window's editing with it.
                        let registry = registry.clone();
                        let root = state.index.root().to_path_buf();
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let responses =
                                super::debug_bridge::handle(&registry, &root, request).await;
                            super::debug_bridge::reply(&tx, responses);
                        });
                    }
                    Err(e) => log::debug!("unparseable debug frame on {key}: {e}"),
                }
            }
            // Server → client only.
            CHANNEL_RUN => {}
            _ => {}
        }
    }

    room.detach(client_id);
    writer.abort();
    // The window is gone, so its debuggers go with it. Each holds a running
    // program and a scratch directory; leaking either would mean a process
    // still executing somebody's code after they closed the tab.
    if let Some(debuggers) = debuggers {
        debuggers.stop_all().await;
    }
    // Flush before the room can be dropped: an edit that never reached the
    // file is an edit the collaborator watched happen and then lost.
    state.rooms.persist_now(&room).await;
    state.rooms.drop_if_empty(&key).await;
    Ok(())
}
