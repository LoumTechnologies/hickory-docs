//! Node trait and core node types.
//!
//! - `Node` trait: object-safe computation node
//! - `NodeTrace`: immutable trace through the node tree
//! - `StringNode`: leaf node holding literal text
//! - `SeparatorNode`: marker wrapper for separator detection

use std::pin::Pin;
use std::sync::Arc;

use futures::{Stream, StreamExt};

use crate::context::Context;

/// Type alias for boxed streams.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

// ---------------------------------------------------------------------------
// Node trait
// ---------------------------------------------------------------------------

/// Object-safe trait for tree-structured computation nodes.
pub trait Node: Send + Sync {
    /// Primary method: returns a stream of `NodeTrace` vectors.
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>>;

    /// Type discriminator without downcasting.
    fn is_separator(&self) -> bool {
        false
    }

    /// Extract string value if this is a `StringNode`.
    fn as_string_value(&self) -> Option<&str> {
        None
    }

    /// Per-node identifier in bytes. Defaults to address-based.
    fn id_bytes(&self) -> Vec<u8> {
        let addr = (self as *const _ as *const ()).addr() as u128;
        addr.to_le_bytes().to_vec()
    }
}

// ---------------------------------------------------------------------------
// NodeTrace
// ---------------------------------------------------------------------------

/// Immutable, cloneable trace representing a computation path through nodes.
#[derive(Clone)]
pub struct NodeTrace {
    final_result: Arc<dyn Node>,
    trace: Vec<Arc<dyn Node>>,
}

impl NodeTrace {
    pub fn new(final_result: Arc<dyn Node>) -> Self {
        Self {
            final_result,
            trace: Vec::new(),
        }
    }

    /// Record a passthrough node in the trace without changing the final result.
    pub fn add_to_trace(&self, node: Arc<dyn Node>) -> Self {
        let mut t = self.trace.clone();
        t.push(node);
        Self {
            final_result: self.final_result.clone(),
            trace: t,
        }
    }

    /// Record a transformation: changes the final result and records the
    /// transformer in the trace.
    pub fn transform(&self, new_result: Arc<dyn Node>, transformer: Arc<dyn Node>) -> Self {
        let mut t = self.trace.clone();
        t.push(transformer);
        Self {
            final_result: new_result,
            trace: t,
        }
    }

    pub fn final_result(&self) -> Arc<dyn Node> {
        self.final_result.clone()
    }

    pub fn trace(&self) -> &[Arc<dyn Node>] {
        &self.trace
    }
}

// ---------------------------------------------------------------------------
// StringNode
// ---------------------------------------------------------------------------

/// Terminal node representing a string value.
pub struct StringNode {
    value: String,
    stable_id: Option<Vec<u8>>,
}

impl StringNode {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            stable_id: None,
        }
    }

    pub fn new_with_id(value: impl Into<String>, id_bytes: Option<Vec<u8>>) -> Self {
        Self {
            value: value.into(),
            stable_id: id_bytes,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Node for StringNode {
    fn as_string_value(&self) -> Option<&str> {
        Some(self.value())
    }

    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let item = vec![NodeTrace::new(self as Arc<dyn Node>)];
        Box::pin(futures::stream::once(async move { item }))
    }

    fn id_bytes(&self) -> Vec<u8> {
        if let Some(b) = &self.stable_id {
            return b.clone();
        }
        let addr = (self as *const StringNode as *const ()).addr() as u128;
        addr.to_le_bytes().to_vec()
    }
}

// ---------------------------------------------------------------------------
// SeparatorNode
// ---------------------------------------------------------------------------

/// Marker node that wraps another node and adds itself to traces for later filtering.
pub struct SeparatorNode {
    value: Arc<dyn Node>,
}

impl SeparatorNode {
    pub fn new(value: Arc<dyn Node>) -> Self {
        Self { value }
    }
}

impl Node for SeparatorNode {
    fn is_separator(&self) -> bool {
        true
    }

    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let inner = self.value.clone().get_stream(context);
        let me: Arc<dyn Node> = self;

        Box::pin(inner.map(move |items| {
            items
                .into_iter()
                .map(|trace| trace.add_to_trace(me.clone()))
                .collect::<Vec<_>>()
        }))
    }
}
