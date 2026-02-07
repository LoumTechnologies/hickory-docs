//! CSV file data source.
//!
//! Reads a CSV file as an initial Add event. Future: watch for changes.

use std::path::PathBuf;
use std::sync::Arc;

use arrow::csv::ReaderBuilder;
use arrow::datatypes::Schema;
use futures::stream;
use hick_flow::Context;

use crate::changes::{ChangeStream, RecordChange};
use crate::schema::LiveSchema;
use crate::sources::ChangeSource;

/// Data source that reads a CSV file into Arrow RecordBatches.
pub struct CsvSource {
    schema: LiveSchema,
    path: PathBuf,
}

impl CsvSource {
    /// Create a CSV source that reads from the given path.
    ///
    /// The caller provides the expected schema; the CSV reader will parse
    /// columns according to it.
    pub fn new(schema: LiveSchema, path: impl Into<PathBuf>) -> Self {
        Self {
            schema,
            path: path.into(),
        }
    }
}

impl ChangeSource for CsvSource {
    fn schema(&self) -> &LiveSchema {
        &self.schema
    }

    fn stream(&self, _context: Context) -> ChangeStream {
        let path = self.path.clone();
        let arrow_schema = self.schema.arrow_schema.clone();

        // Read the entire CSV file into batches and emit as a single Add
        match read_csv(&path, &arrow_schema) {
            Ok(batches) => {
                let events: Vec<RecordChange> = batches
                    .into_iter()
                    .map(RecordChange::add)
                    .collect();
                Box::pin(stream::iter(events))
            }
            Err(e) => {
                log::error!("Failed to read CSV file {}: {}", path.display(), e);
                Box::pin(stream::empty())
            }
        }
    }
}

fn read_csv(
    path: &std::path::Path,
    schema: &Arc<Schema>,
) -> Result<Vec<arrow::record_batch::RecordBatch>, arrow::error::ArrowError> {
    let file = std::fs::File::open(path).map_err(|e| {
        arrow::error::ArrowError::IoError(
            format!("opening {}: {}", path.display(), e),
            e,
        )
    })?;
    let reader = ReaderBuilder::new(schema.clone())
        .with_header(true)
        .build(file)?;

    reader.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field};
    use futures::StreamExt;
    use std::io::Write;

    #[tokio::test]
    async fn reads_csv_file() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("test.csv");
        {
            let mut f = std::fs::File::create(&csv_path).unwrap();
            writeln!(f, "id,name").unwrap();
            writeln!(f, "1,alice").unwrap();
            writeln!(f, "2,bob").unwrap();
        }

        let arrow_schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));
        let live_schema = LiveSchema::new_strict(arrow_schema, vec!["id".into()]);

        let source = CsvSource::new(live_schema, &csv_path);
        let events: Vec<_> = source.stream(Context::default()).collect().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].num_rows(), 2);
    }
}
