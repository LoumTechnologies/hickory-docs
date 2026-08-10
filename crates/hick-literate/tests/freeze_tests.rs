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

use hick_literate::cache::{CacheConfig, ExecCacheEntry, cache_store, sha256_hex};
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
/// text, secret names.
fn record(cc: &CacheConfig, container: &str, image: &str, command: &str, output: &str) {
    let key = hick_literate::cache::exec_cache_key(image, "", command, &[]);
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
    let cc = CacheConfig::new(scratch.path(), false, false);
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
    let cc = CacheConfig::new(scratch.path(), false, false);
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
async fn a_frozen_cell_without_a_recording_fails_with_an_actionable_error() {
    let scratch = Scratch::new("no-recording");
    let cc = CacheConfig::new(scratch.path(), false, false);

    let src = hick_doc(
        r#"<hick:container name="frozen" image="alpine" />
<hick:exec container="frozen" freeze="true">
echo never recorded
</hick:exec>"#,
    );

    let Err(err) = run(&src, Some(&cc)).await else {
        panic!("a frozen cell with no recording must fail, not execute");
    };
    let msg = err.to_string();
    for expected in [
        "frozen",              // which container
        "echo never recorded", // which command
        "freeze=\"true\"",     // why it was frozen
        // The next step, named as a command the shipped binary really
        // accepts — see docs/guarantees/verification/recordings-are-written-only-when-asked-for.md
        "hickory run --cache",
        "container image", // why an edit invalidates the recording
    ] {
        assert!(
            msg.contains(expected),
            "error should mention {expected:?}: {msg}"
        );
    }
}

// ---------------------------------------------------------------------------
// The run-wide flag, and overriding it
// ---------------------------------------------------------------------------

#[tokio::test]
async fn global_freeze_requires_every_cell_to_be_recorded() {
    // The pre-existing behaviour of `hickory run --freeze`, unchanged: cells that
    // declare nothing inherit the run-wide default.
    let scratch = Scratch::new("global");
    let cc = CacheConfig::new(scratch.path(), true, true);
    record(&cc, "a", "alpine", "\necho a\n", "a\n");

    let src = hick_doc(
        r#"<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:exec container="a">
echo a
</hick:exec>
<hick:exec container="b">
echo b
</hick:exec>"#,
    );

    let Err(err) = run(&src, Some(&cc)).await else {
        panic!("the unrecorded cell must fail the run under --freeze");
    };
    let msg = err.to_string();
    assert!(
        msg.contains("echo b"),
        "should name the unrecorded cell: {msg}"
    );
    assert!(
        msg.contains("--freeze"),
        "should say the run-wide flag is why: {msg}"
    );
}

#[tokio::test]
async fn a_cell_can_opt_out_of_a_run_wide_freeze() {
    // freeze="false" beats --freeze: the integration test still runs even
    // though the rest of the document is being checked against recordings.
    let scratch = Scratch::new("opt-out");
    let marker = scratch.path().join("integration-ran.txt");
    let cc = CacheConfig::new(scratch.path(), true, true);
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
    let cc = CacheConfig::new(scratch.path(), true, true);
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
async fn a_frozen_cell_without_any_cache_directory_says_so() {
    // Embedded run paths (server preview, watch, agent tools) pass no cache
    // config. A frozen cell there cannot be evaluated, and must say that
    // rather than quietly executing as if it were never frozen.
    let src = hick_doc(
        r#"<hick:container name="frozen" image="alpine" />
<hick:exec container="frozen" freeze="true">
echo hi
</hick:exec>"#,
    );

    let Err(err) = run(&src, None).await else {
        panic!("a frozen cell with no cache directory must not silently execute");
    };
    let msg = err.to_string();
    assert!(
        msg.contains("no cache directory") && msg.contains("freeze=\"true\""),
        "error should explain there is nothing to check against: {msg}"
    );
}
