//! Strongly-typed state machine for the script-first ReAct agent loop.

use crate::tools::ScriptResult;

/// State of the agent in the ReAct loop.
#[derive(Debug, Clone)]
pub enum AgentState {
    /// Initial state, or after receiving user input.
    Idle,

    /// Waiting for LLM response.
    Thinking { messages_sent: usize },

    /// LLM returned Python script(s) in code blocks. Ready to execute.
    Executing {
        scripts: Vec<String>,
        thought: Option<String>,
    },

    /// Script(s) executed. Ready to send results back to LLM.
    Observing { results: Vec<ScriptResult> },

    /// Agent produced final answer (no code blocks in response).
    Done { message: String },

    /// Error occurred.
    Error { message: String },
}

impl AgentState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, AgentState::Done { .. } | AgentState::Error { .. })
    }

    /// Transition: Idle/Observing -> Thinking (send to LLM)
    pub fn to_thinking(messages_sent: usize) -> Self {
        AgentState::Thinking { messages_sent }
    }

    /// Transition: Thinking -> Executing (LLM returned Python code blocks)
    pub fn to_executing(scripts: Vec<String>, thought: Option<String>) -> Self {
        AgentState::Executing { scripts, thought }
    }

    /// Transition: Thinking -> Done (LLM returned final answer, no code blocks)
    pub fn to_done(message: String) -> Self {
        AgentState::Done { message }
    }

    /// Transition: Executing -> Observing (scripts executed)
    pub fn to_observing(results: Vec<ScriptResult>) -> Self {
        AgentState::Observing { results }
    }

    /// Transition: any -> Error
    pub fn to_error(message: impl Into<String>) -> Self {
        AgentState::Error {
            message: message.into(),
        }
    }
}
