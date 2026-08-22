//! `TransformNode` — generic pattern for nodes that wrap children and
//! transform their output.

use std::sync::Arc;

use futures::StreamExt;

use crate::context::Context;
use crate::insertion_point::InsertionPoint;
use hick_lang::SourceSpan;

use crate::node::{BoxStream, Node, NodeTrace, SourceOrigin, SpanNode, StringNode};

/// A node that subscribes to its children's combined stream and transforms
/// each trace's string output via a caller-supplied function.
///
/// For each `NodeTrace` in each emission:
/// - If `final_result.as_string_value()` returns `Some`: calls `transformer_fn`,
///   creates a new `StringNode` with the result, and calls `trace.transform(new_node, self)`.
/// - Otherwise: passes through via `trace.add_to_trace(self)`.
pub struct TransformNode<F>
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    children: Arc<InsertionPoint>,
    transformer_fn: F,
    label: String,
}

impl<F> TransformNode<F>
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    pub fn new(children: Arc<InsertionPoint>, transformer_fn: F, label: impl Into<String>) -> Self {
        Self {
            children,
            transformer_fn,
            label: label.into(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl<F> Node for TransformNode<F>
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let children_stream = self.children.clone().get_stream(context);
        let me: Arc<dyn Node> = self.clone();
        let me2 = self;

        Box::pin(children_stream.map(move |items| {
            items
                .into_iter()
                .map(|trace| {
                    if let Some(v) = trace.final_result().as_string_value() {
                        let transformed = (me2.transformer_fn)(v);
                        let new_node: Arc<dyn Node> = Arc::new(StringNode::new(transformed));
                        trace.transform(new_node, me.clone())
                    } else {
                        trace.add_to_trace(me.clone())
                    }
                })
                .collect::<Vec<_>>()
        }))
    }

    fn as_string_value(&self) -> Option<&str> {
        Some(&self.label)
    }
}

// ---------------------------------------------------------------------------
// Provenance-aware transform
// ---------------------------------------------------------------------------

/// A segment of transformed text with its provenance origin.
#[derive(Debug, Clone)]
pub struct TransformSegment {
    /// The text content.
    pub text: String,
    /// How this segment was produced.
    pub origin: TransformSegmentOrigin,
}

/// How a transform segment was produced.
#[derive(Debug, Clone)]
pub enum TransformSegmentOrigin {
    /// Text passed through unchanged — inherits origin from the input node.
    Passthrough,
    /// Text was produced by a substitution.
    Substituted { pattern: String },
}

/// The part of `origin` that covers `[offset, offset + len)` of an input text
/// that is `input_len` bytes long.
///
/// This is what makes a split paragraph keep its ribbons. A segmenter cuts one
/// prose node into several — "text", a substituted value, "more text" — and
/// each piece inherits the origin of the whole. Handing every piece the whole
/// node's span is a claim that a 12-byte segment came from a 200-byte region,
/// and the lineage layer (correctly) refuses it: it keeps an origin only when
/// the span's length matches the bytes it produced, so every piece degraded to
/// `synthetic`. One substitution anywhere in a paragraph therefore erased the
/// ribbons for the whole paragraph.
///
/// Narrowing is only sound when the node's text IS its source span byte for
/// byte — which is the same condition lineage checks before trusting the
/// origin at all. When it does not hold (a dedent removed something), the
/// origin is returned unchanged and the old degrade-to-synthetic behaviour
/// stands: a wrong mapping is worse than no mapping.
///
/// `start_line` / `start_col` stay the node's own. They locate the region for
/// a human reading an error, and recomputing them would need the source text
/// this node does not have; the byte offsets are what lineage maps with.
fn narrowed(origin: &SourceOrigin, offset: usize, len: usize, input_len: usize) -> SourceOrigin {
    fn cut(span: &SourceSpan, offset: usize, len: usize, input_len: usize) -> Option<SourceSpan> {
        if span.end.checked_sub(span.start) != Some(input_len) {
            return None;
        }
        let start = span.start.checked_add(offset)?;
        let end = start.checked_add(len)?;
        if end > span.end {
            return None;
        }
        Some(SourceSpan {
            start,
            end,
            ..*span
        })
    }
    match origin {
        SourceOrigin::Literal { file, span } => match cut(span, offset, len, input_len) {
            Some(span) => SourceOrigin::Literal {
                file: file.clone(),
                span,
            },
            None => origin.clone(),
        },
        SourceOrigin::Paste {
            selector,
            file,
            span: Some(span),
        } => match cut(span, offset, len, input_len) {
            Some(span) => SourceOrigin::Paste {
                selector: selector.clone(),
                file: file.clone(),
                span: Some(span),
            },
            None => origin.clone(),
        },
        SourceOrigin::Agent {
            session,
            turn,
            file,
            span: Some(span),
        } => match cut(span, offset, len, input_len) {
            Some(span) => SourceOrigin::Agent {
                session: session.clone(),
                turn: *turn,
                file: file.clone(),
                span: Some(span),
            },
            None => origin.clone(),
        },
        other => other.clone(),
    }
}

/// A provenance-aware transform node that tracks which parts of the output
/// were modified by substitutions.
///
/// Unlike `TransformNode` which produces a single `StringNode` per input trace,
/// this node produces multiple `SpanNode`s — one per segment — each with
/// appropriate `SourceOrigin`. Passthrough segments inherit the input node's
/// origin; substituted segments get `SourceOrigin::Synthetic`.
pub struct ProvenanceTransformNode<F>
where
    F: Fn(&str) -> Vec<TransformSegment> + Send + Sync + 'static,
{
    children: Arc<InsertionPoint>,
    segmenter_fn: F,
    label: String,
}

impl<F> ProvenanceTransformNode<F>
where
    F: Fn(&str) -> Vec<TransformSegment> + Send + Sync + 'static,
{
    pub fn new(children: Arc<InsertionPoint>, segmenter_fn: F, label: impl Into<String>) -> Self {
        Self {
            children,
            segmenter_fn,
            label: label.into(),
        }
    }
}

impl<F> Node for ProvenanceTransformNode<F>
where
    F: Fn(&str) -> Vec<TransformSegment> + Send + Sync + 'static,
{
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let children_stream = self.children.clone().get_stream(context);
        let me: Arc<dyn Node> = self.clone();
        let me2 = self;

        Box::pin(children_stream.map(move |items| {
            let mut result_traces = Vec::new();

            for trace in items {
                if let Some(v) = trace.final_result().as_string_value() {
                    let segments = (me2.segmenter_fn)(v);

                    // Get the original node's source origin for passthrough segments
                    let original_origin = trace.final_result().source_origin().cloned();

                    // How far into the INPUT text the next segment starts.
                    // A passthrough segment consumed its own bytes; a
                    // substituted one consumed the pattern it replaced, which
                    // is why the pattern is carried on the segment at all.
                    let input_len = v.len();
                    let mut consumed = 0usize;

                    for segment in segments {
                        let node: Arc<dyn Node> = match &segment.origin {
                            TransformSegmentOrigin::Passthrough => {
                                let len = segment.text.len();
                                let node: Arc<dyn Node> = match original_origin {
                                    Some(ref origin) => Arc::new(SpanNode::new(
                                        segment.text,
                                        narrowed(origin, consumed, len, input_len),
                                    )),
                                    None => Arc::new(StringNode::new(segment.text)),
                                };
                                consumed += len;
                                node
                            }
                            TransformSegmentOrigin::Substituted { pattern } => {
                                consumed += pattern.len();
                                Arc::new(SpanNode::new(segment.text, SourceOrigin::Synthetic))
                            }
                        };
                        // Each segment becomes its own trace entry
                        let segment_trace = NodeTrace::new(node);
                        let segment_trace = segment_trace.add_to_trace(me.clone());
                        result_traces.push(segment_trace);
                    }
                } else {
                    result_traces.push(trace.add_to_trace(me.clone()));
                }
            }

            result_traces
        }))
    }

    fn as_string_value(&self) -> Option<&str> {
        Some(&self.label)
    }
}

#[cfg(test)]
mod narrowing_tests {
    use super::*;

    fn span(start: usize, end: usize) -> SourceSpan {
        SourceSpan::new(start, end, 1, 0)
    }

    fn literal(start: usize, end: usize) -> SourceOrigin {
        SourceOrigin::Literal {
            file: Arc::from("notes.hick"),
            span: span(start, end),
        }
    }

    fn span_of(origin: &SourceOrigin) -> (usize, usize) {
        match origin {
            SourceOrigin::Literal { span, .. } => (span.start, span.end),
            other => panic!("expected a literal origin, got {other:?}"),
        }
    }

    #[test]
    fn a_segment_gets_the_part_of_the_span_it_actually_covers() {
        // The whole node is bytes 100..120 of the document; the segmenter cut
        // out bytes 5..12 of it. Handing that segment the whole 20-byte span
        // is the claim lineage refuses, and refusing it is what used to erase
        // the ribbon for the entire paragraph.
        let origin = literal(100, 120);
        assert_eq!(span_of(&narrowed(&origin, 0, 5, 20)), (100, 105));
        assert_eq!(span_of(&narrowed(&origin, 5, 7, 20)), (105, 112));
        assert_eq!(span_of(&narrowed(&origin, 12, 8, 20)), (112, 120));
    }

    #[test]
    fn a_node_whose_text_is_not_its_span_is_left_alone_rather_than_guessed_at() {
        // A dedent removed bytes, so offsets into the text no longer index
        // the source. The old whole-span origin stands, and the lineage layer
        // degrades it to synthetic — which is the honest answer.
        let origin = literal(100, 120);
        assert_eq!(span_of(&narrowed(&origin, 0, 5, 14)), (100, 120));
    }

    #[test]
    fn a_segment_that_would_run_past_the_span_is_left_alone() {
        let origin = literal(100, 120);
        assert_eq!(span_of(&narrowed(&origin, 18, 5, 20)), (100, 120));
    }

    #[test]
    fn an_origin_with_no_span_to_narrow_comes_back_unchanged() {
        let exec = SourceOrigin::Exec {
            container: Arc::from("build"),
            tag_line: 3,
        };
        assert!(matches!(
            narrowed(&exec, 0, 4, 20),
            SourceOrigin::Exec { tag_line: 3, .. }
        ));
        assert!(matches!(
            narrowed(&SourceOrigin::Synthetic, 0, 4, 20),
            SourceOrigin::Synthetic
        ));
    }
}
