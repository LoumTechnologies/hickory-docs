//! `HICKORY_CELL_TIMEOUT` through the shipped binary.
//!
//! These protect `docs/guarantees/execution/a-cell-cannot-hang-a-run.md`:
//! the machine-wide default is really read from the environment of the
//! process the user runs, a timed-out cell fails the run with the
//! user-facing message, and a malformed value fails loudly at the start of
//! the run instead of silently running unbounded.
//!
//! Environment variables are process-global, so these tests set them only on
//! the SPAWNED `hick` process, never in this test process — that is what
//! keeps them from racing every other test.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

mod common;
use common::{runs_until_killed, sleeps_then_echoes};

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

fn write_doc(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("doc.hick");
    let source = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="doc.md">
<hick:container name="c" image="alpine" />
{body}
</hick:doc>
"#
    );
    std::fs::write(&path, source).unwrap();
    path
}

#[test]
fn the_env_default_cuts_a_sleeping_cell() {
    let sleeper = runs_until_killed();
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        &format!(r#"<hick:exec container="c">{}</hick:exec>"#, sleeper),
    );
    let started = Instant::now();
    let out = hick()
        .arg("run")
        .arg(&doc)
        .env("HICKORY_CELL_TIMEOUT", "1")
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a cell sleeping past HICKORY_CELL_TIMEOUT must fail the run"
    );
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "the run must end near the 1s limit, not after the 30s sleep"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("timed out"), "stderr: {stderr}");
    assert!(
        stderr.contains(sleeper.as_str()),
        "must name the command: {stderr}"
    );
    assert!(
        stderr.contains("HICKORY_CELL_TIMEOUT") && stderr.contains("timeout=\"<seconds>\""),
        "must offer both next steps: {stderr}"
    );
}

#[test]
fn a_malformed_env_value_fails_loudly_before_any_cell_runs() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        r#"<hick:exec container="c">echo should-not-run > ran.txt</hick:exec>"#,
    );
    let out = hick()
        .arg("run")
        .arg(&doc)
        .env("HICKORY_CELL_TIMEOUT", "2m")
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a malformed HICKORY_CELL_TIMEOUT must never silently run unbounded"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("HICKORY_CELL_TIMEOUT") && stderr.contains("2m"),
        "must name the variable and the rejected value: {stderr}"
    );
    assert!(
        stderr.contains("whole number of seconds"),
        "must say what a valid value looks like: {stderr}"
    );
}

#[test]
fn a_cell_attribute_beats_the_env_default() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(
        dir.path(),
        &format!(
            r#"<hick:exec container="c" timeout="30">{}</hick:exec>"#,
            sleeps_then_echoes(2, "ok")
        ),
    );
    let status = hick()
        .arg("run")
        .arg(&doc)
        .env("HICKORY_CELL_TIMEOUT", "1")
        .status()
        .unwrap();
    assert!(
        status.success(),
        "the cell's own timeout=\"30\" must override the 1s machine default"
    );
}
