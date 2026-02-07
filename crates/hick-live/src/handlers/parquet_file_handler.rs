//! Tag handler for `<hick:parquet-file>`.

use std::sync::Arc;

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

use crate::output::parquet::{ParquetFileNode, ParquetMode};

/// Handler for `<hick:parquet-file path="..." mode="...">` tags.
///
/// Wraps its child source node in a `ParquetFileNode` that writes
/// change events to a Parquet file.
pub struct ParquetFileHandler;

impl TagHandler for ParquetFileHandler {
    fn tag_name(&self) -> &str {
        "parquet-file"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let path = tag_attr(tag, "path")
            .ok_or_else(|| anyhow::anyhow!("parquet-file requires a path attribute"))?;
        let mode = match tag_attr(tag, "mode").as_deref() {
            Some("materialized") => ParquetMode::Materialized,
            _ => ParquetMode::Events,
        };

        let source = ctx.first_child_node(tag)
            .ok_or_else(|| anyhow::anyhow!("parquet-file requires a child source node"))?;

        let schema = super::infer_schema(&source);

        let parquet_node = ParquetFileNode::new(path, source, schema, mode);
        Ok(TagResult::Node(Arc::new(parquet_node)))
    }
}
