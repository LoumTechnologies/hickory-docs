//! The backends, actually spawned.
//!
//! Protects docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md
//!
//! Everything else in this crate is testable without a process. This file is
//! the part that is not: that the script we embed really is a program the
//! machine's own interpreter can run, that it speaks the framing, and that it
//! answers in this protocol's shapes. A backend that parses in principle and
//! deadlocks in practice is the failure worth catching here.
//!
//! Skipped, loudly, on a machine without the interpreter. A CI runner with no
//! node must not turn a missing dependency into a red build for a feature
//! that is designed to be absent gracefully.

use hick_formula::{Formula, Session, Value};

fn have(interpreters: &[&str]) -> bool {
    interpreters.iter().any(|name| {
        std::process::Command::new(name)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

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

fn number(n: f64) -> Value {
    Value::Number { value: n }
}

async fn evaluate(language: &str, formulas: Vec<Formula>) -> Vec<hick_formula::FormulaResult> {
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start(dir.path(), language)
        .await
        .unwrap_or_else(|e| panic!("the {language} backend starts: {e:#}"));
    let answer = session.evaluate(formulas).await.expect("it answers");
    session.shutdown().await;
    answer.results
}

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

#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_language_says_what_this_build_carries() {
    let dir = tempfile::tempdir().unwrap();
    let error = match Session::start(dir.path(), "cobol").await {
        Ok(_) => panic!("cobol must not start"),
        Err(e) => format!("{e:#}"),
    };
    assert!(error.contains("cobol"), "{error}");
}

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

// ---------------------------------------------------------------------------
// A whole sheet, through the levels, against a real interpreter.
// ---------------------------------------------------------------------------

use hick_formula::{CellRef, Sheet, evaluate_sheet};

fn sheet(entries: &[(&str, &str)]) -> Sheet {
    entries
        .iter()
        .map(|(at, text)| (CellRef::parse(at).unwrap(), text.to_string()))
        .collect()
}

fn at(computed: &hick_formula::Computed, cell: &str) -> Option<String> {
    computed.values.get(&CellRef::parse(cell).unwrap()).cloned()
}

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

// ---------------------------------------------------------------------------
// Stepping: the same evaluation, kept.
// Protects docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md
// ---------------------------------------------------------------------------

use hick_formula::{Value as CellValue, trace_sheet};

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
