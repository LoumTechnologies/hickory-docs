//! Tag handler for `<hick:subscribe>`.

use std::sync::Arc;

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::{HickNode, HickTag};
use hick_sink::{SinkDef, SinkSlot};

use crate::operators::subscribe::{SlotBinding, SubscribeNode};

/// Handler for `<hick:subscribe>` tags.
///
/// Dispatches source data rows to an API sink backend with slot bindings.
///
/// Example:
/// ```xml
/// <hick:subscribe sink="email">
///   <hick:slot name="to" column="recipient"/>
///   <hick:slot name="body" column="report_body"/>
///   <hick:data-source .../>
/// </hick:subscribe>
/// ```
pub struct SubscribeHandler;

impl TagHandler for SubscribeHandler {
    fn tag_name(&self) -> &str {
        "subscribe"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let sink_type = tag_attr(tag, "sink")
            .ok_or_else(|| anyhow::anyhow!("subscribe requires a 'sink' attribute"))?;

        let mut slot_bindings = Vec::new();
        let mut child_source = None;

        for child in &tag.children {
            if let HickNode::Tag(child_tag) = child {
                if child_tag.name == "slot" {
                    let slot_name = tag_attr(child_tag, "name")
                        .ok_or_else(|| anyhow::anyhow!("slot requires a 'name' attribute"))?;
                    let column = tag_attr(child_tag, "column")
                        .ok_or_else(|| anyhow::anyhow!("slot requires a 'column' attribute"))?;
                    slot_bindings.push(SlotBinding { slot_name, column });
                } else if let Some(registry) = ctx.registry
                    && let Some(handler) = registry.find(&child_tag.name)
                {
                    if let Ok(TagResult::Node(node)) = handler.process(child_tag, ctx) {
                        child_source = Some(node);
                    }
                }
            }
        }

        let source = child_source.ok_or_else(|| {
            anyhow::anyhow!("subscribe requires a child source (e.g. data-source)")
        })?;

        if slot_bindings.is_empty() {
            anyhow::bail!("subscribe requires at least one <slot> binding");
        }

        let input_schema = super::infer_schema(&source);

        // Build a minimal SinkDef from slot bindings — full definitions come
        // from external sink-defs config in the plan-runner.
        let sink_def = SinkDef {
            sink_type: sink_type.clone(),
            slots: slot_bindings
                .iter()
                .map(|b| SinkSlot {
                    name: b.slot_name.clone(),
                    accepted_classifications: vec![],
                    rejected_classifications: vec![],
                })
                .collect(),
            required_capabilities: vec![],
        };

        let subscribe =
            SubscribeNode::new(source, sink_type, sink_def, slot_bindings, &input_schema);
        Ok(TagResult::Node(Arc::new(subscribe)))
    }
}
