//! A weave with no recording must not overwrite an artifact with `[never run]`.
//!
//! Guarantee: docs/guarantees/verification/a-weave-without-a-recording-keeps-the-artifact.md
//!
//! Found by dogfooding (issue #25): a `hick weave` on a checkout with no
//! `.hick-cache` replaced a committed SVG with the four words `[never run]`
//! and exited 0, while the app — which executes rather than replaying — went
//! on rendering the chart. Disk and app disagreed and nothing said so.

use std::path::Path;

use hickory_cli::{ExecutorChoice, RunMode, run_doc, write_outputs_detailed};

/// A document that produces an artifact from a cell, plus a woven report.
///
/// Two outputs on purpose. The picture is the artifact this test is about —
/// the thing a weave must not overwrite with a marker. The text file is how
/// the report is checked, because a picture weaves as `![…](…)` rather than
/// as its bytes, so the marker can only show up in a file that HAS bytes a
/// reader reads. See
/// docs/guarantees/authoring/a-generated-picture-shows-as-a-picture.md.
fn doc() -> &'static str {
    r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
# Report

<hick:container name="c" image="host" />

<hick:file path="chart.svg">
<hick:exec container="c" show="output">
echo "<svg>real bytes</svg>"
</hick:exec>
</hick:file>

<hick:file path="log.txt">
<hick:exec container="c" show="output">
echo real-log
</hick:exec>
</hick:file>
</hick:doc>
"##
}

async fn weave(doc_path: &Path) -> hickory_cli::WrittenOutputs {
    let run = run_doc(doc_path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .expect("weave");
    write_outputs_detailed(&run, None).expect("write outputs")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_weave_without_a_recording_keeps_the_artifact_and_still_writes_the_report() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("doc.hick");
    std::fs::write(&doc_path, doc()).unwrap();

    // A real run commits the artifact, exactly as a first `hick run` would.
    let run = run_doc(&doc_path, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .expect("run");
    write_outputs_detailed(&run, None).expect("write outputs");
    let chart = dir.path().join("chart.svg");
    let real_bytes = std::fs::read_to_string(&chart).expect("chart written");
    assert!(
        real_bytes.contains("real bytes"),
        "the run must produce the artifact for this test to mean anything: {real_bytes:?}"
    );

    // Now weave with no recording anywhere — the state of a fresh clone,
    // since `.hick-cache/` is gitignored.
    //
    // The removal is the whole setup, and it has to be explicit: `hick run`
    // records whatever it executes, whatever the cache mode says
    // (`CacheMode::records()` is unconditionally true, and its doc comment
    // argues why). So the run above leaves a recording beside the document,
    // and without this the weave would find it, replay the real bytes, and
    // report nothing preserved — which is what a fresh clone precisely does
    // not do.
    std::fs::remove_dir_all(dir.path().join(".hick-cache")).expect("run wrote a recording");
    let outputs = weave(&doc_path).await;

    assert_eq!(
        std::fs::read_to_string(&chart).unwrap(),
        real_bytes,
        "a weave with no recording must leave the committed artifact alone"
    );
    assert!(
        outputs.preserved.iter().any(|p| p == &chart),
        "the preserved artifact must be reported, not silently skipped: {outputs:?}"
    );
    assert!(
        !outputs.written.iter().any(|p| p == &chart),
        "the artifact must not be counted as written: {outputs:?}"
    );

    // The weave target is this weave's own report, so it IS written, and it
    // says plainly that the cell has no recording.
    let report = dir.path().join("report.md");
    assert!(
        outputs.written.iter().any(|p| p == &report),
        "the woven report must still be written: {outputs:?}"
    );
    let woven = std::fs::read_to_string(&report).unwrap();
    assert!(
        woven.contains("[never run]"),
        "the report must say the cell never ran rather than pretending otherwise: {woven}"
    );
    // The picture is referenced rather than inlined, which is why the marker
    // above is checked through the text file instead.
    assert!(woven.contains("![chart.svg](chart.svg)"), "{woven}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_weave_still_creates_an_artifact_that_is_not_there_yet() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = dir.path().join("doc.hick");
    std::fs::write(&doc_path, doc()).unwrap();

    // Nothing has ever run and nothing is on disk: there is no artifact to
    // destroy, so the marker is the honest content of the file.
    let outputs = weave(&doc_path).await;
    let chart = dir.path().join("chart.svg");
    assert!(
        outputs.written.iter().any(|p| p == &chart),
        "a missing artifact must still be written: {outputs:?}"
    );
    assert!(
        std::fs::read_to_string(&chart)
            .unwrap()
            .contains("[never run]"),
        "and it must say the cell never ran"
    );
}
