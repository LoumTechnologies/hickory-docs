//! Tag handler for `<hick:where>`.

use std::sync::Arc;

use anyhow::{Result, bail};
use arrow::array::{Array, AsArray, BooleanArray};
use arrow::datatypes::{DataType, Float64Type, Int32Type, Int64Type};
use hick_handlers::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};
use hick_lang::HickTag;

use crate::operators::where_op::{Predicate, WhereNode};

/// Handler for `<hick:where expr="...">` tags.
///
/// Wraps its child source node in a `WhereNode` that filters rows using
/// the expression in the `expr` attribute.
///
/// Currently supports simple expressions like:
/// - `column = value` (string equality)
/// - `column > value` (numeric comparison)
/// - `column < value` (numeric comparison)
/// - `column >= value` (numeric comparison)
/// - `column <= value` (numeric comparison)
/// - `column != value` (inequality)
pub struct WhereHandler;

impl TagHandler for WhereHandler {
    fn tag_name(&self) -> &str {
        "where"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        let expr = tag_attr(tag, "expr")
            .ok_or_else(|| anyhow::anyhow!("where requires an expr attribute"))?;

        let source = ctx
            .first_child_node(tag)
            .ok_or_else(|| anyhow::anyhow!("where requires a child source node"))?;

        let schema = super::infer_schema(&source);

        let predicate = parse_predicate(&expr)?;
        let where_node = WhereNode::new(source, predicate, schema);
        Ok(TagResult::Node(Arc::new(where_node)))
    }
}

/// Parse a simple predicate expression into an Arrow compute predicate.
fn parse_predicate(expr: &str) -> Result<Predicate> {
    // Try operators in order of longest match first
    let operators = [">=", "<=", "!=", "=", ">", "<"];
    for op in &operators {
        if let Some((col_name, value)) = expr.split_once(op) {
            let col_name = col_name.trim().to_string();
            let value = value.trim().to_string();
            let op = op.to_string();
            return Ok(Arc::new(move |batch| {
                let schema = batch.schema();
                let col_idx = match schema.column_with_name(&col_name) {
                    Some((idx, _)) => idx,
                    None => return BooleanArray::from(vec![false; batch.num_rows()]),
                };
                let col = batch.column(col_idx);
                apply_predicate(col, &op, &value, batch.num_rows())
            }));
        }
    }
    bail!("Cannot parse where expression: {expr}")
}

fn apply_predicate(
    col: &dyn arrow::array::Array,
    op: &str,
    value: &str,
    num_rows: usize,
) -> BooleanArray {
    match col.data_type() {
        DataType::Utf8 => {
            let arr = col.as_string::<i32>();
            BooleanArray::from(
                (0..num_rows)
                    .map(|i| {
                        if arr.is_null(i) {
                            Some(false)
                        } else {
                            let v = arr.value(i);
                            // Strip surrounding quotes from value if present
                            let cmp_val = value.trim_matches('\'').trim_matches('"');
                            Some(match op {
                                "=" => v == cmp_val,
                                "!=" => v != cmp_val,
                                ">" => v > cmp_val,
                                "<" => v < cmp_val,
                                ">=" => v >= cmp_val,
                                "<=" => v <= cmp_val,
                                _ => false,
                            })
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        }
        DataType::Int32 => {
            let arr = col.as_primitive::<Int32Type>();
            if let Ok(val) = value.parse::<i32>() {
                BooleanArray::from(
                    (0..num_rows)
                        .map(|i| {
                            if arr.is_null(i) {
                                Some(false)
                            } else {
                                let v = arr.value(i);
                                Some(match op {
                                    "=" => v == val,
                                    "!=" => v != val,
                                    ">" => v > val,
                                    "<" => v < val,
                                    ">=" => v >= val,
                                    "<=" => v <= val,
                                    _ => false,
                                })
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                BooleanArray::from(vec![false; num_rows])
            }
        }
        DataType::Int64 => {
            let arr = col.as_primitive::<Int64Type>();
            if let Ok(val) = value.parse::<i64>() {
                BooleanArray::from(
                    (0..num_rows)
                        .map(|i| {
                            if arr.is_null(i) {
                                Some(false)
                            } else {
                                let v = arr.value(i);
                                Some(match op {
                                    "=" => v == val,
                                    "!=" => v != val,
                                    ">" => v > val,
                                    "<" => v < val,
                                    ">=" => v >= val,
                                    "<=" => v <= val,
                                    _ => false,
                                })
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                BooleanArray::from(vec![false; num_rows])
            }
        }
        DataType::Float64 => {
            let arr = col.as_primitive::<Float64Type>();
            if let Ok(val) = value.parse::<f64>() {
                BooleanArray::from(
                    (0..num_rows)
                        .map(|i| {
                            if arr.is_null(i) {
                                Some(false)
                            } else {
                                let v = arr.value(i);
                                Some(match op {
                                    "=" => (v - val).abs() < f64::EPSILON,
                                    "!=" => (v - val).abs() >= f64::EPSILON,
                                    ">" => v > val,
                                    "<" => v < val,
                                    ">=" => v >= val,
                                    "<=" => v <= val,
                                    _ => false,
                                })
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                BooleanArray::from(vec![false; num_rows])
            }
        }
        _ => BooleanArray::from(vec![false; num_rows]),
    }
}
