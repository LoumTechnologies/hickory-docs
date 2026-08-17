//! Per-cell execution timeouts through the live pipeline.
//!
//! These protect `docs/guarantees/execution/a-cell-cannot-hang-a-run.md`:
//! the run-wide default bounds every cell, a cell's own `timeout=` attribute
//! overrides it in both directions, and `timeout="0"` is an explicit
//! unbounded declaration — never a silent one.

#![cfg(unix)] // the sleeping cells below use `sh`'s `sleep`

use std::sync::Arc;
use std::time::{Duration, Instant};

use hick_literate::cell_timeout::CellTimeoutDefault;
use hick_literate::{LocalExecutor, PipelineConfig, run_pipeline_live};

fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="c" image="alpine" />
{body}
</hick:doc>"#
    )
}

async fn run_with_default(src: &str, cell_timeout: CellTimeoutDefault) -> anyhow::Result<()> {
    let config = PipelineConfig {
        cell_timeout,
        ..Default::default()
    };
    run_pipeline_live(
        &[("test.hick", src)],
        &config,
        &[],
        None,
        Arc::new(LocalExecutor::new()?),
    )
    .await
    .map(|_| ())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_run_wide_default_bounds_a_cell_without_an_attribute() {
    let src = hick_doc(r#"<hick:exec container="c">sleep 30</hick:exec>"#);
    let started = Instant::now();
    let err = run_with_default(&src, CellTimeoutDefault::limit(Duration::from_millis(300)))
        .await
        .unwrap_err();
    assert!(started.elapsed() < Duration::from_secs(10));
    let msg = err.to_string();
    assert!(msg.contains("timed out"), "{msg}");
    assert!(msg.contains("sleep 30"), "must name the command: {msg}");
    assert!(msg.contains("HICKORY_CELL_TIMEOUT"), "{msg}");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_timeout_attribute_overrides_the_default_upward() {
    // Default far too small for the cell; its own timeout="5" rescues it.
    let src = hick_doc(r#"<hick:exec container="c" timeout="5">sleep 0.3; echo ok</hick:exec>"#);
    run_with_default(&src, CellTimeoutDefault::limit(Duration::from_millis(50)))
        .await
        .expect("the cell's own timeout must win over the run-wide default");
}

#[tokio::test(flavor = "multi_thread")]
async fn timeout_zero_is_explicitly_unbounded() {
    // Same tiny default, but the cell declares itself unbounded.
    let src = hick_doc(r#"<hick:exec container="c" timeout="0">sleep 0.3; echo ok</hick:exec>"#);
    run_with_default(&src, CellTimeoutDefault::limit(Duration::from_millis(50)))
        .await
        .expect("timeout=\"0\" must remove the limit for this one cell");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_default_constructed_config_still_has_a_limit() {
    // No caller decision at all must still mean "bounded": the built-in
    // 120-second default, not unbounded.
    assert_eq!(
        CellTimeoutDefault::default(),
        CellTimeoutDefault::limit(Duration::from_secs(120))
    );
}
