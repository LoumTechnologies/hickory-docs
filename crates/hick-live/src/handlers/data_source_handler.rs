//! Tag handler for `<hick:data-source>`.

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

/// Handler for `<hick:data-source type="..." ...>` tags.
///
/// Placeholder: recognises the tag so the pipeline doesn't skip it.
/// A full implementation would construct a `StreamingStructuredDataNode`
/// with the appropriate `ChangeSource` based on the `type` attribute.
pub struct DataSourceHandler;

impl TagHandler for DataSourceHandler {
    fn tag_name(&self) -> &str {
        "data-source"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
        let _source_type = tag_attr(tag, "type").unwrap_or_default();
        log::debug!("data-source handler: type={}", _source_type);
        Ok(TagResult::Declaration)
    }
}
