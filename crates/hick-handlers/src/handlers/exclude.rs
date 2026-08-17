//! Handler for `<hick:exclude>` tags.

use anyhow::Result;
use hick_lang::HickTag;

use crate::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};

/// Handler for `<hick:exclude>` tags.
///
/// Registers a glob pattern for files to exclude from the pipeline output.
///
/// # Attributes
///
/// - `pattern` - Required glob pattern to exclude (e.g., `*.template-only`)
pub struct ExcludeHandler;

impl TagHandler for ExcludeHandler {
    fn tag_name(&self) -> &str {
        "exclude"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Declaration
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let pattern = tag_attr(tag, "pattern").unwrap_or_default();
        if !pattern.is_empty() {
            ctx.state.register_exclusion(pattern);
        }

        Ok(TagResult::Declaration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_exec::state::MultiDocumentState;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_tag(name: &str, attrs: Vec<(String, String)>) -> HickTag {
        HickTag {
            name: name.to_string(),
            attributes: attrs,
            children: vec![],
            self_closing: true,
            source_line: 1,
            source_column: 0,
            source_span: None,
        }
    }

    fn make_ctx<'a>(
        state: &'a Arc<MultiDocumentState>,
        transcripts: &'a HashMap<String, Vec<crate::TranscriptEntry>>,
    ) -> ProcessingContext<'a> {
        ProcessingContext {
            state,
            transcripts,
            indent: 0,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        }
    }

    #[test]
    fn exclude_handler_registers_pattern() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "exclude",
            vec![("pattern".to_string(), "*.template-only".to_string())],
        );

        let handler = ExcludeHandler;
        let result = handler.process(&tag, &ctx).unwrap();
        assert!(matches!(result, TagResult::Declaration));

        let patterns = state.get_exclusion_patterns();
        assert_eq!(patterns, vec!["*.template-only".to_string()]);
    }

    #[test]
    fn exclude_handler_ignores_empty_pattern() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag("exclude", vec![("pattern".to_string(), "".to_string())]);

        let handler = ExcludeHandler;
        handler.process(&tag, &ctx).unwrap();

        let patterns = state.get_exclusion_patterns();
        assert!(patterns.is_empty());
    }
}
