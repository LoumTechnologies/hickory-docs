use std::any::Any;
use std::sync::Arc;

use hick_flow::{BoxStream, Context, Node, NodeTrace, StringNode};
use tokio::sync::watch;

use crate::namespace::NamespaceUri;

/// Identity of a namespace-qualified element, used to subscribe to specific
/// elements in the reactive node bridge.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ElementIdentity {
    pub namespace_uri: NamespaceUri,
    pub local_name: String,
    pub id: String,
}

/// A snapshot of a Yrs XML element's state, used as the value emitted by `YrsElementNode`.
#[derive(Clone, Debug)]
pub struct ElementSnapshot {
    pub local_name: String,
    pub attributes: Vec<(String, String)>,
    pub text_content: String,
}

impl ElementSnapshot {
    pub fn to_string_repr(&self) -> String {
        let mut s = format!("<{}", self.local_name);
        for (k, v) in &self.attributes {
            s.push_str(&format!(" {}=\"{}\"", k, v));
        }
        if self.text_content.is_empty() {
            s.push_str("/>");
        } else {
            s.push('>');
            s.push_str(&self.text_content);
            s.push_str(&format!("</{}>", self.local_name));
        }
        s
    }
}

/// A hick-flow `Node` backed by a `tokio::sync::watch` channel.
///
/// Each Yrs change pushes a new `ElementSnapshot` into the watch channel.
/// `get_stream()` yields `NodeTrace` batches from the watch receiver,
/// enabling reactive composition via InsertionPoint/DynamicCombineLatest.
pub struct YrsElementNode {
    rx: watch::Receiver<ElementSnapshot>,
}

impl YrsElementNode {
    pub fn new(rx: watch::Receiver<ElementSnapshot>) -> Self {
        Self { rx }
    }

    pub fn current_snapshot(&self) -> ElementSnapshot {
        self.rx.borrow().clone()
    }
}

impl Node for YrsElementNode {
    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let mut rx = self.rx.clone();

        let stream = async_stream::stream! {
            // Emit current value first
            {
                let snap = rx.borrow().clone();
                let string_node: Arc<dyn Node> = Arc::new(StringNode::new(snap.to_string_repr()));
                yield vec![NodeTrace::new(string_node)];
            }

            // Then emit on each change
            while rx.changed().await.is_ok() {
                let snap = rx.borrow().clone();
                let string_node: Arc<dyn Node> = Arc::new(StringNode::new(snap.to_string_repr()));
                yield vec![NodeTrace::new(string_node)];
            }
        };

        Box::pin(stream)
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

/// Sender half for pushing element snapshots from the Yrs bridge.
pub struct ElementSnapshotSender {
    tx: watch::Sender<ElementSnapshot>,
}

impl ElementSnapshotSender {
    pub fn new(initial: ElementSnapshot) -> (Self, watch::Receiver<ElementSnapshot>) {
        let (tx, rx) = watch::channel(initial);
        (Self { tx }, rx)
    }

    pub fn send(&self, snapshot: ElementSnapshot) {
        let _ = self.tx.send(snapshot);
    }
}
