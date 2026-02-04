//! Handler for `<hick:val>` tags.

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::StringNode;
use hick_lang::HickTag;
use log::warn;

use crate::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};

/// Handler for `<hick:val>` tags.
///
/// Outputs the value of a variable.
///
/// # Attributes
///
/// - `name` - Required variable name to output
///
/// # Example
///
/// ```xml
/// <hick:var name="version">1.0.0</hick:var>
/// <hick:val name="version" />  <!-- outputs: 1.0.0 -->
/// ```
pub struct ValHandler;

impl TagHandler for ValHandler {
    fn name(&self) -> &str {
        "val"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Main
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let var_name = tag_attr(tag, "name").unwrap_or_default();

        if let Some(value) = ctx.state.resolve_var(&var_name) {
            Ok(TagResult::Node(Arc::new(StringNode::new(value))))
        } else {
            warn!("Variable '{}' not found", var_name);
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
    fn val_handler_resolves_variable() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_var("version".to_string(), "2.0.0".to_string());

        let transcripts = HashMap::new();
        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            parent_ip: None,
            document_name: "test.hick",
        };

        let tag = make_tag("val", vec![("name".to_string(), "version".to_string())]);

        let handler = ValHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        match result {
            TagResult::Node(node) => {
                assert_eq!(node.as_string_value(), Some("2.0.0"));
            }
            _ => panic!("Expected Node result"),
        }
    }

    #[test]
    fn val_handler_missing_returns_declaration() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            parent_ip: None,
            document_name: "test.hick",
        };

        let tag = make_tag("val", vec![("name".to_string(), "missing".to_string())]);

        let handler = ValHandler;
        let result = handler.process(&tag, &ctx).unwrap();

        assert!(matches!(result, TagResult::Declaration));
    }
}
