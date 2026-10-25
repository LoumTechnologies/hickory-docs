<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/mod.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `eb652ab5c8bd6a9677a5be882322c6471495283b8a85ae8b70e52f294092c246`.

<book:entry path="crates/hick-lang/src/project/mod.rs">
mod comparison;
mod discovery;
mod edit;
mod evaluate;
mod patch;
mod reconstruct;
mod tree;

pub use comparison::{ComparisonSource, EditPolicy, ProjectComparison, RevisionIdentity};
pub use evaluate::{EvaluatedProject, ProjectInventory, evaluate_project};
pub use patch::{PatchResult, apply_project_patch};
pub use tree::{
    ByteOrigin, DifferenceKind, EDITOR_BUFFER_PREFIX, EntryKind, ProjectDifference, ProjectEntry,
    ProjectTree, validate_project_path,
};

pub use reconstruct::{
    AgentProjectCapture, IncorporationCandidate, IncorporationError, transition_response,
    unified_transition,
};

pub use discovery::load_project_sources;

pub use edit::{regenerate_patch, reverse_project_edit};

mod snapshot;
pub use snapshot::ProjectSnapshot;

mod recorded;
pub use recorded::RecordedCell;

mod coordinates;
pub use coordinates::project_byte_location;

mod debug;
pub use debug::latest_debug_project;
</book:entry>

</book:doc>
