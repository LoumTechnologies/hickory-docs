//! Executor selection — the canopy adapter boundary.
//!
//! TODO(canopy): when `crates/hickory-executor-canopy` lands, add it as a
//! dependency and construct `CanopyExecutor` here from `CANOPY_URL`,
//! `CANOPY_TOKEN`, `CANOPY_NODE`, `CANOPY_IMAGE_MAP` (see
//! docs/specs/freeform/canopy-integration.md). This module is the ONLY place
//! in the server that may know about canopy. Until then,
//! `Config::from_env` refuses `HICKORY_EXECUTOR=canopy` in strict mode and
//! degrades to local in dev, so `ExecutorKind::Canopy` never reaches here.

use std::sync::Arc;

use anyhow::{Result, bail};
use hick_literate::{Executor, LocalExecutor};

use crate::config::ExecutorKind;

pub fn build_executor(kind: ExecutorKind) -> Result<Arc<dyn Executor>> {
    match kind {
        ExecutorKind::Local => Ok(Arc::new(LocalExecutor::new()?)),
        ExecutorKind::Canopy => bail!(
            "canopy executor not built into this binary (hickory-executor-canopy has not landed)"
        ),
    }
}
