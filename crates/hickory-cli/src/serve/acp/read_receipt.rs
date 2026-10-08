//! Derive the receipt from the exact range returned by an ACP file read.
use hickory_agent::ContextRead;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) fn read(path: &str, text: &str, params: &Value) -> (String, Option<ContextRead>) {
    let start = params["line"].as_u64().unwrap_or(1).saturating_sub(1) as usize;
    let limit = params["limit"].as_u64().map_or(usize::MAX, |n| n as usize);
    let count = text.split_inclusive('\n').count();
    let shown = count.saturating_sub(start).min(limit);
    let content = text.split_inclusive('\n').skip(start).take(limit).collect();
    let receipt = (shown > 0).then(|| ContextRead {
        path: path.into(),
        commit: None,
        sha256: hex::encode(Sha256::digest(text.as_bytes())),
        first_line: start + 1,
        last_line: start + shown,
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
