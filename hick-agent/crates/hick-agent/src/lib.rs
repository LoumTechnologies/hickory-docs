//! AI command-line coding agent.
//!
//! This crate is a thin CLI/TUI wrapper around [`hick_agent_engine`], which
//! re-exports the script-first strategy and SDK building blocks.

/// Event types for agent-to-UI communication.
pub use hick_agent_engine::events;
/// Network request filtering and approval prompts.
pub use hick_agent_engine::network_filter;
/// Code block extraction and script execution tools.
pub use hick_agent_engine::tools;

/// Commands sent from the UI to the agent core.
pub use hick_agent_engine::AgentCommand;
/// Events emitted by the agent core to the UI.
pub use hick_agent_engine::AgentEvent;
/// Security configuration loaded from `.hick/agent-security.toml`.
pub use hick_agent_engine::AgentSecurityConfig;
/// Anthropic Messages API client with streaming support.
pub use hick_agent_engine::AnthropicClient;
/// A chunk of streamed LLM output.
pub use hick_agent_engine::ChatChunk;
/// LLM client wrapper that checks a [`NetworkFilter`] before each API call.
pub use hick_agent_engine::FilteredLlmClient;
/// Trait for LLM chat completion clients.
pub use hick_agent_engine::LlmClient;
/// Request sent to the UI to prompt the user for network access approval.
pub use hick_agent_engine::NetworkApprovalRequest;
/// Filters outbound network requests by host against configurable rules.
pub use hick_agent_engine::NetworkFilter;
/// Verdict for a filtered network request: Allow or Block.
pub use hick_agent_engine::NetworkVerdict;
/// OpenAI-compatible chat completion client.
pub use hick_agent_engine::OpenAiCompatClient;
/// Watches `.hick` files, transparently reruns the pipeline, and applies reverse provenance.
pub use hick_agent_engine::PipelineWatcher;
/// Script-first ReAct loop.
pub use hick_agent_engine::ReactLoop;
/// Validates outbound API sink calls against capability tokens and sink definitions.
pub use hick_agent_engine::SinkGuard;
/// Local GGUF model inference client using llama.cpp (requires `local` feature).
#[cfg(feature = "local")]
pub use hick_agent_engine::LocalLlmClient;

// TUI module — unique to the CLI binary
pub mod tui;
