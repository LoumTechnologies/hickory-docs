//! Event types for communication between the agent core and UI frontends.

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
    /// About to execute a script (any language).
    ScriptStarted { code: String, lang: String },
    /// Script execution finished.
    ScriptFinished(ScriptResult),
    /// Agent produced a final answer (no more code blocks).
    Done(String),
    /// Agent was interrupted by the user.
    Interrupted,
    /// An error occurred.
    Error(String),
    /// Data classification label applied to agent output.
    DataClassification {
        source: String,
        classification: String,
    },
    /// A security rule blocked an operation.
    SecurityViolation { description: String },
    /// The agent re-prompted the LLM due to a malformed response.
    Reprompt { reason: String, attempt: usize },
}

/// Commands sent from the UI to the agent core.
#[derive(Debug, Clone)]
pub enum AgentCommand {
    /// User submitted a message.
    Message(String),
    /// User typed `! <cmd>` — run the command directly in the WASM container
    /// without calling the LLM.
    DirectCommand(String),
    /// User wants to quit the application.
    Quit,
}
