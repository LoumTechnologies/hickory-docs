//! Tag handler for `<hick:select>`.

use std::sync::Arc;

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

use crate::operators::select::SelectNode;

/// Handler for `<hick:select columns="...">` tags.
///
/// Wraps its child source node in a `SelectNode` that projects only
/// the named columns.
pub struct SelectHandler;

impl TagHandler for SelectHandler {
    fn tag_name(&self) -> &str {
        "select"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let columns_str = tag_attr(tag, "columns")
            .ok_or_else(|| anyhow::anyhow!("select requires a columns attribute"))?;

        let source = ctx
            .first_child_node(tag)
            .ok_or_else(|| anyhow::anyhow!("select requires a child source node"))?;

        let input_schema = super::infer_schema(&source);

        let columns: Vec<&str> = columns_str
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let select_node = SelectNode::columns(source, &columns, &input_schema);
        Ok(TagResult::Node(Arc::new(select_node)))
    }
}
