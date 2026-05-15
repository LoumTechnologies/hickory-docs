//! Hick-XML-aware agent extension crate.
//!
//! Consolidates all agent behaviour specific to the Hickory XML pipeline
//! format: edit format parsing, pipeline watching, reverse provenance, file
//! I/O, and high-level composition helpers.
//!
//! Extends [`hick_agent_sdk`] without polluting the generic SDK with
//! Hickory-specific knowledge.
//!
//! # Key types
//!
//! | Type | Purpose |
//! |------|---------|
//! | [`HickEditAgent`] | Composable hick-edit behaviour layered on `AgentRuntime` |
//! | [`HickEditFormat`] | Trait for hick XML edit operations |
//! | [`HickEditService`] | Concrete edit service backed by hick and hick-lens |
//! | [`PipelineWatcher`] | Watches `.hick` files and transparently reruns the pipeline |
//! | [`PasteSlotRegistry`] | Index of named paste slots in the current pipeline |

pub mod edit_format;
pub mod file_io;
pub mod hick_edit_agent;
pub mod paste_slot_registry;
pub mod pipeline_watcher;
pub mod reverse_provenance;

// ---------------------------------------------------------------------------
// Edit format
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Pipeline watcher
// ---------------------------------------------------------------------------

/// Watches `.hick` files, transparently reruns the pipeline, and applies
/// reverse provenance when output files are edited directly.
pub use pipeline_watcher::PipelineWatcher;
/// Metadata written to `.hick/pipeline.state` after each successful run.
pub use pipeline_watcher::PipelineState;

// ---------------------------------------------------------------------------
// Paste slot registry
// ---------------------------------------------------------------------------

/// Indexes named `<hick:paste>` slots so the agent can discover composition
/// points without scanning every source file on each request.
pub use paste_slot_registry::PasteSlotRegistry;

// ---------------------------------------------------------------------------
// Hick edit agent
// ---------------------------------------------------------------------------

/// Composable hick-XML edit behaviour layered on top of [`hick_agent_sdk::AgentRuntime`].
pub use hick_edit_agent::HickEditAgent;
