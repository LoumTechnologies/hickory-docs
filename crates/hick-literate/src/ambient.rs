//! Ambient contribution: every other `.hick` beside a document offers its
//! fragments to it, without either naming the other.
//!
//! Split out of `lib.rs` because two things now read this folder scan — the
//! splice itself, and the warning that says when a `#id` paste bound to a
//! sibling this document never named
//! (`docs/guarantees/authoring/an-ambient-id-paste-is-named.md`).

use std::sync::Arc;

use hick_exec::state::MultiDocumentState;
use hick_lang::{HickDocument, HickNode};
use log::debug;

use crate::tag_attr;

/// Whether a document keeps its fragments to itself.
///
/// `<hick:private />` is the opt-out from ambient contribution. The default
/// is the other way round — a fragment is offered to the folder — because the
/// case this exists for is many documents feeding one shared file, and making
/// each of them announce itself puts the coupling straight back.
fn declares_private(nodes: &[HickNode]) -> bool {
    nodes
        .iter()
        .any(|node| matches!(node, HickNode::Tag(tag) if tag.name == "private"))
}

/// Append a `hick:upstream` edge for every other `.hick` beside this one.
///
/// Directory-scoped and not recursive: a folder is the unit a person can hold
/// in their head, and walking a whole repository would let a fragment in some
/// unrelated corner change this document's output.
///
/// A sibling that cannot be read or does not PARSE is skipped rather than
/// failing this run: that is the other document's error, reported when it is
/// run, and it must not stop an unrelated document from weaving.
pub(crate) fn attach_ambient_contributors(
    doc: &mut HickDocument,
    name: &str,
    base_dir: &std::path::Path,
) -> std::io::Result<()> {
    // Every file already spliced into this document, by include or by
    // upstream, at any depth. `span_files` is the right record because both
    // branches stamp it; the resolver's `seen` set is a cycle-detection
    // STACK and is popped on the way back out, so by the time it returns it
    // holds nothing but the document itself.
    let already_merged: std::collections::HashSet<std::path::PathBuf> = doc
        .span_files
        .iter()
        .filter_map(|f| std::fs::canonicalize(f).ok())
        .collect();
    // A document named without a directory has `""` as its parent, and
    // `read_dir("")` is an error rather than the current directory.
    let dir = if base_dir.as_os_str().is_empty() {
        std::path::Path::new(".")
    } else {
        base_dir
    };
    let self_path = std::fs::canonicalize(name).ok();
    let mut siblings: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("hick") {
            continue;
        }
        let canonical = std::fs::canonicalize(&path).ok();
        if canonical == self_path {
            continue;
        }
        // Already reachable through an `include` or `upstream` the author
        // wrote — possibly several hops away, which is why this asks the
        // resolver's own record rather than re-reading the tags. Splicing it
        // a second time is a duplicate-id error, not a no-op.
        if let Some(canonical) = &canonical
            && already_merged.contains(canonical)
        {
            continue;
        }
        siblings.push(path);
    }
    // Directory read order is not stable across filesystems, and a `.class`
    // paste concatenates in document order — so without this the same folder
    // could weave two different files on two machines.
    siblings.sort();

    for path in siblings {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(sibling) = hick_lang::parse(&source) else {
            continue;
        };
        if declares_private(&sibling.nodes) {
            debug!("{}: private, not contributing to {name}", path.display());
            continue;
        }
        if let Err(e) = hick_lang::attach_contribution(doc, &path, &source, 0) {
            debug!("{}: not contributing to {name}: {e}", path.display());
        }
    }
    Ok(())
}

/// Record `id` → element kind for every tag in `nodes`, at any depth.
pub(crate) fn register_element_kinds(nodes: &[HickNode], state: &Arc<MultiDocumentState>) {
    for node in nodes {
        let HickNode::Tag(tag) = node else { continue };
        if let Some(id) = tag_attr(tag, "id") {
            state.element_kinds.register(&id, &tag.name);
        }
        register_element_kinds(&tag.children, state);
    }
}
