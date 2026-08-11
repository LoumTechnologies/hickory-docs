//! The collaboration socket, local edition.
//!
//! The frame protocol is the hosted server's, because it is the same client:
//! `0x00` Yjs sync/awareness, `0x01` run events. `0x02` (the LSP bridge) is
//! not served here — a local session has no per-project checkout to start
//! child language servers against, and answering the channel with silence is
//! better than pretending: the client already degrades when the bridge is
//! unavailable.
//!
//! The room machinery is `hickory-collab`, shared with the hosted server. What
//! differs is only the gate in front of it.

use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use futures::{SinkExt as _, StreamExt as _};
use hickory_collab::{CHANNEL_RUN, CHANNEL_YJS};
use serde::Deserialize;
use tokio::sync::mpsc;

use super::LocalState;
use super::api::ApiError;
use super::share::Caller;

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize)]
pub struct WsParams {
    doc: String,
    token: String,
}

/// `WS /api/ws?doc=doc:<id>&token=<session token>`.
pub async fn ws_handler(
    State(state): State<LocalState>,
    Query(params): Query<WsParams>,
    upgrade: WebSocketUpgrade,
) -> Response {
    // The socket carries its token in the query string: a WebSocket handshake
    // from a browser cannot set an Authorization header. A mismatch is a flat
    // refusal with no hint about which part was wrong.
    let Some(caller) = state.caller_for(&params.token) else {
        return ApiError::forbidden("this session link is not valid").into_response();
    };

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
        if let Err(e) = run_socket(state, caller, doc_id, socket).await {
            log::debug!("local ws session ended: {e:#}");
        }
    })
}

async fn run_socket(
    state: LocalState,
    caller: Caller,
    key: String,
    socket: WebSocket,
) -> anyhow::Result<()> {
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
                    .handle_yjs_payload(&room, client_id, &tx, &data[1..], caller.can_edit())
                    .await
                {
                    log::debug!("yjs message error on {key}: {e:#}");
                    break;
                }
            }
            // Server → client only.
            CHANNEL_RUN => {}
            _ => {}
        }
    }

    room.detach(client_id);
    writer.abort();
    // Flush before the room can be dropped: an edit that never reached the
    // file is an edit the collaborator watched happen and then lost.
    state.rooms.persist_now(&room).await;
    state.rooms.drop_if_empty(&key).await;
    Ok(())
}
