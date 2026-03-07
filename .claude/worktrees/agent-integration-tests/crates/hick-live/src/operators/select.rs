//! Select (projection) operator for incremental collections.
//!
//! Projects columns from each change event. Strictness is preserved only
//! if all key columns survive the projection.

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use futures::StreamExt;
use hick_flow::Context;
use hick_flow::node::{BoxStream, Node, NodeTrace, NodeValue};

use crate::changes::RecordChange;
use crate::node::{NodeExt, RecordChangeNode};
use crate::schema::LiveSchema;

/// Projection function type: transforms a RecordBatch by selecting/computing columns.
pub type Projection = Arc<dyn Fn(&RecordBatch) -> RecordBatch + Send + Sync>;

/// Projects columns from upstream change events.
///
/// On Add: apply projection, emit Add with transformed batch.
/// On Remove: apply projection, emit Remove.
/// Strictness: preserved if key columns survive projection (tracked by `output_schema`).
pub struct SelectNode {
    pub source: Arc<dyn Node>,
    pub projection: Projection,
    pub output_schema: LiveSchema,
}

impl SelectNode {
    pub fn new(source: Arc<dyn Node>, projection: Projection, output_schema: LiveSchema) -> Self {
        Self {
            source,
            projection,
            output_schema,
        }
    }

    /// Create a SelectNode that projects a fixed set of column names.
    pub fn columns(source: Arc<dyn Node>, columns: &[&str], input_schema: &LiveSchema) -> Self {
        let col_names: Vec<String> = columns.iter().map(|s| s.to_string()).collect();
        let output_schema = input_schema.project(&col_names);

        let col_indices: Vec<usize> = columns
            .iter()
            .filter_map(|name| {
                input_schema
                    .arrow_schema
                    .column_with_name(name)
                    .map(|(i, _)| i)
            })
            .collect();

        let projection: Projection = Arc::new(move |batch| batch.project(&col_indices).unwrap());

        Self {
            source,
            projection,
            output_schema,
        }
    }
}

impl Node for SelectNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let source_stream = self.source.clone().get_stream(context);
        let projection = self.projection.clone();
        let me: Arc<dyn Node> = self;

        Box::pin(source_stream.map(move |traces| {
            let projection = projection.clone();
            let me = me.clone();
            traces
                .into_iter()
                .map(|trace| {
                    if let Some(change) = trace.final_result().as_record_change() {
                        let projected = projection(&change.records);
                        let new_change =
                            RecordChange::new(change.change_type, projected, change.is_strict);
                        let new_node: Arc<dyn Node> = Arc::new(RecordChangeNode::new(new_change));
                        trace.transform(new_node, me.clone())
                    } else {
                        trace.add_to_trace(me.clone())
                    }
                })
                .collect()
        }))
    }

    fn node_value(&self) -> NodeValue {
        NodeValue::None
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StreamingStructuredDataNode;
    use crate::changes::ChangeType;
    use crate::sources::memory::MemorySource;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use futures::StreamExt;

    fn test_schema() -> (Arc<Schema>, LiveSchema) {
        let arrow = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
            Field::new("age", DataType::Int32, true),
        ]));
        let live = LiveSchema::new_strict(arrow.clone(), vec!["id".into()]);
        (arrow, live)
    }

    fn test_batch() -> RecordBatch {
        let (schema, _) = test_schema();
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(StringArray::from(vec!["alice", "bob"])),
                Arc::new(Int32Array::from(vec![Some(30), Some(25)])),
            ],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn select_projects_columns() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let select = Arc::new(SelectNode::columns(data_node, &["id", "name"], &live));

        let mut stream = select.get_stream(Context::default());
        let batch = stream.next().await.unwrap();
        assert_eq!(batch.len(), 1);

        let node = batch[0].final_result();
        let change = node.as_record_change().unwrap();
        assert_eq!(change.change_type, ChangeType::Add);
        assert_eq!(change.records.num_columns(), 2);
        assert_eq!(change.records.schema().field(0).name(), "id");
        assert_eq!(change.records.schema().field(1).name(), "name");
    }

    #[tokio::test]
    async fn select_preserves_strictness_when_keys_survive() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        // Projecting id + name: key column "id" survives
        let select = Arc::new(SelectNode::columns(data_node, &["id", "name"], &live));

        let events: Vec<_> = select.get_stream(Context::default()).collect().await;
        let node = events[0][0].final_result();
        let change = node.as_record_change().unwrap();
        assert!(change.is_strict);
    }

    #[tokio::test]
    async fn select_loses_strictness_when_keys_dropped() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        // Projecting name + age: key column "id" is dropped
        let select = Arc::new(SelectNode::columns(data_node, &["name", "age"], &live));

        // The output schema should not be strict
        assert!(!select.output_schema.is_strict);
    }
}
