//! AI command-line coding agent.
//!
//! Uses Hick XML as the edit format and Hick WASM containers for sandboxed
//! command execution. All operations run in-process except for third-party
//! MCP servers (if configured).
//!
//! The agent is cleanly separated from:
//! - [`hick_agent_core::CommandExecutor`] — Sandboxed command execution
//! - [`hick_agent_core::HickEditFormat`] — Hick XML pipeline and edit operations

pub mod agent;
pub mod backing;
pub mod context;
pub mod file_io;
pub mod llm;
pub mod llm_anthropic;
pub mod llm_openai_compat;
pub mod pipeline_watcher;
pub mod react_loop;
pub mod state;
pub mod tools;

pub use agent::{Agent, AgentConfig};
pub use llm::{ChatChunk, LlmClient};
pub use llm_anthropic::AnthropicClient;
pub use llm_openai_compat::OpenAiCompatClient;
pub use pipeline_watcher::PipelineWatcher;
pub use react_loop::ReactLoop;
