//! Reading an index, and refusing to show what it cannot explain.
//!
//! `docs/specs/freeform/an-index-beside-the-language-server.md`. The
//! producing half is [`crate::index_install`]; this is the half that answers
//! a question with what it produced.
//!
//! Three rules from that document shape everything here, and each is a
//! refusal rather than a feature:
//!
//! **An index is a cache, and is marked as one.** It is stale the moment you
//! type. It may speed up an answer and may never *be* the answer to a
//! question about correctness, so every report says when it was built and
//! whether the files it covers have moved since.
//!
//! **Positions go through lineage or are not shown.** An index of woven files
//! holds `(file, line, column)` for files a person never edits. Showing one
//! without mapping it back to a document span would send them to a file that
//! regenerates over their edit — the exact failure
//! `a-generated-file-refuses-an-edit` exists to prevent. A reference in a
//! generated file that cannot be mapped back is **dropped**, and the count of
//! dropped ones is reported, because silently showing fewer results than
//! exist is its own kind of lie.
//!
//! **The index is never required.** Nothing here is on the path of any
//! navigation feature that already works.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// What an index was built from, written beside it.
///
/// Staleness is not a second mechanism: it is the same question recordings
/// already answer — did an input change — asked with the same tool, a hash of
/// the bytes that went in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltFrom {
    /// RFC 3339, so a report can say "indexed at 14:02" where the age matters.
    pub at: String,
    /// Project-relative path → SHA-256 at index time.
    pub inputs: HashMap<String, String>,
}

impl BuiltFrom {
    /// Which of the indexed files have changed since, in a stable order.
    pub fn moved_on(&self, root: &Path) -> Vec<String> {
        let mut stale: Vec<String> = self
            .inputs
            .iter()
            .filter(|(path, was)| {
                let now = std::fs::read(root.join(path))
                    .ok()
                    .map(|bytes| format!("{:x}", Sha256::digest(&bytes)));
                // A file that has been deleted has certainly moved on.
                now.as_deref() != Some(was.as_str())
            })
            .map(|(path, _)| path.clone())
            .collect();
        stale.sort();
        stale
    }
}

/// Where an index and its provenance live for one language.
pub fn index_path(root: &Path, language: &str) -> PathBuf {
    root.join(".hick-cache/index")
        .join(format!("{language}.scip"))
}

fn built_from_path(root: &Path, language: &str) -> PathBuf {
    root.join(".hick-cache/index")
        .join(format!("{language}.built-from.json"))
}

/// Record what an index covers, right after building it.
pub fn write_built_from(root: &Path, language: &str, at: &str) -> Result<BuiltFrom> {
    let index = read_index(&index_path(root, language))?;
    let inputs = index
        .documents
        .iter()
        .filter_map(|doc| {
            let bytes = std::fs::read(root.join(&doc.relative_path)).ok()?;
            Some((
                doc.relative_path.clone(),
                format!("{:x}", Sha256::digest(&bytes)),
            ))
        })
        .collect();
    let built = BuiltFrom {
        at: at.to_string(),
        inputs,
    };
    std::fs::write(
        built_from_path(root, language),
        serde_json::to_vec_pretty(&built)?,
    )?;
    Ok(built)
}

/// What an index says it was built from, when it says.
pub fn built_from(root: &Path, language: &str) -> Option<BuiltFrom> {
    let text = std::fs::read_to_string(built_from_path(root, language)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Read a SCIP index off disk.
pub fn read_index(path: &Path) -> Result<scip::types::Index> {
    let bytes = std::fs::read(path).with_context(|| {
        format!(
            "no index at {} — `hick index build` writes one",
            path.display()
        )
    })?;
    <scip::types::Index as protobuf::Message>::parse_from_bytes(&bytes)
        .with_context(|| format!("{} is not a SCIP index", path.display()))
}

/// One place a name appears, before anything has been mapped.
#[derive(Debug, Clone)]
pub struct RawHit {
    /// Project-relative, as the indexer recorded it.
    pub path: String,
    /// 0-based.
    pub line: u32,
    pub symbol: String,
    pub is_definition: bool,
}

/// Every occurrence whose symbol mentions `query`.
///
/// Substring rather than exact: a SCIP symbol is a long structured string
/// (`scip-typescript npm . . src/`a.ts`/invoiceRef().`), and a person types
/// the name they are looking for, not that.
pub fn occurrences(index: &scip::types::Index, query: &str) -> Vec<RawHit> {
    let mut hits = Vec::new();
    for document in &index.documents {
        for occurrence in &document.occurrences {
            if !occurrence.symbol.contains(query) {
                continue;
            }
            let Some(line) = occurrence.range.first() else {
                continue;
            };
            hits.push(RawHit {
                path: document.relative_path.clone(),
                line: *line as u32,
                symbol: occurrence.symbol.clone(),
                // `SymbolRole::Definition` is bit 1 of a bitset.
                is_definition: occurrence.symbol_roles & 1 != 0,
            });
        }
    }
    hits.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
    hits
}

/// One place a name appears, after it has been explained.
#[derive(Debug, Clone)]
pub struct Hit {
    /// Where a person should be sent — a document when the file is generated,
    /// the file itself when it is not.
    pub path: String,
    /// 0-based line in `path`.
    pub line: u32,
    pub is_definition: bool,
    /// The generated file this came from, when `path` is a document.
    pub through: Option<String>,
}

/// What a lookup found, and what it refused to show.
#[derive(Debug, Default)]
pub struct Found {
    pub hits: Vec<Hit>,
    /// Occurrences in generated files that lineage could not explain.
    ///
    /// Counted and reported rather than dropped in silence: showing fewer
    /// results than exist, with no sign that anything was left out, is its
    /// own kind of lie.
    pub unmapped: usize,
}

/// A generated file's lineage, as the index needs it: byte ranges in the
/// output, and the document byte span each came from.
pub type Lineage = Vec<hickory_lineage::Provenance>;

/// Turn raw occurrences into places a person can be sent.
///
/// `generated` maps an output path to the document that writes it, and
/// `lineage_of` produces that output's provenance and current text. Both are
/// passed in so this is testable without weaving anything.
pub fn explain(
    raw: &[RawHit],
    generated: &HashMap<String, String>,
    mut lineage_of: impl FnMut(&str) -> Option<(Lineage, String, String)>,
) -> Found {
    let mut found = Found::default();
    for hit in raw {
        let Some(_document) = generated.get(&hit.path) else {
            // Not a generated file: it is a file the person edits, and
            // sending them to it is right.
            found.hits.push(Hit {
                path: hit.path.clone(),
                line: hit.line,
                is_definition: hit.is_definition,
                through: None,
            });
            continue;
        };
        // Generated: it must be explained, or it is not shown.
        let Some((lineage, output_text, doc_text)) = lineage_of(&hit.path) else {
            found.unmapped += 1;
            continue;
        };
        let Some(offset) = line_start(&output_text, hit.line) else {
            found.unmapped += 1;
            continue;
        };
        let Some(origin) = lineage
            .iter()
            .find(|p| p.start <= offset && offset < p.end)
            .and_then(|p| p.origin.location())
        else {
            // Synthetic bytes, or an agent's without a document span. There
            // is nowhere to send anybody, so nobody is sent.
            found.unmapped += 1;
            continue;
        };
        let (doc_path, span_start, _) = origin;
        let Some(doc_line) = line_of(&doc_text, span_start) else {
            found.unmapped += 1;
            continue;
        };
        found.hits.push(Hit {
            path: doc_path.to_string(),
            line: doc_line,
            is_definition: hit.is_definition,
            through: Some(hit.path.clone()),
        });
    }
    // An indexer records several occurrences at one position — a definition
    // and the enclosing range of the thing it defines, for instance — and
    // once they are mapped back to a document line they are the same place
    // said twice. A person reading a list of results wants places, not
    // occurrences.
    found.hits.dedup_by(|a, b| {
        a.path == b.path && a.line == b.line && a.is_definition == b.is_definition
    });
    found
}

/// Byte offset of the start of 0-based `line`.
fn line_start(text: &str, line: u32) -> Option<usize> {
    if line == 0 {
        return Some(0);
    }
    let mut seen = 0;
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            seen += 1;
            if seen == line {
                return Some(index + 1);
            }
        }
    }
    None
}

/// 0-based line containing byte `offset`.
fn line_of(text: &str, offset: usize) -> Option<u32> {
    if offset > text.len() {
        return None;
    }
    Some(
        text.as_bytes()[..offset]
            .iter()
            .filter(|b| **b == b'\n')
            .count() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hickory_lineage::{Origin, Provenance};

    fn generated() -> HashMap<String, String> {
        HashMap::from([("out.ts".to_string(), "doc.hick".to_string())])
    }

    #[test]
    fn a_hit_in_a_hand_written_file_is_shown_as_itself() {
        let raw = vec![RawHit {
            path: "src/a.ts".into(),
            line: 3,
            symbol: "x".into(),
            is_definition: false,
        }];
        let found = explain(&raw, &generated(), |_| None);
        assert_eq!(found.hits.len(), 1);
        assert_eq!(found.hits[0].path, "src/a.ts");
        assert!(found.hits[0].through.is_none());
        assert_eq!(found.unmapped, 0);
    }

    #[test]
    fn a_hit_in_a_generated_file_is_shown_at_the_document_that_wrote_it() {
        // The whole point: a reference found in `out.ts` is only useful if it
        // opens the document fragment that contributed those bytes.
        let output = "line zero\nline one\nline two\n";
        let doc = "# A document\n\n<hick:file path=\"out.ts\">\nline zero\nline one\nline two\n</hick:file>\n";
        let body_at = doc.find("line zero").unwrap();
        let lineage = vec![Provenance {
            start: 0,
            end: output.len(),
            origin: Origin::Literal {
                doc_path: "doc.hick".into(),
                span: (body_at, doc.len()),
            },
        }];
        let raw = vec![RawHit {
            path: "out.ts".into(),
            line: 1,
            symbol: "x".into(),
            is_definition: true,
        }];
        let found = explain(&raw, &generated(), |_| {
            Some((lineage.clone(), output.to_string(), doc.to_string()))
        });
        assert_eq!(found.unmapped, 0);
        assert_eq!(found.hits.len(), 1);
        assert_eq!(found.hits[0].path, "doc.hick");
        assert_eq!(found.hits[0].through.as_deref(), Some("out.ts"));
        assert!(found.hits[0].is_definition);
        // Line 3 of the document is where the block's body starts.
        assert_eq!(found.hits[0].line, 3);
    }

    #[test]
    fn a_generated_hit_with_no_lineage_is_not_shown_and_is_counted() {
        // Showing it would send somebody to edit a file that regenerates over
        // them. Dropping it silently would show fewer results than exist with
        // no sign anything was left out.
        let raw = vec![RawHit {
            path: "out.ts".into(),
            line: 1,
            symbol: "x".into(),
            is_definition: false,
        }];
        let found = explain(&raw, &generated(), |_| None);
        assert!(found.hits.is_empty());
        assert_eq!(found.unmapped, 1);
    }

    #[test]
    fn synthetic_bytes_have_nowhere_to_send_anybody() {
        let output = "generated header\nreal line\n";
        let lineage = vec![Provenance {
            start: 0,
            end: output.len(),
            origin: Origin::Synthetic,
        }];
        let raw = vec![RawHit {
            path: "out.ts".into(),
            line: 0,
            symbol: "x".into(),
            is_definition: false,
        }];
        let found = explain(&raw, &generated(), |_| {
            Some((lineage.clone(), output.to_string(), String::new()))
        });
        assert!(found.hits.is_empty());
        assert_eq!(found.unmapped, 1);
    }

    #[test]
    fn line_arithmetic_survives_the_ends() {
        let text = "a\nbb\nccc";
        assert_eq!(line_start(text, 0), Some(0));
        assert_eq!(line_start(text, 1), Some(2));
        assert_eq!(line_start(text, 2), Some(5));
        assert_eq!(line_start(text, 9), None);
        assert_eq!(line_of(text, 0), Some(0));
        assert_eq!(line_of(text, 2), Some(1));
        assert_eq!(line_of(text, 5), Some(2));
        assert_eq!(line_of(text, 999), None);
    }

    #[test]
    fn a_changed_file_is_what_makes_an_index_stale() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.ts"), "original\n").unwrap();
        let built = BuiltFrom {
            at: "2026-08-27T14:02:00Z".into(),
            inputs: HashMap::from([(
                "a.ts".to_string(),
                format!("{:x}", Sha256::digest(b"original\n")),
            )]),
        };
        assert!(built.moved_on(dir.path()).is_empty());
        std::fs::write(dir.path().join("a.ts"), "edited\n").unwrap();
        assert_eq!(built.moved_on(dir.path()), vec!["a.ts".to_string()]);
        // A deleted file has certainly moved on.
        std::fs::remove_file(dir.path().join("a.ts")).unwrap();
        assert_eq!(built.moved_on(dir.path()), vec!["a.ts".to_string()]);
    }
}
