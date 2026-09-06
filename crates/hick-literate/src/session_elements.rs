//! The session vocabulary as elements: what a conversation's record looks
//! like as blocks, and what each block knows about where it came from.
//!
//! A session file is a document (`a-session-is-the-conversation.md`), so
//! the agent pane is a lens over it rather than a renderer of its own. Each
//! element renders to a block the frontend draws as a card, and the ones
//! that touch files declare their provenance beside that:
//!
//! - `read` (harness-written after a read tool) and a read tool call with a
//!   file argument declare a **context** link — this file, these lines,
//!   were in front of the model.
//! - `wrote` (harness-written after an edit tool) and an edit tool call
//!   declare a **lineage** link — this turn's output landed there.
//! - `assistant` prose declares a **declared** link for every markdown link
//!   or `path:line` mention in it — what the model pointed at, an assertion.
//!
//! Derived from the record, never from the model's opinion of itself
//! (`docs/specs/freeform/three-provenances.md`). Claude Code transcripts
//! ingested as sessions (`hick ingest --from claude-code`) carry the same
//! elements with the tool names Claude Code uses, so they get the same
//! links. See docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.

use std::path::Path;

use hick_blocks::{
    AttrSpec, Block, Descend, Element, Family, Link, LinkTarget, Registry, attr, span_of,
};
use hick_lang::HickTag;

/// What the session elements render from: where the open folder is, so a
/// transcript's absolute paths fold to the folder's spelling.
#[derive(Debug, Default, Clone)]
pub struct SessionFacts {
    pub root: Option<std::path::PathBuf>,
}

impl SessionFacts {
    /// A path as the folder knows it.
    pub fn relative(&self, path: &str) -> String {
        let clean = path.replace('\\', "/");
        if let Some(root) = &self.root {
            let root = root.to_string_lossy().replace('\\', "/");
            let root = root.trim_end_matches('/');
            if let Some(rest) = clean.strip_prefix(&format!("{root}/")) {
                return rest.to_string();
            }
        }
        clean.trim_start_matches("./").to_string()
    }
}

/// The session elements over the folder's facts.
pub fn session_registry() -> Registry<SessionFacts> {
    let mut registry = Registry::new();
    registry
        .register(SessionRoot)
        .register(UserElement)
        .register(AssistantElement)
        .register(ToolElement)
        .register(ToolResultElement)
        .register(ReadElement)
        .register(WroteElement)
        .register(ContextElement)
        .register(ObservationElement)
        .register(ActionElement)
        .register(ReasoningElement)
        .register(InputElement)
        .register(MetaElement("usage"))
        .register(MetaElement("next"));
    registry
}

/// `a-b` or `a` as lines.
pub fn parse_lines(text: &str) -> Option<(usize, usize)> {
    let mut parts = text.trim().splitn(2, '-');
    let a: usize = parts.next()?.trim().parse().ok()?;
    let b: usize = match parts.next() {
        Some(b) => b.trim().parse().ok()?,
        None => a,
    };
    (a > 0).then_some((a, b.max(a)))
}

fn target(facts: &SessionFacts, path: &str, lines: Option<(usize, usize)>) -> LinkTarget {
    LinkTarget {
        path: facts.relative(path),
        lines,
    }
}

fn lines_text(lines: Option<(usize, usize)>) -> String {
    match lines {
        Some((a, b)) if a == b => format!(" line {a}"),
        Some((a, b)) => format!(" lines {a}–{b}"),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------

/// `<hick:session>`: the conversation's root when the file is wrapped. Draws
/// nothing; what it holds is drawn.
struct SessionRoot;
impl Element<SessionFacts> for SessionRoot {
    fn name(&self) -> &'static str {
        "session"
    }
    fn descend(&self) -> Descend {
        Descend::All
    }
    fn render(&self, _: &HickTag, _: &SessionFacts) -> Option<Block> {
        None
    }
}

struct UserElement;
impl Element<SessionFacts> for UserElement {
    fn name(&self) -> &'static str {
        "user"
    }
    fn kind(&self) -> &'static str {
        "session-user"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const A: &[AttrSpec] = &[AttrSpec::optional("turn", "which turn this prompt opens")];
        A
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-user", span_of(tag))
                .with("turn", attr(tag, "turn").or_else(|| attr(tag, "id")))
                .with("body", tag.text_content().trim()),
        )
    }
}

struct AssistantElement;
impl Element<SessionFacts> for AssistantElement {
    fn name(&self) -> &'static str {
        "assistant"
    }
    fn kind(&self) -> &'static str {
        "session-assistant"
    }
    fn descend(&self) -> Descend {
        Descend::All
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        // The prose alone: tool calls and actions nested in the answer are
        // blocks of their own, drawn after it.
        let mut prose = String::new();
        for node in &tag.children {
            if let hick_lang::HickNode::Text(t, _) = node {
                prose.push_str(t);
            }
        }
        Some(Block::new("session-assistant", span_of(tag)).with("body", prose.trim()))
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        let mut prose = String::new();
        for node in &tag.children {
            if let hick_lang::HickNode::Text(t, _) = node {
                prose.push_str(t);
            }
        }
        mentions(&prose)
            .into_iter()
            .map(|(path, lines)| Link {
                family: Family::Declared,
                span: span_of(tag),
                title: format!(
                    "Declared — the answer points at {}{}. The model's own reference, not a derivation.",
                    facts.relative(&path),
                    lines_text(lines)
                ),
                to: target(facts, &path, lines),
            })
            .collect()
    }
}

/// Tools that show the model a file, by the names hick's agent and Claude
/// Code use, and the ones that change one.
const READ_TOOLS: &[&str] = &["read_doc", "read_output", "read_file", "read", "Read"];
const WRITE_TOOLS: &[&str] = &[
    "edit_doc",
    "write_doc",
    "edit_file",
    "write_file",
    "replace_lines",
    "insert_lines",
    "Edit",
    "Write",
    "MultiEdit",
    "NotebookEdit",
];

fn tool_arg(tag: &HickTag, names: &[&str]) -> Option<String> {
    // hick's agent writes `<hick:arg name="doc">…</hick:arg>` children;
    // an ingested transcript may carry attributes instead.
    for child in tag.child_tags() {
        if child.name == "arg"
            && let Some(name) = child.get_attribute("name")
            && names.contains(&name)
        {
            return Some(child.text_content().trim().to_string());
        }
    }
    names.iter().find_map(|n| attr(tag, n))
}

struct ToolElement;
impl Element<SessionFacts> for ToolElement {
    fn name(&self) -> &'static str {
        "tool"
    }
    fn kind(&self) -> &'static str {
        "session-tool"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const A: &[AttrSpec] = &[
            AttrSpec::required("name", "the tool called"),
            AttrSpec::optional("call", "the provider's id for this call"),
        ];
        A
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        let args: Vec<(String, String)> = tag
            .child_tags()
            .filter(|c| c.name == "arg")
            .map(|c| {
                (
                    c.get_attribute("name").unwrap_or_default().to_string(),
                    c.text_content().trim().to_string(),
                )
            })
            .collect();
        Some(
            Block::new("session-tool", span_of(tag))
                .with("name", attr(tag, "name"))
                .with("call", attr(tag, "call"))
                .with("args", args),
        )
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        let Some(name) = attr(tag, "name") else {
            return vec![];
        };
        let Some(file) = tool_arg(
            tag,
            &[
                "file_path",
                "path",
                "file",
                "doc",
                "output",
                "notebook_path",
            ],
        ) else {
            return vec![];
        };
        if READ_TOOLS.contains(&name.as_str()) {
            // Claude Code's Read takes an offset and a limit; hick's names lines.
            let lines = tool_arg(tag, &["lines"])
                .and_then(|l| parse_lines(&l))
                .or_else(|| {
                    let offset: usize = tool_arg(tag, &["offset"])?.parse().ok()?;
                    let limit: usize = tool_arg(tag, &["limit"])?.parse().ok()?;
                    (offset > 0 && limit > 0).then(|| (offset, offset + limit - 1))
                });
            vec![Link {
                family: Family::Context,
                span: span_of(tag),
                title: format!(
                    "Context — the {name} tool showed the model {}{} during this turn. Derived from the session record.",
                    facts.relative(&file),
                    lines_text(lines)
                ),
                to: target(facts, &file, lines),
            }]
        } else if WRITE_TOOLS.contains(&name.as_str()) {
            vec![Link {
                family: Family::Lineage,
                span: span_of(tag),
                title: format!(
                    "Lineage — the {name} tool wrote {} during this turn: the answer is the source, the file is its output.",
                    facts.relative(&file)
                ),
                to: target(facts, &file, None),
            }]
        } else {
            vec![]
        }
    }
}

struct ToolResultElement;
impl Element<SessionFacts> for ToolResultElement {
    fn name(&self) -> &'static str {
        "tool-result"
    }
    fn kind(&self) -> &'static str {
        "session-tool-result"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-tool-result", span_of(tag))
                .with("name", attr(tag, "name"))
                .with("ok", attr(tag, "ok").map(|v| v == "true"))
                .with("body", tag.text_content().trim()),
        )
    }
}

struct ReadElement;
impl Element<SessionFacts> for ReadElement {
    fn name(&self) -> &'static str {
        "read"
    }
    fn kind(&self) -> &'static str {
        "session-read"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const A: &[AttrSpec] = &[
            AttrSpec::required("file", "the file shown to the model"),
            AttrSpec::optional("lines", "which lines, `a-b`"),
            AttrSpec::optional("sha256", "the content hash it was read at"),
            AttrSpec::optional("commit", "the commit it was read at"),
        ];
        A
    }
    fn render(&self, tag: &HickTag, facts: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-read", span_of(tag))
                .with("file", attr(tag, "file").map(|f| facts.relative(&f)))
                .with("lines", attr(tag, "lines"))
                .with("sha256", attr(tag, "sha256"))
                .with("commit", attr(tag, "commit")),
        )
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        let Some(file) = attr(tag, "file") else {
            return vec![];
        };
        let lines = attr(tag, "lines").and_then(|l| parse_lines(&l));
        vec![Link {
            family: Family::Context,
            span: span_of(tag),
            title: format!(
                "Context — {}{} was in front of the model when this answer was produced. Derived from the session record.",
                facts.relative(&file),
                lines_text(lines)
            ),
            to: target(facts, &file, lines),
        }]
    }
}

struct WroteElement;
impl Element<SessionFacts> for WroteElement {
    fn name(&self) -> &'static str {
        "wrote"
    }
    fn kind(&self) -> &'static str {
        "session-wrote"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const A: &[AttrSpec] = &[
            AttrSpec::required("file", "the file the edit landed in"),
            AttrSpec::optional("lines", "the lines it left, `a-b`"),
            AttrSpec::optional("hashes", "the hashline hashes of those lines"),
        ];
        A
    }
    fn render(&self, tag: &HickTag, facts: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-wrote", span_of(tag))
                .with("file", attr(tag, "file").map(|f| facts.relative(&f)))
                .with("lines", attr(tag, "lines")),
        )
    }
    fn links(&self, tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
        let Some(file) = attr(tag, "file") else {
            return vec![];
        };
        let lines = attr(tag, "lines").and_then(|l| parse_lines(&l));
        vec![Link {
            family: Family::Lineage,
            span: span_of(tag),
            title: format!(
                "Lineage — this turn wrote {}{}: the answer is the source, the file is its output.",
                facts.relative(&file),
                lines_text(lines)
            ),
            to: target(facts, &file, lines),
        }]
    }
}

struct ContextElement;
impl Element<SessionFacts> for ContextElement {
    fn name(&self) -> &'static str {
        "context"
    }
    fn kind(&self) -> &'static str {
        "session-context"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-context", span_of(tag))
                .with("context_kind", attr(tag, "kind"))
                .with("body", tag.text_content().trim()),
        )
    }
}

struct ObservationElement;
impl Element<SessionFacts> for ObservationElement {
    fn name(&self) -> &'static str {
        "observation"
    }
    fn kind(&self) -> &'static str {
        "session-observation"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-observation", span_of(tag))
                .with("source", attr(tag, "source"))
                .with("exit", attr(tag, "exit"))
                .with("body", tag.text_content().trim()),
        )
    }
}

struct ActionElement;
impl Element<SessionFacts> for ActionElement {
    fn name(&self) -> &'static str {
        "action"
    }
    fn kind(&self) -> &'static str {
        "session-action"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-action", span_of(tag))
                .with("lang", attr(tag, "lang").or_else(|| attr(tag, "language")))
                .with("body", tag.text_content().trim()),
        )
    }
}

/// `<hick:reasoning>`: the model's reasoning, kept apart from its answer and
/// drawn folded (`reasoning-is-shown-apart-from-the-answer.md`).
struct ReasoningElement;
impl Element<SessionFacts> for ReasoningElement {
    fn name(&self) -> &'static str {
        "reasoning"
    }
    fn kind(&self) -> &'static str {
        "session-reasoning"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(Block::new("session-reasoning", span_of(tag)).with("body", tag.text_content().trim()))
    }
}

/// `<hick:input>`: a tool's multi-line payload, raw.
struct InputElement;
impl Element<SessionFacts> for InputElement {
    fn name(&self) -> &'static str {
        "input"
    }
    fn kind(&self) -> &'static str {
        "session-input"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(
            Block::new("session-input", span_of(tag))
                .with("name", attr(tag, "name"))
                .with("body", tag.text_content().trim()),
        )
    }
}

/// The harness's bookkeeping — `<hick:usage>` totals and the `<hick:next>`
/// protocol marker. Facts of the record, not of the conversation: drawn as
/// nothing, so the lens folds them away rather than showing raw markup
/// between the cards.
struct MetaElement(&'static str);
impl Element<SessionFacts> for MetaElement {
    fn name(&self) -> &'static str {
        self.0
    }
    fn kind(&self) -> &'static str {
        "session-meta"
    }
    fn render(&self, tag: &HickTag, _: &SessionFacts) -> Option<Block> {
        Some(Block::new("session-meta", span_of(tag)).with("element", self.0))
    }
}

// ---------------------------------------------------------------------------
// What prose points at
// ---------------------------------------------------------------------------

/// Files an answer names: markdown links to local targets (with a
/// `#L10-L20` fragment as lines), and bare `path/to/file.ext:12` mentions.
/// Needs a dot in the last segment, so `at 3:15` in prose is not a file,
/// and no scheme, so a URL is not one either.
pub fn mentions(text: &str) -> Vec<(String, Option<(usize, usize)>)> {
    let mut out: Vec<(String, Option<(usize, usize)>)> = Vec::new();
    let mut add = |path: String, lines: Option<(usize, usize)>| {
        if path.is_empty()
            || path.ends_with('/')
            || out.iter().any(|(p, l)| *p == path && *l == lines)
        {
            return;
        }
        out.push((path, lines));
    };
    // [label](target "title")
    let mut rest = text;
    while let Some(open) = rest.find("](") {
        let after = &rest[open + 2..];
        let Some(close) = after.find(')') else { break };
        let target = after[..close].split_whitespace().next().unwrap_or("");
        if !target.is_empty()
            && !target.contains("://")
            && !target.starts_with("mailto:")
            && !target.starts_with('#')
        {
            let (path, fragment) = target.split_once('#').unwrap_or((target, ""));
            let lines = fragment.strip_prefix('L').and_then(|f| {
                let f = f.replace('L', "");
                parse_lines(&f)
            });
            if path
                .rsplit('/')
                .next()
                .is_some_and(|last| last.contains('.'))
            {
                add(path.to_string(), lines);
            }
        }
        rest = &after[close + 1..];
    }
    // path:12 or path:12-20
    for token in text.split(|c: char| c.is_whitespace() || "()`'\"<>,;".contains(c)) {
        let Some((path, lines)) = token.rsplit_once(':') else {
            continue;
        };
        let Some(lines) = parse_lines(lines.trim_end_matches(|c: char| ".?!".contains(c))) else {
            continue;
        };
        let last = path.rsplit('/').next().unwrap_or(path);
        if !last.contains('.') || path.contains("://") || path.starts_with('#') {
            continue;
        }
        let path = path.trim_end_matches(|c: char| ".?!".contains(c));
        add(path.to_string(), Some(lines));
    }
    out
}

/// The blocks of a session document.
pub fn session_blocks(source: &str, root: Option<&Path>) -> Vec<Block> {
    let facts = SessionFacts {
        root: root.map(Path::to_path_buf),
    };
    let (doc, _) = hick_lang::parse_lenient(source);
    session_registry().blocks(&doc, &facts)
}

/// The links of a session document, with the lines each span covers.
pub fn session_links(source: &str, root: Option<&Path>) -> Vec<LinkWithLines> {
    let facts = SessionFacts {
        root: root.map(Path::to_path_buf),
    };
    let (doc, _) = hick_lang::parse_lenient(source);
    session_registry()
        .links(&doc, &facts)
        .into_iter()
        .map(|link| {
            let lines = (
                line_of(source, link.span.0),
                line_of(source, link.span.1.saturating_sub(1).max(link.span.0)),
            );
            LinkWithLines { link, lines }
        })
        .collect()
}

/// A link with the 1-based lines its span starts and ends on — what the
/// overlay anchors on.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkWithLines {
    #[serde(flatten)]
    pub link: Link,
    pub lines: (usize, usize),
}

fn line_of(source: &str, byte: usize) -> usize {
    source[..byte.min(source.len())].matches('\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    type Summary = (String, String, Option<(usize, usize)>, usize);
    type Where = (String, String, Option<(usize, usize)>);

    const SESSION: &str = r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-08-22T12:56:34Z">
<hick:user turn="t1">Why is checkout slow?</hick:user>
<hick:assistant>
<hick:tool name="read_doc">
<hick:arg name="doc">meetings/sync.hick</hick:arg>
</hick:tool>
</hick:assistant>
<hick:tool-result name="read_doc" ok="true">
lines…
</hick:tool-result>
<hick:read file="/home/me/notes/data/latency.csv" sha256="ff" lines="1-8"/>
<hick:assistant>
Checkout slowed after the pool change; see [the sync](meetings/sync.hick#L12-L20) and data/latency.csv:3.
</hick:assistant>
<hick:wrote file="notes/today.hick" lines="72-75" hashes="a b c d"/>
</hick:session>
"#;

    #[test]
    fn a_session_is_blocks_of_its_own_kinds() {
        let blocks = session_blocks(SESSION, Some(Path::new("/home/me/notes")));
        let kinds: Vec<&str> = blocks.iter().map(|b| b.kind.as_str()).collect();
        assert_eq!(
            kinds,
            vec![
                "session-user",
                "session-assistant",
                "session-tool",
                "session-tool-result",
                "session-read",
                "session-assistant",
                "session-wrote"
            ]
        );
        assert_eq!(blocks[0].str_prop("turn"), Some("t1"));
        assert_eq!(blocks[4].str_prop("file"), Some("data/latency.csv"));
        assert!(
            blocks[5]
                .str_prop("body")
                .unwrap()
                .starts_with("Checkout slowed")
        );
    }

    #[test]
    fn the_three_families_come_from_the_record_and_the_prose() {
        let links = session_links(SESSION, Some(Path::new("/home/me/notes")));
        let summary: Vec<Summary> = links
            .iter()
            .map(|l| {
                (
                    format!("{:?}", l.link.family).to_lowercase(),
                    l.link.to.path.clone(),
                    l.link.to.lines,
                    l.lines.0,
                )
            })
            .collect();
        assert_eq!(
            summary,
            vec![
                ("context".into(), "meetings/sync.hick".into(), None, 4),
                (
                    "context".into(),
                    "data/latency.csv".into(),
                    Some((1, 8)),
                    11
                ),
                (
                    "declared".into(),
                    "meetings/sync.hick".into(),
                    Some((12, 20)),
                    12
                ),
                (
                    "declared".into(),
                    "data/latency.csv".into(),
                    Some((3, 3)),
                    12
                ),
                (
                    "lineage".into(),
                    "notes/today.hick".into(),
                    Some((72, 75)),
                    15
                ),
            ]
        );
    }

    #[test]
    fn a_claude_code_transcript_reads_the_same_way() {
        let src = r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:user>fix it</hick:user>
<hick:assistant>
<hick:tool name="Read" call="t1"><hick:arg name="file_path">/home/me/notes/src/lib.rs</hick:arg><hick:arg name="offset">10</hick:arg><hick:arg name="limit">5</hick:arg></hick:tool>
<hick:tool name="Edit" call="t2"><hick:arg name="file_path">/home/me/notes/src/lib.rs</hick:arg></hick:tool>
<hick:tool name="Bash" call="t3"><hick:arg name="command">ls</hick:arg></hick:tool>
</hick:assistant>
</hick:session>
"#;
        let links = session_links(src, Some(Path::new("/home/me/notes")));
        let summary: Vec<Where> = links
            .iter()
            .map(|l| {
                (
                    format!("{:?}", l.link.family).to_lowercase(),
                    l.link.to.path.clone(),
                    l.link.to.lines,
                )
            })
            .collect();
        assert_eq!(
            summary,
            vec![
                ("context".into(), "src/lib.rs".into(), Some((10, 14))),
                ("lineage".into(), "src/lib.rs".into(), None),
            ]
        );
    }

    #[test]
    fn mentions_find_links_and_path_lines_and_nothing_else() {
        let found = mentions(
            "See [x](../a/b.md#L3-L4), src/lib.rs:42, `data/x.csv:1-8`; not https://e.com/x.md:3, not at 3:15, not README.",
        );
        assert_eq!(
            found,
            vec![
                ("../a/b.md".into(), Some((3, 4))),
                ("src/lib.rs".into(), Some((42, 42))),
                ("data/x.csv".into(), Some((1, 8))),
            ]
        );
    }
}
