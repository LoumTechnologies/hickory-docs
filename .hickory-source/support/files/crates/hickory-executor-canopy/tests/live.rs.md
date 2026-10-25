<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-executor-canopy/tests/live.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `9291635ec5c301f19cd57826371929cbb96e03e92dabb310308011579bf36ff8`.

<book:entry path="crates/hickory-executor-canopy/tests/live.rs">
//! Live smoke test against a real canopy-agent.
//!
//! Ignored by default: it needs a running agent (e.g. cloud-canopy's
//! `just agent-dev`) plus real env config. Run with:
//!
//! ```text
//! HICKORY_CANOPY_LIVE=1 \
//! CANOPY_AGENT=/tmp/canopy-dev.sock \
//! CANOPY_IMAGE_MAP='{"alpine:3.20":"/nix/store/...-canopy-sandbox-image"}' \
//! cargo test -p hickory-executor-canopy --test live -- --ignored
//! ```
//!
//! Status of the last attempt (2026-08-05): the dev agent's stub compute
//! backend accepts SpawnSandbox but boots no real guest, so the attach
//! round-trip cannot complete there — see the "Live smoke status" section of
//! docs/specs/freeform/canopy-integration.md for exactly how far it got.

use hickory_executor::Executor;
use hickory_executor_canopy::CanopyExecutor;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a live canopy-agent; set HICKORY_CANOPY_LIVE=1 and CANOPY_* env"]
async fn live_spawn_exec_destroy() {
    if std::env::var("HICKORY_CANOPY_LIVE").as_deref() != Ok("1") {
        eprintln!("HICKORY_CANOPY_LIVE != 1; skipping live smoke");
        return;
    }
    let ex = CanopyExecutor::from_env().expect("CANOPY_* env must be configured");
    let image = std::env::var("HICKORY_CANOPY_LIVE_IMAGE").unwrap_or_else(|_| {
        ex.config()
            .image_map
            .keys()
            .next()
            .cloned()
            .expect("CANOPY_IMAGE_MAP must configure at least one image")
    });

    ex.ensure_started("live-smoke", &image)
        .await
        .expect("spawn + boot");
    let out = ex
        .execute("live-smoke", "echo hickory-live-ok")
        .await
        .expect("exec over the attach stream");
    assert_eq!(out.trim(), "hickory-live-ok");
    ex.shutdown().await.expect("destroy sandbox");
}
</book:entry>

</book:doc>
