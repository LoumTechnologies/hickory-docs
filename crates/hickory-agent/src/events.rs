//! Structured events emitted by the agent loop.
//!
//! These serialize to JSON for the WebSocket run channel: the server wraps
//! each one as `{run_id: <session_id>, exec_id: "agent", event: ...}` per
//! `docs/specs/freeform/api.md`. Where an event maps naturally onto the
//! block-model `TranscriptEvent` shape (`cmd`/`out`/`err`/`exit`) the field
//! names match, so the web UI's transcript player can reuse its renderer.

use serde::{Deserialize, Serialize};

use crate::script::ScriptResult;

/// The `exec_id` agent sessions stream under on the WS run channel.
pub const AGENT_EXEC_ID: &str = "agent";

/// Events emitted by the agent loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentEvent {
    /// Session started: model, session file path.
    SessionStarted { model: String, session_path: String },
    /// The user prompt was submitted.
    UserMessage { text: String },
    /// The agent is calling the LLM.
    Thinking,
    /// A streamed token from the LLM response.
    Token { data: String },
    /// LLM response complete (full accepted response text).
    ResponseComplete { text: String },
    /// About to execute a script (`cmd`-like: `data` is the code).
    ScriptStarted { lang: String, data: String },
    /// Script execution finished (`exit`-like: carries the exit code plus
    /// captured output).
    ScriptFinished { result: ScriptResult },
    /// About to execute a document tool (`data` is the raw tool XML).
    ToolStarted { name: String, data: String },
    /// Document tool finished; `text` is the tool-result observation.
    ToolFinished {
        name: String,
        ok: bool,
        text: String,
    },
    /// The agent re-prompted the LLM after a malformed response.
    Reprompt { reason: String, attempt: usize },
    /// Agent produced a final answer.
    Done { summary: String },
    /// An unrecoverable error occurred.
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_serialize_with_kind_tag() {
        let ev = AgentEvent::ScriptStarted {
            lang: "python".into(),
            data: "print(1)".into(),
        };
        let json = serde_json::to_value(&ev).unwrap();
        assert_eq!(json["kind"], "script_started");
        assert_eq!(json["lang"], "python");
    }
}
