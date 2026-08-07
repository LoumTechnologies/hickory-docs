//! Parser and AST for `.hick` files.
//!
//! Tags prefixed with the namespace bound to `http://www.hickorydocs.com/1.0`
//! are parsed as structured elements. Content between them is raw text that
//! can contain `&`, `<`, `>` without escaping.
//!
//! The namespace prefix is detected from the `xmlns:` declaration on the root
//! element. For example, `<h:doc xmlns:h="http://www.hickorydocs.com/1.0">`
//! uses prefix `h`, while `<hick:doc xmlns:hick="...">` uses prefix `hick`.

use std::fmt;

/// The XML namespace URI that identifies hick elements.
pub const HICK_NAMESPACE: &str = "http://www.hickorydocs.com/1.0";

// ---------------------------------------------------------------------------
// Source spans
// ---------------------------------------------------------------------------

/// Byte-level source location for provenance tracking.
///
/// Tracks where a piece of text or tag originated in the `.hick` source file.
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
}

impl SourceSpan {
    pub fn new(start: usize, end: usize, start_line: usize, start_col: usize) -> Self {
        Self {
            start,
            end,
            start_line,
            start_col,
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

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

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

/// A parsed `.hick` document.
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
}

// ---------------------------------------------------------------------------
// Session AST types
// ---------------------------------------------------------------------------

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

/// A parsed `.hick` session file (root element: `hick:session`).
///
/// Session files record AI agent conversations as both a transcript and a
/// replayable program.  Running `hick run session.hick` re-executes all
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

    /// Find all tags with a given local name (non-recursive).
    pub fn find_tags(&self, name: &str) -> Vec<&HickTag> {
        self.tags().filter(|t| t.name == name).collect()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Namespace prefix detection
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a `.hick` file from source text.
///
/// The parser detects the namespace prefix from the `xmlns:` declaration
/// that binds to [`HICK_NAMESPACE`], then only recognizes tags with that
/// prefix. Everything else is treated as raw text.
pub fn parse(source: &str) -> Result<HickDocument, ParseError> {
    let mut parser = Parser::new(source);
    parser.parse_document()
}

/// Parse a `.hick` session file whose root element is `<PREFIX:session>`.
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

// ---------------------------------------------------------------------------
// Include resolution
// ---------------------------------------------------------------------------

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
    let new_nodes =
        resolve_includes_in_nodes(std::mem::take(&mut doc.nodes), base_dir, seen, &mut merged)?;
    doc.nodes = new_nodes;
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
            if (tag.name == "copy" || tag.name == "cut")
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
fn fragments_of(nodes: Vec<HickNode>) -> Vec<HickNode> {
    let mut out = Vec::new();
    collect_fragments_any(&nodes, &mut out);
    out
}

fn collect_fragments_any(nodes: &[HickNode], out: &mut Vec<HickNode>) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.name == "copy" || tag.name == "cut" {
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

fn collect_matching<'a>(nodes: &'a [HickNode], selector: &str, out: &mut Vec<&'a HickTag>) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if (tag.name == "copy" || tag.name == "cut") && fragment_matches(tag, selector) {
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
/// This is what `hickory check` compares, and why checking a transform never
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

fn resolve_includes_in_nodes(
    nodes: Vec<HickNode>,
    base_dir: &std::path::Path,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
    merged: &mut std::collections::HashSet<std::path::PathBuf>,
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
                    upstream_doc.nodes =
                        resolve_includes_in_nodes(upstream_doc.nodes, upstream_dir, seen, merged)?;
                    result.extend(fragments_of(upstream_doc.nodes));
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

                // Recursively resolve includes in the included doc
                let included_dir = canonical.parent().unwrap_or(base_dir);
                resolve_includes(&mut included_doc, included_dir, seen)?;

                // Splice children into parent. `include` is textual
                // composition — a manual out of chapters. To make another
                // document's fragments SELECTABLE without rendering it, the
                // element is `hick:upstream`.
                result.extend(included_doc.nodes);

                seen.remove(&canonical);
            }
            HickNode::Tag(mut tag) => {
                // Check `when` attribute (skip if needed -- handled upstream)
                // Recurse into children for nested includes (e.g. inside file tags)
                tag.children = resolve_includes_in_nodes(tag.children, base_dir, seen, merged)?;
                result.push(HickNode::Tag(tag));
            }
            text => {
                result.push(text);
            }
        }
    }
    Ok(result)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    /// 0-based column position in current line.
    col: usize,
    /// The detected namespace prefix (e.g. `"hick"`, `"h"`).
    prefix: String,
    /// Opening tag marker, e.g. `"<hick:"`.
    open_marker: String,
    /// Closing tag marker, e.g. `"</hick:"`.
    close_marker: String,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let prefix = detect_prefix(input);
        let open_marker = format!("<{prefix}:");
        let close_marker = format!("</{prefix}:");
        Self {
            input,
            pos: 0,
            line: 1,
            col: 0,
            prefix,
            open_marker,
            close_marker,
        }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Advance past the XML declaration if present.
    fn skip_xml_declaration(&mut self) {
        let rem = self.remaining().trim_start();
        let skipped_ws = self.remaining().len() - rem.len();
        if rem.starts_with("<?xml")
            && let Some(end) = rem.find("?>")
        {
            let total = skipped_ws + end + 2;
            self.advance(total);
        }
    }

    /// If the cursor is at `<!--`, skip past the matching `-->`.
    ///
    /// Returns `true` if a comment was skipped.
    fn skip_comment(&mut self) -> Result<bool, ParseError> {
        let rem = self.remaining();
        if !rem.starts_with("<!--") {
            return Ok(false);
        }
        let comment_line = self.line;
        match rem[4..].find("-->") {
            Some(end) => {
                self.advance(4 + end + 3);
                Ok(true)
            }
            None => Err(ParseError::UnclosedComment { line: comment_line }),
        }
    }

    /// Advance `n` bytes, tracking line and column numbers.
    fn advance(&mut self, n: usize) {
        let slice = &self.input[self.pos..self.pos + n];
        for ch in slice.chars() {
            if ch == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        }
        self.pos += n;
    }

    /// Parse the full document.
    fn parse_document(&mut self) -> Result<HickDocument, ParseError> {
        self.skip_xml_declaration();

        // Find the root <PREFIX:doc ...> tag
        let doc_marker = format!("<{}:doc", self.prefix);
        let root_start = self
            .remaining()
            .find(&doc_marker)
            .ok_or(ParseError::MissingRoot)?;
        self.advance(root_start);

        // Parse the opening tag attributes and consume `>`
        let tag = self.parse_open_tag()?;
        if tag.name != "doc" {
            return Err(ParseError::MissingRoot);
        }

        // Extract weave path from the doc tag attributes
        let weave_path = tag.get_attribute("weave").map(|s| s.to_string());
        let volatile = tag.get_attribute("volatile") == Some("true");

        // Parse children until </PREFIX:doc>
        let nodes = self.parse_children("doc", tag.source_line)?;

        Ok(HickDocument {
            nodes,
            source: self.input.to_string(),
            prefix: self.prefix.clone(),
            weave_path,
            volatile,
        })
    }

    /// Parse a session document whose root element is `<PREFIX:session>`.
    fn parse_session_document(&mut self) -> Result<SessionDocument, ParseError> {
        self.skip_xml_declaration();

        let session_marker = format!("<{}:session", self.prefix);
        let root_start = self
            .remaining()
            .find(&session_marker)
            .ok_or(ParseError::MissingRoot)?;
        self.advance(root_start);

        let tag = self.parse_open_tag()?;
        if tag.name != "session" {
            return Err(ParseError::MissingRoot);
        }

        let start_time = tag.get_attribute("start").map(|s| s.to_string());

        let raw_nodes = self.parse_children("session", tag.source_line)?;
        let nodes = extract_session_nodes(&raw_nodes);

        Ok(SessionDocument {
            nodes,
            source: self.input.to_string(),
            prefix: self.prefix.clone(),
            start_time,
        })
    }

    /// Parse children until the matching closing tag is found.
    /// Elements whose content is captured verbatim (raw text up to the next
    /// matching close marker) instead of being parsed for nested tags.
    ///
    /// These carry the agent session vocabulary's payloads: `hick:input`
    /// (multi-line tool-call payloads, e.g. replacement text that may quote
    /// unbalanced hick fragments) and `hick:tool-result` (tool observations
    /// that may excerpt arbitrary document slices). The only substring such
    /// content cannot contain is its own literal close tag.
    fn parse_children(
        &mut self,
        close_name: &str,
        open_line: usize,
    ) -> Result<Vec<HickNode>, ParseError> {
        let mut nodes = Vec::new();
        let mut text_start = self.pos;
        let mut text_start_line = self.line;
        let mut text_start_col = self.col;

        loop {
            if self.is_eof() {
                return Err(ParseError::UnclosedTag {
                    prefix: self.prefix.clone(),
                    name: close_name.to_string(),
                    line: open_line,
                });
            }

            // Look for the next opening marker, closing marker, or comment
            let rem = self.remaining();
            let next_open = rem.find(self.open_marker.as_str());
            let next_close = rem.find(self.close_marker.as_str());
            let next_comment = rem.find("<!--");

            // Determine the nearest marker
            let nearest = [next_open, next_close, next_comment]
                .into_iter()
                .flatten()
                .min();

            match nearest {
                None => {
                    // No more tags -- rest is text (will be caught as unclosed)
                    return Err(ParseError::UnclosedTag {
                        prefix: self.prefix.clone(),
                        name: close_name.to_string(),
                        line: open_line,
                    });
                }
                Some(offset) => {
                    let abs = self.pos + offset;

                    // Is this a comment?
                    if self.input[abs..].starts_with("<!--") {
                        // Flush text before the comment
                        if abs > text_start {
                            let span =
                                SourceSpan::new(text_start, abs, text_start_line, text_start_col);
                            nodes.push(HickNode::Text(
                                self.input[text_start..abs].to_string(),
                                Some(span),
                            ));
                        }
                        self.advance(offset);
                        self.skip_comment()?;
                        text_start = self.pos;
                        text_start_line = self.line;
                        text_start_col = self.col;
                        continue;
                    }

                    // Is this a closing tag?
                    if self.input[abs..].starts_with(&self.close_marker) {
                        // Flush text before this closing tag
                        if abs > text_start {
                            let span =
                                SourceSpan::new(text_start, abs, text_start_line, text_start_col);
                            nodes.push(HickNode::Text(
                                self.input[text_start..abs].to_string(),
                                Some(span),
                            ));
                        }
                        self.advance(offset);

                        // Parse closing tag name
                        let (name, _) = self.parse_close_tag()?;
                        if name == close_name {
                            return Ok(nodes);
                        } else {
                            return Err(ParseError::UnexpectedClose {
                                prefix: self.prefix.clone(),
                                name,
                                line: self.line,
                            });
                        }
                    }

                    // It's an opening tag
                    // Flush text before this tag
                    if abs > text_start {
                        let span =
                            SourceSpan::new(text_start, abs, text_start_line, text_start_col);
                        nodes.push(HickNode::Text(
                            self.input[text_start..abs].to_string(),
                            Some(span),
                        ));
                    }
                    self.advance(offset);

                    let tag = self.parse_open_tag()?;

                    if tag.self_closing {
                        nodes.push(HickNode::Tag(tag));
                    } else if is_raw_content_tag(&tag.name) {
                        // Verbatim-capture element (session vocabulary:
                        // tool payloads and tool results): content is raw
                        // text up to the next matching close marker, so
                        // captured fragments containing unbalanced
                        // hick-like markers cannot break the parse.
                        let close = format!("{}{}>", self.close_marker, tag.name);
                        let rem = self.remaining();
                        let end = rem.find(&close).ok_or_else(|| ParseError::UnclosedTag {
                            prefix: self.prefix.clone(),
                            name: tag.name.clone(),
                            line: tag.source_line,
                        })?;
                        let raw = rem[..end].to_string();
                        let span =
                            SourceSpan::new(self.pos, self.pos + raw.len(), self.line, self.col);
                        self.advance(end + close.len());
                        let mut children = Vec::new();
                        if !raw.is_empty() {
                            children.push(HickNode::Text(raw, Some(span)));
                        }
                        nodes.push(HickNode::Tag(HickTag { children, ..tag }));
                    } else {
                        // Parse nested children
                        let tag_name = tag.name.clone();
                        let tag_line = tag.source_line;
                        let children = self.parse_children(&tag_name, tag_line)?;
                        nodes.push(HickNode::Tag(HickTag { children, ..tag }));
                    }

                    text_start = self.pos;
                    text_start_line = self.line;
                    text_start_col = self.col;
                }
            }
        }
    }

    /// Parse an opening tag. Cursor must be at the opening marker.
    fn parse_open_tag(&mut self) -> Result<HickTag, ParseError> {
        let tag_line = self.line;
        let tag_col = self.col;
        let tag_start_offset = self.pos;

        // Skip the opening marker (e.g. `<hick:` or `<h:`)
        self.advance(self.open_marker.len());

        // Read tag name (until whitespace, `>`, or `/`)
        let name_start = self.pos;
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' || ch == b'>' || ch == b'/' {
                break;
            }
            self.pos += 1;
            self.col += 1;
        }
        let name = self.input[name_start..self.pos].to_string();
        if name.is_empty() {
            return Err(ParseError::Syntax {
                line: tag_line,
                message: format!("empty tag name after <{}:", self.prefix),
            });
        }

        // Parse attributes
        let mut attributes = Vec::new();
        let mut self_closing = false;

        loop {
            self.skip_ws();
            if self.is_eof() {
                return Err(ParseError::Syntax {
                    line: tag_line,
                    message: format!("unexpected EOF in <{}:{name}>", self.prefix),
                });
            }

            let ch = self.input.as_bytes()[self.pos];

            if ch == b'>' {
                self.advance(1);
                break;
            }

            if ch == b'/' {
                // Check for />
                if self.pos + 1 < self.input.len() && self.input.as_bytes()[self.pos + 1] == b'>' {
                    self_closing = true;
                    self.advance(2);
                    break;
                }
                return Err(ParseError::Syntax {
                    line: tag_line,
                    message: "unexpected '/' not followed by '>'".to_string(),
                });
            }

            // Parse attribute: name="value"
            let attr_name = self.read_attr_name(tag_line)?;
            self.skip_ws();

            if self.is_eof() || self.input.as_bytes()[self.pos] != b'=' {
                return Err(ParseError::Syntax {
                    line: self.line,
                    message: format!("expected '=' after attribute '{attr_name}'"),
                });
            }
            self.advance(1); // skip =
            self.skip_ws();

            let attr_value = self.read_attr_value(tag_line)?;
            attributes.push((attr_name, attr_value));
        }

        let tag_end_offset = self.pos;
        Ok(HickTag {
            name,
            attributes,
            children: Vec::new(),
            self_closing,
            source_line: tag_line,
            source_column: tag_col,
            source_span: Some(SourceSpan::new(
                tag_start_offset,
                tag_end_offset,
                tag_line,
                tag_col,
            )),
        })
    }

    /// Parse a closing tag. Cursor must be at the closing marker.
    fn parse_close_tag(&mut self) -> Result<(String, usize), ParseError> {
        let line = self.line;
        // Skip the closing marker (e.g. `</hick:` or `</h:`)
        self.advance(self.close_marker.len());

        let name_start = self.pos;
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b'>' || ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                break;
            }
            self.pos += 1;
        }
        let name = self.input[name_start..self.pos].to_string();

        // Update line count
        let name_slice = &self.input[name_start..self.pos];
        self.line += name_slice.chars().filter(|&c| c == '\n').count();

        self.skip_ws();
        if !self.is_eof() && self.input.as_bytes()[self.pos] == b'>' {
            self.advance(1);
        }

        Ok((name, line))
    }

    fn skip_ws(&mut self) {
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                if ch == b'\n' {
                    self.line += 1;
                }
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn read_attr_name(&mut self, _context_line: usize) -> Result<String, ParseError> {
        let start = self.pos;
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b'='
                || ch == b' '
                || ch == b'\t'
                || ch == b'\n'
                || ch == b'\r'
                || ch == b'>'
                || ch == b'/'
            {
                break;
            }
            self.pos += 1;
        }
        let name = &self.input[start..self.pos];
        if name.is_empty() {
            return Err(ParseError::Syntax {
                line: self.line,
                message: "empty attribute name".to_string(),
            });
        }
        Ok(name.to_string())
    }

    fn read_attr_value(&mut self, context_line: usize) -> Result<String, ParseError> {
        if self.is_eof() {
            return Err(ParseError::Syntax {
                line: context_line,
                message: "unexpected EOF reading attribute value".to_string(),
            });
        }

        let quote = self.input.as_bytes()[self.pos];
        if quote != b'"' && quote != b'\'' {
            return Err(ParseError::Syntax {
                line: self.line,
                message: "attribute value must be quoted".to_string(),
            });
        }
        self.advance(1); // skip opening quote

        let start = self.pos;
        while !self.is_eof() && self.input.as_bytes()[self.pos] != quote {
            if self.input.as_bytes()[self.pos] == b'\n' {
                self.line += 1;
            }
            self.pos += 1;
        }

        if self.is_eof() {
            return Err(ParseError::Syntax {
                line: context_line,
                message: "unterminated attribute value".to_string(),
            });
        }

        let value = self.input[start..self.pos].to_string();
        self.advance(1); // skip closing quote
        Ok(value)
    }
}

// ---------------------------------------------------------------------------
// Session node extraction
// ---------------------------------------------------------------------------

/// Is `name` a verbatim-capture element? See the note on `parse_children`.
fn is_raw_content_tag(name: &str) -> bool {
    matches!(name, "input" | "tool-result")
}

/// Convert raw [`HickNode`]s (from a `hick:session` root) into typed
/// [`SessionNode`]s.  Unknown tags are silently ignored.
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
                            _ => {}
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
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_document() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
</hick:doc>"#;
        let doc = parse(src).unwrap();
        assert_eq!(doc.prefix, "hick");
        assert!(doc.weave_path.is_none());
        assert!(
            doc.nodes.is_empty()
                || doc
                    .nodes
                    .iter()
                    .all(|n| matches!(n, HickNode::Text(t, _) if t.trim().is_empty()))
        );
    }

    #[test]
    fn parse_weave_attribute() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="TEMPLATE.md">
<hick:file path="out.txt">content</hick:file>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        assert_eq!(doc.weave_path, Some("TEMPLATE.md".to_string()));
    }

    #[test]
    fn parse_weave_attribute_with_path() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="docs/README.md">
</hick:doc>"#;
        let doc = parse(src).unwrap();
        assert_eq!(doc.weave_path, Some("docs/README.md".to_string()));
    }

    #[test]
    fn parse_text_content_with_special_chars() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
This has & and < and > without escaping.
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let text: String = doc
            .nodes
            .iter()
            .filter_map(|n| match n {
                HickNode::Text(t, _) => Some(t.as_str()),
                _ => None,
            })
            .collect();
        assert!(text.contains("&"));
        assert!(text.contains("<"));
        assert!(text.contains(">"));
    }

    #[test]
    fn parse_self_closing_tag() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:allow network="github.com:443" />
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let tags: Vec<_> = doc.find_tags("allow");
        assert_eq!(tags.len(), 1);
        assert!(tags[0].self_closing);
        assert_eq!(tags[0].get_attribute("network"), Some("github.com:443"));
    }

    #[test]
    fn parse_nested_tags() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:deny network="*" />
</hick:container>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let containers = doc.find_tags("container");
        assert_eq!(containers.len(), 1);
        let c = containers[0];
        assert_eq!(c.get_attribute("name"), Some("reporter"));
        assert_eq!(c.get_attribute("image"), Some("python:3.12"));

        let child_tags: Vec<_> = c.child_tags().collect();
        assert_eq!(child_tags.len(), 2);
        assert_eq!(child_tags[0].name, "allow");
        assert_eq!(child_tags[1].name, "deny");
    }

    #[test]
    fn parse_exec_with_raw_content() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
cat /etc/alpine-release
</hick:exec>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let execs = doc.find_tags("exec");
        assert_eq!(execs.len(), 1);
        assert_eq!(execs[0].get_attribute("container"), Some("demo"));
        assert_eq!(execs[0].get_attribute("image"), Some("alpine"));

        let content = execs[0].text_content();
        assert!(content.contains("cat /etc/alpine-release"));
    }

    #[test]
    fn parse_file_with_nested_exec() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="Getting Started.md">
# Getting Started

    <hick:exec container="demo" image="alpine">
    cat /etc/alpine-release
    </hick:exec>

Done!
</hick:file>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].get_attribute("path"), Some("Getting Started.md"));

        let execs: Vec<_> = files[0].child_tags().collect();
        assert_eq!(execs.len(), 1);
        assert_eq!(execs[0].name, "exec");
    }

    #[test]
    fn parse_copy_paste_cut() {
        let src = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="analyzer">
<hick:copy id="version-info">
python -c "import sys; print(sys.version)"
</hick:copy>
</hick:exec>

<hick:file path="report.md">
Python: <hick:paste select="#version-info" />
</hick:file>

<hick:cut id="internal-only">
This is removed from output.
</hick:cut>
</hick:doc>"##;
        let doc = parse(src).unwrap();

        let copies = doc.tags().flat_map(|t| {
            let mut found = Vec::new();
            find_tags_recursive(t, "copy", &mut found);
            found
        });
        assert_eq!(copies.count(), 1);

        let pastes = doc.tags().flat_map(|t| {
            let mut found = Vec::new();
            find_tags_recursive(t, "paste", &mut found);
            found
        });
        let pastes: Vec<_> = pastes.collect();
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0].get_attribute("select"), Some(r#"#version-info"#));
        assert!(pastes[0].self_closing);

        let cuts = doc.find_tags("cut");
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].get_attribute("id"), Some("internal-only"));
    }

    // -----------------------------------------------------------------------
    // Session node tests
    // -----------------------------------------------------------------------

    fn minimal_session(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-01-01T00:00:00Z">
{body}
</hick:session>"#
        )
    }

    #[test]
    fn parse_session_user_node() {
        let src = minimal_session("<hick:user>add a login page</hick:user>");
        let doc = parse_session(&src).unwrap();
        assert_eq!(doc.start_time.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(
            doc.nodes[0],
            SessionNode::User {
                text: "add a login page".into()
            }
        );
    }

    #[test]
    fn parse_session_assistant_without_action() {
        let src = minimal_session("<hick:assistant>Here is the final answer.</hick:assistant>");
        let doc = parse_session(&src).unwrap();
        assert_eq!(
            doc.nodes[0],
            SessionNode::Assistant {
                text: "Here is the final answer.".into(),
                actions: vec![],
            }
        );
    }

    #[test]
    fn parse_session_assistant_with_action() {
        let src = minimal_session(
            r#"<hick:assistant>
I will create the file now.
<hick:action lang="python">
import os
print(os.getcwd())
</hick:action>
</hick:assistant>"#,
        );
        let doc = parse_session(&src).unwrap();
        match &doc.nodes[0] {
            SessionNode::Assistant { actions, .. } => {
                assert_eq!(actions.len(), 1);
                assert_eq!(actions[0].lang, "python");
                assert!(actions[0].code.contains("os.getcwd()"));
            }
            other => panic!("expected Assistant, got {other:?}"),
        }
    }

    #[test]
    fn parse_session_observation_node() {
        let src = minimal_session(
            r#"<hick:observation source="action-0" exit="0">Files created: src/login.py</hick:observation>"#,
        );
        let doc = parse_session(&src).unwrap();
        assert_eq!(
            doc.nodes[0],
            SessionNode::Observation {
                source: Some("action-0".into()),
                exit_code: Some(0),
                text: "Files created: src/login.py".into(),
            }
        );
    }

    #[test]
    fn parse_session_command_node() {
        let src = minimal_session("<hick:command>ls -la src/</hick:command>");
        let doc = parse_session(&src).unwrap();
        assert_eq!(
            doc.nodes[0],
            SessionNode::Command {
                text: "ls -la src/".into()
            }
        );
    }

    #[test]
    fn parse_session_tool_nodes_ride_along_with_raw_payloads() {
        // The agent tool vocabulary: hick:tool/hick:arg inside an assistant
        // turn, hick:input payloads and hick:tool-result observations that
        // may quote UNBALANCED hick fragments — captured verbatim, the
        // session still parses, and unknown tags are gracefully skipped.
        let src = minimal_session(
            r#"<hick:user>fix the constant</hick:user>
<hick:assistant>
Following the pointer.
<hick:tool name="edit_doc">
<hick:arg name="run">a1b2..c3d4</hick:arg>
<hick:input>
<hick:copy id="dup">unbalanced open, no close
</hick:input>
</hick:tool>
</hick:assistant>
<hick:tool-result name="edit_doc" ok="true">
edited region:
ffff|</hick:copy>
ffff|<hick:file path="gen.rs">// slice
</hick:tool-result>
<hick:assistant>Done.</hick:assistant>"#,
        );
        let doc = parse_session(&src).expect("tool session must parse");
        // user + two assistants; tool and tool-result are inert/skipped.
        assert_eq!(doc.nodes.len(), 3);
        assert!(matches!(doc.nodes[0], SessionNode::User { .. }));
        assert!(
            matches!(&doc.nodes[1], SessionNode::Assistant { text, actions }
                if text.contains("Following the pointer.") && actions.is_empty())
        );
        assert!(matches!(&doc.nodes[2], SessionNode::Assistant { text, .. } if text == "Done."));
    }

    #[test]
    fn parse_session_full_turn_sequence() {
        let src = minimal_session(
            r#"<hick:user>add a login page</hick:user>
<hick:assistant>
I'll create the login page now.
<hick:action lang="python">
print("creating login page")
</hick:action>
</hick:assistant>
<hick:observation source="action-0" exit="0">creating login page</hick:observation>"#,
        );
        let doc = parse_session(&src).unwrap();
        assert_eq!(doc.nodes.len(), 3);
        assert!(matches!(doc.nodes[0], SessionNode::User { .. }));
        assert!(matches!(doc.nodes[1], SessionNode::Assistant { .. }));
        assert!(matches!(doc.nodes[2], SessionNode::Observation { .. }));
    }

    #[test]
    fn parse_session_command_and_observation() {
        let src = minimal_session(
            r#"<hick:command>ls -la</hick:command>
<hick:observation source="command-0" exit="0">total 8
-rw-r--r-- 1 user user 42 main.py
</hick:observation>"#,
        );
        let doc = parse_session(&src).unwrap();
        assert_eq!(doc.nodes.len(), 2);
        assert!(matches!(doc.nodes[0], SessionNode::Command { .. }));
        assert!(matches!(doc.nodes[1], SessionNode::Observation { .. }));
    }

    #[test]
    fn is_session_source_detection() {
        let session_src = minimal_session("<hick:user>hello</hick:user>");
        assert!(is_session_source(&session_src));

        let pipeline_src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="out.txt">hello</hick:file>
</hick:doc>"#;
        assert!(!is_session_source(pipeline_src));
    }

    #[test]
    fn parse_session_mixed_with_unknown_tags() {
        // Unknown tags (not user/assistant/observation/command) are silently skipped
        let src = minimal_session(
            r#"<hick:user>hello</hick:user>
<hick:metadata version="1"/>
<hick:command>echo hi</hick:command>"#,
        );
        let doc = parse_session(&src).unwrap();
        assert_eq!(doc.nodes.len(), 2); // metadata skipped
        assert!(matches!(doc.nodes[0], SessionNode::User { .. }));
        assert!(matches!(doc.nodes[1], SessionNode::Command { .. }));
    }

    #[test]
    fn pipeline_doc_with_session_like_nodes_parses_ok() {
        // A pipeline file can contain hick:user etc. without error (they're just unknown tags)
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="out.txt">hello</hick:file>
<hick:user>this is fine</hick:user>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        let users = doc.find_tags("user");
        assert_eq!(users.len(), 1);
    }

    fn find_tags_recursive<'a>(tag: &'a HickTag, name: &str, out: &mut Vec<&'a HickTag>) {
        if tag.name == name {
            out.push(tag);
        }
        for child in tag.child_tags() {
            find_tags_recursive(child, name, out);
        }
    }

    #[test]
    fn parse_full_example_untrusted_code() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
  <hick:allow file-write="/output/*" />
</hick:container>

<hick:container name="mailer" image="alpine">
  <hick:allow network="smtp.gmail.com:587" />
  <hick:deny network="*" />
  <hick:allow file-read="/output/*" />
</hick:container>

<hick:volume name="shared-output" />

<hick:exec container="reporter" mount="shared-output:/output">
pip install untrusted-package
python -c "
import untrusted_package
with open('/output/report.html', 'w') as f:
    f.write(untrusted_package.generate())
"
</hick:exec>

<hick:exec container="mailer" mount="shared-output:/output">
apk add msmtp
cat /output/report.html | msmtp bob@example.com
</hick:exec>

</hick:doc>"#;
        let doc = parse(src).unwrap();

        // Containers
        let containers = doc.find_tags("container");
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].get_attribute("name"), Some("reporter"));
        assert_eq!(containers[1].get_attribute("name"), Some("mailer"));

        // Reporter capabilities
        let reporter_rules: Vec<_> = containers[0].child_tags().collect();
        assert_eq!(reporter_rules.len(), 4);

        // Volumes
        let volumes = doc.find_tags("volume");
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].get_attribute("name"), Some("shared-output"));

        // Execs
        let execs = doc.find_tags("exec");
        assert_eq!(execs.len(), 2);
        assert_eq!(execs[0].get_attribute("container"), Some("reporter"));
        assert_eq!(execs[1].get_attribute("container"), Some("mailer"));
        assert_eq!(
            execs[0].get_attribute("mount"),
            Some("shared-output:/output")
        );
    }

    #[test]
    fn parse_attenuate_and_fork() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:attenuate container="sandbox">
  <hick:deny network="*" />
</hick:attenuate>

<hick:fork from="base" to="analyzer-a">
  <hick:deny network="*" />
</hick:fork>

<hick:confirm message="Proceed?" />
</hick:doc>"#;
        let doc = parse(src).unwrap();

        let attenuates = doc.find_tags("attenuate");
        assert_eq!(attenuates.len(), 1);
        assert_eq!(attenuates[0].get_attribute("container"), Some("sandbox"));

        let forks = doc.find_tags("fork");
        assert_eq!(forks.len(), 1);
        assert_eq!(forks[0].get_attribute("from"), Some("base"));
        assert_eq!(forks[0].get_attribute("to"), Some("analyzer-a"));

        let confirms = doc.find_tags("confirm");
        assert_eq!(confirms.len(), 1);
        assert_eq!(confirms[0].get_attribute("message"), Some("Proceed?"));
    }

    #[test]
    fn parse_secret_tag() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="deploy" image="alpine">
  <hick:secret name="PULUMI_TOKEN" from="pulumi-token" />
</hick:container>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let containers = doc.find_tags("container");
        let secrets: Vec<_> = containers[0]
            .child_tags()
            .filter(|t| t.name == "secret")
            .collect();
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].get_attribute("name"), Some("PULUMI_TOKEN"));
        assert_eq!(secrets[0].get_attribute("from"), Some("pulumi-token"));
    }

    #[test]
    fn error_on_unclosed_tag() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo">
echo hello
</hick:doc>"#;
        let err = parse(src).unwrap_err();
        assert!(matches!(err, ParseError::UnexpectedClose { .. }));
    }

    #[test]
    fn error_on_missing_root() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<html><body>Hello</body></html>"#;
        let err = parse(src).unwrap_err();
        assert!(matches!(err, ParseError::MissingRoot));
    }

    #[test]
    fn line_numbers_tracked() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<hick:exec container="demo">
echo hello
</hick:exec>

</hick:doc>"#;
        let doc = parse(src).unwrap();
        let execs = doc.find_tags("exec");
        assert_eq!(execs[0].source_line, 4);
    }

    #[test]
    fn single_quoted_attributes() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:allow network='github.com:443' />
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let allows = doc.find_tags("allow");
        assert_eq!(allows[0].get_attribute("network"), Some("github.com:443"));
    }

    #[test]
    fn multiple_execs_same_container() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
apk add curl
</hick:exec>
<hick:exec container="demo">
curl --version
</hick:exec>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let execs = doc.find_tags("exec");
        assert_eq!(execs.len(), 2);
        assert_eq!(execs[0].get_attribute("image"), Some("alpine"));
        assert!(execs[1].get_attribute("image").is_none());
    }

    // -----------------------------------------------------------------------
    // Custom prefix tests
    // -----------------------------------------------------------------------

    #[test]
    fn detect_prefix_from_xmlns() {
        assert_eq!(
            detect_prefix(r#"<h:doc xmlns:h="http://www.hickorydocs.com/1.0">"#),
            "h"
        );
        assert_eq!(
            detect_prefix(r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">"#),
            "hick"
        );
        assert_eq!(
            detect_prefix(r#"<hickory:doc xmlns:hickory="http://www.hickorydocs.com/1.0">"#),
            "hickory"
        );
        // Single-quoted
        assert_eq!(
            detect_prefix(r#"<x:doc xmlns:x='http://www.hickorydocs.com/1.0'>"#),
            "x"
        );
        // No xmlns -> fallback
        assert_eq!(detect_prefix(r#"<hick:doc>"#), "hick");
    }

    #[test]
    fn parse_with_short_prefix() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:file path="out.txt">hello world</h:file>
</h:doc>"#;
        let doc = parse(src).unwrap();
        assert_eq!(doc.prefix, "h");
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].get_attribute("path"), Some("out.txt"));
        assert_eq!(files[0].text_content(), "hello world");
    }

    #[test]
    fn custom_prefix_ignores_hick_in_text() {
        // Using prefix "h", so <hick:...> in text is NOT parsed as a tag
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:file path="guide.md">
Here is an example hick file:

<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
  <hick:file path="out.txt">hello</hick:file>
</hick:doc>

The above produces a file called out.txt.
</h:file>
</h:doc>"#;
        let doc = parse(src).unwrap();
        assert_eq!(doc.prefix, "h");

        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);

        // The <hick:...> example text should be raw text content, not parsed tags
        let content = files[0].text_content();
        assert!(
            content.contains("<hick:doc"),
            "expected literal <hick:doc in text, got: {content}"
        );
        assert!(
            content.contains("<hick:file"),
            "expected literal <hick:file in text, got: {content}"
        );
        assert!(
            content.contains("</hick:file>"),
            "expected literal </hick:file> in text, got: {content}"
        );
    }

    #[test]
    fn custom_prefix_nested_tags() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:container name="demo" image="alpine">
  <h:allow network="github.com:443" />
  <h:deny network="*" />
  <h:secret name="TOKEN" from="my-token" />
</h:container>
<h:exec container="demo">
echo hello
</h:exec>
</h:doc>"#;
        let doc = parse(src).unwrap();
        let containers = doc.find_tags("container");
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].get_attribute("name"), Some("demo"));

        let child_tags: Vec<_> = containers[0].child_tags().collect();
        assert_eq!(child_tags.len(), 3);
        assert_eq!(child_tags[0].name, "allow");
        assert_eq!(child_tags[1].name, "deny");
        assert_eq!(child_tags[2].name, "secret");

        let execs = doc.find_tags("exec");
        assert_eq!(execs.len(), 1);
    }

    #[test]
    fn custom_prefix_copy_paste() {
        let src = r##"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:copy id="ver">1.0</h:copy>
<h:file path="out.txt"><h:paste select="#ver" /></h:file>
</h:doc>"##;
        let doc = parse(src).unwrap();
        let copies = doc.find_tags("copy");
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].text_content(), "1.0");

        let files = doc.find_tags("file");
        let pastes: Vec<_> = files[0]
            .child_tags()
            .filter(|t| t.name == "paste")
            .collect();
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0].get_attribute("select"), Some("#ver"));
    }

    #[test]
    fn custom_prefix_error_on_unclosed() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:exec container="demo">
echo hello
</h:doc>"#;
        let err = parse(src).unwrap_err();
        assert!(matches!(err, ParseError::UnexpectedClose { .. }));
    }

    // Source column tracking tests
    #[test]
    fn source_column_at_line_start() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="test.txt">content</hick:file>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].source_column, 0);
    }

    #[test]
    fn source_column_indented() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
    <hick:file path="test.txt">content</hick:file>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].source_column, 4);
    }

    #[test]
    fn source_column_nested() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
    <hick:when test="true">
        <hick:file path="test.txt">content</hick:file>
    </hick:when>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        // find_tags is non-recursive, so we need to search inside the when tag
        let whens = doc.find_tags("when");
        assert_eq!(whens.len(), 1);
        let files: Vec<_> = whens[0].child_tags().filter(|t| t.name == "file").collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].source_column, 8);
    }

    // Dedent tests
    #[test]
    fn dedent_no_indent() {
        let text = "line1\nline2\nline3";
        assert_eq!(dedent(text, 4), "line1\nline2\nline3");
    }

    #[test]
    fn dedent_uniform_indent() {
        let text = "    line1\n    line2\n    line3";
        assert_eq!(dedent(text, 4), "line1\nline2\nline3");
    }

    #[test]
    fn dedent_partial_strip() {
        let text = "    line1\n        line2\n    line3";
        assert_eq!(dedent(text, 4), "line1\n    line2\nline3");
    }

    #[test]
    fn dedent_leading_newline() {
        let text = "\n    def hello():\n        print('hi')";
        assert_eq!(dedent(text, 4), "def hello():\n    print('hi')");
    }

    #[test]
    fn dedent_trailing_newline() {
        let text = "    line1\n    line2\n";
        assert_eq!(dedent(text, 4), "line1\nline2\n");
    }

    #[test]
    fn dedent_zero_indent() {
        let text = "    line1\n    line2";
        assert_eq!(dedent(text, 0), "    line1\n    line2");
    }

    #[test]
    fn dedent_more_than_available() {
        let text = "  line1\n  line2";
        assert_eq!(dedent(text, 8), "line1\nline2");
    }

    // Source span tests
    #[test]
    fn text_nodes_have_spans() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
hello world
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let text_nodes: Vec<_> = doc
            .nodes
            .iter()
            .filter_map(|n| match n {
                HickNode::Text(t, span) => Some((t.as_str(), *span)),
                _ => None,
            })
            .collect();
        assert!(!text_nodes.is_empty(), "should have at least one text node");
        let (text, span) = &text_nodes[0];
        assert!(text.contains("hello world"));
        let span = span.expect("text node should have a span");
        assert!(span.start < span.end, "span should be non-empty");
        // Verify the span references the correct bytes in the source
        assert_eq!(&src[span.start..span.end], *text);
    }

    #[test]
    fn tag_has_source_span() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="test.txt">content</hick:file>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        let span = files[0].source_span.expect("tag should have a source span");
        // The span should cover from '<' to '>' of the opening tag
        let tag_text = &src[span.start..span.end];
        assert!(
            tag_text.starts_with("<hick:file"),
            "span should start with opening tag, got: {tag_text}"
        );
        assert!(
            tag_text.ends_with(">"),
            "span should end with >, got: {tag_text}"
        );
    }

    #[test]
    fn self_closing_tag_span() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:allow network="github.com:443" />
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let allows = doc.find_tags("allow");
        let span = allows[0]
            .source_span
            .expect("self-closing tag should have span");
        let tag_text = &src[span.start..span.end];
        assert!(tag_text.starts_with("<hick:allow"));
        assert!(tag_text.ends_with("/>"));
    }

    #[test]
    fn comment_is_skipped() {
        let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<!-- this is a comment -->
<hick:file path="a.rs">hello</hick:file>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].get_attribute("path"), Some("a.rs"));
    }

    #[test]
    fn comment_out_hick_file_block() {
        let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="keep.rs">kept</hick:file>
<!--
<hick:file path="removed.rs">this should not appear</hick:file>
-->
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].get_attribute("path"), Some("keep.rs"));
    }

    #[test]
    fn multiple_comments() {
        let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<!-- first -->
<hick:file path="a.rs">a</hick:file>
<!-- second -->
<!-- third -->
<hick:file path="b.rs">b</hick:file>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn unclosed_comment_error() {
        let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<!-- unclosed comment
<hick:file path="a.rs">a</hick:file>
</hick:doc>"#;
        let err = parse(src).unwrap_err();
        assert!(
            matches!(err, ParseError::UnclosedComment { line: 2 }),
            "expected UnclosedComment at line 2, got: {err:?}"
        );
    }

    #[test]
    fn comment_preserves_surrounding_text() {
        let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
before<!-- comment -->after
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let texts: Vec<&str> = doc
            .nodes
            .iter()
            .filter_map(|n| match n {
                HickNode::Text(t, _) => Some(t.as_str()),
                _ => None,
            })
            .collect();
        let combined = texts.join("");
        assert!(
            combined.contains("before"),
            "missing 'before': {combined:?}"
        );
        assert!(combined.contains("after"), "missing 'after': {combined:?}");
        assert!(
            !combined.contains("comment"),
            "comment leaked: {combined:?}"
        );
    }
}
