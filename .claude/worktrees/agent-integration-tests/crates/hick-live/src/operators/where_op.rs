//! Where (filter) operator for incremental collections.
//!
//! Filters rows from a `ChangeStream` using a predicate function.
//! Preserves strictness: if input is strict, output is strict.

use std::sync::Arc;

use arrow::array::BooleanArray;
use arrow::compute::filter_record_batch;
use arrow::record_batch::RecordBatch;
use futures::StreamExt;
use hick_flow::Context;
use hick_flow::node::{BoxStream, Node, NodeTrace, NodeValue};

use crate::changes::RecordChange;
use crate::node::{NodeExt, RecordChangeNode};
use crate::schema::LiveSchema;

/// Predicate function type: takes a batch and returns a boolean mask.
pub type Predicate = Arc<dyn Fn(&RecordBatch) -> BooleanArray + Send + Sync>;

/// Filters rows from upstream using a predicate applied per-batch.
///
/// On Add: filter rows, emit Add with matching rows.
/// On Remove: filter rows, emit Remove with matching rows.
/// Strictness is preserved from input.
pub struct WhereNode {
    pub source: Arc<dyn Node>,
    pub predicate: Predicate,
    pub schema: LiveSchema,
}

impl WhereNode {
    pub fn new(source: Arc<dyn Node>, predicate: Predicate, schema: LiveSchema) -> Self {
        Self {
            source,
            predicate,
            schema,
        }
    }
}

impl Node for WhereNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let source_stream = self.source.clone().get_stream(context);
        let predicate = self.predicate.clone();
        let me: Arc<dyn Node> = self;

        Box::pin(source_stream.filter_map(move |traces| {
            let predicate = predicate.clone();
            let me = me.clone();
            async move {
                let mut result_traces = Vec::new();
                for trace in traces {
                    if let Some(change) = trace.final_result().as_record_change() {
                        let mask = predicate(&change.records);
                        match filter_record_batch(&change.records, &mask) {
                            Ok(filtered) if filtered.num_rows() > 0 => {
                                let new_change = RecordChange::new(
                                    change.change_type,
                                    filtered,
                                    change.is_strict,
                                );
                                let new_node: Arc<dyn Node> =
                                    Arc::new(RecordChangeNode::new(new_change));
                                result_traces.push(trace.transform(new_node, me.clone()));
                            }
                            _ => {
                                // No matching rows — skip this trace
                            }
                        }
                    } else {
                        // Non-structured trace — pass through
                        result_traces.push(trace.add_to_trace(me.clone()));
                    }
                }
                if result_traces.is_empty() {
                    None
                } else {
                    Some(result_traces)
                }
            }
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
    use arrow::array::{AsArray, Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use futures::StreamExt;

    fn test_schema() -> (Arc<Schema>, LiveSchema) {
        let arrow = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));
        let live = LiveSchema::new_strict(arrow.clone(), vec!["id".into()]);
        (arrow, live)
    }

    fn test_batch(ids: &[i32], names: &[&str]) -> RecordBatch {
        let (schema, _) = test_schema();
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(ids.to_vec())),
                Arc::new(StringArray::from(
                    names.iter().map(|s| *s).collect::<Vec<_>>(),
                )),
            ],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn where_filters_rows() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(
            live.clone(),
            test_batch(&[1, 2, 3], &["alice", "bob", "charlie"]),
        ));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        // Filter: id > 1
        let predicate: Predicate = Arc::new(|batch| {
            let ids = batch
                .column(0)
                .as_primitive::<arrow::datatypes::Int32Type>();
            BooleanArray::from(ids.iter().map(|v| v.map(|x| x > 1)).collect::<Vec<_>>())
        });

        let where_node = Arc::new(WhereNode::new(data_node, predicate, live));

        let mut stream = where_node.get_stream(Context::default());
        let batch = stream.next().await.unwrap();
        assert_eq!(batch.len(), 1);

        let node = batch[0].final_result();
        let change = node.as_record_change().unwrap();
        assert_eq!(change.change_type, ChangeType::Add);
        assert_eq!(change.num_rows(), 2);

        let ids = change
            .records
            .column(0)
            .as_primitive::<arrow::datatypes::Int32Type>();
        assert_eq!(ids.value(0), 2);
        assert_eq!(ids.value(1), 3);
    }

    #[tokio::test]
    async fn where_skips_empty_filtered_result() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(
            live.clone(),
            test_batch(&[1, 2], &["alice", "bob"]),
        ));
        // Push a change that will be entirely filtered out
        source.push(RecordChange::add(test_batch(&[3], &["charlie"])));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        // Filter: id > 10 (nothing matches)
        let predicate: Predicate = Arc::new(|batch| {
            let ids = batch
                .column(0)
                .as_primitive::<arrow::datatypes::Int32Type>();
            BooleanArray::from(ids.iter().map(|v| v.map(|x| x > 10)).collect::<Vec<_>>())
        });

        let where_node = Arc::new(WhereNode::new(data_node, predicate, live));

        let mut stream = where_node.get_stream(Context::default());
        // Both events should be filtered out entirely (empty batches skipped)
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn where_preserves_remove_strictness() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(
            live.clone(),
            test_batch(&[1, 2], &["alice", "bob"]),
        ));
        source.push(RecordChange::remove_strict(test_batch(&[1], &["alice"])));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        // Filter: id == 1
        let predicate: Predicate = Arc::new(|batch| {
            let ids = batch
                .column(0)
                .as_primitive::<arrow::datatypes::Int32Type>();
            BooleanArray::from(ids.iter().map(|v| v.map(|x| x == 1)).collect::<Vec<_>>())
        });

        let where_node = Arc::new(WhereNode::new(data_node, predicate, live));

        let events: Vec<_> = where_node.get_stream(Context::default()).collect().await;
        assert_eq!(events.len(), 2);

        // First: Add with id=1
        let add_node = events[0][0].final_result();
        let add = add_node.as_record_change().unwrap();
        assert_eq!(add.change_type, ChangeType::Add);
        assert!(add.is_strict);
        assert_eq!(add.num_rows(), 1);

        // Second: Remove with id=1 (strict preserved)
        let remove_node = events[1][0].final_result();
        let remove = remove_node.as_record_change().unwrap();
        assert_eq!(remove.change_type, ChangeType::Remove);
        assert!(remove.is_strict);
    }
}
