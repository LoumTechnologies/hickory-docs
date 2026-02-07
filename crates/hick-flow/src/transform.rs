//! `TransformNode` — generic pattern for nodes that wrap children and
//! transform their output.

use std::sync::Arc;

use futures::StreamExt;

use crate::context::Context;
use crate::insertion_point::InsertionPoint;
use crate::node::{BoxStream, Node, NodeTrace, StringNode};

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
