//! Cell tiers: which gate a run is, and what a cell it will not execute does
//! to the rest of the verification.
//!
//! A document's cells are not all worth the same wait. `tier="push"` on a
//! cell says *this one is expensive and its answer moves slowly*; a run
//! started with `--tier commit` then refuses to execute it. That refusal is
//! not the same thing as a missing baseline: the document decided it, on
//! purpose, and every other output of the document is still compared. See
//! `docs/guarantees/verification/a-push-tier-cell-is-skipped-at-commit.md`.

use std::collections::{BTreeMap, BTreeSet};

use hick_exec::attrs::CellTier;

use crate::CellId;
use crate::PipelineResult;

/// Cells a run declined to execute because of their tier, and the first line
/// of each one's command — enough for a report to name it.
pub type SkippedCells = BTreeMap<CellId, String>;

/// The gate a run is: the most expensive tier it is willing to execute.
///
/// [`TierGate::Push`] is the default and admits everything, so a run with no
/// `--tier` behaves exactly as it always has. [`TierGate::Commit`] is the
/// pre-commit hook's gate: a `tier="push"` cell is answered from a recording
/// if one matches, and skipped otherwise.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TierGate {
    /// `--tier commit`: execute only `tier="commit"` cells.
    Commit,
    /// The default, and `--tier push`: execute every cell.
    #[default]
    Push,
}

impl TierGate {
    /// The tier this gate admits, as a [`CellTier`].
    pub fn admits(self) -> CellTier {
        match self {
            TierGate::Commit => CellTier::Commit,
            TierGate::Push => CellTier::Push,
        }
    }

    /// Whether this gate declines to execute a cell of tier `cell`.
    ///
    /// Declining is not the same as never consulting: a skipped cell is
    /// still answered from a recording whose key matches, which is the
    /// whole reason `--tier commit` verifies as much as it does.
    pub fn skips(self, cell: CellTier) -> bool {
        !cell.runs_at(self.admits())
    }

    /// The gate named on the command line.
    ///
    /// The error names both values, exactly as a bad `tier=` attribute does:
    /// a mistyped gate that silently ran everything would make a commit take
    /// the hour this flag exists to avoid, and one that silently ran nothing
    /// would be worse.
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "commit" => Ok(TierGate::Commit),
            "push" => Ok(TierGate::Push),
            other => Err(format!(
                "`--tier {other}` is not a tier. Use `--tier commit` (run only the cells a \
                 commit is worth waiting for; a tier=\"push\" cell is answered from its \
                 recording or skipped) or `--tier push` (run everything, which is what \
                 omitting --tier does)."
            )),
        }
    }
}

impl std::fmt::Display for TierGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TierGate::Commit => "commit",
            TierGate::Push => "push",
        })
    }
}

impl PipelineResult {
    /// Output paths that carry bytes from a cell with no baseline.
    ///
    /// A weave never executes: a cell whose recording it cannot find is woven
    /// as `[never run]`, and every file that cell fed then *says* the cell
    /// never ran. As a statement about this weave that is honest; as bytes
    /// written over a committed artifact it is destruction — the SVG a real
    /// run produced is replaced by a marker, and the app, which executes,
    /// goes on rendering the chart. Disk and app disagree, silently.
    ///
    /// So the writer needs to know which files those are, and provenance
    /// already knows: a span whose origin is the exec cell named in
    /// [`PipelineResult::never_run`] is exactly a byte the cell contributed.
    ///
    /// **Agent cells are not covered.** `SourceOrigin::Agent` carries the
    /// session and turn rather than a source line, so an agent cell's bytes
    /// cannot be matched back to its [`CellId`]. Agent cells write into
    /// documents rather than into `hick:file` products, so this has no
    /// bearing on the artifact case; it is stated because the omission is
    /// deliberate rather than overlooked.
    pub fn outputs_missing_a_recording(&self) -> BTreeSet<String> {
        self.outputs_fed_by(&self.never_run.keys().cloned().collect())
    }

    /// Output paths that carry bytes from a cell this run skipped for its
    /// tier.
    ///
    /// The same reasoning as [`PipelineResult::outputs_missing_a_recording`],
    /// for the other reason a cell can contribute nothing. These outputs are
    /// neither written over nor compared for drift: the run knows it did not
    /// produce them, and manufacturing a diff out of a cell the document
    /// asked it not to run would be a lie about the document.
    pub fn outputs_from_skipped_cells(&self) -> BTreeSet<String> {
        self.outputs_fed_by(&self.skipped.keys().cloned().collect())
    }

    /// Output paths any of `cells` contributed bytes to, by provenance.
    fn outputs_fed_by(&self, cells: &BTreeSet<CellId>) -> BTreeSet<String> {
        if cells.is_empty() {
            return BTreeSet::new();
        }
        let mut affected = BTreeSet::new();
        for (path, map) in &self.provenance_maps {
            let fed = map.spans().iter().any(|span| match &span.origin {
                hick_exec::node::SourceOrigin::Exec {
                    container,
                    tag_line,
                } => cells.contains(&CellId::exec(container.as_ref(), *tag_line)),
                hick_exec::node::SourceOrigin::Script { tag_line } => {
                    cells.contains(&CellId::containerless(*tag_line))
                }
                _ => false,
            });
            if fed {
                affected.insert(path.clone());
            }
        }
        affected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_push_gate_runs_everything() {
        assert!(!TierGate::Push.skips(CellTier::Commit));
        assert!(!TierGate::Push.skips(CellTier::Push));
        assert_eq!(
            TierGate::default(),
            TierGate::Push,
            "a run that says nothing about tiers must behave exactly as it \
             did before tiers existed"
        );
    }

    #[test]
    fn the_commit_gate_skips_only_push_cells() {
        assert!(!TierGate::Commit.skips(CellTier::Commit));
        assert!(TierGate::Commit.skips(CellTier::Push));
    }

    #[test]
    fn an_unknown_gate_is_refused_naming_both() {
        let err = TierGate::parse("pre-push").expect_err("only two gates exist");
        assert!(
            err.contains("--tier commit") && err.contains("--tier push"),
            "{err}"
        );
        assert_eq!(TierGate::parse("COMMIT").unwrap(), TierGate::Commit);
        assert_eq!(TierGate::parse(" push ").unwrap(), TierGate::Push);
    }
}
