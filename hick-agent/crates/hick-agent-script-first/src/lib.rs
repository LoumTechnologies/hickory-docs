//! Script-first ReAct agent strategy — extension crate on top of `hick-agent-sdk`.
//!
//! Provides one specific agent strategy: the LLM is prompted to write fenced
//! code blocks (Python, shell, JS, C#, TypeScript), those scripts are extracted
//! and executed in a WASM sandbox, and stdout is fed back as the next
//! observation.
//!
//! This crate has zero knowledge of hick XML, hick pipelines, or hickory-specific
//! formats. It works on any codebase.

pub mod events;
pub mod react_loop;
pub mod state;
pub mod tools;

/// Events emitted by the agent core to the UI (tokens, script results, errors).
pub use events::AgentEvent;
/// Commands sent from the UI to the agent core (user messages, quit).
pub use events::AgentCommand;
/// Script-first ReAct loop: extracts code blocks from LLM responses and executes them in a sandbox.
pub use react_loop::ReactLoop;
/// State machine for the script-first ReAct agent loop.
pub use state::AgentState;
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
