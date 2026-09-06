//! `tier="push"` and `hick test --tier commit`, through the shipped binary.
//!
//! These protect
//! `docs/guarantees/verification/a-push-tier-cell-is-skipped-at-commit.md`.
//!
//! The document below is deliberately hostile: its `tier="push"` cell prints
//! `one` and its `<hick:expect>` claims `three`. Any gate that RAN the cell
//! would exit 3. So "exit 0" here is a real statement — the cell was not
//! executed and its expectation was not evaluated — rather than a pass that
//! happens to agree.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::echo_lines;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

/// A document with one ordinary cell (whose expectation holds) and one
/// `tier="push"` cell (whose expectation does not).
fn doc_source() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="tiered.md">
# Tiered

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c">
{cheap}
<hick:expect match="regex-lines">cheap
</hick:expect>
</hick:exec>

<hick:exec container="c" tier="push">
{slow}
<hick:expect match="regex-lines">three
</hick:expect>
</hick:exec>

<hick:file path="tangled.txt">
a product no cell feeds
</hick:file>
</hick:doc>
"#,
        cheap = echo_lines(&["cheap"]),
        slow = echo_lines(&["one"]),
    )
}

fn write_doc(dir: &Path) -> PathBuf {
    let path = dir.join("tiered.hick");
    std::fs::write(&path, doc_source()).unwrap();
    path
}

/// Throw away every recording the project keeps.
///
/// A flagless `hick run` records what it ran (see
/// `recordings-are-written-only-when-asked-for.md`), so a document that has
/// just been run has a baseline for every cell — which is exactly the state
/// in which nothing is ever skipped. Removing them puts the project in the
/// other state that really happens: a clone whose `.hick-cache` was never
/// committed, where the commit tier has to decide what to do with a cell it
/// may not execute.
fn forget_recordings(project_dir: &Path) {
    let _ = std::fs::remove_dir_all(project_dir.join(".hick-cache"));
}

/// Bring the document's outputs on disk up to date, ignoring the failing
/// expectation (`hick run` records outcomes but does not fail on them).
fn run_once(doc: &Path) {
    let out = hick().arg("run").arg(doc).output().unwrap();
    assert!(
        out.status.success(),
        "hick run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn commit_tier_skips_a_push_cell_and_still_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    run_once(&doc);
    forget_recordings(dir.path());

    let out = hick()
        .args(["test", "--tier", "commit"])
        .arg(&doc)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(
        out.status.code(),
        Some(0),
        "a cell the document tiered out of this gate is a decision, not an \
         unverifiable baseline and not a failed claim: {stderr}"
    );
    assert!(
        stderr.contains("SKIP") && stderr.contains("tier=\"push\""),
        "the skip must be reported, naming the document and the cell: {stderr}"
    );
    assert!(
        stderr.contains("hick test ."),
        "and must name the gate that does run it: {stderr}"
    );
    assert!(
        !stderr.contains("FAIL"),
        "the skipped cell's expectation must not be evaluated: {stderr}"
    );
}

#[test]
fn the_same_document_fails_at_the_push_tier() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    run_once(&doc);
    forget_recordings(dir.path());

    for args in [vec!["test"], vec!["test", "--tier", "push"]] {
        let out = hick().args(&args).arg(&doc).output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        assert_eq!(
            out.status.code(),
            Some(3),
            "{args:?} must run every cell, and this document's push cell \
             claims something untrue: {stderr}"
        );
        assert!(!stderr.contains("SKIP"), "{args:?} skips nothing: {stderr}");
    }
}

#[test]
fn a_recording_answers_a_push_cell_at_the_commit_tier() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    // `--cache` records every cell, so the push cell now HAS a baseline.
    let out = hick().args(["run", "--cache"]).arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "hick run --cache failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = hick()
        .args(["test", "--tier", "commit"])
        .arg(&doc)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        !stderr.contains("SKIP"),
        "a recording that still matches the cell's key answers it, exactly \
         as it does for a frozen cell — nothing is skipped: {stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(3),
        "and everything downstream is then verified normally, which for this \
         document means the false claim is caught at the commit tier too: {stderr}"
    );
}

#[test]
fn an_output_no_skipped_cell_feeds_is_still_checked_for_drift() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    run_once(&doc);
    forget_recordings(dir.path());
    // A tangled product the push cell contributes nothing to. Drift here is
    // exactly what a commit is meant to catch, tiers or no tiers.
    std::fs::write(dir.path().join("tangled.txt"), "edited by hand\n").unwrap();

    let out = hick()
        .args(["test", "--tier", "commit"])
        .arg(&doc)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert_eq!(
        out.status.code(),
        Some(1),
        "skipping a cell must not stop the rest of the document's outputs \
         from being compared: {stderr}"
    );
    assert!(stderr.contains("tangled.txt"), "{stderr}");
}

#[test]
fn an_unknown_tier_is_refused_naming_both() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    let out = hick()
        .args(["test", "--tier", "prepush"])
        .arg(&doc)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(!out.status.success());
    assert!(
        stderr.contains("--tier commit") && stderr.contains("--tier push"),
        "{stderr}"
    );
}
