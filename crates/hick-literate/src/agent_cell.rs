//! The boundary between the pipeline and an agent cell's reasoning.
//!
//! `<hick:agent>` is a DAG vertex scheduled by the topological loop in
//! [`crate::run_pipeline_live`] (see
//! `docs/specs/freeform/agent-placement-spike.md`, which settled placement as
//! **exec**). Running one needs an LLM client, and `hick-literate` must not
//! grow an LLM dependency — the same reason the flow placement was going to
//! pull its client out of `Context`'s extension map. So the pipeline calls
//! through this trait and the caller supplies the implementation:
//! `hickory-cli` builds one over `hickory_agent::run_agent`, tests build one
//! over `ScriptedLlmClient`.
//!
//! **No runner configured is a supported state, not an error.** A machine
//! with no API key — CI, an offline laptop, the server's live preview — still
//! runs every other cell in the document, and reports the agent cell as
//! *unverifiable* rather than failing the run or, worse, silently weaving as
//! if nothing were missing. That is `third-party-integration-mocking`'s
//! graceful-degradation rule applied to the one integration that spends money.

use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;

/// One agent cell, as handed to a runner.
#[derive(Debug, Clone)]
pub struct AgentRequest {
    /// The document the cell lives in. The agent's only write channel is
    /// `edit_doc`/`edit_output` against this path.
    pub doc_path: PathBuf,
    /// Directory the run resolves relative paths against; sessions land in
    /// `<project_dir>/sessions/`.
    pub project_dir: PathBuf,
    /// The cell's prompt.
    pub prompt: String,
    /// The cell's `max-turns=`, when declared.
    ///
    /// A graph invariant, not a cost policy: a cell that never settles blocks
    /// the document, so a runner must treat an exhausted budget as a failure
    /// and never as a partial result (`agent-cells.md`, "max_turns is a graph
    /// invariant").
    pub max_turns: Option<usize>,
    /// The cell's `model=`, when declared. A runner that cannot honour it
    /// should fail rather than quietly substitute another model — the
    /// recording key names the model, so substituting one invalidates the
    /// baseline the document is verified against.
    pub model: Option<String>,
    /// 1-based source line of the cell, for messages.
    pub source_line: usize,
}

/// What a runner reports back about one settled agent cell.
#[derive(Debug, Clone)]
pub struct AgentRunOutcome {
    /// The agent's final answer, recorded as the cell's transcript output.
    pub summary: String,
    /// The model that actually ran. This — not the declared attribute — is
    /// what the recording is keyed on when the cell declares no `model=`.
    pub model: String,
    /// Whether the agent changed the document's source on disk.
    ///
    /// The pipeline re-prepares the document when this is true, which is the
    /// only way an `<hick:exec>` written after the cell can observe the
    /// agent's edits in the same pass. A runner that reports `false` after
    /// editing costs the run its fixed point; hashing the file either side of
    /// the run is the cheap, honest way to answer it.
    pub edited_source: bool,
    /// Path of the written `hick:session` document, when one was written.
    /// Lineage points into it, so it is durable repo state, not cache.
    pub session_path: Option<PathBuf>,
}

/// Runs one agent cell to settlement.
#[async_trait]
pub trait AgentRunner: Send + Sync {
    /// The model this runner would use for a cell that declares none.
    ///
    /// Read *before* the cell runs, because the recording key needs a model to
    /// look anything up with. A runner must not later run a different model
    /// than it named here.
    fn model_name(&self) -> &str;

    /// Run one cell to settlement.
    async fn run(&self, request: AgentRequest) -> Result<AgentRunOutcome>;
}
