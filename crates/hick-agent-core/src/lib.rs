//! Reusable abstractions for AI coding agents.
//!
//! This crate provides clean separation between:
//! - **Sandboxed command execution** — Run shell commands in Hick WASM containers
//! - **Hick edit format** — Parse, run pipeline, query structure, compute and apply edits
//!
//! Other agents (CLI, MCP, IDE plugins) can use these same abstractions.

pub mod command;
pub mod edit_format;

/// Trait for executing shell commands in a sandboxed environment.
pub use command::CommandExecutor;
/// Structured output (stdout/stderr and exit code) from a sandboxed command execution.
pub use command::CommandOutput;
/// Configuration for the WASM sandbox (images directory, workdir, capabilities).
pub use command::WasmSandboxConfig;
/// Executes commands inside Hick WASM Linux containers with persistent state.
pub use command::WasmSandboxExecutor;

/// Trait for Hick XML edit operations: run pipelines, query structure, compute and apply edits.
pub use edit_format::HickEditFormat;
/// Concrete implementation of [`HickEditFormat`] backed by hick and hick-lens.
pub use edit_format::HickEditService;
/// Output from a pipeline run, including generated files, provenance maps, and AST mappings.
pub use edit_format::PipelineOutput;
/// A single structural query match with element metadata and provenance origins.
pub use edit_format::QueryMatch;
/// A computed source edit (file, byte range, replacement content, confidence).
pub use edit_format::SourceEdit;
