<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0cf7120750a0"></a>
## `crates/hick-lang/src/lib.rs`

Parser and AST for `.md` files.
Tags prefixed with the namespace bound to `http://www.hickorydocs.com/1.0`
are parsed as structured elements. Content between them is raw text that
can contain `&`, `<`, `>` without escaping.
The namespace prefix is detected from the `xmlns:` declaration on the root
element. For example, `<h:doc xmlns:h="http://www.hickorydocs.com/1.0">`
uses prefix `h`, while `<hick:doc xmlns:hick="...">` uses prefix `hick`.

Related contracts: [A Claude Code Transcript Imports As A Session](../../../../guarantees/agent/a-claude-code-transcript-imports-as-a-session/contract.md), [A Claim Says Who Is Asserting It, And Never Looks Verified](../../../../guarantees/authoring/a-claim-says-who-is-asserting-it/contract.md), [A Document May Begin With Markdown](../../../../guarantees/authoring/a-document-may-begin-with-markdown/contract.md), [A Syntax Error Names The Invariant It Hit](../../../../guarantees/authoring/a-syntax-error-names-the-invariant-it-hit/contract.md), [A Transcript Derives Its Speaker Turns, And Cannot Disagree With Itself](../../../../guarantees/authoring/a-transcript-derives-its-speaker-turns/contract.md), [Every Document Weaves A Markdown File Of Its Own Name](../../../../guarantees/authoring/every-document-weaves-a-markdown-file/contract.md), [Many Documents Contribute To One File, And None Of Them Names The Others](../../../../guarantees/authoring/many-documents-contribute-to-one-file/contract.md), [Bind the document language before quoting code](../../../../guarantees/language/prefix-only-parsing-no-escaping/contract.md), [The Editor Reads A Document With The Parser The Server Uses](../../../../guarantees/language/the-editor-reads-with-the-parser-the-server-uses/contract.md), [An Edited AI Passage Stops Claiming The Model Wrote It](../../../../guarantees/lineage/an-edited-ai-passage-stops-claiming-the-model-wrote-it/contract.md), [An Included Span Names Its Own File, And An Edit Through It Lands There](../../../../guarantees/lineage/included-spans-name-their-own-file/contract.md), [Rebuild the program from its Markdown source](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md), [A Recording A Document Keeps Lives In The Document](../../../../guarantees/verification/a-recording-a-document-keeps-lives-in-the-document/contract.md), [A Report Is Not a Reproducible Artifact](../../../../guarantees/verification/a-report-is-not-a-reproducible-artifact/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hick-lang/src/lib.rs">
//! Parser and AST for `.md` files.
//!
//! Tags prefixed with the namespace bound to `http://www.hickorydocs.com/1.0`
//! are parsed as structured elements. Content between them is raw text that
//! can contain `&`, `<`, `>` without escaping.
//!
//! The namespace prefix is detected from the `xmlns:` declaration on the root
//! element. For example, `<h:doc xmlns:h="http://www.hickorydocs.com/1.0">`
//! uses prefix `h`, while `<hick:doc xmlns:hick="...">` uses prefix `hick`.
use std::fmt;
mod recording;
pub use recording::{
    ExecShow, agent_cache_key, exec_cache_key, inputs_digest, render_transcript_entry,
};
mod project;
pub use project::{
    AgentProjectCapture, ByteOrigin, ComparisonSource, DifferenceKind, EDITOR_BUFFER_PREFIX,
    EditPolicy, EntryKind, EvaluatedProject, IncorporationCandidate, IncorporationError,
    PatchResult, ProjectComparison, ProjectDifference, ProjectEntry, ProjectInventory,
    ProjectSnapshot, ProjectTree, RecordedCell, RevisionIdentity, apply_project_patch,
    evaluate_project, latest_debug_project, load_project_sources, project_byte_location,
    regenerate_patch, reverse_project_edit, transition_response, unified_transition,
    validate_project_path,
};
mod fence;
mod literal_files;
pub mod notebook;
pub use literal_files::{LiteralFile, LiteralSegment, literal_files, strip_opening_break};
mod parser;
use parser::Parser;
pub mod structure;
pub use structure::{Structure, StructureBlock, StructureTag, structure};
/// The XML namespace URI that identifies hick elements.
pub const HICK_NAMESPACE: &str = "http://www.hickorydocs.com/1.0";
/// Byte-level source location for provenance tracking.
///
/// Tracks where a piece of text or tag originated in the `.md` source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceSpan {
    /// Byte offset of the first character (inclusive).
    pub start: usize,
    /// Byte offset past the last character (exclusive).
    pub end: usize,
    /// 1-based line number where the span starts.
    pub start_line: usize,
    /// 0-based column number where the span starts.
    pub start_col: usize,
    /// Which FILE the offsets index: `None` for the document that was
    /// parsed, `Some(i)` for [`HickDocument::span_files`]`[i]` — set by
    /// include/upstream splicing, whose spliced spans are offsets into the
    /// *included* file. Without this, provenance attributes an included
    /// span to the including document, and a reverse edit lands in the
    /// wrong file at those offsets — silent corruption.
    #[cfg_attr(feature = "serde", serde(default))]
    pub file_id: Option<u16>,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize, start_line: usize, start_col: usize) -> Self {
        Self {
            start,
            end,
            start_line,
            start_col,
            file_id: None,
        }
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A node in the hick document tree.
#[derive(Debug, Clone, PartialEq)]
pub enum HickNode {
    /// Raw text content with optional source span.
    Text(String, Option<SourceSpan>),
    Tag(HickTag),
}

/// A structured hick element with attributes and children.
#[derive(Debug, Clone, PartialEq)]
pub struct HickTag {
    /// Local name after the namespace prefix, e.g. `"file"`, `"exec"`, `"container"`.
    pub name: String,
    /// Attribute pairs in document order.
    pub attributes: Vec<(String, String)>,
    /// Child nodes (text and nested tags).
    pub children: Vec<HickNode>,
    /// Whether this was a self-closing tag (`<prefix:allow ... />`).
    pub self_closing: bool,
    /// 1-based line number where the opening tag starts.
    pub source_line: usize,
    /// 0-based column number where the opening tag starts (for dedenting content).
    pub source_column: usize,
    /// Byte-level span covering the opening tag (from `<` to closing `>`).
    pub source_span: Option<SourceSpan>,
    /// Byte-level span of the closing tag (`</prefix:name>`), when the element
    /// has one: `None` for a self-closing tag, for a tag built by hand, and —
    /// under [`parse_lenient`] — for a tag nobody closed.
    pub close_span: Option<SourceSpan>,
}

impl HickTag {
    /// Get the value of an attribute by name.
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Iterate over child tags (skipping text nodes).
    pub fn child_tags(&self) -> impl Iterator<Item = &HickTag> {
        self.children.iter().filter_map(|n| match n {
            HickNode::Tag(t) => Some(t),
            HickNode::Text(..) => None,
        })
    }

    /// Collect all text content (direct and nested) into a single string.
    pub fn text_content(&self) -> String {
        let mut out = String::new();
        collect_text(&self.children, &mut out);
        out
    }
}

fn collect_text(nodes: &[HickNode], out: &mut String) {
    for node in nodes {
        match node {
            HickNode::Text(t, _) => out.push_str(t),
            HickNode::Tag(tag) => collect_text(&tag.children, out),
        }
    }
}

/// A parsed `.md` document.
#[derive(Debug, Clone)]
pub struct HickDocument {
    /// Children of the root doc element.
    pub nodes: Vec<HickNode>,
    /// Original source text.
    pub source: String,
    /// The XML namespace prefix used in this document (e.g. `"hick"`, `"h"`).
    pub prefix: String,
    /// Path for weave output (literate programming documentation).
    /// Set from the `weave` attribute on the root `<hick:doc>` element.
    pub weave_path: Option<String>,
    /// Whether the woven output is a REPORT rather than a reproducible
    /// artifact, from `volatile="true"` on the root `<hick:doc>` element.
    ///
    /// Drift checking asks "do these bytes reproduce", which is only
    /// meaningful when the inputs are fixed. A document over live data — a
    /// growing corpus, a dashboard — answers no every time, and a check that
    /// always fails is one people learn to ignore. Expectations still apply:
    /// `hick:expect` asks "did the claim hold", which stays meaningful.
    pub volatile: bool,
    /// YAML frontmatter, when this document is bare and opens with one.
    ///
    /// A *view* over bytes that are also present in [`HickDocument::nodes`] as
    /// text — never a replacement for them. `None` for a wrapped document,
    /// which configures itself with attributes on its root instead.
    pub frontmatter: Option<Frontmatter>,
    /// Files whose text was spliced into `nodes` by include/upstream
    /// resolution, as canonical paths. A span whose
    /// [`SourceSpan::file_id`] is `Some(i)` holds byte offsets into
    /// `span_files[i]`, not into [`HickDocument::source`]. Empty until
    /// [`resolve_includes`] runs, and stays empty for a document that
    /// includes nothing.
    pub span_files: Vec<String>,
    /// The root element of a wrapped document (`<hick:doc …>`), with its
    /// attributes and both spans and no children — so a reader that draws
    /// the document's tags can draw this one too. `None` for a bare document.
    pub root_tag: Option<HickTag>,
}

// Session AST types

/// An embedded script block inside a `hick:assistant` node.
///
/// `lang` is the language identifier (e.g. `"python"`, `"sh"`, `"js"`).
/// `code` is the raw script content.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionBlock {
    /// Language identifier (e.g. `"python"`, `"sh"`).
    pub lang: String,
    /// Raw script source code.
    pub code: String,
}

/// A typed turn in an AI agent session, extracted from `hick:session` nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionNode {
    /// A user message (inert during replay — loaded as context only).
    User { text: String },
    /// A full LLM response with optional embedded action blocks.
    ///
    /// During replay: embedded `hick:action` scripts are executed; the LLM is
    /// not called.
    Assistant {
        /// Prose text of the LLM response (excluding action blocks).
        text: String,
        /// Embedded script blocks to execute during replay.
        actions: Vec<ActionBlock>,
    },
    /// Captured stdout/stderr from a prior script or command execution.
    ///
    /// Inert during replay — the output was already captured in the original
    /// session and does not need to be re-executed.
    Observation {
        /// Identifies which action produced this observation (e.g. `"action-0"`).
        source: Option<String>,
        /// Exit code of the originating command.
        exit_code: Option<i32>,
        /// Captured output text.
        text: String,
    },
    /// A direct shell command typed by the user with the `!` prefix.
    ///
    /// Executed in the WASM container during replay.
    Command { text: String },
}

/// A parsed `.md` session file (root element: `hick:session`).
///
/// Session files record AI agent conversations as both a transcript and a
/// replayable program.  Running `hick run session.md` re-executes all
/// side-effecting steps without calling the LLM.
#[derive(Debug, Clone)]
pub struct SessionDocument {
    /// Typed session turns in document order.
    pub nodes: Vec<SessionNode>,
    /// Original source text.
    pub source: String,
    /// The XML namespace prefix used in this document.
    pub prefix: String,
    /// RFC 3339 timestamp from the `start` attribute of the root element.
    pub start_time: Option<String>,
}

impl HickDocument {
    /// Iterate over top-level tags.
    pub fn tags(&self) -> impl Iterator<Item = &HickTag> {
        self.nodes.iter().filter_map(|n| match n {
            HickNode::Tag(t) => Some(t),
            HickNode::Text(..) => None,
        })
    }

    /// Every tag in the document, at any depth, in document order.
    ///
    /// `tags()` is the top level only, which is the right question for a
    /// document's sections and the wrong one for its cells: an exec inside a
    /// `hick:file` is still a cell.
    pub fn all_tags(&self) -> Vec<&HickTag> {
        fn walk<'a>(nodes: &'a [HickNode], out: &mut Vec<&'a HickTag>) {
            for node in nodes {
                if let HickNode::Tag(tag) = node {
                    out.push(tag);
                    walk(&tag.children, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.nodes, &mut out);
        out
    }

    /// Find all tags with a given local name (non-recursive).
    pub fn find_tags(&self, name: &str) -> Vec<&HickTag> {
        self.tags().filter(|t| t.name == name).collect()
    }
}

// Errors

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("missing root doc element (no xmlns for {HICK_NAMESPACE})")]
    MissingRoot,

    #[error("unclosed tag <{prefix}:{name}> opened at line {line}")]
    UnclosedTag {
        prefix: String,
        name: String,
        line: usize,
    },

    #[error("line {line}: unexpected closing tag </{prefix}:{name}>")]
    UnexpectedClose {
        prefix: String,
        name: String,
        line: usize,
    },

    #[error("unclosed comment starting at line {line} (missing closing `-->`)")]
    UnclosedComment { line: usize },
}

// Namespace prefix detection

/// Scan the source for `xmlns:PREFIX="http://www.hickorydocs.com/1.0"` and
/// return the prefix. Falls back to `"hick"` if no declaration is found.
fn detect_prefix(input: &str) -> String {
    let xmlns_marker = "xmlns:";

    let mut search_from = 0;
    while let Some(rel_pos) = input[search_from..].find(xmlns_marker) {
        let prefix_start = search_from + rel_pos + xmlns_marker.len();

        // Read prefix chars until = or whitespace
        let mut prefix_end = prefix_start;
        while prefix_end < input.len() {
            let ch = input.as_bytes()[prefix_end];
            if ch == b'=' || ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                break;
            }
            prefix_end += 1;
        }

        if prefix_end == prefix_start {
            search_from = prefix_end;
            continue;
        }

        let prefix = &input[prefix_start..prefix_end];

        // Skip whitespace before '='
        let mut val_pos = prefix_end;
        while val_pos < input.len() {
            let ch = input.as_bytes()[val_pos];
            if ch != b' ' && ch != b'\t' && ch != b'\n' && ch != b'\r' {
                break;
            }
            val_pos += 1;
        }

        if val_pos >= input.len() || input.as_bytes()[val_pos] != b'=' {
            search_from = val_pos;
            continue;
        }
        val_pos += 1; // skip =

        // Skip whitespace after '='
        while val_pos < input.len() {
            let ch = input.as_bytes()[val_pos];
            if ch != b' ' && ch != b'\t' && ch != b'\n' && ch != b'\r' {
                break;
            }
            val_pos += 1;
        }

        if val_pos >= input.len() {
            search_from = val_pos;
            continue;
        }

        let quote = input.as_bytes()[val_pos];
        if quote != b'"' && quote != b'\'' {
            search_from = val_pos;
            continue;
        }
        val_pos += 1; // skip opening quote

        if input[val_pos..].starts_with(HICK_NAMESPACE) {
            return prefix.to_string();
        }

        search_from = val_pos;
    }

    "hick".to_string()
}

// Bare documents: frontmatter, prefix resolution, wrapped/bare detection

/// YAML frontmatter at the head of a bare document.
///
/// See `docs/specs/freeform/bare-documents.md`. Frontmatter is recognised
/// **only** in a bare document — a wrapped one configures itself with
/// attributes on its root, and two mechanisms for one thing in one file is how
/// a format rots.
///
/// The bytes stay in [`HickDocument::nodes`] as ordinary text. This struct is
/// a *view* over them, not a replacement for them: that is what makes the
/// block weave through verbatim and keeps its spans addressable by lineage,
/// with no reconstruction step on the reverse-edit path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frontmatter {
    /// The block including both `---` fences, byte-for-byte.
    pub raw: String,
    /// Top-level keys in source order, with their scalar values.
    ///
    /// A key whose value is a block (a nested mapping or a multi-line list)
    /// is present with an empty value: the format does not get an opinion
    /// about what a non-reserved key means, so it is recorded and not
    /// interpreted.
    pub entries: Vec<(String, String)>,
    /// Items of any key written as a block list, in source order.
    ///
    /// Recording that `tags` has three items is structure, not meaning — this
    /// says nothing about what `tags` denotes, only what shape it was written
    /// in, so that [`Frontmatter::list`] can answer the same way for the block
    /// and inline spellings.
    pub lists: Vec<(String, Vec<String>)>,
}

impl Frontmatter {
    /// The value of a top-level key, if it has one.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// A key's items, whichever of YAML's two list spellings was used.
    ///
    /// `tags: [a, b]` and a block of `- a` / `- b` lines answer identically.
    /// A key that is absent, or whose value is a plain scalar, has no items.
    pub fn list(&self, key: &str) -> Vec<String> {
        if let Some((_, items)) = self.lists.iter().find(|(k, _)| k == key) {
            return items.clone();
        }
        let Some(value) = self.get(key) else {
            return Vec::new();
        };
        let Some(inner) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) else {
            return Vec::new();
        };
        inner
            .split(',')
            .map(|item| unquote_scalar(item.trim()).to_string())
            .filter(|item| !item.is_empty())
            .collect()
    }
}

/// Keys frontmatter reserves to configure the document. Everything else is
/// metadata and is never interpreted.
pub const RESERVED_FRONTMATTER_KEYS: [&str; 4] = ["weave", "prefix", "volatile", "standings"];

/// The standings a `<hick:claim>` may declare without the document saying so.
///
/// A claim's standing is an assertion about an assertion — nothing verifies
/// it — so the vocabulary is small and closed enough to be comparable across
/// repositories. A document extends it with a `standings:` frontmatter key.
/// See `docs/specs/freeform/provenance-and-standing.md`.
pub const STANDINGS: [&str; 4] = ["expert", "judgment", "report", "assumption"];

/// Strip the surrounding quotes from a YAML scalar, if it has a matching pair.
fn unquote_scalar(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && (bytes[0] == b'"' || bytes[0] == b'\'')
        && bytes[bytes.len() - 1] == bytes[0]
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

/// Split a `key: value` line at column 0, returning the key and its scalar.
///
/// Returns `None` for anything that is not a top-level mapping entry.
fn frontmatter_entry(line: &str) -> Option<(&str, &str)> {
    let key_end = line.find(':')?;
    let key = &line[..key_end];
    if key.is_empty() {
        return None;
    }
    let mut chars = key.chars();
    let first = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.') {
        return None;
    }
    let rest = &line[key_end + 1..];
    // `key:value` is not a YAML mapping entry; `key:` and `key: value` are.
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    Some((key, unquote_scalar(rest.trim())))
}

/// Recognise YAML frontmatter at the head of `input`.
///
/// Returns the parsed view and the byte offset at which the document body
/// begins. Conservative by construction: anything this does not recognise as a
/// mapping is content, which is the safe direction to be wrong in — a note
/// that opens with a `---` horizontal rule keeps its rule rather than losing
/// its first paragraph to a metadata block.
///
/// The recognised subset is deliberately smaller than YAML: top-level
/// `key: value` entries, comments, blank lines, and indented or `- ` block
/// content under a key. See `bare-documents.md` for why this is a subset and
/// not a dependency.
fn split_frontmatter(input: &str) -> Option<(Frontmatter, usize)> {
    let after_fence = input
        .strip_prefix("---\n")
        .or_else(|| input.strip_prefix("---\r\n"))?;
    let fence_len = input.len() - after_fence.len();

    let mut entries: Vec<(String, String)> = Vec::new();
    let mut lists: Vec<(String, Vec<String>)> = Vec::new();
    let mut offset = fence_len;
    let mut saw_entry = false;

    for line in after_fence.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let body = body.strip_suffix('\r').unwrap_or(body);

        if body == "---" || body == "..." {
            let end = offset + line.len();
            let raw = input[..end].to_string();
            // A fenced block with no mapping entry at all is not frontmatter.
            // `---\n---\n` opening a note is two horizontal rules.
            if !saw_entry {
                return None;
            }
            return Some((
                Frontmatter {
                    raw,
                    entries,
                    lists,
                },
                end,
            ));
        }

        if body.trim().is_empty() || body.trim_start().starts_with('#') {
            offset += line.len();
            continue;
        }

        // Indented content and list items belong to the entry above them.
        if body.starts_with([' ', '\t']) || body.starts_with("- ") {
            if !saw_entry {
                return None;
            }
            if let Some(item) = body.trim_start().strip_prefix("- ")
                && let Some((key, _)) = entries.last()
            {
                let item = unquote_scalar(item.trim()).to_string();
                match lists.iter_mut().find(|(k, _)| k == key) {
                    Some((_, items)) => items.push(item),
                    None => lists.push((key.clone(), vec![item])),
                }
            }
            offset += line.len();
            continue;
        }

        match frontmatter_entry(body) {
            Some((key, value)) => {
                saw_entry = true;
                entries.push((key.to_string(), value.to_string()));
            }
            // A top-level line that is not a mapping entry means this was
            // never a mapping. Fall back to content.
            None => return None,
        }

        offset += line.len();
    }

    // Unterminated: no closing fence anywhere in the file.
    None
}

/// The namespace prefix declared by an `xmlns:` binding to [`HICK_NAMESPACE`],
/// if the source has one.
fn declared_prefix(input: &str) -> Option<String> {
    let declared = detect_prefix(input);
    let marker = format!("xmlns:{declared}");
    input.contains(&marker).then_some(declared)
}

/// The prefix a document's tags carry.
///
/// An `xmlns:` declaration wins. Failing that, a bare document may name one in
/// frontmatter. Failing both, it is `hick` — which is why an ordinary note
/// declares nothing, and why documentation *about* hick, which rebinds to `h:`
/// so that `hick:` examples stay literal text, keeps its explicit root.
fn resolve_prefix(input: &str) -> String {
    if let Some(prefix) = declared_prefix(input) {
        return prefix;
    }
    if let Some((frontmatter, _)) = split_frontmatter(input)
        && let Some(prefix) = frontmatter.get("prefix")
        && !prefix.is_empty()
        && prefix
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return prefix.to_string();
    }
    "hick".to_string()
}

/// Does this source open an explicit `<PREFIX:doc>` root?
///
/// The question is answered from the **prologue** — an optional XML
/// declaration, comments, and frontmatter — and not by searching the file.
/// Searching was safe while prose could not precede the root; now that it can,
/// a sentence mentioning `<hick:doc` would be parsed as one.
fn opens_root(input: &str, prefix: &str, root: &str) -> bool {
    let mut rest = input.trim_start();

    if rest.starts_with("<?xml")
        && let Some(end) = rest.find("?>")
    {
        rest = rest[end + 2..].trim_start();
    }

    if let Some((_, body_at)) = split_frontmatter(rest) {
        rest = rest[body_at..].trim_start();
    }

    loop {
        rest = rest.trim_start();
        if rest.starts_with("<!--") {
            match rest.find("-->") {
                Some(end) => {
                    rest = &rest[end + 3..];
                    continue;
                }
                None => return false,
            }
        }
        break;
    }

    let marker = format!("<{prefix}:{root}");
    let Some(after) = rest.strip_prefix(&marker) else {
        return false;
    };
    // `<hick:document` is not `<hick:doc`.
    after
        .chars()
        .next()
        .is_none_or(|c| c.is_whitespace() || c == '>' || c == '/')
}

/// The markdown file a document weaves when it does not name one: its own
/// file name with a `.md` extension.
///
/// `notes/standup.md` weaves `notes/standup.md`. See `bare-documents.md` —
/// a note that has no readable form is not a note.
///
/// The returned path is the **file name only**, because `weave` — like every
/// `<hick:file path=…>` — is resolved relative to the document's own output
/// directory, and an absolute one is refused as an escaping output path. So
/// this is `standup.md`, which lands beside `notes/standup.md`.
pub fn default_weave_path(doc_path: &std::path::Path) -> String {
    let stem = doc_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    format!("{stem}.md")
}

/// The value of `weave` that means "write no markdown".
pub const WEAVE_NONE: &str = "none";

// Parser

/// Parse a `.md` file from source text.
///
/// The parser detects the namespace prefix from the `xmlns:` declaration
/// that binds to [`HICK_NAMESPACE`], then only recognizes tags with that
/// prefix. Everything else is treated as raw text.
pub fn parse(source: &str) -> Result<HickDocument, ParseError> {
    let mut parser = Parser::new(source);
    parser.parse_document()
}

/// Parse a document that may be malformed, and say what was wrong with it.
///
/// This is the parse an editor runs on every keystroke, where the document is
/// malformed most of the time — a tag is half typed, its closer not yet
/// written. It never fails: an unclosed element extends to the end of the
/// document, a closing tag that matches nothing is text, a tag that does not
/// parse is text from its `<` onward, and an unclosed comment is text. The
/// second half of the answer is the first error [`parse`] would have reported,
/// so a reader can both draw the document and say where it stops being one.
///
/// The recovered tree is for *reading* a document's structure. It must never
/// be run or woven: [`parse`] is what decides whether a document is one.
pub fn parse_lenient(source: &str) -> (HickDocument, Option<ParseError>) {
    let mut parser = Parser::new_lenient(source);
    let doc = parser
        .parse_document()
        .expect("a lenient parse recovers from every error");
    (doc, parser.recovered.take())
}

/// Parse a `.md` file that came from `doc_path`, resolving the weave default.
///
/// This is [`parse`] plus the one thing a parser cannot know on its own: which
/// markdown file the document weaves when it does not name one. Every document
/// weaves a `.md` of its own name unless it says `weave="none"`, because a note
/// with no readable form is not a note. See `bare-documents.md`.
///
/// Callers that only inspect a document's structure — analyses, tests,
/// language-server buffers — want [`parse`] and its unresolved
/// [`HickDocument::weave_path`]. Callers that are about to *write* the
/// document's outputs want this.
pub fn parse_from_path(
    source: &str,
    doc_path: &std::path::Path,
) -> Result<HickDocument, ParseError> {
    let mut doc = parse(source)?;
    doc.weave_path = match doc.weave_path.take() {
        Some(explicit) if explicit == WEAVE_NONE => None,
        Some(explicit) => Some(explicit),
        None => Some(default_weave_path(doc_path)),
    };
    Ok(doc)
}

/// Parse a `.md` session file whose root element is `<PREFIX:session>`.
///
/// Returns typed [`SessionNode`]s extracted from the session's turns.
/// Returns [`ParseError::MissingRoot`] if the source does not contain a
/// `<PREFIX:session>` root element.
pub fn parse_session(source: &str) -> Result<SessionDocument, ParseError> {
    let mut parser = Parser::new(source);
    parser.parse_session_document()
}

/// Return `true` if `source` appears to be a session file (contains a
/// `<PREFIX:session` root element).  This is a fast pre-check that avoids
/// full parsing when deciding whether to enter replay mode.
pub fn is_session_source(source: &str) -> bool {
    let prefix = detect_prefix(source);
    let marker = format!("<{prefix}:session");
    source.contains(&marker)
}

// Include resolution

/// Resolve all `<hick:include>` tags in a parsed document, recursively.
///
/// `base_dir` is the directory of the document being processed.
/// `seen` tracks already-included paths for cycle detection.
pub fn resolve_includes(
    doc: &mut HickDocument,
    base_dir: &std::path::Path,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
) -> Result<(), ParseError> {
    let mut merged: std::collections::HashSet<std::path::PathBuf> =
        std::collections::HashSet::new();
    let mut span_files: Vec<String> = std::mem::take(&mut doc.span_files);
    let new_nodes = resolve_includes_in_nodes(
        std::mem::take(&mut doc.nodes),
        base_dir,
        seen,
        &mut merged,
        &mut span_files,
    )?;
    doc.nodes = new_nodes;
    doc.span_files = span_files;
    // Only documents that actually declare a pipeline pay for the uniqueness
    // check, so this cannot fail a document that worked before.
    if !merged.is_empty() {
        check_unique_ids(&doc.nodes)?;
    }
    Ok(())
}

/// `#id` must identify exactly one fragment across a document and everything
/// upstream of it.
///
/// A `.class` selector is explicitly multi-match — several documents
/// contributing the same class is how a chain accumulates decisions — but an
/// `#id` that resolves to two different fragments has no defensible answer.
/// Picking one by a precedence rule is the failure mode that silently binds a
/// requirement to the wrong text, so this is an error instead.
fn check_unique_ids(nodes: &[HickNode]) -> Result<(), ParseError> {
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut stack: Vec<&HickNode> = nodes.iter().collect();
    let mut dupes: Vec<(String, usize, usize)> = Vec::new();
    while let Some(node) = stack.pop() {
        if let HickNode::Tag(tag) = node {
            if is_fragment_tag(&tag.name)
                && let Some(id) = tag.get_attribute("id")
                && let Some(first) = seen.insert(id.to_string(), tag.source_line)
            {
                dupes.push((id.to_string(), first, tag.source_line));
            }
            stack.extend(tag.children.iter());
        }
    }
    if let Some((id, a, b)) = dupes.into_iter().min_by_key(|(_, a, _)| *a) {
        return Err(ParseError::Syntax {
            line: b.min(a),
            message: format!(
                "duplicate fragment id '#{id}' in this document and its upstream \
                 (also at line {}). Ids must be unique across the pipeline; use a \
                 class if you meant several fragments to match.",
                a.max(b)
            ),
        });
    }
    Ok(())
}

/// Every fragment block in `nodes`, in document order, and nothing else.
/// Splice another document's OWN fragments in as an ambient contribution.
///
/// Non-recursive on purpose. An ambient edge means "this folder's documents
/// offer their fragments", and what a document offers is what it declares —
/// following its upstreams as well would make every document upstream of
/// every other, which is a cycle the moment any explicit chain exists.
/// Whatever the contributor itself upstreams is either in this folder and
/// contributing on its own account, or deliberately out of scope.
///
/// Appended, never prepended, so a local fragment keeps its place: `#id`
/// resolution takes the first match, and a document's own declaration must
/// win over one it never asked for. For `.class` both are collected, which is
/// the additive behaviour the whole feature exists for.
///
/// Spans are stamped with the contributor's file, so lineage and the reverse
/// edit land in the document that wrote the bytes.
pub fn attach_contribution(
    doc: &mut HickDocument,
    contributor: &std::path::Path,
    source: &str,
    line: usize,
) -> Result<(), ParseError> {
    let contributed = parse(source)?;
    let mut fragments = fragments_of(contributed.nodes);
    if fragments.is_empty() {
        return Ok(());
    }
    let mut span_files: Vec<String> = std::mem::take(&mut doc.span_files);
    let file_id = span_file_id(&mut span_files, contributor, line)?;
    stamp_span_file(&mut fragments, file_id);
    doc.span_files = span_files;
    doc.nodes.push(HickNode::Tag(HickTag {
        name: "upstream".to_string(),
        attributes: vec![("file".to_string(), contributor.display().to_string())],
        children: fragments,
        self_closing: false,
        source_line: line,
        source_column: 0,
        source_span: None,
        close_span: None,
    }));
    Ok(())
}

fn fragments_of(nodes: Vec<HickNode>) -> Vec<HickNode> {
    let mut out = Vec::new();
    collect_fragments_any(&nodes, &mut out);
    out
}

/// Every fragment tag travels upstream — `transcript` included, so a note
/// downstream of a meeting can quote a turn (`#transcript-u7`) or one
/// speaker (`.said-sam`). The turns themselves are derived after splicing,
/// from the transcript's raw bytes, which keep their file stamp.
fn collect_fragments_any(nodes: &[HickNode], out: &mut Vec<HickNode>) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if is_fragment_tag(&tag.name) {
                out.push(HickNode::Tag(tag.clone()));
                continue;
            }
            collect_fragments_any(&tag.children, out);
        }
    }
}

/// Every fragment block (`copy`/`cut`) in `doc` matching `selector`, in
/// document order. The selector grammar is `hick:paste`'s: `#id`, `.class`,
/// or a comma-separated list.
pub fn fragments_matching<'a>(doc: &'a HickDocument, selector: &str) -> Vec<&'a HickTag> {
    let mut out = Vec::new();
    collect_matching(&doc.nodes, selector, &mut out);
    out
}

/// Elements whose content is a selectable fragment.
///
/// `transcript` and `said` join `copy`/`cut` because an ingested meeting is
/// exactly the thing a `hick:transform` wants to point at — either whole
/// (`select="#t"`) or one speaker's turns (`select=".said-sam"`). Without
/// them the derived turns would be addressable in principle and selectable by
/// nothing. See `docs/specs/freeform/ingest.md`.
fn is_fragment_tag(name: &str) -> bool {
    matches!(name, "copy" | "cut" | "transcript" | "said")
}

fn collect_matching<'a>(nodes: &'a [HickNode], selector: &str, out: &mut Vec<&'a HickTag>) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if is_fragment_tag(&tag.name) && fragment_matches(tag, selector) {
                // No recursion into a match: selecting a whole transcript must
                // not also select each of its turns and count them twice.
                out.push(tag);
                continue;
            }
            collect_matching(&tag.children, selector, out);
        }
    }
}

/// The raw text directly inside a tag (its `Text` children, concatenated).
pub fn tag_text(tag: &HickTag) -> String {
    let mut out = String::new();
    for child in &tag.children {
        if let HickNode::Text(t, _) = child {
            out.push_str(t);
        }
    }
    out
}

/// Fingerprint of a transform's inputs: the bytes it was written from plus the
/// instruction it was written under.
///
/// This is what `hick test` compares, and why checking a transform never
/// needs a model: an LLM-written passage cannot be re-derived byte-for-byte, so
/// the document does not claim it reproduces — it claims it was written from
/// EXACTLY these bytes under EXACTLY this instruction, and that neither has
/// changed since. The instruction is inside the fingerprint deliberately:
/// editing "summarize" to "summarize briefly" must invalidate the passage, or
/// stale prose would keep its attestation.
///
/// FNV-1a/64 folded to 8 hex characters — stable across runs and machines, and
/// short enough to sit in an attribute a human reads in a diff.
pub fn transform_fingerprint(input: &str, instruct: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in input
        .bytes()
        .chain(b"\0".iter().copied())
        .chain(instruct.bytes())
    {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:08x}", (hash ^ (hash >> 32)) as u32)
}

/// A fingerprint of a passage's own bytes, for `wrote=`.
///
/// `transform_fingerprint` pins what a passage was written FROM. This pins
/// what was written — the other half, and the one that says whether the words
/// in the document are still the words the model produced.
///
/// Trimmed before hashing, because a passage's surrounding newlines are the
/// element's formatting rather than its content: re-indenting a document must
/// not read as somebody rewriting the model.
pub fn passage_fingerprint(passage: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in passage.trim().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:08x}", (hash ^ (hash >> 32)) as u32)
}

fn fragment_matches(tag: &HickTag, selector: &str) -> bool {
    selector
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .any(|sel| {
            if let Some(id) = sel.strip_prefix('#') {
                tag.get_attribute("id") == Some(id)
            } else if let Some(class) = sel.strip_prefix('.') {
                tag.get_attribute("class")
                    .is_some_and(|c| c.split_whitespace().any(|x| x == class))
            } else {
                false
            }
        })
}

/// Register `canonical` in the span-file table and return its id.
fn span_file_id(
    span_files: &mut Vec<String>,
    canonical: &std::path::Path,
    line: usize,
) -> Result<u16, ParseError> {
    let path = canonical.to_string_lossy().to_string();
    if let Some(existing) = span_files.iter().position(|p| *p == path) {
        return Ok(existing as u16);
    }
    if span_files.len() > usize::from(u16::MAX) {
        return Err(ParseError::Syntax {
            line,
            message: "more than 65 536 distinct files spliced into one document".into(),
        });
    }
    span_files.push(path);
    Ok((span_files.len() - 1) as u16)
}

/// Stamp every span that does not yet name a file with `file_id`.
///
/// Spliced spans hold byte offsets into the file they were parsed from, so
/// they must say which file that is before they leave the splice — an
/// unstamped span is read as offsets into the INCLUDING document, which is
/// the corruption this exists to prevent. Nested includes are stamped by
/// their own (deeper) splice first, which is why only `None` is overwritten.
fn stamp_span_file(nodes: &mut [HickNode], file_id: u16) {
    for node in nodes {
        match node {
            HickNode::Text(_, Some(span)) => {
                if span.file_id.is_none() {
                    span.file_id = Some(file_id);
                }
            }
            HickNode::Text(_, None) => {}
            HickNode::Tag(tag) => {
                if let Some(span) = &mut tag.source_span
                    && span.file_id.is_none()
                {
                    span.file_id = Some(file_id);
                }
                stamp_span_file(&mut tag.children, file_id);
            }
        }
    }
}

fn resolve_includes_in_nodes(
    nodes: Vec<HickNode>,
    base_dir: &std::path::Path,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
    merged: &mut std::collections::HashSet<std::path::PathBuf>,
    span_files: &mut Vec<String>,
) -> Result<Vec<HickNode>, ParseError> {
    let mut result = Vec::new();
    for node in nodes {
        match node {
            // A pipeline edge. Everything upstream becomes SELECTABLE — its
            // fragments, transitively — and nothing of it is rendered. This is
            // what lets a requirements document quote a decision three hops
            // back without restating the import at every hop.
            HickNode::Tag(tag) if tag.name == "upstream" => {
                let file_attr = tag
                    .get_attribute("file")
                    .ok_or_else(|| ParseError::Syntax {
                        line: tag.source_line,
                        message: "<hick:upstream> missing 'file' attribute".into(),
                    })?;
                let canonical = std::path::Path::new(base_dir)
                    .join(file_attr)
                    .canonicalize()
                    .map_err(|e| ParseError::Syntax {
                        line: tag.source_line,
                        message: format!("upstream '{file_attr}': {e}"),
                    })?;
                if !seen.insert(canonical.clone()) {
                    return Err(ParseError::Syntax {
                        line: tag.source_line,
                        message: format!("circular upstream: {}", canonical.display()),
                    });
                }
                // A diamond (A upstream of B and C, both upstream of D) is the
                // normal shape of a pipeline, not an error — but D must merge
                // A's fragments once, or every id in it would collide with
                // itself.
                if merged.insert(canonical.clone()) {
                    let source =
                        std::fs::read_to_string(&canonical).map_err(|e| ParseError::Syntax {
                            line: tag.source_line,
                            message: format!("cannot read '{}': {e}", canonical.display()),
                        })?;
                    let mut upstream_doc = parse(&source)?;
                    let upstream_dir = canonical.parent().unwrap_or(base_dir);
                    upstream_doc.nodes = resolve_includes_in_nodes(
                        upstream_doc.nodes,
                        upstream_dir,
                        seen,
                        merged,
                        span_files,
                    )?;
                    // Stamp AFTER the recursion: spans belonging to files
                    // the upstream itself spliced are already stamped, and
                    // what remains is the upstream's own text.
                    let file_id = span_file_id(span_files, &canonical, tag.source_line)?;
                    let mut fragments = fragments_of(upstream_doc.nodes);
                    stamp_span_file(&mut fragments, file_id);
                    // The edge stays in the tree, holding what it brought: a
                    // `hick:upstream` node whose children are the upstream's
                    // fragments. Selectors reach into it; the weave renders
                    // none of it — which is what keeps a spliced transcript
                    // selectable turn by turn without reprinting the meeting
                    // in every document downstream of it.
                    result.push(HickNode::Tag(HickTag {
                        children: fragments,
                        self_closing: false,
                        ..tag
                    }));
                } else {
                    result.push(HickNode::Tag(HickTag {
                        children: Vec::new(),
                        self_closing: true,
                        ..tag
                    }));
                }
                seen.remove(&canonical);
            }
            HickNode::Tag(tag) if tag.name == "include" => {
                let file_attr = tag
                    .get_attribute("file")
                    .ok_or_else(|| ParseError::Syntax {
                        line: tag.source_line,
                        message: "<hick:include> missing 'file' attribute".into(),
                    })?;

                let include_path = base_dir.join(file_attr);
                let canonical = include_path
                    .canonicalize()
                    .map_err(|e| ParseError::Syntax {
                        line: tag.source_line,
                        message: format!("include path '{}': {e}", include_path.display()),
                    })?;

                // Cycle detection
                if !seen.insert(canonical.clone()) {
                    return Err(ParseError::Syntax {
                        line: tag.source_line,
                        message: format!("circular include: {}", canonical.display()),
                    });
                }

                // Parse included file
                let source =
                    std::fs::read_to_string(&canonical).map_err(|e| ParseError::Syntax {
                        line: tag.source_line,
                        message: format!("cannot read '{}': {e}", canonical.display()),
                    })?;
                let mut included_doc = parse(&source)?;

                // Recursively resolve includes in the included doc, sharing
                // the outer span-file table so a nested splice's ids are
                // valid in the document that finally holds the nodes.
                let included_dir = canonical.parent().unwrap_or(base_dir);
                let mut inner_merged: std::collections::HashSet<std::path::PathBuf> =
                    std::collections::HashSet::new();
                included_doc.nodes = resolve_includes_in_nodes(
                    included_doc.nodes,
                    included_dir,
                    seen,
                    &mut inner_merged,
                    span_files,
                )?;
                if !inner_merged.is_empty() {
                    check_unique_ids(&included_doc.nodes)?;
                }

                // Splice children into parent. `include` is textual
                // composition — a manual out of chapters. To make another
                // document's fragments SELECTABLE without rendering it, the
                // element is `hick:upstream`. Stamp after the recursion, so
                // only the included file's own text gets its id.
                let file_id = span_file_id(span_files, &canonical, tag.source_line)?;
                stamp_span_file(&mut included_doc.nodes, file_id);
                result.extend(included_doc.nodes);

                seen.remove(&canonical);
            }
            HickNode::Tag(mut tag) => {
                // Check `when` attribute (skip if needed -- handled upstream)
                // Recurse into children for nested includes (e.g. inside file tags)
                tag.children =
                    resolve_includes_in_nodes(tag.children, base_dir, seen, merged, span_files)?;
                result.push(HickNode::Tag(tag));
            }
            text => {
                result.push(text);
            }
        }
    }
    Ok(result)
}

fn extract_session_nodes(nodes: &[HickNode]) -> Vec<SessionNode> {
    let mut result = Vec::new();
    for node in nodes {
        match node {
            HickNode::Tag(tag) => match tag.name.as_str() {
                "user" => result.push(SessionNode::User {
                    text: tag.text_content(),
                }),
                "assistant" => {
                    let mut prose = String::new();
                    let mut actions = Vec::new();
                    for child in &tag.children {
                        match child {
                            HickNode::Text(t, _) => prose.push_str(t),
                            HickNode::Tag(child_tag) if child_tag.name == "action" => {
                                let lang = child_tag
                                    .get_attribute("lang")
                                    .unwrap_or("python")
                                    .to_string();
                                actions.push(ActionBlock {
                                    lang,
                                    code: child_tag.text_content(),
                                });
                            }
                            // Anything else inside an assistant turn is part
                            // of what it said — the same rule
                            // `session_view` applies. It matters for
                            // `hick:input`, which is how a session records
                            // prose that quotes a hick tag: dropping it here
                            // meant the two readers of one session disagreed
                            // about what the model had answered, and this one
                            // silently lost the answer.
                            HickNode::Tag(child_tag) => {
                                prose.push_str(&child_tag.text_content());
                            }
                        }
                    }
                    result.push(SessionNode::Assistant {
                        text: prose.trim().to_string(),
                        actions,
                    });
                }
                "observation" => {
                    let source = tag.get_attribute("source").map(|s| s.to_string());
                    let exit_code = tag
                        .get_attribute("exit")
                        .and_then(|s| s.parse::<i32>().ok());
                    result.push(SessionNode::Observation {
                        source,
                        exit_code,
                        text: tag.text_content(),
                    });
                }
                "command" => result.push(SessionNode::Command {
                    text: tag.text_content(),
                }),
                _ => {} // unknown tags are gracefully skipped
            },
            HickNode::Text(..) => {} // whitespace between turns is ignored
        }
    }
    result
}

impl fmt::Display for HickTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<hick:{}", self.name)?;
        for (k, v) in &self.attributes {
            write!(f, " {k}=\"{v}\"")?;
        }
        if self.self_closing {
            write!(f, " />")?;
        } else {
            write!(f, ">")?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Content dedenting
// ---------------------------------------------------------------------------

/// Dedent text content by removing up to `max_indent` leading spaces from each line.
///
/// This is used to normalize content within hick elements. When an element is indented
/// in the source file, its content typically has the same indentation which should be
/// stripped for the output.
///
/// The function handles the common case where content starts with a newline (the opening
/// tag is on its own line), stripping that leading newline as well.
///
/// # Examples
///
/// ```
/// use hick_lang::dedent;
///
/// let text = "\n    def hello():\n        print('hi')";
/// let result = dedent(text, 4);
/// assert_eq!(result, "def hello():\n    print('hi')");
/// ```
pub fn dedent(text: &str, max_indent: usize) -> String {
    if max_indent == 0 {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    let mut first = true;

    // Handle content that starts with a newline (opening tag on its own line)
    let text_trimmed = if let Some(stripped) = text.strip_prefix('\n') {
        stripped
    } else {
        text
    };

    for line in text_trimmed.lines() {
        if !first {
            result.push('\n');
        }
        first = false;

        // Count leading spaces
        let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
        let strip = leading_spaces.min(max_indent);
        result.push_str(&line[strip..]);
    }

    // Preserve trailing newline if original had one
    if text_trimmed.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "lib/tests/tests.rs"]
mod tests;
</storycode:entry>

</storycode:doc>
