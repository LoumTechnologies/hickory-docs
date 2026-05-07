//! `TransformNode` — generic pattern for nodes that wrap children and
//! transform their output.

use std::sync::Arc;

use futures::StreamExt;

use crate::context::Context;
use crate::insertion_point::InsertionPoint;
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

                    for segment in segments {
                        let node: Arc<dyn Node> = match segment.origin {
                            TransformSegmentOrigin::Passthrough => {
                                // Inherit origin from input node
                                if let Some(ref origin) = original_origin {
                                    Arc::new(SpanNode::new(segment.text, origin.clone()))
                                } else {
                                    Arc::new(StringNode::new(segment.text))
                                }
                            }
                            TransformSegmentOrigin::Substituted { .. } => {
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
