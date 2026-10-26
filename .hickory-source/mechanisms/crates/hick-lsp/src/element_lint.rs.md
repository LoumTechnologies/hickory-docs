<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/element_lint.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5c82cb63a986"></a>
## `crates/hick-lsp/src/element_lint.rs`

<a id="f-5c82cb63a986-element_diagnostics"></a>
### `element_diagnostics`

Element-vocabulary diagnostics shared with `hick run` and `hick lint`.

<storycode:copy id="f-5c82cb63a986-element_diagnostics">
//! Element-vocabulary diagnostics shared with `hick run` and `hick lint`.

use hick_lang::HickDocument;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::structural::byte_to_position;

/// Report attributes a known hick element does not declare.
pub fn element_diagnostics(source: &str, doc: &HickDocument) -> Vec<Diagnostic> {
    hick_blocks::attribute_errors(source, doc)
        .into_iter()
        .map(|error| {
            let (sl, sc) = byte_to_position(source, error.span.0);
            let (el, ec) = byte_to_position(source, error.span.1);
            Diagnostic {
                range: Range {
                    start: Position {
                        line: sl,
                        character: sc,
                    },
                    end: Position {
                        line: el,
                        character: ec,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("hick-elements".to_string()),
                message: error.to_string(),
                ..Default::default()
            }
        })
        .collect()
}

</storycode:copy>
<a id="f-5c82cb63a986-tests"></a>
### `tests`

<storycode:copy id="f-5c82cb63a986-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_only_the_unknown_attribute() {
        let source = "<hick:paste from=\"#test1\" />";
        let doc = hick_lang::parse(source).unwrap();
        let diagnostics = element_diagnostics(source, &doc);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .message
                .contains("does not accept attribute `from`")
        );
        assert_eq!(diagnostics[0].range.start.character, 12);
        assert_eq!(diagnostics[0].range.end.character, 16);
    }
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/element_lint.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/element_lint.rs"><storycode:paste select="#f-5c82cb63a986-element_diagnostics"/><storycode:paste select="#f-5c82cb63a986-tests"/></storycode:file>

</storycode:doc>
