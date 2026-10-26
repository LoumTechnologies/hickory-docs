<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Report Is Not a Reproducible Artifact

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a document whose root declares `volatile="true"`, when `hick test`
runs, then its woven output is excluded from drift comparison while every
other output of that document is still compared byte-for-byte, and every
`hick:expect` expectation is still enforced.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/a-report-is-not-a-reproducible-artifact.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-c6bfdff8fc14"></a>
## `crates/hickory-cli/tests/volatile_outputs.rs`

Related contracts: [A Drift Report Names What Differs](../a-drift-report-names-what-differs/contract.md), [A Report Is Not a Reproducible Artifact](contract.md).

<a id="f-c6bfdff8fc14-common"></a>
### `common`

A volatile output is a report; a normal output must still reproduce.
Found by dogfooding: `docs/analysis/claude-code-corpus.md` measures a
corpus that grows every time the CLI runs, so its woven report differed
from the committed copy on literally every check. A check that always
fails is one people stop reading, which would have quietly cost us the
drift guarantee on every OTHER file in the document.

<storycode:copy id="f-c6bfdff8fc14-common">
//! A volatile output is a report; a normal output must still reproduce.
//!
//! Guarantee: docs/guarantees/verification/a-report-is-not-a-reproducible-artifact.md
//!
//! Found by dogfooding: `docs/analysis/claude-code-corpus.md` measures a
//! corpus that grows every time the CLI runs, so its woven report differed
//! from the committed copy on literally every check. A check that always
//! fails is one people stop reading, which would have quietly cost us the
//! drift guarantee on every OTHER file in the document.

use std::path::Path;

use hickory_cli::{CheckFailure, ExecutorChoice, RunMode, check_failures, run_doc, write_outputs};

mod common;
</storycode:copy>
<a id="f-c6bfdff8fc14-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-c6bfdff8fc14-use_declaration">
use common::changes_every_run;

</storycode:copy>
<a id="f-c6bfdff8fc14-doc"></a>
### `doc`

A document whose exec output changes on every run, weaving a volatile
report and tangling a reproducible script.
The cell has to genuinely differ run to run or this document is not
volatile at all and the exemption below is never exercised — see
`changes_every_run`, which is why the body is per-shell.

<storycode:copy id="f-c6bfdff8fc14-doc">
/// A document whose exec output changes on every run, weaving a volatile
/// report and tangling a reproducible script.
///
/// The cell has to genuinely differ run to run or this document is not
/// volatile at all and the exemption below is never exercised — see
/// `changes_every_run`, which is why the body is per-shell.
fn doc() -> String {
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md" volatile="true">
# Report over changing data

<hick:container name="c" image="host" />

<hick:copy id="body" class="script">
print("stable")
</hick:copy>

<hick:file path="stable.py">
<hick:paste select=".script" />
</hick:file>

<hick:exec container="c">
{}
</hick:exec>
</hick:doc>
"##,
        changes_every_run()
    )
}

</storycode:copy>
<a id="f-c6bfdff8fc14-run_and_check"></a>
### `run_and_check`

<storycode:copy id="f-c6bfdff8fc14-run_and_check">
async fn run_and_check(dir: &Path) -> Vec<CheckFailure> {
    let doc_path = dir.join("doc.md");
    let run = run_doc(&doc_path, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .expect("run");
    check_failures(&run, None).expect("check")
}

</storycode:copy>
<a id="f-c6bfdff8fc14-a_volatile_report_does_not_drift_but_its_files_still_do"></a>
### `a_volatile_report_does_not_drift_but_its_files_still_do`

<storycode:copy id="f-c6bfdff8fc14-a_volatile_report_does_not_drift_but_its_files_still_do">
#[tokio::test(flavor = "multi_thread")]
async fn a_volatile_report_does_not_drift_but_its_files_still_do() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("doc.md");
    std::fs::write(&doc_path, doc()).unwrap();

    // Commit the outputs, exactly as a first `hick run` would.
    let run = run_doc(&doc_path, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .expect("first run");
    write_outputs(&run, None).expect("write outputs");

    // A second run produces a DIFFERENT report (the timestamp moved) but an
    // identical script. Only the script is a reproducibility claim.
    let failures = run_and_check(dir.path()).await;
    assert!(
        failures.is_empty(),
        "a volatile report must not be reported as drift: {failures:?}"
    );

    // The non-volatile file is still protected: tamper with it and check
    // must fail. Without this half, `volatile` could be silently disabling
    // the drift guarantee for the whole document.
    let script = dir.path().join("stable.py");
    let original = std::fs::read_to_string(&script).unwrap();
    std::fs::write(&script, format!("{original}# tampered\n")).unwrap();

    let failures = run_and_check(dir.path()).await;
    let drifted: Vec<_> = failures
        .iter()
        .filter_map(|f| match f {
            CheckFailure::Drift { output_path, .. } => Some(output_path.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(drifted.len(), 1, "expected exactly one drift: {failures:?}");
    assert!(
        drifted[0].ends_with("stable.py"),
        "the tampered script must be the one flagged: {drifted:?}"
    );
}

</storycode:copy>
<a id="f-c6bfdff8fc14-the_root_volatile_attribute_is_actually_parsed"></a>
### `the_root_volatile_attribute_is_actually_parsed`

<storycode:copy id="f-c6bfdff8fc14-the_root_volatile_attribute_is_actually_parsed">
#[test]
fn the_root_volatile_attribute_is_actually_parsed() {
    // The root `hick:doc` is the document, not one of its child nodes, so
    // `find_tags("doc")` matches nothing — the first implementation read the
    // flag that way and silently never found it. This pins the real path.
    let doc = hick_lang::parse(&doc()).expect("parse");
    assert!(doc.volatile, "root volatile=\"true\" was not parsed");
    assert_eq!(doc.weave_path.as_deref(), Some("report.md"));
    assert!(
        doc.find_tags("doc").is_empty(),
        "if the root ever becomes a child node, revisit volatile_outputs"
    );
}

</storycode:copy>
<a id="f-c6bfdff8fc14-xml_entities_in_an_exec_body_are_warned_about"></a>
### `xml_entities_in_an_exec_body_are_warned_about`

The escaping trap: hick does not unescape, so `&lt;` in an exec body is
five characters handed to the shell. The failure that follows names the
shell, not the cause.

<storycode:copy id="f-c6bfdff8fc14-xml_entities_in_an_exec_body_are_warned_about">
/// The escaping trap: hick does not unescape, so `&lt;` in an exec body is
/// five characters handed to the shell. The failure that follows names the
/// shell, not the cause.
#[test]
fn xml_entities_in_an_exec_body_are_warned_about() {
    let doc = hick_lang::parse(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
         <hick:container name=\"c\" image=\"host\" />\n\
         <hick:exec container=\"c\">\n\
         python3 - &lt;&lt;'PY'\n\
         </hick:exec>\n\
         </hick:doc>\n",
    )
    .expect("parse");
    let warnings = hickory_cli::escaping_warnings(&doc);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("&lt;"), "{warnings:?}");
    assert!(
        warnings[0].contains("does not unescape"),
        "the warning must name the cause: {warnings:?}"
    );
}

</storycode:copy>
<a id="f-c6bfdff8fc14-a_clean_exec_body_produces_no_warning"></a>
### `a_clean_exec_body_produces_no_warning`

<storycode:copy id="f-c6bfdff8fc14-a_clean_exec_body_produces_no_warning">
#[test]
fn a_clean_exec_body_produces_no_warning() {
    // No false positives on the correct spelling — this must stay silent or
    // people will tune the warning out.
    let doc = hick_lang::parse(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
         <hick:container name=\"c\" image=\"host\" />\n\
         <hick:exec container=\"c\">\n\
         python3 - <<'PY'\n\
         </hick:exec>\n\
         </hick:doc>\n",
    )
    .expect("parse");
    assert!(hickory_cli::escaping_warnings(&doc).is_empty());
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/volatile_outputs.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/volatile_outputs.rs"><storycode:paste select="#f-c6bfdff8fc14-common"/><storycode:paste select="#f-c6bfdff8fc14-use_declaration"/><storycode:paste select="#f-c6bfdff8fc14-doc"/><storycode:paste select="#f-c6bfdff8fc14-run_and_check"/><storycode:paste select="#f-c6bfdff8fc14-a_volatile_report_does_not_drift_but_its_files_still_do"/><storycode:paste select="#f-c6bfdff8fc14-the_root_volatile_attribute_is_actually_parsed"/><storycode:paste select="#f-c6bfdff8fc14-xml_entities_in_an_exec_body_are_warned_about"/><storycode:paste select="#f-c6bfdff8fc14-a_clean_exec_body_produces_no_warning"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)
- [experiments/claude-code-corpus/corpus_scan.py](../../../mechanisms/experiments/claude-code-corpus/corpus_scan.py.md#output-5af8170a3224)

### Tests and fixtures

- [crates/hickory-cli/tests/volatile_outputs.rs](contract.md#output-c6bfdff8fc14)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
