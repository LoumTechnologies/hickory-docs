//! Session logging as literate programming.
//!
//! Every agent session is written **incrementally** as a `hick:session`
//! document — the same format `hick-lang` parses ([`hick_lang::parse_session`])
//! and `hickory promote` turns into a clean pipeline. The session file is the
//! durable, replayable record of the conversation: user turns, assistant
//! responses with embedded `<hick:action>` script blocks, and captured
//! `<hick:observation>` output.
//!
//! Per the hick no-escaping invariant, content is written raw (no CDATA, no
//! entity escaping): only `hick:`-prefixed tags are structured. The writer
//! strips the agent's own `<hick:next>` protocol tags from prose before
//! logging; other raw `<hick:...>` fragments inside content are the caller's
//! responsibility (well-formed nested tags survive the parse as unknown tags).

use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Utc;

/// A single event to be recorded into a session log.
#[derive(Debug)]
pub enum SessionEvent<'a> {
    /// A user message (`<hick:user>`).
    User { text: &'a str },
    /// A full LLM response (`<hick:assistant>`), with an optional embedded
    /// script block (`<hick:action lang="...">`).
    Assistant {
        /// Prose text of the response (excluding the code block).
        prose: &'a str,
        /// `(lang, code)` of the embedded action, if the response ran code.
        action: Option<(&'a str, &'a str)>,
    },
    /// A tool-invoking LLM response: an `<hick:assistant>` element carrying
    /// the raw `<hick:tool>` invocation XML verbatim.
    ToolCall {
        /// Prose text of the response (excluding the tool element).
        prose: &'a str,
        /// The verbatim `<hick:tool>...</hick:tool>` XML.
        xml: &'a str,
    },
    /// The result of a tool invocation (`<hick:tool-result>`). Inert during
    /// replay, like observations.
    ToolResult {
        /// Tool name (e.g. `"edit_output"`).
        name: &'a str,
        /// Whether the tool succeeded.
        ok: bool,
        /// The observation text returned to the model.
        text: &'a str,
    },
    /// Captured output from a script run (`<hick:observation>`).
    Observation {
        /// Identifies which action produced this observation (e.g.
        /// `"action-0"`).
        source: &'a str,
        /// Exit code of the originating command.
        exit_code: Option<i32>,
        /// Captured output text.
        text: &'a str,
    },
    /// Session ended — writes the closing `</hick:session>` tag.
    End,
}

/// Sink for structured agent session events.
///
/// Implementations must be `Send + Sync` so the log handle can be shared
/// across async tasks. The only required method is
/// [`record`][SessionLog::record]; implementations may buffer, write
/// synchronously, or discard events.
pub trait SessionLog: Send + Sync {
    fn record(&self, event: SessionEvent<'_>);
}

/// A no-op [`SessionLog`] that discards every event.
pub struct NullSessionLog;

impl SessionLog for NullSessionLog {
    fn record(&self, _event: SessionEvent<'_>) {}
}

// ---------------------------------------------------------------------------
// HickSessionLog
// ---------------------------------------------------------------------------

/// Writes a `hick:session` document incrementally to a file.
///
/// The XML declaration and `<hick:session>` root element are written at
/// construction; every [`record`][SessionLog::record] call appends the
/// corresponding element and flushes, so partial sessions on disk stay
/// readable. [`SessionEvent::End`] writes the closing root tag.
pub struct HickSessionLog {
    inner: Mutex<BufWriter<std::fs::File>>,
    path: PathBuf,
}

impl HickSessionLog {
    /// Create the session file at `path` (parent directories are created)
    /// and write the document header.
    pub fn create(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(&path)?;
        let mut writer = BufWriter::new(file);
        let start = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            writer,
            r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="{start}">"#
        )?;
        writer.flush()?;
        Ok(Self {
            inner: Mutex::new(writer),
            path,
        })
    }

    /// The path of the session file being written.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Strip the agent's `<hick:next>...</hick:next>` protocol tags from prose so
/// they don't appear as structured tags in the session document.
fn strip_protocol_tags(text: &str) -> String {
    text.replace("<hick:next>code</hick:next>", "")
        .replace("<hick:next>tool</hick:next>", "")
        .replace("<hick:next>done</hick:next>", "")
        .trim()
        .to_string()
}

impl SessionLog for HickSessionLog {
    fn record(&self, event: SessionEvent<'_>) {
        let Ok(mut writer) = self.inner.lock() else {
            return;
        };
        // Session logging must never crash the agent — swallow write errors.
        let result: std::io::Result<()> = (|| {
            match event {
                SessionEvent::User { text } => {
                    writeln!(writer, "<hick:user>{}</hick:user>", text.trim())?;
                }
                SessionEvent::Assistant { prose, action } => {
                    let prose = strip_protocol_tags(prose);
                    match action {
                        Some((lang, code)) => {
                            writeln!(writer, "<hick:assistant>")?;
                            if !prose.is_empty() {
                                writeln!(writer, "{prose}")?;
                            }
                            writeln!(writer, r#"<hick:action lang="{lang}">"#)?;
                            writeln!(writer, "{}", code.trim_end())?;
                            writeln!(writer, "</hick:action>")?;
                            writeln!(writer, "</hick:assistant>")?;
                        }
                        None => {
                            writeln!(writer, "<hick:assistant>{prose}</hick:assistant>")?;
                        }
                    }
                }
                SessionEvent::ToolCall { prose, xml } => {
                    let prose = strip_protocol_tags(prose);
                    writeln!(writer, "<hick:assistant>")?;
                    if !prose.is_empty() {
                        writeln!(writer, "{prose}")?;
                    }
                    writeln!(writer, "{}", xml.trim())?;
                    writeln!(writer, "</hick:assistant>")?;
                }
                SessionEvent::ToolResult { name, ok, text } => {
                    writeln!(writer, r#"<hick:tool-result name="{name}" ok="{ok}">"#)?;
                    writeln!(writer, "{}", text.trim_end())?;
                    writeln!(writer, "</hick:tool-result>")?;
                }
                SessionEvent::Observation {
                    source,
                    exit_code,
                    text,
                } => {
                    let exit = exit_code
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".into());
                    writeln!(
                        writer,
                        r#"<hick:observation source="{source}" exit="{exit}">{}</hick:observation>"#,
                        text.trim_end()
                    )?;
                }
                SessionEvent::End => {
                    writeln!(writer, "</hick:session>")?;
                }
            }
            writer.flush()
        })();
        let _ = result;
    }
}

// ---------------------------------------------------------------------------
// Session file naming
// ---------------------------------------------------------------------------

/// Build the conventional session file path for a project:
/// `<project_dir>/sessions/<timestamp>-<slug>.hick`, where the slug is
/// derived from the prompt.
pub fn session_file_path(project_dir: &Path, prompt: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
    let slug = slugify(prompt);
    project_dir
        .join("sessions")
        .join(format!("{timestamp}-{slug}.hick"))
}

/// Lowercase-alphanumeric-and-hyphens slug, capped at 40 characters.
fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut last_hyphen = true;
    for c in text.chars() {
        if slug.len() >= 40 {
            break;
        }
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_hyphen = false;
        } else if !last_hyphen {
            slug.push('-');
            last_hyphen = true;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "session".into()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_lang::SessionNode;
    use tempfile::tempdir;

    fn write_sample_session(path: &Path) {
        let log = HickSessionLog::create(path).unwrap();
        log.record(SessionEvent::User {
            text: "add a greeting file",
        });
        log.record(SessionEvent::Assistant {
            prose: "<hick:next>code</hick:next>\nI'll create it now.",
            action: Some(("python", "print('creating greeting')")),
        });
        log.record(SessionEvent::Observation {
            source: "action-0",
            exit_code: Some(0),
            text: "creating greeting",
        });
        log.record(SessionEvent::Assistant {
            prose: "Done — the greeting file exists.",
            action: None,
        });
        log.record(SessionEvent::End);
    }

    #[test]
    fn session_round_trips_through_hick_lang_parser() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sessions/test.hick");
        write_sample_session(&path);

        let source = std::fs::read_to_string(&path).unwrap();
        let doc = hick_lang::parse_session(&source).expect("session must parse");
        assert!(doc.start_time.is_some());
        assert_eq!(doc.nodes.len(), 4);
        assert_eq!(
            doc.nodes[0],
            SessionNode::User {
                text: "add a greeting file".into()
            }
        );
        match &doc.nodes[1] {
            SessionNode::Assistant { text, actions } => {
                assert_eq!(text, "I'll create it now.");
                assert_eq!(actions.len(), 1);
                assert_eq!(actions[0].lang, "python");
                assert!(actions[0].code.contains("creating greeting"));
            }
            other => panic!("expected Assistant, got {other:?}"),
        }
        match &doc.nodes[2] {
            SessionNode::Observation {
                source,
                exit_code,
                text,
            } => {
                assert_eq!(source.as_deref(), Some("action-0"));
                assert_eq!(*exit_code, Some(0));
                assert_eq!(text, "creating greeting");
            }
            other => panic!("expected Observation, got {other:?}"),
        }
        assert!(matches!(doc.nodes[3], SessionNode::Assistant { .. }));
        assert!(hick_lang::is_session_source(&source));
    }

    #[test]
    fn partial_session_is_readable_before_end() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("partial.hick");
        let log = HickSessionLog::create(&path).unwrap();
        log.record(SessionEvent::User { text: "hello" });
        // No End yet — the file on disk must still contain the turn.
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(source.contains("<hick:user>hello</hick:user>"));
    }

    #[test]
    fn slugify_produces_safe_names() {
        assert_eq!(slugify("Add a login page!"), "add-a-login-page");
        assert_eq!(slugify("///"), "session");
        assert!(slugify(&"x".repeat(100)).len() <= 40);
    }
}
