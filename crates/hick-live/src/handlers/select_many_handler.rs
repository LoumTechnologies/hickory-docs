//! Tag handler for `<hick:select-many>`.

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult};
use hick_lang::HickTag;

/// Handler for `<hick:select-many>` tags.
///
/// Placeholder: recognises the tag so the pipeline doesn't skip it.
pub struct SelectManyHandler;

impl TagHandler for SelectManyHandler {
    fn tag_name(&self) -> &str {
        "select-many"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, _tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
        log::debug!("select-many handler");
        Ok(TagResult::Declaration)
    }
}
