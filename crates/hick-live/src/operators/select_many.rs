//! SelectMany (flat-map) operator for incremental collections.
//!
//! For each row in an Add event, subscribes to an inner stream produced by
//! a function. For each row in a Remove event, unsubscribes the corresponding
//! inner stream. All inner events are merged into a single output stream.

use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use futures::StreamExt;
use hick_flow::node::{BoxStream, Node, NodeTrace, NodeValue};
use hick_flow::Context;

use crate::changes::{ChangeStream, ChangeType, RecordChange};
use crate::node::{NodeExt, RecordChangeNode};
use crate::schema::LiveSchema;

/// Function that produces an inner ChangeStream for each row batch.
pub type InnerFn = Arc<dyn Fn(&RecordBatch) -> ChangeStream + Send + Sync>;

/// Flat-maps upstream change events through an inner stream function.
///
/// For each Add batch: invoke `inner_fn` and collect its output.
/// For each Remove batch: produce corresponding Remove output.
/// This is a simplified version that processes inner streams eagerly
/// (collects them inline rather than maintaining persistent subscriptions).
pub struct SelectManyNode {
    pub source: Arc<dyn Node>,
    pub inner_fn: InnerFn,
    pub output_schema: LiveSchema,
}

impl SelectManyNode {
    pub fn new(source: Arc<dyn Node>, inner_fn: InnerFn, output_schema: LiveSchema) -> Self {
        Self {
            source,
            inner_fn,
            output_schema,
        }
    }
}

impl Node for SelectManyNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let source_stream = self.source.clone().get_stream(context);
        let inner_fn = self.inner_fn.clone();
        let me: Arc<dyn Node> = self;

        Box::pin(source_stream.then(move |traces| {
            let inner_fn = inner_fn.clone();
            let me = me.clone();
            async move {
                let mut result_traces = Vec::new();
                for trace in traces {
                    if let Some(change) = trace.final_result().as_record_change() {
                        match change.change_type {
                            ChangeType::Add => {
                                // For Add events, invoke inner_fn and collect all output
                                let inner_stream = inner_fn(&change.records);
                                let inner_changes: Vec<RecordChange> =
                                    inner_stream.collect().await;
                                for inner_change in inner_changes {
                                    let new_node: Arc<dyn Node> =
                                        Arc::new(RecordChangeNode::new(inner_change));
                                    result_traces.push(trace.transform(new_node, me.clone()));
                                }
                            }
                            ChangeType::Remove => {
                                // For Remove events, pass through as-is
                                // (inner stream cleanup would happen in a stateful version)
                                let new_change = RecordChange::new(
                                    ChangeType::Remove,
                                    change.records.clone(),
                                    change.is_strict,
                                );
                                let new_node: Arc<dyn Node> =
                                    Arc::new(RecordChangeNode::new(new_change));
                                result_traces.push(trace.transform(new_node, me.clone()));
                            }
                        }
                    } else {
                        result_traces.push(trace.add_to_trace(me.clone()));
                    }
                }
                result_traces
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
    use crate::sources::memory::MemorySource;
    use crate::StreamingStructuredDataNode;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use futures::stream;
    use futures::StreamExt;

    fn input_schema() -> (Arc<Schema>, LiveSchema) {
        let arrow = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("count", DataType::Int32, false),
        ]));
        let live = LiveSchema::new_strict(arrow.clone(), vec!["id".into()]);
        (arrow, live)
    }

    fn output_arrow_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("parent_id", DataType::Int32, false),
            Field::new("item", DataType::Utf8, false),
        ]))
    }

    fn input_batch(ids: &[i32], counts: &[i32]) -> RecordBatch {
        let (schema, _) = input_schema();
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(ids.to_vec())),
                Arc::new(Int32Array::from(counts.to_vec())),
            ],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn select_many_expands_rows() {
        let (_, live) = input_schema();

        let source = Arc::new(MemorySource::new(live.clone(), input_batch(&[1, 2], &[2, 1])));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let out_schema = output_arrow_schema();
        let output_live = LiveSchema::new_strict(out_schema.clone(), vec!["parent_id".into()]);

        // inner_fn: for each row, produce `count` items
        let inner_fn: InnerFn = Arc::new(move |batch| {
            let ids = batch
                .column(0)
                .as_any()
                .downcast_ref::<Int32Array>()
                .unwrap();
            let counts = batch
                .column(1)
                .as_any()
                .downcast_ref::<Int32Array>()
                .unwrap();

            let out_schema = Arc::new(Schema::new(vec![
                Field::new("parent_id", DataType::Int32, false),
                Field::new("item", DataType::Utf8, false),
            ]));

            let mut all_ids = Vec::new();
            let mut all_items = Vec::new();
            for i in 0..batch.num_rows() {
                let id = ids.value(i);
                let count = counts.value(i);
                for j in 0..count {
                    all_ids.push(id);
                    all_items.push(format!("item_{j}"));
                }
            }

            let result = RecordBatch::try_new(
                out_schema,
                vec![
                    Arc::new(Int32Array::from(all_ids)),
                    Arc::new(StringArray::from(all_items)),
                ],
            )
            .unwrap();

            Box::pin(stream::once(async move { RecordChange::add(result) }))
        });

        let select_many = Arc::new(SelectManyNode::new(data_node, inner_fn, output_live));

        let events: Vec<_> = select_many.get_stream(Context::default()).collect().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].len(), 1);

        let node = events[0][0].final_result();
        let change = node.as_record_change().unwrap();
        assert_eq!(change.change_type, ChangeType::Add);
        // id=1 has count=2, id=2 has count=1 → 3 output rows
        assert_eq!(change.num_rows(), 3);
    }
}
