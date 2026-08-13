//! The `hick` binary's implementation of [`AgentRunner`]: run one
//! `<hick:agent>` cell through the real ReAct loop.
//!
//! `hick-literate` schedules the cell as a DAG vertex and asks this to settle
//! it. Keeping the LLM on this side of the boundary is what stops the pipeline
//! crate from growing a model dependency — the same separation the flow
//! placement was going to get from `Context`'s extension map.
//!
//! **Absence is a first-class answer.** [`LlmAgentRunner::from_env`] returns
//! `None` when no provider key is present, and every caller then runs the
//! document with `agent_runner: None`, which reports each agent cell as
//! *unverifiable* instead of failing. That is what makes `hick test` usable
//! in CI, on a plane, and on a fresh clone.

use std::sync::Arc;

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use hick_literate::agent_cell::{AgentRequest, AgentRunOutcome, AgentRunner};
use hickory_agent::{AgentConfig, LlmClient, ScriptLimits, client_for, run_agent};
use hickory_executor::Executor;

/// Provider selector used when the environment names none.
const DEFAULT_PROVIDER: &str = "anthropic";

/// Environment variable naming the provider for agent cells.
const PROVIDER_ENV: &str = "HICKORY_AGENT_PROVIDER";

/// Environment variable overriding the model for agent cells.
const MODEL_ENV: &str = "HICKORY_AGENT_MODEL";

/// Turn budget used when a cell declares no `max-turns=`.
const DEFAULT_MAX_TURNS: usize = 20;

/// Runs an agent cell through [`run_agent`].
pub struct LlmAgentRunner {
    llm: Arc<dyn LlmClient>,
    executor: Arc<dyn Executor>,
}

impl LlmAgentRunner {
    /// Build a runner over an explicit client (tests use a scripted one).
    pub fn new(llm: Arc<dyn LlmClient>, executor: Arc<dyn Executor>) -> Self {
        Self { llm, executor }
    }

    /// Build a runner from the environment, or `None` when no provider key is
    /// available.
    ///
    /// Deliberately never an error: "this machine cannot run an agent cell" is
    /// an ordinary, expected state, and turning it into a failed run would
    /// mean a document containing one cannot be checked anywhere without
    /// spending money.
    pub fn from_env(executor: Arc<dyn Executor>) -> Option<Self> {
        let selector = std::env::var(PROVIDER_ENV).unwrap_or_else(|_| DEFAULT_PROVIDER.into());
        let model = std::env::var(MODEL_ENV).ok();
        match client_for(&selector, model.as_deref(), None) {
            Ok(llm) => Some(Self::new(llm, executor)),
            Err(e) => {
                log::info!(
                    "no agent runner configured ({e}); <hick:agent> cells will be reported as \
                     unverifiable rather than executed"
                );
                None
            }
        }
    }
}

#[async_trait]
impl AgentRunner for LlmAgentRunner {
    fn model_name(&self) -> &str {
        self.llm.model_name()
    }

    async fn run(&self, request: AgentRequest) -> Result<AgentRunOutcome> {
        // A cell that names a model this runner is not running must fail
        // rather than run anyway: the recording is keyed by the model, so
        // substituting one silently invalidates the baseline the document is
        // verified against.
        if let Some(declared) = &request.model
            && declared != self.llm.model_name()
        {
            anyhow::bail!(
                "the agent cell at line {} declares model=\"{declared}\", but this run is \
                 configured for \"{}\".\n\
                 Next steps: set {MODEL_ENV}={declared} and re-run, or change the cell's model= \
                 to match.\n\
                 Running a different model would not just cost differently — an agent \
                 recording is keyed by the prompt AND the model, so the answer would be filed \
                 under a key the document never asks for.",
                request.source_line,
                self.llm.model_name(),
            );
        }

        let before = std::fs::read(&request.doc_path).ok();

        let mut config = AgentConfig::new(request.prompt.clone(), request.project_dir.clone());
        config.doc_path = Some(request.doc_path.clone());
        config.max_turns = request.max_turns.unwrap_or(DEFAULT_MAX_TURNS);
        config.script_limits = ScriptLimits::default();

        let mut sink = |_event: hickory_agent::AgentEvent| {};
        let outcome = run_agent(self.llm.as_ref(), self.executor.clone(), &config, &mut sink)
            .await
            .with_context(|| {
                format!(
                    "the agent cell at line {} did not settle",
                    request.source_line
                )
            })?;

        // Whether the document changed is answered by the bytes, not by
        // trusting the loop to report its own writes.
        let after = std::fs::read(&request.doc_path).ok();
        let edited_source = before != after;

        Ok(AgentRunOutcome {
            summary: outcome.summary,
            model: self.llm.model_name().to_string(),
            edited_source,
            session_path: Some(outcome.session_path),
        })
    }
}
