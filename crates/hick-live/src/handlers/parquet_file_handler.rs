//! Tag handler for `<hick:parquet-file>`.

use anyhow::Result;
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

/// Handler for `<hick:parquet-file path="..." mode="...">` tags.
///
/// Placeholder: recognises the tag so the pipeline doesn't skip it.
/// A full implementation would build a `ParquetFileNode` and wire it
/// into the pipeline output.
pub struct ParquetFileHandler;

impl TagHandler for ParquetFileHandler {
    fn tag_name(&self) -> &str {
        "parquet-file"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
        let _path = tag_attr(tag, "path").unwrap_or_default();
        let _mode = tag_attr(tag, "mode").unwrap_or_else(|| "events".into());
        log::debug!("parquet-file handler: path={}, mode={}", _path, _mode);
        Ok(TagResult::Declaration)
    }
}
