//! Tag handler for `<hick:select-many>`.

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult};
use hick_lang::HickTag;

/// Handler for `<hick:select-many>` tags.
///
/// Placeholder: `SelectManyNode` requires a programmatic `InnerFn` closure
/// that cannot be expressed via XML attributes alone. This handler is
/// registered so the pipeline recognises the tag. A real implementation
/// would require an expression language or a companion config mechanism.
pub struct SelectManyHandler;

impl TagHandler for SelectManyHandler {
    fn tag_name(&self) -> &str {
        "select-many"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, _tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
        log::debug!("select-many handler: no XML-level implementation yet");
        Ok(TagResult::Declaration)
    }
}
