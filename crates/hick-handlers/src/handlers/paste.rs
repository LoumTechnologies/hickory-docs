//! Handler for `<hick:paste>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::StringNode;
use hick_lang::{HickTag, dedent};
use log::warn;

use crate::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};

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

        if let Some(content) = ctx.state.resolve_paste(&selector) {
            let dedented = dedent(&content, ctx.indent);
            Ok(TagResult::Node(Arc::new(StringNode::new(dedented))))
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
        }
    }

    fn make_ctx<'a>(state: &'a Arc<MultiDocumentState>, transcripts: &'a HashMap<String, Vec<crate::TranscriptEntry>>) -> ProcessingContext<'a> {
        ProcessingContext {
            state,
            transcripts,
            indent: 0,
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

        let tag = make_tag("paste", vec![("select".to_string(), "#missing".to_string())]);

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        assert!(matches!(result, TagResult::Declaration));
    }
}
