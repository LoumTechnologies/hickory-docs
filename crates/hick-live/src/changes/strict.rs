//! Strictness enforcement for change streams.
//!
//! `make_strict()` wraps a `ChangeStream` and buffers all active rows so that
//! Remove events always carry full row data, even when the upstream only
//! provides key columns.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{Array, AsArray, BooleanArray};
use arrow::compute::filter_record_batch;
use arrow::datatypes::{DataType, Int32Type, Int64Type};
use arrow::record_batch::RecordBatch;
use futures::StreamExt;

use crate::changes::{ChangeStream, ChangeType, RecordChange};
use crate::schema::LiveSchema;

/// Wrap a `ChangeStream` to guarantee strict Remove events.
///
/// Maintains an in-memory buffer of all active rows. When a non-strict Remove
/// arrives (key columns only), the full row data is looked up from the buffer
/// and emitted as a strict Remove.
///
/// O(n) memory where n is the number of active rows.
pub fn make_strict(source: ChangeStream, schema: &LiveSchema) -> ChangeStream {
    let key_col_indices: Vec<usize> = schema
        .key_columns
        .iter()
        .filter_map(|name| schema.arrow_schema.column_with_name(name).map(|(i, _)| i))
        .collect();

    let full_schema = schema.arrow_schema.clone();

    Box::pin(async_stream::stream! {
        // Buffer: row key -> (batch index, row index within batch)
        let mut active_batches: Vec<RecordBatch> = Vec::new();

        let mut source = std::pin::pin!(source);
        while let Some(change) = source.next().await {
            match change.change_type {
                ChangeType::Add => {
                    // Buffer the new rows
                    active_batches.push(change.records.clone());
                    yield RecordChange::add(change.records);
                }
                ChangeType::Remove => {
                    if change.is_strict {
                        // Already strict — remove from buffer and pass through
                        remove_from_buffer(&mut active_batches, &change.records, &key_col_indices);
                        yield RecordChange::remove_strict(change.records);
                    } else {
                        // Non-strict: look up full rows from buffer
                        let full_rows = lookup_and_remove(
                            &mut active_batches,
                            &change.records,
                            &key_col_indices,
                            &full_schema,
                        );
                        if let Some(batch) = full_rows {
                            yield RecordChange::remove_strict(batch);
                        }
                    }
                }
            }
        }
    })
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

/// Remove matching rows from the buffer (used for strict removes).
fn remove_from_buffer(
    buffer: &mut Vec<RecordBatch>,
    to_remove: &RecordBatch,
    key_col_indices: &[usize],
) {
    for row in 0..to_remove.num_rows() {
        let key = row_key(to_remove, key_col_indices, row);
        let mut removed = false;
        for batch in buffer.iter_mut() {
            if removed {
                break;
            }
            let n = batch.num_rows();
            let mut keep = vec![true; n];
            for i in 0..n {
                if !removed && row_key(batch, key_col_indices, i) == key {
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
    buffer.retain(|b| b.num_rows() > 0);
}

/// Look up full rows from the buffer by key, remove them, and return a batch
/// with the full schema.
fn lookup_and_remove(
    buffer: &mut Vec<RecordBatch>,
    keys_batch: &RecordBatch,
    key_col_indices: &[usize],
    _full_schema: &Arc<arrow::datatypes::Schema>,
) -> Option<RecordBatch> {
    // Build a map of key -> row indices to find in the buffer
    let mut keys_to_find: HashMap<String, usize> = HashMap::new();
    for row in 0..keys_batch.num_rows() {
        let key = row_key(keys_batch, key_col_indices, row);
        keys_to_find.insert(key, row);
    }

    let mut found_batches: Vec<RecordBatch> = Vec::new();

    for batch in buffer.iter_mut() {
        if keys_to_find.is_empty() {
            break;
        }
        let n = batch.num_rows();
        let mut keep = vec![true; n];
        let mut found_mask = vec![false; n];

        for i in 0..n {
            let key = row_key(batch, key_col_indices, i);
            if keys_to_find.remove(&key).is_some() {
                keep[i] = false;
                found_mask[i] = true;
            }
        }

        // Extract found rows
        let found_bool = BooleanArray::from(found_mask);
        if let Ok(found) = filter_record_batch(batch, &found_bool) {
            if found.num_rows() > 0 {
                found_batches.push(found);
            }
        }

        // Remove found rows from buffer
        let keep_bool = BooleanArray::from(keep);
        if let Ok(filtered) = filter_record_batch(batch, &keep_bool) {
            *batch = filtered;
        }
    }

    buffer.retain(|b| b.num_rows() > 0);

    if found_batches.is_empty() {
        return None;
    }

    // Concatenate all found batches
    let schema = found_batches[0].schema();
    arrow::compute::concat_batches(&schema, &found_batches).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{Field, Schema};
    use futures::stream;
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

    /// Key-only batch (simulating a non-strict Remove).
    fn key_only_batch(ids: &[i32]) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
        ]));
        RecordBatch::try_new(schema, vec![Arc::new(Int32Array::from(ids.to_vec()))])
            .unwrap()
    }

    #[tokio::test]
    async fn make_strict_passes_through_strict_removes() {
        let (_, live) = test_schema();
        let events = vec![
            RecordChange::add(test_batch(&[1, 2], &["a", "b"])),
            RecordChange::remove_strict(test_batch(&[1], &["a"])),
        ];
        let source: ChangeStream = Box::pin(stream::iter(events));
        let strict_stream = make_strict(source, &live);

        let results: Vec<_> = strict_stream.collect().await;
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].change_type, ChangeType::Add);
        assert_eq!(results[1].change_type, ChangeType::Remove);
        assert!(results[1].is_strict);
        assert_eq!(results[1].num_rows(), 1);
        // Full row data should be present
        assert_eq!(results[1].records.num_columns(), 2);
    }

    #[tokio::test]
    async fn make_strict_materializes_non_strict_removes() {
        let (_, mut live) = test_schema();
        live.is_strict = false;

        let events = vec![
            RecordChange::add(test_batch(&[1, 2, 3], &["alice", "bob", "charlie"])),
            RecordChange::remove_keys_only(key_only_batch(&[2])),
        ];
        let source: ChangeStream = Box::pin(stream::iter(events));
        let strict_stream = make_strict(source, &live);

        let results: Vec<_> = strict_stream.collect().await;
        assert_eq!(results.len(), 2);

        // The remove should now be strict with full row data
        let remove = &results[1];
        assert_eq!(remove.change_type, ChangeType::Remove);
        assert!(remove.is_strict);
        assert_eq!(remove.num_rows(), 1);
        assert_eq!(remove.records.num_columns(), 2);

        // Verify we got the right row (id=2, name="bob")
        let ids = remove
            .records
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(ids.value(0), 2);

        let names = remove
            .records
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(names.value(0), "bob");
    }

    #[tokio::test]
    async fn make_strict_handles_multiple_removes() {
        let (_, mut live) = test_schema();
        live.is_strict = false;

        let events = vec![
            RecordChange::add(test_batch(&[1, 2, 3], &["a", "b", "c"])),
            RecordChange::remove_keys_only(key_only_batch(&[1])),
            RecordChange::remove_keys_only(key_only_batch(&[3])),
        ];
        let source: ChangeStream = Box::pin(stream::iter(events));
        let strict_stream = make_strict(source, &live);

        let results: Vec<_> = strict_stream.collect().await;
        assert_eq!(results.len(), 3);

        // First remove: id=1
        assert!(results[1].is_strict);
        let ids = results[1]
            .records
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(ids.value(0), 1);

        // Second remove: id=3
        assert!(results[2].is_strict);
        let ids = results[2]
            .records
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(ids.value(0), 3);
    }
}
