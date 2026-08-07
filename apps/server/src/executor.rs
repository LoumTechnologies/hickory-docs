//! Executor selection — the canopy adapter boundary.
//!
//! This module is the ONLY place in the server that may know about
//! `hickory-executor-canopy` (see
//! docs/guarantees/execution/canopy-api-isolated-to-one-crate.md). The
//! canopy env contract lives in docs/specs/freeform/canopy-integration.md
//! and is parsed by the adapter crate itself, never here.

use std::sync::Arc;

use anyhow::{Context as _, Result};
use hick_literate::{Executor, LocalExecutor};
use hickory_executor_canopy::{CanopyConfig, CanopyExecutor};

use crate::config::{Config, ExecutorKind};

/// Boot-time validation of the configured executor. Strict mode fails fast
/// on a misconfigured canopy env; dev degrades to local with a warning.
pub fn validate_executor(config: &mut Config) -> Result<()> {
    if config.executor == ExecutorKind::Canopy
        && let Err(e) = CanopyConfig::from_env()
    {
        if config.app_env.strict() {
            return Err(e).context("HICKORY_EXECUTOR=canopy but the canopy env is invalid");
        }
        log::warn!(
            "HICKORY_EXECUTOR=canopy but the canopy env is invalid ({e:#}); \
             falling back to local (dev graceful degradation)"
        );
        config.executor = ExecutorKind::Local;
    }
    Ok(())
}

/// Image-ref → store-path map for `GET /api/executor` when the canopy
/// executor is configured. Display-only: parse failures degrade to an
/// empty map (resolution errors surface at run time).
pub fn canopy_image_map() -> std::collections::HashMap<String, String> {
    hickory_executor_canopy::image_map_summary()
}

/// Build a fresh executor for one run.
///
/// Async because the docker backend probes the daemon before returning: an
/// unreachable Docker should fail the run with one clear message, not a cell
/// halfway through the document.
pub async fn build_executor(kind: ExecutorKind) -> Result<Arc<dyn Executor>> {
    match kind {
        ExecutorKind::Local => Ok(Arc::new(LocalExecutor::new()?)),
        ExecutorKind::Docker => Ok(Arc::new(
            hickory_executor_docker::DockerExecutor::new()
                .await
                .context("building docker executor")?,
        )),
        ExecutorKind::Canopy => Ok(Arc::new(
            CanopyExecutor::from_env().context("building canopy executor")?,
        )),
    }
}
