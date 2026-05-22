//! hick: universal AI coding agent.
//!
//! A CLI/TUI binary wiring [`hick_agent_sdk`] (SDK v2 engine),
//! [`hick_agent_script_first`] (script-first strategy via [`ScriptBlockIntentParser`]),
//! and [`hick_agent_hick_edit`] (hick pipeline integration) together.
//! Extension activation is driven by `.hick/agent.toml`.

/// WASM sandbox backend backed by `hick_literate::executor::ContainerExecutor`.
pub mod wasm_backend;

/// Local GGUF model inference client using llama.cpp (requires `local` feature).
#[cfg(feature = "local")]
pub mod llm_local;
#[cfg(feature = "local")]
pub use llm_local::LocalLlmClient;
