//! Generated-output lineage: the `Provenance[]` shape from
//! `docs/specs/freeform/api.md` ("Generated outputs & lineage (v0.2)"),
//! built from a pipeline's [`ProvenanceMap`], plus reverse mapping of
//! output edits to source-document edits.
//!
//! The mapping is byte-precise: an output range is editable exactly when
//! every byte of it maps 1:1 onto bytes of a source `.hick` document
//! (Literal text, or a paste whose bytes reproduce a copy block verbatim).
//! Everything else — separators, exec output, variable values — is
//! `synthetic` for editing purposes and rejected with the offending range.

use std::collections::HashMap;

use hick_flow::{ProvenanceMap, SourceOrigin};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// api.md Provenance[] shape
// ---------------------------------------------------------------------------

/// One provenance entry: a byte range in the generated output and where it
/// came from. Serializes exactly as pinned in api.md.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    /// Byte range in the output content (start inclusive, end exclusive).
    pub start: usize,
    pub end: usize,
    pub origin: Origin,
}

/// Where an output range originated. `kind: "synthetic"` carries no source
/// location and is not editable; all other kinds carry a byte span in a
/// source document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Origin {
    Literal {
        doc_path: String,
        span: (usize, usize),
    },
    Paste {
        doc_path: String,
        span: (usize, usize),
    },
    Exec {
        doc_path: String,
        span: (usize, usize),
    },
    Variable {
        doc_path: String,
        span: (usize, usize),
    },
    /// Bytes authored by an agent cell. Always carries the session id and
    /// turn; carries a document span only when the bytes are byte-identical
    /// to one. A reader who cannot open the session still gets the id and
    /// turn — that is the designed outcome, not an error.
    Agent {
        session: String,
        turn: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        doc_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        span: Option<(usize, usize)>,
    },
    Substitution {
        doc_path: String,
        span: (usize, usize),
    },
    Synthetic,
}

impl Origin {
    /// The source location of any non-synthetic origin (for display).
    /// Editability is decided by [`Origin::source`], not this.
    pub fn location(&self) -> Option<(&str, usize, usize)> {
        match self {
            Origin::Literal { doc_path, span }
            | Origin::Paste { doc_path, span }
            | Origin::Exec { doc_path, span }
            | Origin::Variable { doc_path, span }
            | Origin::Substitution { doc_path, span } => Some((doc_path, span.0, span.1)),
            Origin::Agent {
                doc_path: Some(doc_path),
                span: Some(span),
                ..
            } => Some((doc_path, span.0, span.1)),
            Origin::Agent { .. } | Origin::Synthetic => None,
        }
    }

    /// The agent session and turn behind these bytes, if any.
    pub fn agent(&self) -> Option<(&str, usize)> {
        match self {
            Origin::Agent { session, turn, .. } => Some((session, *turn)),
            _ => None,
        }
    }

    /// The source location, when this origin is byte-precise (editable).
    pub fn source(&self) -> Option<(&str, usize, usize)> {
        match self {
            // Only Literal and Paste origins are guaranteed byte-identical to
            // their source span; exec/variable/substitution values are
            // derived, so they are never produced with spans today (see
            // `from_provenance_map`) and would not be editable anyway.
            Origin::Literal { doc_path, span } | Origin::Paste { doc_path, span } => {
                Some((doc_path, span.0, span.1))
            }
            // An agent writes through `edit_doc`, so its bytes are in the
            // document and editable — but only when a byte-precise span was
            // recorded. Without one it behaves like any other derived value.
            Origin::Agent {
                doc_path: Some(doc_path),
                span: Some(span),
                ..
            } => Some((doc_path, span.0, span.1)),
            _ => None,
        }
    }
}

/// Convert a pipeline [`ProvenanceMap`] into the api.md `Provenance[]` shape.
///
/// Origins without a byte-precise source location (variable values,
/// substituted segments, separators) are reported as `synthetic`; exec output
/// is too, HERE, and `hickory_cli::output_lineage` then gives it the cell's
/// own span as an `Exec` origin — drawable, never editable:
/// honest about editability — those bytes cannot be mapped back to source
/// bytes 1:1.
pub fn from_provenance_map(map: &ProvenanceMap) -> Vec<Provenance> {
    map.spans()
        .iter()
        .map(|s| {
            let out_len = s.output_end - s.output_start;
            let origin = match &s.origin {
                SourceOrigin::Literal { file, span } if span.len() == out_len => Origin::Literal {
                    doc_path: file.to_string(),
                    span: (span.start, span.end),
                },
                SourceOrigin::Paste {
                    file: Some(file),
                    span: Some(span),
                    ..
                } if span.len() == out_len => Origin::Paste {
                    doc_path: file.to_string(),
                    span: (span.start, span.end),
                },
                // Never degrades to `synthetic`: the session id and turn are
                // the whole point of the variant, and they survive even when
                // no byte-precise document span was recorded.
                SourceOrigin::Agent {
                    session,
                    turn,
                    file,
                    span,
                } => {
                    let located = match (file, span) {
                        (Some(f), Some(sp)) if sp.len() == out_len => {
                            Some((f.to_string(), (sp.start, sp.end)))
                        }
                        _ => None,
                    };
                    let (doc_path, span) = match located {
                        Some((f, sp)) => (Some(f), Some(sp)),
                        None => (None, None),
                    };
                    Origin::Agent {
                        session: session.to_string(),
                        turn: *turn,
                        doc_path,
                        span,
                    }
                }
                _ => Origin::Synthetic,
            };
            Provenance {
                start: s.output_start,
                end: s.output_end,
                origin,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Output edits → source edits
// ---------------------------------------------------------------------------

/// An edit to a generated output file: replace bytes `start..end` with `text`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEdit {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// A resulting edit to a source document, api.md shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceEdit {
    pub doc_path: String,
    pub span: (usize, usize),
    pub text: String,
}

/// Why an edit batch could not be mapped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineageError {
    /// The edit overlaps a synthetic (non-editable) output range.
    /// Carries the offending output byte range.
    SyntheticOverlap { start: usize, end: usize },
    /// Structurally invalid edits (out of bounds, overlapping, char
    /// boundaries, …).
    InvalidEdit(String),
    /// Edits that map to overlapping or non-contiguous source regions.
    Conflict(String),
}

impl std::fmt::Display for LineageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineageError::SyntheticOverlap { start, end } => write!(
                f,
                "edit overlaps a synthetic (non-editable) output range at bytes {start}..{end}"
            ),
            LineageError::InvalidEdit(m) => write!(f, "invalid edit: {m}"),
            LineageError::Conflict(m) => write!(f, "conflicting edits: {m}"),
        }
    }
}

impl std::error::Error for LineageError {}

/// Map output edits to source-document edits through provenance.
///
/// Rules:
/// - Edits must be in-bounds, on char boundaries, and mutually non-overlapping.
/// - Every byte of a replaced range must be covered by an editable
///   (span-carrying) provenance entry; overlap with a `synthetic` range (or a
///   provenance gap) fails with [`LineageError::SyntheticOverlap`] carrying
///   the offending output range.
/// - A replaced range may cross several provenance entries only when their
///   source spans are contiguous in one document (the edit is then applied to
///   the union). Pure insertions attach to the entry containing (or ending
///   at) the insertion point.
/// - Resulting source edits must not overlap each other (e.g. one copy block
///   pasted twice, both occurrences edited differently).
///
/// The returned edits are sorted by descending source span so they can be
/// applied back-to-front without offset fixups.
pub fn map_edits(
    content: &str,
    edits: &[OutputEdit],
    provenance: &[Provenance],
) -> Result<Vec<SourceEdit>, LineageError> {
    // Validate edit ranges.
    let mut sorted: Vec<&OutputEdit> = edits.iter().collect();
    sorted.sort_by_key(|e| (e.start, e.end));
    for e in &sorted {
        if e.start > e.end || e.end > content.len() {
            return Err(LineageError::InvalidEdit(format!(
                "range {}..{} out of bounds for {}-byte output",
                e.start,
                e.end,
                content.len()
            )));
        }
        if !content.is_char_boundary(e.start) || !content.is_char_boundary(e.end) {
            return Err(LineageError::InvalidEdit(format!(
                "range {}..{} does not fall on UTF-8 character boundaries",
                e.start, e.end
            )));
        }
    }
    for w in sorted.windows(2) {
        if w[1].start < w[0].end {
            return Err(LineageError::InvalidEdit(format!(
                "edits {}..{} and {}..{} overlap",
                w[0].start, w[0].end, w[1].start, w[1].end
            )));
        }
    }

    let mut source_edits: Vec<SourceEdit> = Vec::new();
    for e in &sorted {
        source_edits.push(map_one(e, provenance)?);
    }

    // Reject overlapping source edits (same source bytes targeted twice).
    let mut by_pos: Vec<&SourceEdit> = source_edits.iter().collect();
    by_pos.sort_by_key(|s| (s.doc_path.clone(), s.span.0, s.span.1));
    for w in by_pos.windows(2) {
        if w[0].doc_path == w[1].doc_path && w[1].span.0 < w[0].span.1 {
            return Err(LineageError::Conflict(format!(
                "edits map to overlapping source ranges {}..{} and {}..{} in {}",
                w[0].span.0, w[0].span.1, w[1].span.0, w[1].span.1, w[0].doc_path
            )));
        }
    }

    // Descending source order for back-to-front application.
    source_edits.sort_by(|a, b| {
        (b.doc_path.as_str(), b.span.0, b.span.1).cmp(&(a.doc_path.as_str(), a.span.0, a.span.1))
    });
    Ok(source_edits)
}

/// Is `entry`'s source span also feeding a DIFFERENT output range (the same
/// copy block pasted more than once)? Editing such a range cannot be
/// reproduced exactly: rewriting the shared source bytes changes every
/// occurrence on the next run, not just the edited one.
fn source_is_duplicated(entry: &Provenance, provenance: &[Provenance]) -> bool {
    let Some((doc, s, e)) = entry.origin.source() else {
        return false;
    };
    if s == e {
        return false;
    }
    provenance.iter().any(|q| {
        (q.start, q.end) != (entry.start, entry.end)
            && q.origin
                .source()
                .is_some_and(|(d2, s2, e2)| d2 == doc && s < e2 && e > s2)
    })
}

fn reject_if_duplicated(entry: &Provenance, provenance: &[Provenance]) -> Result<(), LineageError> {
    if source_is_duplicated(entry, provenance) {
        let (doc, s, e) = entry.origin.source().expect("duplicated implies source");
        return Err(LineageError::Conflict(format!(
            "output bytes {}..{} come from {doc} bytes {s}..{e}, which is woven \
             into more than one place — editing one occurrence cannot be \
             reproduced exactly; edit the source block instead",
            entry.start, entry.end
        )));
    }
    Ok(())
}

/// Map a single output edit to one source edit.
fn map_one(e: &OutputEdit, provenance: &[Provenance]) -> Result<SourceEdit, LineageError> {
    // Pure insertion: attach to the entry containing the point, or the entry
    // ending exactly at it.
    if e.start == e.end {
        // Prefer an editable attachment: the entry containing the point,
        // else the entry ending exactly at it (so inserting at the boundary
        // of an editable range and a synthetic separator still works).
        let containing = provenance
            .iter()
            .find(|p| p.start <= e.start && e.start < p.end);
        let ending = provenance.iter().rfind(|p| p.end == e.start);
        let entry = [containing, ending]
            .into_iter()
            .flatten()
            .find(|p| p.origin.source().is_some())
            .or(containing)
            .or(ending)
            .ok_or(LineageError::SyntheticOverlap {
                start: e.start,
                end: e.end,
            })?;
        let (doc_path, src_start, _src_end) =
            entry
                .origin
                .source()
                .ok_or(LineageError::SyntheticOverlap {
                    start: entry.start,
                    end: entry.end,
                })?;
        reject_if_duplicated(entry, provenance)?;
        let at = src_start + (e.start - entry.start);
        return Ok(SourceEdit {
            doc_path: doc_path.to_string(),
            span: (at, at),
            text: e.text.clone(),
        });
    }

    // Replacement: walk the overlapping provenance entries in output order,
    // requiring full editable coverage and source contiguity.
    let mut overlapping: Vec<&Provenance> = provenance
        .iter()
        .filter(|p| p.start < e.end && p.end > e.start)
        .collect();
    overlapping.sort_by_key(|p| p.start);

    let mut cursor = e.start;
    let mut doc: Option<&str> = None;
    let mut src_start = 0usize;
    let mut src_end = 0usize;
    for p in &overlapping {
        if p.start > cursor {
            // Gap in provenance coverage: not editable.
            return Err(LineageError::SyntheticOverlap {
                start: cursor,
                end: p.start,
            });
        }
        let Some((p_doc, p_src_start, _)) = p.origin.source() else {
            return Err(LineageError::SyntheticOverlap {
                start: p.start.max(e.start),
                end: p.end.min(e.end),
            });
        };
        reject_if_duplicated(p, provenance)?;
        let overlap_start = e.start.max(p.start);
        let overlap_end = e.end.min(p.end);
        let mapped_start = p_src_start + (overlap_start - p.start);
        let mapped_end = p_src_start + (overlap_end - p.start);
        match doc {
            None => {
                doc = Some(p_doc);
                src_start = mapped_start;
                src_end = mapped_end;
            }
            Some(d) => {
                if d != p_doc {
                    return Err(LineageError::Conflict(format!(
                        "edit {}..{} spans multiple source documents ({d} and {p_doc})",
                        e.start, e.end
                    )));
                }
                if mapped_start != src_end {
                    return Err(LineageError::Conflict(format!(
                        "edit {}..{} spans source regions that are not contiguous \
                         ({src_end} vs {mapped_start} in {d})",
                        e.start, e.end
                    )));
                }
                src_end = mapped_end;
            }
        }
        cursor = overlap_end;
    }
    if cursor < e.end {
        return Err(LineageError::SyntheticOverlap {
            start: cursor,
            end: e.end,
        });
    }
    let doc = doc.ok_or(LineageError::SyntheticOverlap {
        start: e.start,
        end: e.end,
    })?;
    Ok(SourceEdit {
        doc_path: doc.to_string(),
        span: (src_start, src_end),
        text: e.text.clone(),
    })
}

/// Apply source edits to their documents. `sources` maps doc path → content.
/// Edits must be non-overlapping; they are applied back-to-front per doc.
/// Returns the set of changed documents.
pub fn apply_source_edits(
    sources: &HashMap<String, String>,
    edits: &[SourceEdit],
) -> Result<HashMap<String, String>, LineageError> {
    let mut result: HashMap<String, String> = HashMap::new();
    let mut per_doc: HashMap<&str, Vec<&SourceEdit>> = HashMap::new();
    for e in edits {
        per_doc.entry(e.doc_path.as_str()).or_default().push(e);
    }
    for (doc_path, mut doc_edits) in per_doc {
        let Some(src) = sources.get(doc_path) else {
            return Err(LineageError::InvalidEdit(format!(
                "unknown source document {doc_path}"
            )));
        };
        doc_edits.sort_by_key(|e| std::cmp::Reverse(e.span.0));
        let mut content = src.clone();
        for e in doc_edits {
            if e.span.0 > e.span.1 || e.span.1 > content.len() {
                return Err(LineageError::InvalidEdit(format!(
                    "source span {}..{} out of bounds in {doc_path}",
                    e.span.0, e.span.1
                )));
            }
            // An error, not a panic: `replace_range` aborts the process on a
            // non-boundary offset, and a span pointing mid-character is
            // exactly what stale or mis-attributed provenance produces.
            if !content.is_char_boundary(e.span.0) || !content.is_char_boundary(e.span.1) {
                return Err(LineageError::InvalidEdit(format!(
                    "source span {}..{} does not fall on UTF-8 character boundaries \
                     in {doc_path} — the provenance is stale for this document",
                    e.span.0, e.span.1
                )));
            }
            content.replace_range(e.span.0..e.span.1, &e.text);
        }
        result.insert(doc_path.to_string(), content);
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Language detection
// ---------------------------------------------------------------------------

/// Map an output path to a language identifier, by extension.
pub fn language_for_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or_default();
    match ext.to_ascii_lowercase().as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        "sh" | "bash" => "shell",
        "html" | "htm" => "html",
        "css" => "css",
        "sql" => "sql",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "hpp" | "cxx" => "cpp",
        "rb" => "ruby",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "hick" => "hick",
        "xml" => "xml",
        "txt" => "text",
        _ => "text",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use hick_flow::{ProvenanceSpan, SourceOrigin};
    use hick_lang::SourceSpan;
    use std::sync::Arc;

    fn prov(entries: &[(usize, usize, Origin)]) -> Vec<Provenance> {
        entries
            .iter()
            .map(|(s, e, o)| Provenance {
                start: *s,
                end: *e,
                origin: o.clone(),
            })
            .collect()
    }

    fn lit(doc: &str, s: usize, e: usize) -> Origin {
        Origin::Literal {
            doc_path: doc.to_string(),
            span: (s, e),
        }
    }

    #[test]
    fn provenance_map_converts_to_api_shape() {
        let mut map = ProvenanceMap::new();
        map.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: SourceOrigin::Literal {
                file: Arc::from("a.hick"),
                span: SourceSpan::new(10, 15, 1, 0),
            },
        });
        map.push(ProvenanceSpan {
            output_start: 5,
            output_end: 8,
            origin: SourceOrigin::Synthetic,
        });
        map.push(ProvenanceSpan {
            output_start: 8,
            output_end: 12,
            origin: SourceOrigin::Paste {
                selector: Arc::from("#x"),
                file: Some(Arc::from("a.hick")),
                span: Some(SourceSpan::new(30, 34, 3, 0)),
            },
        });
        // Exec output has no byte-precise source span → synthetic.
        map.push(ProvenanceSpan {
            output_start: 12,
            output_end: 20,
            origin: SourceOrigin::Exec {
                container: Arc::from("gen"),
                tag_line: 7,
            },
        });
        // A length-mismatched literal (transformed text) must degrade to
        // synthetic rather than claim byte precision.
        map.push(ProvenanceSpan {
            output_start: 20,
            output_end: 22,
            origin: SourceOrigin::Literal {
                file: Arc::from("a.hick"),
                span: SourceSpan::new(50, 60, 5, 0),
            },
        });

        let p = from_provenance_map(&map);
        assert_eq!(p.len(), 5);
        assert_eq!(p[0].origin, lit("a.hick", 10, 15));
        assert_eq!(p[1].origin, Origin::Synthetic);
        assert_eq!(
            p[2].origin,
            Origin::Paste {
                doc_path: "a.hick".to_string(),
                span: (30, 34)
            }
        );
        assert_eq!(p[3].origin, Origin::Synthetic);
        assert_eq!(p[4].origin, Origin::Synthetic);

        // Serde shape matches api.md.
        let json = serde_json::to_value(&p[0]).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "start": 0, "end": 5,
                "origin": {"kind": "literal", "doc_path": "a.hick", "span": [10, 15]}
            })
        );
        let json = serde_json::to_value(&p[1]).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"start": 5, "end": 8, "origin": {"kind": "synthetic"}})
        );
    }

    // Protects docs/guarantees/lineage/agent-lineage-degrades-without-a-session.md
    #[test]
    fn agent_origin_keeps_session_and_turn_with_or_without_a_span() {
        let mut map = ProvenanceMap::new();
        map.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: SourceOrigin::Agent {
                session: Arc::from("abc123"),
                turn: 7,
                file: Some(Arc::from("a.hick")),
                span: Some(SourceSpan::new(10, 15, 1, 0)),
            },
        });
        // No document span recorded: still an agent origin, never synthetic —
        // losing the session id is exactly the failure this variant exists to
        // prevent.
        map.push(ProvenanceSpan {
            output_start: 5,
            output_end: 9,
            origin: SourceOrigin::Agent {
                session: Arc::from("def456"),
                turn: 0,
                file: None,
                span: None,
            },
        });
        // A length-mismatched span is not byte-precise, so the span is
        // dropped while the session survives.
        map.push(ProvenanceSpan {
            output_start: 9,
            output_end: 11,
            origin: SourceOrigin::Agent {
                session: Arc::from("ghi789"),
                turn: 3,
                file: Some(Arc::from("a.hick")),
                span: Some(SourceSpan::new(50, 60, 5, 0)),
            },
        });

        let p = from_provenance_map(&map);
        assert_eq!(p[0].origin.agent(), Some(("abc123", 7)));
        assert_eq!(p[0].origin.location(), Some(("a.hick", 10, 15)));
        // Agent bytes are written through the document, so they are editable
        // when byte-precise.
        assert_eq!(p[0].origin.source(), Some(("a.hick", 10, 15)));

        assert_eq!(p[1].origin.agent(), Some(("def456", 0)));
        assert_eq!(p[1].origin.location(), None);
        assert_eq!(p[1].origin.source(), None);

        assert_eq!(p[2].origin.agent(), Some(("ghi789", 3)));
        assert_eq!(p[2].origin.location(), None);

        // Serde shape: additive `kind`, span omitted rather than null.
        assert_eq!(
            serde_json::to_value(&p[1]).unwrap(),
            serde_json::json!({
                "start": 5, "end": 9,
                "origin": {"kind": "agent", "session": "def456", "turn": 0}
            })
        );
        let round: Provenance =
            serde_json::from_value(serde_json::to_value(&p[0]).unwrap()).unwrap();
        assert_eq!(round.origin, p[0].origin);
    }

    #[test]
    fn maps_edit_within_one_span() {
        let p = prov(&[(0, 10, lit("d.hick", 100, 110))]);
        let edits = vec![OutputEdit {
            start: 3,
            end: 7,
            text: "XY".to_string(),
        }];
        let out = map_edits("0123456789", &edits, &p).unwrap();
        assert_eq!(
            out,
            vec![SourceEdit {
                doc_path: "d.hick".to_string(),
                span: (103, 107),
                text: "XY".to_string()
            }]
        );
    }

    #[test]
    fn splits_edit_across_contiguous_spans() {
        // Two output spans whose source spans are adjacent.
        let p = prov(&[
            (0, 5, lit("d.hick", 100, 105)),
            (5, 10, lit("d.hick", 105, 110)),
        ]);
        let edits = vec![OutputEdit {
            start: 3,
            end: 8,
            text: "Z".to_string(),
        }];
        let out = map_edits("0123456789", &edits, &p).unwrap();
        assert_eq!(out[0].span, (103, 108));
        assert_eq!(out[0].text, "Z");
    }

    #[test]
    fn rejects_non_contiguous_source_spans() {
        let p = prov(&[
            (0, 5, lit("d.hick", 100, 105)),
            (5, 10, lit("d.hick", 200, 205)),
        ]);
        let edits = vec![OutputEdit {
            start: 3,
            end: 8,
            text: "Z".to_string(),
        }];
        assert!(matches!(
            map_edits("0123456789", &edits, &p),
            Err(LineageError::Conflict(_))
        ));
    }

    #[test]
    fn rejects_synthetic_overlap_with_offending_range() {
        let p = prov(&[
            (0, 5, lit("d.hick", 100, 105)),
            (5, 8, Origin::Synthetic),
            (8, 12, lit("d.hick", 105, 109)),
        ]);
        let edits = vec![OutputEdit {
            start: 4,
            end: 9,
            text: "Z".to_string(),
        }];
        assert_eq!(
            map_edits("0123456789ab", &edits, &p),
            Err(LineageError::SyntheticOverlap { start: 5, end: 8 })
        );
    }

    #[test]
    fn rejects_gap_in_provenance() {
        let p = prov(&[(0, 4, lit("d.hick", 100, 104))]);
        let edits = vec![OutputEdit {
            start: 2,
            end: 6,
            text: "Z".to_string(),
        }];
        assert_eq!(
            map_edits("0123456789", &edits, &p),
            Err(LineageError::SyntheticOverlap { start: 4, end: 6 })
        );
    }

    #[test]
    fn insertion_at_span_boundary_attaches_to_preceding_span() {
        let p = prov(&[(0, 5, lit("d.hick", 100, 105)), (5, 8, Origin::Synthetic)]);
        let edits = vec![OutputEdit {
            start: 5,
            end: 5,
            text: "ins".to_string(),
        }];
        // Byte 5 is the boundary between the editable span and the synthetic
        // separator: the insertion attaches to the editable entry ending
        // there (the sweep test proves this reproduces byte-for-byte).
        let out = map_edits("01234567", &edits, &p).unwrap();
        assert_eq!(out[0].span, (105, 105));
        assert_eq!(out[0].text, "ins");

        // At the very end of the output, the trailing editable span takes it.
        let p2 = prov(&[(0, 5, lit("d.hick", 100, 105))]);
        let edits2 = vec![OutputEdit {
            start: 5,
            end: 5,
            text: "ins".to_string(),
        }];
        let out = map_edits("01234", &edits2, &p2).unwrap();
        assert_eq!(out[0].span, (105, 105));
        assert_eq!(out[0].text, "ins");
    }

    #[test]
    fn rejects_overlapping_output_edits() {
        let p = prov(&[(0, 10, lit("d.hick", 0, 10))]);
        let edits = vec![
            OutputEdit {
                start: 0,
                end: 5,
                text: "a".into(),
            },
            OutputEdit {
                start: 4,
                end: 8,
                text: "b".into(),
            },
        ];
        assert!(matches!(
            map_edits("0123456789", &edits, &p),
            Err(LineageError::InvalidEdit(_))
        ));
    }

    #[test]
    fn rejects_edits_mapping_to_same_source_bytes() {
        // One copy block pasted twice: editing both occurrences conflicts.
        let p = prov(&[
            (0, 5, lit("d.hick", 100, 105)),
            (5, 10, lit("d.hick", 100, 105)),
        ]);
        let edits = vec![
            OutputEdit {
                start: 0,
                end: 5,
                text: "a".into(),
            },
            OutputEdit {
                start: 5,
                end: 10,
                text: "b".into(),
            },
        ];
        assert!(matches!(
            map_edits("0123456789", &edits, &p),
            Err(LineageError::Conflict(_))
        ));
    }

    #[test]
    fn applies_source_edits_back_to_front() {
        let mut sources = HashMap::new();
        sources.insert("d.hick".to_string(), "hello world".to_string());
        let edits = vec![
            SourceEdit {
                doc_path: "d.hick".to_string(),
                span: (6, 11),
                text: "there".to_string(),
            },
            SourceEdit {
                doc_path: "d.hick".to_string(),
                span: (0, 5),
                text: "hi".to_string(),
            },
        ];
        let out = apply_source_edits(&sources, &edits).unwrap();
        assert_eq!(out["d.hick"], "hi there");
    }

    #[test]
    fn language_detection_by_extension() {
        assert_eq!(language_for_path("out/gen.rs"), "rust");
        assert_eq!(language_for_path("a/b.PY"), "python");
        assert_eq!(language_for_path("x.unknownext"), "text");
        assert_eq!(language_for_path("noext"), "text");
    }
}
