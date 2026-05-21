//! Append-only eval results store: per-machine NDJSON files with git sync.
//!
//! Each machine appends exclusively to its own file named `<hostname>_<uuid>.ndjson`,
//! keyed by a stable UUID written once to `~/.hick/machine-id`. Storage lives under
//! `.hick/eval/` in the git repo root, falling back to `~/.hick/eval/` outside a repo.
//!
//! Because no two machines share a file, git merges are always conflict-free.
//! [`GitBackend::read_all`] aggregates all machine files at query time.
//! Pruning is recorded as [`EvalRecord::Prune`] entries appended on top of existing
//! lines — no line is ever deleted.

mod backend;
mod git;
mod machine_id;
mod noop;
mod path;
mod pruning;
mod record;

/// Abstraction over storage transport.
pub use backend::StorageBackend;
/// Git-backed real implementation of [`StorageBackend`].
pub use git::GitBackend;
/// Stable per-machine identity stored in `~/.hick/machine-id`.
pub use machine_id::MachineId;
/// In-memory no-op backend for tests.
pub use noop::NoopBackend;
/// Resolve the directory where `.ndjson` files are stored.
pub use path::resolve_store_dir;
/// Pruning analytics: config ranking, Wilson CI, is_pruned predicate.
pub use pruning::{auto_prune_candidates, compute_config_stats, is_pruned, wilson_ci, ConfigStats};
/// NDJSON record types.
pub use record::{
    BacktrackingEvent, EvalRecord, EvalResultRecord, HumanRatingRecord, JudgeScoreRecord,
    PruneRecord, TokenCost, UnpruneRecord,
};
