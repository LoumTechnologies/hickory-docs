<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lsp/src/element_lint.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ca901d1f089e38cda9044f0b3c257a66f5c5ee5184efd13a6c41ba7565c5791e`.

<book:entry path="crates/hick-lsp/src/element_lint.rs">
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
</book:entry>

</book:doc>
