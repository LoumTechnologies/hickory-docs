//! AI command-line coding agent.
//!
//! Uses a script-first model: the LLM writes Python scripts in code blocks,
//! which are executed and their output returned as observations. The Hick
//! pipeline runs transparently via the PipelineWatcher, and structural
//! operations are available to scripts via the `hickory` Python SDK.

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
