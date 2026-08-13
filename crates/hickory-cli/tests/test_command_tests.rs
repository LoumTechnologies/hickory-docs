//! Integration tests for `hick run` / `hick test`.
//!
//! `test_fails_on_drifted_expectation` is the test backing the guarantee in
//! `docs/guarantees/verification/test-fails-on-drift.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
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
    let out = hick().args(["run"]).arg(&doc).output().expect("run hick");
    assert!(
        out.status.success(),
        "run must not fail on unmet expectations: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("1 failed"), "stderr: {stderr}");
}

#[test]
fn test_passes_on_matching_expectations() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    // First run writes the woven output so check has committed files.
    assert!(hick().args(["run"]).arg(&doc).status().unwrap().success());
    let out = hick().args(["test"]).arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "check must pass: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_fails_on_drifted_expectation() {
    // The guarantee: documentation drift is a build failure, never a warning.
    // The failing block must be reported with doc path, line, and
    // expected-vs-actual.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "drifted.hick", DRIFTED_DOC);
    let out = hick().args(["test"]).arg(&doc).output().unwrap();
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
fn test_fails_on_committed_output_drift() {
    // Drift between the freshly woven output and the committed file also
    // fails check, even when expectations pass.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    assert!(hick().args(["run"]).arg(&doc).status().unwrap().success());
    // Tamper with the committed woven markdown.
    let woven = dir.path().join("passing.md");
    std::fs::write(&woven, "stale hand-edited content\n").unwrap();
    let out = hick().args(["test"]).arg(&doc).output().unwrap();
    assert!(!out.status.success(), "test must detect output drift");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("passing.md"), "stderr: {stderr}");
}

#[test]
fn test_fails_on_drifted_fixture_copy_of_shipped_example() {
    // The shipped example, deliberately drifted (apple 12 -> apple 13).
    let out = hick()
        .args(["test"])
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
    let out = hick().args(["test"]).arg(&doc).output().unwrap();
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
    let out2 = hick().args(["test"]).arg(&doc2).output().unwrap();
    assert!(!out2.status.success(), "must fail when lines uncovered");
}

#[test]
fn json_block_model_has_spans_transcripts_statuses() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    let out = hick().args(["run", "--json"]).arg(&doc).output().unwrap();
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
    let out = hick()
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

// ---------------------------------------------------------------------------
// Four outcomes: verified / drifted / unverifiable / expectation-failed
//
// These protect
// docs/guarantees/verification/test-separates-unverifiable-from-drifted.md.
// ---------------------------------------------------------------------------

/// The frozen cell's command, shared between the document and the recording
/// so the cache key the pipeline computes is the one the test wrote.
const FROZEN_COMMAND: &str = "\nprintf 'one\\ntwo\\n'\n";

fn frozen_doc() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="frozen.md">
# Frozen

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c" freeze="true">{FROZEN_COMMAND}</hick:exec>
</hick:doc>
"#
    )
}

/// Pre-record what a run would have stored for the frozen cell, without
/// running it — the same shape `crates/hick-literate/tests/freeze_tests.rs`
/// uses.
fn record_frozen_cell(project_dir: &Path, output: &str) {
    let cc =
        hick_literate::cache::CacheConfig::new(project_dir, hick_literate::cache::CacheMode::Off);
    let key = hick_literate::cache::exec_cache_key(
        "alpine:3.20",
        "",
        FROZEN_COMMAND,
        &[],
        &hick_literate::cache::inputs_digest(&[]),
        &[],
    );
    hick_literate::cache::cache_store(
        &cc,
        "c",
        &key,
        &hick_literate::cache::ExecCacheEntry {
            commands: vec![FROZEN_COMMAND.trim().to_string()],
            output: output.to_string(),
            output_hash: hick_literate::cache::sha256_hex(output),
        },
    )
    .unwrap();
}

#[test]
fn test_exits_verified_when_nothing_changed() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    assert!(hick().arg("run").arg(&doc).status().unwrap().success());
    let out = hick().arg("test").arg(&doc).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "verified is exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_exits_drifted_when_a_committed_output_is_out_of_date() {
    // Drift is "you forgot to regenerate": every expectation holds, but the
    // committed bytes no longer match what the document produces. A CI job
    // may reasonably auto-fix this one, so it must not share a code with a
    // failed expectation.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "passing.hick", PASSING_DOC);
    assert!(hick().arg("run").arg(&doc).status().unwrap().success());
    std::fs::write(dir.path().join("passing.md"), "hand-edited\n").unwrap();
    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "drifted is exit 1: {stderr}");
    assert!(stderr.contains("DRIFTED"), "names the outcome: {stderr}");
}

#[test]
fn test_exits_expectation_failed_when_a_claim_is_false() {
    // A failed hick:expect is NOT drift: the document says something untrue
    // of its own output, and no amount of regenerating fixes that. It gets
    // its own code (3) so CI can auto-fix drift and never auto-fix this.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "drifted.hick", DRIFTED_DOC);
    // Commit the woven output first, so the ONLY finding is the expectation.
    assert!(hick().arg("run").arg(&doc).status().unwrap().success());
    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(3),
        "a failed expectation is exit 3, not drift's 1: {stderr}"
    );
    assert!(
        stderr.contains("EXPECTATION FAILED"),
        "names the outcome: {stderr}"
    );
    assert!(
        stderr.contains("three") && stderr.contains("two"),
        "shows expected vs actual: {stderr}"
    );
}

#[test]
fn a_failed_expectation_outranks_drift() {
    // Both present: the exit code must be the one that says "a human decides",
    // or a CI job that auto-regenerates on drift would quietly bury a false
    // claim by committing over it.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "drifted.hick", DRIFTED_DOC);
    assert!(hick().arg("run").arg(&doc).status().unwrap().success());
    std::fs::write(dir.path().join("drifted.md"), "hand-edited\n").unwrap();
    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(3),
        "expectation-failed outranks drifted: {stderr}"
    );
    // Both are still REPORTED — only the exit code is a single verdict.
    assert!(
        stderr.contains("drifted.md"),
        "drift reported too: {stderr}"
    );
}

#[test]
fn test_exits_unverifiable_when_a_cell_has_no_baseline() {
    // The bug this closes: a document whose cell never ran used to pass
    // `check`. A frozen cell with no recording never executes AND has
    // nothing to be checked against — nothing was ever established, which is
    // not the same fact as "something changed".
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "frozen.hick", &frozen_doc());
    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "unverifiable gets its own exit code, not drift's: {stderr}"
    );
    // Per .instructions/user-facing-errors.md: which cell, why, what to do.
    assert!(
        stderr.contains("frozen.hick"),
        "names the document: {stderr}"
    );
    assert!(stderr.contains("container 'c'"), "names the cell: {stderr}");
    assert!(
        stderr.contains("line 7") || stderr.contains("line 8"),
        "names the source line of the exec: {stderr}"
    );
    assert!(
        stderr.contains("freeze=\"true\""),
        "says why there is no baseline: {stderr}"
    );
    assert!(
        stderr.contains("remove freeze=\"true\""),
        "says one thing to do about it: {stderr}"
    );
    // #10: the remedy must be the one that exists NOW. `hick run` records
    // a cell frozen from the start on its first run, so that is what the
    // message points at — not the old un-freeze / record / re-freeze dance,
    // and never `hick run --cache`, a binary that has never existed.
    // docs/guarantees/verification/recordings-are-written-only-when-asked-for.md
    assert!(
        stderr.contains("hick run "),
        "must name the command that writes a recording: {stderr}"
    );
    assert!(
        !stderr.contains("hick run --cache"),
        "must not name a binary that does not exist: {stderr}"
    );
    assert!(
        !stderr.contains("restore freeze"),
        "the two-step dance is gone; no message may still describe it: {stderr}"
    );
    assert!(
        !stderr.contains("hick test --cache") && !stderr.contains("test --cache"),
        "`hick test` has no --cache flag: {stderr}"
    );
}

#[test]
fn run_records_a_cell_frozen_from_the_start_and_test_then_verifies_it() {
    // Issue #10 end to end, through the shipped binary: write the cell frozen,
    // `hick run` once, and `hick test` is verified — with no edit to the
    // document in between, and no --cache flag anywhere.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "frozen.hick", &frozen_doc());

    let out = hick().arg("test").arg(&doc).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "before any run there is no baseline: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !dir.path().join(".hick-cache").exists(),
        "`hick test` must not create a recording, or it verifies its own work"
    );

    let out = hick().arg("run").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "run establishes the baseline instead of failing: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = hick().arg("test").arg(&doc).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "the cell has a baseline now: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_reports_unverifiable_when_the_recording_directory_exists_but_the_cell_is_not_in_it() {
    // A recording directory with the wrong (or no) entry for this cell is
    // still no baseline for THIS cell.
    let dir = tempfile::tempdir().unwrap();
    record_frozen_cell(dir.path(), "one\ntwo\n");
    // Retire the recording the way an edit would: a different command means a
    // different key, so the directory exists but this cell is not in it.
    let doc = write_doc(
        dir.path(),
        "frozen.hick",
        &frozen_doc().replace("printf 'one\\ntwo\\n'", "printf 'changed\\n'"),
    );
    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("printf 'changed"),
        "names the command that has no recording: {stderr}"
    );
}

#[test]
fn unverifiable_outranks_drift() {
    // A cell with no baseline makes the whole document's drift verdict
    // untrustworthy, so the stronger "nothing was established" wins even
    // though the committed output is also stale.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "frozen.hick", &frozen_doc());
    std::fs::write(dir.path().join("frozen.md"), "hand-edited\n").unwrap();
    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "unverifiable outranks drifted: {stderr}"
    );
    assert!(stderr.contains("UNVERIFIABLE"), "{stderr}");
}

#[test]
fn a_failed_expectation_outranks_unverifiable() {
    // An unverifiable cell never evaluates an expectation, so a failed
    // expectation always belongs to a cell that really ran: it is a genuine
    // finding, not a consequence of the missing baseline, and it is the one
    // outcome no automation may act on by itself.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        "mixed.hick",
        &format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="mixed.md">
# Mixed

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c" freeze="true">{FROZEN_COMMAND}</hick:exec>

<hick:exec container="c">
printf 'one\ntwo\n'
<hick:expect match="exact">one
three
</hick:expect>
</hick:exec>
</hick:doc>
"#
        ),
    );
    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(3),
        "expectation-failed outranks unverifiable: {stderr}"
    );
    // Both findings are still printed; only the verdict is singular.
    assert!(stderr.contains("UNVERIFIABLE"), "{stderr}");
    assert!(stderr.contains("EXPECTATION FAILED"), "{stderr}");
}

#[test]
fn a_frozen_cell_served_from_its_recording_is_verified_not_unverifiable() {
    // The interaction that must not regress: freeze is a baseline, not the
    // absence of one. A frozen cell WITH a recording has been verified
    // against something, and must report verified.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), "frozen.hick", &frozen_doc());
    record_frozen_cell(dir.path(), "one\ntwo\n");

    // `run` serves the frozen cell from the recording and writes frozen.md.
    let run_out = hick().arg("run").arg(&doc).output().unwrap();
    assert!(
        run_out.status.success(),
        "run: {}",
        String::from_utf8_lossy(&run_out.stderr)
    );

    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "a frozen cell with a recording HAS a baseline: {stderr}"
    );
    assert!(
        !stderr.contains("UNVERIFIABLE"),
        "must not be reported unverifiable: {stderr}"
    );
}

/// `check` was renamed to `test` with no alias and no deprecation shim, so
/// the old verb must be an unknown subcommand — not a hidden alias that
/// quietly keeps working. `cargo check` promises "don't build, don't run",
/// which is the opposite of what this command does: it re-executes every
/// cell in the document.
#[test]
fn the_old_check_subcommand_is_gone() {
    let out = hick().arg("check").arg("some-doc.hick").output().unwrap();
    assert!(
        !out.status.success(),
        "the removed `hick check` subcommand still runs: {out:?}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unrecognized subcommand"),
        "`hick check` did not fail as an unknown subcommand: {stderr}"
    );
}
