//! Handler for `<hick:substitute>` tags.

use anyhow::Result;
use hick_exec::state::SubstitutionDef;
use hick_lang::HickTag;

use crate::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};

/// Handler for `<hick:substitute>` tags.
///
/// Registers a text substitution pattern that will be applied to all file
/// output content.
///
/// # Attributes
///
/// - `name` - Substitution name (also used as fallback variable lookup for value)
/// - `pattern` - The text pattern to search for
/// - `value` - The replacement value (falls back to resolving `name` as a variable)
/// - `variants` - When "true", generate all case variants of the pattern/value
pub struct SubstituteHandler;

impl TagHandler for SubstituteHandler {
    fn tag_name(&self) -> &str {
        "substitute"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Declaration
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let name = tag_attr(tag, "name").unwrap_or_default();
        let pattern = tag_attr(tag, "pattern").unwrap_or_default();
        let value = tag_attr(tag, "value")
            .or_else(|| ctx.state.resolve_var(&name))
            .unwrap_or_default();
        let variants = tag_attr(tag, "variants")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        ctx.state.register_substitution(
            name,
            SubstitutionDef {
                pattern,
                value,
                variants,
            },
        );

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
            close_span: None,
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
            paste_line_indent: None,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        }
    }

    #[test]
    fn substitute_handler_registers_substitution() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "substitute",
            vec![
                ("name".to_string(), "project".to_string()),
                ("pattern".to_string(), "MyTemplate".to_string()),
                ("value".to_string(), "MyApp".to_string()),
            ],
        );

        let handler = SubstituteHandler;
        let result = handler.process(&tag, &ctx).unwrap();
        assert!(matches!(result, TagResult::Declaration));

        let subs = state.get_substitutions();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].pattern, "MyTemplate");
        assert_eq!(subs[0].value, "MyApp");
    }

    #[test]
    fn substitute_handler_falls_back_to_var() {
        let state = Arc::new(MultiDocumentState::default());
        state.register_var("project_name".to_string(), "FallbackApp".to_string());

        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = make_tag(
            "substitute",
            vec![
                ("name".to_string(), "project_name".to_string()),
                ("pattern".to_string(), "MyTemplate".to_string()),
            ],
        );

        let handler = SubstituteHandler;
        handler.process(&tag, &ctx).unwrap();

        let subs = state.get_substitutions();
        assert_eq!(subs[0].value, "FallbackApp");
    }
}
