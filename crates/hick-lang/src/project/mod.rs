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
    ByteOrigin, DifferenceKind, EntryKind, ProjectDifference, ProjectEntry, ProjectTree,
    validate_project_path,
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
