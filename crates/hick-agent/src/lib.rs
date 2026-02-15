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
pub mod llm;
pub mod llm_openai;
pub mod llm_xai;

pub use agent::{Agent, AgentConfig};
pub use llm::LlmClient;
pub use llm_openai::OpenAiClient;
pub use llm_xai::XaiClient;
