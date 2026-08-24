//! Merge system for resolving conflicts between pipeline output and user edits.
//!
//! Provides a `MergeStrategy` trait with trivial and LLM-based implementations,
//! plus a `MergeOrchestrator` that performs three-way merge using version store
//! snapshots as the base.

pub mod error;
pub mod llm_backend;
pub mod merge_strategy;
/// The merged view's N-way alignment: which regions N sources agree on.
pub mod nway;
pub mod orchestrator;
pub mod trivial_strategy;

/// Error type for merge operations, including conflicts and LLM API failures.
pub use error::MergeError;
/// [`MergeStrategy`] that delegates conflict resolution to an LLM API with optional caching.
pub use llm_backend::LlmMergeStrategy;
/// Trait for resolving three-way merge conflicts between base, generated, and edited versions.
pub use merge_strategy::MergeStrategy;
/// The merged view: N symmetric sources, shared regions and variants.
pub use nway::{MergedView, Region, Route, Source, Write, merged_view, rebuild, writes_for_edit};
/// Three-way merge orchestrator that compares pipeline output against user edits using version store snapshots.
pub use orchestrator::{MergeOrchestrator, MergeResult};
/// Trivial strategies: fail on conflict, keep user edits, or take pipeline output.
pub use trivial_strategy::{FailOnConflict, KeepEdited, TakeGenerated};
