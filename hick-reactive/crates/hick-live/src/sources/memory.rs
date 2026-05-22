//! In-memory data source for testing.

use std::sync::Mutex;

use arrow::record_batch::RecordBatch;
use futures::stream;
use hick_flow::Context;

use crate::changes::{ChangeStream, RecordChange};
use crate::schema::LiveSchema;
use crate::sources::ChangeSource;

/// In-memory data source that emits its initial data as a single Add event.
///
/// Additional changes can be queued via `push()` before calling `stream()`.
pub struct MemorySource {
    schema: LiveSchema,
    initial: RecordBatch,
    pending: Mutex<Vec<RecordChange>>,
}

impl MemorySource {
    /// Create a source with the given initial data.
    pub fn new(schema: LiveSchema, initial: RecordBatch) -> Self {
        Self {
            schema,
            initial,
            pending: Mutex::new(Vec::new()),
        }
    }

    /// Queue an additional change event to be emitted after the initial Add.
    pub fn push(&self, change: RecordChange) {
        self.pending.lock().unwrap().push(change);
    }
}

impl ChangeSource for MemorySource {
    fn schema(&self) -> &LiveSchema {
        &self.schema
    }

    fn stream(&self, _context: Context) -> ChangeStream {
        let mut events = vec![RecordChange::add(self.initial.clone())];
        events.extend(self.pending.lock().unwrap().drain(..));
        Box::pin(stream::iter(events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changes::ChangeType;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use futures::StreamExt;
    use std::sync::Arc;

    fn test_schema() -> LiveSchema {
        let arrow = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));
        LiveSchema::new_strict(arrow, vec!["id".into()])
    }

    fn test_batch(ids: &[i32], names: &[&str]) -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));
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
    async fn emits_initial_add() {
        let source = MemorySource::new(test_schema(), test_batch(&[1, 2], &["a", "b"]));
        let events: Vec<_> = source.stream(Context::default()).collect().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].change_type, ChangeType::Add);
        assert_eq!(events[0].num_rows(), 2);
    }

    #[tokio::test]
    async fn emits_pending_changes() {
        let source = MemorySource::new(test_schema(), test_batch(&[1], &["a"]));
        source.push(RecordChange::add(test_batch(&[2], &["b"])));
        source.push(RecordChange::remove_strict(test_batch(&[1], &["a"])));

        let events: Vec<_> = source.stream(Context::default()).collect().await;
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].change_type, ChangeType::Add);
        assert_eq!(events[1].change_type, ChangeType::Add);
        assert_eq!(events[2].change_type, ChangeType::Remove);
    }
}
