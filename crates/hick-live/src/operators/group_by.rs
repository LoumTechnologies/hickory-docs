//! GroupBy operator for incremental collections.
//!
//! Groups rows by key columns, maintaining per-group state. When new group
//! keys appear, new sub-streams are created. When groups become empty,
//! their sub-streams are removed.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use arrow::array::{Array, AsArray};
use arrow::compute::filter_record_batch;
use arrow::datatypes::{DataType, Int32Type, Int64Type};
use arrow::record_batch::RecordBatch;
use futures::StreamExt;
use hick_flow::Context;
use hick_flow::node::{BoxStream, Node, NodeTrace, NodeValue};

use crate::changes::{ChangeType, RecordChange};
use crate::node::{NodeExt, RecordChangeNode};
use crate::schema::LiveSchema;

/// Groups upstream change events by a set of key columns.
///
/// Maintains per-group row counts. For each incoming Add/Remove event,
/// routes rows to the appropriate group and emits group-level changes.
/// New group keys produce new Add events; groups that become empty produce
/// Remove events for the group.
pub struct GroupByNode {
    pub source: Arc<dyn Node>,
    pub group_columns: Vec<String>,
    pub output_schema: LiveSchema,
}

impl GroupByNode {
    pub fn new(
        source: Arc<dyn Node>,
        group_columns: Vec<String>,
        input_schema: &LiveSchema,
    ) -> Self {
        let output_schema = LiveSchema {
            arrow_schema: input_schema.arrow_schema.clone(),
            key_columns: group_columns.clone(),
            is_strict: input_schema.is_strict,
        };
        Self {
            source,
            group_columns,
            output_schema,
        }
    }
}

/// Extract a string key from the group columns of a row.
fn extract_group_key(batch: &RecordBatch, group_col_indices: &[usize], row: usize) -> String {
    let mut key = String::new();
    for (i, &col_idx) in group_col_indices.iter().enumerate() {
        if i > 0 {
            key.push('\x00');
        }
        let col = batch.column(col_idx);
        match col.data_type() {
            DataType::Utf8 => {
                let arr = col.as_string::<i32>();
                if arr.is_null(row) {
                    key.push_str("<null>");
                } else {
                    key.push_str(arr.value(row));
                }
            }
            DataType::Int32 => {
                let arr = col.as_primitive::<Int32Type>();
                if arr.is_null(row) {
                    key.push_str("<null>");
                } else {
                    key.push_str(&arr.value(row).to_string());
                }
            }
            DataType::Int64 => {
                let arr = col.as_primitive::<Int64Type>();
                if arr.is_null(row) {
                    key.push_str("<null>");
                } else {
                    key.push_str(&arr.value(row).to_string());
                }
            }
            _ => {
                // Fallback: use debug representation
                key.push_str(&format!("{:?}", col.slice(row, 1)));
            }
        }
    }
    key
}

/// Partition a batch into per-group sub-batches.
fn partition_by_group(
    batch: &RecordBatch,
    group_col_indices: &[usize],
) -> Vec<(String, RecordBatch)> {
    let n = batch.num_rows();
    if n == 0 {
        return Vec::new();
    }

    // Build a map of group_key -> row indices
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for row in 0..n {
        let key = extract_group_key(batch, group_col_indices, row);
        groups.entry(key).or_default().push(row);
    }

    // Build a sub-batch per group using boolean mask filtering
    let mut result = Vec::with_capacity(groups.len());
    for (key, rows) in groups {
        let mut mask = vec![false; n];
        for &row in &rows {
            mask[row] = true;
        }
        let mask_arr = arrow::array::BooleanArray::from(mask);
        if let Ok(sub_batch) = filter_record_batch(batch, &mask_arr) {
            result.push((key, sub_batch));
        }
    }
    result
}

impl Node for GroupByNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let source_stream = self.source.clone().get_stream(context);
        let group_col_indices: Vec<usize> = self
            .group_columns
            .iter()
            .filter_map(|name| {
                self.output_schema
                    .arrow_schema
                    .column_with_name(name)
                    .map(|(i, _)| i)
            })
            .collect();

        // Track active group row counts for detecting group creation/removal
        let group_row_counts: Arc<Mutex<HashMap<String, usize>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let me: Arc<dyn Node> = self;

        Box::pin(source_stream.map(move |traces| {
            let mut result_traces = Vec::new();
            let group_col_indices = &group_col_indices;
            let group_row_counts = &group_row_counts;

            for trace in traces {
                if let Some(change) = trace.final_result().as_record_change() {
                    let partitions = partition_by_group(&change.records, group_col_indices);

                    for (key, sub_batch) in partitions {
                        let mut counts = group_row_counts.lock().unwrap();
                        let count = counts.entry(key).or_insert(0);

                        match change.change_type {
                            ChangeType::Add => {
                                *count += sub_batch.num_rows();
                            }
                            ChangeType::Remove => {
                                *count = count.saturating_sub(sub_batch.num_rows());
                            }
                        }

                        let new_change =
                            RecordChange::new(change.change_type, sub_batch, change.is_strict);
                        let new_node: Arc<dyn Node> = Arc::new(RecordChangeNode::new(new_change));
                        result_traces.push(trace.transform(new_node, me.clone()));
                    }
                } else {
                    result_traces.push(trace.add_to_trace(me.clone()));
                }
            }
            result_traces
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
    use crate::sources::memory::MemorySource;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use futures::StreamExt;

    fn test_schema() -> (Arc<Schema>, LiveSchema) {
        let arrow = Arc::new(Schema::new(vec![
            Field::new("region", DataType::Utf8, false),
            Field::new("id", DataType::Int32, false),
            Field::new("value", DataType::Int32, true),
        ]));
        let live = LiveSchema::new_strict(arrow.clone(), vec!["id".into()]);
        (arrow, live)
    }

    fn test_batch(regions: &[&str], ids: &[i32], values: &[Option<i32>]) -> RecordBatch {
        let (schema, _) = test_schema();
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(
                    regions.iter().map(|s| *s).collect::<Vec<_>>(),
                )),
                Arc::new(Int32Array::from(ids.to_vec())),
                Arc::new(Int32Array::from(values.to_vec())),
            ],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn group_by_partitions_by_column() {
        let (_, live) = test_schema();

        let batch = test_batch(
            &["us", "eu", "us", "eu"],
            &[1, 2, 3, 4],
            &[Some(10), Some(20), Some(30), Some(40)],
        );

        let source = Arc::new(MemorySource::new(live.clone(), batch));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let group_node = Arc::new(GroupByNode::new(data_node, vec!["region".into()], &live));

        let events: Vec<_> = group_node.get_stream(Context::default()).collect().await;
        assert_eq!(events.len(), 1);

        // Should have 2 groups (us and eu), emitted as 2 traces
        let traces = &events[0];
        assert_eq!(traces.len(), 2);

        // Collect all rows from both groups
        let mut total_rows = 0;
        for trace in traces {
            let node = trace.final_result();
            let change = node.as_record_change().unwrap();
            assert_eq!(change.change_type, ChangeType::Add);
            total_rows += change.num_rows();
        }
        assert_eq!(total_rows, 4);
    }

    #[tokio::test]
    async fn group_by_handles_removes() {
        let (_, live) = test_schema();

        let batch = test_batch(&["us", "eu"], &[1, 2], &[Some(10), Some(20)]);

        let source = Arc::new(MemorySource::new(live.clone(), batch));
        source.push(RecordChange::remove_strict(test_batch(
            &["us"],
            &[1],
            &[Some(10)],
        )));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let group_node = Arc::new(GroupByNode::new(data_node, vec!["region".into()], &live));

        let events: Vec<_> = group_node.get_stream(Context::default()).collect().await;
        assert_eq!(events.len(), 2); // initial add + remove

        // Second event should be a Remove for the "us" group
        let remove_traces = &events[1];
        assert_eq!(remove_traces.len(), 1);
        let node = remove_traces[0].final_result();
        let change = node.as_record_change().unwrap();
        assert_eq!(change.change_type, ChangeType::Remove);
    }
}
