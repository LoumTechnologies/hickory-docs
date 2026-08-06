//! Character-level provenance tracking for generated output.
//!
//! Maps output byte ranges back to their source origins, enabling
//! tools like hick-lens to know which `.hick` source positions
//! produced each region of a generated file.

use crate::node::SourceOrigin;

/// A single span in the output mapped to a source origin.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvenanceSpan {
    /// Start byte offset in the output (inclusive).
    pub output_start: usize,
    /// End byte offset in the output (exclusive).
    pub output_end: usize,
    /// Where this output region came from.
    pub origin: SourceOrigin,
}

/// Maps output byte ranges to their source origins.
///
/// Built by [`converge_with_provenance()`](crate::converge::converge_with_provenance)
/// during file convergence. Spans are stored in output-order and do not overlap.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProvenanceMap {
    spans: Vec<ProvenanceSpan>,
}

impl ProvenanceMap {
    pub fn new() -> Self {
        Self { spans: Vec::new() }
    }

    /// Add a span to the map.
    pub fn push(&mut self, span: ProvenanceSpan) {
        self.spans.push(span);
    }

    /// Number of tracked spans.
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// All spans, in output order.
    pub fn spans(&self) -> &[ProvenanceSpan] {
        &self.spans
    }

    /// Find the origin of the output at the given byte offset.
    pub fn origin_at(&self, byte_offset: usize) -> Option<&SourceOrigin> {
        self.spans
            .iter()
            .find(|s| s.output_start <= byte_offset && byte_offset < s.output_end)
            .map(|s| &s.origin)
    }

    /// Find all spans that overlap the given output range.
    pub fn spans_in_range(&self, start: usize, end: usize) -> Vec<&ProvenanceSpan> {
        self.spans
            .iter()
            .filter(|s| s.output_start < end && s.output_end > start)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_lang::SourceSpan;
    use std::sync::Arc;

    #[test]
    fn empty_map() {
        let map = ProvenanceMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert!(map.origin_at(0).is_none());
    }

    #[test]
    fn origin_at_finds_correct_span() {
        let mut map = ProvenanceMap::new();
        map.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: SourceOrigin::Literal {
                file: Arc::from("a.hick"),
                span: SourceSpan::new(0, 5, 1, 0),
            },
        });
        map.push(ProvenanceSpan {
            output_start: 5,
            output_end: 10,
            origin: SourceOrigin::Paste {
                selector: Arc::from("#ver"),
                file: None,
                span: None,
            },
        });

        match map.origin_at(2) {
            Some(SourceOrigin::Literal { file, .. }) => assert_eq!(&**file, "a.hick"),
            other => panic!("Expected Literal, got {:?}", other),
        }
        match map.origin_at(7) {
            Some(SourceOrigin::Paste { selector, .. }) => assert_eq!(&**selector, "#ver"),
            other => panic!("Expected Paste, got {:?}", other),
        }
        assert!(map.origin_at(10).is_none());
    }

    #[test]
    fn spans_in_range_returns_overlapping() {
        let mut map = ProvenanceMap::new();
        map.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: SourceOrigin::Synthetic,
        });
        map.push(ProvenanceSpan {
            output_start: 5,
            output_end: 10,
            origin: SourceOrigin::Synthetic,
        });
        map.push(ProvenanceSpan {
            output_start: 10,
            output_end: 15,
            origin: SourceOrigin::Synthetic,
        });

        let overlapping = map.spans_in_range(3, 12);
        assert_eq!(overlapping.len(), 3); // all three overlap [3, 12)

        let overlapping = map.spans_in_range(5, 10);
        assert_eq!(overlapping.len(), 1); // only the middle span
        assert_eq!(overlapping[0].output_start, 5);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        let mut map = ProvenanceMap::new();
        map.push(ProvenanceSpan {
            output_start: 0,
            output_end: 42,
            origin: SourceOrigin::Literal {
                file: Arc::from("main.hick"),
                span: SourceSpan::new(100, 142, 5, 2),
            },
        });
        map.push(ProvenanceSpan {
            output_start: 42,
            output_end: 60,
            origin: SourceOrigin::Exec {
                container: Arc::from("codegen"),
                tag_line: 10,
            },
        });
        map.push(ProvenanceSpan {
            output_start: 60,
            output_end: 70,
            origin: SourceOrigin::Paste {
                selector: Arc::from("#ver"),
                file: None,
                span: None,
            },
        });
        map.push(ProvenanceSpan {
            output_start: 70,
            output_end: 80,
            origin: SourceOrigin::Variable {
                name: Arc::from("version"),
            },
        });
        map.push(ProvenanceSpan {
            output_start: 80,
            output_end: 90,
            origin: SourceOrigin::Synthetic,
        });

        let json = serde_json::to_string(&map).unwrap();
        let map2: ProvenanceMap = serde_json::from_str(&json).unwrap();

        assert_eq!(map2.len(), 5);
        assert_eq!(map2.spans()[0].output_start, 0);
        assert_eq!(map2.spans()[0].output_end, 42);
        match &map2.spans()[0].origin {
            SourceOrigin::Literal { file, span } => {
                assert_eq!(file.as_ref(), "main.hick");
                assert_eq!(span.start, 100);
                assert_eq!(span.end, 142);
            }
            other => panic!("Expected Literal, got {:?}", other),
        }
        match &map2.spans()[4].origin {
            SourceOrigin::Synthetic => {}
            other => panic!("Expected Synthetic, got {:?}", other),
        }
    }
}
