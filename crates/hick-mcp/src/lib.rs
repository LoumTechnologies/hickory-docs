//! MCP server exposing hick lens and exec operations for AI coding agents.
//!
//! Provides tools for querying generated output structure, mapping back to
//! `.hick` source positions, computing source edits, and running pipelines.

pub mod server;
pub mod state;
