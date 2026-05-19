//! copper-ledge: standard AI coding agent.
//!
//! A thin CLI/TUI wrapper wiring [`hick_agent_sdk`] (runtime) and
//! [`hick_agent_script_first`] (script-first ReAct strategy). Zero knowledge of
//! hick XML, hick pipelines, or any hickory-specific format.

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
/// OpenAI-compatible chat completion client.
pub use hick_agent_sdk::OpenAiCompatClient;
/// Script-first ReAct loop.
pub use hick_agent_script_first::ReactLoop;
/// Validates outbound API sink calls against capability tokens and sink definitions.
pub use hick_agent_sdk::SinkGuard;
