//! Embeddable agent engine — ReAct loop, LLM abstraction, security stack.
//!
//! This crate contains the core logic of the AI coding agent, extracted from
//! `hick-agent` so it can be embedded in other frontends (web, native apps,
//! headless runners) without pulling in TUI/CLI dependencies.
//!
//! The agent uses a script-first model: the LLM writes fenced code (shell plus
//! Python/JavaScript/C#/TypeScript paths, etc.), executed in a sandbox with
//! output returned as observations.
//! The Hick pipeline runs transparently via the PipelineWatcher.

pub mod agent;
pub mod backing;
pub mod context;
pub mod events;
pub mod file_io;
pub mod llm;
pub mod llm_anthropic;
#[cfg(feature = "local")]
pub mod llm_local;
pub mod llm_openai_compat;
pub mod network_filter;
pub mod pipeline_watcher;
pub mod react_loop;
pub mod reverse_provenance;
pub mod security_config;
pub mod sink_guard;
pub mod state;
pub mod tools;

/// AI coding agent wrapping an LLM client and project configuration.
pub use agent::Agent;
/// Configuration for the coding agent (project directory, images directory, default image).
pub use agent::AgentConfig;
/// Commands sent from the UI to the agent core (user messages, quit).
pub use events::AgentCommand;
/// Events emitted by the agent core to the UI (tokens, script results, errors).
pub use events::AgentEvent;
/// A chunk of streamed LLM output containing a text delta and optional finish reason.
pub use llm::ChatChunk;
/// Trait for LLM chat completion clients (streaming and non-streaming).
pub use llm::LlmClient;
/// A chat message with a role and content string.
pub use llm::Message;
/// Chat message role: System, User, or Assistant.
pub use llm::Role;
/// Anthropic Messages API client with streaming support.
pub use llm_anthropic::AnthropicClient;
/// Local GGUF model inference client using llama.cpp (requires `local` feature).
#[cfg(feature = "local")]
pub use llm_local::LocalLlmClient;
/// OpenAI-compatible chat completion client (works with OpenAI, xAI, Groq, Ollama, etc.).
pub use llm_openai_compat::OpenAiCompatClient;
/// LLM client wrapper that checks a [`NetworkFilter`] before each API call.
pub use network_filter::FilteredLlmClient;
/// Request sent to the UI to prompt the user for network access approval.
pub use network_filter::NetworkApprovalRequest;
/// Filters outbound network requests by host against configurable rules.
pub use network_filter::NetworkFilter;
/// Verdict for a filtered network request: Allow or Block.
pub use network_filter::NetworkVerdict;
/// Watches `.hick` files, transparently reruns the pipeline, and applies reverse provenance.
pub use pipeline_watcher::PipelineWatcher;
/// Script-first ReAct loop: extracts code blocks from LLM responses and executes them in a sandbox.
pub use react_loop::ReactLoop;
/// Security configuration loaded from `.hick/agent-security.toml` (network, file access, sinks).
pub use security_config::AgentSecurityConfig;
/// Validates outbound API sink calls against capability tokens and sink definitions.
pub use sink_guard::SinkGuard;

// Re-export dependencies used by downstream binaries (main.rs)
pub use hick_policy;
