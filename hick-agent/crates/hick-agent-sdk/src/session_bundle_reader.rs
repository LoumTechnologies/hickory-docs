//! Read-only replay reader for session bundles produced by [`XmlSessionLog`].
//!
//! Parses `session.xml` (Agent Log XML, namespace
//! `http://www.hickorydocs.com/agent-log/1.0`) and maps each recorded element
//! to the corresponding [`UiEvent`] variant.  Unknown or malformed elements are
//! silently skipped — older sessions remain loadable as the format evolves.

use std::path::Path;

use tokio::sync::mpsc;

use crate::ui_event::{IdGen, NodeId, UiEvent};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Read-only replay reader for Agent Log XML session bundles.
pub struct SessionBundleReader;

impl SessionBundleReader {
    /// Parse `<bundle_dir>/session.xml` and return all mapped [`UiEvent`]s.
    pub fn read(bundle_dir: &Path) -> anyhow::Result<Vec<UiEvent>> {
        let xml_path = bundle_dir.join("session.xml");
        let content = std::fs::read_to_string(&xml_path)
            .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", xml_path.display()))?;
        Ok(parse_session_xml(&content))
    }

    /// Parse the session bundle and send each [`UiEvent`] over `tx`.
    ///
    /// Stops early and returns `Ok(())` if the receiver is dropped.
    pub async fn stream(
        bundle_dir: &Path,
        tx: mpsc::Sender<UiEvent>,
    ) -> anyhow::Result<()> {
        let events = Self::read(bundle_dir)?;
        for ev in events {
            if tx.send(ev).await.is_err() {
                break;
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

fn parse_session_xml(xml: &str) -> Vec<UiEvent> {
    let id_gen = IdGen::new();
    let mut events: Vec<UiEvent> = Vec::new();
    // Tracks the node_id of the last ActionStart so ActionEnd can reuse it.
    let mut pending_action_id: Option<NodeId> = None;

    for line in xml.lines() {
        let line = line.trim();
        let Some(tag) = extract_log_tag(line) else {
            continue;
        };

        match tag {
            "session-start" => {
                let node_id = id_gen.next();
                events.push(UiEvent::SessionStarted {
                    node_id,
                    model: attr(line, "model"),
                    sandbox_image: attr(line, "sandbox"),
                    provider_info: None,
                });
            }
            "user" => {
                let node_id = id_gen.next();
                events.push(UiEvent::UserTurn {
                    node_id,
                    parent_id: None,
                    text: text_content(line),
                });
            }
            "assistant" => {
                // Emit a resolved Thinking node then the full AssistantDone.
                let thinking_id = id_gen.next();
                let assistant_id = id_gen.next();
                events.push(UiEvent::Thinking {
                    node_id: thinking_id,
                    parent_id: None,
                    text: None,
                });
                events.push(UiEvent::AssistantDone {
                    node_id: assistant_id,
                    text: text_content(line),
                });
            }
            "action" => {
                let node_id = id_gen.next();
                pending_action_id = Some(node_id);
                events.push(UiEvent::ActionStart {
                    node_id,
                    parent_id: None,
                    lang: attr(line, "lang").unwrap_or_else(|| "sh".into()),
                    code: text_content(line),
                    collapsed_by_default: true,
                });
            }
            "observation" => {
                // Reuse the node_id from the preceding ActionStart when available
                // so ConversationTree patches the same node.
                let node_id = pending_action_id.take().unwrap_or_else(|| id_gen.next());
                let exit_code: Option<i32> =
                    attr(line, "exit").and_then(|s| s.parse().ok());
                events.push(UiEvent::ActionEnd {
                    node_id,
                    stdout: text_content(line),
                    stderr: String::new(),
                    exit_code,
                    collapsed_by_default: true,
                });
            }
            "mcp-call" => {
                events.push(UiEvent::McpCall {
                    node_id: id_gen.next(),
                    parent_id: None,
                    server: attr(line, "server").unwrap_or_else(|| "unknown".into()),
                    method: attr(line, "method").unwrap_or_else(|| "unknown".into()),
                    collapsed_by_default: true,
                });
            }
            "mcp-response" => {
                events.push(UiEvent::McpResponse {
                    node_id: id_gen.next(),
                    server: attr(line, "server").unwrap_or_else(|| "unknown".into()),
                    method: attr(line, "method").unwrap_or_else(|| "unknown".into()),
                    collapsed_by_default: true,
                });
            }
            "security-violation" => {
                events.push(UiEvent::SecurityViolation {
                    node_id: id_gen.next(),
                    description: text_content(line),
                });
            }
            "session-end" => {
                events.push(UiEvent::Done { node_id: id_gen.next() });
            }
            // state-transition, diagnostic, session root — skip silently.
            _ => {}
        }
    }

    events
}

// ---------------------------------------------------------------------------
// Line-level helpers
// ---------------------------------------------------------------------------

/// Extract the local name after `log:` from a line that starts with `<log:`.
///
/// Returns `None` for lines that do not contain a `<log:` element open tag.
fn extract_log_tag(line: &str) -> Option<&str> {
    let start = line.find("<log:")?;
    let rest = &line[start + 5..]; // skip past "<log:"
    let end = rest
        .find(|c: char| c == ' ' || c == '/' || c == '>')
        .unwrap_or(rest.len());
    let tag = &rest[..end];
    if tag.is_empty() { None } else { Some(tag) }
}

/// Extract the value of `name="..."` from a line.  Returns `None` if absent.
fn attr(line: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = line.find(needle.as_str())? + needle.len();
    let end = line[start..].find('"')? + start;
    Some(xml_unescape(&line[start..end]))
}

/// Extract the text content between the opening and closing `log:*` tags,
/// stripping CDATA wrappers if present.
fn text_content(line: &str) -> String {
    // Find the end of the opening tag.
    let Some(open_end) = line.find('>') else {
        return String::new();
    };
    let after_open = &line[open_end + 1..];

    // Self-closing → no content.
    if open_end > 0 && line.as_bytes()[open_end - 1] == b'/' {
        return String::new();
    }

    // Strip trailing `</log:tag>` close tag.
    let raw = match after_open.rfind("</") {
        Some(i) => &after_open[..i],
        None => after_open,
    };

    // Unwrap CDATA.
    let raw = raw.trim();
    if let Some(inner) = raw.strip_prefix("<![CDATA[") {
        if let Some(inner) = inner.strip_suffix("]]>") {
            return inner.to_string();
        }
    }

    xml_unescape(raw)
}

/// Reverse the five standard XML entity references.
fn xml_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_log::{LogEvent, LogLevel};
    use crate::xml_session_log::XmlSessionLog;
    use crate::session_log::SessionLog as _;
    use tempfile::tempdir;

    fn write_and_read(write: impl FnOnce(&XmlSessionLog)) -> Vec<UiEvent> {
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        write(&log);
        SessionBundleReader::read(dir.path()).unwrap()
    }

    #[test]
    fn test_session_start_and_end() {
        let events = write_and_read(|log| {
            log.record(LogEvent::SessionStart {
                model: "claude-sonnet",
                sandbox_image: Some("alpine"),
                max_turns: 10,
            });
            log.record(LogEvent::SessionEnd);
        });

        let has_session = events
            .iter()
            .any(|e| matches!(e, UiEvent::SessionStarted { model: Some(m), .. } if m == "claude-sonnet"));
        assert!(has_session, "missing SessionStarted: {events:?}");

        let has_done = events.iter().any(|e| matches!(e, UiEvent::Done { .. }));
        assert!(has_done, "missing Done: {events:?}");
    }

    #[test]
    fn test_user_and_assistant_turn() {
        let events = write_and_read(|log| {
            log.record(LogEvent::UserTurn { text: "hello world" });
            log.record(LogEvent::LlmResponse { text: "hi there" });
        });

        let user = events.iter().find(|e| matches!(e, UiEvent::UserTurn { .. }));
        assert!(user.is_some(), "missing UserTurn");

        if let Some(UiEvent::UserTurn { text, .. }) = user {
            assert_eq!(text, "hello world");
        }

        let done = events
            .iter()
            .find(|e| matches!(e, UiEvent::AssistantDone { text, .. } if text == "hi there"));
        assert!(done.is_some(), "missing AssistantDone with correct text");
    }

    #[test]
    fn test_action_start_and_end_share_node_id() {
        let events = write_and_read(|log| {
            log.record(LogEvent::ActionStart { lang: "python", code: "print('x')" });
            log.record(LogEvent::ActionEnd { stdout: "x\n", stderr: "", exit_code: Some(0) });
        });

        let start = events
            .iter()
            .find(|e| matches!(e, UiEvent::ActionStart { .. }));
        let end = events.iter().find(|e| matches!(e, UiEvent::ActionEnd { .. }));
        assert!(start.is_some() && end.is_some());

        let start_id = if let Some(UiEvent::ActionStart { node_id, .. }) = start {
            *node_id
        } else {
            panic!("no ActionStart");
        };
        let end_id = if let Some(UiEvent::ActionEnd { node_id, .. }) = end {
            *node_id
        } else {
            panic!("no ActionEnd");
        };
        assert_eq!(start_id, end_id, "ActionStart and ActionEnd must share node_id");
    }

    #[test]
    fn test_mcp_call_and_response() {
        let events = write_and_read(|log| {
            let params = serde_json::json!({"tool": "read_file"});
            let result = serde_json::json!({"content": "data"});
            log.record(LogEvent::McpCall { server: "github", method: "tools/call", params: &params });
            log.record(LogEvent::McpResponse { server: "github", method: "tools/call", result: &result });
        });

        let call = events.iter().find(|e| matches!(e, UiEvent::McpCall { .. }));
        let resp = events.iter().find(|e| matches!(e, UiEvent::McpResponse { .. }));
        assert!(call.is_some(), "missing McpCall");
        assert!(resp.is_some(), "missing McpResponse");

        if let Some(UiEvent::McpCall { server, method, .. }) = call {
            assert_eq!(server, "github");
            assert_eq!(method, "tools/call");
        }
    }

    #[test]
    fn test_security_violation() {
        let events = write_and_read(|log| {
            log.record(LogEvent::SecurityViolation { description: "denied" });
        });
        let v = events
            .iter()
            .find(|e| matches!(e, UiEvent::SecurityViolation { description, .. } if description == "denied"));
        assert!(v.is_some());
    }

    #[test]
    fn test_xml_entities_unescaped() {
        let events = write_and_read(|log| {
            log.record(LogEvent::UserTurn { text: "a < b && c > d" });
        });
        if let Some(UiEvent::UserTurn { text, .. }) = events.iter().find(|e| matches!(e, UiEvent::UserTurn { .. })) {
            assert_eq!(text, "a < b && c > d");
        } else {
            panic!("missing UserTurn");
        }
    }

    #[test]
    fn test_unknown_elements_skipped() {
        // Inject an unknown element directly via the Custom escape hatch.
        let dir = tempdir().unwrap();
        let log = XmlSessionLog::new(dir.path().to_path_buf()).unwrap();
        log.record(LogEvent::UserTurn { text: "before" });
        log.record(LogEvent::Custom {
            xml_fragment: "  <log:future-element foo=\"bar\">content</log:future-element>",
            attachments: &[],
        });
        log.record(LogEvent::UserTurn { text: "after" });

        let events = SessionBundleReader::read(dir.path()).unwrap();
        let user_turns: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, UiEvent::UserTurn { .. }))
            .collect();
        assert_eq!(user_turns.len(), 2, "both user turns should be present");
    }

    #[test]
    fn test_diagnostic_skipped() {
        let events = write_and_read(|log| {
            log.record(LogEvent::DiagnosticLog {
                level: LogLevel::Info,
                target: "test",
                message: "diag msg",
            });
        });
        // Diagnostic → Info UiEvent is NOT emitted; only unknown tags are skipped.
        // There should be zero events.
        assert!(events.is_empty(), "diagnostic should not map to any UiEvent");
    }

    #[test]
    fn test_sandbox_image_attr() {
        let events = write_and_read(|log| {
            log.record(LogEvent::SessionStart {
                model: "m",
                sandbox_image: Some("my-image"),
                max_turns: 1,
            });
        });
        let found = events.iter().any(|e| {
            matches!(e, UiEvent::SessionStarted { sandbox_image: Some(img), .. } if img == "my-image")
        });
        assert!(found, "sandbox_image not parsed: {events:?}");
    }
}
