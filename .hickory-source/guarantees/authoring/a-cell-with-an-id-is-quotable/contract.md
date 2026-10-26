<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Cell With An Id Is Quotable, And The Quote Keeps The Cell's Provenance

For: Authors editing executable documents and the files those documents produce.

Given `<hick:exec id="p95" …>` that shows output, when another place in the
document says `<hick:paste select="#p95" />` — before or after the cell, in
prose or in a generated file — then what the cell SHOWS (its transcript,
rendered per `show=`, without the trailing line break) is pasted there, and
the pasted bytes carry the cell's **exec** origin, so `hick lineage` and the
app's ribbons end at the computation rather than one hop short of it.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-cell-with-an-id-is-quotable.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-addcddbb639c"></a>
## `crates/hick-handlers/src/handlers/exec.rs`

Related contracts: [A Cell With An Id Is Quotable, And The Quote Keeps The Cell's Provenance](contract.md).

<a id="f-addcddbb639c-ExecNode"></a>
### `ExecNode`

<storycode:copy id="f-addcddbb639c-ExecNode">
//! Handler for `<hick:exec>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::{BoxStream, Context, Node, NodeTrace, NodeValue, SourceOrigin, StringNode};
use hick_lang::HickTag;

use crate::{
    ExecShow, ProcessingContext, ProcessingPhase, TagHandler, TagResult, parse_exec_show,
    render_transcript, tag_attr,
};

// ---------------------------------------------------------------------------
// ExecNode — provenance wrapper
// ---------------------------------------------------------------------------

/// Provenance wrapper around `StringNode` for exec transcript output.
///
/// Carries the container name, enabling trace-based identification of
/// which container produced the output.
pub(crate) struct ExecNode {
    inner: StringNode,
    container: String,
    origin: SourceOrigin,
}

</storycode:copy>
<a id="f-addcddbb639c-impl-ExecNode"></a>
### `impl-ExecNode`

<storycode:copy id="f-addcddbb639c-impl-ExecNode">
impl ExecNode {
    pub fn new(content: impl Into<String>, container: impl Into<String>, tag_line: usize) -> Self {
        let c: String = container.into();
        Self {
            inner: StringNode::new(content),
            origin: SourceOrigin::Exec {
                container: Arc::from(c.as_str()),
                tag_line,
            },
            container: c,
        }
    }
}

</storycode:copy>
<a id="f-addcddbb639c-quotable_exec_node"></a>
### `quotable_exec_node`

The cell's shown output as a quotable fragment: what `show=` renders,
minus the trailing line break a transcript ends with — a paste of `#cell`
into a sentence wants the number, not the number and a newline. Carries
the cell's exec provenance, so the paste's ribbon ends at the computation.
`None` when the cell shows nothing (`show="none"`) or has no transcript.

<storycode:copy id="f-addcddbb639c-quotable_exec_node">
/// The cell's shown output as a quotable fragment: what `show=` renders,
/// minus the trailing line break a transcript ends with — a paste of `#cell`
/// into a sentence wants the number, not the number and a newline. Carries
/// the cell's exec provenance, so the paste's ribbon ends at the computation.
/// `None` when the cell shows nothing (`show="none"`) or has no transcript.
pub fn quotable_exec_node(
    tag: &HickTag,
    ctx: &ProcessingContext,
) -> Option<(Arc<dyn Node>, String)> {
    let container_name = tag_attr(tag, "container").unwrap_or_default();
    let show = parse_exec_show(tag);
    if show == ExecShow::None {
        return None;
    }
    let entries = ctx.transcripts.get(&container_name)?;
    let own: Vec<crate::TranscriptEntry> = entries
        .iter()
        .filter(|e| e.source_line == Some(tag.source_line))
        .cloned()
        .collect();
    let rendered = if own.is_empty() {
        render_transcript(entries, show)
    } else {
        render_transcript(&own, show)
    };
    let text = rendered.trim_end_matches(['\n', '\r']).to_string();
    let node: Arc<dyn Node> = Arc::new(ExecNode::new(
        text.clone(),
        &container_name,
        tag.source_line,
    ));
    Some((node, text))
}

</storycode:copy>
<a id="f-addcddbb639c-impl-std-fmt-Debug-for-ExecNode"></a>
### `impl-std::fmt::Debug for ExecNode`

<storycode:copy id="f-addcddbb639c-impl-std-fmt-Debug-for-ExecNode">
impl std::fmt::Debug for ExecNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExecNode(container={:?})", self.container)
    }
}

</storycode:copy>
<a id="f-addcddbb639c-impl-Node-for-ExecNode"></a>
### `impl-Node for ExecNode`

<storycode:copy id="f-addcddbb639c-impl-Node-for-ExecNode">
impl Node for ExecNode {
    fn as_string_value(&self) -> Option<&str> {
        self.inner.as_string_value()
    }

    fn node_value(&self) -> NodeValue {
        self.inner.node_value()
    }

    fn source_origin(&self) -> Option<&SourceOrigin> {
        Some(&self.origin)
    }

    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let trace = vec![NodeTrace::new(self as Arc<dyn Node>)];
        Box::pin(futures::stream::once(async move { trace }))
    }
}

</storycode:copy>
<a id="f-addcddbb639c-ExecHandler"></a>
### `ExecHandler`

ExecHandler
Handler for `<hick:exec>` tags.
Renders execution transcript output for a container into file content.
# Attributes
- `container` - Required container name to pull transcript from
- `show` - Optional: "all" (default), "command", "output", or "none"

<storycode:copy id="f-addcddbb639c-ExecHandler">
// ---------------------------------------------------------------------------
// ExecHandler
// ---------------------------------------------------------------------------

/// Handler for `<hick:exec>` tags.
///
/// Renders execution transcript output for a container into file content.
///
/// # Attributes
///
/// - `container` - Required container name to pull transcript from
/// - `show` - Optional: "all" (default), "command", "output", or "none"
pub struct ExecHandler;

</storycode:copy>
<a id="f-addcddbb639c-impl-TagHandler-for-ExecHandler"></a>
### `impl-TagHandler for ExecHandler`

<storycode:copy id="f-addcddbb639c-impl-TagHandler-for-ExecHandler">
impl TagHandler for ExecHandler {
    fn tag_name(&self) -> &str {
        "exec"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let container_name = tag_attr(tag, "container").unwrap_or_default();
        let show = parse_exec_show(tag);

        if show == ExecShow::None {
            return Ok(TagResult::Declaration);
        }

        if let Some(entries) = ctx.transcripts.get(&container_name) {
            // When entries carry source-line provenance, render only the
            // entries belonging to THIS exec tag; otherwise fall back to the
            // container's whole transcript (legacy behavior). An exec with no
            // command of its own (a self-closing reference exec) also renders
            // the whole transcript.
            let own: Vec<crate::TranscriptEntry> = entries
                .iter()
                .filter(|e| e.source_line == Some(tag.source_line))
                .cloned()
                .collect();
            let own_is_reference = !own.is_empty()
                && own
                    .iter()
                    .all(|e| e.commands.iter().all(|c| c.is_empty()) && e.output.is_empty());
            let rendered = if own.is_empty() || own_is_reference {
                render_transcript(entries, show)
            } else {
                render_transcript(&own, show)
            };
            Ok(TagResult::Node(Arc::new(ExecNode::new(
                rendered,
                &container_name,
                tag.source_line,
            ))))
        } else {
            Ok(TagResult::Node(Arc::new(ExecNode::new(
                format!("[no transcript for container '{container_name}']"),
                &container_name,
                tag.source_line,
            ))))
        }
    }
}

</storycode:copy>
<a id="f-addcddbb639c-tests"></a>
### `tests`

<storycode:copy id="f-addcddbb639c-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use crate::TranscriptEntry;
    use hick_exec::state::MultiDocumentState;
    use std::collections::HashMap;

    fn make_tag(name: &str, attrs: Vec<(String, String)>) -> HickTag {
        HickTag {
            name: name.to_string(),
            attributes: attrs,
            children: vec![],
            self_closing: true,
            source_line: 1,
            source_column: 0,
            source_span: None,
            close_span: None,
        }
    }

    #[test]
    fn exec_handler_renders_transcript() {
        let state = Arc::new(MultiDocumentState::default());
        let mut transcripts = HashMap::new();
        transcripts.insert(
            "builder".to_string(),
            vec![TranscriptEntry {
                commands: vec!["echo hello".to_string()],
                output: "hello".to_string(),
                source_line: None,
            }],
        );

        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            paste_line_indent: None,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        };

        let tag = make_tag(
            "exec",
            vec![("container".to_string(), "builder".to_string())],
        );

        let handler = ExecHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("$ echo hello\nhello\n"));
            }
            _ => panic!("Expected Node result"),
        }
    }

    #[test]
    fn exec_handler_show_none_returns_declaration() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();

        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            paste_line_indent: None,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        };

        let tag = make_tag(
            "exec",
            vec![
                ("container".to_string(), "builder".to_string()),
                ("show".to_string(), "none".to_string()),
            ],
        );

        let handler = ExecHandler;
        let result = handler.process(&tag, &ctx).unwrap();
        assert!(matches!(result, TagResult::Declaration));
    }

    #[test]
    fn exec_handler_missing_container() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();

        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            paste_line_indent: None,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        };

        let tag = make_tag(
            "exec",
            vec![("container".to_string(), "missing".to_string())],
        );

        let handler = ExecHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(
                    node.as_string_value(),
                    Some("[no transcript for container 'missing']")
                );
            }
            _ => panic!("Expected Node result"),
        }
    }

    #[test]
    fn exec_handler_show_command_only() {
        let state = Arc::new(MultiDocumentState::default());
        let mut transcripts = HashMap::new();
        transcripts.insert(
            "builder".to_string(),
            vec![TranscriptEntry {
                commands: vec!["ls".to_string()],
                output: "file.txt".to_string(),
                source_line: None,
            }],
        );

        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            paste_line_indent: None,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        };

        let tag = make_tag(
            "exec",
            vec![
                ("container".to_string(), "builder".to_string()),
                ("show".to_string(), "command".to_string()),
            ],
        );

        let handler = ExecHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("$ ls\n"));
            }
            _ => panic!("Expected Node result"),
        }
    }
}
</storycode:copy>

### Assemble `crates/hick-handlers/src/handlers/exec.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-handlers/src/handlers/exec.rs"><storycode:paste select="#f-addcddbb639c-ExecNode"/><storycode:paste select="#f-addcddbb639c-impl-ExecNode"/><storycode:paste select="#f-addcddbb639c-quotable_exec_node"/><storycode:paste select="#f-addcddbb639c-impl-std-fmt-Debug-for-ExecNode"/><storycode:paste select="#f-addcddbb639c-impl-Node-for-ExecNode"/><storycode:paste select="#f-addcddbb639c-ExecHandler"/><storycode:paste select="#f-addcddbb639c-impl-TagHandler-for-ExecHandler"/><storycode:paste select="#f-addcddbb639c-tests"/></storycode:file>

<a id="output-28b43931eb63"></a>
## `crates/hick-handlers/src/handlers/paste.rs`

Related contracts: [A Cell With An Id Is Quotable, And The Quote Keeps The Cell's Provenance](contract.md).

<a id="f-28b43931eb63-PasteNode"></a>
### `PasteNode`

<storycode:copy id="f-28b43931eb63-PasteNode">
//! Handler for `<hick:paste>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::{BoxStream, Context, Node, NodeTrace, NodeValue, SourceOrigin, StringNode};
use hick_lang::{HickTag, dedent};
use log::warn;

use crate::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, has_flag, tag_attr};

// ---------------------------------------------------------------------------
// PasteNode — provenance wrapper
// ---------------------------------------------------------------------------

/// Provenance wrapper around `StringNode` for paste output.
///
/// Carries the selector used to resolve the paste, enabling trace-based
/// identification of where content originated.
pub(crate) struct PasteNode {
    inner: StringNode,
    selector: String,
    origin: SourceOrigin,
}

</storycode:copy>
<a id="f-28b43931eb63-impl-PasteNode"></a>
### `impl-PasteNode`

<storycode:copy id="f-28b43931eb63-impl-PasteNode">
impl PasteNode {
    pub fn new(content: impl Into<String>, selector: impl Into<String>) -> Self {
        Self::with_source(content, selector, None)
    }

    /// Construct with an optional byte-precise source location: the file and
    /// span of the copy block whose bytes this paste reproduces verbatim.
    pub fn with_source(
        content: impl Into<String>,
        selector: impl Into<String>,
        source: Option<(Arc<str>, hick_lang::SourceSpan)>,
    ) -> Self {
        let sel: String = selector.into();
        let (file, span) = match source {
            Some((f, s)) => (Some(f), Some(s)),
            None => (None, None),
        };
        Self {
            inner: StringNode::new(content),
            origin: SourceOrigin::Paste {
                selector: Arc::from(sel.as_str()),
                file,
                span,
            },
            selector: sel,
        }
    }
}

</storycode:copy>
<a id="f-28b43931eb63-impl-std-fmt-Debug-for-PasteNode"></a>
### `impl-std::fmt::Debug for PasteNode`

<storycode:copy id="f-28b43931eb63-impl-std-fmt-Debug-for-PasteNode">
impl std::fmt::Debug for PasteNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PasteNode(select={:?})", self.selector)
    }
}

</storycode:copy>
<a id="f-28b43931eb63-impl-Node-for-PasteNode"></a>
### `impl-Node for PasteNode`

<storycode:copy id="f-28b43931eb63-impl-Node-for-PasteNode">
impl Node for PasteNode {
    fn as_string_value(&self) -> Option<&str> {
        self.inner.as_string_value()
    }

    fn node_value(&self) -> NodeValue {
        self.inner.node_value()
    }

    fn source_origin(&self) -> Option<&SourceOrigin> {
        Some(&self.origin)
    }

    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let trace = vec![NodeTrace::new(self as Arc<dyn Node>)];
        Box::pin(futures::stream::once(async move { trace }))
    }
}

</storycode:copy>
<a id="f-28b43931eb63-PasteHandler"></a>
### `PasteHandler`

PasteHandler
Handler for `<hick:paste>` tags.
Pastes content from previously registered copy/cut blocks.
Supports both ID-based (`#id`) and class-based (`.class`) selectors.
# Attributes
- `select` - Required CSS-like selector:
- `#id` - Paste content from block with matching ID
- `.class` - Concatenate content from all blocks with matching class (in document order)
- `separator` - Optional string inserted between blocks for class selectors
- `min` - Optional minimum number of matching blocks (error if fewer)
- `max` - Optional maximum number of matching blocks (error if more)

<storycode:copy id="f-28b43931eb63-PasteHandler">
// ---------------------------------------------------------------------------
// PasteHandler
// ---------------------------------------------------------------------------

/// Handler for `<hick:paste>` tags.
///
/// Pastes content from previously registered copy/cut blocks.
/// Supports both ID-based (`#id`) and class-based (`.class`) selectors.
///
/// # Attributes
///
/// - `select` - Required CSS-like selector:
///   - `#id` - Paste content from block with matching ID
///   - `.class` - Concatenate content from all blocks with matching class (in document order)
/// - `separator` - Optional string inserted between blocks for class selectors
/// - `min` - Optional minimum number of matching blocks (error if fewer)
/// - `max` - Optional maximum number of matching blocks (error if more)
pub struct PasteHandler;

</storycode:copy>
<a id="f-28b43931eb63-align_block_paste"></a>
### `align_block_paste`

Place a block paste on the indentation of its otherwise-empty host line.
Copy blocks conventionally start on the line after their opening tag. That
opening newline is syntax for the copy, not an empty line in the file that
receives an aligned paste. Subsequent nonblank lines get the same prefix;
a final newline does not, because the following source node owns its line.

<storycode:copy id="f-28b43931eb63-align_block_paste">
/// Place a block paste on the indentation of its otherwise-empty host line.
///
/// Copy blocks conventionally start on the line after their opening tag. That
/// opening newline is syntax for the copy, not an empty line in the file that
/// receives an aligned paste. Subsequent nonblank lines get the same prefix;
/// a final newline does not, because the following source node owns its line.
fn align_block_paste(content: &str, prefix: &str) -> String {
    let content = content
        .strip_prefix("\r\n")
        .or_else(|| content.strip_prefix('\n'))
        .unwrap_or(content);
    let mut out = String::with_capacity(content.len() + prefix.len());
    let mut first = true;
    for line in content.split_inclusive('\n') {
        if !first && line != "\n" && line != "\r\n" {
            out.push_str(prefix);
        }
        out.push_str(line);
        first = false;
    }
    out
}

</storycode:copy>
<a id="f-28b43931eb63-impl-TagHandler-for-PasteHandler"></a>
### `impl-TagHandler for PasteHandler`

<storycode:copy id="f-28b43931eb63-impl-TagHandler-for-PasteHandler">
impl TagHandler for PasteHandler {
    fn tag_name(&self) -> &str {
        "paste"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let selector = tag_attr(tag, "select").unwrap_or_default();
        let separator = tag_attr(tag, "separator");
        let min: Option<usize> = tag_attr(tag, "min").and_then(|v| v.parse().ok());
        let max: Option<usize> = tag_attr(tag, "max").and_then(|v| v.parse().ok());
        // Dedup is the COLLECTOR's policy, not the contributor's: several
        // documents independently asking for `bin/` is the normal shape of a
        // shared file, and none of them should have to know about the others.
        let distinct = has_flag(tag, "distinct");

        // Validate min/max constraints. Recorded on the state as well as
        // returned: the caller that renders a `<hick:file>` body logs a
        // handler error and carries on, which used to weave an EMPTY file and
        // exit 0 — the exact silence `resolve_paste_node` was fixed for.
        let count = ctx.state.count_paste_matches(&selector, distinct);
        if let Some(min_val) = min
            && count < min_val
        {
            let msg = format!(
                "paste selector '{selector}' matched {count} block(s), but min={min_val} is \
                 required.\n  Nothing carries that id or class, or the document that does is not \
                 part of this run.\n  Check the spelling of the selector, and that the \
                 contributing document is public (a `<hick:private />` document's fragments are \
                 its own)."
            );
            ctx.state.record_paste_failure(msg.clone());
            anyhow::bail!("{msg}");
        }
        if let Some(max_val) = max
            && count > max_val
        {
            let msg = format!(
                "paste selector '{selector}' matched {count} block(s), but max={max_val} is \
                 allowed.\n  Narrow the selector, or add `distinct` if the extra matches are \
                 repeats of the same text."
            );
            ctx.state.record_paste_failure(msg.clone());
            anyhow::bail!("{msg}");
        }

        let sep = separator.as_deref();

        // Try node-based resolution first (supports reactive streaming)
        if let Some(node) = ctx.state.resolve_paste_node(&selector, sep, distinct) {
            // Direct string values are wrapped in a PasteNode (dedented as
            // needed). When the pasted bytes are byte-identical to the copy
            // block's source bytes, propagate the source span so output edits
            // can be mapped back to the copy block.
            if let Some(s) = node.as_string_value() {
                let dedented = dedent(s, ctx.indent);
                let dedented = ctx
                    .paste_line_indent
                    .as_deref()
                    .map_or(dedented.clone(), |prefix| {
                        align_block_paste(&dedented, prefix)
                    });
                let source = match node.source_origin() {
                    Some(SourceOrigin::Literal { file, span }) if dedented == s => {
                        Some((file.clone(), *span))
                    }
                    // A quoted CELL (`#cell-id`) keeps the cell's own origin:
                    // the pasted number is the computation's, and the ribbon
                    // should end there rather than go synthetic.
                    Some(SourceOrigin::Exec { .. }) if dedented == s => {
                        return Ok(TagResult::Node(node));
                    }
                    _ => None,
                };
                return Ok(TagResult::Node(Arc::new(PasteNode::with_source(
                    dedented, &selector, source,
                ))));
            }
            return Ok(TagResult::Node(node));
        }

        // Fallback to string-based resolution (backward compat)
        if let Some(content) = ctx.state.resolve_paste(&selector, sep, distinct) {
            let dedented = dedent(&content, ctx.indent);
            let dedented = ctx
                .paste_line_indent
                .as_deref()
                .map_or(dedented.clone(), |prefix| {
                    align_block_paste(&dedented, prefix)
                });
            Ok(TagResult::Node(Arc::new(PasteNode::new(
                dedented, &selector,
            ))))
        } else {
            warn!("Paste selector '{}' not found", selector);
            Ok(TagResult::Declaration)
        }
    }
}

</storycode:copy>
<a id="f-28b43931eb63-tests"></a>
### `tests`

<storycode:copy id="f-28b43931eb63-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use hick_exec::state::MultiDocumentState;
    use std::collections::HashMap;

    fn make_tag(name: &str, attrs: Vec<(String, String)>) -> HickTag {
        HickTag {
            name: name.to_string(),
            attributes: attrs,
            children: vec![],
            self_closing: true,
            source_line: 1,
            source_column: 0,
            source_span: None,
            close_span: None,
        }
    }

    fn make_ctx<'a>(
        state: &'a Arc<MultiDocumentState>,
        transcripts: &'a HashMap<String, Vec<crate::TranscriptEntry>>,
    ) -> ProcessingContext<'a> {
        ProcessingContext {
            state,
            transcripts,
            indent: 0,
            paste_line_indent: None,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        }
    }

    #[test]
    fn paste_handler_resolves_id() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_copy("ver".to_string(), "1.0.0".to_string());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag("paste", vec![("select".to_string(), "#ver".to_string())]);

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("1.0.0"));
            }
            _ => panic!("Expected Node result"),
        }
    }

    #[test]
    fn paste_handler_resolves_class() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_copy_with_class("".to_string(), Some("items"), "a;".to_string());
        state.register_copy_with_class("".to_string(), Some("items"), "b;".to_string());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag("paste", vec![("select".to_string(), ".items".to_string())]);

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("a;b;"));
            }
            _ => panic!("Expected Node result"),
        }
    }

    #[test]
    fn paste_handler_missing_returns_declaration() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "paste",
            vec![("select".to_string(), "#missing".to_string())],
        );

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        assert!(matches!(result, TagResult::Declaration));
    }

    // -----------------------------------------------------------------------
    // Separator tests
    // -----------------------------------------------------------------------

    #[test]
    fn paste_handler_separator() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_copy_with_class("".to_string(), Some("items"), "a".to_string());
        state.register_copy_with_class("".to_string(), Some("items"), "b".to_string());
        state.register_copy_with_class("".to_string(), Some("items"), "c".to_string());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "paste",
            vec![
                ("select".to_string(), ".items".to_string()),
                ("separator".to_string(), ", ".to_string()),
            ],
        );

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("a, b, c"));
            }
            _ => panic!("Expected Node result"),
        }
    }

    #[test]
    fn paste_handler_separator_newline() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_copy_with_class("".to_string(), Some("deps"), "serde".to_string());
        state.register_copy_with_class("".to_string(), Some("deps"), "tokio".to_string());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "paste",
            vec![
                ("select".to_string(), ".deps".to_string()),
                ("separator".to_string(), "\n".to_string()),
            ],
        );

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("serde\ntokio"));
            }
            _ => panic!("Expected Node result"),
        }
    }

    // -----------------------------------------------------------------------
    // Min/max tests
    // -----------------------------------------------------------------------

    #[test]
    fn paste_handler_min_satisfied() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_copy_with_class("".to_string(), Some("routes"), "GET /".to_string());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "paste",
            vec![
                ("select".to_string(), ".routes".to_string()),
                ("min".to_string(), "1".to_string()),
            ],
        );

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn paste_handler_min_violated() {
        let state = Arc::new(MultiDocumentState::default());
        // No blocks registered for .routes

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "paste",
            vec![
                ("select".to_string(), ".routes".to_string()),
                ("min".to_string(), "1".to_string()),
            ],
        );

        let handler = PasteHandler;
        match handler.process(&tag, &ctx) {
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("min=1"), "Error should mention min: {msg}");
            }
            Ok(_) => panic!("Expected error with min=1 and 0 blocks"),
        }
    }

    #[test]
    fn paste_handler_max_satisfied() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_copy("cfg".to_string(), "value".to_string());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "paste",
            vec![
                ("select".to_string(), "#cfg".to_string()),
                ("min".to_string(), "1".to_string()),
                ("max".to_string(), "1".to_string()),
            ],
        );

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn paste_handler_max_violated() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_copy_with_class("".to_string(), Some("plugins"), "a".to_string());
        state.register_copy_with_class("".to_string(), Some("plugins"), "b".to_string());
        state.register_copy_with_class("".to_string(), Some("plugins"), "c".to_string());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "paste",
            vec![
                ("select".to_string(), ".plugins".to_string()),
                ("max".to_string(), "2".to_string()),
            ],
        );

        let handler = PasteHandler;
        match handler.process(&tag, &ctx) {
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("max=2"), "Error should mention max: {msg}");
            }
            Ok(_) => panic!("Expected error with max=2 and 3 blocks"),
        }
    }

    #[test]
    fn paste_handler_min_zero_allows_missing() {
        let state = Arc::new(MultiDocumentState::default());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        // min=0 explicitly allows no matches (same as no min)
        let tag = make_tag(
            "paste",
            vec![
                ("select".to_string(), ".optional".to_string()),
                ("min".to_string(), "0".to_string()),
            ],
        );

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn paste_handler_id_min_required() {
        let state = Arc::new(MultiDocumentState::default());
        // No block registered for #db-config

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "paste",
            vec![
                ("select".to_string(), "#db-config".to_string()),
                ("min".to_string(), "1".to_string()),
            ],
        );

        let handler = PasteHandler;
        assert!(
            handler.process(&tag, &ctx).is_err(),
            "Expected error with min=1 and missing #db-config",
        );
    }
}
</storycode:copy>

### Assemble `crates/hick-handlers/src/handlers/paste.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-handlers/src/handlers/paste.rs"><storycode:paste select="#f-28b43931eb63-PasteNode"/><storycode:paste select="#f-28b43931eb63-impl-PasteNode"/><storycode:paste select="#f-28b43931eb63-impl-std-fmt-Debug-for-PasteNode"/><storycode:paste select="#f-28b43931eb63-impl-Node-for-PasteNode"/><storycode:paste select="#f-28b43931eb63-PasteHandler"/><storycode:paste select="#f-28b43931eb63-align_block_paste"/><storycode:paste select="#f-28b43931eb63-impl-TagHandler-for-PasteHandler"/><storycode:paste select="#f-28b43931eb63-tests"/></storycode:file>

<a id="output-8105fecec152"></a>
## `crates/hickory-cli/tests/exec_paste.rs`

Related contracts: [A Cell With An Id Is Quotable, And The Quote Keeps The Cell's Provenance](contract.md).

<a id="f-8105fecec152-hick"></a>
### `hick`

A cell with an id is quotable: `<hick:paste select="#cell"/>` pastes what
the cell shows, with the cell's exec provenance — so a finding can carry
the number a cell printed instead of retyping it. And a copy inside a
claim still registers, so a claim can wrap the fragment it asserts.

<storycode:copy id="f-8105fecec152-hick">
//! A cell with an id is quotable: `<hick:paste select="#cell"/>` pastes what
//! the cell shows, with the cell's exec provenance — so a finding can carry
//! the number a cell printed instead of retyping it. And a copy inside a
//! claim still registers, so a claim can wrap the fragment it asserts.
//! Guarantee: docs/guarantees/authoring/a-cell-with-an-id-is-quotable.md

fn hick() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-8105fecec152-a_cells_output_pastes_by_id_and_a_copy_inside_a_claim_registers"></a>
### `a_cells_output_pastes_by_id_and_a_copy_inside_a_claim_registers`

<storycode:copy id="f-8105fecec152-a_cells_output_pastes_by_id_and_a_copy_inside_a_claim_registers">
#[test]
fn a_cells_output_pastes_by_id_and_a_copy_inside_a_claim_registers() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("n.md");
    std::fs::write(
        &doc,
        r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="n.md">
<hick:container name="sh" image="host" />

Before the cell: <hick:paste select="#p95" />

<hick:exec id="p95" container="sh" show="output">printf '212 ms'</hick:exec>

<hick:claim by="nate" standing="judgment"><hick:copy id="j" class="message">Fine for now.</hick:copy></hick:claim>

<hick:file path="out.txt">p95 is <hick:paste select="#p95" />; <hick:paste select="#j" /></hick:file>
</hick:doc>
"##,
    )
    .unwrap();
    let out = hick()
        .env("HICKORY_EXECUTOR", "local")
        .args(["run", doc.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let file = std::fs::read_to_string(dir.path().join("out.txt")).unwrap();
    assert_eq!(file.trim(), "p95 is 212 ms; Fine for now.");
    let woven = std::fs::read_to_string(dir.path().join("n.md")).unwrap();
    assert!(
        woven.contains("Before the cell: 212 ms"),
        "a paste before the cell resolves too: {woven}"
    );

    // The pasted bytes carry the cell's provenance, not a literal span.
    let lineage = hick()
        .env("HICKORY_EXECUTOR", "local")
        .args([
            "lineage",
            doc.to_str().unwrap(),
            "--output",
            "out.txt",
            "--json",
        ])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&lineage.stdout).unwrap();
    let kinds: Vec<String> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["origin"]["kind"].as_str().unwrap_or("?").to_string())
        .collect();
    assert!(kinds.iter().any(|k| k == "exec"), "{kinds:?}");
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/exec_paste.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/exec_paste.rs"><storycode:paste select="#f-8105fecec152-hick"/><storycode:paste select="#f-8105fecec152-a_cells_output_pastes_by_id_and_a_copy_inside_a_claim_registers"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-handlers/src/handlers/exec.rs](contract.md#output-addcddbb639c)
- [crates/hick-handlers/src/handlers/paste.rs](contract.md#output-28b43931eb63)
- [crates/hick-literate/src/lib.rs](../../../mechanisms/crates/hick-literate/src/lib.rs.md#output-e099e80d2a99)

### Tests and fixtures

- [crates/hickory-cli/tests/exec_paste.rs](contract.md#output-8105fecec152)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
