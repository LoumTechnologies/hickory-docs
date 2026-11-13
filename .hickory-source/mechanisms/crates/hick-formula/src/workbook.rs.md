<dataflow:doc xmlns:dataflow="http://www.hickorydocs.com/1.0">
# Calculate document tables through one cell graph

For maintainers connecting authored tables, formula cells and explicit code runs.

Give an authored table an identifier such as `Prices`, then use `Prices!B2`,
`Prices!$B$2`, `sum(Prices!B2:B4)` or `sum(Prices!B:B)` in another table's
formula. The table ID is stable; cell coordinates are positional. Local row and
column operations preserve references to other tables. Ranges and columns are
selectors over cells, rather than separate graph node kinds.

A Python block declares `tables="Orders"` and reads `table("Orders").rows` or
`table("Orders")["Total"]`. The same declared input also exposes authored source
through `table("Orders", raw=True)`. A block with `table="Summary"` publishes
exactly one complete `display_table(...)` output. Its strings remain data even
when they begin with `=`. Formula reads may reuse an exact saved recording but
never run a producing block implicitly. The existing execution DAG orders
producer blocks; the cell graph resolves formula dependencies between them.

The browser and native adapters send immutable document snapshots to this same
resolver. Receipt refresh changes calculation identity independently of source,
so dependent formulas can update without rebuilding an editor. Missing or
unusable outputs block their consumers while independent authored tables remain
available. Physical mobile gesture acceptance remains separate from compilation
and the shared local-bundle browser checks.

## Name tables and preserve typed published values

A table ID is a stable document-local identifier. Formula addresses remain positional within it. Generated rows are already values, so a string beginning with equals must never become executable source.

<dataflow:copy id="workbook-model">
//! Document-local formula dependencies. Backend sessions still evaluate only expressions.
use crate::{CellRef, Formula, Session, Value};
use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const MAX_CELLS: usize = 20_000;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Table {
    pub id: String,
    pub language: Option<String>,
    pub rows: Vec<Vec<Json>>,
    #[serde(default = "yes")]
    pub header: bool,
    #[serde(default)]
    pub generated: bool,
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub unavailable: Option<String>,
}
fn yes() -> bool {
    true
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub table: String,
    pub at: CellRef,
}
impl Cell {
    pub fn label(&self) -> String {
        format!("{}!{}", self.table, self.at.label())
    }
}
#[derive(Clone, Debug)]
pub struct Reference {
    pub start: usize,
    pub end: usize,
    pub cells: Vec<Cell>,
    pub range: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct Step {
    pub cell: String,
    pub level: usize,
    pub expression: String,
    pub bindings: Vec<(String, Value)>,
    pub value: Option<String>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default)]
pub struct Evaluation {
    pub values: BTreeMap<Cell, Value>,
    pub errors: BTreeMap<Cell, String>,
    pub steps: Vec<Step>,
}
#[derive(Clone, Debug)]
pub struct Workbook {
    pub tables: BTreeMap<String, Table>,
}
</dataflow:copy>

## Resolve cells and schedule real dependencies

Resolve each reference to exact cells before calling a language backend. Ranges and columns expand to read sets. Scheduling those cells permits two tables to depend on independent cells in each other; collapsing the tables would invent a cycle. Failed dependencies refuse downstream evaluation, and limits bound both cells and graph edges.

<dataflow:copy id="workbook-graph">
impl Workbook {
    pub fn new(tables: Vec<Table>) -> Result<Self> {
        let mut book = Self {
            tables: BTreeMap::new(),
        };
        let mut count = 0;
        for table in tables {
            ensure!(
                !table.id.is_empty()
                    && table.id.len() <= 64
                    && table
                        .id
                        .bytes()
                        .enumerate()
                        .all(|(i, c)| c.is_ascii_alphabetic()
                            || c == b'_'
                            || i > 0 && c.is_ascii_digit()),
                "Table IDs must be identifiers such as Prices or OrderLines; rename {:?}",
                table.id
            );
            count += table.rows.len() * table.rows.iter().map(Vec::len).max().unwrap_or(0);
            ensure!(
                count <= MAX_CELLS,
                "Document tables exceed {MAX_CELLS} cells; use a code block for larger datasets"
            );
            ensure!(
                !book.tables.contains_key(&table.id),
                "Duplicate table ID {}; choose a unique ID",
                table.id
            );
            book.tables.insert(table.id.clone(), table);
        }
        Ok(book)
    }
    pub fn selector(&self, selector: &str, local: &str) -> Result<Vec<Cell>> {
        let (name, address) = selector.split_once('!').unwrap_or((local, selector));
        let table = self
            .tables
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown table {name}; declare a table with that id"))?;
        ensure!(
            !table.pending,
            "Table {name} is unavailable: {}",
            table
                .unavailable
                .as_deref()
                .unwrap_or("no current successful output; run its producing code block")
        );
        let width = table.rows.iter().map(Vec::len).max().unwrap_or(0);
        let mut out = Vec::new();
        let (first, last) = if address.is_empty() {
            (
                CellRef::new(0, 0),
                CellRef::new(width.saturating_sub(1), table.rows.len().saturating_sub(1)),
            )
        } else if let Some((a, b)) = address.split_once(':') {
            let a = a.replace('$', "");
            let b = b.replace('$', "");
            if a.bytes().all(|c| c.is_ascii_alphabetic())
                && b.bytes().all(|c| c.is_ascii_alphabetic())
            {
                if table.rows.len() <= usize::from(table.header) { return Ok(out); }
                let first = CellRef::parse(&format!("{a}{}", 1 + usize::from(table.header)))
                    .ok_or_else(|| anyhow::anyhow!("Invalid column {a}"))?;
                let last = CellRef::parse(&format!("{b}1"))
                    .ok_or_else(|| anyhow::anyhow!("Invalid column {b}"))?;
                (
                    first,
                    CellRef::new(last.column, table.rows.len().saturating_sub(1)),
                )
            } else {
                (
                    CellRef::parse(&a).ok_or_else(|| anyhow::anyhow!("Invalid cell {a}"))?,
                    CellRef::parse(&b).ok_or_else(|| anyhow::anyhow!("Invalid cell {b}"))?,
                )
            }
        } else {
            let at =
                CellRef::parse(address).ok_or_else(|| anyhow::anyhow!("Invalid cell {address}"))?;
            (at, at)
        };
        if address.is_empty() && (width == 0 || table.rows.is_empty()) {
            return Ok(out);
        }
        let rows = first.row.abs_diff(last.row) + 1;
        let columns = first.column.abs_diff(last.column) + 1;
        ensure!(
            rows.checked_mul(columns).is_some_and(|n| n <= MAX_CELLS),
            "Range {selector} exceeds {MAX_CELLS} cells"
        );
        for row in first.row.min(last.row)..=first.row.max(last.row) {
            for column in first.column.min(last.column)..=first.column.max(last.column) {
                out.push(Cell {
                    table: name.into(),
                    at: CellRef::new(column, row),
                });
            }
        }
        Ok(out)
    }
    pub fn selected(&self, selector: &str) -> Result<Vec<Cell>> {
        if self.tables.contains_key(selector) {
            self.selector(&format!("{selector}!"), selector)
        } else {
            self.selector(selector, "")
        }
    }
    fn formula(&self, cell: &Cell) -> Option<&str> {
        let table = self.tables.get(&cell.table)?;
        if table.generated || table.language.is_none() {
            return None;
        }
        table
            .rows
            .get(cell.at.row)?
            .get(cell.at.column)?
            .as_str()?
            .trim_start()
            .strip_prefix('=')
            .map(str::trim)
    }
    fn literal(&self, cell: &Cell) -> Value {
        let Some(table) = self.tables.get(&cell.table) else {
            return Value::Empty;
        };
        let value = table
            .rows
            .get(cell.at.row)
            .and_then(|row| row.get(cell.at.column))
            .unwrap_or(&Json::Null);
        match value {
            Json::Null => Value::Empty,
            Json::Bool(value) => Value::Bool { value: *value },
            Json::Number(value) => Value::Number {
                value: value.as_f64().unwrap_or(0.),
            },
            Json::String(value) if !table.generated => match Value::from_cell(value) {
                Value::Number { value: n } if !n.is_finite() => Value::Text {
                    value: value.clone(),
                },
                v => v,
            },
            Json::String(value) => Value::Text {
                value: value.clone(),
            },
            Json::Array(values) => Value::List {
                value: values
                    .iter()
                    .map(|v| Value::Text {
                        value: v.to_string(),
                    })
                    .collect(),
            },
            value => Value::Text {
                value: value.to_string(),
            },
        }
    }
    /// Resolve exact tokens before asking any language backend. Quotes/comments are data.
    pub fn references(
        &self,
        expression: &str,
        local: &str,
        language: &str,
    ) -> Result<Vec<Reference>> {
        let bytes = expression.as_bytes();
        let mut i = 0;
        let mut refs = Vec::new();
        let python = matches!(language, "python" | "py" | "python3");
        while i < bytes.len() {
            if python && bytes[i] == b'#' || !python && bytes[i..].starts_with(b"//") {
                i = expression[i..]
                    .find('\n')
                    .map_or(bytes.len(), |n| i + n + 1);
                continue;
            }
            if !python && bytes[i..].starts_with(b"/*") {
                i = expression[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |n| i + n + 4);
                continue;
            }
            if matches!(bytes[i], b'\'' | b'"' | b'`') {
                let q = bytes[i];
                let triple = bytes[i..].starts_with(&[q, q, q]);
                let count = if triple { 3 } else { 1 };
                i += count;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i = (i + 2).min(bytes.len());
                    } else if bytes[i..].starts_with(&vec![q; count]) {
                        i += count;
                        break;
                    } else {
                        i += 1;
                    }
                }
                continue;
            }
            let boundary =
                i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || b"_.".contains(&bytes[i - 1]));
            if !boundary || !(bytes[i].is_ascii_alphabetic() || matches!(bytes[i], b'_' | b'$')) {
                i += 1;
                continue;
            }
            let start = i;
            let mut name_end = i;
            while name_end < bytes.len()
                && (bytes[name_end].is_ascii_alphanumeric() || bytes[name_end] == b'_')
            {
                name_end += 1;
            }
            let qualified = bytes.get(name_end) == Some(&b'!');
            let (name, address_start) = if qualified {
                (&expression[start..name_end], name_end + 1)
            } else {
                (local, start)
            };
            let mut end = address_start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || b"$:".contains(&bytes[end]))
            {
                end += 1;
            }
            let address = &expression[address_start..end];
            let shape = address.split(':').all(|s| CellRef::parse(s).is_some())
                || address.contains(':')
                    && address
                        .replace([':', '$'], "")
                        .bytes()
                        .all(|c| c.is_ascii_alphabetic());
            if end > address_start
                && shape
                && (end == bytes.len()
                    || !(bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_'))
            {
                let cells = self.selector(&format!("{name}!{address}"), local)?;
                refs.push(Reference {
                    start,
                    end,
                    cells,
                    range: address.contains(':'),
                });
                i = end;
            } else {
                i = name_end.max(start + 1);
            }
        }
        ensure!(
            refs.iter().map(|r| r.cells.len()).sum::<usize>() <= MAX_CELLS,
            "Expression references too many cells; reduce its ranges"
        );
        Ok(refs)
    }
    /// Fine-grained closure prevents false cycles caused by collapsing tables.
    pub fn dependencies(&self, selectors: &[String]) -> Result<BTreeSet<Cell>> {
        let mut todo = Vec::new();
        for selector in selectors {
            todo.extend(self.selected(selector)?);
        }
        let mut seen = BTreeSet::new();
        let mut edges = 0;
        while let Some(cell) = todo.pop() {
            if !seen.insert(cell.clone()) {
                continue;
            }
            if let Some(expression) = self.formula(&cell) {
                for reference in self.references(
                    expression,
                    &cell.table,
                    self.tables[&cell.table].language.as_deref().unwrap(),
                )? {
                    edges += reference.cells.len();
                    ensure!(
                        edges <= 200_000,
                        "Too many table dependencies; reduce the selected ranges"
                    );
                    todo.extend(reference.cells);
                }
            }
        }
        Ok(seen)
    }
    pub async fn evaluate(&self, root: &Path, selectors: &[String]) -> Result<Evaluation> {
        let needed = self.dependencies(selectors)?;
        let mut result = Evaluation::default();
        let mut pending = BTreeSet::new();
        let mut reads = BTreeMap::new();
        for cell in needed {
            if let Some(expression) = self.formula(&cell) {
                reads.insert(
                    cell.clone(),
                    self.references(
                        expression,
                        &cell.table,
                        self.tables[&cell.table].language.as_deref().unwrap(),
                    )?,
                );
                pending.insert(cell);
            } else {
                result.values.insert(cell.clone(), self.literal(&cell));
            }
        }
        let mut sessions: BTreeMap<String, Session> = BTreeMap::new();
        let mut level = 0;
        while !pending.is_empty() {
            let ready: Vec<_> = pending
                .iter()
                .filter(|cell| {
                    reads[*cell]
                        .iter()
                        .flat_map(|r| &r.cells)
                        .all(|dep| !pending.contains(dep))
                })
                .cloned()
                .collect();
            if ready.is_empty() {
                let names = pending
                    .iter()
                    .map(Cell::label)
                    .collect::<Vec<_>>()
                    .join(", ");
                for cell in pending {
                    result.errors.insert(cell,format!("Cyclic table dependency, or blocked by one: {names}. Break the circular reference."));
                }
                break;
            }
            let mut batches: BTreeMap<String, Vec<(Cell, Formula, Vec<(String, Value)>)>> =
                BTreeMap::new();
            for cell in ready {
                pending.remove(&cell);
                if let Some(failed) = reads[&cell]
                    .iter()
                    .flat_map(|r| &r.cells)
                    .find(|dep| result.errors.contains_key(*dep))
                {
                    result.errors.insert(
                        cell,
                        format!(
                            "Dependency {} failed; fix it before recalculating",
                            failed.label()
                        ),
                    );
                    continue;
                }
                let mut expression = self.formula(&cell).unwrap().to_string();
                let mut bindings = Vec::new();
                let mut observed = Vec::new();
                for (index, reference) in reads[&cell].iter().enumerate().rev() {
                    let name = format!("__hick_ref_{index}");
                    let values: Vec<_> = reference
                        .cells
                        .iter()
                        .map(|dep| {
                            let value = result.values.get(dep).cloned().unwrap_or(Value::Empty);
                            observed.push((dep.label(), value.clone()));
                            value
                        })
                        .collect();
                    let value = if reference.range {
                        Value::List { value: values }
                    } else {
                        values.into_iter().next().unwrap_or(Value::Empty)
                    };
                    expression.replace_range(reference.start..reference.end, &name);
                    bindings.push((name, value));
                }
                let language = self.tables[&cell.table].language.clone().unwrap();
                batches.entry(language).or_default().push((
                    cell.clone(),
                    Formula {
                        id: cell.label(),
                        expression,
                        bindings,
                    },
                    observed,
                ));
            }
            for (language, batch) in batches {
                if !sessions.contains_key(&language) {
                    sessions.insert(language.clone(), Session::start(root, &language).await?);
                }
                let answer = sessions
                    .get_mut(&language)
                    .unwrap()
                    .evaluate(batch.iter().map(|(_, f, _)| f.clone()).collect())
                    .await?;
                let mut answers: BTreeMap<_, _> = answer
                    .results
                    .into_iter()
                    .map(|a| (a.id.clone(), a))
                    .collect();
                for (cell, _, bindings) in batch {
                    let answer = answers.remove(&cell.label());
                    let mut step = Step {
                        cell: cell.label(),
                        level,
                        expression: self.formula(&cell).unwrap().into(),
                        bindings,
                        value: None,
                        error: None,
                    };
                    match answer {
                        Some(answer) if answer.error.is_none() && answer.value.is_some() => {
                            let value = answer.value.unwrap();
                            step.value = Some(value.to_cell());
                            result.values.insert(cell.clone(), value);
                        }
                        answer => {
                            let error = answer
                                .and_then(|a| a.error)
                                .map(|e| e.message)
                                .unwrap_or_else(|| {
                                    "Formula backend returned no value; retry evaluation".into()
                                });
                            step.error = Some(error.clone());
                            result.errors.insert(cell.clone(), error);
                        }
                    }
                    result.steps.push(step);
                }
            }
            level += 1;
        }
        for (_, session) in sessions {
            session.shutdown().await;
        }
        Ok(result)
    }
    /// Export just the declared selection to a code cell; source and values stay separate.
    pub fn data(&self, selector: &str, result: &Evaluation, raw: bool) -> Result<Json> {
        let cells = self.selected(selector)?;
        if let Some(error) = cells.iter().find_map(|cell| result.errors.get(cell)) {
            bail!("{selector}: {error}");
        }
        let name = selector.split('!').next().unwrap();
        let table = &self.tables[name];
        let selected_columns: BTreeSet<_> = cells.iter().map(|c| c.at.column).collect();
        let columns: Vec<_> = selected_columns
            .into_iter()
            .map(|column| {
                if table.header {
                    table
                        .rows
                        .first()
                        .and_then(|r| r.get(column))
                        .map(|v| {
                            v.as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| v.to_string())
                        })
                        .unwrap_or_default()
                } else {
                    CellRef::new(column, 0)
                        .label()
                        .trim_end_matches('1')
                        .to_string()
                }
            })
            .collect();
        let mut rows: BTreeMap<usize, Vec<Json>> = BTreeMap::new();
        for cell in cells {
            if table.header && cell.at.row == 0 {
                continue;
            }
            let value = if raw {
                table
                    .rows
                    .get(cell.at.row)
                    .and_then(|r| r.get(cell.at.column))
                    .cloned()
                    .unwrap_or(Json::Null)
            } else {
                json_value(result.values.get(&cell).unwrap_or(&Value::Empty))
            };
            rows.entry(cell.at.row).or_default().push(value);
        }
        let rows: Vec<_> = rows.into_values().collect();
        Ok(serde_json::json!({"columns":columns,"rows":rows}))
    }
}
pub fn json_value(value: &Value) -> Json {
    match value {
        Value::Number { value } => serde_json::json!(value),
        Value::Bool { value } => Json::Bool(*value),
        Value::Text { value } => Json::String(value.clone()),
        Value::List { value } => Json::Array(value.iter().map(json_value).collect()),
        Value::Empty => Json::Null,
    }
}
</dataflow:copy>

## Join authored tables and explicit execution outputs

Discover declarations through the real document parser. A code block publishes one named table; until a current successful result is supplied, that table remains unavailable. A truncated display is useful for reading but cannot silently feed an incomplete calculation.

<dataflow:copy id="workbook-outputs">
/// A published output is a value, never executable formula source. Truncated previews cannot feed calculations.
pub fn output_table(id: &str, output: &str) -> Result<Table> {
    let records: Vec<Json> = output
        .split('\n')
        .filter_map(|line| line.strip_prefix(hick_lang::notebook::DISPLAY_PREFIX))
        .filter_map(|json| serde_json::from_str::<Json>(json).ok())
        .filter(|v| v["kind"] == "table")
        .collect();
    ensure!(
        records.len() == 1,
        "Producer for {id} must display exactly one table; run it and check its output"
    );
    let record = &records[0];
    ensure!(
        record["has_more"] != true && record["columns_truncated"] != true,
        "Output {id} is a truncated preview; increase its display limit before using it as input"
    );
    let header = record["schema"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Output {id} has no schema"))?
        .iter()
        .map(|c| c["name"].clone())
        .collect();
    let mut rows = vec![header];
    for row in record["rows"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Output {id} has no rows"))?
    {
        rows.push(
            row.as_array()
                .ok_or_else(|| anyhow::anyhow!("Output {id} contains an invalid row"))?
                .clone(),
        );
    }
    Ok(Table {
        id: id.into(),
        language: None,
        rows,
        header: true,
        generated: true,
        pending: false,
        unavailable: None,
    })
}
/// Discover declarations through the language parser; literal source never invents tables.
pub fn document_tables(
    doc: &hick_lang::HickDocument,
    outputs: &BTreeMap<usize, String>,
) -> Result<Vec<Table>> {
    let mut tables = Vec::new();
    for tag in doc.all_tags() {
        if tag.name == "table" {
            if let Some(id) = tag.get_attribute("id") {
                let delimiter = match tag.get_attribute("delimiter") {
                    Some("tab" | "\\t") => '\t',
                    Some(s) => s.chars().next().unwrap_or(','),
                    None => ',',
                };
                let body = tag.text_content();
                let body = body
                    .strip_prefix("\r\n")
                    .or_else(|| body.strip_prefix('\n'))
                    .unwrap_or(&body);
                let rows = hick_lang::notebook::csv_rows(body, delimiter)
                    .into_iter()
                    .map(|row| row.into_iter().map(Json::String).collect())
                    .collect();
                tables.push(Table {
                    id: id.into(),
                    language: tag.get_attribute("language").map(str::to_string),
                    rows,
                    header: tag.get_attribute("header") != Some("false"),
                    generated: false,
                    pending: false,
                    unavailable: None,
                });
            }
        } else if tag.name == "exec" {
            if let Some(id) = tag.get_attribute("table") {
                let published = outputs
                    .get(&tag.source_line)
                    .map(|output| output_table(id, output));
                let table = match published {
                    Some(Ok(table)) => table,
                    other => Table {
                        id: id.into(),
                        language: None,
                        rows: Vec::new(),
                        header: true,
                        generated: true,
                        pending: true,
                        unavailable: other.and_then(Result::err).map(|error| error.to_string()),
                    },
                };
                tables.push(table);
            }
        }
    }
    Ok(tables)
}
</dataflow:copy>

## Check values, cycles, source boundaries and publication

These focused witnesses use real Python evaluation for qualified cells, ranges, absolute references and column selectors. They also distinguish valid cell ordering from table-level apparent cycles, keep quoted text out of the graph and protect generated equals-prefixed strings.

<dataflow:copy id="workbook-witnesses">
#[cfg(test)]
mod tests {
    // Contract: .hickory-source/mechanisms/crates/hick-formula/src/workbook.rs.md
    use super::*;
    fn table(id: &str, rows: &[&[&str]]) -> Table {
        Table {
            id: id.into(),
            language: Some("python".into()),
            rows: rows
                .iter()
                .map(|r| r.iter().map(|v| Json::String((*v).into())).collect())
                .collect(),
            header: false,
            generated: false,
            pending: false,
            unavailable: None,
        }
    }
    fn value<'a>(result: &'a Evaluation, name: &str, address: &str) -> &'a Value {
        &result.values[&Cell {
            table: name.into(),
            at: CellRef::parse(address).unwrap(),
        }]
    }
    #[tokio::test]
    async fn whole_columns_skip_declared_headers() -> Result<()> {
        let mut prices = table("Prices", &[&["Price"], &["3"], &["5"]]); prices.header = true;
        let book = Workbook::new(vec![prices, table("Result", &[&["=sum(Prices!A:A)"]])])?;
        let root = tempfile::tempdir()?;
        let result = book.evaluate(root.path(), &["Result".into()]).await?;
        assert_eq!(value(&result, "Result", "A1"), &Value::Number { value: 8. });
        Ok(())
    }
    #[test]
    fn an_unusable_output_does_not_block_independent_tables() -> Result<()> {
        let doc = hick_lang::parse(
            r#"<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:table id="Input">Value
7
</h:table><h:exec table="Output">pass</h:exec></h:doc>"#,
        )?;
        let line = doc.find_tags("exec")[0].source_line;
        let book = Workbook::new(document_tables(
            &doc,
            &BTreeMap::from([(line, "not a table".into())]),
        )?)?;
        assert!(book.dependencies(&["Input".into()]).is_ok());
        assert!(
            book.dependencies(&["Output".into()])
                .unwrap_err()
                .to_string()
                .contains("exactly one table")
        );
        Ok(())
    }
    #[tokio::test]
    async fn named_cells_ranges_columns_and_absolute_references_share_one_graph() -> Result<()> {
        let book = Workbook::new(vec![
            table("Prices", &[&["2"], &["3"]]),
            table(
                "Order",
                &[
                    &["=sum(Prices!A1:A2) + Prices!$A$1"],
                    &["=sum(Prices!A:A)"],
                    &["=A1*2"],
                ],
            ),
        ])?;
        let root = tempfile::tempdir()?;
        let result = book.evaluate(root.path(), &["Order".into()]).await?;
        assert_eq!(value(&result, "Order", "A1"), &Value::Number { value: 7. });
        assert_eq!(value(&result, "Order", "A2"), &Value::Number { value: 5. });
        assert_eq!(value(&result, "Order", "A3"), &Value::Number { value: 14. });
        assert!(result.errors.is_empty());
        Ok(())
    }
    #[tokio::test]
    async fn table_level_cycles_are_not_cell_level_cycles_and_real_cycles_fail() -> Result<()> {
        let root = tempfile::tempdir()?;
        let book = Workbook::new(vec![
            table("A", &[&["2", "=B!A1*3"]]),
            table("B", &[&["=A!A1+1"]]),
        ])?;
        let result = book.evaluate(root.path(), &["A".into()]).await?;
        assert_eq!(value(&result, "A", "B1"), &Value::Number { value: 9. });
        let cyclic = Workbook::new(vec![table("A", &[&["=B!A1"]]), table("B", &[&["=A!A1"]])])?;
        let result = cyclic.evaluate(root.path(), &["A".into()]).await?;
        assert_eq!(result.errors.len(), 2);
        assert!(result.steps.is_empty());
        Ok(())
    }
    #[test]
    fn references_skip_strings_comments_and_bound_ranges() -> Result<()> {
        let book = Workbook::new(vec![table("A", &[&["1"]])])?;
        assert_eq!(
            book.references("A!$A$1 + 'A!B2' # A!C3", "A", "python")?
                .len(),
            1
        );
        assert!(book.references("Missing!A1", "A", "python").is_err());
        assert!(book.references("A!A1:ZZ999999999", "A", "python").is_err());
        assert!(Workbook::new(vec![table("A", &[]), table("A", &[])]).is_err());
        Ok(())
    }
    #[tokio::test]
    async fn code_reads_typed_values_or_explicit_raw_source_and_selected_columns() -> Result<()> {
        let mut input = table(
            "Orders",
            &[&["Item", "Total"], &["Pen", "=2+3"], &["Book", "=3*4"]],
        );
        input.header = true;
        let book = Workbook::new(vec![input])?;
        let root = tempfile::tempdir()?;
        let result = book.evaluate(root.path(), &["Orders!B:B".into()]).await?;
        assert_eq!(
            book.data("Orders!B:B", &result, false)?,
            serde_json::json!({"columns":["Total"],"rows":[[5.],[12.]]})
        );
        assert_eq!(
            book.data("Orders!B2:B3", &result, true)?["rows"][0][0],
            "=2+3"
        );
        Ok(())
    }
    #[test]
    fn generated_output_is_literal_and_truncated_output_cannot_feed_formulas() -> Result<()> {
        let output = format!(
            "{}{}\n",
            hick_lang::notebook::DISPLAY_PREFIX,
            serde_json::json!({"kind":"table","schema":[{"name":"text","dtype":"str"}],"rows":[["=danger()"]],"has_more":false})
        );
        let generated = output_table("Result", &output)?;
        let book = Workbook::new(vec![generated])?;
        let cell = Cell {
            table: "Result".into(),
            at: CellRef::new(0, 1),
        };
        assert!(book.formula(&cell).is_none());
        assert_eq!(
            book.literal(&cell),
            Value::Text {
                value: "=danger()".into()
            }
        );
        assert!(output_table("Result", &output.replace("false", "true")).is_err());
        Ok(())
    }
}
</dataflow:copy>

<dataflow:file path="crates/hick-formula/src/workbook.rs"><dataflow:paste select="#workbook-model"/><dataflow:paste select="#workbook-graph"/><dataflow:paste select="#workbook-outputs"/><dataflow:paste select="#workbook-witnesses"/></dataflow:file>
</dataflow:doc>
