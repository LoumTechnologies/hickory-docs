//! Core change types for incremental collection processing.
//!
//! Mirrors LiveLinq's `CollectionChangeType` pattern: Add and Remove only.
//! Updates are expressed as Remove + Add (same as the C# original).

pub mod strict;

use std::pin::Pin;
use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::Stream;

/// Whether rows are being added to or removed from the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Add,
    Remove,
}

/// An Arrow-based change event carrying a batch of rows being added or removed.
#[derive(Debug, Clone)]
pub struct RecordChange {
    /// Whether these rows are being added or removed.
    pub change_type: ChangeType,
    /// The rows involved in this change.
    pub records: RecordBatch,
    /// When true, Remove events carry full row data (not just key columns).
    pub is_strict: bool,
}

impl RecordChange {
    /// Create a new change event.
    pub fn new(change_type: ChangeType, records: RecordBatch, is_strict: bool) -> Self {
        Self {
            change_type,
            records,
            is_strict,
        }
    }

    /// Create an Add event with full row data (always strict).
    pub fn add(records: RecordBatch) -> Self {
        Self::new(ChangeType::Add, records, true)
    }

    /// Create a strict Remove event (carries full row data).
    pub fn remove_strict(records: RecordBatch) -> Self {
        Self::new(ChangeType::Remove, records, true)
    }

    /// Create a non-strict Remove event (may carry only key columns).
    pub fn remove_keys_only(records: RecordBatch) -> Self {
        Self::new(ChangeType::Remove, records, false)
    }

    /// Number of rows in this change.
    pub fn num_rows(&self) -> usize {
        self.records.num_rows()
    }

    /// Add a `_change_type` column to the batch for event-log output.
    pub fn with_change_type_column(&self) -> RecordBatch {
        let label = match self.change_type {
            ChangeType::Add => "add",
            ChangeType::Remove => "remove",
        };
        let n = self.records.num_rows();
        let change_col = Arc::new(StringArray::from(vec![label; n]));

        let mut fields: Vec<Arc<Field>> = self.records.schema().fields().iter().cloned().collect();
        fields.push(Arc::new(Field::new("_change_type", DataType::Utf8, false)));

        let mut columns: Vec<Arc<dyn arrow::array::Array>> =
            self.records.columns().iter().cloned().collect();
        columns.push(change_col);

        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(schema, columns).expect("schema mismatch adding _change_type column")
    }
}

/// A stream of incremental change events.
///
/// Every `ChangeStream` starts with an `Add` containing the initial state,
/// followed by subsequent Add/Remove events as the collection evolves.
pub type ChangeStream = Pin<Box<dyn Stream<Item = RecordChange> + Send + 'static>>;

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Int32Array;
    use arrow::datatypes::Field;

    fn test_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]))
    }

    fn test_batch(ids: &[i32], names: &[&str]) -> RecordBatch {
        RecordBatch::try_new(
            test_schema(),
            vec![
                Arc::new(Int32Array::from(ids.to_vec())),
                Arc::new(StringArray::from(
                    names.iter().map(|s| *s).collect::<Vec<_>>(),
                )),
            ],
        )
        .unwrap()
    }

    #[test]
    fn add_event_is_strict() {
        let batch = test_batch(&[1, 2], &["a", "b"]);
        let change = RecordChange::add(batch);
        assert_eq!(change.change_type, ChangeType::Add);
        assert!(change.is_strict);
        assert_eq!(change.num_rows(), 2);
    }

    #[test]
    fn remove_strict_carries_full_data() {
        let batch = test_batch(&[1], &["a"]);
        let change = RecordChange::remove_strict(batch);
        assert_eq!(change.change_type, ChangeType::Remove);
        assert!(change.is_strict);
    }

    #[test]
    fn remove_keys_only_is_non_strict() {
        let batch = test_batch(&[1], &["a"]);
        let change = RecordChange::remove_keys_only(batch);
        assert_eq!(change.change_type, ChangeType::Remove);
        assert!(!change.is_strict);
    }

    #[test]
    fn with_change_type_column_adds_column() {
        let batch = test_batch(&[1, 2], &["a", "b"]);
        let change = RecordChange::add(batch);
        let with_col = change.with_change_type_column();

        assert_eq!(with_col.num_columns(), 3);
        assert_eq!(with_col.schema().field(2).name(), "_change_type");
        assert_eq!(with_col.num_rows(), 2);

        let col = with_col
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(col.value(0), "add");
        assert_eq!(col.value(1), "add");
    }
}
