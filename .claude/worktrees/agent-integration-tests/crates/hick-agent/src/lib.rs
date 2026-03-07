//! AI command-line coding agent.
//!
//! Uses a script-first model: the LLM writes code in fenced blocks
//! (Python, TypeScript, or C#), which are executed and their output
//! returned as observations. The Hick pipeline runs transparently via
//! the PipelineWatcher, and structural operations are available to
//! Python scripts via the `hickory` SDK.

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
pub mod pipeline_watcher;
pub mod react_loop;
pub mod reverse_provenance;
pub mod state;
pub mod tools;
pub mod tui;

pub use agent::{Agent, AgentConfig};
pub use events::{AgentCommand, AgentEvent};
pub use llm::{ChatChunk, LlmClient, Message, Role};
pub use llm_anthropic::AnthropicClient;
#[cfg(feature = "local")]
pub use llm_local::LocalLlmClient;
pub use llm_openai_compat::OpenAiCompatClient;
pub use pipeline_watcher::PipelineWatcher;
pub use react_loop::ReactLoop;
pub use tools::{CodeBlock, Language, ScriptResult};
