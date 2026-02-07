//! Tag handler for `<hick:group-by>`.

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

/// Handler for `<hick:group-by columns="...">` tags.
///
/// Placeholder: recognises the tag so the pipeline doesn't skip it.
pub struct GroupByHandler;

impl TagHandler for GroupByHandler {
    fn tag_name(&self) -> &str {
        "group-by"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
        let _columns = tag_attr(tag, "columns").unwrap_or_default();
        log::debug!("group-by handler: columns={}", _columns);
        Ok(TagResult::Declaration)
    }
}
