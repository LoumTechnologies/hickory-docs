//! Tag handler for `<hick:group-by>`.

use std::sync::Arc;

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

use crate::operators::group_by::GroupByNode;

/// Handler for `<hick:group-by columns="...">` tags.
///
/// Wraps its child source node in a `GroupByNode` that partitions rows
/// by the named group columns.
pub struct GroupByHandler;

impl TagHandler for GroupByHandler {
    fn tag_name(&self) -> &str {
        "group-by"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let columns_str = tag_attr(tag, "columns")
            .ok_or_else(|| anyhow::anyhow!("group-by requires a columns attribute"))?;

        let source = ctx.first_child_node(tag)
            .ok_or_else(|| anyhow::anyhow!("group-by requires a child source node"))?;

        let input_schema = super::infer_schema(&source);

        let group_columns: Vec<String> = columns_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let group_node = GroupByNode::new(source, group_columns, &input_schema);
        Ok(TagResult::Node(Arc::new(group_node)))
    }
}
