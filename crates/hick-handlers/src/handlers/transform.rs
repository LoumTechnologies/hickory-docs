//! Handler for `<hick:transform>` tags.
//!
//! A transform is a passage an LLM wrote FROM another fragment, under an
//! instruction — the bridge between two representations of one fact that a
//! `hick:paste` cannot cross, because a paste copies bytes verbatim and a
//! sentence is not a JSON literal.
//!
//! Weaving one is deliberately dumb: the pinned body is emitted verbatim and
//! no model is called. The passage lives inline in the document precisely so
//! that it is reviewable in a diff, editable by hand, and free to render in
//! CI. Regeneration is an explicit `hickory refresh`; staleness is checked by
//! fingerprint (see `hick_lang::transform_fingerprint`).

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::StringNode;
use hick_lang::HickTag;

use crate::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};

/// Handler for `<hick:transform>` tags.
///
/// # Attributes
///
/// - `select` — selector for the input fragment(s) the passage was written from
/// - `instruct` — the instruction it was written under
/// - `from` — fingerprint of (input bytes + instruction) at the time it was written
pub struct TransformHandler;

impl TagHandler for TransformHandler {
    fn tag_name(&self) -> &str {
        "transform"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        // The pinned passage IS the output. A transform that regenerated here
        // would call a model during `hickory test`, making verification cost
        // money and return different bytes every run.
        let body = hick_lang::dedent(&hick_lang::tag_text(tag), ctx.indent);
        let _ = tag_attr(tag, "select");
        Ok(TagResult::Node(Arc::new(StringNode::new(
            body.trim_start_matches('\n').to_string(),
        ))))
    }
}
