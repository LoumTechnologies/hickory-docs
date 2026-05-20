//! Script-first agent strategy — extension crate on top of `hick-agent-sdk`.
//!
//! Provides the script-first agent strategy: the LLM writes fenced code blocks
//! (Python, shell, JS, C#, TypeScript), those scripts are extracted and executed
//! in a WASM sandbox, and stdout is fed back as the next observation.
//!
//! This crate has zero knowledge of hick XML, hick pipelines, or hickory-specific
//! formats. It works on any codebase.

pub mod agent_ui_bridge;
pub mod events;
pub mod react_loop;
pub mod tools;

/// Events emitted by the agent core to the UI (tokens, script results, errors).
pub use events::AgentEvent;
/// Bridges core [`AgentEvent`]s into SDK [`UiEvent`]s (shared UI layer).
pub use agent_ui_bridge::AgentUiBridge;
/// Fanout task: merges `AgentEvent`s, approvals, and `UiCommand`s into `UiEvent`s.
/// Handles `RewindTo` commands directly; forwards others via [`ui_command_to_agent`].
pub use agent_ui_bridge::run_ui_event_fanout_task;
/// Best-effort [`UiCommand`] → [`AgentCommand`] mapping for chrome-only controls.
pub use agent_ui_bridge::ui_command_to_agent;
/// Commands sent from the UI to the agent core (user messages, quit).
pub use events::AgentCommand;
/// [`MatchIntent`] impl for the script-first strategy — parses `<hick:next>` responses.
pub use react_loop::ScriptBlockIntentParser;
/// Payload produced by [`ScriptBlockIntentParser`] for code or invalid responses.
pub use react_loop::ScriptBlockPayload;
/// Validate a `<hick:next>` response for protocol compliance.
pub use react_loop::validate_response;
/// Build the correction message to inject when an LLM response violates the protocol.
pub use react_loop::correction_message;
/// Default system prompt for the script-first agent strategy.
pub use react_loop::SYSTEM_PROMPT;
/// Sandbox execution context appended to the system prompt in WASM mode.
pub use react_loop::SANDBOX_EXECUTION_CONTEXT;
/// Shell probe command run once per session to enumerate sandbox runtimes.
pub use react_loop::SANDBOX_RUNTIME_PROBE;
/// A code block extracted from an LLM response.
pub use tools::CodeBlock;
/// Execution backend for an extracted fenced code block.
pub use tools::Language;
/// Result of executing a script.
pub use tools::ScriptResult;
/// Extract all supported fenced code blocks from an LLM response.
pub use tools::extract_code_blocks;
/// Backward-compatible wrapper: extract only Python blocks as raw strings.
pub use tools::extract_python_blocks;

/// Re-export hick_policy so consumers can reference `hick_agent_script_first::hick_policy`.
pub use hick_policy;
