//! Declared citations: `cites="…"` resolved to the places it names.
//!
//! Split out of `lib.rs` because it is one self-contained question — what
//! does this document SAY it rests on — with its own vocabulary. See
//! `docs/guarantees/lineage/a-citation-that-points-at-nothing-fails.md`.

use std::path::Path;

use anyhow::Result;

use crate::{line_of, transform_document};

/// One declared citation: an element that says `cites="…"` and the fragments
/// its selectors name — DECLARED provenance, the author's assertion, resolved
/// to places so the app can draw it as one (and never as lineage).
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct DeclaredCite {
    /// The `cites=` value as written.
    pub select: String,
    /// The citing element.
    pub from: CitePlace,
    /// What the selectors resolved to — possibly in upstream documents,
    /// possibly nothing (a dangling citation is reported, not hidden).
    pub to: Vec<CitePlace>,
    /// The individual selectors in `select` that matched nothing at all.
    ///
    /// Resolved one at a time rather than as a set, because the set was
    /// hiding them: `cites="#a,#b"` where only `#a` exists produced a
    /// non-empty `to`, and `#b` vanished with no warning anywhere — the
    /// weave still printed its name beside the ones that resolved, so a
    /// claim sourced to nothing looked exactly like a claim sourced to
    /// something.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dangling: Vec<String>,
}

/// A place a citation points at or comes from: a file and a line range.
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct CitePlace {
    pub path: String,
    pub first_line: usize,
    pub last_line: usize,
    /// The element's name (`claim`, `transform`, `copy`, `said`, …).
    pub element: String,
    /// Its id, when it has one.
    pub id: Option<String>,
}

/// Every `cites=` in the document (its own elements, not spliced ones),
/// resolved against the document as the pipeline sees it — upstreams and
/// transcript turns included. Paths are the document's path for its own
/// elements and the spliced file's canonical path for upstream fragments.
pub fn declared_cites(doc_path: &Path, source: &str) -> Result<Vec<DeclaredCite>> {
    let doc = transform_document(doc_path, source)?;
    let own_lines = |span: Option<hick_lang::SourceSpan>| -> (usize, usize) {
        span.map(|s| {
            (
                line_of(source, s.start),
                line_of(source, s.end.max(s.start)),
            )
        })
        .unwrap_or((0, 0))
    };
    // Line numbers in a spliced file need that file's text.
    let mut file_sources: std::collections::HashMap<usize, String> = Default::default();
    let mut place_of = |tag: &hick_lang::HickTag| -> CitePlace {
        let (path, first, last) = match tag.source_span {
            Some(s) => match s.file_id {
                None => {
                    let (a, b) = own_lines(Some(s));
                    (doc_path.display().to_string(), a, b)
                }
                Some(id) => {
                    let path = doc.span_files[usize::from(id)].clone();
                    let text = file_sources
                        .entry(usize::from(id))
                        .or_insert_with(|| std::fs::read_to_string(&path).unwrap_or_default())
                        .clone();
                    (
                        path,
                        line_of(&text, s.start),
                        line_of(&text, s.end.max(s.start)),
                    )
                }
            },
            None => {
                // A derived turn has no tag span; its text child carries the
                // span into the transcript's file.
                match tag.children.first() {
                    Some(hick_lang::HickNode::Text(_, Some(s))) => match s.file_id {
                        None => {
                            let (a, b) = own_lines(Some(*s));
                            (doc_path.display().to_string(), a, b)
                        }
                        Some(id) => {
                            let path = doc.span_files[usize::from(id)].clone();
                            let text = file_sources
                                .entry(usize::from(id))
                                .or_insert_with(|| {
                                    std::fs::read_to_string(&path).unwrap_or_default()
                                })
                                .clone();
                            (
                                path,
                                line_of(&text, s.start),
                                line_of(&text, s.end.max(s.start)),
                            )
                        }
                    },
                    _ => (doc_path.display().to_string(), 0, 0),
                }
            }
        };
        CitePlace {
            path,
            first_line: first,
            last_line: last,
            element: tag.name.clone(),
            id: tag.get_attribute("id").map(str::to_string),
        }
    };
    let mut out = Vec::new();
    let mut stack: Vec<&hick_lang::HickTag> = doc.tags().collect();
    let mut citing: Vec<&hick_lang::HickTag> = Vec::new();
    while let Some(tag) = stack.pop() {
        // Only this document's own elements declare for it.
        if tag.source_span.is_some_and(|s| s.file_id.is_some()) {
            continue;
        }
        if tag
            .get_attribute("cites")
            .is_some_and(|c| !c.trim().is_empty())
        {
            citing.push(tag);
        }
        for child in &tag.children {
            if let hick_lang::HickNode::Tag(t) = child {
                stack.push(t);
            }
        }
    }
    citing.sort_by_key(|t| t.source_line);
    for tag in citing {
        let select = tag
            .get_attribute("cites")
            .unwrap_or_default()
            .trim()
            .to_string();
        // One selector at a time. `elements_matching` takes the whole
        // comma-separated list and answers with the union, which cannot say
        // which of them contributed nothing.
        //
        // `elements_matching`, not `fragments_matching`: a citation names the
        // element that backs a sentence, and that is usually NOT text — it is
        // the cell whose expectations prove it, the table holding the rows,
        // the file under discussion. Resolving against the pasteable set
        // reported `cites="#cited-tests"` as dangling while the
        // `<hick:exec id="cited-tests">` it names sat two lines below it.
        let mut to = Vec::new();
        let mut dangling = Vec::new();
        for one in select.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let matched = hick_lang::elements_matching(&doc, one);
            if matched.is_empty() {
                dangling.push(one.to_string());
                continue;
            }
            for fragment in matched {
                let place = place_of(fragment);
                if !to.contains(&place) {
                    to.push(place);
                }
            }
        }
        out.push(DeclaredCite {
            select,
            from: place_of(tag),
            to,
            dangling,
        });
    }
    Ok(out)
}
