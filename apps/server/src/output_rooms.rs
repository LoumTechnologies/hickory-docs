//! Live collaborative editing for generated OUTPUT files.
//!
//! One Yjs room per `(doc_id, output_path)`, symmetric to `crate::ws`'s
//! per-document `Room` — same wire protocol (`WS /api/ws?doc=output:<doc-uuid>:<path>`),
//! same diff-and-patch reconciliation primitive (`ws::text_delta`/`byte_delta`,
//! never a wholesale replace), same debounce-coalescing shape.
//!
//! The one thing that's genuinely different from a doc room: an output
//! file's content is *derived* — the pipeline regenerates it on every run —
//! so a room here reconciles in two directions instead of one:
//!
//! 1. **Live edits → source**, debounced (`schedule_persist` below), by
//!    calling the same `apply_output_edits` the REST endpoint
//!    (`routes::outputs::edit_outputs`) uses — a live room's debounced apply
//!    and a one-shot manual apply are two callers of one function.
//! 2. **A re-run → the room** (`reconcile_rerun`, called from `runs.rs`
//!    after a run persists fresh `run_outputs`), diff-patched into any open
//!    room exactly like `ws::RoomRegistry::apply_external_source` reconciles
//!    a doc room, so a live collaborator's in-progress edit survives a
//!    re-run instead of being stomped.
//!
//! Both directions read/write a `baseline`: the text the room's live buffer
//! is currently a diff against. It must advance after EITHER direction
//! succeeds, or the next debounce tick re-diffs against a stale baseline and
//! either double-applies an old edit or silently drops a new one.
//!
//! Deliberately NOT persisted: no `crdt_state` column, unlike doc rooms. An
//! output room's authoritative content is always reproducible by re-running
//! the pipeline, and the debounce that resolves live edits into source is
//! short (750ms) — a room dropped between two debounce ticks can lose a few
//! in-flight keystrokes that were never resolved into source, which is a
//! real but narrow gap (the source document, which IS durable, is never at
//! risk).

use std::collections::HashMap;
use std::hash::{Hash as _, Hasher as _};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures::{SinkExt as _, StreamExt as _};
use tokio::sync::mpsc;
use uuid::Uuid;
use yrs::sync::protocol::Protocol as _;
use yrs::sync::{Awareness, DefaultProtocol, Message, MessageReader, SyncMessage};
use yrs::updates::decoder::{Decode as _, DecoderV1};
use yrs::{GetString as _, ReadTxn as _, Text as _, Transact as _, Update};

use crate::AppState;
use crate::error::ApiError;
use crate::routes::docs::DocRow;
use crate::routes::outputs::{apply_output_edits, load_output};
use hickory_collab::{byte_delta, doc_with_client_id, send_yjs, text_delta, yjs_frame};

pub struct OutputRoom {
    doc_id: Uuid,
    output_path: String,
    #[allow(dead_code)] // kept for parity with ws::Room / future entitlement use
    project_id: Uuid,
    #[allow(dead_code)]
    owner_id: Uuid,
    awareness: tokio::sync::Mutex<Awareness>,
    baseline: tokio::sync::Mutex<String>,
    clients: std::sync::Mutex<HashMap<u64, mpsc::UnboundedSender<Vec<u8>>>>,
    generation: AtomicU64,
    persisted: AtomicU64,
}

impl OutputRoom {
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

    async fn current_text(&self) -> String {
        let awareness = self.awareness.lock().await;
        let doc = awareness.doc();
        let text = doc.get_or_insert_text("source");
        let txn = doc.transact();
        text.get_string(&txn)
    }
}

/// Client id derivation for an output room. Distinct input from
/// `ws::stable_client_id` (keyed by a bare doc `Uuid`) since a room here is
/// keyed by `(doc_id, path)` — two different rooms on the same doc must
/// never mint operations under the same id.
fn stable_output_client_id(doc_id: Uuid, path: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    doc_id.hash(&mut hasher);
    path.hash(&mut hasher);
    // Same JS-safe-integer constraint as ws::stable_client_id: truncate to 32
    // bits so the browser never decodes an id above Number.MAX_SAFE_INTEGER.
    (hasher.finish() as u32) as u64
}

#[derive(Default)]
pub struct OutputRoomRegistry {
    rooms: tokio::sync::Mutex<HashMap<(Uuid, String), Arc<OutputRoom>>>,
}

impl OutputRoomRegistry {
    async fn get_or_create(
        &self,
        state: &AppState,
        doc: &DocRow,
        output_path: &str,
    ) -> Result<Arc<OutputRoom>, ApiError> {
        let key = (doc.id, output_path.to_string());
        if let Some(room) = self.rooms.lock().await.get(&key) {
            return Ok(room.clone());
        }
        // Outside the lock: a DB read, no need to serialize other rooms
        // behind it.
        let row = load_output(state, doc.id, output_path).await?;
        let ydoc = doc_with_client_id(stable_output_client_id(doc.id, output_path));
        {
            let text = ydoc.get_or_insert_text("source");
            let mut txn = ydoc.transact_mut();
            if !row.content.is_empty() {
                text.insert(&mut txn, 0, &row.content);
            }
        }
        let room = Arc::new(OutputRoom {
            doc_id: doc.id,
            output_path: output_path.to_string(),
            project_id: doc.project_id,
            owner_id: doc.owner_id,
            awareness: tokio::sync::Mutex::new(Awareness::new(ydoc)),
            baseline: tokio::sync::Mutex::new(row.content),
            clients: std::sync::Mutex::new(HashMap::new()),
            generation: AtomicU64::new(0),
            persisted: AtomicU64::new(0),
        });
        // Another connection may have created the room while this one
        // awaited `load_output` above; keep whichever won the race.
        let mut rooms = self.rooms.lock().await;
        Ok(rooms.entry(key).or_insert(room).clone())
    }

    async fn drop_if_empty(&self, doc_id: Uuid, output_path: &str) {
        let key = (doc_id, output_path.to_string());
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(&key)
            && room.client_count() == 0
        {
            rooms.remove(&key);
        }
    }

    /// Reconcile freshly woven output content into a live room, if one is
    /// open for `(doc_id, output_path)` — called from `runs.rs` right after
    /// a run persists `run_outputs`. No-ops if no room is open (mirrors
    /// `ws::RoomRegistry::apply_external_source`).
    pub async fn reconcile_rerun(&self, doc_id: Uuid, output_path: &str, new_content: &str) {
        let room = {
            let rooms = self.rooms.lock().await;
            rooms.get(&(doc_id, output_path.to_string())).cloned()
        };
        let Some(room) = room else { return };

        let awareness = room.awareness.lock().await;
        let ydoc = awareness.doc();
        let text = ydoc.get_or_insert_text("source");
        let current = {
            let txn = ydoc.transact();
            text.get_string(&txn)
        };
        if current != new_content {
            let update = {
                let mut txn = ydoc.transact_mut();
                let before = txn.state_vector();
                let (start, del_len, insert) = text_delta(&current, new_content);
                if del_len > 0 {
                    text.remove_range(&mut txn, start as u32, del_len as u32);
                }
                if !insert.is_empty() {
                    text.insert(&mut txn, start as u32, insert);
                }
                txn.encode_diff_v1(&before)
            };
            drop(awareness);
            room.broadcast(
                &yjs_frame(&Message::Sync(SyncMessage::Update(update))),
                None,
            );
        }
        // Advance the baseline AFTER merging, whether or not the merge was a
        // no-op — otherwise the next debounce tick diffs against the
        // pre-rerun text and tries to "resolve" the rerun's own changes into
        // source as if a user had typed them.
        *room.baseline.lock().await = new_content.to_string();
    }
}

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

/// The output-room counterpart of `ws::run_socket`: same per-connection
/// framing, but only the Yjs channel — output rooms have no run-event stream
/// and no LSP bridge of their own (both are document-level concerns).
pub async fn run_output_socket(
    state: AppState,
    doc: DocRow,
    output_path: String,
    socket: WebSocket,
) -> anyhow::Result<()> {
    let doc_id = doc.id;
    let room = state
        .output_rooms
        .get_or_create(&state, &doc, &output_path)
        .await
        .map_err(|e| anyhow::anyhow!(e.message))?;
    let client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
    room.clients.lock().unwrap().insert(client_id, tx.clone());

    let (mut sink, mut stream) = socket.split();

    let writer = tokio::spawn(async move {
        while let Some(frame) = rx.recv().await {
            if sink.send(WsMessage::Binary(frame.into())).await.is_err() {
                break;
            }
        }
    });

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
        if data.is_empty() || data[0] != crate::CHANNEL_YJS {
            continue; // no run/LSP channels on an output room
        }
        if let Err(e) = handle_yjs_payload(&state, &room, client_id, &tx, &data[1..]).await {
            log::debug!("yjs message error on output {doc_id}:{output_path}: {e:#}");
        }
    }

    room.clients.lock().unwrap().remove(&client_id);
    writer.abort();
    // Re-load the doc rather than reuse the connect-time copy: `doc.source`
    // may be minutes stale by disconnect time, and `apply_output_edits`'s
    // staleness check trusts it verbatim when the mapped edit lands back in
    // this same doc (see routes::outputs::apply_output_edits).
    match crate::routes::docs::load_doc(&state, doc_id).await {
        Ok(fresh_doc) => persist_now(&state, &room, &fresh_doc).await,
        Err(e) => log::warn!("output room {doc_id}:{output_path}: doc vanished on flush: {e:#}"),
    }
    state.output_rooms.drop_if_empty(doc_id, &output_path).await;
    Ok(())
}

async fn handle_yjs_payload(
    state: &AppState,
    room: &Arc<OutputRoom>,
    client_id: u64,
    tx: &mpsc::UnboundedSender<Vec<u8>>,
    payload: &[u8],
) -> anyhow::Result<()> {
    let protocol = DefaultProtocol;
    let mut decoder = DecoderV1::new(yrs::encoding::read::Cursor::new(payload));
    let mut reader = MessageReader::new(&mut decoder);
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

/// Debounced resolve-into-source: wait for edits to quiesce (750ms, matching
/// a doc room's own persist debounce), then diff the room's live text
/// against `baseline` and, if it changed, apply through the same path the
/// manual "apply" REST endpoint uses.
fn schedule_persist(state: AppState, room: Arc<OutputRoom>) {
    let generation = room.generation.fetch_add(1, Ordering::SeqCst) + 1;
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(750)).await;
        if room.generation.load(Ordering::SeqCst) == generation {
            let doc = match crate::routes::docs::load_doc(&state, room.doc_id).await {
                Ok(d) => d,
                Err(e) => {
                    log::warn!("output room {}: doc vanished: {e:#}", room.doc_id);
                    return;
                }
            };
            persist_now(&state, &room, &doc).await;
        }
    });
}

async fn persist_now(state: &AppState, room: &Arc<OutputRoom>, doc: &DocRow) {
    // Unlike ws::Room::persist_now, `persisted` is only advanced on SUCCESS
    // (below) — a failed apply must leave this generation un-persisted so a
    // retry (rescheduled in the Err arm) doesn't short-circuit on the "already
    // persisted this generation" check.
    let generation = room.generation.load(Ordering::SeqCst);
    if room.persisted.load(Ordering::SeqCst) == generation {
        return; // nothing new since the last successful persist
    }
    let current = room.current_text().await;
    let baseline = room.baseline.lock().await.clone();
    if current == baseline {
        room.persisted.store(generation, Ordering::SeqCst);
        return;
    }
    let (start, del_len, insert) = byte_delta(&baseline, &current);
    let edit = hickory_lineage::OutputEdit {
        start,
        end: start + del_len,
        text: insert.to_string(),
    };
    match apply_output_edits(state, doc, &room.output_path, &[edit]).await {
        Ok(_) => {
            *room.baseline.lock().await = current;
            room.persisted.store(generation, Ordering::SeqCst);
        }
        Err(e) => {
            // Leave the room's live text untouched — never roll back a
            // collaborator's keystrokes — and reschedule: without this,
            // silence from every client (no further edits) would mean this
            // edit is stuck forever, since nothing else re-triggers a
            // persist attempt. No client-visible conflict UI yet; see module
            // docs — a future re-run's `reconcile_rerun` can also clear this
            // by moving the baseline forward.
            log::warn!(
                "output room {}:{} could not resolve into source, retrying: {e:#}",
                room.doc_id,
                room.output_path
            );
            schedule_persist(state.clone(), Arc::clone(room));
        }
    }
}
