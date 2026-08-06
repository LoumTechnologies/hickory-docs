//! Integration tests for `hickory run` / `hickory check`.
//!
//! `check_fails_on_drifted_expectation` is the test backing the guarantee in
//! `docs/guarantees/verification/check-fails-on-drift.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn hickory() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hickory"))
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn write_doc(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    path
}

const PASSING_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="passing.md">
# Passing

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c">
printf 'one\ntwo\n'
<hick:expect match="exact">one
two
</hick:expect>
</hick:exec>
</hick:doc>
"#;

const DRIFTED_DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="drifted.md">
# Drifted

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c">
printf 'one\ntwo\n'
<hick:expect match="exact">one
three
</hick:expect>
</hick:exec>
</hick:doc>
"#;

#[test]
fn run_succeeds_and_records_failed_expectation_without_failing() {
    // On `run`, expectations are evaluated and recorded but do NOT fail.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "drifted.hick", DRIFTED_DOC);
    let out = hickory()
        .args(["run"])
        .arg(&doc)
        .output()
        .expect("run hickory");
    assert!(
        out.status.success(),
        "run must not fail on unmet expectations: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("1 failed"), "stderr: {stderr}");
}

#[test]
fn check_passes_on_matching_expectations() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    // First run writes the woven output so check has committed files.
    assert!(
        hickory()
            .args(["run"])
            .arg(&doc)
            .status()
            .unwrap()
            .success()
    );
    let out = hickory().args(["check"]).arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "check must pass: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn check_fails_on_drifted_expectation() {
    // The guarantee: documentation drift is a build failure, never a warning.
    // The failing block must be reported with doc path, line, and
    // expected-vs-actual.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "drifted.hick", DRIFTED_DOC);
    let out = hickory().args(["check"]).arg(&doc).output().unwrap();
    assert!(
        !out.status.success(),
        "check must exit non-zero on an unmet expectation"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("drifted.hick"), "doc path listed: {stderr}");
    assert!(stderr.contains(":7"), "block source line listed: {stderr}");
    assert!(stderr.contains("three"), "expected shown: {stderr}");
    assert!(stderr.contains("two"), "actual shown: {stderr}");
}

#[test]
fn check_fails_on_committed_output_drift() {
    // Drift between the freshly woven output and the committed file also
    // fails check, even when expectations pass.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    assert!(
        hickory()
            .args(["run"])
            .arg(&doc)
            .status()
            .unwrap()
            .success()
    );
    // Tamper with the committed woven markdown.
    let woven = dir.path().join("passing.md");
    std::fs::write(&woven, "stale hand-edited content\n").unwrap();
    let out = hickory().args(["check"]).arg(&doc).output().unwrap();
    assert!(!out.status.success(), "check must detect output drift");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("passing.md"), "stderr: {stderr}");
}

#[test]
fn check_fails_on_drifted_fixture_copy_of_shipped_example() {
    // The shipped example, deliberately drifted (apple 12 -> apple 13).
    let out = hickory()
        .args(["check"])
        .arg(fixture("drifted-tour.hick"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("apple 13"), "stderr: {stderr}");
    assert!(stderr.contains("apple 12"), "stderr: {stderr}");
}

#[test]
fn regex_lines_expectations_pass_and_fail() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        "regex.hick",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="c" image="alpine:3.20" />
<hick:exec container="c">
echo "value: 42"
<hick:expect match="regex-lines">value: \d+
</hick:expect>
</hick:exec>
</hick:doc>
"#,
    );
    let out = hickory().args(["check"]).arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "regex-lines must match: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Expectation must cover ALL output lines.
    let doc2 = write_doc(
        dir.path(),
        "regex-short.hick",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="c" image="alpine:3.20" />
<hick:exec container="c">
printf 'a\nb\n'
<hick:expect match="regex-lines">a
</hick:expect>
</hick:exec>
</hick:doc>
"#,
    );
    let out2 = hickory().args(["check"]).arg(&doc2).output().unwrap();
    assert!(!out2.status.success(), "must fail when lines uncovered");
}

#[test]
fn json_block_model_has_spans_transcripts_statuses() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    let out = hickory()
        .args(["run", "--json"])
        .arg(&doc)
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let blocks = v["blocks"].as_array().unwrap();
    let exec = blocks
        .iter()
        .find(|b| b["kind"] == "exec")
        .expect("an exec block");
    assert_eq!(exec["status"], "ok");
    assert_eq!(exec["expect"]["match"], "exact");
    assert!(exec["span"].as_array().unwrap().len() == 2);
    let events = exec["transcript"].as_array().unwrap();
    assert_eq!(events.first().unwrap()["kind"], "cmd");
    assert_eq!(events.last().unwrap()["kind"], "exit");
}

#[test]
fn canopy_executor_without_config_fails_actionably() {
    // The canopy executor is wired, but without its env config the run must
    // fail fast with an actionable message (which env var to set, and the
    // way back to the local executor). The agent endpoint env name itself is
    // asserted only loosely: canopy API knowledge stays in the adapter crate
    // (docs/guarantees/execution/canopy-api-isolated-to-one-crate.md).
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    let out = hickory()
        .args(["run"])
        .arg(&doc)
        .env("HICKORY_EXECUTOR", "canopy")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("is not set"), "{stderr}");
    assert!(stderr.contains("HICKORY_EXECUTOR=local"), "{stderr}");
}
