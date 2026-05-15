//! Re-export shim for `hick-agent-core`.
//!
//! Hick-XML edit types have moved to [`hick_agent_hick_edit`]. Command
//! execution types remain here while their API differs from [`hick_agent_sdk`].

pub mod command;

/// Trait for executing shell commands in a sandboxed environment.
pub use command::CommandExecutor;
/// Structured output (stdout/stderr and exit code) from a sandboxed command execution.
pub use command::CommandOutput;
/// Configuration for the WASM sandbox (images directory, workdir, capabilities).
pub use command::WasmSandboxConfig;
/// Executes commands inside Hick WASM Linux containers with persistent state.
pub use command::WasmSandboxExecutor;

/// Trait for Hick XML edit operations: run pipelines, query structure, compute and apply edits.
pub use hick_agent_hick_edit::HickEditFormat;
/// Concrete implementation of [`HickEditFormat`] backed by hick and hick-lens.
pub use hick_agent_hick_edit::HickEditService;
/// Output from a pipeline run, including generated files, provenance maps, and AST mappings.
pub use hick_agent_hick_edit::PipelineOutput;
/// A single structural query match with element metadata and provenance origins.
pub use hick_agent_hick_edit::QueryMatch;
/// A computed source edit (file, byte range, replacement content, confidence).
pub use hick_agent_hick_edit::SourceEdit;
