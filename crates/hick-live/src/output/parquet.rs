//! Parquet file output for change streams.
//!
//! Two modes:
//! - **Events**: append-only event log with `_change_type` column
//! - **Materialized**: maintains current state by applying Add/Remove events

use std::sync::Arc;

use arrow::array::{Array, AsArray, BooleanArray};
use arrow::compute::filter_record_batch;
use arrow::datatypes::{DataType, Int32Type, Int64Type};
use arrow::record_batch::RecordBatch;
use futures::StreamExt;
use hick_flow::node::{BoxStream, Node, NodeTrace, NodeValue};
use hick_flow::Context;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;

use crate::changes::ChangeType;
use crate::node::NodeExt;
use crate::schema::LiveSchema;

/// How the parquet file represents the change stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParquetMode {
    /// Append-only event log: each change is written with a `_change_type` column.
    Events,
    /// Materialized view: file reflects accumulated current state.
    Materialized,
}

/// Output node that writes a change stream to a Parquet file.
///
/// This node consumes the upstream `NodeTrace` stream, extracts
/// `RecordChange` events, and writes them to a parquet file.
pub struct ParquetFileNode {
    pub path: String,
    pub source: Arc<dyn Node>,
    pub schema: LiveSchema,
    pub mode: ParquetMode,
}

impl ParquetFileNode {
    pub fn new(
        path: impl Into<String>,
        source: Arc<dyn Node>,
        schema: LiveSchema,
        mode: ParquetMode,
    ) -> Self {
        Self {
            path: path.into(),
            source,
            schema,
            mode,
        }
    }
}

impl Node for ParquetFileNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let source_stream = self.source.clone().get_stream(context);
        let path = self.path.clone();
        let mode = self.mode;
        let schema = self.schema.clone();
        let _me: Arc<dyn Node> = self;

        Box::pin(async_stream::stream! {
            let mut all_batches: Vec<RecordBatch> = Vec::new();
            // For materialized mode: accumulated state keyed by row identity
            let mut materialized: Vec<RecordBatch> = Vec::new();

            let mut source_stream = std::pin::pin!(source_stream);
            while let Some(traces) = source_stream.next().await {
                for trace in &traces {
                    if let Some(change) = trace.final_result().as_record_change() {
                        match mode {
                            ParquetMode::Events => {
                                all_batches.push(change.with_change_type_column());
                            }
                            ParquetMode::Materialized => {
                                match change.change_type {
                                    ChangeType::Add => {
                                        materialized.push(change.records.clone());
                                    }
                                    ChangeType::Remove => {
                                        if change.is_strict {
                                            remove_rows(&mut materialized, &change.records, &schema);
                                        } else {
                                            log::warn!(
                                                "Non-strict Remove in materialized mode for {}: rows may not be correctly removed",
                                                path
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Yield the traces as-is (parquet writing is a side effect)
                yield traces;
            }

            // Write the parquet file after the stream completes
            let batches_to_write = match mode {
                ParquetMode::Events => &all_batches,
                ParquetMode::Materialized => &materialized,
            };

            if let Err(e) = write_parquet(&path, batches_to_write) {
                log::error!("Failed to write parquet file {}: {}", path, e);
            }
        })
    }

    fn node_value(&self) -> NodeValue {
        NodeValue::None
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

/// Write a sequence of RecordBatches to a parquet file.
fn write_parquet(path: &str, batches: &[RecordBatch]) -> anyhow::Result<()> {
    if batches.is_empty() {
        return Ok(());
    }

    let schema = batches[0].schema();
    let props = WriterProperties::builder().build();

    let file = std::fs::File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;

    for batch in batches {
        writer.write(batch)?;
    }

    writer.close()?;
    Ok(())
}

/// Remove rows from the materialized state that match the given batch.
///
/// Matches by key columns: for each row to remove, finds and removes the
/// first matching row in the accumulated state.
fn remove_rows(state: &mut Vec<RecordBatch>, to_remove: &RecordBatch, schema: &LiveSchema) {
    if schema.key_columns.is_empty() {
        return;
    }

    // Build a set of keys to remove
    let key_col_indices: Vec<usize> = schema
        .key_columns
        .iter()
        .filter_map(|name| schema.arrow_schema.column_with_name(name).map(|(i, _)| i))
        .collect();

    for row in 0..to_remove.num_rows() {
        let key = row_key(to_remove, &key_col_indices, row);

        // Find and remove the first matching row in state
        let mut removed = false;
        for batch in state.iter_mut() {
            if removed {
                break;
            }
            let n = batch.num_rows();
            let mut keep = vec![true; n];
            for i in 0..n {
                if !removed && row_key(batch, &key_col_indices, i) == key {
                    keep[i] = false;
                    removed = true;
                }
            }
            if removed {
                let mask = BooleanArray::from(keep);
                if let Ok(filtered) = filter_record_batch(batch, &mask) {
                    *batch = filtered;
                }
            }
        }
    }

    // Remove empty batches
    state.retain(|b| b.num_rows() > 0);
}

/// Extract a string key from the given columns of a row.
fn row_key(batch: &RecordBatch, key_col_indices: &[usize], row: usize) -> String {
    let mut key = String::new();
    for (i, &col_idx) in key_col_indices.iter().enumerate() {
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
                key.push_str(&format!("{:?}", col.slice(row, 1)));
            }
        }
    }
    key
}

/// Read a parquet file back into RecordBatches (used in tests).
pub fn read_parquet(path: &str) -> anyhow::Result<Vec<RecordBatch>> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    let file = std::fs::File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let reader = builder.build()?;
    let batches: Result<Vec<_>, _> = reader.collect();
    Ok(batches?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::RecordChange;
    use crate::sources::memory::MemorySource;
    use crate::StreamingStructuredDataNode;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{Field, Schema};

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
    async fn events_mode_writes_all_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.parquet");

        let (_, live) = test_schema();
        let source = Arc::new(MemorySource::new(live.clone(), test_batch(&[1, 2], &["a", "b"])));
        source.push(RecordChange::remove_strict(test_batch(&[1], &["a"])));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let parquet_node = Arc::new(ParquetFileNode::new(
            path.to_str().unwrap(),
            data_node,
            live,
            ParquetMode::Events,
        ));

        // Consume the stream to trigger writing
        let _: Vec<_> = parquet_node.get_stream(Context::default()).collect().await;

        // Read back and verify
        let batches = read_parquet(path.to_str().unwrap()).unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 3); // 2 add + 1 remove

        // Check _change_type column exists
        assert!(batches[0].schema().column_with_name("_change_type").is_some());
    }

    #[tokio::test]
    async fn materialized_mode_applies_removes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("materialized.parquet");

        let (_, live) = test_schema();
        let source = Arc::new(MemorySource::new(live.clone(), test_batch(&[1, 2, 3], &["a", "b", "c"])));
        source.push(RecordChange::remove_strict(test_batch(&[2], &["b"])));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let parquet_node = Arc::new(ParquetFileNode::new(
            path.to_str().unwrap(),
            data_node,
            live,
            ParquetMode::Materialized,
        ));

        // Consume stream
        let _: Vec<_> = parquet_node.get_stream(Context::default()).collect().await;

        // Read back — should have 2 rows (id=1, id=3)
        let batches = read_parquet(path.to_str().unwrap()).unwrap();
        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
        assert_eq!(total_rows, 2);

        // No _change_type column in materialized mode
        assert!(batches[0].schema().column_with_name("_change_type").is_none());
    }
}
