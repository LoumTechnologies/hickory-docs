<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-merge/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2beaf546dcbc"></a>
## `crates/hick-merge/src/lib.rs`

<a id="f-2beaf546dcbc-error"></a>
### `error`

Merge system for resolving conflicts between pipeline output and user edits.
Provides a `MergeStrategy` trait with trivial and LLM-based implementations,
plus a `MergeOrchestrator` that performs three-way merge using version store
snapshots as the base.

<storycode:copy id="f-2beaf546dcbc-error">
//! Merge system for resolving conflicts between pipeline output and user edits.
//!
//! Provides a `MergeStrategy` trait with trivial and LLM-based implementations,
//! plus a `MergeOrchestrator` that performs three-way merge using version store
//! snapshots as the base.

pub mod error;
</storycode:copy>
<a id="f-2beaf546dcbc-llm_backend"></a>
### `llm_backend`

<storycode:copy id="f-2beaf546dcbc-llm_backend">
pub mod llm_backend;
</storycode:copy>
<a id="f-2beaf546dcbc-merge_strategy"></a>
### `merge_strategy`

<storycode:copy id="f-2beaf546dcbc-merge_strategy">
pub mod merge_strategy;
</storycode:copy>
<a id="f-2beaf546dcbc-nway"></a>
### `nway`

The merged view's N-way alignment: which regions N sources agree on.

<storycode:copy id="f-2beaf546dcbc-nway">
/// The merged view's N-way alignment: which regions N sources agree on.
pub mod nway;
</storycode:copy>
<a id="f-2beaf546dcbc-orchestrator"></a>
### `orchestrator`

<storycode:copy id="f-2beaf546dcbc-orchestrator">
pub mod orchestrator;
</storycode:copy>
<a id="f-2beaf546dcbc-trivial_strategy"></a>
### `trivial_strategy`

<storycode:copy id="f-2beaf546dcbc-trivial_strategy">
pub mod trivial_strategy;

</storycode:copy>
<a id="f-2beaf546dcbc-use_declaration"></a>
### `use_declaration`

Error type for merge operations, including conflicts and LLM API failures.

<storycode:copy id="f-2beaf546dcbc-use_declaration">
/// Error type for merge operations, including conflicts and LLM API failures.
pub use error::MergeError;
</storycode:copy>
<a id="f-2beaf546dcbc-use_declaration-2"></a>
### `use_declaration`

[`MergeStrategy`] that delegates conflict resolution to an LLM API with optional caching.

<storycode:copy id="f-2beaf546dcbc-use_declaration-2">
/// [`MergeStrategy`] that delegates conflict resolution to an LLM API with optional caching.
pub use llm_backend::LlmMergeStrategy;
</storycode:copy>
<a id="f-2beaf546dcbc-use_declaration-3"></a>
### `use_declaration`

Trait for resolving three-way merge conflicts between base, generated, and edited versions.

<storycode:copy id="f-2beaf546dcbc-use_declaration-3">
/// Trait for resolving three-way merge conflicts between base, generated, and edited versions.
pub use merge_strategy::MergeStrategy;
</storycode:copy>
<a id="f-2beaf546dcbc-use_declaration-4"></a>
### `use_declaration`

The merged view: N symmetric sources, shared regions and variants.

<storycode:copy id="f-2beaf546dcbc-use_declaration-4">
/// The merged view: N symmetric sources, shared regions and variants.
pub use nway::{MergedView, Region, Route, Source, Write, merged_view, rebuild, writes_for_edit};
</storycode:copy>
<a id="f-2beaf546dcbc-use_declaration-5"></a>
### `use_declaration`

Three-way merge orchestrator that compares pipeline output against user edits using version store snapshots.

<storycode:copy id="f-2beaf546dcbc-use_declaration-5">
/// Three-way merge orchestrator that compares pipeline output against user edits using version store snapshots.
pub use orchestrator::{MergeOrchestrator, MergeResult};
</storycode:copy>
<a id="f-2beaf546dcbc-use_declaration-6"></a>
### `use_declaration`

Trivial strategies: fail on conflict, keep user edits, or take pipeline output.

<storycode:copy id="f-2beaf546dcbc-use_declaration-6">
/// Trivial strategies: fail on conflict, keep user edits, or take pipeline output.
pub use trivial_strategy::{FailOnConflict, KeepEdited, TakeGenerated};
</storycode:copy>

### Assemble `crates/hick-merge/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-merge/src/lib.rs"><storycode:paste select="#f-2beaf546dcbc-error"/><storycode:paste select="#f-2beaf546dcbc-llm_backend"/><storycode:paste select="#f-2beaf546dcbc-merge_strategy"/><storycode:paste select="#f-2beaf546dcbc-nway"/><storycode:paste select="#f-2beaf546dcbc-orchestrator"/><storycode:paste select="#f-2beaf546dcbc-trivial_strategy"/><storycode:paste select="#f-2beaf546dcbc-use_declaration"/><storycode:paste select="#f-2beaf546dcbc-use_declaration-2"/><storycode:paste select="#f-2beaf546dcbc-use_declaration-3"/><storycode:paste select="#f-2beaf546dcbc-use_declaration-4"/><storycode:paste select="#f-2beaf546dcbc-use_declaration-5"/><storycode:paste select="#f-2beaf546dcbc-use_declaration-6"/></storycode:file>

</storycode:doc>
