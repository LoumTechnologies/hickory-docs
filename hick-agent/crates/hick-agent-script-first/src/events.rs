//! Event types for communication between the agent core and UI frontends.

use crate::tools::ScriptResult;

/// Events emitted by the agent core to the UI.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Informational message (startup, status).
    Info(String),
    /// User submitted a message — emitted before `Thinking` so UIs can create `UserTurn` nodes.
    UserMessage(String),
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
    /// A progress line from a running `/mcp install` task.
    McpInstallProgress(String),
    /// `/mcp install` completed successfully; lists installed tool names.
    McpInstallComplete { tools: Vec<String> },
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
    /// Rewind the LLM context to a branch fork point.
    ///
    /// `user_assistant_history` contains the alternating User/Assistant messages
    /// up to and including the fork node. System messages from the current history
    /// are prepended by the agent task before calling `set_history`.
    RewindTo {
        fork_node_id: hick_agent_sdk::NodeId,
        user_assistant_history: Vec<hick_agent_sdk::Message>,
    },
}
