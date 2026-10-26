<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-formula/tests/backends.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6d63eb1fec85"></a>
## `crates/hick-formula/tests/backends.rs`

Related contracts: [Cell formatting keeps numbers numeric](../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md), [A Formula Is An Expression In A Real Language](../../../../guarantees/execution/a-formula-is-an-expression-in-a-real-language/contract.md), [Stepping A Table Replays The Order The Host Chose](../../../../guarantees/execution/stepping-a-table-replays-the-order-the-host-chose/contract.md).

[Read the test `have` beside its requirement](../../../../guarantees/execution/a-formula-is-an-expression-in-a-real-language/contract.md#f-6d63eb1fec85-have).
<a id="f-6d63eb1fec85-formula"></a>
### `formula`

<storycode:copy id="f-6d63eb1fec85-formula">
fn formula(id: &str, expression: &str, bindings: &[(&str, Value)]) -> Formula {
    Formula {
        id: id.to_string(),
        expression: expression.to_string(),
        bindings: bindings
            .iter()
            .map(|(name, value)| (name.to_string(), value.clone()))
            .collect(),
    }
}

</storycode:copy>
<a id="f-6d63eb1fec85-number"></a>
### `number`

<storycode:copy id="f-6d63eb1fec85-number">
fn number(n: f64) -> Value {
    Value::Number { value: n }
}

</storycode:copy>
<a id="f-6d63eb1fec85-evaluate"></a>
### `evaluate`

<storycode:copy id="f-6d63eb1fec85-evaluate">
async fn evaluate(language: &str, formulas: Vec<Formula>) -> Vec<hick_formula::FormulaResult> {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start(dir.path(), language)
        .await
        .unwrap_or_else(|e| panic!("the {language} backend starts: {e:#}"));
    let answer = session.evaluate(formulas).await.expect("it answers");
    session.shutdown().await;
    answer.results
}

</storycode:copy>
<a id="f-6d63eb1fec85-python_evaluates_an_expression_with_its_references_bound"></a>
### `python_evaluates_an_expression_with_its_references_bound`

<storycode:copy id="f-6d63eb1fec85-python_evaluates_an_expression_with_its_references_bound">
#[tokio::test(flavor = "multi_thread")]
async fn python_evaluates_an_expression_with_its_references_bound() {
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let results = evaluate(
        "python",
        vec![formula(
            "A1",
            "B1 * 2 + C1",
            &[("B1", number(20.0)), ("C1", number(2.0))],
        )],
    )
    .await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value, Some(number(42.0)));
    assert_eq!(results[0].error, None);
}

</storycode:copy>
<a id="f-6d63eb1fec85-python_reports_its_own_error_message_not_a_spreadsheet_code"></a>
### `python_reports_its_own_error_message_not_a_spreadsheet_code`

<storycode:copy id="f-6d63eb1fec85-python_reports_its_own_error_message_not_a_spreadsheet_code">
#[tokio::test(flavor = "multi_thread")]
async fn python_reports_its_own_error_message_not_a_spreadsheet_code() {
    // A NameError naming the thing that is missing is the only part the
    // author can act on; `#VALUE!` throws that away.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let results = evaluate("python", vec![formula("A1", "nope + 1", &[])]).await;
    let message = results[0].error.as_ref().expect("an error").message.clone();
    assert!(message.contains("NameError"), "{message}");
    assert!(message.contains("nope"), "{message}");
    assert_eq!(results[0].value, None);
}

</storycode:copy>
<a id="f-6d63eb1fec85-python_can_aggregate_a_column"></a>
### `python_can_aggregate_a_column`

<storycode:copy id="f-6d63eb1fec85-python_can_aggregate_a_column">
#[tokio::test(flavor = "multi_thread")]
async fn python_can_aggregate_a_column() {
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let results = evaluate(
        "python",
        vec![formula(
            "A4",
            "sum([A1, A2, A3])",
            &[
                ("A1", number(1.0)),
                ("A2", number(2.0)),
                ("A3", number(3.0)),
            ],
        )],
    )
    .await;
    assert_eq!(results[0].value, Some(number(6.0)));
}

</storycode:copy>
<a id="f-6d63eb1fec85-ranges_are_values_in_python_and_javascript"></a>
### `ranges_are_values_in_python_and_javascript`

<storycode:copy id="f-6d63eb1fec85-ranges_are_values_in_python_and_javascript">
#[tokio::test(flavor = "multi_thread")]
async fn ranges_are_values_in_python_and_javascript() {
    let entries = [("A1", "1"), ("A2", "2"), ("A3", "3"), ("B1", "=sum(A1:A3)")];
    for language in ["python", "javascript"] {
        let interpreters = if language == "python" {
            &["python3", "python"][..]
        } else {
            &["node"][..]
        };
        if !have(interpreters) {
            eprintln!("skipped: no {language} interpreter");
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let computed = evaluate_sheet(dir.path(), language, &sheet(&entries))
            .await
            .unwrap();
        assert_eq!(at(&computed, "B1").as_deref(), Some("6"), "{language}");
    }
}

</storycode:copy>
<a id="f-6d63eb1fec85-python_reads_an_empty_cell_as_none_so_blanks_can_be_skipped"></a>
### `python_reads_an_empty_cell_as_none_so_blanks_can_be_skipped`

<storycode:copy id="f-6d63eb1fec85-python_reads_an_empty_cell_as_none_so_blanks_can_be_skipped">
#[tokio::test(flavor = "multi_thread")]
async fn python_reads_an_empty_cell_as_none_so_blanks_can_be_skipped() {
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let results = evaluate(
        "python",
        vec![formula(
            "A3",
            "sum(x for x in [A1, A2] if x is not None)",
            &[("A1", number(5.0)), ("A2", Value::Empty)],
        )],
    )
    .await;
    assert_eq!(results[0].value, Some(number(5.0)));
}

</storycode:copy>
<a id="f-6d63eb1fec85-javascript_evaluates_an_expression_with_its_references_bound"></a>
### `javascript_evaluates_an_expression_with_its_references_bound`

<storycode:copy id="f-6d63eb1fec85-javascript_evaluates_an_expression_with_its_references_bound">
#[tokio::test(flavor = "multi_thread")]
async fn javascript_evaluates_an_expression_with_its_references_bound() {
    if !have(&["node"]) {
        eprintln!("skipped: no node on this machine");
        return;
    }
    let results = evaluate(
        "javascript",
        vec![formula(
            "A1",
            "B1 * 2 + C1",
            &[("B1", number(20.0)), ("C1", number(2.0))],
        )],
    )
    .await;
    assert_eq!(results[0].value, Some(number(42.0)));
}

</storycode:copy>
<a id="f-6d63eb1fec85-javascript_reports_its_own_error_message"></a>
### `javascript_reports_its_own_error_message`

<storycode:copy id="f-6d63eb1fec85-javascript_reports_its_own_error_message">
#[tokio::test(flavor = "multi_thread")]
async fn javascript_reports_its_own_error_message() {
    if !have(&["node"]) {
        eprintln!("skipped: no node on this machine");
        return;
    }
    let results = evaluate("javascript", vec![formula("A1", "nope.x", &[])]).await;
    let message = results[0].error.as_ref().expect("an error").message.clone();
    assert!(message.contains("nope"), "{message}");
}

</storycode:copy>
<a id="f-6d63eb1fec85-javascript_returns_text_as_text"></a>
### `javascript_returns_text_as_text`

<storycode:copy id="f-6d63eb1fec85-javascript_returns_text_as_text">
#[tokio::test(flavor = "multi_thread")]
async fn javascript_returns_text_as_text() {
    if !have(&["node"]) {
        eprintln!("skipped: no node on this machine");
        return;
    }
    let results = evaluate(
        "javascript",
        vec![formula(
            "A1",
            "B1.toUpperCase()",
            &[(
                "B1",
                Value::Text {
                    value: "ada".into(),
                },
            )],
        )],
    )
    .await;
    assert_eq!(
        results[0].value,
        Some(Value::Text {
            value: "ADA".to_string()
        })
    );
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_batch_comes_back_whole_and_matched_by_id"></a>
### `a_batch_comes_back_whole_and_matched_by_id`

<storycode:copy id="f-6d63eb1fec85-a_batch_comes_back_whole_and_matched_by_id">
#[tokio::test(flavor = "multi_thread")]
async fn a_batch_comes_back_whole_and_matched_by_id() {
    // One request per batch rather than per cell: four hundred formulas
    // would otherwise be four hundred round trips.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let results = evaluate(
        "python",
        vec![
            formula("A1", "1 + 1", &[]),
            formula("A2", "2 + 2", &[]),
            formula("A3", "bad(", &[]),
        ],
    )
    .await;
    assert_eq!(results.len(), 3);
    let by_id = |id: &str| results.iter().find(|r| r.id == id).unwrap();
    assert_eq!(by_id("A1").value, Some(number(2.0)));
    assert_eq!(by_id("A2").value, Some(number(4.0)));
    // One bad formula fails alone; the rest of the batch still has answers.
    assert!(by_id("A3").error.is_some());
}

</storycode:copy>
<a id="f-6d63eb1fec85-an_unknown_language_says_what_this_build_carries"></a>
### `an_unknown_language_says_what_this_build_carries`

<storycode:copy id="f-6d63eb1fec85-an_unknown_language_says_what_this_build_carries">
#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_language_says_what_this_build_carries() {
    let dir = tempfile::tempdir().unwrap();
    let error = match Session::start(dir.path(), "cobol").await {
        Ok(_) => panic!("cobol must not start"),
        Err(e) => format!("{e:#}"),
    };
    assert!(error.contains("cobol"), "{error}");
}

</storycode:copy>
<a id="f-6d63eb1fec85-an_empty_batch_asks_the_backend_nothing"></a>
### `an_empty_batch_asks_the_backend_nothing`

<storycode:copy id="f-6d63eb1fec85-an_empty_batch_asks_the_backend_nothing">
#[tokio::test(flavor = "multi_thread")]
async fn an_empty_batch_asks_the_backend_nothing() {
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start(dir.path(), "python").await.unwrap();
    assert!(
        session
            .evaluate(Vec::new())
            .await
            .unwrap()
            .results
            .is_empty()
    );
    session.shutdown().await;
}

</storycode:copy>
<a id="f-6d63eb1fec85-use_declaration"></a>
### `use_declaration`

A whole sheet, through the levels, against a real interpreter.

<storycode:copy id="f-6d63eb1fec85-use_declaration">
// ---------------------------------------------------------------------------
// A whole sheet, through the levels, against a real interpreter.
// ---------------------------------------------------------------------------

use hick_formula::{CellRef, Sheet, evaluate_sheet};

</storycode:copy>
<a id="f-6d63eb1fec85-sheet"></a>
### `sheet`

<storycode:copy id="f-6d63eb1fec85-sheet">
fn sheet(entries: &[(&str, &str)]) -> Sheet {
    entries
        .iter()
        .map(|(at, text)| (CellRef::parse(at).unwrap(), text.to_string()))
        .collect()
}

</storycode:copy>
<a id="f-6d63eb1fec85-at"></a>
### `at`

<storycode:copy id="f-6d63eb1fec85-at">
fn at(computed: &hick_formula::Computed, cell: &str) -> Option<String> {
    computed.values.get(&CellRef::parse(cell).unwrap()).cloned()
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_chain_evaluates_in_order_with_each_link_seeing_the_last"></a>
### `a_chain_evaluates_in_order_with_each_link_seeing_the_last`

<storycode:copy id="f-6d63eb1fec85-a_chain_evaluates_in_order_with_each_link_seeing_the_last">
#[tokio::test(flavor = "multi_thread")]
async fn a_chain_evaluates_in_order_with_each_link_seeing_the_last() {
    // The host's whole contribution: A3 must see A2's computed value, not
    // its formula text.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let computed = evaluate_sheet(
        dir.path(),
        "python",
        &sheet(&[("A1", "10"), ("A2", "=A1*2"), ("A3", "=A2+5")]),
    )
    .await
    .unwrap();
    assert_eq!(at(&computed, "A2").as_deref(), Some("20"));
    assert_eq!(at(&computed, "A3").as_deref(), Some("25"));
    assert!(computed.errors.is_empty(), "{:?}", computed.errors);
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_total_row_reads_the_column_above_it"></a>
### `a_total_row_reads_the_column_above_it`

<storycode:copy id="f-6d63eb1fec85-a_total_row_reads_the_column_above_it">
#[tokio::test(flavor = "multi_thread")]
async fn a_total_row_reads_the_column_above_it() {
    // The thing anybody actually opens a spreadsheet to do.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let computed = evaluate_sheet(
        dir.path(),
        "python",
        &sheet(&[
            ("B1", "120"),
            ("B2", "90"),
            ("B3", "35"),
            ("B4", "=sum([B1, B2, B3])"),
        ]),
    )
    .await
    .unwrap();
    assert_eq!(at(&computed, "B4").as_deref(), Some("245"));
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_whole_number_comes_back_without_a_decimal_point"></a>
### `a_whole_number_comes_back_without_a_decimal_point`

<storycode:copy id="f-6d63eb1fec85-a_whole_number_comes_back_without_a_decimal_point">
#[tokio::test(flavor = "multi_thread")]
async fn a_whole_number_comes_back_without_a_decimal_point() {
    // A table of counts full of `245.0` is a table that has been through a
    // computer.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let computed = evaluate_sheet(dir.path(), "python", &sheet(&[("A1", "=1+1")]))
        .await
        .unwrap();
    assert_eq!(at(&computed, "A1").as_deref(), Some("2"));
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_broken_cell_fails_alone_and_leaves_its_dependants_their_own_trouble"></a>
### `a_broken_cell_fails_alone_and_leaves_its_dependants_their_own_trouble`

<storycode:copy id="f-6d63eb1fec85-a_broken_cell_fails_alone_and_leaves_its_dependants_their_own_trouble">
#[tokio::test(flavor = "multi_thread")]
async fn a_broken_cell_fails_alone_and_leaves_its_dependants_their_own_trouble() {
    // A cell that depends on a broken one must report ITS OWN error, not
    // inherit a string that happens to be somebody else's message.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let computed = evaluate_sheet(
        dir.path(),
        "python",
        &sheet(&[("A1", "=missing+1"), ("A2", "=A1*2"), ("A3", "=1+1")]),
    )
    .await
    .unwrap();
    assert!(computed.errors.contains_key(&CellRef::parse("A1").unwrap()));
    // A2 saw an empty A1 and failed on its own terms (None * 2).
    assert!(computed.errors.contains_key(&CellRef::parse("A2").unwrap()));
    let a2 = &computed.errors[&CellRef::parse("A2").unwrap()];
    assert!(!a2.contains("missing"), "A1's message leaked into A2: {a2}");
    // ...and the unrelated cell is fine.
    assert_eq!(at(&computed, "A3").as_deref(), Some("2"));
}

</storycode:copy>
<a id="f-6d63eb1fec85-javascript_evaluates_a_sheet_the_same_way_python_does"></a>
### `javascript_evaluates_a_sheet_the_same_way_python_does`

<storycode:copy id="f-6d63eb1fec85-javascript_evaluates_a_sheet_the_same_way_python_does">
#[tokio::test(flavor = "multi_thread")]
async fn javascript_evaluates_a_sheet_the_same_way_python_does() {
    if !have(&["node"]) {
        eprintln!("skipped: no node on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let computed = evaluate_sheet(
        dir.path(),
        "javascript",
        &sheet(&[("A1", "10"), ("A2", "=A1*2"), ("A3", "=A2+5")]),
    )
    .await
    .unwrap();
    assert_eq!(at(&computed, "A2").as_deref(), Some("20"));
    assert_eq!(at(&computed, "A3").as_deref(), Some("25"));
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_reference_to_an_empty_cell_is_empty_not_an_error"></a>
### `a_reference_to_an_empty_cell_is_empty_not_an_error`

<storycode:copy id="f-6d63eb1fec85-a_reference_to_an_empty_cell_is_empty_not_an_error">
#[tokio::test(flavor = "multi_thread")]
async fn a_reference_to_an_empty_cell_is_empty_not_an_error() {
    // A table with a gap in it is a table.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let computed = evaluate_sheet(
        dir.path(),
        "python",
        &sheet(&[
            ("A1", "5"),
            ("A3", "=sum(x for x in [A1, A2] if x is not None)"),
        ]),
    )
    .await
    .unwrap();
    assert_eq!(at(&computed, "A3").as_deref(), Some("5"));
}

</storycode:copy>
<a id="f-6d63eb1fec85-the_backend_installs_itself_on_first_use"></a>
### `the_backend_installs_itself_on_first_use`

<storycode:copy id="f-6d63eb1fec85-the_backend_installs_itself_on_first_use">
#[tokio::test(flavor = "multi_thread")]
async fn the_backend_installs_itself_on_first_use() {
    // "Auto-installed" literally: nothing was run, nothing was downloaded,
    // and the script is simply there.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    assert!(!dir.path().join(".hick-cache").exists());
    let _ = evaluate_sheet(dir.path(), "python", &sheet(&[("A1", "=1+1")]))
        .await
        .unwrap();
    assert!(
        dir.path()
            .join(".hick-cache/formula/python_backend.py")
            .is_file(),
        "the backend put itself in place"
    );
}

</storycode:copy>
<a id="f-6d63eb1fec85-use_declaration-2"></a>
### `use_declaration`

Stepping: the same evaluation, kept.
Protects docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md

<storycode:copy id="f-6d63eb1fec85-use_declaration-2">
// ---------------------------------------------------------------------------
// Stepping: the same evaluation, kept.
// Protects docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md
// ---------------------------------------------------------------------------

use hick_formula::{Value as CellValue, trace_sheet};

</storycode:copy>
<a id="f-6d63eb1fec85-a_trace_is_one_step_per_formula_in_the_order_they_ran"></a>
### `a_trace_is_one_step_per_formula_in_the_order_they_ran`

<storycode:copy id="f-6d63eb1fec85-a_trace_is_one_step_per_formula_in_the_order_they_ran">
#[tokio::test(flavor = "multi_thread")]
async fn a_trace_is_one_step_per_formula_in_the_order_they_ran() {
    // A literal has no turn: nothing was evaluated, so there is nothing to
    // step through, and a step showing `10 → 10` would be furniture.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let traced = trace_sheet(
        dir.path(),
        "python",
        &sheet(&[("A1", "10"), ("A2", "=A1*2"), ("A3", "=A2+5")]),
    )
    .await
    .unwrap();
    let order: Vec<String> = traced.steps.iter().map(|s| s.cell.label()).collect();
    assert_eq!(order, vec!["A2", "A3"]);
    assert_eq!(traced.steps[0].expression, "A1*2");
    assert_eq!(traced.steps[0].value.as_deref(), Some("20"));
    assert_eq!(traced.steps[1].value.as_deref(), Some("25"));
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_step_says_what_the_cell_read_at_the_moment_it_ran"></a>
### `a_step_says_what_the_cell_read_at_the_moment_it_ran`

<storycode:copy id="f-6d63eb1fec85-a_step_says_what_the_cell_read_at_the_moment_it_ran">
#[tokio::test(flavor = "multi_thread")]
async fn a_step_says_what_the_cell_read_at_the_moment_it_ran() {
    // The one thing a value in the grid cannot tell you. A2 read A1 as the
    // NUMBER 10 — not as the text "10", and not as A1's formula.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let traced = trace_sheet(
        dir.path(),
        "python",
        &sheet(&[("A1", "10"), ("B1", ""), ("A2", "=A1*2")]),
    )
    .await
    .unwrap();
    let step = &traced.steps[0];
    assert_eq!(
        step.bindings,
        vec![(
            CellRef::parse("A1").unwrap(),
            CellValue::Number { value: 10.0 }
        )]
    );
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_step_reading_a_blank_cell_shows_empty_rather_than_a_blank_string"></a>
### `a_step_reading_a_blank_cell_shows_empty_rather_than_a_blank_string`

<storycode:copy id="f-6d63eb1fec85-a_step_reading_a_blank_cell_shows_empty_rather_than_a_blank_string">
#[tokio::test(flavor = "multi_thread")]
async fn a_step_reading_a_blank_cell_shows_empty_rather_than_a_blank_string() {
    // Summing a column skips blanks rather than treating them as zero-length
    // text, and the step is where that difference is visible.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let traced = trace_sheet(dir.path(), "python", &sheet(&[("A1", "=Z9")]))
        .await
        .unwrap();
    assert_eq!(
        traced.steps[0].bindings,
        vec![(CellRef::parse("Z9").unwrap(), CellValue::Empty)]
    );
}

</storycode:copy>
<a id="f-6d63eb1fec85-independent_cells_share_a_level_and_a_chain_does_not"></a>
### `independent_cells_share_a_level_and_a_chain_does_not`

<storycode:copy id="f-6d63eb1fec85-independent_cells_share_a_level_and_a_chain_does_not">
#[tokio::test(flavor = "multi_thread")]
async fn independent_cells_share_a_level_and_a_chain_does_not() {
    // The level is the batch, and the batch is the round trip. A debugger
    // that showed four hundred independent cells as four hundred rounds
    // would be describing a program that does not exist.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let traced = trace_sheet(
        dir.path(),
        "python",
        &sheet(&[
            ("A1", "1"),
            ("B1", "=A1+1"),
            ("C1", "=A1+2"),
            ("D1", "=B1+C1"),
        ]),
    )
    .await
    .unwrap();
    let levels: Vec<(String, usize)> = traced
        .steps
        .iter()
        .map(|s| (s.cell.label(), s.level))
        .collect();
    assert_eq!(
        levels,
        vec![
            ("B1".to_string(), 0),
            ("C1".to_string(), 0),
            ("D1".to_string(), 1)
        ]
    );
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_broken_cell_keeps_its_own_step_and_its_dependant_reads_empty"></a>
### `a_broken_cell_keeps_its_own_step_and_its_dependant_reads_empty`

<storycode:copy id="f-6d63eb1fec85-a_broken_cell_keeps_its_own_step_and_its_dependant_reads_empty">
#[tokio::test(flavor = "multi_thread")]
async fn a_broken_cell_keeps_its_own_step_and_its_dependant_reads_empty() {
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let traced = trace_sheet(
        dir.path(),
        "python",
        &sheet(&[("A1", "=nope + 1"), ("A2", "=A1")]),
    )
    .await
    .unwrap();
    assert!(traced.steps[0].error.is_some(), "{:?}", traced.steps[0]);
    assert!(traced.steps[0].value.is_none());
    // Not the upstream message: a cell that depends on a broken one reports
    // its own trouble.
    assert_eq!(
        traced.steps[1].bindings,
        vec![(CellRef::parse("A1").unwrap(), CellValue::Empty)]
    );
}

</storycode:copy>
<a id="f-6d63eb1fec85-a_circle_has_no_steps_at_all"></a>
### `a_circle_has_no_steps_at_all`

<storycode:copy id="f-6d63eb1fec85-a_circle_has_no_steps_at_all">
#[tokio::test(flavor = "multi_thread")]
async fn a_circle_has_no_steps_at_all() {
    // A circle has no order, and inventing one to step through would be the
    // debugger telling its first lie. No interpreter is needed to know it.
    let dir = tempfile::tempdir().unwrap();
    let traced = trace_sheet(
        dir.path(),
        "python",
        &sheet(&[("A1", "=B1"), ("B1", "=A1")]),
    )
    .await
    .unwrap();
    assert!(traced.steps.is_empty());
    assert_eq!(traced.computed.errors.len(), 2);
}

</storycode:copy>
<a id="f-6d63eb1fec85-stepping_and_computing_are_the_same_evaluation"></a>
### `stepping_and_computing_are_the_same_evaluation`

<storycode:copy id="f-6d63eb1fec85-stepping_and_computing_are_the_same_evaluation">
#[tokio::test(flavor = "multi_thread")]
async fn stepping_and_computing_are_the_same_evaluation() {
    // The reason `evaluate_sheet` delegates: a debugger that disagrees with
    // the program is worse than none.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let s = sheet(&[
        ("A1", "3"),
        ("A2", "=A1*7"),
        ("A3", "=A2+1"),
        ("B1", "=nope"),
    ]);
    let computed = evaluate_sheet(dir.path(), "python", &s).await.unwrap();
    let traced = trace_sheet(dir.path(), "python", &s).await.unwrap();
    assert_eq!(traced.computed, computed);
    for step in &traced.steps {
        match (&step.value, &step.error) {
            (Some(value), None) => assert_eq!(computed.values.get(&step.cell), Some(value)),
            (None, Some(message)) => assert_eq!(computed.errors.get(&step.cell), Some(message)),
            other => panic!("a step is a value or an error: {other:?}"),
        }
    }
}

</storycode:copy>
[Read the test `monetary_numbers_are_numeric_in_python_and_javascript` beside its requirement](../../../../guarantees/authoring/cell-formatting-keeps-numbers-numeric/contract.md#f-6d63eb1fec85-monetary_numbers_are_numeric_in_python_and_javascript).

### Assemble `crates/hick-formula/tests/backends.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-formula/tests/backends.rs"><storycode:paste select="#f-6d63eb1fec85-have"/><storycode:paste select="#f-6d63eb1fec85-formula"/><storycode:paste select="#f-6d63eb1fec85-number"/><storycode:paste select="#f-6d63eb1fec85-evaluate"/><storycode:paste select="#f-6d63eb1fec85-python_evaluates_an_expression_with_its_references_bound"/><storycode:paste select="#f-6d63eb1fec85-python_reports_its_own_error_message_not_a_spreadsheet_code"/><storycode:paste select="#f-6d63eb1fec85-python_can_aggregate_a_column"/><storycode:paste select="#f-6d63eb1fec85-ranges_are_values_in_python_and_javascript"/><storycode:paste select="#f-6d63eb1fec85-python_reads_an_empty_cell_as_none_so_blanks_can_be_skipped"/><storycode:paste select="#f-6d63eb1fec85-javascript_evaluates_an_expression_with_its_references_bound"/><storycode:paste select="#f-6d63eb1fec85-javascript_reports_its_own_error_message"/><storycode:paste select="#f-6d63eb1fec85-javascript_returns_text_as_text"/><storycode:paste select="#f-6d63eb1fec85-a_batch_comes_back_whole_and_matched_by_id"/><storycode:paste select="#f-6d63eb1fec85-an_unknown_language_says_what_this_build_carries"/><storycode:paste select="#f-6d63eb1fec85-an_empty_batch_asks_the_backend_nothing"/><storycode:paste select="#f-6d63eb1fec85-use_declaration"/><storycode:paste select="#f-6d63eb1fec85-sheet"/><storycode:paste select="#f-6d63eb1fec85-at"/><storycode:paste select="#f-6d63eb1fec85-a_chain_evaluates_in_order_with_each_link_seeing_the_last"/><storycode:paste select="#f-6d63eb1fec85-a_total_row_reads_the_column_above_it"/><storycode:paste select="#f-6d63eb1fec85-a_whole_number_comes_back_without_a_decimal_point"/><storycode:paste select="#f-6d63eb1fec85-a_broken_cell_fails_alone_and_leaves_its_dependants_their_own_trouble"/><storycode:paste select="#f-6d63eb1fec85-javascript_evaluates_a_sheet_the_same_way_python_does"/><storycode:paste select="#f-6d63eb1fec85-a_reference_to_an_empty_cell_is_empty_not_an_error"/><storycode:paste select="#f-6d63eb1fec85-the_backend_installs_itself_on_first_use"/><storycode:paste select="#f-6d63eb1fec85-use_declaration-2"/><storycode:paste select="#f-6d63eb1fec85-a_trace_is_one_step_per_formula_in_the_order_they_ran"/><storycode:paste select="#f-6d63eb1fec85-a_step_says_what_the_cell_read_at_the_moment_it_ran"/><storycode:paste select="#f-6d63eb1fec85-a_step_reading_a_blank_cell_shows_empty_rather_than_a_blank_string"/><storycode:paste select="#f-6d63eb1fec85-independent_cells_share_a_level_and_a_chain_does_not"/><storycode:paste select="#f-6d63eb1fec85-a_broken_cell_keeps_its_own_step_and_its_dependant_reads_empty"/><storycode:paste select="#f-6d63eb1fec85-a_circle_has_no_steps_at_all"/><storycode:paste select="#f-6d63eb1fec85-stepping_and_computing_are_the_same_evaluation"/><storycode:paste select="#f-6d63eb1fec85-monetary_numbers_are_numeric_in_python_and_javascript"/></storycode:file>

</storycode:doc>
