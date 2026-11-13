<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/formula.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c63d0db349cf"></a>
## `crates/hickory-cli/src/serve/formula.rs`

Related contracts: [A Formula Is An Expression In A Real Language](../../../../../guarantees/execution/a-formula-is-an-expression-in-a-real-language/contract.md), [Stepping A Table Replays The Order The Host Chose](../../../../../guarantees/execution/stepping-a-table-replays-the-order-the-host-chose/contract.md).

<a id="f-c63d0db349cf-MAX_CELLS"></a>
### `MAX_CELLS`

Evaluating a table's formulas.
Two routes over one evaluation, because the protocol's whole design is that
the interesting work — references, ordering, cycles — is the host's and is
already done by the time a backend is asked anything (see `hick_formula`).
What is left here is turning a CSV grid into a sheet, handing it over, and
turning the answer back into cells.
`evaluate` answers what the grid displays. `trace` answers the same
evaluation with every cell's turn kept, which is what the table's debugger
steps through — the same code path, so the two can never disagree about
what happened.
The backend installs itself on first use. That is affordable *because* it
is a local write of a script this binary carries — there is no network
path, so "automatic" costs nothing and cannot fail halfway. It is exactly
the property `hick lsp install` cannot have, which is why that one asks.

<storycode:copy id="f-c63d0db349cf-MAX_CELLS">
//! Evaluating a table's formulas.
//!
//! Two routes over one evaluation, because the protocol's whole design is that
//! the interesting work — references, ordering, cycles — is the host's and is
//! already done by the time a backend is asked anything (see `hick_formula`).
//! What is left here is turning a CSV grid into a sheet, handing it over, and
//! turning the answer back into cells.
//!
//! `evaluate` answers what the grid displays. `trace` answers the same
//! evaluation with every cell's turn kept, which is what the table's debugger
//! steps through — the same code path, so the two can never disagree about
//! what happened.
//!
//! The backend installs itself on first use. That is affordable *because* it
//! is a local write of a script this binary carries — there is no network
//! path, so "automatic" costs nothing and cannot fail halfway. It is exactly
//! the property `hick lsp install` cannot have, which is why that one asks.

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use hick_formula::{CellRef, Sheet, Value as FormulaValue};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// A table bigger than this is not a document's table any more, and
/// evaluating it would tie up an interpreter for a long time to fill a grid
/// nobody is reading.
const MAX_CELLS: usize = 20_000;

</storycode:copy>
<a id="f-c63d0db349cf-EvaluateBody"></a>
### `EvaluateBody`

Document-scoped requests resolve named tables from one immutable editor snapshot. The request’s current rows must agree with that source before evaluation. A generated table is admitted only from a replay whose saved source matches exactly; missing or stale producer output never triggers execution during a formula read. Ordinary standalone CSV keeps its existing single-sheet request shape.

<storycode:copy id="f-c63d0db349cf-EvaluateBody">
#[derive(Deserialize)]
pub struct EvaluateBody {
    /// The language formulas in this table are written in.
    pub language: String,
    /// The grid, row-major, exactly as the CSV parses. Ragged rows are fine.
    pub rows: Vec<Vec<String>>,
    #[serde(default)]
    pub scope: Option<FormulaScope>,
}

/// Scope is an immutable editor snapshot. Generated values are borrowed only
/// from an exact matching saved run; observing a formula never runs a producer.
#[derive(Deserialize)]
pub struct FormulaScope {
    pub source: String,
    pub table: String,
    pub document: Option<String>,
}
async fn scoped(state: &LocalState, body: &EvaluateBody, trace: bool) -> ApiResult<Json<Value>> {
    let scope = body.scope.as_ref().unwrap();
    if scope.source.len() > 1_000_000 {
        return Err(ApiError::unprocessable(
            "Document is too large for inline table calculation; use a code block",
        ));
    }
    let doc = hick_lang::parse(&scope.source).map_err(|e| {
        ApiError::unprocessable(format!(
            "Save a valid document before resolving table references: {e}"
        ))
    })?;
    let mut outputs = std::collections::BTreeMap::new();
    if doc
        .find_tags("exec")
        .iter()
        .any(|tag| tag.get_attribute("table").is_some())
    {
        if let Some(id) = &scope.document {
            if let Ok(run) = state.weave(id).await {
                if run.source == scope.source {
                    outputs = hick_literate::table_inputs::recorded_outputs(
                        &run.doc,
                        &run.result.transcripts,
                    );
                }
            }
        }
    }
    let mut tables =
        hick_formula::workbook::document_tables(&doc, &outputs).map_err(missing_interpreter)?;
    let table = tables
        .iter_mut()
        .find(|table| table.id == scope.table)
        .ok_or_else(|| {
            ApiError::unprocessable(format!(
                "Unknown table {}; give this table a unique id",
                scope.table
            ))
        })?;
    if serde_json::to_value(&table.rows).ok() != serde_json::to_value(&body.rows).ok() {
        return Err(ApiError::unprocessable(
            "Table source changed; reopen its current grid before calculating",
        ));
    }
    if table.language.is_none() && !table.generated {
        table.language = Some(body.language.clone());
    }
    let book = hick_formula::workbook::Workbook::new(tables).map_err(missing_interpreter)?;
    let result = book
        .evaluate(state.index.root(), &[scope.table.clone()])
        .await
        .map_err(missing_interpreter)?;
    let values: serde_json::Map<_, _> = result
        .values
        .iter()
        .filter(|(cell, _)| {
            cell.table == scope.table
                && body
                    .rows
                    .get(cell.at.row)
                    .and_then(|row| row.get(cell.at.column))
                    .is_some_and(|text| text.trim_start().starts_with('='))
        })
        .map(|(cell, value)| (cell.at.label(), json!(value.to_cell())))
        .collect();
    let errors: serde_json::Map<_, _> = result
        .errors
        .iter()
        .filter(|(cell, _)| cell.table == scope.table)
        .map(|(cell, error)| (cell.at.label(), json!(error)))
        .collect();
    let steps: Vec<_> = result.steps.iter().map(|step|json!({"cell":step.cell.strip_prefix(&format!("{}!",scope.table)).unwrap_or(&step.cell),"level":step.level,"expression":step.expression,"value":step.value,"error":step.error,"bindings":step.bindings.iter().map(|(cell,value)|json!({"cell":cell,"text":value.to_cell(),"kind":kind_of(value)})).collect::<Vec<_>>()})).collect();
    if trace {
        Ok(Json(json!({"values":values,"errors":errors,"steps":steps})))
    } else {
        Ok(Json(json!({"values":values,"errors":errors})))
    }
}
</storycode:copy>
<a id="f-c63d0db349cf-evaluate"></a>
### `evaluate`

`POST /api/formula/evaluate` — every formula in a grid, computed.
Answers `{values, errors}` keyed by A1 label, and only for cells that are
formulas: a literal has nothing to compute and echoing it back would make
the response the size of the table for no reason.

<storycode:copy id="f-c63d0db349cf-evaluate">
/// `POST /api/formula/evaluate` — every formula in a grid, computed.
///
/// Answers `{values, errors}` keyed by A1 label, and only for cells that are
/// formulas: a literal has nothing to compute and echoing it back would make
/// the response the size of the table for no reason.
pub async fn evaluate(
    State(state): State<LocalState>,
    Json(body): Json<EvaluateBody>,
) -> ApiResult<Json<Value>> {
    if body.scope.is_some() { return scoped(&state, &body, false).await; }
    let sheet = sheet_of(&body)?;
    let root = state.index.root().to_path_buf();
    let computed = hick_formula::evaluate_sheet(&root, &body.language, &sheet)
        .await
        .map_err(missing_interpreter)?;

    Ok(Json(json!({
        "values": labelled(&computed.values),
        "errors": labelled(&computed.errors),
    })))
}

</storycode:copy>
<a id="f-c63d0db349cf-trace"></a>
### `trace`

`POST /api/formula/trace` — the same evaluation, cell by cell.
What the table's debugger steps through. It is deliberately the same code
path as `evaluate`: `trace_sheet` IS the evaluator, and `evaluate_sheet` is
it with the steps dropped. A debugger walking its own copy of the order
would eventually disagree with the grid about what happened, and a
debugger that disagrees with the program is worse than none.
The steps are the HOST's contribution made visible — which cell went when,
what it read, and what the value it read was worth at that moment. Stepping
*inside* an expression is a different tool: that is the language's
debugger, and it is `hick-dap`'s job.

<storycode:copy id="f-c63d0db349cf-trace">
/// `POST /api/formula/trace` — the same evaluation, cell by cell.
///
/// What the table's debugger steps through. It is deliberately the same code
/// path as `evaluate`: `trace_sheet` IS the evaluator, and `evaluate_sheet` is
/// it with the steps dropped. A debugger walking its own copy of the order
/// would eventually disagree with the grid about what happened, and a
/// debugger that disagrees with the program is worse than none.
///
/// The steps are the HOST's contribution made visible — which cell went when,
/// what it read, and what the value it read was worth at that moment. Stepping
/// *inside* an expression is a different tool: that is the language's
/// debugger, and it is `hick-dap`'s job.
pub async fn trace(
    State(state): State<LocalState>,
    Json(body): Json<EvaluateBody>,
) -> ApiResult<Json<Value>> {
    if body.scope.is_some() { return scoped(&state, &body, true).await; }
    let sheet = sheet_of(&body)?;
    let root = state.index.root().to_path_buf();
    let traced = hick_formula::trace_sheet(&root, &body.language, &sheet)
        .await
        .map_err(missing_interpreter)?;

    let steps: Vec<Value> = traced
        .steps
        .iter()
        .map(|step| {
            let bindings: Vec<Value> = step
                .bindings
                .iter()
                .map(|(cell, value)| {
                    json!({
                        "cell": cell.label(),
                        // Both, because they answer different questions: the
                        // text is what the expression saw, and the kind is
                        // why an empty cell is not the empty string.
                        "text": value.to_cell(),
                        "kind": kind_of(value),
                    })
                })
                .collect();
            json!({
                "cell": step.cell.label(),
                "level": step.level,
                "expression": step.expression,
                "bindings": bindings,
                "value": step.value,
                "error": step.error,
            })
        })
        .collect();

    Ok(Json(json!({
        "steps": steps,
        "values": labelled(&traced.computed.values),
        "errors": labelled(&traced.computed.errors),
    })))
}

</storycode:copy>
<a id="f-c63d0db349cf-sheet_of"></a>
### `sheet_of`

The grid as a sheet, refusing a table that is really a dataset.

<storycode:copy id="f-c63d0db349cf-sheet_of">
/// The grid as a sheet, refusing a table that is really a dataset.
fn sheet_of(body: &EvaluateBody) -> Result<Sheet, ApiError> {
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
    Ok(sheet)
}

</storycode:copy>
<a id="f-c63d0db349cf-missing_interpreter"></a>
### `missing_interpreter`

A missing interpreter is the common case and is not a server fault: the
table still renders, it just does not compute.

<storycode:copy id="f-c63d0db349cf-missing_interpreter">
/// A missing interpreter is the common case and is not a server fault: the
/// table still renders, it just does not compute.
fn missing_interpreter(error: anyhow::Error) -> ApiError {
    ApiError::unprocessable(format!("{error:#}"))
}

</storycode:copy>
<a id="f-c63d0db349cf-labelled"></a>
### `labelled`

A map keyed by cell, keyed by A1 label instead.

<storycode:copy id="f-c63d0db349cf-labelled">
/// A map keyed by cell, keyed by A1 label instead.
fn labelled(map: &std::collections::BTreeMap<CellRef, String>) -> serde_json::Map<String, Value> {
    map.iter()
        .map(|(cell, text)| (cell.label(), Value::String(text.clone())))
        .collect()
}

</storycode:copy>
<a id="f-c63d0db349cf-kind_of"></a>
### `kind_of`

Which sort of value a binding was. `empty` is the one worth naming: a
blank cell is not the empty string, and a debugger that showed both as
nothing would hide the difference that made `sum` skip it.

<storycode:copy id="f-c63d0db349cf-kind_of">
/// Which sort of value a binding was. `empty` is the one worth naming: a
/// blank cell is not the empty string, and a debugger that showed both as
/// nothing would hide the difference that made `sum` skip it.
fn kind_of(value: &FormulaValue) -> &'static str {
    match value {
        FormulaValue::Number { .. } => "number",
        FormulaValue::Text { .. } => "text",
        FormulaValue::Bool { .. } => "bool",
        FormulaValue::List { .. } => "list",
        FormulaValue::Empty => "empty",
    }
}

</storycode:copy>
<a id="f-c63d0db349cf-languages"></a>
### `languages`

`GET /api/formula/languages` — what this machine can evaluate right now.
The UI asks so it can offer the languages that will work rather than the
ones this build knows about. A machine with no node should not be offered
JavaScript formulas and then told no.

<storycode:copy id="f-c63d0db349cf-languages">
/// `GET /api/formula/languages` — what this machine can evaluate right now.
///
/// The UI asks so it can offer the languages that will work rather than the
/// ones this build knows about. A machine with no node should not be offered
/// JavaScript formulas and then told no.
pub async fn languages(State(_state): State<LocalState>) -> Json<Value> {
    Json(json!({ "languages": hick_formula::available() }))
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/formula.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/formula.rs"><storycode:paste select="#f-c63d0db349cf-MAX_CELLS"/><storycode:paste select="#f-c63d0db349cf-EvaluateBody"/><storycode:paste select="#f-c63d0db349cf-evaluate"/><storycode:paste select="#f-c63d0db349cf-trace"/><storycode:paste select="#f-c63d0db349cf-sheet_of"/><storycode:paste select="#f-c63d0db349cf-missing_interpreter"/><storycode:paste select="#f-c63d0db349cf-labelled"/><storycode:paste select="#f-c63d0db349cf-kind_of"/><storycode:paste select="#f-c63d0db349cf-languages"/></storycode:file>

</storycode:doc>
