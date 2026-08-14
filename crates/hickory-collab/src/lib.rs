//! The collaborative editing layer, independent of where it is hosted.
//!
//! A **room** is one live Yjs document plus the sockets attached to it. This
//! crate owns the parts that are the same whether the room lives on a Fly
//! machine backed by Postgres or on a contributor's laptop backed by a `.hick`
//! file: the sync protocol, the document-doubling defences, the debounced
//! persist, and the broadcast.
//!
//! What differs between hosts is exactly one thing — where the two persisted
//! values (the source text and the encoded CRDT state) live — so that is the
//! whole of [`DocStore`]. Everything else is shared, deliberately: two
//! implementations of a Yjs room is how the 49 MB document-doubling bug that
//! migration 0003 exists to prevent happens a second time.
//!
//! Transport is *not* here. Each host owns its own socket loop, because the
//! frames it multiplexes differ (the hosted server bridges an LSP channel; a
//! local server may not) — and keeping axum out of this crate keeps it a
//! library about documents rather than about HTTP.
//!
//! See `docs/specs/freeform/local-collaboration.md`.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;
use yrs::sync::protocol::Protocol as _;
use yrs::sync::{Awareness, DefaultProtocol, Message, MessageReader, SyncMessage};
use yrs::updates::decoder::{Decode as _, DecoderV1};
use yrs::updates::encoder::Encode as _;
use yrs::{
    Doc, GetString as _, OffsetKind, Options, ReadTxn as _, Text as _, Transact as _, Update,
};

/// WS channel prefixes. One byte, then the payload (api.md).
pub const CHANNEL_YJS: u8 = 0x00;
pub const CHANNEL_RUN: u8 = 0x01;
pub const CHANNEL_LSP: u8 = 0x02;
/// Debugging: breakpoints, stepping and evaluation for the app's debugger.
///
/// Carries the session API's own verbs rather than raw DAP, because DAP
/// speaks in the coordinates of the file being run and the app speaks in the
/// document's — and that mapping belongs beside the code that already does
/// it, not in the browser.
pub const CHANNEL_DEBUG: u8 = 0x03;

/// Hard ceiling on a live document's text. Well above any real document and
/// well below the point where a browser stalls rendering it.
pub const MAX_DOC_BYTES: usize = 4 * 1024 * 1024;

/// How long edits must quiesce before a room persists.
const PERSIST_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(750);

/// Which document a room is for.
///
/// A string rather than a `Uuid` because the two hosts identify documents
/// differently — a database row id, or a path relative to the served
/// directory — and the room does not care which it was handed.
pub type DocKey = String;

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

/// Where a room's two durable values live.
///
/// Implementations are expected to be cheap to clone-share (`Arc`) and to
/// tolerate being called from a background task after the last socket closed:
/// the final persist happens on the way out.
#[async_trait]
pub trait DocStore: Send + Sync {
    /// The document's current source text.
    async fn load_source(&self, key: &DocKey) -> Result<String>;

    /// The encoded CRDT state, if this document has been collaborated on
    /// before. `None` means "never had a room", not "empty document".
    async fn load_crdt(&self, key: &DocKey) -> Result<Option<Vec<u8>>>;

    /// Persist both values together.
    ///
    /// They must be written as a pair: storing text without the state that
    /// produced it is what makes a re-created room mint a rival operation
    /// history instead of resuming this one.
    async fn save(&self, key: &DocKey, source: &str, crdt: &[u8]) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Y.Doc construction
// ---------------------------------------------------------------------------

/// A room's Y.Doc, given an already-derived stable client id.
///
/// `offset_kind` MUST be `Utf16`. yrs defaults to byte offsets, but every
/// index on the wire and in the browser is a UTF-16 code-unit offset (Yjs
/// semantics). With the default, any index math silently lands in the wrong
/// place on documents containing non-ASCII — `remove_range` clips the wrong
/// span and the replacement text is spliced mid-character.
pub fn doc_with_client_id(client_id: u64) -> Doc {
    Doc::with_options(Options {
        client_id,
        offset_kind: OffsetKind::Utf16,
        ..Options::default()
    })
}

/// Yjs client id for the server's own operations on a document, stable across
/// restarts and room re-creations.
///
/// A UUID-shaped key keeps the exact derivation the hosted server has always
/// used — the first four bytes of the uuid — because documents already carry
/// persisted CRDT state minted under it, and changing the derivation would
/// make an old seed and a new seed rival operations rather than duplicates.
/// Any other key (a local path) is hashed.
///
/// The result MUST stay inside JavaScript's safe-integer range: yrs stores
/// client ids as u64, but the browser decodes them into JS numbers, and a
/// value above 2^53 makes Yjs throw "Integer out of Range" and drop the whole
/// update — the client silently never receives the document. Yjs itself mints
/// 32-bit ids, so 32 bits is both safe and conventional.
pub fn stable_client_id(key: &str) -> u64 {
    if let Ok(uuid) = uuid::Uuid::parse_str(key) {
        let b = uuid.as_bytes();
        return u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as u64;
    }
    // FNV-1a, truncated to 32 bits. Any stable hash would do; this one is
    // three lines and has no dependency.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in key.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    (hash as u32) as u64
}

// ---------------------------------------------------------------------------
// Text reconciliation
// ---------------------------------------------------------------------------

/// Minimal replace turning `current` into `target`, in raw byte offsets:
/// `(start_byte, bytes_to_delete, text_to_insert)`.
pub fn byte_delta<'a>(current: &str, target: &'a str) -> (usize, usize, &'a str) {
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
    (
        prefix_bytes,
        current.len() - prefix_bytes - suffix_bytes,
        &target[prefix_bytes..target.len() - suffix_bytes],
    )
}

/// Minimal replace turning `current` into `target`, in UTF-16 code units
/// (Yjs text indices): `(offset, units_to_delete, text_to_insert)`.
pub fn text_delta<'a>(current: &str, target: &'a str) -> (usize, usize, &'a str) {
    let (prefix_bytes, del_bytes, insert) = byte_delta(current, target);
    let offset = current[..prefix_bytes].encode_utf16().count();
    let del_len = current[prefix_bytes..prefix_bytes + del_bytes]
        .encode_utf16()
        .count();
    (offset, del_len, insert)
}

// ---------------------------------------------------------------------------
// Framing
// ---------------------------------------------------------------------------

/// Frame a Yjs protocol message for the wire.
pub fn yjs_frame(msg: &Message) -> Vec<u8> {
    let payload = msg.encode_v1();
    let mut frame = Vec::with_capacity(payload.len() + 1);
    frame.push(CHANNEL_YJS);
    frame.extend_from_slice(&payload);
    frame
}

/// Send one framed Yjs protocol message to a single client.
pub fn send_yjs(tx: &mpsc::UnboundedSender<Vec<u8>>, msg: &Message) {
    let _ = tx.send(yjs_frame(msg));
}

/// Frame a run-channel JSON payload for the wire.
pub fn run_frame(payload: &serde_json::Value) -> Vec<u8> {
    let json = serde_json::to_vec(payload).expect("serializable run event");
    let mut frame = Vec::with_capacity(json.len() + 1);
    frame.push(CHANNEL_RUN);
    frame.extend_from_slice(&json);
    frame
}

// ---------------------------------------------------------------------------
// Room
// ---------------------------------------------------------------------------

/// One live document and the sockets attached to it.
pub struct Room {
    key: DocKey,
    awareness: tokio::sync::Mutex<Awareness>,
    clients: std::sync::Mutex<HashMap<u64, mpsc::UnboundedSender<Vec<u8>>>>,
    /// Bumped on every doc mutation; drives the debounced persist.
    generation: AtomicU64,
    /// Last generation that was persisted.
    persisted: AtomicU64,
}

impl Room {
    /// Which document this room is for.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Send `frame` to every attached client, optionally excluding one.
    pub fn broadcast(&self, frame: &[u8], except: Option<u64>) {
        let clients = self.clients.lock().unwrap();
        for (id, tx) in clients.iter() {
            if Some(*id) != except {
                let _ = tx.send(frame.to_vec());
            }
        }
    }

    /// Attach a client, returning nothing: the caller already holds the sender.
    pub fn attach(&self, client_id: u64, tx: mpsc::UnboundedSender<Vec<u8>>) {
        self.clients.lock().unwrap().insert(client_id, tx);
    }

    /// Detach a client.
    pub fn detach(&self, client_id: u64) {
        self.clients.lock().unwrap().remove(&client_id);
    }

    pub fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }

    /// The handshake a newly attached client must receive: server sync-step-1,
    /// then current awareness — one protocol message per frame, because the
    /// web client processes exactly one per frame.
    pub async fn handshake(&self, tx: &mpsc::UnboundedSender<Vec<u8>>) {
        let awareness = self.awareness.lock().await;
        let sv = awareness.doc().transact().state_vector();
        send_yjs(tx, &Message::Sync(SyncMessage::SyncStep1(sv)));
        if let Ok(update) = awareness.update() {
            send_yjs(tx, &Message::Awareness(update));
        }
    }

    /// Current text plus the encoded CRDT state, taken under one lock so the
    /// two can never describe different revisions.
    pub async fn snapshot(&self) -> (String, Vec<u8>) {
        let awareness = self.awareness.lock().await;
        let doc = awareness.doc();
        let text = doc.get_or_insert_text("source");
        let txn = doc.transact();
        (
            text.get_string(&txn),
            txn.encode_state_as_update_v1(&yrs::StateVector::default()),
        )
    }

    /// The room's current text alone.
    pub async fn text(&self) -> String {
        let awareness = self.awareness.lock().await;
        let doc = awareness.doc();
        let text = doc.get_or_insert_text("source");
        let txn = doc.transact();
        text.get_string(&txn)
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Every live room, and the store behind them.
pub struct RoomRegistry {
    rooms: tokio::sync::Mutex<HashMap<DocKey, Arc<Room>>>,
    store: Arc<dyn DocStore>,
}

impl RoomRegistry {
    pub fn new(store: Arc<dyn DocStore>) -> Self {
        Self {
            rooms: tokio::sync::Mutex::new(HashMap::new()),
            store,
        }
    }

    /// The live room for `key`, if any.
    pub async fn get(&self, key: &str) -> Option<Arc<Room>> {
        self.rooms.lock().await.get(key).cloned()
    }

    /// Broadcast a run-channel JSON payload to every socket on a document.
    pub async fn publish_run_event(&self, key: &str, payload: &serde_json::Value) {
        if let Some(room) = self.get(key).await {
            room.broadcast(&run_frame(payload), None);
        }
    }

    /// The room for `key`, creating and loading it if this is the first socket.
    ///
    /// Two rules hold room creation together, and both exist because of a real
    /// incident (migration 0003 — one document reached 49 MB, 3268 copies of
    /// itself):
    ///
    /// 1. **Resume, never re-seed.** Stored CRDT state is applied as-is, so a
    ///    re-created room continues the operation history a reconnecting client
    ///    already holds. Seeding a fresh `Doc` per room minted rival operations
    ///    that the CRDT merged by concatenation.
    /// 2. **Stable client id.** Even with no stored state, the seed operation is
    ///    minted under an id derived from the key, so two independent seeds of
    ///    the same source are the *same* operation and dedupe.
    ///
    /// If the source moved on out of band — a lineage edit, a run, a save — the
    /// difference is applied to the restored document as ordinary CRDT edits
    /// (common prefix and suffix preserved, so concurrent edits elsewhere in
    /// the file survive) rather than by starting over.
    pub async fn get_or_create(&self, key: &DocKey) -> Result<Arc<Room>> {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(key) {
            return Ok(room.clone());
        }

        let source = self.store.load_source(key).await?;
        let stored = self.store.load_crdt(key).await.unwrap_or_else(|e| {
            log::error!("reading crdt state for {key} failed: {e:#}");
            None
        });

        let ydoc = doc_with_client_id(stable_client_id(key));
        let text = ydoc.get_or_insert_text("source");
        if let Some(bytes) = stored {
            match Update::decode_v1(&bytes) {
                Ok(update) => {
                    let mut txn = ydoc.transact_mut();
                    if let Err(e) = txn.apply_update(update) {
                        log::error!("applying stored crdt state for {key} failed: {e}");
                    }
                }
                Err(e) => log::error!("stored crdt state for {key} is unreadable: {e}"),
            }
        }
        let current = {
            let txn = ydoc.transact();
            text.get_string(&txn)
        };
        if current != source {
            let mut txn = ydoc.transact_mut();
            let (start, del_len, insert) = text_delta(&current, &source);
            if del_len > 0 {
                text.remove_range(&mut txn, start as u32, del_len as u32);
            }
            if !insert.is_empty() {
                text.insert(&mut txn, start as u32, insert);
            }
        }

        let room = Arc::new(Room {
            key: key.clone(),
            awareness: tokio::sync::Mutex::new(Awareness::new(ydoc)),
            clients: std::sync::Mutex::new(HashMap::new()),
            generation: AtomicU64::new(0),
            persisted: AtomicU64::new(0),
        });
        rooms.insert(key.clone(), room.clone());
        Ok(room)
    }

    /// Drop the room for `key` if nothing is attached to it.
    pub async fn drop_if_empty(&self, key: &str) {
        let mut rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(key)
            && room.client_count() == 0
        {
            rooms.remove(key);
        }
    }

    /// Apply a source rewrite that happened outside the CRDT (an
    /// `/outputs/edit` resolved through provenance, a file changed on disk) to
    /// the live room, if one exists.
    ///
    /// Without this the room keeps serving its pre-edit text and its next
    /// debounced persist writes that stale text back over the source, so the
    /// edit silently disappears a few seconds after it lands.
    pub async fn apply_external_source(self: &Arc<Self>, key: &str, source: &str) {
        let Some(room) = self.get(key).await else {
            return;
        };
        let update = {
            let awareness = room.awareness.lock().await;
            let ydoc = awareness.doc();
            let text = ydoc.get_or_insert_text("source");
            let current = {
                let txn = ydoc.transact();
                text.get_string(&txn)
            };
            if current == source {
                return;
            }
            let mut txn = ydoc.transact_mut();
            let before = txn.state_vector();
            let (start, del_len, insert) = text_delta(&current, source);
            if del_len > 0 {
                text.remove_range(&mut txn, start as u32, del_len as u32);
            }
            if !insert.is_empty() {
                text.insert(&mut txn, start as u32, insert);
            }
            txn.encode_diff_v1(&before)
        };
        room.broadcast(
            &yjs_frame(&Message::Sync(SyncMessage::Update(update))),
            None,
        );
        // Keep the stored state in step with the source that was just written,
        // so a room re-created later resumes from the edited text.
        self.clone().schedule_persist(room);
    }

    /// Handle one inbound Yjs payload (the bytes after the channel prefix).
    ///
    /// `may_write` is the caller's permission to *change* the document. It is
    /// applied per message rather than per socket, because the Yjs protocol
    /// multiplexes reading and writing down one channel: `SyncStep1` and
    /// `AwarenessQuery` are how a client ASKS for the document, and refusing
    /// those to a read-only viewer does not make them read-only — it makes
    /// them blind. Only `SyncStep2`/`Update` mutate, so only those are dropped.
    ///
    /// Awareness updates are allowed either way: a viewer's cursor and name
    /// are how the people editing know someone is watching, and they touch no
    /// document state.
    ///
    /// Returns an error only when the socket should be closed — a malformed
    /// message, or a document that has grown past [`MAX_DOC_BYTES`].
    pub async fn handle_yjs_payload(
        self: &Arc<Self>,
        room: &Arc<Room>,
        client_id: u64,
        tx: &mpsc::UnboundedSender<Vec<u8>>,
        payload: &[u8],
        may_write: bool,
    ) -> Result<()> {
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
                        if !may_write {
                            // Silently ignored rather than closing the socket:
                            // a viewer whose editor optimistically emits an
                            // update should keep receiving everyone else's.
                            continue;
                        }
                        let update = Update::decode_v1(&bytes)?;
                        protocol.handle_update(&awareness, update)?;
                        let after = {
                            let d = awareness.doc();
                            let t = d.get_or_insert_text("source");
                            let txn = d.transact();
                            t.get_string(&txn).len()
                        };
                        if after > MAX_DOC_BYTES {
                            // A hick document is prose and cells; nothing
                            // legitimate reaches megabytes. Runaway growth here
                            // means a sync bug is concatenating copies of the
                            // text, and every extra round doubles it — refuse
                            // the socket instead of letting it persist and
                            // freeze every client that opens the doc.
                            log::error!(
                                "doc {} grew to {after} bytes (cap {MAX_DOC_BYTES}); \
                                 closing socket without persisting",
                                room.key
                            );
                            return Err(anyhow::anyhow!("document exceeds {MAX_DOC_BYTES} bytes"));
                        }
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
            self.clone().schedule_persist(room.clone());
        }
        Ok(())
    }

    /// Debounced persistence: wait for edits to quiesce, then write through
    /// the store.
    pub fn schedule_persist(self: Arc<Self>, room: Arc<Room>) {
        let generation = room.generation.fetch_add(1, Ordering::SeqCst) + 1;
        tokio::spawn(async move {
            tokio::time::sleep(PERSIST_DEBOUNCE).await;
            if room.generation.load(Ordering::SeqCst) == generation {
                self.persist_now(&room).await;
            }
        });
    }

    /// Persist immediately, if anything changed since the last write.
    pub async fn persist_now(&self, room: &Arc<Room>) {
        let generation = room.generation.load(Ordering::SeqCst);
        if room.persisted.swap(generation, Ordering::SeqCst) == generation {
            return; // nothing new since the last persist
        }
        let (source, crdt_state) = room.snapshot().await;
        if let Err(e) = self.store.save(&room.key, &source, &crdt_state).await {
            log::error!("persisting {} failed: {e:#}", room.key);
            // Let the next edit try again rather than recording this
            // generation as durable.
            room.persisted
                .store(generation.saturating_sub(1), Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemStore {
        source: Mutex<String>,
        crdt: Mutex<Option<Vec<u8>>>,
        saves: AtomicU64,
    }

    #[async_trait]
    impl DocStore for MemStore {
        async fn load_source(&self, _key: &DocKey) -> Result<String> {
            Ok(self.source.lock().unwrap().clone())
        }
        async fn load_crdt(&self, _key: &DocKey) -> Result<Option<Vec<u8>>> {
            Ok(self.crdt.lock().unwrap().clone())
        }
        async fn save(&self, _key: &DocKey, source: &str, crdt: &[u8]) -> Result<()> {
            *self.source.lock().unwrap() = source.to_string();
            *self.crdt.lock().unwrap() = Some(crdt.to_vec());
            self.saves.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn registry(store: Arc<MemStore>) -> Arc<RoomRegistry> {
        Arc::new(RoomRegistry::new(store))
    }

    #[tokio::test]
    async fn a_recreated_room_resumes_instead_of_doubling_the_document() {
        // The regression behind migration 0003: a room that re-seeds a fresh
        // Doc mints rival operations, and the merge concatenates both copies.
        let store = Arc::new(MemStore::default());
        *store.source.lock().unwrap() = "hello world\n".to_string();
        let reg = registry(store.clone());

        let room = reg.get_or_create(&"doc-1".to_string()).await.unwrap();
        reg.persist_now(&room).await;
        // Force a persist by pretending an edit happened.
        reg.clone().schedule_persist(room.clone());
        reg.persist_now(&room).await;
        drop(room);
        reg.drop_if_empty("doc-1").await;

        let again = reg.get_or_create(&"doc-1".to_string()).await.unwrap();
        assert_eq!(again.text().await, "hello world\n");
    }

    #[tokio::test]
    async fn a_source_that_moved_on_disk_is_reconciled_not_restarted() {
        let store = Arc::new(MemStore::default());
        *store.source.lock().unwrap() = "one\ntwo\nthree\n".to_string();
        let reg = registry(store.clone());
        let room = reg.get_or_create(&"doc-1".to_string()).await.unwrap();
        reg.persist_now(&room).await;
        reg.drop_if_empty("doc-1").await;
        drop(room);

        // Something else rewrote the middle line.
        *store.source.lock().unwrap() = "one\nTWO\nthree\n".to_string();
        let room = reg.get_or_create(&"doc-1".to_string()).await.unwrap();
        assert_eq!(room.text().await, "one\nTWO\nthree\n");
    }

    #[tokio::test]
    async fn an_external_rewrite_reaches_a_live_room() {
        let store = Arc::new(MemStore::default());
        *store.source.lock().unwrap() = "before\n".to_string();
        let reg = registry(store.clone());
        let room = reg.get_or_create(&"doc-1".to_string()).await.unwrap();

        reg.apply_external_source("doc-1", "after\n").await;
        assert_eq!(room.text().await, "after\n");
    }

    #[test]
    fn a_uuid_key_keeps_the_derivation_documents_were_seeded_with() {
        // Changing this for existing documents would make an old seed and a
        // new seed rival operations rather than duplicates.
        let uuid = uuid::Uuid::parse_str("00112233-4455-6677-8899-aabbccddeeff").unwrap();
        let expected = u32::from_le_bytes([0x00, 0x11, 0x22, 0x33]) as u64;
        assert_eq!(stable_client_id(&uuid.to_string()), expected);
    }

    #[test]
    fn a_path_key_is_stable_and_javascript_safe() {
        let a = stable_client_id("docs/tour.hick");
        assert_eq!(a, stable_client_id("docs/tour.hick"));
        assert_ne!(a, stable_client_id("docs/other.hick"));
        // Above 2^53 the browser drops the update and the client silently
        // never receives the document.
        assert!(a < (1u64 << 53));
    }

    #[test]
    fn deltas_reconcile_non_ascii_exactly() {
        // UTF-16 indices, not bytes: an emoji is two code units.
        let (offset, del, insert) = text_delta("héllo 🌲 world", "héllo 🌲 there");
        let rebuilt = {
            let mut units: Vec<u16> = "héllo 🌲 world".encode_utf16().collect();
            let ins: Vec<u16> = insert.encode_utf16().collect();
            units.splice(offset..offset + del, ins);
            String::from_utf16(&units).unwrap()
        };
        assert_eq!(rebuilt, "héllo 🌲 there");
    }
}
