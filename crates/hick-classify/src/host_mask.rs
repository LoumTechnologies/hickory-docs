//! Host-side data masking utilities.
//!
//! Applies masking operations to Arrow RecordBatches, CSV files, and Parquet
//! files. Uses the same masking semantics as the streaming MaskNode in hick-live.

use std::path::Path;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, AsArray, Float64Array, Int32Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Float64Type, Int32Type, Int64Type};
use arrow::record_batch::RecordBatch;

use crate::{MaskOperation, MaskSpec};

// ---------------------------------------------------------------------------
// Core masking functions
// ---------------------------------------------------------------------------

/// Convert an Arrow array value at index to string.
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

/// Apply a single mask spec to a column array.
pub fn mask_column(array: &ArrayRef, spec: &MaskSpec) -> ArrayRef {
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
                _ => array.clone(),
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
            fn bucket_numeric(
                values: impl Iterator<Item = Option<f64>>,
                bucket_size: f64,
            ) -> ArrayRef {
                let bucketed: Vec<Option<String>> = values
                    .map(|v| {
                        v.map(|val| {
                            let lo = (val / bucket_size).floor() * bucket_size;
                            let hi = lo + bucket_size;
                            format!("{lo}-{hi}")
                        })
                    })
                    .collect();
                Arc::new(StringArray::from(bucketed))
            }

            match array.data_type() {
                DataType::Int32 => {
                    let arr = array.as_primitive::<Int32Type>();
                    bucket_numeric(
                        (0..arr.len()).map(|i| {
                            if arr.is_null(i) {
                                None
                            } else {
                                Some(arr.value(i) as f64)
                            }
                        }),
                        bucket_size,
                    )
                }
                DataType::Int64 => {
                    let arr = array.as_primitive::<Int64Type>();
                    bucket_numeric(
                        (0..arr.len()).map(|i| {
                            if arr.is_null(i) {
                                None
                            } else {
                                Some(arr.value(i) as f64)
                            }
                        }),
                        bucket_size,
                    )
                }
                DataType::Float64 => {
                    let arr = array.as_primitive::<Float64Type>();
                    bucket_numeric(
                        (0..arr.len()).map(|i| {
                            if arr.is_null(i) {
                                None
                            } else {
                                Some(arr.value(i))
                            }
                        }),
                        bucket_size,
                    )
                }
                _ => array.clone(),
            }
        }
    }
}

/// Mask an Arrow RecordBatch according to MaskSpecs.
pub fn mask_batch(batch: &RecordBatch, specs: &[MaskSpec]) -> RecordBatch {
    let schema = batch.schema();
    let mut columns: Vec<ArrayRef> = batch.columns().to_vec();

    for spec in specs {
        if let Some((idx, _)) = schema.column_with_name(&spec.column) {
            columns[idx] = mask_column(&columns[idx], spec);
        }
    }

    // Update schema for columns that changed type (mask/hash/bucket → Utf8)
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
    for spec in specs {
        if let Some((idx, _)) = schema.column_with_name(&spec.column) {
            match spec.operation {
                MaskOperation::Mask | MaskOperation::Hash | MaskOperation::Bucket => {
                    let old_field = schema.field(idx);
                    fields[idx] = Arc::new(Field::new(old_field.name(), DataType::Utf8, true));
                }
                MaskOperation::Redact => {
                    let old_field = schema.field(idx);
                    fields[idx] = Arc::new(Field::new(
                        old_field.name(),
                        old_field.data_type().clone(),
                        true, // redacted columns become nullable
                    ));
                }
            }
        }
    }
    let new_schema = Arc::new(arrow::datatypes::Schema::new(fields));
    RecordBatch::try_new(new_schema, columns).unwrap()
}

/// Mask a CSV file and write the masked output to a new file.
pub fn mask_csv(
    input: &Path,
    output: &Path,
    specs: &[MaskSpec],
) -> Result<(), Box<dyn std::error::Error>> {
    use arrow::csv;
    use std::fs::File;

    // Infer schema from CSV
    let file = File::open(input)?;
    let (schema, _) = arrow::csv::reader::Format::default()
        .with_header(true)
        .infer_schema(file, Some(100))?;
    let schema = Arc::new(schema);

    // Read batches
    let file = File::open(input)?;
    let reader = csv::ReaderBuilder::new(schema)
        .with_header(true)
        .build(file)?;

    let out_file = File::create(output)?;
    let mut writer: Option<csv::Writer<File>> = None;

    for batch_result in reader {
        let batch = batch_result?;
        let masked = mask_batch(&batch, specs);
        if writer.is_none() {
            writer = Some(csv::WriterBuilder::new().build(out_file.try_clone()?));
        }
        writer.as_mut().unwrap().write(&masked)?;
    }

    Ok(())
}

/// Mask a Parquet file and write the masked output to a new file.
pub fn mask_parquet(
    input: &Path,
    output: &Path,
    specs: &[MaskSpec],
) -> Result<(), Box<dyn std::error::Error>> {
    use parquet::arrow::ArrowWriter;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use std::fs::File;

    let file = File::open(input)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let reader = builder.build()?;

    let out_file = File::create(output)?;
    let mut writer: Option<ArrowWriter<File>> = None;

    for batch_result in reader {
        let batch = batch_result?;
        let masked = mask_batch(&batch, specs);
        if writer.is_none() {
            writer = Some(ArrowWriter::try_new(
                out_file.try_clone()?,
                masked.schema(),
                None,
            )?);
        }
        writer.as_mut().unwrap().write(&masked)?;
    }

    if let Some(w) = writer {
        w.close()?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int32Array, StringArray};
    use arrow::datatypes::Schema;

    fn test_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
            Field::new("ssn", DataType::Utf8, false),
            Field::new("salary", DataType::Int32, true),
        ]));
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

    #[test]
    fn mask_batch_hash() {
        let batch = test_batch();
        let specs = vec![MaskSpec::hash("ssn").with_salt("test-salt")];
        let masked = mask_batch(&batch, &specs);

        assert_eq!(masked.num_rows(), 2);
        assert_eq!(masked.schema().field(2).data_type(), &DataType::Utf8);

        let ssn_col = masked.column(2).as_string::<i32>();
        // Hashed values are 64-char hex strings
        assert_eq!(ssn_col.value(0).len(), 64);
        assert_ne!(ssn_col.value(0), ssn_col.value(1));
    }

    #[test]
    fn mask_batch_bucket() {
        let batch = test_batch();
        let specs = vec![MaskSpec::bucket("salary", 10000.0)];
        let masked = mask_batch(&batch, &specs);

        let salary_col = masked.column(3).as_string::<i32>();
        assert_eq!(salary_col.value(0), "70000-80000");
        assert_eq!(salary_col.value(1), "80000-90000");
    }

    #[test]
    fn mask_batch_preserves_row_count() {
        let batch = test_batch();
        let specs = vec![MaskSpec::redact("ssn")];
        let masked = mask_batch(&batch, &specs);

        assert_eq!(masked.num_rows(), batch.num_rows());
        assert_eq!(masked.num_columns(), batch.num_columns());
    }

    #[test]
    fn mask_csv_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("input.csv");
        let output_path = dir.path().join("output.csv");

        // Write test CSV
        std::fs::write(
            &input_path,
            "id,name,ssn\n1,alice,123-45-6789\n2,bob,987-65-4321\n",
        )
        .unwrap();

        let specs = vec![MaskSpec::hash("ssn").with_salt("s")];
        mask_csv(&input_path, &output_path, &specs).unwrap();

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.starts_with("id,name,ssn\n"));
        // Verify hashed values are present (not original SSNs)
        assert!(!content.contains("123-45-6789"));
        assert!(!content.contains("987-65-4321"));
    }

    #[test]
    fn mask_parquet_roundtrip() {
        use parquet::arrow::ArrowWriter;
        use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
        use std::fs::File;

        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("input.parquet");
        let output_path = dir.path().join("output.parquet");

        // Write test parquet
        let batch = test_batch();
        let file = File::create(&input_path).unwrap();
        let mut writer = ArrowWriter::try_new(file, batch.schema(), None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();

        let specs = vec![MaskSpec::bucket("salary", 10000.0)];
        mask_parquet(&input_path, &output_path, &specs).unwrap();

        // Read back
        let file = File::open(&output_path).unwrap();
        let reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let batches: Vec<_> = reader.map(|b| b.unwrap()).collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].num_rows(), 2);

        let salary_col = batches[0].column(3).as_string::<i32>();
        assert_eq!(salary_col.value(0), "70000-80000");
    }
}
