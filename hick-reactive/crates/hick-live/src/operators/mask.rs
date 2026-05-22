//! Mask operator for incremental collections.
//!
//! Applies masking operations (mask, redact, hash, bucket) to specified
//! columns, downgrading their classification labels.

use std::sync::Arc;

use arrow::array::{Array, ArrayRef, AsArray, Float64Array, Int32Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Float64Type, Int32Type, Int64Type};
use arrow::record_batch::RecordBatch;
use futures::StreamExt;
use hick_classify::{MaskOperation, MaskSpec};
use hick_flow::Context;
use hick_flow::node::{BoxStream, Node, NodeTrace, NodeValue};

use crate::changes::RecordChange;
use crate::node::{NodeExt, RecordChangeNode};
use crate::schema::LiveSchema;

/// Applies masking operations to columns in upstream change events.
pub struct MaskNode {
    pub source: Arc<dyn Node>,
    pub mask_specs: Vec<MaskSpec>,
    pub output_schema: LiveSchema,
}

impl MaskNode {
    pub fn new(
        source: Arc<dyn Node>,
        mask_specs: Vec<MaskSpec>,
        input_schema: &LiveSchema,
    ) -> Self {
        let mut classifications = input_schema.classifications.clone();
        for spec in &mask_specs {
            if let Some(target) = &spec.target_classification {
                classifications.set(&spec.column, target.clone());
            }
        }
        let output_schema = LiveSchema {
            arrow_schema: input_schema.arrow_schema.clone(),
            key_columns: input_schema.key_columns.clone(),
            is_strict: input_schema.is_strict,
            classifications,
        };
        Self {
            source,
            mask_specs,
            output_schema,
        }
    }
}

/// Apply a single mask spec to a column array.
fn mask_column(array: &ArrayRef, spec: &MaskSpec) -> ArrayRef {
    match spec.operation {
        MaskOperation::Mask => {
            let mask_val = spec.effective_mask_value();
            let len = array.len();
            let masked: Vec<Option<&str>> = (0..len)
                .map(|i| {
                    if array.is_null(i) {
                        None
                    } else {
                        Some(mask_val)
                    }
                })
                .collect();
            Arc::new(StringArray::from(masked))
        }
        MaskOperation::Redact => {
            let len = array.len();
            // Create a null array of the same type
            match array.data_type() {
                DataType::Utf8 => {
                    let nulls: Vec<Option<&str>> = vec![None; len];
                    Arc::new(StringArray::from(nulls))
                }
                DataType::Int32 => {
                    let nulls: Vec<Option<i32>> = vec![None; len];
                    Arc::new(Int32Array::from(nulls))
                }
                DataType::Int64 => {
                    let nulls: Vec<Option<i64>> = vec![None; len];
                    Arc::new(Int64Array::from(nulls))
                }
                DataType::Float64 => {
                    let nulls: Vec<Option<f64>> = vec![None; len];
                    Arc::new(Float64Array::from(nulls))
                }
                _ => array.clone(), // unsupported type: pass through
            }
        }
        MaskOperation::Hash => {
            let salt = spec.hash_salt.as_deref().unwrap_or("");
            let len = array.len();
            let hashed: Vec<Option<String>> = (0..len)
                .map(|i| {
                    if array.is_null(i) {
                        return None;
                    }
                    let val_str = value_to_string(array, i);
                    let input = format!("{salt}{val_str}");
                    let hash = blake3::hash(input.as_bytes());
                    Some(hash.to_hex().to_string())
                })
                .collect();
            Arc::new(StringArray::from(hashed))
        }
        MaskOperation::Bucket => {
            let bucket_size = spec.bucket_size.unwrap_or(1.0);
            match array.data_type() {
                DataType::Int32 => {
                    let arr = array.as_primitive::<Int32Type>();
                    let bucketed: Vec<Option<String>> = (0..arr.len())
                        .map(|i| {
                            if arr.is_null(i) {
                                return None;
                            }
                            let v = arr.value(i) as f64;
                            let lo = (v / bucket_size).floor() * bucket_size;
                            let hi = lo + bucket_size;
                            Some(format!("{lo}-{hi}"))
                        })
                        .collect();
                    Arc::new(StringArray::from(bucketed))
                }
                DataType::Int64 => {
                    let arr = array.as_primitive::<Int64Type>();
                    let bucketed: Vec<Option<String>> = (0..arr.len())
                        .map(|i| {
                            if arr.is_null(i) {
                                return None;
                            }
                            let v = arr.value(i) as f64;
                            let lo = (v / bucket_size).floor() * bucket_size;
                            let hi = lo + bucket_size;
                            Some(format!("{lo}-{hi}"))
                        })
                        .collect();
                    Arc::new(StringArray::from(bucketed))
                }
                DataType::Float64 => {
                    let arr = array.as_primitive::<Float64Type>();
                    let bucketed: Vec<Option<String>> = (0..arr.len())
                        .map(|i| {
                            if arr.is_null(i) {
                                return None;
                            }
                            let v = arr.value(i);
                            let lo = (v / bucket_size).floor() * bucket_size;
                            let hi = lo + bucket_size;
                            Some(format!("{lo}-{hi}"))
                        })
                        .collect();
                    Arc::new(StringArray::from(bucketed))
                }
                _ => array.clone(),
            }
        }
    }
}

pub fn value_to_string(array: &ArrayRef, i: usize) -> String {
    match array.data_type() {
        DataType::Utf8 => {
            let arr = array.as_string::<i32>();
            arr.value(i).to_string()
        }
        DataType::Int32 => {
            let arr = array.as_primitive::<Int32Type>();
            arr.value(i).to_string()
        }
        DataType::Int64 => {
            let arr = array.as_primitive::<Int64Type>();
            arr.value(i).to_string()
        }
        DataType::Float64 => {
            let arr = array.as_primitive::<Float64Type>();
            arr.value(i).to_string()
        }
        _ => format!("{:?}", array.slice(i, 1)),
    }
}

fn apply_masks(batch: &RecordBatch, specs: &[MaskSpec]) -> RecordBatch {
    let schema = batch.schema();
    let mut columns: Vec<ArrayRef> = batch.columns().to_vec();

    for spec in specs {
        if let Some((idx, _)) = schema.column_with_name(&spec.column) {
            columns[idx] = mask_column(&columns[idx], spec);
        }
    }

    // For mask/hash/bucket operations, output columns become Utf8
    // so we need to update the schema accordingly
    let mut fields: Vec<arrow::datatypes::FieldRef> = schema.fields().iter().cloned().collect();
    for spec in specs {
        if let Some((idx, _)) = schema.column_with_name(&spec.column) {
            match spec.operation {
                MaskOperation::Mask | MaskOperation::Hash | MaskOperation::Bucket => {
                    let old_field = schema.field(idx);
                    fields[idx] = Arc::new(arrow::datatypes::Field::new(
                        old_field.name(),
                        DataType::Utf8,
                        true,
                    ));
                }
                MaskOperation::Redact => {} // preserves original type
            }
        }
    }
    let new_schema = Arc::new(arrow::datatypes::Schema::new(fields));
    RecordBatch::try_new(new_schema, columns).unwrap()
}

impl Node for MaskNode {
    fn get_stream(self: Arc<Self>, context: Context) -> BoxStream<Vec<NodeTrace>> {
        let source_stream = self.source.clone().get_stream(context);
        let specs = self.mask_specs.clone();
        let me: Arc<dyn Node> = self;

        Box::pin(source_stream.map(move |traces| {
            let specs = &specs;
            let me = me.clone();
            traces
                .into_iter()
                .map(|trace| {
                    if let Some(change) = trace.final_result().as_record_change() {
                        let masked = apply_masks(&change.records, specs);
                        let new_change =
                            RecordChange::new(change.change_type, masked, change.is_strict);
                        let new_node: Arc<dyn Node> = Arc::new(RecordChangeNode::new(new_change));
                        trace.transform(new_node, me.clone())
                    } else {
                        trace.add_to_trace(me.clone())
                    }
                })
                .collect()
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
    use hick_classify::{Classification, ColumnClassifications};
    use serde_json::json;

    fn test_schema() -> (Arc<Schema>, LiveSchema) {
        let arrow = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
            Field::new("ssn", DataType::Utf8, false),
            Field::new("salary", DataType::Int32, true),
        ]));
        let mut cls = ColumnClassifications::new();
        cls.set(
            "ssn",
            Classification::new(vec![json!({"sensitivity": "secret"})]),
        );
        cls.set(
            "salary",
            Classification::new(vec![json!({"sensitivity": "confidential"})]),
        );
        let live =
            LiveSchema::new_strict(arrow.clone(), vec!["id".into()]).with_classifications(cls);
        (arrow, live)
    }

    fn test_batch() -> RecordBatch {
        let (schema, _) = test_schema();
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(StringArray::from(vec!["alice", "bob"])),
                Arc::new(StringArray::from(vec!["123-45-6789", "987-65-4321"])),
                Arc::new(Int32Array::from(vec![Some(75000), Some(85000)])),
            ],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn mask_utf8_column() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let specs = vec![MaskSpec::mask("ssn").with_mask_value("XXX-XX-XXXX")];
        let mask_node = Arc::new(MaskNode::new(data_node, specs, &live));

        let mut stream = mask_node.get_stream(Context::default());
        let traces = stream.next().await.unwrap();
        let node = traces[0].final_result();
        let change = node.as_record_change().unwrap();

        let ssn_col = change.records.column(2).as_string::<i32>();
        assert_eq!(ssn_col.value(0), "XXX-XX-XXXX");
        assert_eq!(ssn_col.value(1), "XXX-XX-XXXX");
    }

    #[tokio::test]
    async fn hash_preserves_equality() {
        let (_, live) = test_schema();

        // Same SSN twice
        let (schema, _) = test_schema();
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["a", "b", "c"])),
                Arc::new(StringArray::from(vec!["same-ssn", "different", "same-ssn"])),
                Arc::new(Int32Array::from(vec![Some(1), Some(2), Some(3)])),
            ],
        )
        .unwrap();

        let source = Arc::new(MemorySource::new(live.clone(), batch));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let specs = vec![MaskSpec::hash("ssn").with_salt("test-salt")];
        let mask_node = Arc::new(MaskNode::new(data_node, specs, &live));

        let mut stream = mask_node.get_stream(Context::default());
        let traces = stream.next().await.unwrap();
        let node = traces[0].final_result();
        let change = node.as_record_change().unwrap();

        let ssn_col = change.records.column(2).as_string::<i32>();
        // same input → same hash
        assert_eq!(ssn_col.value(0), ssn_col.value(2));
        // different input → different hash
        assert_ne!(ssn_col.value(0), ssn_col.value(1));
    }

    #[tokio::test]
    async fn bucket_rounds_correctly() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let specs = vec![MaskSpec::bucket("salary", 10000.0)];
        let mask_node = Arc::new(MaskNode::new(data_node, specs, &live));

        let mut stream = mask_node.get_stream(Context::default());
        let traces = stream.next().await.unwrap();
        let node = traces[0].final_result();
        let change = node.as_record_change().unwrap();

        let salary_col = change.records.column(3).as_string::<i32>();
        assert_eq!(salary_col.value(0), "70000-80000"); // 75000 → 70000-80000
        assert_eq!(salary_col.value(1), "80000-90000"); // 85000 → 80000-90000
    }

    #[tokio::test]
    async fn classification_downgraded_after_masking() {
        let (_, live) = test_schema();

        let source = Arc::new(MemorySource::new(live.clone(), test_batch()));
        let data_node = Arc::new(StreamingStructuredDataNode::new(live.clone(), source));

        let target = Classification::new(vec![json!({"sensitivity": "pseudonymized"})]);
        let specs = vec![MaskSpec::hash("ssn").with_target_classification(target.clone())];
        let mask_node = Arc::new(MaskNode::new(data_node, specs, &live));

        let cls = &mask_node.output_schema.classifications;
        let ssn_cls = cls.get("ssn").unwrap();
        assert_eq!(ssn_cls, &target);
        // salary classification unchanged
        let salary_cls = cls.get("salary").unwrap();
        assert_eq!(
            salary_cls,
            &Classification::new(vec![json!({"sensitivity": "confidential"})])
        );
    }
}
