//! Tag handler for `<hick:select>`.

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

/// Handler for `<hick:select columns="...">` tags.
///
/// Placeholder: recognises the tag so the pipeline doesn't skip it.
/// A full implementation would build a `SelectNode` from the column list.
pub struct SelectHandler;

impl TagHandler for SelectHandler {
    fn tag_name(&self) -> &str {
        "select"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
        let _columns = tag_attr(tag, "columns").unwrap_or_default();
        log::debug!("select handler: columns={}", _columns);
        Ok(TagResult::Declaration)
    }
}
