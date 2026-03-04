//! AI command-line coding agent.
//!
//! This crate is a thin CLI/TUI wrapper around [`hick_agent_engine`], which
//! contains the core agent logic (ReAct loop, LLM abstraction, security stack).
//!
//! All engine modules are re-exported for backward compatibility.

// Re-export all engine modules
pub use hick_agent_engine::agent;
pub use hick_agent_engine::backing;
pub use hick_agent_engine::context;
pub use hick_agent_engine::events;
pub use hick_agent_engine::file_io;
pub use hick_agent_engine::llm;
pub use hick_agent_engine::llm_anthropic;
#[cfg(feature = "local")]
pub use hick_agent_engine::llm_local;
pub use hick_agent_engine::llm_openai_compat;
pub use hick_agent_engine::network_filter;
pub use hick_agent_engine::pipeline_watcher;
pub use hick_agent_engine::react_loop;
pub use hick_agent_engine::reverse_provenance;
pub use hick_agent_engine::security_config;
pub use hick_agent_engine::sink_guard;
pub use hick_agent_engine::state;
pub use hick_agent_engine::tools;

// Re-export key types at crate root (backward compatibility)
pub use hick_agent_engine::{
    Agent, AgentCommand, AgentConfig, AgentEvent, AgentSecurityConfig, AnthropicClient, ChatChunk,
    FilteredLlmClient, LlmClient, NetworkApprovalRequest, NetworkFilter, NetworkVerdict,
    OpenAiCompatClient, PipelineWatcher, ReactLoop, SinkGuard,
};
#[cfg(feature = "local")]
pub use hick_agent_engine::LocalLlmClient;

// TUI module — unique to the CLI binary
pub mod tui;
