//! AI command-line coding agent.
//!
//! This crate is a thin CLI/TUI wrapper around [`hick_agent_engine`], which
//! contains the core agent logic (ReAct loop, LLM abstraction, security stack).
//!
//! All engine modules are re-exported for backward compatibility.

// Re-export all engine modules
/// Agent core: LLM client wrapper and project configuration.
pub use hick_agent_engine::agent;
/// Backing store utilities for agent state persistence.
pub use hick_agent_engine::backing;
/// Context assembly for LLM prompts.
pub use hick_agent_engine::context;
/// Event types for agent-to-UI communication.
pub use hick_agent_engine::events;
/// File I/O helpers for loading and writing `.hick` sources.
pub use hick_agent_engine::file_io;
/// LLM client trait and message types.
pub use hick_agent_engine::llm;
/// Anthropic Messages API client.
pub use hick_agent_engine::llm_anthropic;
/// Local GGUF model inference (requires `local` feature).
#[cfg(feature = "local")]
pub use hick_agent_engine::llm_local;
/// OpenAI-compatible chat completion client.
pub use hick_agent_engine::llm_openai_compat;
/// Network request filtering and approval prompts.
pub use hick_agent_engine::network_filter;
/// Transparent pipeline watcher with reverse provenance.
pub use hick_agent_engine::pipeline_watcher;
/// Script-first ReAct loop for code extraction and sandbox execution.
pub use hick_agent_engine::react_loop;
/// Reverse provenance: map output edits back to `.hick` sources.
pub use hick_agent_engine::reverse_provenance;
/// Agent security configuration (network, file access, sinks).
pub use hick_agent_engine::security_config;
/// API sink guard for validating outbound calls from agent scripts.
pub use hick_agent_engine::sink_guard;
/// Agent session state tracking.
pub use hick_agent_engine::state;
/// Code block extraction and script execution tools.
pub use hick_agent_engine::tools;

// Re-export key types at crate root (backward compatibility)
/// AI coding agent wrapping an LLM client and project configuration.
///
/// Re-exported from [`hick_agent_engine::Agent`] for backward compatibility.
pub use hick_agent_engine::Agent;
/// Commands sent from the UI to the agent core (user messages, quit).
///
/// Re-exported from [`hick_agent_engine::AgentCommand`] for backward compatibility.
pub use hick_agent_engine::AgentCommand;
/// Configuration for the coding agent (project directory, images directory, default image).
///
/// Re-exported from [`hick_agent_engine::AgentConfig`] for backward compatibility.
pub use hick_agent_engine::AgentConfig;
/// Events emitted by the agent core to the UI (tokens, script results, errors).
///
/// Re-exported from [`hick_agent_engine::AgentEvent`] for backward compatibility.
pub use hick_agent_engine::AgentEvent;
/// Security configuration loaded from `.hick/agent-security.toml` (network, file access, sinks).
///
/// Re-exported from [`hick_agent_engine::AgentSecurityConfig`] for backward compatibility.
pub use hick_agent_engine::AgentSecurityConfig;
/// Anthropic Messages API client with streaming support.
///
/// Re-exported from [`hick_agent_engine::AnthropicClient`] for backward compatibility.
pub use hick_agent_engine::AnthropicClient;
/// A chunk of streamed LLM output containing a text delta and optional finish reason.
///
/// Re-exported from [`hick_agent_engine::ChatChunk`] for backward compatibility.
pub use hick_agent_engine::ChatChunk;
/// LLM client wrapper that checks a [`NetworkFilter`] before each API call.
///
/// Re-exported from [`hick_agent_engine::FilteredLlmClient`] for backward compatibility.
pub use hick_agent_engine::FilteredLlmClient;
/// Trait for LLM chat completion clients (streaming and non-streaming).
///
/// Re-exported from [`hick_agent_engine::LlmClient`] for backward compatibility.
pub use hick_agent_engine::LlmClient;
/// Request sent to the UI to prompt the user for network access approval.
///
/// Re-exported from [`hick_agent_engine::NetworkApprovalRequest`] for backward compatibility.
pub use hick_agent_engine::NetworkApprovalRequest;
/// Filters outbound network requests by host against configurable rules.
///
/// Re-exported from [`hick_agent_engine::NetworkFilter`] for backward compatibility.
pub use hick_agent_engine::NetworkFilter;
/// Verdict for a filtered network request: Allow or Block.
///
/// Re-exported from [`hick_agent_engine::NetworkVerdict`] for backward compatibility.
pub use hick_agent_engine::NetworkVerdict;
/// OpenAI-compatible chat completion client (works with OpenAI, xAI, Groq, Ollama, etc.).
///
/// Re-exported from [`hick_agent_engine::OpenAiCompatClient`] for backward compatibility.
pub use hick_agent_engine::OpenAiCompatClient;
/// Watches `.hick` files, transparently reruns the pipeline, and applies reverse provenance.
///
/// Re-exported from [`hick_agent_engine::PipelineWatcher`] for backward compatibility.
pub use hick_agent_engine::PipelineWatcher;
/// Script-first ReAct loop: extracts code blocks from LLM responses and executes them in a sandbox.
///
/// Re-exported from [`hick_agent_engine::ReactLoop`] for backward compatibility.
pub use hick_agent_engine::ReactLoop;
/// Validates outbound API sink calls against capability tokens and sink definitions.
///
/// Re-exported from [`hick_agent_engine::SinkGuard`] for backward compatibility.
pub use hick_agent_engine::SinkGuard;
/// Local GGUF model inference client using llama.cpp (requires `local` feature).
///
/// Re-exported from [`hick_agent_engine::LocalLlmClient`] for backward compatibility.
#[cfg(feature = "local")]
pub use hick_agent_engine::LocalLlmClient;

// TUI module — unique to the CLI binary
pub mod tui;
