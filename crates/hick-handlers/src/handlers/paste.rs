//! Handler for `<hick:paste>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::{BoxStream, Context, Node, NodeTrace, NodeValue, SourceOrigin, StringNode};
use hick_lang::{HickTag, dedent};
use log::warn;

use crate::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};

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

        // Validate min/max constraints
        let count = ctx.state.count_paste_matches(&selector);
        if let Some(min_val) = min
            && count < min_val
        {
            anyhow::bail!(
                "Paste selector '{}' matched {} block(s), but min={} required",
                selector,
                count,
                min_val,
            );
        }
        if let Some(max_val) = max
            && count > max_val
        {
            anyhow::bail!(
                "Paste selector '{}' matched {} block(s), but max={} allowed",
                selector,
                count,
                max_val,
            );
        }

        let sep = separator.as_deref();

        // Try node-based resolution first (supports reactive streaming)
        if let Some(node) = ctx.state.resolve_paste_node(&selector, sep) {
            // Direct string values are wrapped in a PasteNode (dedented as
            // needed). When the pasted bytes are byte-identical to the copy
            // block's source bytes, propagate the source span so output edits
            // can be mapped back to the copy block.
            if let Some(s) = node.as_string_value() {
                let dedented = dedent(s, ctx.indent);
                let source = match node.source_origin() {
                    Some(SourceOrigin::Literal { file, span }) if dedented == s => {
                        Some((file.clone(), *span))
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
        if let Some(content) = ctx.state.resolve_paste(&selector, sep) {
            let dedented = dedent(&content, ctx.indent);
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
            registry: None,
            context: None,
            source_file: None,
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
