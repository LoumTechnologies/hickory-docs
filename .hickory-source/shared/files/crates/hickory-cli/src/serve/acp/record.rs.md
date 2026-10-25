<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/acp/record.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../../../../guarantees/agent/acp-agents-are-first-class/contract.md)
- [Recording failure is visible](../../../../../../../guarantees/agent/recording-failure-is-visible/contract.md)

Captured SHA-256: `914ed7c8f85417697521abaea14082910ed8d504b7fc9014adb3c297b728ce4c`.

<book:entry path="crates/hickory-cli/src/serve/acp/record.rs">
//! One atomic, closed session document. ACP reports are evidence, never exec cells.
use anyhow::Result;
use hickory_agent::{HickSessionLog, SessionEvent, SessionLog};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub struct Record {
    pub path: PathBuf,
    failure: std::sync::Mutex<Option<String>>,
}
impl Record {
    pub fn existing(path: PathBuf) -> Self {
        Self {
            path,
            failure: Default::default(),
        }
    }
    pub fn check(&self) -> Result<()> {
        if let Some(error) = &*self.failure.lock().unwrap() {
            anyhow::bail!(
                "Session recording is incomplete at {}: {error}",
                self.path.display()
            );
        }
        anyhow::ensure!(
            self.path.is_file(),
            "Session recording is incomplete at {}: record is unavailable",
            self.path.display()
        );
        Ok(())
    }
    fn persist(&self, operation: impl FnOnce() -> Result<()>) -> Result<()> {
        let result = self.check().and_then(|()| operation());
        if let Err(error) = &result {
            *self.failure.lock().unwrap() = Some(format!("{error:#}"));
        }
        result
    }
    pub fn open(path: PathBuf, doc: &Path) -> Result<Self> {
        if !path.exists() {
            let log = HickSessionLog::create_for(&path, Some(doc))?;
            log.record(SessionEvent::End);
            log.check()?;
        }
        Ok(Self::existing(path))
    }
    pub fn event(&self, event: SessionEvent<'_>) -> Result<()> {
        self.persist(|| {
            let log = HickSessionLog::append_or_create(&self.path)?;
            log.record(event);
            log.record(SessionEvent::End);
            log.check()?;
            Ok(())
        })
    }
    pub fn context(&self, kind: &str, data: &Value) -> Result<()> {
        self.persist(|| {
            let source = std::fs::read_to_string(&self.path)?;
            let mut data = data.clone();
            if let Some(object) = data.as_object_mut() {
                object.insert(
                    "recorded_at".into(),
                    json!(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)?
                            .as_millis()
                            .to_string()
                    ),
                );
            }
            let content = serde_json::to_string(&data)?.replace('<', "\\u003c");
            let source = format!(
                "{}<hick:context kind=\"{kind}\">{content}</hick:context>\n</hick:session>\n",
                source
                    .trim_end()
                    .strip_suffix("</hick:session>")
                    .unwrap_or(&source)
            );
            super::super::store::write_atomic(&self.path, source.as_bytes())
        })
    }
    /// Once the canonical assistant is durable, discard redundant token checkpoints.
    pub fn compact_stream(&self, turn: &str) -> Result<()> {
        let mut source = std::fs::read_to_string(&self.path)?;
        let doc = hick_lang::parse(&source)?;
        let mut spans: Vec<_> = doc
            .all_tags()
            .into_iter()
            .filter(|t| t.name == "context" && t.get_attribute("kind") == Some("acp-stream"))
            .filter(|t| {
                serde_json::from_str::<Value>(&t.text_content()).is_ok_and(|v| v["turn"] == turn)
            })
            .filter_map(|t| Some((t.source_span.as_ref()?.start, t.close_span.as_ref()?.end)))
            .collect();
        spans.sort_unstable();
        for (start, end) in spans.into_iter().rev() {
            let end = end + usize::from(source.as_bytes().get(end) == Some(&b'\n'));
            source.replace_range(start..end, "");
        }
        super::super::store::write_atomic(&self.path, source.as_bytes())
    }

    pub fn session(&self, agent: &str, session: &str) -> Result<()> {
        self.context(
            "acp-session",
            &json!({"agent": agent, "sessionId": session}),
        )
    }
}

pub fn metadata(path: &Path) -> Option<Value> {
    let source = std::fs::read_to_string(path).ok()?;
    let doc = hick_lang::parse(&source).ok()?;
    doc.all_tags()
        .into_iter()
        .filter(|t| t.name == "context" && t.get_attribute("kind") == Some("acp-session"))
        .filter_map(|t| serde_json::from_str(&t.text_content()).ok())
        .next_back()
}

pub fn status(path: &Path, turn: &str) -> Option<Value> {
    let source = std::fs::read_to_string(path).ok()?;
    let doc = hick_lang::parse(&source).ok()?;
    doc.all_tags()
        .into_iter()
        .filter(|t| t.name == "context" && t.get_attribute("kind") == Some("acp-turn-status"))
        .filter_map(|t| serde_json::from_str::<Value>(&t.text_content()).ok())
        .rfind(|v| v["turn"] == turn)
}

/// Recover streamed prose if the application exited before the final assistant.
pub fn partial_answer(path: &Path, turn: &str) -> Option<String> {
    let source = std::fs::read_to_string(path).ok()?;
    let doc = hick_lang::parse(&source).ok()?;
    let text: String = doc
        .all_tags()
        .into_iter()
        .filter(|t| t.name == "context" && t.get_attribute("kind") == Some("acp-stream"))
        .filter_map(|t| serde_json::from_str::<Value>(&t.text_content()).ok())
        .filter(|v| v["turn"] == turn && v["kind"] == "agent_message_chunk")
        .filter_map(|v| v["text"].as_str().map(str::to_string))
        .collect();
    (!text.is_empty()).then_some(text)
}

pub fn edit_mode(path: &Path) -> Option<super::edits::Mode> {
    let source = std::fs::read_to_string(path).ok()?;
    let doc = hick_lang::parse(&source).ok()?;
    doc.all_tags()
        .into_iter()
        .filter(|t| t.name == "context" && t.get_attribute("kind") == Some("acp-edit-mode"))
        .filter_map(|t| serde_json::from_str::<Value>(&t.text_content()).ok())
        .filter_map(|v| serde_json::from_value(v["mode"].clone()).ok())
        .next_back()
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/acp-agents-are-first-class.md
    #[test]
    fn interrupted_stream_remains_parseable_and_recoverable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.md");
        let record = Record::open(path.clone(), &dir.path().join("note.md")).unwrap();
        record
            .event(SessionEvent::UserTurn {
                text: "hello",
                turn: "turn-1",
                parent: None,
                provider: "acp:codex",
                model: "default",
            })
            .unwrap();
        for text in ["Partial ", "reply with <hick:file> and </hick:context>."] {
            record
                .context(
                    "acp-stream",
                    &json!({"turn":"turn-1","kind":"agent_message_chunk","text":text}),
                )
                .unwrap();
            hick_lang::parse_session(&std::fs::read_to_string(&path).unwrap()).unwrap();
        }
        assert_eq!(
            partial_answer(&path, "turn-1").unwrap(),
            "Partial reply with <hick:file> and </hick:context>."
        );
        assert_eq!(super::super::recovered_status(&path, "turn-1").0, "stopped");
    }
    // Guarantee: docs/guarantees/agent/recording-failure-is-visible.md
    #[test]
    fn an_activity_recording_gap_remains_a_failure_after_storage_recovers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.md");
        let record = Record::open(path.clone(), Path::new("note.md")).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert!(
            record
                .context("acp-activity", &json!({"kind":"execute"}))
                .is_err()
        );
        std::fs::write(path, "<hick:session></hick:session>\n").unwrap();
        assert!(
            record
                .check()
                .unwrap_err()
                .to_string()
                .contains("incomplete")
        );
        assert!(
            record
                .context("acp-turn-status", &json!({"status":"ok"}))
                .is_err()
        );
    }
}
</book:entry>

</book:doc>
