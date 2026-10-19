//! Derive the receipt from the exact range returned by an ACP file read.
use hickory_agent::ContextRead;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) fn read(path: &str, text: &str, params: &Value) -> (String, Option<ContextRead>) {
    let start = params["line"].as_u64().unwrap_or(1).saturating_sub(1) as usize;
    let limit = params["limit"].as_u64().map_or(usize::MAX, |n| n as usize);
    let count = text.split_inclusive('\n').count();
    let shown = count.saturating_sub(start).min(limit);
    let content: String = text.split_inclusive('\n').skip(start).take(limit).collect();
    let receipt = (shown > 0).then(|| ContextRead {
        path: path.into(),
        source_path: None,
        commit: None,
        sha256: hex::encode(Sha256::digest(text.as_bytes())),
        first_line: start + 1,
        last_line: start + shown,
        content: content.clone(),
        unique: hick_context_store::unique_passage(text, &content),
        byte_span: (
            text.split_inclusive('\n').take(start).map(str::len).sum(),
            text.split_inclusive('\n')
                .take(start + shown)
                .map(str::len)
                .sum(),
        ),
    });
    (content, receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Guarantee: docs/guarantees/agent/context-provenance-is-derived-from-the-session.md
    #[test]
    fn receipts_cover_only_returned_lines_and_hash_the_supplied_snapshot() {
        let text = "first\r\nsecond 📝\r\nthird";
        let (content, receipt) = read("note.md", text, &json!({"line":2,"limit":1}));
        assert_eq!(content, "second 📝\r\n");
        let receipt = receipt.unwrap();
        // Guarantee: docs/guarantees/agent/immutable-context-lineage.md
        assert_eq!(receipt.byte_span, (7, 20));
        assert_eq!((receipt.first_line, receipt.last_line), (2, 2));
        assert_eq!(receipt.sha256, hex::encode(Sha256::digest(text.as_bytes())));
        assert_eq!(
            read("note.md", text, &json!({"line":3,"limit":99}))
                .1
                .unwrap()
                .last_line,
            3
        );
        for (text, params) in [
            ("", json!({})),
            (text, json!({"line":99})),
            (text, json!({"limit":0})),
        ] {
            let (content, receipt) = read("note.md", text, &params);
            assert!(content.is_empty());
            assert!(
                receipt.is_none(),
                "an empty response has no returned source lines"
            );
        }
    }
}

/// Import the existing receipt's exact bytes and original snapshot coordinates.
/// This is capture evidence only: external adapter/provider inclusion remains opaque.
pub(crate) fn persist(
    record: &super::record::Record,
    kind: &str,
    locator: &str,
    read: &ContextRead,
) -> anyhow::Result<()> {
    let mut store = hick_context_store::Store::open(
        &record.path.with_extension("context.json"),
        64 * 1024 * 1024,
        1024,
    )?;
    let mut source = store.register(kind, locator)?;
    source.revision = read.sha256.clone();
    let nodes = store.import_chunks_anchored(
        source,
        hick_context_store::span(read.byte_span.0, read.byte_span.1),
        &read.content,
        &hick_context_store::fresh_id(),
        read.unique,
    )?;
    record.context("context-capture", &serde_json::json!({"nodes":nodes.iter().map(|n|&n.id).collect::<Vec<_>>(),"state":"captured","coverage":"adapter_delivery_opaque"}))
}

#[cfg(test)]
mod lineage_tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/immutable-context-lineage.md
    #[test]
    fn retained_acp_receipt_does_not_reread_or_claim_delivery() {
        let dir = tempfile::tempdir().unwrap();
        let session = dir.path().join("session.md");
        let record =
            super::super::record::Record::open(session.clone(), &dir.path().join("note.md"))
                .unwrap();
        let (_, receipt) = read(
            "note.md",
            "before\r\n📝 original\r\n📝 original\r\n",
            &serde_json::json!({"line":2,"limit":1}),
        );
        let receipt = receipt.unwrap();
        std::fs::write(dir.path().join("note.md"), "changed after capture").unwrap();
        persist(&record, "editor_buffer", "buffer-1", &receipt).unwrap();
        let mut store = hick_context_store::Store::open(
            &session.with_extension("context.json"),
            64 * 1024 * 1024,
            1024,
        )
        .unwrap();
        assert_eq!(store.nodes().len(), 1);
        let node = &store.nodes()[0];
        assert_eq!(node.text.as_deref(), Some("📝 original\r\n"));
        assert_eq!(node.source_span.as_ref().unwrap().start, 8);
        assert_eq!(node.source.as_ref().unwrap().revision, receipt.sha256);
        assert_eq!(node.source.as_ref().unwrap().kind, "editor_buffer");
        assert_eq!(store.captures()[0].state, "captured");
        assert!(
            !receipt.unique,
            "the original full snapshot had duplicate passages"
        );
        let old = node.id.clone();
        let mut source = node.source.clone().unwrap();
        let current = "before\r\nedited\r\n📝 original\r\n";
        source.revision = hex::encode(Sha256::digest(current.as_bytes()));
        let newer = store
            .import_read(
                source,
                hick_context_store::span(0, current.len()),
                current,
                "current",
            )
            .unwrap();
        let comparison = store
            .compare(
                &serde_json::from_value(serde_json::json!({"old":old,"current":newer.id})).unwrap(),
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(comparison).unwrap()["state"],
            "ambiguous"
        );
        assert!(
            std::fs::read_to_string(session)
                .unwrap()
                .contains("adapter_delivery_opaque")
        );
    }
}
