<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lang/src/project/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-134665e92482"></a>
## `crates/hick-lang/src/project/mod.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

<a id="f-134665e92482-comparison"></a>
### `comparison`

<storycode:copy id="f-134665e92482-comparison">
mod comparison;
</storycode:copy>
<a id="f-134665e92482-discovery"></a>
### `discovery`

<storycode:copy id="f-134665e92482-discovery">
mod discovery;
</storycode:copy>
<a id="f-134665e92482-edit"></a>
### `edit`

<storycode:copy id="f-134665e92482-edit">
mod edit;
</storycode:copy>
<a id="f-134665e92482-evaluate"></a>
### `evaluate`

<storycode:copy id="f-134665e92482-evaluate">
mod evaluate;
</storycode:copy>
<a id="f-134665e92482-patch"></a>
### `patch`

<storycode:copy id="f-134665e92482-patch">
mod patch;
</storycode:copy>
<a id="f-134665e92482-reconstruct"></a>
### `reconstruct`

<storycode:copy id="f-134665e92482-reconstruct">
mod reconstruct;
</storycode:copy>
<a id="f-134665e92482-tree"></a>
### `tree`

<storycode:copy id="f-134665e92482-tree">
mod tree;

</storycode:copy>
<a id="f-134665e92482-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration">
pub use comparison::{ComparisonSource, EditPolicy, ProjectComparison, RevisionIdentity};
</storycode:copy>
<a id="f-134665e92482-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-2">
pub use evaluate::{EvaluatedProject, ProjectInventory, evaluate_project};
</storycode:copy>
<a id="f-134665e92482-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-3">
pub use patch::{PatchResult, apply_project_patch};
</storycode:copy>
<a id="f-134665e92482-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-4">
pub use tree::{
    ByteOrigin, DifferenceKind, EDITOR_BUFFER_PREFIX, EntryKind, ProjectDifference, ProjectEntry,
    ProjectTree, validate_project_path,
};

</storycode:copy>
<a id="f-134665e92482-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-5">
pub use reconstruct::{
    AgentProjectCapture, IncorporationCandidate, IncorporationError, transition_response,
    unified_transition,
};

</storycode:copy>
<a id="f-134665e92482-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-6">
pub use discovery::load_project_sources;

</storycode:copy>
<a id="f-134665e92482-use_declaration-7"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-7">
pub use edit::{regenerate_patch, reverse_project_edit};

</storycode:copy>
<a id="f-134665e92482-snapshot"></a>
### `snapshot`

<storycode:copy id="f-134665e92482-snapshot">
mod snapshot;
</storycode:copy>
<a id="f-134665e92482-use_declaration-8"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-8">
pub use snapshot::ProjectSnapshot;

</storycode:copy>
<a id="f-134665e92482-recorded"></a>
### `recorded`

<storycode:copy id="f-134665e92482-recorded">
mod recorded;
</storycode:copy>
<a id="f-134665e92482-use_declaration-9"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-9">
pub use recorded::RecordedCell;

</storycode:copy>
<a id="f-134665e92482-coordinates"></a>
### `coordinates`

<storycode:copy id="f-134665e92482-coordinates">
mod coordinates;
</storycode:copy>
<a id="f-134665e92482-use_declaration-10"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-10">
pub use coordinates::project_byte_location;

</storycode:copy>
<a id="f-134665e92482-debug"></a>
### `debug`

<storycode:copy id="f-134665e92482-debug">
mod debug;
</storycode:copy>
<a id="f-134665e92482-use_declaration-11"></a>
### `use_declaration`

<storycode:copy id="f-134665e92482-use_declaration-11">
pub use debug::latest_debug_project;
</storycode:copy>

### Assemble `crates/hick-lang/src/project/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/mod.rs"><storycode:paste select="#f-134665e92482-comparison"/><storycode:paste select="#f-134665e92482-discovery"/><storycode:paste select="#f-134665e92482-edit"/><storycode:paste select="#f-134665e92482-evaluate"/><storycode:paste select="#f-134665e92482-patch"/><storycode:paste select="#f-134665e92482-reconstruct"/><storycode:paste select="#f-134665e92482-tree"/><storycode:paste select="#f-134665e92482-use_declaration"/><storycode:paste select="#f-134665e92482-use_declaration-2"/><storycode:paste select="#f-134665e92482-use_declaration-3"/><storycode:paste select="#f-134665e92482-use_declaration-4"/><storycode:paste select="#f-134665e92482-use_declaration-5"/><storycode:paste select="#f-134665e92482-use_declaration-6"/><storycode:paste select="#f-134665e92482-use_declaration-7"/><storycode:paste select="#f-134665e92482-snapshot"/><storycode:paste select="#f-134665e92482-use_declaration-8"/><storycode:paste select="#f-134665e92482-recorded"/><storycode:paste select="#f-134665e92482-use_declaration-9"/><storycode:paste select="#f-134665e92482-coordinates"/><storycode:paste select="#f-134665e92482-use_declaration-10"/><storycode:paste select="#f-134665e92482-debug"/><storycode:paste select="#f-134665e92482-use_declaration-11"/></storycode:file>

</storycode:doc>
