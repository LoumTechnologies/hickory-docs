//! Schema metadata for incremental collection streams.

use std::sync::Arc;

use arrow::datatypes::Schema;

/// Schema for a live (incrementally-updating) collection.
///
/// Wraps an Arrow schema with additional metadata needed by the change system:
/// key columns for row identity, and strictness for remove-event fidelity.
#[derive(Debug, Clone)]
pub struct LiveSchema {
    /// The Arrow schema describing the columns in each RecordBatch.
    pub arrow_schema: Arc<Schema>,
    /// Column names that form the identity of a row.
    ///
    /// Used for dedup, grouping, and matching Remove events to prior Adds.
    pub key_columns: Vec<String>,
    /// Whether this stream guarantees strict Remove events
    /// (i.e., Remove events carry full row data, not just keys).
    pub is_strict: bool,
}

impl LiveSchema {
    /// Create a new strict schema (all removes carry full data).
    pub fn new_strict(arrow_schema: Arc<Schema>, key_columns: Vec<String>) -> Self {
        Self {
            arrow_schema,
            key_columns,
            is_strict: true,
        }
    }

    /// Create a new non-strict schema.
    pub fn new_non_strict(arrow_schema: Arc<Schema>, key_columns: Vec<String>) -> Self {
        Self {
            arrow_schema,
            key_columns,
            is_strict: false,
        }
    }

    /// Check whether the given column names are all present in the schema.
    pub fn has_columns(&self, names: &[&str]) -> bool {
        names.iter().all(|name| self.arrow_schema.column_with_name(name).is_some())
    }

    /// Return the schema that results from projecting only the given columns.
    pub fn project(&self, columns: &[String]) -> Self {
        let fields: Vec<_> = columns
            .iter()
            .filter_map(|name| {
                self.arrow_schema
                    .column_with_name(name)
                    .map(|(_, f)| f.clone())
            })
            .collect();
        let projected_keys: Vec<String> = self
            .key_columns
            .iter()
            .filter(|k| columns.contains(k))
            .cloned()
            .collect();
        let keys_survive = self.key_columns.iter().all(|k| columns.contains(k));
        Self {
            arrow_schema: Arc::new(Schema::new(fields)),
            key_columns: projected_keys,
            is_strict: self.is_strict && keys_survive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field};

    fn test_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
            Field::new("age", DataType::Int32, true),
        ]))
    }

    #[test]
    fn strict_schema() {
        let schema = LiveSchema::new_strict(test_schema(), vec!["id".into()]);
        assert!(schema.is_strict);
        assert_eq!(schema.key_columns, vec!["id"]);
    }

    #[test]
    fn has_columns_checks_presence() {
        let schema = LiveSchema::new_strict(test_schema(), vec!["id".into()]);
        assert!(schema.has_columns(&["id", "name"]));
        assert!(!schema.has_columns(&["id", "missing"]));
    }

    #[test]
    fn project_preserves_strictness_when_keys_survive() {
        let schema = LiveSchema::new_strict(test_schema(), vec!["id".into()]);
        let projected = schema.project(&["id".into(), "name".into()]);
        assert!(projected.is_strict);
        assert_eq!(projected.arrow_schema.fields().len(), 2);
    }

    #[test]
    fn project_loses_strictness_when_keys_dropped() {
        let schema = LiveSchema::new_strict(test_schema(), vec!["id".into()]);
        let projected = schema.project(&["name".into(), "age".into()]);
        assert!(!projected.is_strict);
        assert!(projected.key_columns.is_empty());
    }
}
