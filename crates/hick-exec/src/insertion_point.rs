//! `InsertionPoint` — dynamic container node that allows adding/removing
//! children while streaming. Uses `DynamicCombineLatest` internally.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::combine_latest::DynamicCombineLatest;
use crate::context::Context;
use crate::node::{BoxStream, Node, NodeTrace, SeparatorNode, StringNode};

// ---------------------------------------------------------------------------
// Operation
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct Operation {
    node: Arc<dyn Node>,
    is_add: bool,
}

// ---------------------------------------------------------------------------
// InsertionPoint
// ---------------------------------------------------------------------------

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
