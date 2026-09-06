//! What the CLI does about a cell a run skipped for its tier.
//!
//! The pipeline decides which cells to skip (`hick_literate::tier`). Two
//! things follow from a skip, and both live here because both are about the
//! *files*: the outputs that cell fed are not compared for drift and are not
//! written over, and the skip is reported so nobody mistakes a fast green
//! commit for a full verification.
//!
//! Guarantee:
//! `docs/guarantees/verification/a-push-tier-cell-is-skipped-at-commit.md`.

use std::collections::BTreeSet;

use crate::DocRun;

/// Outputs of this document that a skipped cell makes meaningless to compare
/// or to write.
///
/// Two sources, and the second is not derivable from the first:
///
/// - **Provenance**, for a `hick:file` product the cell fed: those bytes came
///   out of a cell that did not run, so the fresh copy is a marker and the
///   committed copy is the real thing.
/// - **The weave target**, whenever anything was skipped at all. The woven
///   markdown embeds every cell's transcript, and a transcript that is absent
///   because the cell was skipped is not tracked as provenance the way a
///   file's bytes are. It is the one output that always depends on every
///   cell, so it is named directly rather than inferred.
///
/// Every OTHER output of the document is compared and written exactly as it
/// always was — which is the point of tiers: skipping the expensive cell does
/// not stop the document's tangle from being verified.
pub fn outputs_not_compared(run: &DocRun) -> BTreeSet<String> {
    if run.result.skipped.is_empty() {
        return BTreeSet::new();
    }
    let mut paths = run.result.outputs_from_skipped_cells();
    if let Some(weave) = run.doc.weave_path.as_deref() {
        paths.insert(weave.to_string());
    }
    paths
}

/// Print one line per skipped cell, and return how many there were.
///
/// Named, with the document and the line, for the same reason the
/// unverifiable report names its cell: a skip is a thing the reader may want
/// to go and look at, and "3 cells were skipped" sends them nowhere. The line
/// also names the gate that WOULD run it, because a reader seeing this in a
/// blocked commit needs to know the cell is not unchecked forever.
pub fn report_skips(run: &DocRun) -> usize {
    for (cell, command) in &run.result.skipped {
        eprintln!(
            "SKIP {} {cell}: tier=\"push\", and no recording matches its inputs, so it was \
             not run at the commit tier — its expectations and the woven page were not \
             checked.\n  cell: {command}\n  runs in: hick test {} (no --tier), the pre-push \
             and CI gate.",
            run.doc_path.display(),
            run.doc_path.display(),
        );
    }
    run.result.skipped.len()
}

/// The closing line of a run that skipped cells, or nothing when it skipped
/// none.
///
/// Deliberately not an outcome and not an exit code: a skip at the commit
/// tier is a decision the document made, so the run is still *verified* for
/// everything it did check. Saying so in one sentence, with the command that
/// checks the rest, is the whole report.
pub fn skip_summary(count: usize) -> Option<String> {
    (count > 0).then(|| match count {
        1 => "hick test: 1 cell tiered push was skipped at the commit tier — its expectations \
              and its document's woven page were not checked here. `hick test .` (the \
              pre-push and CI gate) runs it."
            .to_string(),
        n => format!(
            "hick test: {n} cells tiered push were skipped at the commit tier — their \
             expectations and their documents' woven pages were not checked here. \
             `hick test .` (the pre-push and CI gate) runs them."
        ),
    })
}
