//! Tag handler for `<hick:data-source>`.

use std::sync::Arc;

use anyhow::{Result, bail};
use arrow::datatypes::{DataType, Field, Schema};
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

use hick_classify::{Classification, ColumnClassifications};

use crate::node::StreamingStructuredDataNode;
use crate::schema::LiveSchema;
use crate::sources::csv::CsvSource;
use crate::sources::memory::MemorySource;

/// Handler for `<hick:data-source type="..." ...>` tags.
///
/// Constructs a `StreamingStructuredDataNode` with the appropriate
/// `ChangeSource` based on the `type` attribute.
///
/// Supported types:
/// - `csv`: Reads a CSV file. Requires `path` attribute. Schema is inferred
///   from child `<hick:column>` tags or auto-detected.
/// - `memory`: Empty in-memory source (primarily for testing).
pub struct DataSourceHandler;

impl TagHandler for DataSourceHandler {
    fn tag_name(&self) -> &str {
        "data-source"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, _ctx: &ProcessingContext) -> Result<TagResult> {
        let source_type = tag_attr(tag, "type").unwrap_or_default();

        let schema = parse_schema(tag)?;
        let key_columns = parse_key_columns(tag);
        let classifications = parse_classifications(tag);
        let live_schema = LiveSchema::new_strict(Arc::new(schema.clone()), key_columns)
            .with_classifications(classifications);

        match source_type.as_str() {
            "csv" => {
                let path = tag_attr(tag, "path").ok_or_else(|| {
                    anyhow::anyhow!("data-source type=csv requires a path attribute")
                })?;
                let source = Arc::new(CsvSource::new(live_schema.clone(), path));
                let node = StreamingStructuredDataNode::new(live_schema, source);
                Ok(TagResult::Node(Arc::new(node)))
            }
            "memory" => {
                let empty_batch = arrow::record_batch::RecordBatch::new_empty(Arc::new(schema));
                let source = Arc::new(MemorySource::new(live_schema.clone(), empty_batch));
                let node = StreamingStructuredDataNode::new(live_schema, source);
                Ok(TagResult::Node(Arc::new(node)))
            }
            other => {
                bail!("Unknown data-source type: {other}. Supported: csv, memory")
            }
        }
    }
}

/// Parse column definitions from child `<hick:column>` tags.
///
/// Example: `<hick:column name="id" type="int32" key="true"/>`
fn parse_schema(tag: &HickTag) -> Result<Schema> {
    use hick_lang::HickNode;

    let mut fields = Vec::new();
    for child in &tag.children {
        if let HickNode::Tag(child_tag) = child
            && child_tag.name == "column"
        {
            let name = tag_attr(child_tag, "name")
                .ok_or_else(|| anyhow::anyhow!("column requires a name attribute"))?;
            let data_type = match tag_attr(child_tag, "type").as_deref() {
                Some("int32") | Some("i32") => DataType::Int32,
                Some("int64") | Some("i64") => DataType::Int64,
                Some("utf8") | Some("string") | Some("text") => DataType::Utf8,
                Some("float32") | Some("f32") => DataType::Float32,
                Some("float64") | Some("f64") => DataType::Float64,
                Some("bool") | Some("boolean") => DataType::Boolean,
                Some(other) => bail!("Unknown column type: {other}"),
                None => DataType::Utf8, // default to string
            };
            let nullable = tag_attr(child_tag, "nullable")
                .map(|v| v == "true")
                .unwrap_or(true);
            fields.push(Field::new(name, data_type, nullable));
        }
    }

    if fields.is_empty() {
        bail!("data-source requires at least one <column> child tag")
    }

    Ok(Schema::new(fields))
}

/// Parse classification attributes from child `<hick:column>` tags.
fn parse_classifications(tag: &HickTag) -> ColumnClassifications {
    use hick_lang::HickNode;

    let mut cls = ColumnClassifications::new();
    for child in &tag.children {
        if let HickNode::Tag(child_tag) = child
            && child_tag.name == "column"
            && let Some(name) = tag_attr(child_tag, "name")
            && let Some(cls_str) = tag_attr(child_tag, "classification")
        {
            if let Ok(classification) = Classification::from_json_string(&cls_str) {
                cls.set(&name, classification);
            }
        }
    }
    cls
}

/// Parse key columns from child `<hick:column key="true">` tags.
fn parse_key_columns(tag: &HickTag) -> Vec<String> {
    use hick_lang::HickNode;

    let mut keys = Vec::new();
    for child in &tag.children {
        if let HickNode::Tag(child_tag) = child
            && child_tag.name == "column"
            && tag_attr(child_tag, "key").as_deref() == Some("true")
            && let Some(name) = tag_attr(child_tag, "name")
        {
            keys.push(name);
        }
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_exec::state::MultiDocumentState;
    use hick_lang::{HickNode, HickTag};
    use std::collections::HashMap;

    fn make_ctx<'a>(
        state: &'a Arc<MultiDocumentState>,
        transcripts: &'a HashMap<String, Vec<hick_handlers::TranscriptEntry>>,
    ) -> ProcessingContext<'a> {
        ProcessingContext {
            state,
            transcripts,
            indent: 0,
            registry: None,
            context: None,
            source_file: None,
        }
    }

    fn column_tag(name: &str, dtype: &str, key: bool) -> HickNode {
        let mut attrs = vec![
            ("name".to_string(), name.to_string()),
            ("type".to_string(), dtype.to_string()),
        ];
        if key {
            attrs.push(("key".to_string(), "true".to_string()));
        }
        HickNode::Tag(HickTag {
            name: "column".to_string(),
            attributes: attrs,
            children: vec![],
            self_closing: true,
            source_line: 1,
            source_column: 0,
            source_span: None,
        })
    }

    #[test]
    fn data_source_csv_constructs_node() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        // Create a temp CSV file
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("test.csv");
        std::fs::write(&csv_path, "id,name\n1,alice\n2,bob\n").unwrap();

        let tag = HickTag {
            name: "data-source".to_string(),
            attributes: vec![
                ("type".to_string(), "csv".to_string()),
                ("path".to_string(), csv_path.to_str().unwrap().to_string()),
            ],
            children: vec![
                column_tag("id", "int32", true),
                column_tag("name", "string", false),
            ],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
        };

        let handler = DataSourceHandler;
        let result = handler.process(&tag, &ctx).unwrap();
        assert!(matches!(result, TagResult::Node(_)));
    }

    #[test]
    fn data_source_memory_constructs_empty_node() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = HickTag {
            name: "data-source".to_string(),
            attributes: vec![("type".to_string(), "memory".to_string())],
            children: vec![
                column_tag("id", "int32", true),
                column_tag("value", "utf8", false),
            ],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
        };

        let handler = DataSourceHandler;
        let result = handler.process(&tag, &ctx).unwrap();
        assert!(matches!(result, TagResult::Node(_)));
    }

    #[test]
    fn data_source_unknown_type_errors() {
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = make_ctx(&state, &transcripts);

        let tag = HickTag {
            name: "data-source".to_string(),
            attributes: vec![("type".to_string(), "kafka".to_string())],
            children: vec![column_tag("id", "int32", true)],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
        };

        let handler = DataSourceHandler;
        let result = handler.process(&tag, &ctx);
        assert!(result.is_err());
    }
}
