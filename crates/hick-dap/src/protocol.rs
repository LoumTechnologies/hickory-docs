//! The Debug Adapter Protocol, on the wire.
//!
//! Framed exactly like LSP — `Content-Length`, a blank line, one JSON object —
//! but correlated differently, and the difference is the whole reason this is
//! a sibling of `hick-lsp` rather than a generalisation of it:
//!
//! * LSP correlates by `id`, which the client chooses and the server echoes.
//! * DAP correlates by `seq`: **both sides** number their own messages, and a
//!   response carries `request_seq` naming the request it answers.
//!
//! And DAP is event-driven in a way LSP is not. The interesting things a
//! debugger says — `stopped`, `terminated`, `output` — arrive unsolicited,
//! and a client that only waits for responses hangs forever waiting for a
//! breakpoint to "reply".

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Anything that can arrive from an adapter.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Incoming {
    Response(Response),
    Event(Event),
    /// Adapters may send requests to the CLIENT (`runInTerminal`,
    /// `startDebugging`). Kept so an unanswered one is a decision rather than
    /// a parse failure — see `Session::handle_reverse_request`.
    Request(ReverseRequest),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub seq: i64,
    pub request_seq: i64,
    pub success: bool,
    pub command: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub body: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event {
    pub seq: i64,
    pub event: String,
    #[serde(default)]
    pub body: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReverseRequest {
    pub seq: i64,
    pub command: String,
    #[serde(default)]
    pub arguments: Value,
}

/// Encode one message with the header an adapter expects.
pub fn frame(message: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(message).unwrap_or_else(|_| b"{}".to_vec());
    let mut out = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    out.extend_from_slice(&body);
    out
}

/// Parse a `Content-Length` header block into its length.
///
/// Tolerant of the other headers adapters sometimes send (`Content-Type`),
/// and of a header block that has not finished arriving.
pub fn content_length(header: &str) -> Option<usize> {
    header
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_framed_message_carries_its_byte_length() {
        // Bytes, not characters: a header counting characters truncates the
        // first message containing anything outside ASCII, and the adapter
        // then reads the tail of it as the start of the next one.
        let framed = frame(&json!({ "seq": 1, "type": "request", "command": "π" }));
        let text = String::from_utf8(framed).unwrap();
        let (header, body) = text.split_once("\r\n\r\n").unwrap();
        assert_eq!(content_length(header), Some(body.len()));
        assert!(body.len() > body.chars().count());
    }

    #[test]
    fn headers_are_read_case_insensitively_and_around_others() {
        assert_eq!(
            content_length("Content-Type: application/json\r\ncontent-length: 42\r\n"),
            Some(42)
        );
    }

    #[test]
    fn an_incomplete_header_is_not_a_length() {
        assert_eq!(content_length("Content-Len"), None);
    }

    #[test]
    fn the_three_message_kinds_are_told_apart_by_type() {
        let response: Incoming = serde_json::from_value(json!({
            "seq": 2, "type": "response", "request_seq": 1, "success": true,
            "command": "initialize", "body": { "supportsStepBack": false }
        }))
        .unwrap();
        assert!(matches!(response, Incoming::Response(_)));

        let event: Incoming = serde_json::from_value(json!({
            "seq": 3, "type": "event", "event": "stopped",
            "body": { "reason": "breakpoint", "threadId": 1 }
        }))
        .unwrap();
        assert!(matches!(event, Incoming::Event(_)));

        // An adapter asking US for something. Parsed rather than dropped so
        // that leaving it unanswered is a decision we made.
        let reverse: Incoming = serde_json::from_value(json!({
            "seq": 4, "type": "request", "command": "runInTerminal",
            "arguments": { "args": ["python"] }
        }))
        .unwrap();
        assert!(matches!(reverse, Incoming::Request(_)));
    }

    #[test]
    fn a_failed_response_keeps_the_adapters_own_words() {
        // "Could not find a valid tsserver" taught this lesson on the LSP
        // side: the adapter's message is the only thing that says what is
        // actually wrong.
        let response: Response = serde_json::from_value(json!({
            "seq": 9, "type": "response", "request_seq": 8, "success": false,
            "command": "launch", "message": "Cannot find module 'debugpy'"
        }))
        .unwrap();
        assert!(!response.success);
        assert_eq!(
            response.message.as_deref(),
            Some("Cannot find module 'debugpy'")
        );
    }
}
