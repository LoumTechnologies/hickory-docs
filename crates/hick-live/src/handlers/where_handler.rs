//! Tag handler for `<hick:where>`.

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

/// Handler for `<hick:where expr="...">` tags.
///
/// Placeholder: the `expr` attribute is currently ignored. A real
/// implementation would parse it into an Arrow compute predicate
/// (e.g. via DataFusion expressions). For now this handler is
/// registered so the pipeline recognises the tag.
pub struct WhereHandler;

impl TagHandler for WhereHandler {
    fn tag_name(&self) -> &str {
        "where"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
        let _expr = tag_attr(tag, "expr").unwrap_or_default();
        log::debug!("where handler: expr={}", _expr);
        Ok(TagResult::Declaration)
    }
}
