//! The Formula Compute Protocol, on the wire.
//!
//! Framed exactly like LSP and DAP — `Content-Length`, a blank line, one JSON
//! object — because a backend author almost certainly has an LSP or DAP
//! implementation to crib from, and because the framing is the one part of
//! those protocols nobody has ever wished were different.
//!
//! Correlated by `id`, LSP-style, because a backend answers requests and
//! never starts a conversation of its own. There are no events here at all:
//! evaluating an expression either produces a value or fails, and a protocol
//! that left room for a backend to volunteer something would be leaving room
//! for a backend to hang the host.
//!
//! # What is deliberately NOT in this protocol
//!
//! No cell references, no sheet, no dependency order, no notion of "the table
//! changed". The host resolves references to values before it asks (see
//! `graph.rs`), so a backend never learns what a cell is. That is what keeps
//! a backend to sixty lines and what makes two languages in one table
//! evaluate in one order.

use serde::{Deserialize, Serialize};

/// A value that crosses the wire.
///
/// Deliberately narrow. A formula cell lives in a CSV file, and a CSV file
/// holds text — so anything a backend returns has to be renderable as text
/// without losing what it was. Numbers and booleans are called out separately
/// from strings only so a backend can format them the way its language would,
/// and so `=1+1` is `2` rather than `2.0` in one language and `2` in another.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Value {
    Number {
        value: f64,
    },
    Text {
        value: String,
    },
    Bool {
        value: bool,
    },
    /// A rectangular A1 range, in reading order. A range is a value because
    /// `sum(B2:B4)` is one operand in Python and JavaScript, not a bit of
    /// syntax either backend has to understand.
    List {
        value: Vec<Value>,
    },
    /// The cell is empty. Distinct from an empty string, because summing a
    /// column should skip blanks rather than treat them as zero-length text.
    Empty,
}

impl Value {
    /// How this value is written into the CSV.
    ///
    /// A whole number loses its `.0`: a table of counts full of `3.0` is a
    /// table that has been through a computer, and the point of writing back
    /// into a CSV is that a person still recognises their data.
    pub fn to_cell(&self) -> String {
        match self {
            Value::Number { value } => {
                if value.is_finite() && *value == value.trunc() && value.abs() < 1e15 {
                    format!("{}", *value as i64)
                } else {
                    format!("{value}")
                }
            }
            Value::Text { value } => value.clone(),
            Value::Bool { value } => value.to_string(),
            Value::List { value } => value
                .iter()
                .map(Value::to_cell)
                .collect::<Vec<_>>()
                .join(", "),
            Value::Empty => String::new(),
        }
    }

    /// Read a cell's text as a value, the way a spreadsheet does: a number if
    /// it looks like one, otherwise text.
    pub fn from_cell(text: &str) -> Self {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Value::Empty;
        }
        if let Ok(value) = trimmed.parse::<f64>() {
            return Value::Number { value };
        }
        match trimmed {
            "true" | "TRUE" => Value::Bool { value: true },
            "false" | "FALSE" => Value::Bool { value: false },
            _ => Value::Text {
                value: text.to_string(),
            },
        }
    }
}

/// One expression to evaluate, with its references already resolved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Formula {
    /// Opaque to the backend; the host uses it to match answers to cells.
    pub id: String,
    /// The expression, without the leading `=`.
    pub expression: String,
    /// The names the expression may use, and what they are worth. The host
    /// resolved these; a backend never resolves anything.
    pub bindings: Vec<(String, Value)>,
}

/// Everything to evaluate, in the order the host worked out.
///
/// Sent as one request rather than one per cell: a table with four hundred
/// formulas would otherwise be four hundred round trips, and the ORDER —
/// which is the host's whole contribution — would have to be re-established
/// on every one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalRequest {
    pub formulas: Vec<Formula>,
}

/// What one expression came to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaResult {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<FormulaError>,
}

/// Why an expression did not produce a value.
///
/// A message and nothing else. A backend reports what its own language said —
/// a Python `NameError`, a JavaScript `TypeError` — because that message is
/// the one the author can act on, and translating it into a spreadsheet's
/// `#VALUE!` would throw away the only useful part.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaError {
    pub message: String,
}

/// The answers, in any order; the host matches on `id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalResponse {
    pub results: Vec<FormulaResult>,
}

/// A JSON-RPC envelope, as narrow as this protocol needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

/// Frame a message the way LSP does.
pub fn frame(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whole_number_loses_its_decimal_point() {
        // A table of counts full of `3.0` is a table that has been through a
        // computer; the point of writing back into a CSV is that a person
        // still recognises their data.
        assert_eq!(Value::Number { value: 3.0 }.to_cell(), "3");
        assert_eq!(Value::Number { value: -12.0 }.to_cell(), "-12");
        assert_eq!(Value::Number { value: 2.5 }.to_cell(), "2.5");
    }

    #[test]
    fn a_huge_number_keeps_its_float_spelling_rather_than_wrapping() {
        let big = Value::Number { value: 1e18 };
        assert!(
            big.to_cell().contains('e') || big.to_cell().len() > 15,
            "{}",
            big.to_cell()
        );
    }

    #[test]
    fn an_empty_cell_is_not_an_empty_string() {
        // Summing a column should skip blanks rather than treat them as
        // zero-length text.
        assert_eq!(Value::from_cell(""), Value::Empty);
        assert_eq!(Value::from_cell("   "), Value::Empty);
        assert_eq!(Value::Empty.to_cell(), "");
    }

    #[test]
    fn a_cell_that_looks_like_a_number_is_one() {
        assert_eq!(Value::from_cell("42"), Value::Number { value: 42.0 });
        assert_eq!(Value::from_cell(" -3.5 "), Value::Number { value: -3.5 });
        assert_eq!(
            Value::from_cell("007"),
            Value::Number { value: 7.0 },
            "a spreadsheet reads a leading zero as a number too"
        );
    }

    #[test]
    fn anything_else_is_text_with_its_spacing_intact() {
        assert_eq!(
            Value::from_cell(" hello "),
            Value::Text {
                value: " hello ".to_string()
            }
        );
    }

    #[test]
    fn framing_counts_bytes_not_characters() {
        // A formula holding an em dash is one character and three bytes; a
        // reader counting characters would truncate the message.
        let body = "{\"a\":\"—\"}";
        assert!(frame(body).starts_with(&format!("Content-Length: {}", body.len())));
        assert_eq!(body.len(), 11, "nine characters, eleven bytes");
    }

    #[test]
    fn a_request_round_trips_through_json() {
        let request = EvalRequest {
            formulas: vec![Formula {
                id: "A1".into(),
                expression: "B1 * 2".into(),
                bindings: vec![("B1".into(), Value::Number { value: 21.0 })],
            }],
        };
        let text = serde_json::to_string(&request).unwrap();
        assert_eq!(serde_json::from_str::<EvalRequest>(&text).unwrap(), request);
    }

    #[test]
    fn a_result_carries_a_value_or_an_error_and_omits_the_other() {
        let ok = FormulaResult {
            id: "A1".into(),
            value: Some(Value::Number { value: 1.0 }),
            error: None,
        };
        let text = serde_json::to_string(&ok).unwrap();
        assert!(!text.contains("error"), "{text}");
    }
}
