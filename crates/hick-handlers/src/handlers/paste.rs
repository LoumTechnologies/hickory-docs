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

impl std::fmt::Debug for PasteNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PasteNode(select={:?})", self.selector)
    }
}

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
