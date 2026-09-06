//! Everything a run says before it executes anything.
//!
//! Emitted here rather than at the end because the failures these predict
//! kill the run, and a warning printed afterwards is a warning nobody sees.
//! Each function that finds them lives beside what it knows about; this
//! module is only the order they are said in.

use std::path::Path;

use hick_lang::HickDocument;

use crate::{
    absolute_mount_warnings, claim_warnings, diagram_assertion_warnings, escaping_warnings,
    output_collision_warnings,
};

pub(crate) fn warn_before_run(doc_path: &Path, source: &str, doc: &HickDocument) {
    for warning in escaping_warnings(doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // Not gated on the executor: both resolve mounts under the workdir.
    for warning in absolute_mount_warnings(doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // A cell that mounts its own output silently loses every recording it
    // makes, and the symptom (`[never run]` over a real run) points nowhere
    // near the cause.
    // A silent clobber or an unrelated exit code, either far from the
    // second declaration that actually caused it.
    for warning in output_collision_warnings(doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // A drawing nobody checks is the thing this feature exists to prevent.
    for warning in claim_warnings(doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    for warning in diagram_assertion_warnings(doc) {
        log::warn!("{}: {warning}", doc_path.display());
    }
    // A `#id` that bound to a sibling's fragment because this document's own
    // went away is the tangle silently holding somebody else's bytes.
    for warning in hick_literate::ambient::ambient_id_paste_warnings(doc_path, source) {
        log::warn!("{}: {warning}", doc_path.display());
    }
}
