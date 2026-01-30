//! Node trait and core node types ported from HickoryDocs `execution.rs`.
//!
//! - `Node` trait: object-safe computation node
//! - `NodeTrace`: immutable trace through the node tree
//! - `StringNode`: leaf node holding literal text
//! - `SeparatorNode`: marker wrapper for separator detection
//! - `InsertionPoint`: dynamic container that uses `DynamicCombineLatest`

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures::{Stream, StreamExt};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::combine_latest::DynamicCombineLatest;
use crate::state::MultiDocumentState;

/// Type alias for boxed streams.
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send + 'static>>;

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// Execution context passed to all nodes.
#[derive(Clone, Default)]
pub struct Context {
    pub multi_document_state: Option<Arc<MultiDocumentState>>,
    pub verbose: bool,
}

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

    pub fn add_to_trace(&self, node: Arc<dyn Node>) -> Self {
        let mut t = self.trace.clone();
        t.push(node);
        Self {
            final_result: self.final_result.clone(),
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

// ---------------------------------------------------------------------------
// Operation + InsertionPoint
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Operation {
    node: Arc<dyn Node>,
    is_add: bool,
}

/// Dynamic container node that allows adding/removing children while streaming.
/// Internally uses `DynamicCombineLatest`.
pub struct InsertionPoint {
    children_tx: broadcast::Sender<Operation>,
    history: Mutex<Vec<Operation>>,
    separator: Option<Arc<dyn Node>>,
    trim_leading_separator: bool,
    never_complete_if_sources_may_change: bool,
    closed: Arc<AtomicBool>,
}

impl Default for InsertionPoint {
    fn default() -> Self {
        Self::new()
    }
}

impl InsertionPoint {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            children_tx: tx,
            history: Mutex::new(Vec::new()),
            separator: None,
            trim_leading_separator: true,
            never_complete_if_sources_may_change: false,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_separator(separator: Arc<dyn Node>) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            children_tx: tx,
            history: Mutex::new(Vec::new()),
            separator: Some(separator),
            trim_leading_separator: true,
            never_complete_if_sources_may_change: false,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn with_trim(trim_leading_separator: bool) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            children_tx: tx,
            history: Mutex::new(Vec::new()),
            separator: None,
            trim_leading_separator,
            never_complete_if_sources_may_change: false,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn new_with_never_complete(never_complete_if_sources_may_change: bool) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            children_tx: tx,
            history: Mutex::new(Vec::new()),
            separator: None,
            trim_leading_separator: true,
            never_complete_if_sources_may_change,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_separator_and_never_complete(
        separator: Arc<dyn Node>,
        never_complete_if_sources_may_change: bool,
    ) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            children_tx: tx,
            history: Mutex::new(Vec::new()),
            separator: Some(separator),
            trim_leading_separator: true,
            never_complete_if_sources_may_change,
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn add(&self, node: Arc<dyn Node>) {
        let op = Operation { node, is_add: true };
        self.history.lock().unwrap().push(op.clone());
        let _ = self.children_tx.send(op);
    }

    pub fn remove(&self, node: Arc<dyn Node>) {
        let op = Operation {
            node,
            is_add: false,
        };
        self.history.lock().unwrap().push(op.clone());
        let _ = self.children_tx.send(op);
    }

    /// Mark as closed — no more children will be added.
    pub fn close(&self) {
        if !self.closed.swap(true, Ordering::SeqCst) {
            let op = Operation {
                node: Arc::new(StringNode::new("")),
                is_add: false,
            };
            let _ = self.children_tx.send(op);
        }
    }
}

impl Node for InsertionPoint {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let (out_tx, out_rx) = mpsc::unbounded_channel::<Vec<NodeTrace>>();
        let out_stream = UnboundedReceiverStream::new(out_rx);

        let me: Arc<dyn Node> = self.clone();
        let trim = self.trim_leading_separator;
        let separator = self.separator.clone();
        let never_complete = self.never_complete_if_sources_may_change;
        let is_closed = self.closed.clone();

        let initial_ops = { self.history.lock().unwrap().clone() };
        let mut live_rx = self.children_tx.subscribe();

        tokio::spawn(async move {
            fn node_key(n: &Arc<dyn Node>) -> usize {
                Arc::as_ptr(n) as *const () as usize
            }

            let combine = move |nodeses: Vec<Vec<NodeTrace>>| -> Vec<NodeTrace> {
                let mut out = Vec::new();
                for nodes in nodeses {
                    for n in nodes {
                        out.push(n.add_to_trace(me.clone()));
                    }
                }
                out
            };

            let result: DynamicCombineLatest<Vec<NodeTrace>, Vec<NodeTrace>> =
                DynamicCombineLatest::new_with_config(combine, !never_complete);

            let emit_pulse = || {
                let _ = result.add_source(futures::stream::once(async { Vec::<NodeTrace>::new() }));
            };
            emit_pulse();

            let mut ids: HashMap<usize, usize> = HashMap::new();

            let mut apply_op = |op: Operation, ctx: &Context| {
                if op.is_add {
                    let key = node_key(&op.node);

                    let source_id = if let Some(sep) = separator.clone() {
                        let tmp = Arc::new(InsertionPoint::with_trim(false));
                        tmp.add(Arc::new(SeparatorNode::new(sep)));
                        tmp.add(op.node.clone());
                        result.add_source(tmp.get_stream(ctx.clone()))
                    } else {
                        result.add_source(op.node.clone().get_stream(ctx.clone()))
                    };

                    ids.insert(key, source_id);
                } else {
                    let key = node_key(&op.node);
                    if let Some(source_id) = ids.remove(&key) {
                        result.remove_source(source_id);
                        emit_pulse();
                    }
                }
            };

            for op in initial_ops {
                apply_op(op, &context);
            }

            let mut combined = result.stream();

            loop {
                tokio::select! {
                    next = combined.next() => {
                        match next {
                            Some(mut items) => {
                                if trim
                                    && !items.is_empty()
                                    && items[0].trace().iter().any(|n| n.is_separator())
                                {
                                    items.remove(0);
                                }

                                if out_tx.send(items).is_err() {
                                    result.shutdown();
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    op = live_rx.recv(), if !is_closed.load(Ordering::SeqCst) => {
                        match op {
                            Ok(op) => apply_op(op, &context),
                            Err(broadcast::error::RecvError::Closed) => {}
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        }
                    }
                }
            }
        });

        Box::pin(out_stream)
    }
}

// ---------------------------------------------------------------------------
// converge_to_string
// ---------------------------------------------------------------------------

/// Consume a node's stream and return the fully-converged text.
pub async fn converge_to_string(node: Arc<dyn Node>, ctx: Context) -> Option<String> {
    let mut stream = node.get_stream(ctx);
    let mut last: Option<String> = None;

    while let Some(batch) = stream.next().await {
        let mut s = String::new();
        for trace in &batch {
            let final_node = trace.final_result();
            if let Some(v) = final_node.as_string_value() {
                s.push_str(v);
            }
        }
        last = Some(s);
    }

    last
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn string_node_emits_value() {
        let node: Arc<dyn Node> = Arc::new(StringNode::new("hello"));
        let result = converge_to_string(node, Context::default()).await;
        assert_eq!(result.unwrap(), "hello");
    }

    #[tokio::test]
    async fn insertion_point_concatenates_children() {
        let ip = Arc::new(InsertionPoint::new());
        ip.add(Arc::new(StringNode::new("a")));
        ip.add(Arc::new(StringNode::new("b")));
        ip.add(Arc::new(StringNode::new("c")));
        ip.close();

        let result = converge_to_string(ip, Context::default()).await;
        assert_eq!(result.unwrap(), "abc");
    }

    #[tokio::test]
    async fn insertion_point_with_separator() {
        let sep: Arc<dyn Node> = Arc::new(StringNode::new(", "));
        let ip = Arc::new(InsertionPoint::with_separator(sep));
        ip.add(Arc::new(StringNode::new("x")));
        ip.add(Arc::new(StringNode::new("y")));
        ip.add(Arc::new(StringNode::new("z")));
        ip.close();

        let result = converge_to_string(ip, Context::default()).await;
        // Leading separator should be trimmed: "x, y, z"
        assert_eq!(result.unwrap(), "x, y, z");
    }

    #[tokio::test]
    async fn nested_insertion_points() {
        let outer = Arc::new(InsertionPoint::new());
        let inner = Arc::new(InsertionPoint::new());
        inner.add(Arc::new(StringNode::new("inner")));
        inner.close();
        outer.add(Arc::new(StringNode::new("before-")));
        outer.add(inner);
        outer.add(Arc::new(StringNode::new("-after")));
        outer.close();

        let result = converge_to_string(outer, Context::default()).await;
        assert_eq!(result.unwrap(), "before-inner-after");
    }
}
