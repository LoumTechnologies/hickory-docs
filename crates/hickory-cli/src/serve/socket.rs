//! The collaboration socket, local edition.
//!
//! The frame protocol is the hosted server's, because it is the same client:
//! `0x00` Yjs sync/awareness, `0x01` run events, `0x02` the LSP bridge.
//!
//! `0x02` used to go unanswered here, on the reasoning that a local session
//! had no per-project checkout to start child language servers against. That
//! stopped being true when the desktop app began opening a folder: the served
//! directory IS the checkout. It is now bridged to an in-process `hick-lsp`
//! (see [`super::lsp_bridge`]) — one per workspace, shared by every socket,
//! and started on the first `0x02` frame anybody sends, so a session that
//! never asks a language question never spawns a language server.
//!
//! `?doc=workspace` opens a socket with no document room at all: only the
//! language channel. That is how a plain file — `src/main.rs`, with no
//! document and so no room — gets the same language server the documents
//! have.
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

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize)]
pub struct WsParams {
    doc: String,
}

/// `WS /api/ws?doc=doc:<id>`, or `?doc=workspace` for the language channel
/// alone.
pub async fn ws_handler(
    State(state): State<LocalState>,
    Query(params): Query<WsParams>,
    upgrade: WebSocketUpgrade,
) -> Response {
    if params.doc == WORKSPACE_ROOM {
        return upgrade.on_upgrade(move |socket| async move {
            if let Err(e) = run_workspace_socket(state, socket).await {
                log::debug!("local workspace socket ended: {e:#}");
            }
        });
    }
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

/// The room name for a socket that carries no document: the language channel
/// for plain files, which have no room of their own.
const WORKSPACE_ROOM: &str = "workspace";

/// A socket with no document: the language channel and the debug channel,
/// and nothing else — no room, no edits. A plain file's pane asks its
/// language questions and runs its debugger over this one connection.
async fn run_workspace_socket(state: LocalState, socket: WebSocket) -> anyhow::Result<()> {
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let (mut sink, mut stream) = socket.split();
    let writer = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(WsMessage::Binary(frame.into())).await.is_err() {
                break;
            }
        }
    });
    let mut subscribed = false;
    let mut debuggers: Option<std::sync::Arc<crate::debug_sessions::Registry>> = None;
    while let Some(msg) = stream.next().await {
        let data = match msg {
            Ok(WsMessage::Binary(b)) => b.to_vec(),
            Ok(WsMessage::Text(t)) => t.as_bytes().to_vec(),
            Ok(WsMessage::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        };
        match data.first() {
            Some(&CHANNEL_LSP) => forward_lsp(
                &state,
                client_id,
                &tx,
                &mut subscribed,
                &data[1..],
                WORKSPACE_ROOM,
            ),
            Some(&CHANNEL_DEBUG) => {
                forward_debug(&state, &tx, &mut debuggers, &data, WORKSPACE_ROOM)
            }
            _ => continue,
        }
    }
    if subscribed {
        state.lsp.unsubscribe(client_id);
    }
    writer.abort();
    // The window is gone, so its debuggers go with it — a running program
    // must not outlive the tab that started it.
    if let Some(debuggers) = debuggers {
        debuggers.stop_all().await;
    }
    Ok(())
}

/// One `0x02` frame from `client_id`, into the shared session — subscribing
/// on the first one, so a window that never asks a language question never
/// starts a language server.
fn forward_lsp(
    state: &LocalState,
    client_id: u64,
    tx: &mpsc::UnboundedSender<Vec<u8>>,
    subscribed: &mut bool,
    payload: &[u8],
    key: &str,
) {
    if !*subscribed {
        match state.lsp.subscribe(client_id, tx.clone()) {
            Ok(()) => *subscribed = true,
            Err(e) => {
                // The editor degrades without the bridge, so a failure here
                // ends the language channel, never the session that carries
                // the user's edits.
                log::warn!("no language server session for {key}: {e:#}");
                return;
            }
        }
    }
    match serde_json::from_slice(payload) {
        Ok(message) => {
            if let Err(e) = state.lsp.send(client_id, message) {
                log::debug!("language server session ended on {key}: {e:#}");
            }
        }
        Err(e) => log::debug!("unparseable lsp frame on {key}: {e}"),
    }
}

/// One `0x03` frame into this connection's debuggers, starting the registry
/// on the first one.
///
/// One registry per connection: a debug session belongs to the window that
/// started it, and every one of them ends when that window goes away. The
/// document socket and the workspace socket share this, because a plain
/// file's pane debugs over the workspace socket exactly as it asks language
/// questions over it.
fn forward_debug(
    state: &LocalState,
    tx: &mpsc::UnboundedSender<Vec<u8>>,
    debuggers: &mut Option<std::sync::Arc<crate::debug_sessions::Registry>>,
    data: &[u8],
    key: &str,
) {
    let registry = debuggers
        .get_or_insert_with(|| std::sync::Arc::new(crate::debug_sessions::Registry::new()));
    match super::debug_bridge::request_of(data) {
        Ok(request) => {
            // Spawned rather than awaited: a `continue` can take as long as
            // the program does, and blocking here would freeze this window's
            // editing with it.
            let registry = registry.clone();
            let root = state.index.root().to_path_buf();
            let tx = tx.clone();
            tokio::spawn(async move {
                let responses = super::debug_bridge::handle(&registry, &root, request).await;
                super::debug_bridge::reply(&tx, responses);
            });
        }
        Err(e) => log::debug!("unparseable debug frame on {key}: {e}"),
    }
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

    // Subscribed on demand to the workspace's shared language session, and
    // unsubscribed with the connection — which closes what this window was
    // the last to hold open, and nothing else.
    let mut lsp_subscribed = false;
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
                forward_lsp(
                    &state,
                    client_id,
                    &tx,
                    &mut lsp_subscribed,
                    &data[1..],
                    &key,
                );
            }
            CHANNEL_DEBUG => forward_debug(&state, &tx, &mut debuggers, &data, &key),
            // Server → client only.
            CHANNEL_RUN => {}
            _ => {}
        }
    }

    room.detach(client_id);
    if lsp_subscribed {
        state.lsp.unsubscribe(client_id);
    }
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
