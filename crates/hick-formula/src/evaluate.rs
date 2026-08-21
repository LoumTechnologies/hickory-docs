//! Evaluating a whole table: where the host's graph meets a backend.
//!
//! # Batching, and why it is by LEVEL
//!
//! Two things pull against each other. Round trips are expensive, so a table
//! with four hundred formulas should not be four hundred requests. But a
//! formula's bindings are values, resolved by the host before it asks — and a
//! formula depending on another cannot have its bindings until that one has
//! been answered.
//!
//! Sending one batch and asking the backend to feed results forward would fix
//! the round trips by giving the backend a dependency graph, which is exactly
//! the thing this design keeps out of backends (see `lib.rs`).
//!
//! So the batch is a **level**: every cell whose dependencies are already
//! known goes in one request. Cells at the same level cannot depend on each
//! other by construction, so all their bindings exist. The number of round
//! trips becomes the DEPTH of the graph rather than its size — a column of
//! four hundred independent formulas is one request, and a chain of four
//! hundred is four hundred, which is the honest cost of a chain.

use std::collections::BTreeMap;

use anyhow::Result;

use crate::backend;
use crate::graph::{CellRef, Sheet, expression_of, is_formula, references_in};
use crate::protocol::{Formula, Value};
use crate::session::Session;

/// What a sheet came to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Computed {
    /// Every formula cell's value, as it should be displayed.
    pub values: BTreeMap<CellRef, String>,
    /// Every formula cell that failed, and what its language said.
    pub errors: BTreeMap<CellRef, String>,
}

/// The formula cells, grouped so that everything in one group depends only on
/// earlier groups.
///
/// Returns the cycle instead when there is one — with every cell in it marked,
/// because a cycle has no order and pretending otherwise would evaluate half
/// of it against stale values.
pub fn levels(sheet: &Sheet) -> Result<Vec<Vec<CellRef>>, crate::graph::Cycle> {
    let order = crate::graph::evaluation_order(sheet)?;
    let formulas: BTreeMap<CellRef, Vec<CellRef>> = sheet
        .iter()
        .filter(|(_, text)| is_formula(text))
        .map(|(cell, text)| (*cell, references_in(expression_of(text))))
        .collect();

    let mut level_of: BTreeMap<CellRef, usize> = BTreeMap::new();
    for cell in &order {
        // `order` is topological, so every formula dependency already has a
        // level by the time this runs.
        let level = formulas
            .get(cell)
            .map(|deps| {
                deps.iter()
                    .filter_map(|d| level_of.get(d))
                    .map(|l| l + 1)
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        level_of.insert(*cell, level);
    }
    let depth = level_of.values().copied().max().map(|d| d + 1).unwrap_or(0);
    let mut out = vec![Vec::new(); depth];
    for (cell, level) in level_of {
        out[level].push(cell);
    }
    Ok(out)
}

/// Evaluate every formula in `sheet` with the backend for `language`.
///
/// Literal cells are read as values and never sent anywhere — there is
/// nothing to evaluate in `42`, and shipping the whole table to a subprocess
/// to be told so would be absurd.
pub async fn evaluate_sheet(
    root: &std::path::Path,
    language: &str,
    sheet: &Sheet,
) -> Result<Computed> {
    let mut computed = Computed::default();

    let groups = match levels(sheet) {
        Ok(groups) => groups,
        Err(cycle) => {
            // Every cell in the circle gets the same message, naming all of
            // them. Marking one and evaluating the rest would be inventing an
            // order the sheet does not have.
            let message = cycle.message();
            for cell in cycle.cells {
                computed.errors.insert(cell, message.clone());
            }
            return Ok(computed);
        }
    };
    if groups.iter().all(Vec::is_empty) {
        return Ok(computed);
    }

    // Start from the literals: what every formula's references resolve to
    // before anything has been evaluated.
    let mut resolved: BTreeMap<CellRef, Value> = sheet
        .iter()
        .filter(|(_, text)| !is_formula(text))
        .map(|(cell, text)| (*cell, Value::from_cell(text)))
        .collect();

    let mut session = Session::start(root, language).await?;
    for group in groups {
        if group.is_empty() {
            continue;
        }
        let formulas: Vec<Formula> = group
            .iter()
            .map(|cell| {
                let expression = expression_of(sheet.get(cell).map(String::as_str).unwrap_or(""));
                let bindings = references_in(expression)
                    .into_iter()
                    .map(|reference| {
                        (
                            reference.label(),
                            // A reference to a cell that is not there is
                            // empty, not an error: a table with a gap in it
                            // is a table, and `sum` over it should work.
                            resolved.get(&reference).cloned().unwrap_or(Value::Empty),
                        )
                    })
                    .collect();
                Formula {
                    id: cell.label(),
                    expression: expression.to_string(),
                    bindings,
                }
            })
            .collect();

        let answer = session.evaluate(formulas).await?;
        for result in answer.results {
            let Some(cell) = CellRef::parse(&result.id) else {
                continue;
            };
            if let Some(error) = result.error {
                computed.errors.insert(cell, error.message);
                // A failed cell resolves to EMPTY for anything downstream,
                // rather than to its own error text — a cell that depends on
                // a broken one should report its own trouble, not inherit a
                // string that happens to be somebody else's message.
                resolved.insert(cell, Value::Empty);
            } else if let Some(value) = result.value {
                computed.values.insert(cell, value.to_cell());
                resolved.insert(cell, value);
            }
        }
    }
    session.shutdown().await;
    Ok(computed)
}

/// Which languages formulas can be evaluated in on this machine right now.
///
/// Not per project, deliberately: a backend installs itself into whichever
/// project asks, so what limits the answer is the INTERPRETER, which belongs
/// to the machine. A signature taking a root would suggest two projects could
/// differ, and they cannot.
pub fn available() -> Vec<&'static str> {
    backend::BACKENDS
        .iter()
        .filter(|b| backend::find_interpreter(b).is_some())
        .map(|b| b.language)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(text: &str) -> CellRef {
        CellRef::parse(text).unwrap()
    }

    fn sheet(entries: &[(&str, &str)]) -> Sheet {
        entries
            .iter()
            .map(|(at, text)| (cell(at), text.to_string()))
            .collect()
    }

    fn labels(groups: &[Vec<CellRef>]) -> Vec<Vec<String>> {
        groups
            .iter()
            .map(|g| g.iter().map(CellRef::label).collect())
            .collect()
    }

    #[test]
    fn independent_formulas_are_one_request() {
        // The whole point of batching by level: a column of four hundred
        // independent formulas is one round trip.
        let groups = levels(&sheet(&[
            ("A1", "1"),
            ("B1", "=A1+1"),
            ("B2", "=A1+2"),
            ("B3", "=A1+3"),
        ]))
        .unwrap();
        assert_eq!(labels(&groups), vec![vec!["B1", "B2", "B3"]]);
    }

    #[test]
    fn a_chain_is_one_request_per_link() {
        // The honest cost of a chain: each link needs the one before it.
        let groups = levels(&sheet(&[
            ("A1", "1"),
            ("A2", "=A1+1"),
            ("A3", "=A2+1"),
            ("A4", "=A3+1"),
        ]))
        .unwrap();
        assert_eq!(labels(&groups), vec![vec!["A2"], vec!["A3"], vec!["A4"]]);
    }

    #[test]
    fn a_cell_sits_below_its_deepest_dependency() {
        let groups = levels(&sheet(&[
            ("A1", "1"),
            ("B1", "=A1"),
            ("C1", "=B1"),
            ("D1", "=A1+C1"),
        ]))
        .unwrap();
        assert_eq!(labels(&groups), vec![vec!["B1"], vec!["C1"], vec!["D1"]]);
    }

    #[test]
    fn a_sheet_of_literals_has_no_levels_at_all() {
        assert!(
            levels(&sheet(&[("A1", "1"), ("B1", "two")]))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_cycle_is_reported_rather_than_layered() {
        let cycle = levels(&sheet(&[("A1", "=B1"), ("B1", "=A1")])).unwrap_err();
        assert_eq!(cycle.cells, vec![cell("A1"), cell("B1")]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_cycle_marks_every_cell_in_it_without_starting_a_backend() {
        // No interpreter is needed to know a circle has no order — and
        // starting one to be told so would be a subprocess for nothing.
        let dir = tempfile::tempdir().unwrap();
        let computed = evaluate_sheet(
            dir.path(),
            "python",
            &sheet(&[("A1", "=B1"), ("B1", "=A1")]),
        )
        .await
        .expect("a cycle is an answer, not a failure");
        assert_eq!(computed.errors.len(), 2);
        assert!(
            computed.errors[&cell("A1")].contains("circle"),
            "{:?}",
            computed.errors
        );
        assert!(computed.values.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_sheet_with_no_formulas_starts_no_backend() {
        let dir = tempfile::tempdir().unwrap();
        let computed = evaluate_sheet(dir.path(), "python", &sheet(&[("A1", "1")]))
            .await
            .expect("nothing to do is not a failure");
        assert_eq!(computed, Computed::default());
        assert!(
            !dir.path().join(".hick-cache").exists(),
            "nothing was installed for a sheet with no formulas"
        );
    }
}
