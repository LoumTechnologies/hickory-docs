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
    for (path, source) in contributors(doc, name, base_dir)? {
        if let Err(e) = hick_lang::attach_contribution(doc, &path, &source, 0) {
            debug!("{}: not contributing to {name}: {e}", path.display());
        }
    }
    Ok(())
}

/// The documents beside this one that DO contribute, with their bytes, in the
/// order they contribute in.
///
/// Readable, parseable, not `<hick:private />`, and not already spliced in by
/// an `include` or `upstream` the author wrote. A sibling that cannot be read
/// or does not PARSE is skipped rather than failing this run: that is the
/// other document's error, reported when it is run, and it must not stop an
/// unrelated document from weaving.
///
/// Two callers: the splice above, and the warning that names an ambient
/// binding a document never asked for.
pub fn contributors(
    doc: &HickDocument,
    name: &str,
    base_dir: &std::path::Path,
) -> std::io::Result<Vec<(std::path::PathBuf, String)>> {
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

    let mut out = Vec::new();
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
        out.push((path, source));
    }
    Ok(out)
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

/// Say when a `#id` paste bound to a sibling this document never named.
///
/// Fragments are discovered folder-wide, and that is the feature: many
/// documents contribute to one file without any of them naming the others
/// (`docs/specs/freeform/many-documents-contribute-to-one-file.md`). The
/// hazard is the same mechanism read the other way. When a document's OWN
/// fragment for `#imports` is renamed or deleted and a sibling in the folder
/// happens to declare `#imports` too, the paste keeps resolving — to somebody
/// else's bytes — and the tangled file is quietly wrong. It happened twice in
/// one repository, both times on a generic id (`#comment`, `#imports`).
///
/// Only `#id`, and only through an AMBIENT contribution:
///
/// - A `.class` selector is multi-match by design. Collecting a sibling's
///   contribution is the whole point of writing one, and warning about it
///   would make the feature unusable.
/// - A `<hick:upstream>` or `<hick:include>` the author wrote IS the document
///   naming it. Those fragments are already spliced before this looks, so
///   they never reach the sibling scan.
///
/// A warning, never an error: the binding may well be what the author wants,
/// and refusing to weave would break every folder that already relies on it.
/// What it must not be is silent.
pub fn ambient_id_paste_warnings(doc_path: &std::path::Path, source: &str) -> Vec<String> {
    let Ok(mut doc) = hick_lang::parse(source) else {
        return Vec::new();
    };
    let base_dir = doc_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let mut seen = std::collections::HashSet::new();
    if let Ok(canonical) = std::fs::canonicalize(doc_path) {
        seen.insert(canonical);
    }
    // A document whose includes do not resolve has a louder problem than
    // this, and it is reported where includes are resolved.
    if hick_lang::resolve_includes(&mut doc, base_dir, &mut seen).is_err() {
        return Vec::new();
    }
    let wanted = unresolved_own_id_pastes(&doc);
    if wanted.is_empty() {
        return Vec::new();
    }
    let name = doc_path.to_string_lossy().to_string();
    let Ok(siblings) = contributors(&doc, &name, base_dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (id, line) in wanted {
        let selector = format!("#{id}");
        for (path, sibling_source) in &siblings {
            let Ok(sibling) = hick_lang::parse(sibling_source) else {
                continue;
            };
            if hick_lang::fragments_matching(&sibling, &selector).is_empty() {
                continue;
            }
            let file = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            out.push(format!(
                "paste \"{selector}\" at {}:{line} resolved from sibling {file}, which this \
                 document does not name (declare <hick:upstream file=\"{file}\" /> to mean it, \
                 or a .class selector to collect)",
                doc_path.display()
            ));
            // The first contributor wins the id, exactly as resolution does.
            break;
        }
    }
    out
}

/// This document's own `#id` pastes that nothing it declares or names answers
/// — the only ones an ambient sibling can be answering.
fn unresolved_own_id_pastes(doc: &HickDocument) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let mut stack: Vec<&hick_lang::HickTag> = doc.tags().collect();
    while let Some(tag) = stack.pop() {
        for child in &tag.children {
            if let HickNode::Tag(t) = child {
                stack.push(t);
            }
        }
        // Spliced pastes belong to the file that wrote them.
        if tag.name != "paste" || tag.source_span.is_some_and(|s| s.file_id.is_some()) {
            continue;
        }
        let Some(select) = tag_attr(tag, "select") else {
            continue;
        };
        let select = select.trim();
        let Some(id) = select.strip_prefix('#') else {
            continue;
        };
        if id.is_empty() || select.contains(',') {
            continue;
        }
        if !hick_lang::fragments_matching(doc, select).is_empty() {
            continue;
        }
        out.push((id.to_string(), tag.source_line));
    }
    out.sort_by_key(|(_, line)| *line);
    out.dedup();
    out
}
