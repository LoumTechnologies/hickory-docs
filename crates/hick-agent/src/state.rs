//! Strongly-typed state machine for the ReAct agent loop.

use crate::tools::{ToolCall, ToolResult};

/// State of the agent in the ReAct loop.
#[derive(Debug, Clone)]
pub enum AgentState {
    /// Initial state, or after receiving user input.
    Idle,

    /// Waiting for LLM response.
    Thinking { messages_sent: usize },

    /// LLM returned tool call(s). Ready to execute.
    Acting {
        tool_calls: Vec<ToolCall>,
        thought: Option<String>,
    },

    /// Tool(s) executed. Ready to send results back to LLM.
    Observing { results: Vec<ToolResult> },

    /// Agent produced final answer.
    Done { message: String },

    /// Error occurred.
    Error { message: String },
}

impl AgentState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, AgentState::Done { .. } | AgentState::Error { .. })
    }

    /// Transition: Idle/Observing -> Thinking (send to LLM)
    pub fn to_thinking(self, messages_sent: usize) -> Self {
        AgentState::Thinking { messages_sent }
    }

    /// Transition: Thinking -> Acting (LLM returned tool calls)
    pub fn to_acting(tool_calls: Vec<ToolCall>, thought: Option<String>) -> Self {
        AgentState::Acting {
            tool_calls,
            thought,
        }
    }

    /// Transition: Thinking -> Done (LLM returned final answer)
    pub fn to_done(message: String) -> Self {
        AgentState::Done { message }
    }

    /// Transition: Acting -> Observing (tools executed)
    pub fn to_observing(results: Vec<ToolResult>) -> Self {
        AgentState::Observing { results }
    }

    /// Transition: any -> Error
    pub fn to_error(message: impl Into<String>) -> Self {
        AgentState::Error {
            message: message.into(),
        }
    }
}
