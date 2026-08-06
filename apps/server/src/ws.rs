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

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use futures::{SinkExt as _, StreamExt as _};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;
use yrs::sync::protocol::Protocol as _;
use yrs::sync::{Awareness, DefaultProtocol, Message, MessageReader, SyncMessage};
use yrs::updates::decoder::{Decode as _, DecoderV1};
use yrs::updates::encoder::Encode as _;
use yrs::{Doc, GetString as _, ReadTxn as _, Text as _, Transact as _, Update};

use crate::auth::verify_token;
use crate::error::ApiError;
use crate::routes::docs::{DocRow, load_doc};
use crate::{AppState, CHANNEL_LSP, CHANNEL_RUN, CHANNEL_YJS};

// ---------------------------------------------------------------------------
// Rooms
// ---------------------------------------------------------------------------

pub struct Room {
    doc_id: Uuid,
    project_id: Uuid,
    doc_path: String,
    owner_id: Uuid,
    awareness: tokio::sync::Mutex<Awareness>,
    clients: std::sync::Mutex<HashMap<u64, mpsc::UnboundedSender<Vec<u8>>>>,
    /// Bumped on every doc mutation; drives the debounced persist.
    generation: AtomicU64,
    /// Last generation that was persisted.
    persisted: AtomicU64,
}

impl Room {
    fn broadcast(&self, frame: &[u8], except: Option<u64>) {
        let clients = self.clients.lock().unwrap();
        for (id, tx) in clients.iter() {
            if Some(*id) != except {
                let _ = tx.send(frame.to_vec());
            }
        }
    }

    fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }

    /// Current text plus the encoded CRDT state, taken under one lock so the
    /// two can never describe different revisions.
    async fn snapshot(&self) -> (String, Vec<u8>) {
        let awareness = self.awareness.lock().await;
        let doc = awareness.doc();
        let text = doc.get_or_insert_text("source");
        let txn = doc.transact();
        (
            text.get_string(&txn),
            txn.encode_state_as_update_v1(&yrs::StateVector::default()),
        )
    }
}

/// Build the Y.Doc backing a room.
///
/// This is the fix for the document-doubling bug (migration 0003). Two rules
/// hold it together:
///
/// 1. **Resume, never re-seed.** When `docs.crdt_state` exists it is applied
///    as-is, so a re-created room continues the same operation history a
///    still-connected (or reconnecting) client already has. Seeding a fresh
///    `Doc` per room minted rival operations that the CRDT merged by
///    concatenation — every reconnect appended the whole document to itself.
/// 2. **Stable client id.** Even without persisted state (a doc created
///    before migration 0003, or one whose state failed to store), the seed
///    operation is minted under a client id derived from the doc id, so two
///    independent seeds of the same source are the *same* operation and
///    dedupe instead of concatenating.
///
/// If `docs.source` moved on out of band — a lineage edit, a run, a REST
/// save — the difference is applied to the restored document as ordinary
/// CRDT edits (common prefix/suffix preserved, so concurrent edits elsewhere
/// in the file survive), rather than by starting over.
async fn build_room_doc(state: &AppState, doc: &DocRow) -> Doc {
    let stored: Option<Vec<u8>> = sqlx::query_scalar("SELECT crdt_state FROM docs WHERE id = $1")
        .bind(doc.id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or_else(|e| {
            log::error!("reading crdt_state for doc {} failed: {e}", doc.id);
            None
        })
        .flatten();

    let ydoc = Doc::with_client_id(stable_client_id(doc.id));
    let text = ydoc.get_or_insert_text("source");

    if let Some(bytes) = stored {
        match Update::decode_v1(&bytes) {
            Ok(update) => {
                let mut txn = ydoc.transact_mut();
                if let Err(e) = txn.apply_update(update) {
                    log::error!("applying stored crdt_state for doc {} failed: {e}", doc.id);
                }
            }
            Err(e) => log::error!("stored crdt_state for doc {} is unreadable: {e}", doc.id),
        }
    }

    let current = {
        let txn = ydoc.transact();
        text.get_string(&txn)
    };
    if current != doc.source {
        let mut txn = ydoc.transact_mut();
        let (start, del_len, insert) = text_delta(&current, &doc.source);
        if del_len > 0 {
            text.remove_range(&mut txn, start as u32, del_len as u32);
        }
        if !insert.is_empty() {
            text.insert(&mut txn, start as u32, insert);
        }
    }
    ydoc
}

/// Yjs client id for the server's own operations on a document. Derived from
/// the doc id so it is identical across restarts and room re-creations.
fn stable_client_id(doc_id: Uuid) -> u64 {
    let b = doc_id.as_bytes();
    // MUST stay inside JavaScript's safe-integer range. yrs stores client ids
    // as u64, but the browser decodes them into JS numbers: a value above
    // 2^53 makes Yjs throw "Integer out of Range" and drop the whole update,
    // so the client silently never receives the document. Yjs itself mints
    // 32-bit ids, so 32 bits is both safe and conventional — still stable
    // across restarts and distinct per document, which is all this needs.
    u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as u64
}

/// Minimal replace turning `current` into `target`, in UTF-16 code units
/// (Yjs text indices): `(offset, units_to_delete, text_to_insert)`.
fn text_delta<'a>(current: &str, target: &'a str) -> (usize, usize, &'a str) {
    let prefix_bytes = current
        .char_indices()
        .zip(target.char_indices())
        .take_while(|((_, a), (_, b))| a == b)
        .last()
        .map(|((i, c), _)| i + c.len_utf8())
        .unwrap_or(0);
    // Longest common suffix, not overlapping the shared prefix.
    let max_suffix = (current.len() - prefix_bytes).min(target.len() - prefix_bytes);
    let mut suffix_bytes = 0;
    while suffix_bytes < max_suffix {
        let ca = current[..current.len() - suffix_bytes].chars().next_back();
        let cb = target[..target.len() - suffix_bytes].chars().next_back();
        match (ca, cb) {
            (Some(a), Some(b)) if a == b && suffix_bytes + a.len_utf8() <= max_suffix => {
                suffix_bytes += a.len_utf8();
            }
            _ => break,
        }
    }
    let offset = current[..prefix_bytes].encode_utf16().count();
    let del_len = current[prefix_bytes..current.len() - suffix_bytes]
        .encode_utf16()
        .count();
    (
        offset,
        del_len,
        &target[prefix_bytes..target.len() - suffix_bytes],
    )
}

#[derive(Default)]
pub struct RoomRegistry {
    rooms: tokio::sync::Mutex<HashMap<Uuid, Arc<Room>>>,
}

impl RoomRegistry {
    /// Broadcast a run-channel JSON payload to every socket on a doc.
    pub async fn publish_run_event(&self, doc_id: Uuid, payload: &serde_json::Value) {
        let room = { self.rooms.lock().await.get(&doc_id).cloned() };
        if let Some(room) = room {
            let json = serde_json::to_vec(payload).expect("serializable run event");
            let mut frame = Vec::with_capacity(json.len() + 1);
            frame.push(CHANNEL_RUN);
            frame.extend_from_slice(&json);
            room.broadcast(&frame, None);
        }
    }

    async fn get_or_create(&self, state: &AppState, doc: &DocRow) -> Arc<Room> {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(&doc.id) {
            return room.clone();
        }
        let ydoc = build_room_doc(state, doc).await;
        let room = Arc::new(Room {
            doc_id: doc.id,
            project_id: doc.project_id,
            doc_path: doc.path.clone(),
            owner_id: doc.owner_id,
            awareness: tokio::sync::Mutex::new(Awareness::new(ydoc)),
            clients: std::sync::Mutex::new(HashMap::new()),
            generation: AtomicU64::new(0),
            persisted: AtomicU64::new(0),
        });
        rooms.insert(doc.id, room.clone());
        room
    }

    async fn drop_if_empty(&self, doc_id: Uuid) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(&doc_id)
            && room.client_count() == 0
        {
            rooms.remove(&doc_id);
        }
    }

    /// Distinct users currently connected to docs owned by `owner`.
    async fn distinct_editors(&self, owner: Uuid, editor_ids: &EditorTracker) -> usize {
        let _ = self;
        editor_ids.distinct_count(owner)
    }
}

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

pub async fn ws_handler(
    State(state): State<AppState>,
    Query(params): Query<WsParams>,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    let claims = verify_token(&state.config.jwt_secret, &params.token)?;
    let doc_id: Uuid = params
        .doc
        .strip_prefix("doc:")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ApiError::bad_request("doc must be doc:<uuid>"))?;

    let doc = load_doc(&state, doc_id).await?;
    let user = crate::auth::load_user(&state, claims.sub).await?;
    let is_owner = doc.owner_id == user.id;
    if !is_owner && doc.visibility != "public" {
        return Err(ApiError::forbidden("no access to this document"));
    }

    // Editors entitlement: distinct concurrent collaborators on this
    // account's docs (the owner always counts as one of them).
    if !state.editors.contains(doc.owner_id, user.id) {
        let owner = crate::auth::load_user(&state, doc.owner_id).await?;
        let ents = crate::plans::resolve(
            &state.catalog,
            &owner.plan_key,
            owner.price_key.as_deref(),
            &owner.billing_status,
        );
        let current = state
            .rooms
            .distinct_editors(doc.owner_id, &state.editors)
            .await as u64;
        if current >= ents.editors {
            return Err(ApiError::forbidden(format!(
                "editor limit reached for this account's plan ({} editors) — upgrade to add collaborators",
                ents.editors
            )));
        }
    }

    let user_id = user.id;
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
    let doc_id = doc.id;
    let room = state.rooms.get_or_create(&state, &doc).await;
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    room.clients.lock().unwrap().insert(client_id, tx.clone());

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
    {
        let awareness = room.awareness.lock().await;
        let sv = awareness.doc().transact().state_vector();
        send_yjs(&tx, &Message::Sync(SyncMessage::SyncStep1(sv)));
        if let Ok(update) = awareness.update() {
            send_yjs(&tx, &Message::Awareness(update));
        }
    }

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
                if let Err(e) = handle_yjs_payload(&state, &room, client_id, &tx, &data[1..]).await
                {
                    log::debug!("yjs message error on doc {doc_id}: {e:#}");
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
    room.clients.lock().unwrap().remove(&client_id);
    writer.abort();
    // Flush any pending edits before potentially dropping the room.
    persist_now(&state, &room).await;
    state.rooms.drop_if_empty(doc_id).await;
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

fn send_yjs(tx: &mpsc::UnboundedSender<Vec<u8>>, msg: &Message) {
    let payload = msg.encode_v1();
    let mut frame = Vec::with_capacity(payload.len() + 1);
    frame.push(CHANNEL_YJS);
    frame.extend_from_slice(&payload);
    let _ = tx.send(frame);
}

fn yjs_frame(msg: &Message) -> Vec<u8> {
    let payload = msg.encode_v1();
    let mut frame = Vec::with_capacity(payload.len() + 1);
    frame.push(CHANNEL_YJS);
    frame.extend_from_slice(&payload);
    frame
}

async fn handle_yjs_payload(
    state: &AppState,
    room: &Arc<Room>,
    client_id: u64,
    tx: &mpsc::UnboundedSender<Vec<u8>>,
    payload: &[u8],
) -> anyhow::Result<()> {
    let protocol = DefaultProtocol;
    let mut decoder = DecoderV1::new(yrs::encoding::read::Cursor::new(payload));
    let mut reader = MessageReader::new(&mut decoder);
    // Collect first: MessageReader borrows the decoder.
    let messages: Vec<Message> = reader.by_ref().collect::<Result<_, _>>()?;
    let mut mutated = false;
    {
        let awareness = room.awareness.lock().await;
        for message in messages {
            match message {
                Message::Sync(SyncMessage::SyncStep1(sv)) => {
                    if let Some(reply) = protocol.handle_sync_step1(&awareness, sv)? {
                        let _ = tx.send(yjs_frame(&reply));
                    }
                }
                Message::Sync(SyncMessage::SyncStep2(bytes))
                | Message::Sync(SyncMessage::Update(bytes)) => {
                    let update = Update::decode_v1(&bytes)?;
                    protocol.handle_update(&awareness, update)?;
                    mutated = true;
                    room.broadcast(
                        &yjs_frame(&Message::Sync(SyncMessage::Update(bytes))),
                        Some(client_id),
                    );
                }
                Message::Awareness(update) => {
                    protocol.handle_awareness_update(&awareness, update.clone())?;
                    room.broadcast(&yjs_frame(&Message::Awareness(update)), Some(client_id));
                }
                Message::AwarenessQuery => {
                    if let Some(reply) = protocol.handle_awareness_query(&awareness)? {
                        let _ = tx.send(yjs_frame(&reply));
                    }
                }
                Message::Auth(_) | Message::Custom(..) => {}
            }
        }
    }
    if mutated {
        schedule_persist(state.clone(), room.clone());
    }
    Ok(())
}

/// Debounced persistence: wait for edits to quiesce (750 ms), then write the
/// CRDT text back to the doc row and commit it to the project git repo.
fn schedule_persist(state: AppState, room: Arc<Room>) {
    let generation = room.generation.fetch_add(1, Ordering::SeqCst) + 1;
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(750)).await;
        if room.generation.load(Ordering::SeqCst) == generation {
            persist_now(&state, &room).await;
        }
    });
}

async fn persist_now(state: &AppState, room: &Arc<Room>) {
    let generation = room.generation.load(Ordering::SeqCst);
    if room.persisted.swap(generation, Ordering::SeqCst) == generation {
        return; // nothing new since the last persist
    }
    // The CRDT state is stored with the text so a re-created room resumes
    // this operation history instead of minting a rival one (migration 0003).
    let (source, crdt_state) = room.snapshot().await;
    if let Err(e) = sqlx::query(
        "UPDATE docs SET source = $1, crdt_state = $2, updated_at = now() WHERE id = $3",
    )
    .bind(&source)
    .bind(&crdt_state)
    .bind(room.doc_id)
    .execute(&state.db)
    .await
    {
        log::error!("persisting doc {} failed: {e}", room.doc_id);
        return;
    }
    if let Err(e) = state
        .git
        .save_file(
            room.project_id,
            &room.doc_path,
            &source,
            &format!("Collaborative edit: {}", room.doc_path),
        )
        .await
    {
        log::error!("git persist for doc {} failed: {e:#}", room.doc_id);
    }
    let _ = room.owner_id;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// JavaScript's `Number.MAX_SAFE_INTEGER`. Yjs decodes CRDT client ids
    /// into JS numbers, so any id at or above this bound makes the browser
    /// throw "Integer out of Range" and DISCARD the whole update — the
    /// client silently never receives the document, and (before the client
    /// stopped seeding) fell back to inserting its own copy, which the CRDT
    /// merged by concatenation. Measured symptom: a 15 KB document reached
    /// 49 MB, and rendering it took 7.8 s.
    const JS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

    #[test]
    fn stable_client_id_stays_javascript_safe() {
        for _ in 0..1000 {
            let id = stable_client_id(Uuid::new_v4());
            assert!(
                id < JS_MAX_SAFE_INTEGER,
                "client id {id} exceeds JS safe-integer range; Yjs would reject the update"
            );
        }
    }

    #[test]
    fn stable_client_id_is_stable_and_distinct() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_eq!(stable_client_id(a), stable_client_id(a));
        assert_ne!(stable_client_id(a), stable_client_id(b));
    }
}
