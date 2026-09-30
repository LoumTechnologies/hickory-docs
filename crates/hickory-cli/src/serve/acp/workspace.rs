//! Apply external edits against the room revision the tool actually read.
//! CRDT operations preserve concurrent keystrokes outside the changed region.
use super::client::RoomContext;
use anyhow::Result;
use std::collections::HashMap;
use yrs::sync::{Message, SyncMessage};
use yrs::updates::{decoder::Decode as _, encoder::Encode as _};
use yrs::{ReadTxn as _, Text as _, Transact as _, Update};

pub type Snapshot = (String, Vec<u8>);

pub async fn capture(context: &RoomContext) -> Result<HashMap<String, Snapshot>> {
    let mut snapshots = HashMap::new();
    for (id, _) in context.index.entries() {
        if let Some(room) = context.rooms.get(&id).await {
            let snapshot = room.snapshot().await;
            if let Some(path) = context.index.absolute(&id) {
                super::super::store::write_atomic(&path, snapshot.0.as_bytes())?;
            }
            snapshots.insert(id, snapshot);
        }
    }
    Ok(snapshots)
}

pub async fn apply(
    context: &RoomContext,
    id: &str,
    base: &Snapshot,
    source: &str,
) -> Result<String> {
    let Some(room) = context.rooms.get(id).await else {
        return Ok(source.into());
    };
    if base.0 == source {
        return Ok(room.text().await);
    }
    // This detached doc shares history, but mints the external edit under a
    // fresh identity. Applying its update cannot delete later room operations.
    let doc = hickory_collab::doc_with_client_id(super::super::rand_id() & ((1 << 53) - 1));
    let text = doc.get_or_insert_text("source");
    let update = {
        let mut txn = doc.transact_mut();
        txn.apply_update(Update::decode_v1(&base.1)?)?;
        let before = txn.state_vector();
        let (start, len, inserted) = hickory_collab::text_delta(&base.0, source);
        if len > 0 {
            text.remove_range(&mut txn, start as u32, len as u32);
        }
        if !inserted.is_empty() {
            text.insert(&mut txn, start as u32, inserted);
        }
        txn.encode_diff_v1(&before)
    };
    let payload = Message::Sync(SyncMessage::Update(update)).encode_v1();
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    context
        .rooms
        .handle_yjs_payload(&room, doc.client_id(), &tx, &payload, true)
        .await?;
    Ok(room.text().await)
}

pub async fn finish(context: &RoomContext, snapshots: HashMap<String, Snapshot>) -> Result<()> {
    for (id, base) in snapshots {
        if let Some(path) = context.index.absolute(&id) {
            let source = std::fs::read_to_string(&path)?;
            if source != base.0 {
                let merged = apply(context, &id, &base, &source).await?;
                super::super::store::write_atomic(&path, merged.as_bytes())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/acp-agents-are-first-class.md
    #[tokio::test]
    async fn external_edit_keeps_concurrent_typing_and_utf16_coordinates() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let original = "# 📝 Note\n\nhello\n";
        std::fs::write(root.join("note.md"), original).unwrap();
        let prepared = crate::serve::prepare(crate::serve::ServeOptions {
            target: root.clone(),
            port: 0,
            params: vec![],
            executor: crate::ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        })
        .await
        .unwrap();
        let context = RoomContext {
            index: prepared.state.index.clone(),
            rooms: prepared.state.rooms.clone(),
        };
        let id = context.index.sole().unwrap().0;
        let room = context.rooms.get_or_create(&id).await.unwrap();
        let base = room.snapshot().await;
        context
            .rooms
            .apply_external_source(&id, &format!("{original}My concurrent note.\n"))
            .await;
        let merged = apply(
            &context,
            &id,
            &base,
            &original.replace("hello", "agent edit"),
        )
        .await
        .unwrap();
        assert_eq!(merged, "# 📝 Note\n\nagent edit\nMy concurrent note.\n");
    }
}
