//! Freeze is a per-cell declaration, not a run-wide switch.
//!
//! These protect docs/guarantees/verification/freeze-is-declared-per-cell.md.
//!
//! They run against `LocalExecutor`, so "executed" means a real command really
//! ran on this machine; the cells below write to a scratch file so a test can
//! tell "served from the recording" from "ran again" without inspecting
//! transcripts.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use hick_literate::cache::{CacheConfig, CacheMode, ExecCacheEntry, cache_store, sha256_hex};
use hickory_executor::LocalExecutor;

fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
    )
}

/// A scratch project directory, removed on drop, so each test gets its own
/// `.hick-cache/` and never sees another test's recordings.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hick-freeze-test-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn run(
    src: &str,
    cache: Option<&CacheConfig>,
) -> anyhow::Result<hick_literate::PipelineResult> {
    let executor = Arc::new(LocalExecutor::new().unwrap());
    hick_literate::run_pipeline_live(
        &[("test.hick", src)],
        &hick_literate::PipelineConfig::default(),
        &[],
        cache,
        executor,
    )
    .await
}

/// Pre-record what the cache would have stored for a cell, without running it.
/// The key must match what the pipeline computes: image, capabilities, command
/// text, secret names, the digest of the cell's mounted inputs, and its
/// upstream cells' keys. These fixtures mount nothing and have no predecessor,
/// so the last two are the empty digest and an empty list.
fn record(cc: &CacheConfig, container: &str, image: &str, command: &str, output: &str) {
    let key = hick_literate::cache::exec_cache_key(
        image,
        "",
        command,
        &[],
        &hick_literate::cache::inputs_digest(&[]),
        &[],
    );
    cache_store(
        cc,
        container,
        &key,
        &ExecCacheEntry {
            commands: vec![command.trim().to_string()],
            output: output.to_string(),
            output_hash: sha256_hex(output),
        },
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// Cell-level freeze
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_frozen_cell_does_not_freeze_the_run() {
    // The whole point of the change: one cell is checked against its
    // recording while its neighbour executes for real, in the same run, with
    // no run-wide freeze flag anywhere.
    let scratch = Scratch::new("one-cell");
    let marker = scratch.path().join("live-ran.txt");
    let cc = CacheConfig::new(scratch.path(), CacheMode::Off);
    record(&cc, "frozen", "alpine", "\necho recorded\n", "recorded\n");

    let src = hick_doc(&format!(
        r#"<hick:container name="frozen" image="alpine" />
<hick:container name="live" image="alpine" />
<hick:exec container="frozen" freeze="true">
echo recorded
</hick:exec>
<hick:exec container="live">
touch {}
</hick:exec>"#,
        marker.display()
    ));

    run(&src, Some(&cc)).await.expect("run should succeed");
    assert!(
        marker.exists(),
        "the unfrozen cell must still execute — freezing one cell froze the run"
    );
}

#[tokio::test]
async fn a_frozen_cell_serves_its_recording_without_executing() {
    // A frozen cell is answered from the recording, never run. If it ran, the
    // marker file would exist.
    let scratch = Scratch::new("serves-recording");
    let marker = scratch.path().join("frozen-ran.txt");
    let cc = CacheConfig::new(scratch.path(), CacheMode::Off);
    let command = format!("\ntouch {}\n", marker.display());
    record(&cc, "frozen", "alpine", &command, "recorded output\n");

    let src = hick_doc(&format!(
        r#"<hick:container name="frozen" image="alpine" />
<hick:exec container="frozen" freeze="true">{command}</hick:exec>"#
    ));

    let result = run(&src, Some(&cc)).await.expect("run should succeed");
    assert!(
        !marker.exists(),
        "a frozen cell must not execute; it is checked against its recording"
    );
    let entries = result
        .transcripts
        .get("frozen")
        .expect("the frozen cell should still appear in the transcript");
    assert_eq!(
        entries.last().map(|e| e.output.as_str()),
        Some("recorded output\n"),
        "the recorded output should be what the frozen cell reports"
    );
}

#[tokio::test]
async fn a_frozen_cell_without_a_recording_runs_once_and_records_itself() {
    // Issue #10: declaring freeze="true" from the start has to work. The
    // first run has no recording to serve, so it executes the cell and writes
    // one; every later run replays it. The marker file counts executions, so
    // "ran twice" is visible rather than inferred.
    let scratch = Scratch::new("records-on-first-run");
    let marker = scratch.path().join("runs.txt");
    let cc = CacheConfig::new(scratch.path(), CacheMode::Off);
    let command = format!("\nprintf x >> {}\n", marker.display());

    let src = hick_doc(&format!(
        r#"<hick:container name="frozen" image="alpine" />
<hick:exec container="frozen" freeze="true">{command}</hick:exec>"#
    ));

    run(&src, Some(&cc))
        .await
        .expect("the first run establishes the baseline instead of failing");
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "x",
        "a frozen cell with no recording must execute exactly once"
    );

    run(&src, Some(&cc)).await.expect("the second run replays");
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "x",
        "the recording written by the first run must be replayed, not re-executed"
    );
}

#[tokio::test]
async fn editing_a_frozen_cell_re_records_it_on_the_next_run() {
    // The recording is keyed by the command, so editing the command retires
    // it: the cell is unrecorded again, and `run` records it again.
    let scratch = Scratch::new("re-records-after-edit");
    let marker = scratch.path().join("runs.txt");
    let cc = CacheConfig::new(scratch.path(), CacheMode::Off);

    let doc_with = |suffix: &str| {
        hick_doc(&format!(
            r#"<hick:container name="frozen" image="alpine" />
<hick:exec container="frozen" freeze="true">
printf {suffix} >> {}
</hick:exec>"#,
            marker.display()
        ))
    };

    run(&doc_with("a"), Some(&cc)).await.unwrap();
    run(&doc_with("a"), Some(&cc)).await.unwrap();
    assert_eq!(std::fs::read_to_string(&marker).unwrap(), "a");

    run(&doc_with("b"), Some(&cc)).await.unwrap();
    run(&doc_with("b"), Some(&cc)).await.unwrap();
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "ab",
        "a changed command has no recording, so it must run once more and record that"
    );
}

#[tokio::test]
async fn a_frozen_cell_that_ran_once_is_not_re_recorded_over() {
    // The recording is the baseline: a later run must serve it, not refresh
    // it, or freeze would silently track the world instead of the record.
    let scratch = Scratch::new("baseline-is-stable");
    let cc = CacheConfig::new(scratch.path(), CacheMode::Off);
    let command = "\necho live\n";
    let src = hick_doc(&format!(
        r#"<hick:container name="frozen" image="alpine" />
<hick:exec container="frozen" freeze="true">{command}</hick:exec>"#
    ));

    run(&src, Some(&cc)).await.unwrap();
    // Doctor the recording: only a replay can produce this text.
    record(&cc, "frozen", "alpine", command, "DOCTORED\n");

    let result = run(&src, Some(&cc)).await.unwrap();
    assert_eq!(
        result
            .transcripts
            .get("frozen")
            .and_then(|e| e.last())
            .map(|e| e.output.as_str()),
        Some("DOCTORED\n"),
        "the second run must answer from the recording, not execute and overwrite it"
    );
}

// ---------------------------------------------------------------------------
// The run-wide flag, and overriding it
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_run_wide_freeze_serves_what_is_recorded_and_records_what_is_not() {
    // Cells that declare nothing inherit the run-wide default, and the
    // run-wide default is the same three-valued setting a cell can declare —
    // so a miss under `hick run --freeze` establishes the baseline exactly
    // as a miss on a cell-declared freeze does. `hick test --freeze` is
    // the caller that refuses instead; see `test_command_tests.rs`.
    let scratch = Scratch::new("global");
    let marker = scratch.path().join("b-ran.txt");
    let cc = CacheConfig::new(scratch.path(), CacheMode::Require);
    record(&cc, "a", "alpine", "\necho a\n", "recorded a\n");

    let src = hick_doc(&format!(
        r#"<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:exec container="a">
echo a
</hick:exec>
<hick:exec container="b">
printf x >> {}
</hick:exec>"#,
        marker.display()
    ));

    let result = run(&src, Some(&cc))
        .await
        .expect("an unrecorded cell has no baseline yet, and run establishes one");
    assert_eq!(
        result
            .transcripts
            .get("a")
            .and_then(|e| e.last())
            .map(|e| e.output.as_str()),
        Some("recorded a\n"),
        "the recorded cell must be served from its recording"
    );
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "x",
        "the unrecorded cell must execute once"
    );

    run(&src, Some(&cc)).await.unwrap();
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "x",
        "and be replayed from then on"
    );
}

#[tokio::test]
async fn a_cell_can_opt_out_of_a_run_wide_freeze() {
    // freeze="false" beats --freeze: the integration test still runs even
    // though the rest of the document is being checked against recordings.
    let scratch = Scratch::new("opt-out");
    let marker = scratch.path().join("integration-ran.txt");
    let cc = CacheConfig::new(scratch.path(), CacheMode::Require);
    record(&cc, "docs", "alpine", "\necho recorded\n", "recorded\n");

    let src = hick_doc(&format!(
        r#"<hick:container name="docs" image="alpine" />
<hick:container name="integration" image="alpine" />
<hick:exec container="docs">
echo recorded
</hick:exec>
<hick:exec container="integration" freeze="false">
touch {}
</hick:exec>"#,
        marker.display()
    ));

    run(&src, Some(&cc))
        .await
        .expect("the opted-out cell has no recording, but it is allowed to run");
    assert!(
        marker.exists(),
        "freeze=\"false\" must beat --freeze — the cell has to actually execute"
    );
}

#[tokio::test]
async fn an_opted_out_cell_re_executes_instead_of_reusing_its_recording() {
    // Opting out is not just "a miss is not an error": a cell that says it is
    // live must not be answered from a recording either, or an integration
    // test would silently stop testing the world.
    let scratch = Scratch::new("opt-out-ignores-recording");
    let marker = scratch.path().join("ran-again.txt");
    let cc = CacheConfig::new(scratch.path(), CacheMode::Require);
    let command = format!("\ntouch {}\n", marker.display());
    record(&cc, "integration", "alpine", &command, "stale\n");

    let src = hick_doc(&format!(
        r#"<hick:container name="integration" image="alpine" />
<hick:exec container="integration" freeze="false">{command}</hick:exec>"#
    ));

    run(&src, Some(&cc)).await.expect("run should succeed");
    assert!(
        marker.exists(),
        "a live cell with a matching recording must still execute"
    );
}

#[tokio::test]
async fn a_non_boolean_freeze_value_is_rejected() {
    let src = hick_doc(
        r#"<hick:container name="demo" image="alpine" />
<hick:exec container="demo" freeze="maybe">
echo hi
</hick:exec>"#,
    );

    let Err(err) = run(&src, None).await else {
        panic!("freeze=\"maybe\" must not be silently treated as false");
    };
    let msg = err.to_string();
    assert!(
        msg.contains("maybe"),
        "should quote the rejected value: {msg}"
    );
}

#[tokio::test]
async fn a_frozen_cell_without_any_cache_directory_executes_rather_than_failing() {
    // Embedded run paths (server preview, watch, agent tools) pass no cache
    // config at all: they can neither replay the cell nor record it. The
    // declaration cannot be honoured there, and executing is the honest
    // fallback — failing a live preview over a cell that would run fine under
    // `hick run` helps nobody. A warning names the situation.
    let scratch = Scratch::new("no-cache-dir");
    let marker = scratch.path().join("ran.txt");
    let src = hick_doc(&format!(
        r#"<hick:container name="frozen" image="alpine" />
<hick:exec container="frozen" freeze="true">
touch {}
</hick:exec>"#,
        marker.display()
    ));

    run(&src, None)
        .await
        .expect("a run with no recording directory executes the cell instead of failing");
    assert!(
        marker.exists(),
        "with nowhere to read or write a recording, the cell has to execute"
    );
}
