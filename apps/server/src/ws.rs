//! Realtime WebSocket: `WS /api/ws?doc=doc:<id>&token=<JWT>`.
//!
//! One socket, message-framed by a 1-byte channel prefix:
//! - `0x00` + Yjs sync/awareness bytes (y-websocket protocol, `yrs`).
//! - `0x01` + JSON run event (server → client broadcast).
//! - `0x02` + one JSON-RPC 2.0 message (no Content-Length): the LSP bridge to
//!   a per-connection `hick-lsp` session (see `crate::lsp`), lazily started
//!   on the first 0x02 frame and shut down with the socket.
//!
//! The web client (apps/web/src/api/realtime.ts) processes exactly one
//! protocol message per frame, so the server never packs two messages into
//! one frame.
//!
//! The rooms themselves — the Yjs sync protocol, the document-doubling
//! defences, the debounced persist — live in `hickory-collab`, shared with
//! the local server (`hickory serve`). What stays here is what only the
//! hosted deployment has: JWT authorization, the `editors` entitlement, and
//! the LSP bridge.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use futures::{SinkExt as _, StreamExt as _};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::auth::verify_token;
use crate::error::ApiError;
use crate::routes::docs::{DocRow, load_doc};
use crate::{AppState, CHANNEL_LSP, CHANNEL_RUN, CHANNEL_YJS};

/// Tracks which authenticated users hold open sockets per account owner,
/// for the `editors` entitlement.
#[derive(Default)]
pub struct EditorTracker {
    inner: std::sync::Mutex<HashMap<Uuid, HashMap<Uuid, usize>>>,
}

impl EditorTracker {
    pub fn distinct_count(&self, owner: Uuid) -> usize {
        self.inner
            .lock()
            .unwrap()
            .get(&owner)
            .map(|m| m.len())
            .unwrap_or(0)
    }

    pub fn contains(&self, owner: Uuid, user: Uuid) -> bool {
        self.inner
            .lock()
            .unwrap()
            .get(&owner)
            .is_some_and(|m| m.contains_key(&user))
    }

    pub fn add(&self, owner: Uuid, user: Uuid) {
        *self
            .inner
            .lock()
            .unwrap()
            .entry(owner)
            .or_default()
            .entry(user)
            .or_insert(0) += 1;
    }

    pub fn remove(&self, owner: Uuid, user: Uuid) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(users) = inner.get_mut(&owner) {
            if let Some(n) = users.get_mut(&user) {
                *n -= 1;
                if *n == 0 {
                    users.remove(&user);
                }
            }
            if users.is_empty() {
                inner.remove(&owner);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct WsParams {
    doc: String,
    token: String,
}

/// Shared auth + entitlement gate for both the doc room and output room
/// channels: loads the doc, checks read/collab access, and enforces the
/// account's `editors` entitlement (distinct concurrent collaborators).
async fn authorize(
    state: &AppState,
    doc_id: Uuid,
    token: &str,
) -> Result<(DocRow, Uuid), ApiError> {
    let claims = verify_token(&state.config.jwt_secret, token)?;
    let doc = load_doc(state, doc_id).await?;
    let user = crate::auth::load_user(state, claims.sub).await?;
    let is_owner = doc.owner_id == user.id;
    if !is_owner && doc.visibility != "public" {
        return Err(ApiError::forbidden("no access to this document"));
    }

    if !state.editors.contains(doc.owner_id, user.id) {
        let owner = crate::auth::load_user(state, doc.owner_id).await?;
        let ents = crate::plans::resolve(
            &state.catalog,
            &owner.plan_key,
            owner.price_key.as_deref(),
            &owner.billing_status,
        );
        // Distinct *users* holding sockets on this account's docs — not
        // distinct rooms. The tracker owns that count; the room registry is
        // deliberately ignorant of accounts and entitlements.
        let current = state.editors.distinct_count(doc.owner_id) as u64;
        if current >= ents.editors {
            return Err(ApiError::forbidden(format!(
                "editor limit reached for this account's plan ({} editors) — upgrade to add collaborators",
                ents.editors
            )));
        }
    }

    Ok((doc, user.id))
}

pub async fn ws_handler(
    State(state): State<AppState>,
    Query(params): Query<WsParams>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    if let Some(rest) = params.doc.strip_prefix("output:") {
        // `output:<doc-uuid>:<path>` — the path may itself contain `:` only
        // if a doc path ever does (it can't, `GitStore::validate_path`
        // forbids it), so splitting on the first `:` is unambiguous.
        let (doc_id_str, output_path) = rest
            .split_once(':')
            .ok_or_else(|| ApiError::bad_request("output channel must be output:<uuid>:<path>"))?;
        let doc_id: Uuid = doc_id_str
            .parse()
            .map_err(|_| ApiError::bad_request("output channel must be output:<uuid>:<path>"))?;
        let (doc, user_id) = authorize(&state, doc_id, &params.token).await?;
        let output_path = output_path.to_string();
        return Ok(upgrade.on_upgrade(move |socket| async move {
            state.editors.add(doc.owner_id, user_id);
            let owner_id = doc.owner_id;
            let result =
                crate::output_rooms::run_output_socket(state.clone(), doc, output_path, socket)
                    .await;
            state.editors.remove(owner_id, user_id);
            if let Err(e) = result {
                log::debug!("output ws session ended: {e:#}");
            }
        }));
    }

    let doc_id: Uuid = params
        .doc
        .strip_prefix("doc:")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ApiError::bad_request("doc must be doc:<uuid> or output:<uuid>:<path>"))?;
    let (doc, user_id) = authorize(&state, doc_id, &params.token).await?;

    Ok(upgrade.on_upgrade(move |socket| async move {
        state.editors.add(doc.owner_id, user_id);
        let owner_id = doc.owner_id;
        let result = run_socket(state.clone(), doc, socket).await;
        state.editors.remove(owner_id, user_id);
        if let Err(e) = result {
            log::debug!("ws session ended: {e:#}");
        }
    }))
}

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

async fn run_socket(state: AppState, doc: DocRow, socket: WebSocket) -> anyhow::Result<()> {
    let key = doc.id.to_string();
    let room = state.rooms.get_or_create(&key).await?;
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    room.attach(client_id, tx.clone());

    let (mut sink, mut stream) = socket.split();

    // Per-connection LSP bridge, lazily started on the first 0x02 frame.
    let mut lsp_bridge: Option<LspBridge> = None;

    // Writer task: forward framed bytes to the socket.
    let writer = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(WsMessage::Binary(frame.into())).await.is_err() {
                break;
            }
        }
    });

    // Initial handshake: server sync-step-1, then current awareness state —
    // one protocol message per frame (see module docs).
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
                if let Err(e) = state
                    .rooms
                    // The hosted server decides read-vs-write in `authorize`
                    // (ownership and visibility), before the socket exists.
                    .handle_yjs_payload(&room, client_id, &tx, &data[1..], true)
                    .await
                {
                    log::debug!("yjs message error on doc {}: {e:#}", doc.id);
                    // A document past the size cap must not be persisted; the
                    // socket is closed instead (hickory-collab enforces it).
                    break;
                }
            }
            // Run channel is server → client only; ignore anything inbound.
            CHANNEL_RUN => {}
            CHANNEL_LSP => {
                handle_lsp_frame(&state, &doc, &mut lsp_bridge, &tx, &data[1..]).await;
            }
            _ => {}
        }
    }

    if let Some(bridge) = lsp_bridge.take() {
        bridge.shutdown().await;
    }
    room.detach(client_id);
    writer.abort();
    // Flush any pending edits before potentially dropping the room.
    state.rooms.persist_now(&room).await;
    state.rooms.drop_if_empty(&key).await;
    Ok(())
}

// ---------------------------------------------------------------------------
// LSP bridge (channel 0x02)
// ---------------------------------------------------------------------------

/// One hick-lsp session per (project, WS connection): the child process plus
/// the pump task that translates its output back into client URI space.
struct LspBridge {
    session: crate::lsp::LspSession,
    pump: tokio::task::JoinHandle<()>,
}

impl LspBridge {
    async fn start(
        state: &AppState,
        doc: &DocRow,
        tx: &mpsc::UnboundedSender<Vec<u8>>,
    ) -> anyhow::Result<LspBridge> {
        let (session, mut rx) = crate::lsp::LspSession::start(&state.git, doc.project_id).await?;
        let workdir_uri = session.workdir_uri().to_string();
        let pump_state = state.clone();
        let (doc_id, project_id) = (doc.id, doc.project_id);
        let pump_tx = tx.clone();
        let pump = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let msg = crate::lsp::rewrite_outbound(
                    &pump_state,
                    doc_id,
                    project_id,
                    &workdir_uri,
                    msg,
                )
                .await;
                if pump_tx.send(lsp_frame(&msg)).is_err() {
                    break;
                }
            }
        });
        Ok(LspBridge { session, pump })
    }

    async fn shutdown(self) {
        self.session.shutdown().await;
        self.pump.abort();
    }
}

fn lsp_frame(msg: &serde_json::Value) -> Vec<u8> {
    let json = serde_json::to_vec(msg).expect("serializable LSP message");
    let mut frame = Vec::with_capacity(json.len() + 1);
    frame.push(CHANNEL_LSP);
    frame.extend_from_slice(&json);
    frame
}

/// Handle one inbound 0x02 frame. The channel never carries transport errors:
/// when the session cannot start (hick-lsp binary missing) or the child dies,
/// client *requests* are answered with `result: null` and notifications are
/// dropped.
async fn handle_lsp_frame(
    state: &AppState,
    doc: &DocRow,
    bridge: &mut Option<LspBridge>,
    tx: &mpsc::UnboundedSender<Vec<u8>>,
    payload: &[u8],
) {
    let Ok(mut msg) = serde_json::from_slice::<serde_json::Value>(payload) else {
        log::debug!("dropping unparseable LSP frame on doc {}", doc.id);
        return;
    };
    let id = msg.get("id").cloned();
    let method = msg
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or_default()
        .to_string();

    // The server owns the LSP lifecycle: `initialize` is answered here (the
    // contract lets clients start straight at `didOpen`), and shutdown/exit
    // never reach the child — the session dies with the socket.
    match method.as_str() {
        "initialize" => {
            let _ = tx.send(lsp_frame(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": crate::lsp::bridge_initialize_result(),
            })));
            return;
        }
        "initialized" | "exit" => return,
        "shutdown" => {
            let _ = tx.send(lsp_frame(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": null,
            })));
            return;
        }
        _ => {}
    }

    if bridge.is_none() {
        match LspBridge::start(state, doc, tx).await {
            Ok(b) => *bridge = Some(b),
            Err(e) => {
                log::warn!("LSP session for doc {} unavailable: {e:#}", doc.id);
            }
        }
    }

    let sent = match bridge.as_mut() {
        Some(b) => {
            crate::lsp::rewrite_inbound(&mut msg, b.session.workdir_uri());
            b.session.send(&msg).await.is_ok()
        }
        None => false,
    };
    if !sent && let Some(id) = id {
        // Degrade, never error the channel: unanswerable requests get null.
        let _ = tx.send(lsp_frame(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": null,
        })));
    }
}
