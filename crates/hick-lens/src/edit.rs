//! Source edit computation: given a selector match and new content, compute
//! which `.hick` source byte ranges to edit.

use std::sync::Arc;

use hick_flow::node::SourceOrigin;
use hick_flow::provenance::ProvenanceMap;
use hick_lang::SourceSpan;

use crate::mapping::AstElement;

/// Confidence level of a computed source edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditConfidence {
    /// The provenance maps directly to a `.hick` literal span — exact byte
    /// range replacement is possible.
    Exact,
    /// The element spans multiple provenance origins or includes synthesized
    /// content. The edit targets the enclosing hick element and may need
    /// manual review.
    Approximate,
    /// No provenance information is available for this element. The edit
    /// cannot be mapped back to source.
    Unmapped,
}

/// A computed edit to apply to a `.hick` source file.
#[derive(Debug, Clone)]
pub struct SourceEdit {
    /// Path to the `.hick` source file.
    pub file: Arc<str>,
    /// Byte range in the `.hick` source to replace (start inclusive, end exclusive).
    pub source_start: usize,
    pub source_end: usize,
    /// The new content to insert at this position.
    pub new_content: String,
    /// Confidence level of this edit.
    pub confidence: EditConfidence,
    /// The original source span that was matched.
    pub source_span: Option<SourceSpan>,
}

/// Compute source edits for replacing an AST element's content.
///
/// Given an element found via selector query and the provenance map, determines
/// which `.hick` source byte ranges need to be modified.
pub fn compute_edits(
    element: &AstElement,
    new_content: &str,
    provenance: &ProvenanceMap,
) -> Vec<SourceEdit> {
    let overlapping = provenance.spans_in_range(element.output_start, element.output_end);

    if overlapping.is_empty() {
        return vec![SourceEdit {
            file: Arc::from(""),
            source_start: 0,
            source_end: 0,
            new_content: new_content.to_string(),
            confidence: EditConfidence::Unmapped,
            source_span: None,
        }];
    }

    // Check if all overlapping spans come from a single Literal origin in the same file
    let literal_spans: Vec<_> = overlapping
        .iter()
        .filter_map(|s| match &s.origin {
            SourceOrigin::Literal { file, span } => Some((file.clone(), *span)),
            _ => None,
        })
        .collect();

    if literal_spans.len() == overlapping.len() && !literal_spans.is_empty() {
        // All spans are Literal — check if they're from the same file
        let first_file = &literal_spans[0].0;
        if literal_spans
            .iter()
            .all(|(f, _)| f.as_ref() == first_file.as_ref())
        {
            // Compute the enclosing source range
            let source_start = literal_spans.iter().map(|(_, s)| s.start).min().unwrap();
            let source_end = literal_spans.iter().map(|(_, s)| s.end).max().unwrap();

            let confidence = if literal_spans.len() == 1 {
                EditConfidence::Exact
            } else {
                // Multiple literal spans from the same file — still good but approximate
                EditConfidence::Exact
            };

            return vec![SourceEdit {
                file: first_file.clone(),
                source_start,
                source_end,
                new_content: new_content.to_string(),
                confidence,
                source_span: Some(SourceSpan::new(
                    source_start,
                    source_end,
                    literal_spans[0].1.start_line,
                    literal_spans[0].1.start_col,
                )),
            }];
        }
    }

    // Mixed origins — return approximate edits grouped by file
    let mut edits = Vec::new();
    let mut seen_files: std::collections::HashMap<Arc<str>, (usize, usize, SourceSpan)> =
        std::collections::HashMap::new();

    for span in &overlapping {
        if let SourceOrigin::Literal {
            file,
            span: src_span,
        } = &span.origin
        {
            let entry =
                seen_files
                    .entry(file.clone())
                    .or_insert((src_span.start, src_span.end, *src_span));
            entry.0 = entry.0.min(src_span.start);
            entry.1 = entry.1.max(src_span.end);
        }
    }

    if seen_files.is_empty() {
        // No literal origins at all (all synthetic, exec, paste, etc.)
        return vec![SourceEdit {
            file: Arc::from(""),
            source_start: 0,
            source_end: 0,
            new_content: new_content.to_string(),
            confidence: EditConfidence::Unmapped,
            source_span: None,
        }];
    }

    for (file, (start, end, span)) in seen_files {
        edits.push(SourceEdit {
            file,
            source_start: start,
            source_end: end,
            new_content: new_content.to_string(),
            confidence: EditConfidence::Approximate,
            source_span: Some(span),
        });
    }

    edits
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_flow::provenance::{ProvenanceMap, ProvenanceSpan};

    #[test]
    fn exact_edit_single_literal_span() {
        let element = AstElement {
            kind: crate::mapping::AstElementKind::Function,
            name: "greet".to_string(),
            output_start: 0,
            output_end: 14,
            start_line: 0,
            end_line: 0,
        };

        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 14,
            origin: SourceOrigin::Literal {
                file: Arc::from("app.hick"),
                span: SourceSpan::new(100, 114, 10, 4),
            },
        });

        let edits = compute_edits(&element, "fn greet_v2() {}", &prov);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].confidence, EditConfidence::Exact);
        assert_eq!(&*edits[0].file, "app.hick");
        assert_eq!(edits[0].source_start, 100);
        assert_eq!(edits[0].source_end, 114);
        assert_eq!(edits[0].new_content, "fn greet_v2() {}");
    }

    #[test]
    fn unmapped_when_no_provenance() {
        let element = AstElement {
            kind: crate::mapping::AstElementKind::Function,
            name: "greet".to_string(),
            output_start: 0,
            output_end: 14,
            start_line: 0,
            end_line: 0,
        };

        let prov = ProvenanceMap::new();
        let edits = compute_edits(&element, "fn greet_v2() {}", &prov);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].confidence, EditConfidence::Unmapped);
    }

    #[test]
    fn approximate_edit_mixed_origins() {
        let element = AstElement {
            kind: crate::mapping::AstElementKind::Function,
            name: "greet".to_string(),
            output_start: 0,
            output_end: 30,
            start_line: 0,
            end_line: 2,
        };

        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 15,
            origin: SourceOrigin::Literal {
                file: Arc::from("a.hick"),
                span: SourceSpan::new(10, 25, 2, 0),
            },
        });
        prov.push(ProvenanceSpan {
            output_start: 15,
            output_end: 30,
            origin: SourceOrigin::Paste {
                selector: Arc::from("#body"),
            },
        });

        let edits = compute_edits(&element, "fn new_greet() {}", &prov);
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].confidence, EditConfidence::Approximate);
        assert_eq!(&*edits[0].file, "a.hick");
    }
}
