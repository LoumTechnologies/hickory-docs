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

/// Build a fresh executor for one run.
pub fn build_executor(kind: ExecutorKind) -> Result<Arc<dyn Executor>> {
    match kind {
        ExecutorKind::Local => Ok(Arc::new(LocalExecutor::new()?)),
        ExecutorKind::Canopy => Ok(Arc::new(
            CanopyExecutor::from_env().context("building canopy executor")?,
        )),
    }
}
