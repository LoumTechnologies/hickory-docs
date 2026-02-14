//! Node types for structured data streams.
//!
//! `RecordChangeNode` is a leaf node wrapping a single `RecordChange`.
//! `StreamingStructuredDataNode` is a source node that produces a `ChangeStream`.

use std::any::Any;
use std::sync::Arc;

use futures::StreamExt;
use hick_flow::Context;
use hick_flow::node::{BoxStream, Node, NodeTrace, NodeValue};

use crate::changes::{ChangeStream, RecordChange};
use crate::schema::LiveSchema;
use crate::sources::ChangeSource;

// ---------------------------------------------------------------------------
// RecordChangeNode — leaf node wrapping a single change event
// ---------------------------------------------------------------------------

/// Leaf node carrying a single `RecordChange` as its payload.
///
/// Downstream structured-data consumers downcast `NodeTrace.final_result()`
/// to `RecordChangeNode` to extract the change. String-oriented consumers
/// see `NodeValue::None` and skip it.
pub struct RecordChangeNode {
    pub change: RecordChange,
    stable_id: Option<Vec<u8>>,
}

impl RecordChangeNode {
    pub fn new(change: RecordChange) -> Self {
        Self {
            change,
            stable_id: None,
        }
    }

    pub fn new_with_id(change: RecordChange, id: Vec<u8>) -> Self {
        Self {
            change,
            stable_id: Some(id),
        }
    }
}

impl Node for RecordChangeNode {
    fn get_stream(self: Arc<Self>, _context: Context) -> BoxStream<Vec<NodeTrace>> {
        let item = vec![NodeTrace::new(self as Arc<dyn Node>)];
        Box::pin(futures::stream::once(async move { item }))
    }

    fn node_value(&self) -> NodeValue {
        NodeValue::None
    }

    fn id_bytes(&self) -> Vec<u8> {
        if let Some(b) = &self.stable_id {
            return b.clone();
        }
        let addr = (self as *const RecordChangeNode as *const ()).addr() as u128;
        addr.to_le_bytes().to_vec()
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

// ---------------------------------------------------------------------------
// StreamingStructuredDataNode — source node producing a ChangeStream
// ---------------------------------------------------------------------------

/// Source node that produces a stream of `RecordChange` events.
///
/// Each emission on the underlying `ChangeStream` is wrapped in a
/// `RecordChangeNode` and emitted as a `NodeTrace`.
pub struct StreamingStructuredDataNode {
    pub schema: LiveSchema,
    pub source: Arc<dyn ChangeSource>,
    pub stable_id: Option<String>,
}

impl StreamingStructuredDataNode {
    pub fn new(schema: LiveSchema, source: Arc<dyn ChangeSource>) -> Self {
        Self {
            schema,
            source,
            stable_id: None,
        }
    }

    pub fn new_with_id(schema: LiveSchema, source: Arc<dyn ChangeSource>, id: String) -> Self {
        Self {
            schema,
            source,
            stable_id: Some(id),
        }
    }
}

impl Node for StreamingStructuredDataNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let change_stream: ChangeStream = self.source.stream(context);
        Box::pin(change_stream.map(|change| {
            let node: Arc<dyn Node> = Arc::new(RecordChangeNode::new(change));
            vec![NodeTrace::new(node)]
        }))
    }

    fn node_value(&self) -> NodeValue {
        NodeValue::None
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn id_bytes(&self) -> Vec<u8> {
        if let Some(id) = &self.stable_id {
            return id.as_bytes().to_vec();
        }
        let addr = (self as *const StreamingStructuredDataNode as *const ()).addr() as u128;
        addr.to_le_bytes().to_vec()
    }
}

// ---------------------------------------------------------------------------
// NodeExt — ergonomic downcasting for structured data
// ---------------------------------------------------------------------------

/// Extension trait for ergonomic downcasting of `dyn Node` to structured data types.
pub trait NodeExt {
    /// Try to extract the `RecordChange` from a `RecordChangeNode`.
    fn as_record_change(&self) -> Option<&RecordChange>;
}

impl NodeExt for dyn Node {
    fn as_record_change(&self) -> Option<&RecordChange> {
        self.as_any()
            .and_then(|any| any.downcast_ref::<RecordChangeNode>())
            .map(|n| &n.change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::ChangeType;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;

    fn test_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(StringArray::from(vec!["alice", "bob"])),
            ],
        )
        .unwrap()
    }

    #[test]
    fn record_change_node_value_is_none() {
        let node = RecordChangeNode::new(RecordChange::add(test_batch()));
        assert!(matches!(node.node_value(), NodeValue::None));
    }

    #[test]
    fn node_ext_downcast() {
        let node: Arc<dyn Node> = Arc::new(RecordChangeNode::new(RecordChange::add(test_batch())));
        let change = node.as_ref().as_record_change().unwrap();
        assert_eq!(change.change_type, ChangeType::Add);
        assert_eq!(change.num_rows(), 2);
    }

    #[test]
    fn node_ext_returns_none_for_string_node() {
        let node: Arc<dyn Node> = Arc::new(hick_flow::StringNode::new("hello"));
        assert!(node.as_ref().as_record_change().is_none());
    }

    #[tokio::test]
    async fn streaming_node_emits_changes() {
        use crate::sources::memory::MemorySource;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));
        let live_schema = LiveSchema::new_strict(schema, vec!["id".into()]);

        let source = Arc::new(MemorySource::new(live_schema.clone(), test_batch()));
        let node = Arc::new(StreamingStructuredDataNode::new(live_schema, source));

        let mut stream = node.get_stream(Context::default());
        let batch = stream.next().await.unwrap();
        assert_eq!(batch.len(), 1);

        let node = batch[0].final_result();
        let change = node.as_record_change().unwrap();
        assert_eq!(change.change_type, ChangeType::Add);
        assert_eq!(change.num_rows(), 2);
    }
}
