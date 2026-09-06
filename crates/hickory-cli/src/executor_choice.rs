//! `ExecutorChoice`: which backend runs a document's cells, and how one is
//! built for a given run.
//!
//! Its own module rather than another hundred lines of `lib.rs`, which is
//! already far past the file-length ratchet's line and may only shrink
//! (`scripts/check-file-length.sh`). Nothing about the type changed in the
//! move; `crate::ExecutorChoice` still names it.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, bail};
use hick_literate::{Executor, LocalExecutor};

/// Which executor backend to use, from `HICKORY_EXECUTOR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorChoice {
    Local,
    /// The local executor, with each cell confined to its own workdir.
    /// See `hickory-executor-sandbox`.
    Sandbox,
    Docker,
    Canopy,
}

impl ExecutorChoice {
    /// The name this backend is known by — the `HICKORY_EXECUTOR` value, the
    /// `GET /api/executor` field, and what the `hick serve` banner prints.
    pub fn as_str(self) -> &'static str {
        match self {
            ExecutorChoice::Local => "local",
            ExecutorChoice::Sandbox => "sandbox",
            ExecutorChoice::Docker => "docker",
            ExecutorChoice::Canopy => "canopy",
        }
    }

    /// Read `HICKORY_EXECUTOR` (default: `sandbox`).
    ///
    /// The default is the confined one, and it refuses to run where it cannot
    /// confine anything. A document is a file people send each other and
    /// agents write; "read every cell before running it" is advice nobody
    /// follows twice, so the safe execution has to be the one you get without
    /// asking. `HICKORY_EXECUTOR=local` is still there, one variable away,
    /// and says plainly what it gives up.
    pub fn from_env() -> Result<Self> {
        match std::env::var("HICKORY_EXECUTOR").as_deref() {
            Err(_) | Ok("") | Ok("sandbox") => Ok(ExecutorChoice::Sandbox),
            Ok("local") => Ok(ExecutorChoice::Local),
            Ok("docker") => Ok(ExecutorChoice::Docker),
            Ok("canopy") => Ok(ExecutorChoice::Canopy),
            Ok(other) => bail!(
                "unknown HICKORY_EXECUTOR value '{other}' (expected \"local\", \"sandbox\", \
                 \"docker\", or \"canopy\")"
            ),
        }
    }

    /// Build the executor. Canopy reads its `CANOPY_*` env config here
    /// (connection to the node agent is lazy — first use).
    ///
    /// Docker probes the daemon eagerly, so an unreachable Docker is a clear
    /// startup error rather than a cell that fails halfway through a run —
    /// which means this is `async`.
    pub async fn build(self) -> Result<Arc<dyn Executor>> {
        self.build_for(None).await
    }

    /// [`build`](Self::build), naming the project the run belongs to.
    ///
    /// The local executors derive a scratch directory from this so a cell
    /// that prints its own path reproduces. `None` means "use the working
    /// directory", which is right for a caller serving a folder and wrong for
    /// one that was handed a document elsewhere — see
    /// [`LocalExecutor::new_stable_for`].
    pub async fn build_for(self, project: Option<&Path>) -> Result<Arc<dyn Executor>> {
        match self {
            // The derived scratch directory: `hick` is one executor in one
            // process, which is the case it is safe and useful for.
            ExecutorChoice::Local => Ok(Arc::new(LocalExecutor::new_stable_for(project)?)),
            ExecutorChoice::Sandbox => Ok(Arc::new(
                hickory_executor_sandbox::SandboxedExecutor::new_stable_for(project)?,
            )),
            ExecutorChoice::Docker => Ok(Arc::new(
                hickory_executor_docker::DockerExecutor::new().await?,
            )),
            ExecutorChoice::Canopy => Ok(Arc::new(
                hickory_executor_canopy::CanopyExecutor::from_env()?,
            )),
        }
    }

    /// [`build_for`](Self::build_for) for a run that has a DOCUMENT in hand.
    ///
    /// The project is the document's own directory, exactly as before, and
    /// the local executors additionally learn the document itself, which is
    /// what lets them export `HICK_DOC_DIR` and `HICK_DOC_PATH` into every
    /// cell (`LocalExecutor::for_document`). A cell's cwd stays the scratch
    /// directory; the variables are how a document reaches the repository it
    /// is part of without writing an absolute path into a file people send
    /// each other. See
    /// `docs/guarantees/execution/a-cell-knows-where-its-document-is.md`.
    ///
    /// Docker and canopy run a cell somewhere the host's paths do not mean
    /// anything, so they are built exactly as [`build_for`](Self::build_for)
    /// builds them and export nothing: a variable naming a directory that
    /// does not exist in the container would be worse than its absence.
    pub async fn build_for_document(self, doc: &Path) -> Result<Arc<dyn Executor>> {
        let project = doc.parent();
        match self {
            ExecutorChoice::Local => Ok(Arc::new(
                LocalExecutor::new_stable_for(project)?.for_document(doc),
            )),
            ExecutorChoice::Sandbox => Ok(Arc::new(
                hickory_executor_sandbox::SandboxedExecutor::new_stable_for(project)?
                    .for_document(doc),
            )),
            ExecutorChoice::Docker | ExecutorChoice::Canopy => self.build_for(project).await,
        }
    }
}
