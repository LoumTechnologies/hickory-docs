//! Handler for `<hick:copy>` and `<hick:cut>` tags.

use anyhow::Result;
use hick_lang::HickTag;

use crate::{
    ProcessingContext, ProcessingPhase, TagHandler, TagResult, collect_text_children, tag_attr,
};

/// Handler for `<hick:copy>` tags.
///
/// Registers content blocks that can be pasted later via `<hick:paste>`.
/// Supports both ID-based (`#id`) and class-based (`.class`) selectors.
///
/// # Attributes
///
/// - `id` - Optional unique identifier for `#id` selector
/// - `class` - Optional space-separated class names for `.class` selector
///
/// # Example
///
/// ```xml
/// <hick:copy id="version">1.0.0</hick:copy>
/// <hick:copy class="imports">import foo;</hick:copy>
/// <hick:copy id="header" class="sections">...</hick:copy>
/// ```
pub struct CopyHandler;

impl TagHandler for CopyHandler {
    fn name(&self) -> &str {
        "copy"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Declaration
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let id = tag_attr(tag, "id").unwrap_or_default();
        let class = tag_attr(tag, "class");
        let content = collect_text_children(tag);

        ctx.state.register_copy_with_class(id, class.as_deref(), content);

        Ok(TagResult::Declaration)
    }
}

/// Handler for `<hick:cut>` tags.
///
/// Similar to copy, but conceptually marks content as "consumed" (though it's
/// still available via paste).
pub struct CutHandler;

impl TagHandler for CutHandler {
    fn name(&self) -> &str {
        "cut"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Declaration
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let id = tag_attr(tag, "id").unwrap_or_default();
        let class = tag_attr(tag, "class");
        let content = collect_text_children(tag);

        ctx.state.register_cut_with_class(id, class.as_deref(), content);

        Ok(TagResult::Declaration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_exec::state::MultiDocumentState;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_tag(name: &str, attrs: Vec<(String, String)>, children: Vec<hick_lang::HickNode>) -> HickTag {
        HickTag {
            name: name.to_string(),
            attributes: attrs,
            children,
            self_closing: false,
            source_line: 1,
        }
    }

    #[test]
    fn copy_handler_registers_content() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            parent_ip: None,
            document_name: "test.hick",
        };

        let tag = make_tag(
            "copy",
            vec![("id".to_string(), "ver".to_string())],
            vec![hick_lang::HickNode::Text("1.0.0".to_string())],
        );

        let handler = CopyHandler;
        let result = handler.process(&tag, &ctx).unwrap();
        assert!(matches!(result, TagResult::Declaration));

        assert_eq!(state.resolve_paste("#ver"), Some("1.0.0".to_string()));
    }

    #[test]
    fn copy_handler_with_class() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            parent_ip: None,
            document_name: "test.hick",
        };

        let tag = make_tag(
            "copy",
            vec![("class".to_string(), "imports".to_string())],
            vec![hick_lang::HickNode::Text("import foo;".to_string())],
        );

        let handler = CopyHandler;
        handler.process(&tag, &ctx).unwrap();

        assert_eq!(state.resolve_paste(".imports"), Some("import foo;".to_string()));
    }
}
