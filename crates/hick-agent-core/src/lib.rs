//! Reusable abstractions for AI coding agents.
//!
//! This crate provides clean separation between:
//! - **Sandboxed command execution** — Run shell commands in Hick WASM containers
//! - **Hick edit format** — Parse, run pipeline, query structure, compute and apply edits
//!
//! Other agents (CLI, MCP, IDE plugins) can use these same abstractions.

pub mod command;
pub mod edit_format;

pub use command::{CommandExecutor, WasmSandboxConfig, WasmSandboxExecutor};
pub use edit_format::{HickEditFormat, HickEditService, QueryMatch, SourceEdit};
