//! Cell identity, and why a cell has no baseline.
//!
//! Split out of `lib.rs`, which is the pipeline: these two types are what
//! every report about a cell is keyed on, and they are read far more widely
//! than the pipeline that fills them in.

use hick_exec::dag;

// ---------------------------------------------------------------------------
// Cell identity and missing baselines
// ---------------------------------------------------------------------------

/// Identity of one cell in a document.
///
/// An `<hick:exec>` cell names the container it runs in. A cell with **no**
/// container — the agent cell described in
/// `docs/specs/freeform/agent-cells.md` — leaves `container` `None`. That is
/// why this is a struct rather than the `(container, source_line)` tuple it
/// replaces: a key that assumes a container cannot name a cell that has none.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellId {
    /// The container this cell executes in, when it has one.
    pub container: Option<String>,
    /// Line of the cell's opening tag in its source document.
    pub source_line: usize,
}

impl CellId {
    /// A cell that runs inside a container (`<hick:exec container="…">`).
    pub fn exec(container: impl Into<String>, source_line: usize) -> Self {
        Self {
            container: Some(container.into()),
            source_line,
        }
    }

    /// A cell with no container of its own.
    pub fn containerless(source_line: usize) -> Self {
        Self {
            container: None,
            source_line,
        }
    }

    /// The container name, when this cell has one.
    pub fn container(&self) -> Option<&str> {
        self.container.as_deref()
    }
}

impl std::fmt::Display for CellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.container {
            Some(c) => write!(f, "line {} (container '{c}')", self.source_line),
            None => write!(f, "line {}", self.source_line),
        }
    }
}

/// Why a cell has no baseline to be verified against.
///
/// This is the *unverifiable* half of `check`'s verdict: nothing was ever
/// established for the cell, which is a different fact from "what was
/// established has since changed" (drift). See
/// `docs/guarantees/verification/test-separates-unverifiable-from-drifted.md`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NoBaseline {
    /// Weave or dry-run: nothing is executed, and no recording answered this
    /// cell.
    NotExecuted,
    /// The cell is frozen and no recording exists for it, in a mode that may
    /// not establish one (`hick test`). `frozen_by_cell` distinguishes
    /// `freeze="true"` on the cell itself from a run-wide freeze the cell
    /// merely inherited.
    FrozenWithoutRecording {
        /// First non-empty command line, so a report can name the cell.
        command: String,
        frozen_by_cell: bool,
    },
    /// The cell declares `freeze="true"`, but this run has no cache directory
    /// at all — there is nowhere for a recording to live.
    FrozenWithoutCacheDirectory { command: String },
    /// The cell is a `<hick:agent>` cell, no recording answered it, and this
    /// run has no agent runner configured — no model credentials, so nothing
    /// could have run it.
    ///
    /// This is the ordinary state of CI and of any machine without an API key,
    /// and it is deliberately *unverifiable* rather than fatal: the rest of
    /// the document still runs, and the one cell nothing was established for
    /// is named. `model_declared` records whether the cell says which model it
    /// wants, because a cell that does can be replayed from a recording with
    /// no runner at all, and a cell that does not never can.
    AgentWithoutRunner {
        prompt: String,
        model_declared: bool,
    },
}

/// Cells with no baseline, keyed by cell.
pub type NeverRun = std::collections::BTreeMap<CellId, NoBaseline>;

/// The [`CellId`] of a DAG vertex.
///
/// An agent cell is containerless — that is what `CellId::container` being an
/// `Option` is for — even though `ExecInfo` gives it a reserved synthetic
/// container name so the container-keyed plumbing (recording directory,
/// transcript map) stays addressable.
pub(crate) fn cell_id_of(info: &dag::ExecInfo) -> CellId {
    if info.is_agent() {
        CellId::containerless(info.source_line)
    } else {
        CellId::exec(&info.container, info.source_line)
    }
}

/// The first non-empty line of a command, for naming a cell in a report.
pub(crate) fn first_command_line(command: &str) -> String {
    command
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("?")
        .to_string()
}
