//! Handler for `<hick:paste>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::StringNode;
use hick_lang::HickTag;
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
///
/// # Example
///
/// ```xml
/// <hick:paste select="#version" />
/// <hick:paste select=".imports" />
/// ```
pub struct PasteHandler;

impl TagHandler for PasteHandler {
    fn name(&self) -> &str {
        "paste"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Main
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let selector = tag_attr(tag, "select").unwrap_or_default();

        if let Some(content) = ctx.state.resolve_paste(&selector) {
            Ok(TagResult::Node(Arc::new(StringNode::new(content))))
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

    #[test]
    fn paste_handler_resolves_id() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_copy("ver".to_string(), "1.0.0".to_string());

        let transcripts = HashMap::new();
        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            parent_ip: None,
            document_name: "test.hick",
        };

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
        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            parent_ip: None,
            document_name: "test.hick",
        };

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
        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            parent_ip: None,
            document_name: "test.hick",
        };

        let tag = make_tag("paste", vec![("select".to_string(), "#missing".to_string())]);

        let handler = PasteHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        assert!(matches!(result, TagResult::Declaration));
    }
}
