<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/node_bridge.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6febef34dfaf"></a>
## `crates/hick-grove/src/node_bridge.rs`

<a id="f-6febef34dfaf-ElementIdentity"></a>
### `ElementIdentity`

<storycode:copy id="f-6febef34dfaf-ElementIdentity">
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

</storycode:copy>
<a id="f-6febef34dfaf-ElementSnapshot"></a>
### `ElementSnapshot`

A snapshot of a Yrs XML element's state, used as the value emitted by `YrsElementNode`.

<storycode:copy id="f-6febef34dfaf-ElementSnapshot">
/// A snapshot of a Yrs XML element's state, used as the value emitted by `YrsElementNode`.
#[derive(Clone, Debug)]
pub struct ElementSnapshot {
    pub local_name: String,
    pub attributes: Vec<(String, String)>,
    pub text_content: String,
}

</storycode:copy>
<a id="f-6febef34dfaf-impl-ElementSnapshot"></a>
### `impl-ElementSnapshot`

<storycode:copy id="f-6febef34dfaf-impl-ElementSnapshot">
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

</storycode:copy>
<a id="f-6febef34dfaf-YrsElementNode"></a>
### `YrsElementNode`

A hick-flow `Node` backed by a `tokio::sync::watch` channel.
Each Yrs change pushes a new `ElementSnapshot` into the watch channel.
`get_stream()` yields `NodeTrace` batches from the watch receiver,
enabling reactive composition via InsertionPoint/DynamicCombineLatest.

<storycode:copy id="f-6febef34dfaf-YrsElementNode">
/// A hick-flow `Node` backed by a `tokio::sync::watch` channel.
///
/// Each Yrs change pushes a new `ElementSnapshot` into the watch channel.
/// `get_stream()` yields `NodeTrace` batches from the watch receiver,
/// enabling reactive composition via InsertionPoint/DynamicCombineLatest.
pub struct YrsElementNode {
    rx: watch::Receiver<ElementSnapshot>,
}

</storycode:copy>
<a id="f-6febef34dfaf-impl-YrsElementNode"></a>
### `impl-YrsElementNode`

<storycode:copy id="f-6febef34dfaf-impl-YrsElementNode">
impl YrsElementNode {
    pub fn new(rx: watch::Receiver<ElementSnapshot>) -> Self {
        Self { rx }
    }

    pub fn current_snapshot(&self) -> ElementSnapshot {
        self.rx.borrow().clone()
    }
}

</storycode:copy>
<a id="f-6febef34dfaf-impl-Node-for-YrsElementNode"></a>
### `impl-Node for YrsElementNode`

<storycode:copy id="f-6febef34dfaf-impl-Node-for-YrsElementNode">
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

</storycode:copy>
<a id="f-6febef34dfaf-ElementSnapshotSender"></a>
### `ElementSnapshotSender`

Sender half for pushing element snapshots from the Yrs bridge.

<storycode:copy id="f-6febef34dfaf-ElementSnapshotSender">
/// Sender half for pushing element snapshots from the Yrs bridge.
pub struct ElementSnapshotSender {
    tx: watch::Sender<ElementSnapshot>,
}

</storycode:copy>
<a id="f-6febef34dfaf-impl-ElementSnapshotSender"></a>
### `impl-ElementSnapshotSender`

<storycode:copy id="f-6febef34dfaf-impl-ElementSnapshotSender">
impl ElementSnapshotSender {
    pub fn new(initial: ElementSnapshot) -> (Self, watch::Receiver<ElementSnapshot>) {
        let (tx, rx) = watch::channel(initial);
        (Self { tx }, rx)
    }

    pub fn send(&self, snapshot: ElementSnapshot) {
        let _ = self.tx.send(snapshot);
    }
}
</storycode:copy>

### Assemble `crates/hick-grove/src/node_bridge.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/node_bridge.rs"><storycode:paste select="#f-6febef34dfaf-ElementIdentity"/><storycode:paste select="#f-6febef34dfaf-ElementSnapshot"/><storycode:paste select="#f-6febef34dfaf-impl-ElementSnapshot"/><storycode:paste select="#f-6febef34dfaf-YrsElementNode"/><storycode:paste select="#f-6febef34dfaf-impl-YrsElementNode"/><storycode:paste select="#f-6febef34dfaf-impl-Node-for-YrsElementNode"/><storycode:paste select="#f-6febef34dfaf-ElementSnapshotSender"/><storycode:paste select="#f-6febef34dfaf-impl-ElementSnapshotSender"/></storycode:file>

</storycode:doc>
