//! Handler for `<hick:copy>` and `<hick:cut>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::{InsertionPoint, Node, SourceOrigin, SpanNode, StringNode};
use hick_lang::{HickNode, HickTag};

use crate::{
    ProcessingContext, ProcessingPhase, TagHandler, TagResult, collect_text_children, tag_attr,
};

/// Resolve children as both a node and a string.
///
/// Returns `(node, string_fallback)` — the node for reactive storage,
/// and the string for backward-compat string-based storage.
fn resolve_content_node(tag: &HickTag, ctx: &ProcessingContext) -> (Arc<dyn Node>, String) {
    let text = collect_text_children(tag);
    let has_child_tags = tag.children.iter().any(|c| matches!(c, HickNode::Tag(_)));

    if !text.trim().is_empty() || !has_child_tags {
        // Plain-text content: when the tag holds exactly one text child with a
        // known source span (byte-identical to the source bytes — the parser's
        // no-escaping invariant), carry that span as a Literal origin so
        // pasted output can be traced (and edited) back to this copy block.
        if let [HickNode::Text(t, Some(span))] = tag.children.as_slice()
            && let Some(source_file) = ctx.file_of_span(span)
        {
            let origin = SourceOrigin::Literal {
                file: source_file,
                span: *span,
            };
            let node = Arc::new(SpanNode::new(t.clone(), origin));
            return (node as Arc<dyn Node>, text);
        }
        let node = Arc::new(StringNode::new(text.clone()));
        return (node as Arc<dyn Node>, text);
    }

    // Evaluate child tags, collecting both nodes and string parts
    let ip = Arc::new(InsertionPoint::new());
    let mut string_parts = Vec::new();
    for child in &tag.children {
        match child {
            HickNode::Text(t, _) => {
                if !t.is_empty() {
                    ip.add(Arc::new(StringNode::new(t.clone())));
                    string_parts.push(t.clone());
                }
            }
            HickNode::Tag(child_tag) => {
                if let Some(node) = ctx.process_child(child_tag) {
                    if let Some(s) = node.as_string_value() {
                        string_parts.push(s.to_string());
                    }
                    ip.add(node);
                }
            }
        }
    }
    ip.close();
    (ip as Arc<dyn Node>, string_parts.join(""))
}

/// Handler for `<hick:copy>` tags.
///
/// Registers content blocks that can be pasted later via `<hick:paste>`.
/// Supports both ID-based (`#id`) and class-based (`.class`) selectors.
///
/// # Attributes
///
/// - `id` - Optional unique identifier for `#id` selector
/// - `class` - Optional space-separated class names for `.class` selector
pub struct CopyHandler;

impl TagHandler for CopyHandler {
    fn tag_name(&self) -> &str {
        "copy"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Declaration
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let id = tag_attr(tag, "id").unwrap_or_default();
        let class = tag_attr(tag, "class");

        let (node, string_fallback) = resolve_content_node(tag, ctx);
        ctx.state
            .register_copy_node(id, class.as_deref(), node, string_fallback);

        Ok(TagResult::Declaration)
    }
}

/// Handler for `<hick:cut>` tags.
///
/// Similar to copy, but conceptually marks content as "consumed" (though it's
/// still available via paste).
pub struct CutHandler;

impl TagHandler for CutHandler {
    fn tag_name(&self) -> &str {
        "cut"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Declaration
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let id = tag_attr(tag, "id").unwrap_or_default();
        let class = tag_attr(tag, "class");

        let (node, string_fallback) = resolve_content_node(tag, ctx);
        ctx.state
            .register_cut_node(id, class.as_deref(), node, string_fallback);

        Ok(TagResult::Declaration)
    }
}

/// Handler for `<hick:transcript>`: makes a meeting quotable.
///
/// A transcript's turns are derived from its raw bytes before handlers run
/// (`hick_transcript::expand`), each a `<hick:said>` with an id
/// (`#transcript-u7`) and classes (`.said`, `.said-sam`). This registers every
/// turn — and the transcript as a whole — as a pasteable fragment, so a note
/// downstream of a meeting can quote one sentence of it by reference and the
/// paste carries that sentence's span in the meeting file. Without this the
/// turns were selectable by `hick:transform` and by nothing else.
pub struct TranscriptHandler;

impl TagHandler for TranscriptHandler {
    fn tag_name(&self) -> &str {
        "transcript"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Declaration
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let mut whole = Vec::new();
        for child in &tag.children {
            let HickNode::Tag(turn) = child else { continue };
            if turn.name != "said" {
                continue;
            }
            let id = tag_attr(turn, "id").unwrap_or_default();
            let class = tag_attr(turn, "class");
            let (node, text) = resolve_content_node(turn, ctx);
            whole.push(text.clone());
            ctx.state
                .register_copy_node(id, class.as_deref(), node, text);
        }
        // The transcript itself, by its id: the turns' text, one per line —
        // what a summary or a whole-meeting quote wants, without the cue
        // timings of the raw block.
        if let Some(id) = tag_attr(tag, "id")
            && !whole.is_empty()
        {
            let text = whole.join("\n");
            let node = Arc::new(StringNode::new(text.clone()));
            ctx.state
                .register_copy_node(id, tag_attr(tag, "class").as_deref(), node, text);
        }
        Ok(TagResult::Declaration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::{ExecHandler, ValHandler};
    use crate::{TagRegistry, TranscriptEntry};
    use hick_exec::state::MultiDocumentState;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_tag(
        name: &str,
        attrs: Vec<(String, String)>,
        children: Vec<hick_lang::HickNode>,
    ) -> HickTag {
        HickTag {
            name: name.to_string(),
            attributes: attrs,
            children,
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
        }
    }

    fn make_ctx<'a>(
        state: &'a Arc<MultiDocumentState>,
        transcripts: &'a HashMap<String, Vec<TranscriptEntry>>,
    ) -> ProcessingContext<'a> {
        ProcessingContext {
            state,
            transcripts,
            indent: 0,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        }
    }

    fn make_ctx_with_registry<'a>(
        state: &'a Arc<MultiDocumentState>,
        transcripts: &'a HashMap<String, Vec<TranscriptEntry>>,
        registry: &'a TagRegistry,
    ) -> ProcessingContext<'a> {
        ProcessingContext {
            state,
            transcripts,
            indent: 0,
            registry: Some(registry),
            context: None,
            source_file: None,
            span_files: &[],
        }
    }

    #[test]
    fn copy_handler_registers_content() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "copy",
            vec![("id".to_string(), "ver".to_string())],
            vec![hick_lang::HickNode::Text("1.0.0".to_string(), None)],
        );

        let handler = CopyHandler;
        let result = handler.process(&tag, &ctx).unwrap();
        assert!(matches!(result, TagResult::Declaration));

        assert_eq!(state.resolve_paste("#ver", None), Some("1.0.0".to_string()));
    }

    #[test]
    fn copy_handler_with_class() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "copy",
            vec![("class".to_string(), "imports".to_string())],
            vec![hick_lang::HickNode::Text("import foo;".to_string(), None)],
        );

        let handler = CopyHandler;
        handler.process(&tag, &ctx).unwrap();

        assert_eq!(
            state.resolve_paste(".imports", None),
            Some("import foo;".to_string())
        );
    }

    #[test]
    fn copy_with_exec_child() {
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

        let mut registry = TagRegistry::new();
        registry.register(Box::new(ExecHandler));

        let ctx = make_ctx_with_registry(&state, &transcripts, &registry);

        // <hick:copy id="setup-output">
        //   <hick:exec container="builder" show="output" />
        // </hick:copy>
        let exec_tag = HickTag {
            name: "exec".to_string(),
            attributes: vec![
                ("container".to_string(), "builder".to_string()),
                ("show".to_string(), "output".to_string()),
            ],
            children: vec![],
            self_closing: true,
            source_line: 2,
            source_column: 0,
            source_span: None,
        };

        let tag = make_tag(
            "copy",
            vec![("id".to_string(), "setup-output".to_string())],
            vec![HickNode::Tag(exec_tag)],
        );

        let handler = CopyHandler;
        handler.process(&tag, &ctx).unwrap();

        assert_eq!(
            state.resolve_paste("#setup-output", None),
            Some("hello\n".to_string()),
        );
    }

    #[test]
    fn copy_with_val_child() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_var("version".to_string(), "3.0.0".to_string());

        let transcripts = HashMap::new();
        let mut registry = TagRegistry::new();
        registry.register(Box::new(ValHandler));

        let ctx = make_ctx_with_registry(&state, &transcripts, &registry);

        // <hick:copy id="ver">
        //   <hick:val name="version" />
        // </hick:copy>
        let val_tag = HickTag {
            name: "val".to_string(),
            attributes: vec![("name".to_string(), "version".to_string())],
            children: vec![],
            self_closing: true,
            source_line: 2,
            source_column: 0,
            source_span: None,
        };

        let tag = make_tag(
            "copy",
            vec![("id".to_string(), "ver".to_string())],
            vec![HickNode::Tag(val_tag)],
        );

        let handler = CopyHandler;
        handler.process(&tag, &ctx).unwrap();

        assert_eq!(state.resolve_paste("#ver", None), Some("3.0.0".to_string()));
    }

    #[test]
    fn copy_with_mixed_text_and_tag_children() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_var("name".to_string(), "World".to_string());

        let transcripts = HashMap::new();
        let mut registry = TagRegistry::new();
        registry.register(Box::new(ValHandler));

        let ctx = make_ctx_with_registry(&state, &transcripts, &registry);

        // Text-only content takes the fast path even with a registry present
        let tag = make_tag(
            "copy",
            vec![("id".to_string(), "greeting".to_string())],
            vec![hick_lang::HickNode::Text("Hello literal".to_string(), None)],
        );

        let handler = CopyHandler;
        handler.process(&tag, &ctx).unwrap();

        assert_eq!(
            state.resolve_paste("#greeting", None),
            Some("Hello literal".to_string())
        );
    }

    #[test]
    fn cut_with_exec_child() {
        let state = Arc::new(MultiDocumentState::default());
        let mut transcripts = HashMap::new();
        transcripts.insert(
            "runner".to_string(),
            vec![TranscriptEntry {
                commands: vec!["date".to_string()],
                output: "2024-01-01".to_string(),
                source_line: None,
            }],
        );

        let mut registry = TagRegistry::new();
        registry.register(Box::new(ExecHandler));

        let ctx = make_ctx_with_registry(&state, &transcripts, &registry);

        let exec_tag = HickTag {
            name: "exec".to_string(),
            attributes: vec![
                ("container".to_string(), "runner".to_string()),
                ("show".to_string(), "output".to_string()),
            ],
            children: vec![],
            self_closing: true,
            source_line: 2,
            source_column: 0,
            source_span: None,
        };

        let tag = make_tag(
            "cut",
            vec![("id".to_string(), "date-output".to_string())],
            vec![HickNode::Tag(exec_tag)],
        );

        let handler = CutHandler;
        handler.process(&tag, &ctx).unwrap();

        assert_eq!(
            state.resolve_paste("#date-output", None),
            Some("2024-01-01\n".to_string()),
        );
    }
}
