//! Re-export shim for `hick-agent-engine`.
//!
//! All agent logic has been split into two crates:
//! - [`hick_agent_sdk`] — generic building blocks (LLM clients, sandbox, network filter, etc.)
//! - [`hick_agent_script_first`] — script-first ReAct strategy (ReactLoop, PipelineWatcher, events, tools)
//!
//! The `local` llama.cpp provider lives here because llama-cpp-2 is a heavy optional dep
//! that only the binary needs; it is not suitable for the sdk.

// ── Script-first strategy re-exports ─────────────────────────────────────────

/// Commands sent from the UI to the agent core (user messages, quit).
pub use hick_agent_script_first::AgentCommand;
/// Events emitted by the agent core to the UI (tokens, script results, errors).
pub use hick_agent_script_first::AgentEvent;
/// State machine for the script-first ReAct agent loop.
pub use hick_agent_script_first::AgentState;
/// Watches `.hick` files, transparently reruns the pipeline, and applies reverse provenance.
pub use hick_agent_script_first::PipelineWatcher;
/// Script-first ReAct loop: extracts code blocks from LLM responses and executes them in a sandbox.
pub use hick_agent_script_first::ReactLoop;
/// A code block extracted from an LLM response.
pub use hick_agent_script_first::CodeBlock;
/// Execution backend for an extracted fenced code block.
pub use hick_agent_script_first::Language;
/// Result of executing a script.
pub use hick_agent_script_first::ScriptResult;

// ── SDK re-exports ────────────────────────────────────────────────────────────

/// Anthropic Messages API client with streaming support.
pub use hick_agent_sdk::AnthropicClient;
/// A chunk of streamed LLM output.
pub use hick_agent_sdk::ChatChunk;
/// An [`LlmClient`] wrapper that checks a [`NetworkFilter`] before each API call.
pub use hick_agent_sdk::FilteredLlmClient;
/// Trait for LLM chat completion clients (streaming and non-streaming).
pub use hick_agent_sdk::LlmClient;
/// A chat message with a role and content string.
pub use hick_agent_sdk::Message;
/// Request sent to the UI to prompt the user for network access approval.
pub use hick_agent_sdk::NetworkApprovalRequest;
/// Filters outbound network requests by host against configurable rules.
pub use hick_agent_sdk::NetworkFilter;
/// Verdict for a filtered network request: Allow or Block.
pub use hick_agent_sdk::NetworkVerdict;
/// OpenAI-compatible chat completion client.
pub use hick_agent_sdk::OpenAiCompatClient;
/// Chat message role: System, User, or Assistant.
pub use hick_agent_sdk::Role;
/// Top-level agent security configuration loaded from `.hick/agent-security.toml`.
pub use hick_agent_sdk::AgentSecurityConfig;
/// Validates outbound API sink calls against capability tokens and sink definitions.
pub use hick_agent_sdk::SinkGuard;

// ── Sub-module shims for path-qualified imports ────────────────────────────────
//
// tui.rs uses `hick_agent_engine::events::*`, `hick_agent_engine::network_filter::*`,
// and `hick_agent_engine::tools::*`. These thin modules keep those import paths working.

pub mod events {
    pub use hick_agent_script_first::events::*;
}

pub mod network_filter {
    pub use hick_agent_sdk::network_filter::*;
}

pub mod tools {
    pub use hick_agent_script_first::tools::*;
}

// ── hick_policy re-export ─────────────────────────────────────────────────────
//
// main.rs uses `hick_agent_engine::hick_policy::PolicyEvaluator::new(...)`.

pub use hick_policy;

// ── Local llama.cpp provider (stays in this crate) ───────────────────────────

#[cfg(feature = "local")]
pub mod llm_local;
#[cfg(feature = "local")]
/// Local GGUF model inference client using llama.cpp (requires `local` feature).
pub use llm_local::LocalLlmClient;
