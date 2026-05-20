//! hick: universal AI coding agent.
//!
//! A CLI/TUI binary wiring [`hick_agent_sdk`] (runtime),
//! [`hick_agent_script_first`] (script-first ReAct strategy), and
//! [`hick_agent_hick_edit`] (hick pipeline integration) together.
//! Extension activation is driven by `.hick/agent.toml`.

/// Event types for agent-to-UI communication.
pub use hick_agent_script_first::events;
/// Network request filtering and approval prompts.
pub use hick_agent_sdk::network_filter;
/// Code block extraction and script execution tools.
pub use hick_agent_script_first::tools;
/// LLM client traits and message types.
pub use hick_agent_sdk::llm;

/// Commands sent from the UI to the agent core.
pub use hick_agent_script_first::AgentCommand;
/// Events emitted by the agent core to the UI.
pub use hick_agent_script_first::AgentEvent;
/// Security configuration loaded from `.hick/agent-security.toml`.
pub use hick_agent_sdk::AgentSecurityConfig;
/// Anthropic Messages API client with streaming support.
pub use hick_agent_sdk::AnthropicClient;
/// A chunk of streamed LLM output.
pub use hick_agent_sdk::ChatChunk;
/// LLM client wrapper that checks a [`NetworkFilter`] before each API call.
pub use hick_agent_sdk::FilteredLlmClient;
/// Trait for LLM chat completion clients.
pub use hick_agent_sdk::LlmClient;
/// Request sent to the UI to prompt the user for network access approval.
pub use hick_agent_sdk::NetworkApprovalRequest;
/// Filters outbound network requests by host against configurable rules.
pub use hick_agent_sdk::NetworkFilter;
/// Verdict for a filtered network request: Allow or Block.
pub use hick_agent_sdk::NetworkVerdict;
/// Watches `.hick` files, transparently reruns the pipeline, and applies reverse provenance.
pub use hick_agent_hick_edit::PipelineWatcher;
/// Script-first ReAct loop.
pub use hick_agent_script_first::ReactLoop;
/// Validates outbound API sink calls against capability tokens and sink definitions.
pub use hick_agent_sdk::SinkGuard;
/// Local GGUF model inference client using llama.cpp (requires `local` feature).
#[cfg(feature = "local")]
pub use hick_agent_engine::LocalLlmClient;
