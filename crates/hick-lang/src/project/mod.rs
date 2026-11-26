mod comparison;
mod discovery;
mod edit;
mod evaluate;
mod patch;
mod reconstruct;
mod tree;

pub use comparison::{ComparisonSource, EditPolicy, ProjectComparison, RevisionIdentity};
pub use evaluate::{EvaluatedProject, InspectedProjectPatch, ProjectInventory, evaluate_project, inspect_project_patch};
pub use patch::{PatchResult, PatchTarget, apply_project_patch, project_patch_targets};
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
pub use snapshot::{ProjectSnapshot, ProjectSnapshotDelta};

mod recorded;
pub use recorded::RecordedCell;

mod coordinates;
pub use coordinates::project_byte_location;

mod debug;
pub use debug::latest_debug_project;
