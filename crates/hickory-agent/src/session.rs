//! Session logging as literate programming.
//!
//! Every agent session is written **incrementally** as a `hick:session`
//! document — the same format `hick-lang` parses ([`hick_lang::parse_session`])
//! and `hick ingest --from session` turns into a clean pipeline. The session file is the
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

use anyhow::Context as _;
use chrono::Utc;

/// A single event to be recorded into a session log.
#[derive(Debug)]
pub enum SessionEvent<'a> {
    /// A user message (`<hick:user>`).
    User { text: &'a str },
    /// A user message that is one TURN of a conversation: carries the turn
    /// id, its parent's id (the branch point), and what it ran on — so the
    /// session file holds the tree and a restarted app can rebuild it.
    UserTurn {
        text: &'a str,
        turn: &'a str,
        parent: Option<&'a str>,
        provider: &'a str,
        model: &'a str,
    },
    /// A full LLM response (`<hick:assistant>`), with an optional embedded
    /// script block (`<hick:action lang="...">`).
    Assistant {
        /// Prose text of the response (excluding the code block).
        prose: &'a str,
        /// `(lang, code)` of the embedded action, if the response ran code.
        action: Option<(&'a str, &'a str)>,
        /// The model's reasoning for this response, when the provider
        /// exposed it (`<hick:reasoning>`, first child of the assistant
        /// element). Raw content, shown folded.
        reasoning: Option<&'a str>,
    },
    /// A tool-invoking LLM response: an `<hick:assistant>` element carrying
    /// the raw `<hick:tool>` invocation XML verbatim.
    ToolCall {
        /// Prose text of the response (excluding the tool element).
        prose: &'a str,
        /// The verbatim `<hick:tool>...</hick:tool>` XML.
        xml: &'a str,
        /// The model's reasoning, as on [`SessionEvent::Assistant`].
        reasoning: Option<&'a str>,
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
    /// Token usage + spend for one LLM call (`<hick:usage .../>`,
    /// self-closing — the session parser skips unknown tags, so old
    /// readers are unaffected). `turn` is `None` for the session total
    /// written at the end.
    Usage {
        /// Zero-based LLM turn index; `None` = session total.
        turn: Option<usize>,
        /// The four-way token split.
        usage: crate::usage::Usage,
        /// USD cost (`None` when the model has no known price).
        cost_usd: Option<f64>,
    },
    /// A tool showed the model a file (`<hick:read …/>`): what, at which
    /// content hash and commit, which lines. Derived by the tool.
    Read { read: &'a crate::tools::ContextRead },
    /// An edit tool wrote lines (`<hick:wrote …/>`): which file, which lines
    /// as they stand after the edit, and their hashline hashes — so a later
    /// reader can find them again, and can say that every `<hick:read>`
    /// earlier in this session was in front of the model when they were
    /// written.
    Wrote { wrote: &'a crate::tools::Wrote },
    /// Session ended — writes the closing `</hick:session>` tag.
    End,
}

/// Record a tool's outcome: the `<hick:tool-result>` the model sees, then
/// the context it leaves behind — what it showed (`<hick:read>`) or what it
/// wrote (`<hick:wrote>`). One call, so no recorder forgets the second half.
pub fn record_outcome(log: &dyn SessionLog, outcome: &crate::tools::ToolOutcome) {
    log.record(SessionEvent::ToolResult {
        name: &outcome.name,
        ok: outcome.ok,
        text: &outcome.text,
    });
    for read in &outcome.reads {
        log.record(SessionEvent::Read { read });
    }
    if let Some(wrote) = &outcome.wrote {
        log.record(SessionEvent::Wrote { wrote });
    }
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
    /// Every INPUT to the conversation — the user's words, a script's
    /// observation, a tool's result — gets `id="in<n>"`, so context
    /// provenance can point at it and a reader can find it. Seeded from the
    /// file when appending, so ids stay unique across processes.
    inputs: std::sync::atomic::AtomicUsize,
}

/// How many input elements a session file already holds.
fn count_inputs(existing: &str) -> usize {
    ["<hick:user", "<hick:observation", "<hick:tool-result"]
        .iter()
        .map(|tag| existing.matches(tag).count())
        .sum()
}

impl HickSessionLog {
    /// Create the session file at `path` (parent directories are created)
    /// and write the document header.
    pub fn create(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        Self::create_for(path, None)
    }

    /// [`create`](Self::create), naming the document the session is about on
    /// the root element (`doc="…"`), which is how the conversation is found
    /// again for that document.
    pub fn create_for(path: impl Into<PathBuf>, doc: Option<&Path>) -> anyhow::Result<Self> {
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
        let doc_attr = doc
            .map(|d| format!(r#" doc="{}""#, d.display()))
            .unwrap_or_default();
        writeln!(
            writer,
            r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="{start}"{doc_attr}>"#
        )?;
        writer.flush()?;
        Ok(Self {
            inner: Mutex::new(writer),
            path,
            inputs: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    fn next_input_id(&self) -> String {
        let n = self
            .inputs
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("in{n}")
    }

    /// Open an existing session file to append to, or create a new one.
    ///
    /// This is what makes a session survive across *processes*. The built-in
    /// agent holds one log open for a whole run; an external coding agent
    /// makes one `hick doc` call per edit, each in a process of its own,
    /// and without this each call would either truncate the session or refuse
    /// to write it. The session is the product — losing it because the work
    /// came from Claude Code rather than from our loop would make "bring your
    /// own agent" a second-class path in exactly the place it matters.
    ///
    /// A closed session (`</hick:session>` at the end) is reopened by dropping
    /// that line, so the file on disk is a valid, parseable document after
    /// every command rather than only after the last one.
    pub fn append_or_create(path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        Self::append_or_create_for(path, None)
    }

    /// [`append_or_create`](Self::append_or_create), naming the document on
    /// a freshly created root.
    pub fn append_or_create_for(
        path: impl Into<PathBuf>,
        doc: Option<&Path>,
    ) -> anyhow::Result<Self> {
        use std::io::Write as _;

        let path = path.into();
        if !path.exists() {
            return Self::create_for(path, doc);
        }
        let existing = std::fs::read_to_string(&path)
            .with_context(|| format!("reading the session file {}", path.display()))?;
        // Refuse to append to a file that is not a session. Silently turning
        // someone's document into a session log is unrecoverable.
        if !existing.contains("<hick:session") {
            anyhow::bail!(
                "{} is not a hick:session document — refusing to append to it. \
                 Point HICKORY_SESSION at a new file under sessions/.",
                path.display()
            );
        }
        let reopened = match existing.rfind("</hick:session>") {
            Some(at) => existing[..at].to_string(),
            None => existing,
        };
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&path)?;
        let seed = count_inputs(&reopened);
        file.write_all(reopened.as_bytes())?;
        let mut writer = BufWriter::new(file);
        writer.flush()?;
        Ok(Self {
            inner: Mutex::new(writer),
            path,
            inputs: std::sync::atomic::AtomicUsize::new(seed),
        })
    }

    /// The path of the session file being written.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// `<hick:reasoning>` as the first child of an assistant element, when the
/// model exposed any. Raw content: a thought that mentions a tag is a thought,
/// not structure.
fn write_reasoning(
    writer: &mut impl std::io::Write,
    reasoning: Option<&str>,
) -> std::io::Result<()> {
    if let Some(r) = reasoning
        && !r.trim().is_empty()
    {
        writeln!(writer, "<hick:reasoning>")?;
        writeln!(writer, "{}", r.trim_end())?;
        writeln!(writer, "</hick:reasoning>")?;
    }
    Ok(())
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
                    let id = self.next_input_id();
                    writeln!(
                        writer,
                        "<hick:user id=\"{id}\">{}</hick:user>",
                        safe_prose(text.trim())
                    )?;
                }
                SessionEvent::UserTurn {
                    text,
                    turn,
                    parent,
                    provider,
                    model,
                } => {
                    let id = self.next_input_id();
                    let parent = parent
                        .map(|p| format!(r#" parent="{p}""#))
                        .unwrap_or_default();
                    writeln!(
                        writer,
                        r#"<hick:user id="{id}" turn="{turn}"{parent} provider="{provider}" model="{model}">{}</hick:user>"#,
                        safe_prose(text.trim())
                    )?;
                }
                SessionEvent::Assistant {
                    prose,
                    action,
                    reasoning,
                } => {
                    let prose = safe_prose(&strip_protocol_tags(prose));
                    match (action, reasoning) {
                        (None, None) => {
                            writeln!(writer, "<hick:assistant>{prose}</hick:assistant>")?;
                        }
                        (action, reasoning) => {
                            writeln!(writer, "<hick:assistant>")?;
                            write_reasoning(&mut *writer, reasoning)?;
                            if !prose.is_empty() {
                                writeln!(writer, "{prose}")?;
                            }
                            if let Some((lang, code)) = action {
                                writeln!(writer, r#"<hick:action lang="{lang}">"#)?;
                                writeln!(writer, "{}", code.trim_end())?;
                                writeln!(writer, "</hick:action>")?;
                            }
                            writeln!(writer, "</hick:assistant>")?;
                        }
                    }
                }
                SessionEvent::ToolCall {
                    prose,
                    xml,
                    reasoning,
                } => {
                    let prose = safe_prose(&strip_protocol_tags(prose));
                    writeln!(writer, "<hick:assistant>")?;
                    write_reasoning(&mut *writer, reasoning)?;
                    if !prose.is_empty() {
                        writeln!(writer, "{prose}")?;
                    }
                    writeln!(writer, "{}", xml.trim())?;
                    writeln!(writer, "</hick:assistant>")?;
                }
                SessionEvent::ToolResult { name, ok, text } => {
                    let id = self.next_input_id();
                    writeln!(
                        writer,
                        r#"<hick:tool-result id="{id}" name="{name}" ok="{ok}">"#
                    )?;
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
                    let id = self.next_input_id();
                    writeln!(
                        writer,
                        r#"<hick:observation id="{id}" source="{source}" exit="{exit}">{}</hick:observation>"#,
                        text.trim_end()
                    )?;
                }
                SessionEvent::Usage {
                    turn,
                    usage,
                    cost_usd,
                } => {
                    let scope = match turn {
                        Some(t) => format!(r#"turn="{t}""#),
                        None => r#"scope="session""#.to_string(),
                    };
                    let cost = cost_usd
                        .map(|c| format!(r#" cost-usd="{c:.6}""#))
                        .unwrap_or_default();
                    writeln!(
                        writer,
                        r#"<hick:usage {scope} input="{}" cache-write="{}" cache-read="{}" output="{}"{cost}/>"#,
                        usage.input_tokens,
                        usage.cache_creation_input_tokens,
                        usage.cache_read_input_tokens,
                        usage.output_tokens,
                    )?;
                }
                SessionEvent::Read { read } => {
                    let commit = read
                        .commit
                        .as_deref()
                        .map(|c| format!(r#" commit="{c}""#))
                        .unwrap_or_default();
                    writeln!(
                        writer,
                        r#"<hick:read file="{}"{commit} sha256="{}" lines="{}-{}"/>"#,
                        read.path, read.sha256, read.first_line, read.last_line
                    )?;
                }
                SessionEvent::Wrote { wrote } => {
                    writeln!(
                        writer,
                        r#"<hick:wrote file="{}" lines="{}-{}" hashes="{}"/>"#,
                        wrote.file,
                        wrote.first_line,
                        wrote.last_line,
                        wrote.hashes.join(" ")
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

/// Does this text quote a hick tag (or an XML comment), such that writing it
/// straight into an element the parser does NOT capture verbatim would break
/// the session?
///
/// `<!--` counts: the parser skips comments, and an unclosed one swallows the
/// rest of the document.
pub fn quotes_hick(text: &str) -> bool {
    text.contains("<hick:") || text.contains("</hick:") || text.contains("<!--")
}

/// Prose for an element that is not captured verbatim, made safe to write.
///
/// A model explaining `<hick:exec>` — which the model of a tool for writing
/// hick documents does constantly — used to be written straight into
/// `<hick:assistant>`, where the parser reads it as a real opening tag that
/// never closes. The session then failed to parse at its own
/// `</hick:assistant>`, and the whole conversation was lost to every reader.
/// Two of this repository's committed example sessions are already in that
/// state.
///
/// The answer is the one `hick ingest --from claude-code` already uses for imported
/// transcripts: wrap it in `hick:input`, which IS a verbatim-capture element.
/// Both session readers walk through the wrapper, so the text still reads as
/// the same prose. Nothing is escaped and no byte is rewritten — the
/// no-escaping invariant is untouched.
fn safe_prose(text: &str) -> String {
    if quotes_hick(text) {
        // A body quoting its own close tag would end the wrapper early; one
        // space inside the token stops the parser's literal search and still
        // reads correctly.
        let body = text.replace("</hick:input>", "</hick:input >");
        format!("<hick:input>\n{body}\n</hick:input>")
    } else {
        text.to_string()
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

    /// A model explaining hick syntax must not destroy the session recording
    /// it.
    ///
    /// This is not hypothetical: two committed example sessions under
    /// `examples/receipts/hick-agent/sessions/` are unparseable because the
    /// model wrote "`<hick:exec>`" in an answer, and the parser read it as an
    /// opening tag that never closed.
    #[test]
    fn an_answer_that_quotes_a_hick_tag_still_parses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sessions/s.hick");
        let log = HickSessionLog::append_or_create(&path).unwrap();
        log.record(SessionEvent::User {
            text: "What does <hick:exec> do?",
        });
        log.record(SessionEvent::Assistant {
            prose: "It runs a cell. Write it as <hick:exec container=\"x\">.",
            action: None,
            reasoning: None,
        });
        log.record(SessionEvent::End);
        drop(log);

        let source = std::fs::read_to_string(&path).unwrap();
        let doc = hick_lang::parse_session(&source).expect("a quoted tag is prose, not structure");

        let hick_lang::SessionNode::User { text } = &doc.nodes[0] else {
            panic!("expected a user turn, got {:?}", doc.nodes[0]);
        };
        assert_eq!(text.trim(), "What does <hick:exec> do?");

        let hick_lang::SessionNode::Assistant { text, .. } = &doc.nodes[1] else {
            panic!("expected an assistant turn, got {:?}", doc.nodes[1]);
        };
        assert_eq!(
            text.trim(),
            "It runs a cell. Write it as <hick:exec container=\"x\">."
        );
    }

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
            reasoning: None,
        });
        log.record(SessionEvent::Observation {
            source: "action-0",
            exit_code: Some(0),
            text: "creating greeting",
        });
        log.record(SessionEvent::Assistant {
            prose: "Done — the greeting file exists.",
            action: None,
            reasoning: None,
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
        assert!(
            source.contains("<hick:user id=\"in0\">hello</hick:user>"),
            "{source}"
        );
    }

    #[test]
    fn usage_elements_do_not_break_session_parsing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("usage.hick");
        let log = HickSessionLog::create(&path).unwrap();
        log.record(SessionEvent::User { text: "hi" });
        log.record(SessionEvent::Usage {
            turn: Some(0),
            usage: crate::usage::Usage {
                input_tokens: 12,
                cache_creation_input_tokens: 3,
                cache_read_input_tokens: 900,
                output_tokens: 40,
            },
            cost_usd: Some(0.001234),
        });
        log.record(SessionEvent::Assistant {
            prose: "done",
            action: None,
            reasoning: None,
        });
        log.record(SessionEvent::Usage {
            turn: None,
            usage: crate::usage::Usage::default(),
            cost_usd: None,
        });
        log.record(SessionEvent::End);

        let source = std::fs::read_to_string(&path).unwrap();
        assert!(source.contains(r#"<hick:usage turn="0" input="12" cache-write="3" cache-read="900" output="40" cost-usd="0.001234"/>"#));
        assert!(source.contains(r#"<hick:usage scope="session""#));
        // The session parser skips unknown tags: user + assistant survive.
        let doc = hick_lang::parse_session(&source).expect("session with usage must parse");
        assert_eq!(doc.nodes.len(), 2);
    }

    #[test]
    fn slugify_produces_safe_names() {
        assert_eq!(slugify("Add a login page!"), "add-a-login-page");
        assert_eq!(slugify("///"), "session");
        assert!(slugify(&"x".repeat(100)).len() <= 40);
    }
}
