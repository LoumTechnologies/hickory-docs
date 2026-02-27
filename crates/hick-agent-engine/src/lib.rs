//! Embeddable agent engine — ReAct loop, LLM abstraction, security stack.
//!
//! This crate contains the core logic of the AI coding agent, extracted from
//! `hick-agent` so it can be embedded in other frontends (web, native apps,
//! headless runners) without pulling in TUI/CLI dependencies.
//!
//! The agent uses a script-first model: the LLM writes code in fenced markdown
//! blocks, which are executed in a sandbox, with output returned as observations.
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

pub use agent::{Agent, AgentConfig};
pub use events::{AgentCommand, AgentEvent};
pub use llm::{ChatChunk, LlmClient, Message, Role};
pub use llm_anthropic::AnthropicClient;
#[cfg(feature = "local")]
pub use llm_local::LocalLlmClient;
pub use llm_openai_compat::OpenAiCompatClient;
pub use network_filter::{FilteredLlmClient, NetworkApprovalRequest, NetworkFilter, NetworkVerdict};
pub use pipeline_watcher::PipelineWatcher;
pub use react_loop::ReactLoop;
pub use security_config::AgentSecurityConfig;
pub use sink_guard::SinkGuard;

// Re-export dependencies used by downstream binaries (main.rs)
pub use hick_policy;
