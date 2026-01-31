//! Merge system for resolving conflicts between pipeline output and user edits.
//!
//! Provides a `MergeStrategy` trait with trivial and LLM-based implementations,
//! plus a `MergeOrchestrator` that performs three-way merge using version store
//! snapshots as the base.

pub mod error;
pub mod llm_backend;
pub mod merge_strategy;
pub mod orchestrator;
pub mod trivial_strategy;

pub use error::MergeError;
pub use llm_backend::LlmMergeStrategy;
pub use merge_strategy::MergeStrategy;
pub use orchestrator::{MergeOrchestrator, MergeResult};
pub use trivial_strategy::{FailOnConflict, KeepEdited, TakeGenerated};
