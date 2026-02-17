//! Tag handler for `<hick:mask>`.

use std::sync::Arc;

use anyhow::{Result, bail};
use hick_classify::{Classification, MaskOperation, MaskSpec};
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::{HickNode, HickTag};

use crate::operators::mask::MaskNode;

/// Handler for `<hick:mask>` tags.
///
/// Wraps a child source node with masking operations on specified columns.
///
/// Example:
/// ```xml
/// <hick:mask>
///   <hick:column name="ssn" operation="hash" salt="project-salt"
///     target-classification='[{"sensitivity":"pseudonymized"}]'/>
///   <hick:column name="salary" operation="bucket" bucket-size="10000"/>
///   <hick:data-source .../>
/// </hick:mask>
/// ```
pub struct MaskHandler;

impl TagHandler for MaskHandler {
    fn tag_name(&self) -> &str {
        "mask"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let mut mask_specs = Vec::new();
        let mut child_source = None;

        for child in &tag.children {
            if let HickNode::Tag(child_tag) = child {
                if child_tag.name == "column" {
                    mask_specs.push(parse_mask_column(child_tag)?);
                } else if let Some(registry) = ctx.registry
                    && let Some(handler) = registry.find(&child_tag.name)
                {
                    if let Ok(TagResult::Node(node)) = handler.process(child_tag, ctx) {
                        child_source = Some(node);
                    }
                }
            }
        }

        let source = child_source
            .ok_or_else(|| anyhow::anyhow!("mask requires a child source (e.g. data-source)"))?;

        if mask_specs.is_empty() {
            bail!("mask requires at least one <column> with an operation");
        }

        let input_schema = super::infer_schema(&source);
        let mask_node = MaskNode::new(source, mask_specs, &input_schema);
        Ok(TagResult::Node(Arc::new(mask_node)))
    }
}

fn parse_mask_column(tag: &HickTag) -> Result<MaskSpec> {
    let name = tag_attr(tag, "name")
        .ok_or_else(|| anyhow::anyhow!("mask column requires a name attribute"))?;
    let op_str = tag_attr(tag, "operation")
        .ok_or_else(|| anyhow::anyhow!("mask column requires an operation attribute"))?;

    let operation = match op_str.as_str() {
        "mask" => MaskOperation::Mask,
        "redact" => MaskOperation::Redact,
        "hash" => MaskOperation::Hash,
        "bucket" => MaskOperation::Bucket,
        other => bail!("unknown mask operation: {other}"),
    };

    let mut spec = MaskSpec {
        column: name,
        operation,
        target_classification: None,
        mask_value: tag_attr(tag, "mask-value"),
        bucket_size: tag_attr(tag, "bucket-size").and_then(|s| s.parse::<f64>().ok()),
        hash_salt: tag_attr(tag, "salt"),
    };

    if let Some(cls_str) = tag_attr(tag, "target-classification") {
        spec.target_classification = Some(
            Classification::from_json_string(&cls_str)
                .map_err(|e| anyhow::anyhow!("invalid target-classification: {e}"))?,
        );
    }

    Ok(spec)
}
