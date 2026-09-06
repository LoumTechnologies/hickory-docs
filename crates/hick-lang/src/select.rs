//! Selector resolution: which elements an `#id`, a `.class`, or a
//! comma-separated list of them names.
//!
//! Two sets over one grammar, and the difference is the point. A
//! `hick:paste` and a `hick:transform` READ BYTES, so they see the pasteable
//! fragments only. A `cites=` POINTS AT an element — very often the cell
//! whose expectations prove the sentence — so it sees every element with an
//! id. See `docs/specs/freeform/provenance-and-standing.md`.

use crate::{HickDocument, HickNode, HickTag};

/// Every fragment block (`copy`/`cut`) in `doc` matching `selector`, in
/// document order. The selector grammar is `hick:paste`'s: `#id`, `.class`,
/// or a comma-separated list.
pub fn fragments_matching<'a>(doc: &'a HickDocument, selector: &str) -> Vec<&'a HickTag> {
    let mut out = Vec::new();
    collect_matching(&doc.nodes, selector, &mut out);
    out
}

/// Elements whose content is a selectable fragment.
///
/// `transcript` and `said` join `copy`/`cut` because an ingested meeting is
/// exactly the thing a `hick:transform` wants to point at — either whole
/// (`select="#t"`) or one speaker's turns (`select=".said-sam"`). Without
/// them the derived turns would be addressable in principle and selectable by
/// nothing. See `docs/specs/freeform/ingest.md`.
pub(crate) fn is_fragment_tag(name: &str) -> bool {
    matches!(name, "copy" | "cut" | "transcript" | "said")
}

/// Every ELEMENT in `doc` matching `selector`, pasteable or not.
///
/// The difference from [`fragments_matching`] is the whole point: a citation
/// names *what backs a sentence*, and what backs it is very often an
/// `<hick:exec id="…">` whose expectations verify it, a `<hick:table>` whose
/// rows are the evidence, or a `<hick:file>` that is the artefact under
/// discussion. None of those is text a `hick:paste` could reproduce, and
/// resolving `cites="#cited-tests"` against the pasteable set reported a
/// citation that points at a real, present, checkable element as DANGLING —
/// which is the one thing that verdict must never say wrongly.
///
/// So the two resolvers are deliberately different sets over the same
/// selector grammar. `hick:paste` and `hick:transform` read bytes and keep
/// the narrow set; `cites=` points at an element and gets this one. See
/// `docs/specs/freeform/provenance-and-standing.md`.
pub fn elements_matching<'a>(doc: &'a HickDocument, selector: &str) -> Vec<&'a HickTag> {
    let mut out = Vec::new();
    collect_matching_where(&doc.nodes, selector, &|_| true, &mut out);
    out
}

/// The element kind carrying `id` in this document, when one does — the word
/// an error or a listing uses to name it (`exec`, `table`, `file`).
pub fn element_kind_of_id<'a>(doc: &'a HickDocument, id: &str) -> Option<&'a str> {
    elements_matching(doc, &format!("#{id}"))
        .first()
        .map(|t| t.name.as_str())
}

fn collect_matching<'a>(nodes: &'a [HickNode], selector: &str, out: &mut Vec<&'a HickTag>) {
    collect_matching_where(nodes, selector, &|name| is_fragment_tag(name), out);
}

fn collect_matching_where<'a>(
    nodes: &'a [HickNode],
    selector: &str,
    eligible: &dyn Fn(&str) -> bool,
    out: &mut Vec<&'a HickTag>,
) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if eligible(&tag.name) && fragment_matches(tag, selector) {
                // No recursion into a match: selecting a whole transcript must
                // not also select each of its turns and count them twice.
                out.push(tag);
                continue;
            }
            collect_matching_where(&tag.children, selector, eligible, out);
        }
    }
}

fn fragment_matches(tag: &HickTag, selector: &str) -> bool {
    selector
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .any(|sel| {
            if let Some(id) = sel.strip_prefix('#') {
                tag.get_attribute("id") == Some(id)
            } else if let Some(class) = sel.strip_prefix('.') {
                tag.get_attribute("class")
                    .is_some_and(|c| c.split_whitespace().any(|x| x == class))
            } else {
                false
            }
        })
}
