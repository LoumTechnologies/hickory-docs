//! `<hick:capture>` against a real adapter, on a real document.
//!
//! Protects docs/specs/freeform/literate-debugging.md
//!
//! The claim being tested is the one that makes captures worth having: a
//! document can record a value from **inside a function**, on every hit,
//! without anybody stepping — and can do it without the debuggee's writes
//! reaching the project.
//!
//! Skipped loudly when no adapter is installed, for the reason `live_session`
//! is: a suite that quietly tests nothing is worse than one that fails.

use std::path::Path;

use hick_dap::capture::{CaptureSpec, run};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Pricing

<hick:file path="pricing.py">
LINES = [(2, 9.99), (1, 24.50), (3, 1.25)]


def line_total(quantity, unit_price):
    subtotal = quantity * unit_price
    return subtotal


with open("evidence.txt", "w") as handle:
    handle.write("the debuggee wrote this")

print(f"total {sum(line_total(q, p) for q, p in LINES):.2f}")
</hick:file>
</hick:doc>
"##;

/// 1-based line of `    subtotal = quantity * unit_price` in `pricing.py`.
///
/// Six, not five: the generated file opens with the newline that follows
/// `<hick:file …>`, so every line of the block sits one lower than it reads
/// in the document. Being one out here is not a rounding error — line 5 is
/// the `def`, which runs once, at module level, in a different frame.
const SUBTOTAL: u32 = 6;

struct Project {
    dir: tempfile::TempDir,
}

fn project() -> Option<Project> {
    let dir = tempfile::tempdir().ok()?;
    std::fs::create_dir_all(dir.path().join(".git")).ok()?;
    std::fs::write(dir.path().join("doc.hick"), DOC).ok()?;
    borrow_this_repos_adapters(dir.path());
    hick_dap::discover("python", dir.path())?;
    Some(Project { dir })
}

/// Point the scratch project at the adapter this repository installed.
///
/// A developer runs `hick dap install python` once, at the top of this repo.
/// A test project in a temp directory is nowhere near it, so without this the
/// whole file skips on the machine most likely to be running it. CI, which
/// installs debugpy for the machine, does not need it.
#[cfg(unix)]
fn borrow_this_repos_adapters(into: &Path) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cache = repo.join(".hick-cache");
    if cache.join("adapters/python/bin/python3").exists() {
        let _ = std::os::unix::fs::symlink(cache, into.join(".hick-cache"));
    }
}

#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

fn skip() {
    eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
}

fn spec(line: u32, of: &[&str]) -> CaptureSpec {
    CaptureSpec {
        file: "pricing.py".into(),
        line,
        expressions: of.iter().map(|s| (*s).to_string()).collect(),
        condition: None,
        max: 20,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_capture_records_a_value_from_inside_a_function_on_every_hit() {
    let Some(project) = project() else {
        skip();
        return;
    };
    let captured = run(
        Path::new("doc.hick"),
        DOC,
        project.dir.path(),
        &[spec(SUBTOTAL, &["quantity", "quantity * unit_price"])],
    )
    .await
    .expect("the capture run finishes");

    let captured = &captured[0];
    assert_eq!(captured.problem, None, "{captured:?}");
    // Three lines in the cart, so three hits — in run order, not wall-clock.
    assert_eq!(captured.hits.len(), 3, "{captured:?}");
    assert_eq!(captured.hits[0].values[0].value, "2");
    assert!(captured.hits[0].values[1].value.starts_with("19.98"));
    assert_eq!(captured.hits[1].values[0].value, "1");
    assert!(captured.hits.iter().all(|hit| hit.values[0].ok));
    assert!(!captured.truncated);

    // And it weaves as the table the document keeps.
    let table = hick_dap::capture::render(captured);
    assert!(table.contains("| hit | `quantity` |"), "{table}");
    assert!(table.lines().count() == 5, "{table}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_condition_records_only_the_hits_it_names() {
    let Some(project) = project() else {
        skip();
        return;
    };
    let mut spec = spec(SUBTOTAL, &["quantity"]);
    // A raw `>` inside an attribute is what a document would write.
    spec.condition = Some("quantity > 2".into());
    let captured = run(Path::new("doc.hick"), DOC, project.dir.path(), &[spec])
        .await
        .expect("the capture run finishes");

    assert_eq!(captured[0].hits.len(), 1, "{:?}", captured[0]);
    assert_eq!(captured[0].hits[0].values[0].value, "3");
}

#[tokio::test(flavor = "multi_thread")]
async fn hits_are_bounded_and_the_document_is_told() {
    let Some(project) = project() else {
        skip();
        return;
    };
    let mut spec = spec(SUBTOTAL, &["quantity"]);
    spec.max = 2;
    let captured = run(Path::new("doc.hick"), DOC, project.dir.path(), &[spec])
        .await
        .expect("the capture run finishes");

    assert_eq!(captured[0].hits.len(), 2);
    assert!(captured[0].truncated);
    assert!(
        hick_dap::capture::render(&captured[0]).contains("stopped after 2 hits"),
        "silent truncation reads as \"that is all there was\""
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_expression_that_is_not_in_scope_is_recorded_rather_than_fatal() {
    let Some(project) = project() else {
        skip();
        return;
    };
    let captured = run(
        Path::new("doc.hick"),
        DOC,
        project.dir.path(),
        &[spec(SUBTOTAL, &["quantity", "no_such_name"])],
    )
    .await
    .expect("one bad expression does not fail the run");

    assert!(captured[0].hits[0].values[0].ok);
    assert!(!captured[0].hits[0].values[1].ok);
    assert!(!captured[0].hits[0].values[1].value.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_capture_run_writes_nothing_into_the_project() {
    // The debuggee in this document writes a file. A capture run must leave
    // the project exactly as it was — that is what lets a document carry
    // captures without its runs quietly producing artefacts.
    let Some(project) = project() else {
        skip();
        return;
    };
    let before: Vec<String> = std::fs::read_dir(project.dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();

    run(
        Path::new("doc.hick"),
        DOC,
        project.dir.path(),
        &[spec(SUBTOTAL, &["quantity"])],
    )
    .await
    .expect("the capture run finishes");

    let mut after: Vec<String> = std::fs::read_dir(project.dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    after.sort();
    let mut before = before;
    before.sort();
    assert_eq!(before, after, "the capture run left something behind");
    assert!(!project.dir.path().join("evidence.txt").exists());
    // Not even the generated file: it was woven into the scratch clone.
    assert!(!project.dir.path().join("pricing.py").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_capture_follows_a_breakpoint_the_adapter_moves() {
    // Line 4 is blank. Python cannot stop there, so debugpy slides the
    // breakpoint down to the `def` below it. Recording nothing would report
    // "never hit" about a breakpoint that fired — so the capture follows it,
    // and the woven table says it moved.
    let Some(project) = project() else {
        skip();
        return;
    };
    let captured = run(
        Path::new("doc.hick"),
        DOC,
        project.dir.path(),
        &[spec(4, &["LINES"])],
    )
    .await
    .expect("the capture run finishes");

    assert!(captured[0].moved.is_some(), "{:?}", captured[0]);
    assert_eq!(captured[0].hits.len(), 1, "{:?}", captured[0]);
    let table = hick_dap::capture::render(&captured[0]);
    assert!(table.contains("moved this capture"), "{table}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_location_the_document_does_not_generate_says_so() {
    let Some(project) = project() else {
        skip();
        return;
    };
    let captured = run(
        Path::new("doc.hick"),
        DOC,
        project.dir.path(),
        &[spec(9_000, &["quantity"])],
    )
    .await
    .expect("a bad location is reported, not fatal");

    let problem = captured[0].problem.as_deref().unwrap_or_default();
    assert!(problem.contains("counted from 1"), "{problem}");
}
