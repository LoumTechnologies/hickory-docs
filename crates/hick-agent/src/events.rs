//! Event types for communication between the agent core and UI frontends.
//!
//! The core emits [`AgentEvent`]s as it works (streaming tokens, executing
//! scripts, etc.). The UI sends [`AgentCommand`]s back (user messages,
//! interrupt requests). This separation allows multiple frontends (TUI,
//! web, headless) to share the same core logic.

use crate::tools::ScriptResult;

/// Events emitted by the agent core to the UI.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Informational message (startup, status).
    Info(String),
    /// Agent is calling the LLM.
    Thinking,
    /// A streamed token from the LLM response.
    Token(String),
    /// LLM response complete. Contains the full response text.
    ResponseComplete(String),
    /// About to execute a Python script.
    ScriptStarted { code: String },
    /// Script execution finished.
    ScriptFinished(ScriptResult),
    /// Agent produced a final answer (no more code blocks).
    Done(String),
    /// Agent was interrupted by the user.
    Interrupted,
    /// An error occurred.
    Error(String),
}

/// Commands sent from the UI to the agent core.
#[derive(Debug, Clone)]
pub enum AgentCommand {
    /// User submitted a message.
    Message(String),
    /// User wants to quit the application.
    Quit,
}
