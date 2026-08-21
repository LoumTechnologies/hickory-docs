//! Evaluating a table's formulas.
//!
//! One route, because the protocol's whole design is that the interesting
//! work — references, ordering, cycles — is the host's and is already done by
//! the time a backend is asked anything (see `hick_formula`). What is left
//! here is turning a CSV grid into a sheet, handing it over, and turning the
//! answer back into cells.
//!
//! The backend installs itself on first use. That is affordable *because* it
//! is a local write of a script this binary carries — there is no network
//! path, so "automatic" costs nothing and cannot fail halfway. It is exactly
//! the property `hick lsp install` cannot have, which is why that one asks.

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use hick_formula::{CellRef, Sheet};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// A table bigger than this is not a document's table any more, and
/// evaluating it would tie up an interpreter for a long time to fill a grid
/// nobody is reading.
const MAX_CELLS: usize = 20_000;

#[derive(Deserialize)]
pub struct EvaluateBody {
    /// The language formulas in this table are written in.
    pub language: String,
    /// The grid, row-major, exactly as the CSV parses. Ragged rows are fine.
    pub rows: Vec<Vec<String>>,
}

/// `POST /api/formula/evaluate` — every formula in a grid, computed.
///
/// Answers `{values, errors}` keyed by A1 label, and only for cells that are
/// formulas: a literal has nothing to compute and echoing it back would make
/// the response the size of the table for no reason.
pub async fn evaluate(
    State(state): State<LocalState>,
    Json(body): Json<EvaluateBody>,
) -> ApiResult<Json<Value>> {
    let cells: usize = body.rows.iter().map(Vec::len).sum();
    if cells > MAX_CELLS {
        return Err(ApiError::unprocessable(format!(
            "this table has {cells} cells, past the {MAX_CELLS}-cell limit for \
             formula evaluation.\n  \
             A table this size is a dataset rather than a spreadsheet — compute \
             it in an exec cell and let the document write the result."
        )));
    }

    let mut sheet = Sheet::new();
    for (row, fields) in body.rows.iter().enumerate() {
        for (column, text) in fields.iter().enumerate() {
            if text.trim().is_empty() {
                continue;
            }
            sheet.insert(CellRef::new(column, row), text.clone());
        }
    }

    let root = state.index.root().to_path_buf();
    let language = body.language.clone();
    let computed = hick_formula::evaluate_sheet(&root, &language, &sheet)
        .await
        .map_err(|e| {
            // A missing interpreter is the common case and is not a server
            // fault: the table still renders, it just does not compute.
            ApiError::unprocessable(format!("{e:#}"))
        })?;

    let values: serde_json::Map<String, Value> = computed
        .values
        .iter()
        .map(|(cell, text)| (cell.label(), Value::String(text.clone())))
        .collect();
    let errors: serde_json::Map<String, Value> = computed
        .errors
        .iter()
        .map(|(cell, message)| (cell.label(), Value::String(message.clone())))
        .collect();

    Ok(Json(json!({ "values": values, "errors": errors })))
}

/// `GET /api/formula/languages` — what this machine can evaluate right now.
///
/// The UI asks so it can offer the languages that will work rather than the
/// ones this build knows about. A machine with no node should not be offered
/// JavaScript formulas and then told no.
pub async fn languages(State(_state): State<LocalState>) -> Json<Value> {
    Json(json!({ "languages": hick_formula::available() }))
}
