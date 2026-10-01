//! `hick ingest --from claude-code`: a Claude Code transcript, as a `hick:session`.
//!
//! Claude Code keeps every conversation as a JSONL file under
//! `~/.claude/projects/<project>/<session-id>.jsonl` — one JSON object per
//! line, each a record of something that happened: what you typed, what the
//! model said, thought, and called, what the tools answered, and a long tail
//! of harness bookkeeping (token reminders, mode changes, file-history
//! snapshots). This module reads one of those files and writes the same
//! conversation as the session document the built-in agent would have
//! written, so that it opens in the app as a conversation — bubbles, folded
//! work, line numbers — and so that `hick context` can treat it as evidence
//! of what a model was shown.
//!
//! ## What maps to what
//!
//! | Claude Code record                      | hick:session element                          |
//! |-----------------------------------------|-----------------------------------------------|
//! | a prompt you typed (`user`, not meta)   | `<hick:user turn= parent= provider= model= at=>` |
//! | `assistant` text block                  | prose inside `<hick:assistant>`               |
//! | `assistant` thinking block              | `<hick:reasoning>` inside the assistant turn  |
//! | `assistant` tool_use block              | `<hick:tool name= call=>` with `<hick:arg>`s  |
//! | `user` tool_result block                | `<hick:tool-result name= call= ok=>`          |
//! | `assistant` usage                       | `<hick:usage turn= …/>`                       |
//! | compaction, meta prompts, attachments   | `<hick:context kind= at=>`                    |
//! | `ai-title`                              | the session's `title=` and the file's name    |
//!
//! The turn TREE survives: every Claude Code record names its `parentUuid`,
//! so each prompt's `parent=` is the nearest prompt above it on its own
//! branch — a rewound conversation imports as the branches it was, the way
//! the dock draws its own.
//!
//! ## What is kept, and what is not
//!
//! **Kept verbatim**: every prompt, every piece of prose, every thought the
//! model exposed, every tool call with its full input, every tool result,
//! every token count. **Kept as context**: compaction summaries and
//! boundaries, the harness's meta prompts, hook output, files and snippets
//! the harness attached to a turn. **Dropped, and counted**: image bytes (a
//! placeholder names the type and size), thinking signatures (opaque
//! per-provider blobs), `total_tokens_reminder` and `task_reminder`
//! attachments (hundreds per session, all saying "you have N tokens left"),
//! and harness bookkeeping that describes the harness rather than the
//! conversation (mode changes, last-prompt, file-history snapshots,
//! frame-links). Everything dropped is reported on stderr, because a file
//! that says "imported" and is quietly missing half the conversation is the
//! worst result this command could produce.
//!
//! ## The no-escaping invariant, and what it costs here
//!
//! A `.md` document never escapes: only `hick:`-prefixed tags are
//! structure, and a tool result that prints a `.md` file prints real tags.
//! The parser captures four elements verbatim — `hick:input`,
//! `hick:tool-result`, `hick:reasoning`, `hick:context` — and those carry
//! anything. `hick:user` prose, `hick:assistant` prose and `hick:arg` do NOT,
//! so a prompt or an answer that quotes a hick tag is wrapped in
//! `<hick:input>` (its text still reads as the prompt or the prose), and a
//! tool call whose arguments quote one is written as one `<hick:input>`
//! holding the whole input as JSON. The one thing a verbatim element cannot
//! hold is its own closing tag — a tool result that prints a session file
//! would end early — so that exact token is broken with a space inside the
//! body (`</hick:input >`), the one byte-level difference this import ever
//! makes, and it is counted and reported. Transcripts of work ON hick
//! documents hit this constantly; transcripts of anything else never do.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use serde_json::Value;

/// The hick namespace every session document binds.
const NS: &str = "http://www.hickorydocs.com/1.0";

/// Attachment kinds that are reminders to the model, not conversation. There
/// are hundreds per session and they all say the same thing.
const SKIPPED_ATTACHMENTS: &[&str] = &["total_tokens_reminder", "task_reminder"];

/// Record kinds that describe the harness rather than the conversation.
const SKIPPED_RECORDS: &[&str] = &[
    "mode",
    "permission-mode",
    "last-prompt",
    "file-history-snapshot",
    "file-history-delta",
    "frame-link",
    "atis-latch",
    "bridge-session",
    "agent-name",
    "relocated",
    "worktree-state",
];

/// What an import produced, and what it left out.
#[derive(Debug, Default)]
pub struct Stats {
    pub prompts: usize,
    pub assistant_messages: usize,
    pub reasoning: usize,
    pub tool_calls: usize,
    pub tool_results: usize,
    pub context: usize,
    /// Things not carried into the document, by reason, with counts.
    pub dropped: BTreeMap<String, usize>,
    /// Things carried but changed (a prompt wrapped in `hick:input`, a
    /// tool input written as JSON), by reason, with counts.
    pub adapted: BTreeMap<String, usize>,
    /// Why the result cannot be trusted: the document does not parse, or
    /// reads back as a different number of turns than were written. A
    /// converted document with problems is not written to `sessions/`;
    /// `--stdout` still prints it so the problem can be found.
    pub problems: Vec<String>,
}

impl Stats {
    fn drop(&mut self, why: impl Into<String>) {
        *self.dropped.entry(why.into()).or_default() += 1;
    }
    fn adapt(&mut self, why: impl Into<String>) {
        *self.adapted.entry(why.into()).or_default() += 1;
    }
}

/// A converted transcript.
#[derive(Debug)]
pub struct Converted {
    /// The session document.
    pub hick: String,
    /// Claude Code's own title for the session, when it wrote one.
    pub title: Option<String>,
    /// Claude Code's session id.
    pub session_id: Option<String>,
    /// When the first prompt was typed (ISO 8601), for the file name.
    pub started: Option<String>,
    /// The first prompt, for the file name when there is no title.
    pub first_prompt: Option<String>,
    pub stats: Stats,
}

impl Converted {
    /// `<YYYYMMDD-HHMMSS>-<slug>.md`, the same shape the agent's own
    /// sessions have, so the folder sorts by when things happened. The slug
    /// is the title when Claude Code wrote one, else the first prompt.
    /// Deterministic: importing the same transcript twice names one file.
    pub fn file_name(&self) -> String {
        let stamp = self
            .started
            .as_deref()
            .and_then(stamp_of)
            .unwrap_or_else(|| "00000000-000000".to_string());
        let seed = self
            .title
            .as_deref()
            .or(self.first_prompt.as_deref())
            .unwrap_or("session");
        format!("{stamp}-{}.md", slug(seed))
    }
}

/// `2026-08-13T16:29:17.371Z` → `20260813-162917`.
fn stamp_of(iso: &str) -> Option<String> {
    let (date, time) = iso.split_once('T')?;
    let date: String = date.chars().filter(char::is_ascii_digit).collect();
    let time: String = time
        .split(['.', 'Z', '+'])
        .next()?
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    if date.len() != 8 || time.len() != 6 {
        return None;
    }
    Some(format!("{date}-{time}"))
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut last_hyphen = true;
    for c in text.chars() {
        if out.len() >= 40 {
            break;
        }
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_hyphen = false;
        } else if !last_hyphen {
            out.push('-');
            last_hyphen = true;
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() {
        "session".into()
    } else {
        out
    }
}

// ---------------------------------------------------------------------------
// Reading the transcript
// ---------------------------------------------------------------------------

/// One line of the JSONL, with the fields every branch of the conversion
/// reads pulled out once.
struct Record {
    kind: String,
    uuid: Option<String>,
    parent: Option<String>,
    at: Option<String>,
    value: Value,
}

fn str_of(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

fn read_records(jsonl: &str, stats: &mut Stats) -> Vec<Record> {
    let mut out = Vec::new();
    for line in jsonl.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            stats.drop("line that is not JSON");
            continue;
        };
        let kind = str_of(&value, "type").unwrap_or_default();
        // A compaction boundary has no parentUuid but names what it
        // logically follows; that keeps the tree joined across it.
        let parent = str_of(&value, "parentUuid").or_else(|| str_of(&value, "logicalParentUuid"));
        out.push(Record {
            kind,
            uuid: str_of(&value, "uuid"),
            parent,
            at: str_of(&value, "timestamp"),
            value,
        });
    }
    out
}

/// The blocks of a `message.content`: a bare string is one text block.
fn content_blocks(message: &Value) -> Vec<Value> {
    match message.get("content") {
        Some(Value::String(s)) => vec![serde_json::json!({"type": "text", "text": s})],
        Some(Value::Array(items)) => items.clone(),
        _ => Vec::new(),
    }
}

/// Is this `user` record something the person typed (or the harness typed
/// for them as a prompt), rather than a tool result or a meta note?
fn is_prompt(rec: &Record) -> bool {
    if rec.kind != "user" {
        return false;
    }
    if rec
        .value
        .get("isMeta")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return false;
    }
    if rec
        .value
        .get("isCompactSummary")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return false;
    }
    let Some(message) = rec.value.get("message") else {
        return false;
    };
    let blocks = content_blocks(message);
    if blocks.is_empty()
        || blocks
            .iter()
            .any(|b| b.get("type").and_then(Value::as_str) == Some("tool_result"))
    {
        return false;
    }
    !is_harness_echo(&blocks)
}

/// A `user` record the harness wrote for itself: a slash command's echo
/// (`<command-name>/model</command-name>…`) or its local output
/// (`<local-command-stdout>…`). Nobody typed these as a prompt to the model;
/// they are context, and they are imported as such.
fn is_harness_echo(blocks: &[Value]) -> bool {
    let Some(first) = blocks.first() else {
        return false;
    };
    let text = str_of(first, "text").unwrap_or_default();
    let text = text.trim_start();
    text.starts_with("<command-name>")
        || text.starts_with("<command-message>")
        || text.starts_with("<local-command-stdout>")
        || text.starts_with("<local-command-caveat>")
}

// ---------------------------------------------------------------------------
// Writing hick
// ---------------------------------------------------------------------------

/// An attribute, quoted with whichever quote the value does not contain.
/// A value containing both cannot be written (no escaping) and is dropped.
fn attr(out: &mut String, name: &str, value: &str, stats: &mut Stats) {
    let value: String = value
        .chars()
        .map(|c| {
            if c == '\n' || c == '\r' || c == '\t' {
                ' '
            } else {
                c
            }
        })
        .collect();
    if !value.contains('"') {
        let _ = write!(out, r#" {name}="{value}""#);
    } else if !value.contains('\'') {
        let _ = write!(out, " {name}='{value}'");
    } else {
        stats.drop(format!("attribute `{name}` holding both quote kinds"));
    }
}

/// Does this text, placed as ordinary content, parse as structure? A `hick:`
/// tag does, and so does `<!--`, which the parser skips as a comment (and an
/// unclosed one swallows the rest of the document). Any other angle bracket
/// is raw text.
fn quotes_hick(text: &str) -> bool {
    text.contains("<hick:") || text.contains("</hick:") || text.contains("<!--")
}

/// Text that may quote hick tags, inside an element that is NOT captured
/// verbatim: wrap it in `hick:input` (which is) when it needs it. The wrapper
/// reads as the same text — `text_content` walks through it.
fn prose(out: &mut String, text: &str, what: &str, stats: &mut Stats) {
    let text = text.trim_matches('\n');
    if quotes_hick(text) {
        stats.adapt(format!("{what} quoting a hick tag, wrapped in hick:input"));
        let _ = writeln!(out, "<hick:input>");
        let mut body = String::new();
        raw(&mut body, "input", text, stats);
        out.push_str(&body);
        let _ = write!(out, "</hick:input>");
    } else {
        let _ = write!(out, "{text}");
    }
}

/// A verbatim element's body: anything but its own close tag. A body that
/// quotes that exact token — a tool that printed a session file, an edit
/// that wrote one — would end the element early, so the token is broken
/// with a space inside the body (`</hick:input >`), which the parser's
/// literal search no longer matches and a reader still reads. It is the one
/// place an imported byte differs from the transcript, and it is counted.
fn raw(out: &mut String, element: &str, text: &str, stats: &mut Stats) {
    let close = format!("</hick:{element}>");
    let text = if text.contains(&close) {
        stats.adapt(format!(
            "{element} body quoting its own close tag, broken with a space ({close} → </hick:{element} >)"
        ));
        text.replace(&close, &format!("</hick:{element} >"))
    } else {
        text.to_string()
    };
    let _ = writeln!(out, "{}", text.trim_end_matches('\n'));
}

/// The text of a tool result's content — a string, or blocks.
fn tool_result_text(content: &Value, stats: &mut Stats) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => {
            let mut parts = Vec::new();
            for b in blocks {
                match b.get("type").and_then(Value::as_str) {
                    Some("text") => parts.push(str_of(b, "text").unwrap_or_default()),
                    Some("image") => parts.push(image_placeholder(b, stats)),
                    Some("tool_reference") => parts.push(format!(
                        "[tool reference: {}]",
                        str_of(b, "tool_name").unwrap_or_default()
                    )),
                    _ => parts.push(b.to_string()),
                }
            }
            parts.join("\n")
        }
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn image_placeholder(block: &Value, stats: &mut Stats) -> String {
    stats.drop("image bytes (placeholder written)");
    let source = block.get("source");
    let media = source
        .and_then(|s| str_of(s, "media_type"))
        .unwrap_or_else(|| "image".into());
    let size = source
        .and_then(|s| s.get("data"))
        .and_then(Value::as_str)
        .map(|d| d.len() * 3 / 4)
        .unwrap_or(0);
    format!("[image: {media}, {size} bytes]")
}

/// What an attachment record showed the model, as text.
fn attachment_text(att: &Value) -> String {
    if let Some(s) = att.get("content").and_then(Value::as_str) {
        return s.to_string();
    }
    // A file the harness attached: its name and its content.
    if let Some(file) = att.get("content").and_then(|c| c.get("file")) {
        let name = str_of(file, "filePath").unwrap_or_default();
        let body = str_of(file, "content").unwrap_or_default();
        return format!("{name}\n{body}");
    }
    if let Some(snippet) = att.get("snippet").and_then(Value::as_str) {
        let name = str_of(att, "filename").unwrap_or_default();
        return format!("{name}\n{snippet}");
    }
    // Anything else: the attachment as it was, minus the type we already
    // named.
    let mut rest = att.clone();
    if let Some(obj) = rest.as_object_mut() {
        obj.remove("type");
    }
    serde_json::to_string_pretty(&rest).unwrap_or_default()
}

fn context(out: &mut String, kind: &str, at: Option<&str>, text: &str, stats: &mut Stats) {
    if text.trim().is_empty() {
        return;
    }
    stats.context += 1;
    out.push_str("<hick:context");
    attr(out, "kind", kind, stats);
    if let Some(at) = at {
        attr(out, "at", at, stats);
    }
    out.push_str(">\n");
    raw(out, "context", text, stats);
    out.push_str("</hick:context>\n");
}

/// Convert one Claude Code transcript to a session document.
pub fn convert(jsonl: &str) -> Result<Converted> {
    let mut stats = Stats::default();
    let records = read_records(jsonl, &mut stats);
    if records.is_empty() {
        bail!(
            "the file holds no records — is it a Claude Code transcript (one JSON object per line)?"
        );
    }

    // First pass: what every branch needs to know about the whole file.
    let mut title = None;
    let mut session_id = None;
    let mut cwd = None;
    let mut branch = None;
    let mut version = None;
    let mut tool_names: HashMap<String, String> = HashMap::new();
    let mut parent_of: HashMap<String, Option<String>> = HashMap::new();
    let mut prompt_uuids: std::collections::HashSet<String> = Default::default();
    // The model a prompt ran on is the model of the reply that follows it.
    let mut model_after: HashMap<String, String> = HashMap::new();
    let mut awaiting_model: Vec<String> = Vec::new();
    for rec in &records {
        if let Some(u) = &rec.uuid {
            parent_of.insert(u.clone(), rec.parent.clone());
        }
        session_id = session_id.or_else(|| str_of(&rec.value, "sessionId"));
        cwd = cwd.or_else(|| str_of(&rec.value, "cwd"));
        branch = branch.or_else(|| str_of(&rec.value, "gitBranch"));
        version = version.or_else(|| str_of(&rec.value, "version"));
        match rec.kind.as_str() {
            "ai-title" => title = str_of(&rec.value, "aiTitle").or(title),
            "assistant" => {
                if let Some(message) = rec.value.get("message") {
                    if let Some(model) = str_of(message, "model") {
                        for u in awaiting_model.drain(..) {
                            model_after.insert(u, model.clone());
                        }
                    }
                    for b in content_blocks(message) {
                        if b.get("type").and_then(Value::as_str) == Some("tool_use")
                            && let (Some(id), Some(name)) = (str_of(&b, "id"), str_of(&b, "name"))
                        {
                            tool_names.insert(id, name);
                        }
                    }
                }
            }
            "user" if is_prompt(rec) => {
                if let Some(u) = &rec.uuid {
                    prompt_uuids.insert(u.clone());
                    awaiting_model.push(u.clone());
                }
            }
            _ => {}
        }
    }

    // The nearest prompt above a record on its own branch.
    let parent_prompt = |rec: &Record| -> Option<String> {
        let mut cursor = rec.parent.clone();
        let mut hops = 0;
        while let Some(p) = cursor {
            if prompt_uuids.contains(&p) {
                return Some(p);
            }
            hops += 1;
            if hops > 100_000 {
                return None;
            }
            cursor = parent_of.get(&p).cloned().flatten();
        }
        None
    };

    let started = records
        .iter()
        .find(|r| is_prompt(r))
        .and_then(|r| r.at.clone())
        .or_else(|| records.iter().find_map(|r| r.at.clone()));

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<hick:session");
    attr(&mut out, "xmlns:hick", NS, &mut stats);
    if let Some(s) = &started {
        attr(&mut out, "start", s, &mut stats);
    }
    attr(&mut out, "source", "claude-code", &mut stats);
    if let Some(id) = &session_id {
        attr(&mut out, "session", id, &mut stats);
    }
    if let Some(v) = &cwd {
        attr(&mut out, "cwd", v, &mut stats);
    }
    if let Some(v) = &branch {
        attr(&mut out, "branch", v, &mut stats);
    }
    if let Some(v) = &version {
        attr(&mut out, "harness-version", v, &mut stats);
    }
    if let Some(t) = &title {
        attr(&mut out, "title", t, &mut stats);
    }
    out.push_str(">\n");

    let mut input_id = 0usize;
    let mut llm_turn = 0usize;
    let mut first_prompt = None;
    let mut open_message: Option<String> = None; // message.id of the open <hick:assistant>
    let mut open_usage: Option<Value> = None;

    // Close the assistant element being assembled, writing its usage row.
    let close_assistant = |out: &mut String,
                           open: &mut Option<String>,
                           usage: &mut Option<Value>,
                           llm_turn: &mut usize,
                           stats: &mut Stats| {
        if open.take().is_none() {
            return;
        }
        out.push_str("</hick:assistant>\n");
        if let Some(u) = usage.take() {
            let n = |k: &str| u.get(k).and_then(Value::as_u64).unwrap_or(0);
            let _ = writeln!(
                out,
                r#"<hick:usage turn="{}" input="{}" cache-write="{}" cache-read="{}" output="{}"/>"#,
                *llm_turn,
                n("input_tokens"),
                n("cache_creation_input_tokens"),
                n("cache_read_input_tokens"),
                n("output_tokens"),
            );
        }
        *llm_turn += 1;
        stats.assistant_messages += 1;
    };

    for rec in &records {
        let at = rec.at.as_deref();
        // Anything that is not an assistant block ends the assistant message
        // being assembled.
        let continues_assistant = rec.kind == "assistant"
            && open_message.is_some()
            && rec.value.get("message").and_then(|m| str_of(m, "id")) == open_message;
        if !continues_assistant {
            close_assistant(
                &mut out,
                &mut open_message,
                &mut open_usage,
                &mut llm_turn,
                &mut stats,
            );
        }

        match rec.kind.as_str() {
            "user" => {
                let Some(message) = rec.value.get("message") else {
                    stats.drop("user record without a message");
                    continue;
                };
                let blocks = content_blocks(message);
                if is_prompt(rec) {
                    stats.prompts += 1;
                    input_id += 1;
                    let mut text_parts = Vec::new();
                    for b in &blocks {
                        match b.get("type").and_then(Value::as_str) {
                            Some("text") => text_parts.push(str_of(b, "text").unwrap_or_default()),
                            Some("image") => text_parts.push(image_placeholder(b, &mut stats)),
                            other => {
                                stats
                                    .drop(format!("prompt block of type {}", other.unwrap_or("?")));
                            }
                        }
                    }
                    let text = text_parts.join("\n\n");
                    if first_prompt.is_none() {
                        first_prompt = Some(text.clone());
                    }
                    out.push_str("<hick:user");
                    attr(&mut out, "id", &input_id.to_string(), &mut stats);
                    if let Some(u) = &rec.uuid {
                        attr(&mut out, "turn", u, &mut stats);
                        if let Some(p) = parent_prompt(rec) {
                            attr(&mut out, "parent", &p, &mut stats);
                        }
                        attr(&mut out, "provider", "anthropic", &mut stats);
                        if let Some(m) = model_after.get(u) {
                            attr(&mut out, "model", m, &mut stats);
                        }
                    }
                    if let Some(at) = at {
                        attr(&mut out, "at", at, &mut stats);
                    }
                    if rec.value.get("interruptedMessageId").is_some() {
                        attr(&mut out, "interrupts", "true", &mut stats);
                    }
                    out.push('>');
                    prose(&mut out, text.trim(), "prompt", &mut stats);
                    out.push_str("</hick:user>\n");
                    continue;
                }
                // Tool results ride in user records; meta and compaction
                // prompts are context the harness put in front of the model.
                let mut had_result = false;
                for b in &blocks {
                    match b.get("type").and_then(Value::as_str) {
                        Some("tool_result") => {
                            had_result = true;
                            stats.tool_results += 1;
                            input_id += 1;
                            let call = str_of(b, "tool_use_id").unwrap_or_default();
                            let name = tool_names.get(&call).cloned().unwrap_or_else(|| "?".into());
                            let ok = !b.get("is_error").and_then(Value::as_bool).unwrap_or(false);
                            let text = tool_result_text(
                                b.get("content").unwrap_or(&Value::Null),
                                &mut stats,
                            );
                            out.push_str("<hick:tool-result");
                            attr(&mut out, "id", &input_id.to_string(), &mut stats);
                            attr(&mut out, "name", &name, &mut stats);
                            attr(&mut out, "call", &call, &mut stats);
                            attr(
                                &mut out,
                                "ok",
                                if ok { "true" } else { "false" },
                                &mut stats,
                            );
                            if let Some(at) = at {
                                attr(&mut out, "at", at, &mut stats);
                            }
                            out.push_str(">\n");
                            raw(&mut out, "tool-result", &text, &mut stats);
                            out.push_str("</hick:tool-result>\n");
                        }
                        Some("text") => {
                            let kind = if rec
                                .value
                                .get("isCompactSummary")
                                .and_then(Value::as_bool)
                                .unwrap_or(false)
                            {
                                "compact-summary"
                            } else if is_harness_echo(&blocks) {
                                "local-command"
                            } else {
                                "meta"
                            };
                            let text = str_of(b, "text").unwrap_or_default();
                            context(&mut out, kind, at, &text, &mut stats);
                        }
                        other => stats.drop(format!("user block of type {}", other.unwrap_or("?"))),
                    }
                }
                if !had_result && blocks.is_empty() {
                    stats.drop("user record with empty content");
                }
            }
            "assistant" => {
                let Some(message) = rec.value.get("message") else {
                    stats.drop("assistant record without a message");
                    continue;
                };
                let id = str_of(message, "id");
                if open_message.is_none() {
                    out.push_str("<hick:assistant");
                    if let Some(at) = at {
                        attr(&mut out, "at", at, &mut stats);
                    }
                    if let Some(m) = str_of(message, "model") {
                        attr(&mut out, "model", &m, &mut stats);
                    }
                    if let Some(m) = &id {
                        attr(&mut out, "message", m, &mut stats);
                    }
                    out.push_str(">\n");
                    open_message = Some(id.clone().unwrap_or_default());
                }
                // Usage arrives on every block's record; the last one is the
                // whole message's. Stop reason likewise.
                if let Some(u) = message.get("usage") {
                    open_usage = Some(u.clone());
                }
                for b in content_blocks(message) {
                    match b.get("type").and_then(Value::as_str) {
                        Some("text") => {
                            let text = str_of(&b, "text").unwrap_or_default();
                            if text.trim().is_empty() {
                                continue;
                            }
                            prose(&mut out, &text, "assistant prose", &mut stats);
                            out.push('\n');
                        }
                        Some("thinking") => {
                            let text = str_of(&b, "thinking").unwrap_or_default();
                            if b.get("signature").is_some() {
                                stats.drop("thinking signature");
                            }
                            if text.trim().is_empty() {
                                continue;
                            }
                            stats.reasoning += 1;
                            out.push_str("<hick:reasoning>\n");
                            raw(&mut out, "reasoning", &text, &mut stats);
                            out.push_str("</hick:reasoning>\n");
                        }
                        Some("tool_use") => {
                            stats.tool_calls += 1;
                            let name = str_of(&b, "name").unwrap_or_else(|| "?".into());
                            out.push_str("<hick:tool");
                            attr(&mut out, "name", &name, &mut stats);
                            if let Some(call) = str_of(&b, "id") {
                                attr(&mut out, "call", &call, &mut stats);
                            }
                            out.push_str(">\n");
                            tool_input(
                                &mut out,
                                b.get("input").unwrap_or(&Value::Null),
                                &mut stats,
                            );
                            out.push_str("</hick:tool>\n");
                        }
                        other => {
                            stats.drop(format!("assistant block of type {}", other.unwrap_or("?")))
                        }
                    }
                }
            }
            "system" => {
                let subtype = str_of(&rec.value, "subtype").unwrap_or_else(|| "system".into());
                match subtype.as_str() {
                    "compact_boundary" => {
                        let meta = rec.value.get("compactMetadata");
                        let n = |k: &str| meta.and_then(|m| m.get(k)).and_then(Value::as_u64);
                        let text = format!(
                            "Conversation compacted ({}): {} tokens before, {} after.",
                            meta.and_then(|m| str_of(m, "trigger"))
                                .unwrap_or_else(|| "?".into()),
                            n("preTokens").unwrap_or(0),
                            n("postTokens").unwrap_or(0),
                        );
                        context(&mut out, "compact", at, &text, &mut stats);
                    }
                    // How long a turn took is a property of the harness's
                    // clock, not of the conversation.
                    "turn_duration" => stats.drop("turn_duration record"),
                    _ => {
                        let text = str_of(&rec.value, "content").unwrap_or_default();
                        if text.trim().is_empty() {
                            stats.drop(format!("system record `{subtype}` without content"));
                        } else {
                            context(&mut out, &subtype, at, &text, &mut stats);
                        }
                    }
                }
            }
            "attachment" => {
                let Some(att) = rec.value.get("attachment") else {
                    stats.drop("attachment record without an attachment");
                    continue;
                };
                let kind = str_of(att, "type").unwrap_or_else(|| "attachment".into());
                if SKIPPED_ATTACHMENTS.contains(&kind.as_str()) {
                    stats.drop(format!("{kind} attachment"));
                    continue;
                }
                let text = attachment_text(att);
                context(
                    &mut out,
                    &format!("attachment:{kind}"),
                    at,
                    &text,
                    &mut stats,
                );
            }
            "queue-operation" => {
                if str_of(&rec.value, "operation").as_deref() == Some("enqueue")
                    && let Some(text) = str_of(&rec.value, "content")
                {
                    context(&mut out, "queued-prompt", at, &text, &mut stats);
                } else {
                    stats.drop("queue-operation record");
                }
            }
            "ai-title" => {}
            kind if SKIPPED_RECORDS.contains(&kind) => stats.drop(format!("{kind} record")),
            kind => stats.drop(format!("unknown `{kind}` record")),
        }
    }
    close_assistant(
        &mut out,
        &mut open_message,
        &mut open_usage,
        &mut llm_turn,
        &mut stats,
    );
    out.push_str("</hick:session>\n");

    // The one check that matters: the product can read what was written.
    let mut problems = Vec::new();
    match hick_lang::parse(&out) {
        Ok(_) => {
            let view = hickory_agent::session_view::session_view(&out);
            if view.turns.len() != stats.prompts {
                problems.push(format!(
                    "the converted session reads back as {} turn(s) but {} prompt(s) were \
                     written — something quoted hick structure this converter did not contain",
                    view.turns.len(),
                    stats.prompts
                ));
            }
        }
        Err(e) => problems.push(format!("the converted session does not parse as hick: {e}")),
    }
    stats.problems = problems;

    Ok(Converted {
        hick: out,
        title,
        session_id,
        started,
        first_prompt,
        stats,
    })
}

/// A tool call's input: one `<hick:arg>` per field, or — when a field quotes
/// a hick tag or the input is not an object — the whole input as JSON inside
/// `<hick:input>`, which is captured verbatim.
fn tool_input(out: &mut String, input: &Value, stats: &mut Stats) {
    let Some(obj) = input.as_object() else {
        if !input.is_null() {
            stats.adapt("tool input that is not an object, written as JSON in hick:input");
            out.push_str("<hick:input>\n");
            raw(
                out,
                "input",
                &serde_json::to_string_pretty(input).unwrap_or_default(),
                stats,
            );
            out.push_str("</hick:input>\n");
        }
        return;
    };
    let fields: Vec<(String, String)> = obj
        .iter()
        .map(|(k, v)| {
            let text = match v {
                Value::String(s) => s.clone(),
                other => serde_json::to_string_pretty(other).unwrap_or_default(),
            };
            (k.clone(), text)
        })
        .collect();
    let unsafe_arg = fields
        .iter()
        .any(|(k, v)| quotes_hick(v) || quotes_hick(k) || k.contains('"') && k.contains('\''));
    if unsafe_arg {
        stats.adapt("tool input quoting a hick tag, written as JSON in hick:input");
        out.push_str("<hick:input>\n");
        raw(
            out,
            "input",
            &serde_json::to_string_pretty(input).unwrap_or_default(),
            stats,
        );
        out.push_str("</hick:input>\n");
        return;
    }
    for (k, v) in fields {
        out.push_str("<hick:arg");
        attr(out, "name", &k, stats);
        out.push('>');
        if v.contains('\n') {
            out.push('\n');
            out.push_str(v.trim_end_matches('\n'));
            out.push('\n');
        } else {
            out.push_str(&v);
        }
        out.push_str("</hick:arg>\n");
    }
}

// ---------------------------------------------------------------------------
// Files
// ---------------------------------------------------------------------------

/// Where an import landed, or why it did not.
#[derive(Debug)]
pub enum Outcome {
    Written {
        path: PathBuf,
        converted: Box<Converted>,
    },
    /// The file this transcript would write already exists — the same
    /// transcript imported twice names one file, so this is "already done".
    Exists { path: PathBuf },
}

/// Convert `source` and write it under `out_dir`.
pub fn import_file(source: &Path, out_dir: &Path, force: bool) -> Result<Outcome> {
    let jsonl =
        std::fs::read_to_string(source).with_context(|| format!("reading {}", source.display()))?;
    let converted = convert(&jsonl).with_context(|| format!("converting {}", source.display()))?;
    if let Some(problem) = converted.stats.problems.first() {
        bail!(
            "{problem}. Nothing was written; `hick ingest --from claude-code --stdout {}` prints the \
             converted document so the line can be found.",
            source.display()
        );
    }
    let path = out_dir.join(converted.file_name());
    if path.exists() && !force {
        return Ok(Outcome::Exists { path });
    }
    std::fs::create_dir_all(out_dir).with_context(|| format!("creating {}", out_dir.display()))?;
    std::fs::write(&path, &converted.hick)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(Outcome::Written {
        path,
        converted: Box::new(converted),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small transcript with the shapes that matter: a prompt, a reply
    /// that thinks, speaks and calls a tool (split across two records as
    /// Claude Code writes them), the tool's answer, a second prompt on the
    /// same branch, a meta prompt, a compaction boundary, a reminder
    /// attachment, a title, and bookkeeping to skip.
    const SMALL: &str = include_str!("../tests/fixtures/claude-code-small.jsonl");

    #[test]
    fn a_transcript_becomes_a_session_that_reads_back() {
        // docs/guarantees/agent/a-claude-code-transcript-imports-as-a-session.md
        let c = convert(SMALL).unwrap();
        let view = hickory_agent::session_view::session_view(&c.hick);
        assert_eq!(view.turns.len(), 2);
        assert_eq!(view.turns[0].prompt, "Count the lines in fruit.csv");
        assert_eq!(view.turns[0].model.as_deref(), Some("claude-opus-5"));
        assert_eq!(view.turns[0].provider.as_deref(), Some("anthropic"));
        // The second prompt names the first as its parent: the tree is kept.
        assert_eq!(
            view.turns[1].parent.as_deref(),
            Some(view.turns[0].id.as_str())
        );
        let kinds: Vec<&str> = view.turns[0]
            .steps
            .iter()
            .map(|s| match s {
                hickory_agent::session_view::Step::Reasoning { .. } => "reasoning",
                hickory_agent::session_view::Step::Prose { .. } => "prose",
                hickory_agent::session_view::Step::Tool { .. } => "tool",
                hickory_agent::session_view::Step::ToolResult { .. } => "tool-result",
                hickory_agent::session_view::Step::Context { .. } => "context",
                _ => "other",
            })
            .collect();
        // The meta caveat and the compaction both sit on the first turn, as
        // context the model was shown before the second prompt.
        assert_eq!(
            kinds,
            [
                "reasoning",
                "prose",
                "tool",
                "tool-result",
                "prose",
                "context",
                "context"
            ]
        );
        assert_eq!(
            view.turns[0].answer.as_deref(),
            Some("fruit.csv has 3 lines.")
        );
        // Usage rode along.
        assert_eq!(view.turns[0].usage.as_ref().map(|u| u.output), Some(50));
        assert_eq!(c.title.as_deref(), Some("Count fruit lines"));
        assert_eq!(c.file_name(), "20260820-090000-count-fruit-lines.md");
    }

    #[test]
    fn tool_calls_and_results_keep_their_ids_and_names() {
        let c = convert(SMALL).unwrap();
        assert!(
            c.hick
                .contains(r#"<hick:tool name="Bash" call="toolu_01">"#)
        );
        assert!(
            c.hick
                .contains(r#"<hick:arg name="command">wc -l < fruit.csv</hick:arg>"#)
        );
        assert!(
            c.hick
                .contains(r#"<hick:tool-result id="2" name="Bash" call="toolu_01" ok="true""#)
        );
        assert!(c.hick.contains("<hick:usage turn=\"0\" input=\"100\" cache-write=\"5\" cache-read=\"50\" output=\"40\"/>"));
    }

    #[test]
    fn what_is_dropped_is_counted() {
        let c = convert(SMALL).unwrap();
        assert_eq!(
            c.stats.dropped.get("total_tokens_reminder attachment"),
            Some(&1)
        );
        assert_eq!(c.stats.dropped.get("mode record"), Some(&1));
        assert_eq!(c.stats.dropped.get("thinking signature"), Some(&1));
        assert_eq!(c.stats.context, 2); // the caveat echo and the compaction
        assert!(c.hick.contains(r#"<hick:context kind="compact""#));
        assert!(c.hick.contains(r#"<hick:context kind="local-command""#));
    }

    #[test]
    fn a_prompt_or_result_quoting_a_hick_tag_still_parses() {
        // The no-escaping invariant: a tool result that prints an unclosed
        // tag must not swallow the rest of the session, and a prompt that
        // quotes one is wrapped where the parser captures verbatim.
        let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"What does <hick:exec> do?"}}
{"type":"assistant","uuid":"a1","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"id":"m1","model":"claude-opus-5","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"a.hick"}}],"usage":{"input_tokens":1,"output_tokens":1}}}
{"type":"user","uuid":"u2","parentUuid":"a1","timestamp":"2026-01-01T00:00:02.000Z","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","content":"<hick:doc>\n<hick:exec container=\"x\">\nls\n"}]}}
{"type":"assistant","uuid":"a2","parentUuid":"u2","timestamp":"2026-01-01T00:00:03.000Z","message":{"id":"m2","model":"claude-opus-5","content":[{"type":"text","text":"It runs a cell, written as <hick:exec>."}],"usage":{"input_tokens":1,"output_tokens":1}}}
{"type":"user","uuid":"u3","parentUuid":"a2","timestamp":"2026-01-01T00:00:04.000Z","message":{"role":"user","content":"Thanks"}}
"#;
        let c = convert(jsonl).unwrap();
        let view = hickory_agent::session_view::session_view(&c.hick);
        assert_eq!(view.turns.len(), 2);
        assert_eq!(view.turns[0].prompt, "What does <hick:exec> do?");
        assert_eq!(
            view.turns[0].answer.as_deref(),
            Some("It runs a cell, written as <hick:exec>.")
        );
        assert_eq!(view.turns[1].prompt, "Thanks");
        assert_eq!(c.stats.adapted.len(), 2);
    }

    #[test]
    fn a_slash_command_echo_is_context_not_a_prompt() {
        let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"<command-name>/model</command-name>\n<command-message>model</command-message>"}}
{"type":"user","uuid":"u2","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"role":"user","content":"<local-command-stdout>Set model to Sonnet 5</local-command-stdout>"}}
{"type":"user","uuid":"u3","parentUuid":"u2","timestamp":"2026-01-01T00:00:02.000Z","message":{"role":"user","content":"Now help me."}}
"#;
        let c = convert(jsonl).unwrap();
        assert_eq!(c.stats.prompts, 1);
        assert_eq!(c.stats.context, 2);
        assert!(c.hick.contains(r#"<hick:context kind="local-command""#));
        let view = hickory_agent::session_view::session_view(&c.hick);
        assert_eq!(view.turns[0].prompt, "Now help me.");
        assert_eq!(view.turns[0].parent, None);
    }

    #[test]
    fn a_comment_marker_in_prose_is_carried_verbatim_too() {
        // `<!--` is structure to the parser (a comment, skipped) — a prompt
        // that mentions one must not lose the words after it.
        let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"Why does the file start with <!-- woven by hickory?"}}
{"type":"assistant","uuid":"a1","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Write","input":{"content":"<!-- banner\nbody"}}],"usage":{}}}
"#;
        let c = convert(jsonl).unwrap();
        assert!(c.stats.problems.is_empty(), "{:?}", c.stats.problems);
        let view = hickory_agent::session_view::session_view(&c.hick);
        assert_eq!(
            view.turns[0].prompt,
            "Why does the file start with <!-- woven by hickory?"
        );
    }

    #[test]
    fn a_body_quoting_its_own_close_tag_is_broken_not_truncated() {
        // A tool result that printed a session file: the literal
        // `</hick:tool-result>` inside it would end the element early.
        let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"show the session"}}
{"type":"assistant","uuid":"a1","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"s.hick"}}],"usage":{}}}
{"type":"user","uuid":"u2","parentUuid":"a1","timestamp":"2026-01-01T00:00:02.000Z","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","content":"<hick:tool-result name=\"x\">\nhi\n</hick:tool-result>\nafter"}]}}
{"type":"assistant","uuid":"a2","parentUuid":"u2","timestamp":"2026-01-01T00:00:03.000Z","message":{"id":"m2","model":"m","content":[{"type":"text","text":"Done."}],"usage":{}}}
"#;
        let c = convert(jsonl).unwrap();
        assert!(c.stats.problems.is_empty(), "{:?}", c.stats.problems);
        assert!(c.hick.contains("</hick:tool-result >\nafter"));
        let view = hickory_agent::session_view::session_view(&c.hick);
        assert_eq!(view.turns[0].answer.as_deref(), Some("Done."));
        assert_eq!(c.stats.adapted.len(), 1);
    }

    #[test]
    fn a_tool_input_quoting_a_hick_tag_is_written_as_json() {
        let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"edit"}}
{"type":"assistant","uuid":"a1","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Edit","input":{"file_path":"a.hick","new_string":"<hick:file path=\"x\">"}}],"usage":{}}}
"#;
        let c = convert(jsonl).unwrap();
        assert!(c.hick.contains("<hick:input>\n{\n"));
        assert!(!c.hick.contains("<hick:arg"));
        let view = hickory_agent::session_view::session_view(&c.hick);
        assert!(matches!(
            view.turns[0].steps.first(),
            Some(hickory_agent::session_view::Step::Tool { input: Some(_), .. })
        ));
    }

    #[test]
    fn attributes_pick_the_quote_the_value_lacks() {
        let mut s = Stats::default();
        let mut out = String::new();
        attr(&mut out, "title", r#"Fix the "thing""#, &mut s);
        assert_eq!(out, r#" title='Fix the "thing"'"#);
        let mut out = String::new();
        attr(&mut out, "title", r#"It's "both""#, &mut s);
        assert_eq!(out, "");
        assert_eq!(s.dropped.len(), 1);
    }

    #[test]
    fn file_names_are_deterministic_and_dated() {
        assert_eq!(
            stamp_of("2026-08-13T16:29:17.371Z").as_deref(),
            Some("20260813-162917")
        );
        assert_eq!(
            slug("Review uncommitted updates and verify documentation"),
            "review-uncommitted-updates-and-verify-do"
        );
        assert_eq!(slug("///"), "session");
    }

    #[test]
    fn an_empty_or_foreign_file_is_refused_with_a_reason() {
        assert!(convert("").is_err());
        assert!(
            convert("not json\n")
                .unwrap_err()
                .to_string()
                .contains("no records")
        );
    }
}
