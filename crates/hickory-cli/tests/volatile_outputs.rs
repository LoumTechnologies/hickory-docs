//! A volatile output is a report; a normal output must still reproduce.
//!
//! Guarantee: docs/guarantees/verification/a-report-is-not-a-reproducible-artifact.md
//!
//! Found by dogfooding: `docs/analysis/claude-code-corpus.hick` measures a
//! corpus that grows every time the CLI runs, so its woven report differed
//! from the committed copy on literally every check. A check that always
//! fails is one people stop reading, which would have quietly cost us the
//! drift guarantee on every OTHER file in the document.

use std::path::Path;

use hickory_cli::{CheckFailure, ExecutorChoice, RunMode, check_failures, run_doc, write_outputs};

/// A document whose exec output changes on every run (a counter file in the
/// document's own directory), weaving a volatile report and tangling a
/// reproducible script.
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
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
date +%s%N
</hick:exec>
</hick:doc>
"##;

async fn run_and_check(dir: &Path) -> Vec<CheckFailure> {
    let doc_path = dir.join("doc.hick");
    let run = run_doc(&doc_path, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .expect("run");
    check_failures(&run, None).expect("check")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_volatile_report_does_not_drift_but_its_files_still_do() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("doc.hick");
    std::fs::write(&doc_path, DOC).unwrap();

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

#[test]
fn the_root_volatile_attribute_is_actually_parsed() {
    // The root `hick:doc` is the document, not one of its child nodes, so
    // `find_tags("doc")` matches nothing — the first implementation read the
    // flag that way and silently never found it. This pins the real path.
    let doc = hick_lang::parse(DOC).expect("parse");
    assert!(doc.volatile, "root volatile=\"true\" was not parsed");
    assert_eq!(doc.weave_path.as_deref(), Some("report.md"));
    assert!(
        doc.find_tags("doc").is_empty(),
        "if the root ever becomes a child node, revisit volatile_outputs"
    );
}

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
