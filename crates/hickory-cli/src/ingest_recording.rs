//! `hick ingest --from recording <doc>`: a cell's recording becomes the
//! document's own — and `hick run` keeps it current.
//!
//! A recording is evidence a document makes about itself. Kept in
//! `.hick-cache/transcripts/` it is a cache, evictable and gitignored, and a
//! clone holds the reference and not the referent — the `from=` mistake
//! (`scaffolded-files-and-derived-edits.md`). Kept in the document, as
//! `<hick:ingested key="…">` inside the cell, it travels with the document,
//! is matched by the same key the cache is, goes stale the same way, and is
//! refreshed by the same run. Axis 1 and axis 2 of
//! `docs/specs/freeform/three-axes.md`.
//!
//! **Nothing is written that the weave cannot reproduce.** The document with
//! the recordings in it is woven against an EMPTY cache before it is written,
//! and every output file must be byte-identical to the weave from the cache.
//! That is the refactor gate (`hick equiv`) applied to an edit the tool
//! itself is making.
//!
//! **A recording that would not survive the parser is refused.** The body is
//! written verbatim — no escaping, ever, by the language's one invariant —
//! so an output containing `<prefix:` would parse as markup. Such a cell's
//! recording stays in the cache and the refusal names the cell.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use hick_literate::cache::{CacheConfig, CacheMode};
use hick_literate::{CellId, RefreshedRecording};

use crate::ingest_exec::element_content_end;
use crate::{DocRun, ExecutorChoice, RunMode};

/// What an ingest did.
#[derive(Debug)]
pub struct RecordingIngest {
    pub doc_path: PathBuf,
    /// Cells whose recording is now in the document.
    pub ingested: Vec<CellId>,
    /// Cells the document already kept, brought up to date.
    pub refreshed: Vec<CellId>,
    /// Cells left in the cache, and why.
    pub refused: Vec<(CellId, String)>,
    /// Cells with nothing to keep, and why: stale, unrecorded, no key, no
    /// transcript.
    pub without: Vec<(CellId, &'static str)>,
}

/// One edit to the document's source: replace `span` with `text`.
struct Edit {
    span: (usize, usize),
    text: String,
}

/// Find every exec cell's (container, line, open-span end, existing
/// recording span) in the document source.
fn cells(doc: &hick_lang::HickDocument, source: &str) -> Vec<CellSite> {
    let prefix = doc.prefix.clone();
    let mut out = Vec::new();
    for exec in doc.all_tags().into_iter().filter(|t| t.name == "exec") {
        let Some(container) = exec.get_attribute("container") else {
            continue;
        };
        let Some(span) = exec.source_span else {
            continue;
        };
        let Some(close_at) = element_content_end(source, span.end, &prefix, "exec") else {
            continue;
        };
        let existing = exec
            .child_tags()
            .find(|c| c.name == "ingested" && c.get_attribute("key").is_some())
            .and_then(|c| {
                let s = c.source_span?;
                let end = element_content_end(source, s.end, &prefix, "ingested")?;
                Some((s.start, end + format!("</{prefix}:ingested>").len()))
            });
        out.push(CellSite {
            cell: CellId::exec(container.trim(), exec.source_line),
            close_at,
            existing,
        });
    }
    out
}

struct CellSite {
    cell: CellId,
    /// Byte offset of the cell's `</prefix:exec>`.
    close_at: usize,
    /// Byte span of an existing `<prefix:ingested key=…>…</prefix:ingested>`.
    existing: Option<(usize, usize)>,
}

/// The element that keeps one recording.
fn recording_element(prefix: &str, key: &str, output: &str, today: &str) -> String {
    let hash = hick_literate::cache::sha256_hex(output);
    format!(
        "<{prefix}:ingested key=\"{key}\" sha256=\"{hash}\" at=\"{today}\">\n{output}</{prefix}:ingested>"
    )
}

/// Whether the body would be read as markup, which the parser cannot be
/// told not to do.
fn would_parse_as_markup(prefix: &str, output: &str) -> bool {
    output.contains(&format!("<{prefix}:")) || output.contains(&format!("</{prefix}:"))
}

/// Apply edits back to front, so earlier offsets stay valid.
fn apply(source: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.span.0));
    let mut out = source.to_string();
    for edit in edits {
        out.replace_range(edit.span.0..edit.span.1, &edit.text);
    }
    out
}

/// The text files a run produced, for the equivalence gate.
fn text_files(run: &DocRun) -> BTreeMap<String, String> {
    run.result
        .files
        .iter()
        .filter_map(|(path, content)| match content {
            hick_exec::node::FileContent::Text(s) => Some((path.clone(), s.clone())),
            _ => None,
        })
        .collect()
}

/// Keep every recorded cell's recording in the document.
pub async fn ingest_recordings(
    doc_path: &Path,
    params: &[(String, String)],
    executor: ExecutorChoice,
    today: &str,
) -> Result<RecordingIngest> {
    let run = crate::run_doc(doc_path, params, RunMode::Weave, executor).await?;
    let source = run.source.clone();
    let prefix = run.doc.prefix.clone();
    let mut report = RecordingIngest {
        doc_path: doc_path.to_path_buf(),
        ingested: Vec::new(),
        refreshed: Vec::new(),
        refused: Vec::new(),
        without: Vec::new(),
    };
    let mut edits = Vec::new();
    for site in cells(&run.doc, &source) {
        let cell = site.cell.clone();
        if run.result.never_run.contains_key(&cell) {
            report.without.push((cell, "unrecorded"));
            continue;
        }
        if run.result.stale.contains_key(&cell) {
            report.without.push((cell, "stale"));
            continue;
        }
        let Some(key) = run.result.keys.get(&cell) else {
            report.without.push((cell, "no key was computed for it"));
            continue;
        };
        let Some(container) = cell.container.clone() else {
            report.without.push((cell, "it has no container"));
            continue;
        };
        let line = cell.source_line;
        let Some(output) = run
            .result
            .transcripts
            .get(&container)
            .and_then(|entries| entries.iter().find(|e| e.source_line == Some(line)))
            .map(|e| e.output.clone())
        else {
            report
                .without
                .push((cell, "no transcript entry carries its line"));
            continue;
        };
        if would_parse_as_markup(&prefix, &output) {
            report.refused.push((
                cell,
                format!(
                    "its output contains `<{prefix}:`, which the parser would read as markup — \
                     a recording is written verbatim, by the language's one invariant, so this \
                     one stays in the cache"
                ),
            ));
            continue;
        }
        let element = recording_element(&prefix, key, &output, today);
        match site.existing {
            Some(span) => {
                if source[span.0..span.1] != element {
                    edits.push(Edit {
                        span,
                        text: element,
                    });
                }
                report.refreshed.push(cell);
            }
            None => {
                // Exactly where the closing tag was, with no whitespace of
                // its own: the cell's text children are its command, the
                // command is in the key, and a newline added here would
                // change the key of the very cell whose recording this is.
                edits.push(Edit {
                    span: (site.close_at, site.close_at),
                    text: element,
                });
                report.ingested.push(cell);
            }
        }
    }
    if edits.is_empty() {
        return Ok(report);
    }
    let next = apply(&source, edits);

    // The gate: the document with its recordings in it must produce every
    // byte the cache-backed weave did, and every recording it now carries
    // must be the one the weave READS — proven by `from_document`, not by
    // hiding the cache, because a cell whose recording was refused (markup
    // in its output) legitimately still answers from the cache.
    let project_dir = doc_path.parent().unwrap_or(Path::new("."));
    let gate_cc = CacheConfig::new(project_dir, CacheMode::Reuse);
    let name = doc_path.to_string_lossy().to_string();
    let gated = hick_literate::run_pipeline_weave(
        &[(name.as_str(), next.as_str())],
        params,
        Some(&gate_cc),
    )
    .await
    .context("weaving the document with its recordings in it")?;
    let before = text_files(&run);
    let after: BTreeMap<String, String> = gated
        .files
        .iter()
        .filter_map(|(path, content)| match content {
            hick_exec::node::FileContent::Text(s) => Some((path.clone(), s.clone())),
            _ => None,
        })
        .collect();
    if before != after {
        let differing: Vec<&String> = before
            .keys()
            .chain(after.keys())
            .filter(|k| before.get(*k) != after.get(*k))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        // Say WHERE, not only that: the first line that differs in the first
        // differing file, so the report is actionable.
        let mut first_diff = String::new();
        if let Some(path) = differing.first() {
            let a = before.get(*path).cloned().unwrap_or_default();
            let b = after.get(*path).cloned().unwrap_or_default();
            for (i, (x, y)) in a.lines().zip(b.lines()).enumerate() {
                if x != y {
                    first_diff =
                        format!("\n  {path} line {}:\n    cache: {x}\n    doc:   {y}", i + 1);
                    break;
                }
            }
            if first_diff.is_empty() {
                first_diff = format!(
                    "\n  {path}: {} line(s) from the cache, {} with the recordings inside",
                    a.lines().count(),
                    b.lines().count()
                );
            }
        }
        let listed = format!(
            "{}{first_diff}",
            differing
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        anyhow::bail!(
            "refusing to write {}: with its recordings inside it, the document weaves differently \
             from the cache ({} file(s) differ: {listed}). Nothing was written. This is the \
             equivalence gate every ingest passes through; it should not fire, and its firing is \
             a bug worth reporting with this document.",
            doc_path.display(),
            differing.len(),
        );
    }
    // Cells are paired by POSITION between the two parses, not by id: a
    // recording written into an earlier cell moves every later cell's line,
    // and the id carries the line.
    let new_doc = hick_lang::parse(&next).context("parsing the document with its recordings")?;
    let old_cells: Vec<CellId> = cells(&run.doc, &source)
        .into_iter()
        .map(|c| c.cell)
        .collect();
    let new_cells: Vec<CellId> = cells(&new_doc, &next).into_iter().map(|c| c.cell).collect();
    let renamed = |old: &CellId| -> Option<CellId> {
        old_cells
            .iter()
            .position(|c| c == old)
            .and_then(|i| new_cells.get(i).cloned())
    };
    let lost: Vec<String> = report
        .ingested
        .iter()
        .chain(report.refreshed.iter())
        .filter(|cell| renamed(cell).is_none_or(|now| !gated.from_document.contains(&now)))
        .map(|cell| cell.to_string())
        .collect();
    if !lost.is_empty() {
        anyhow::bail!(
            "refusing to write {}: {} recording(s) written into the document would not be the \
             ones the weave reads ({}). Nothing was written.",
            doc_path.display(),
            lost.len(),
            lost.join(", ")
        );
    }
    std::fs::write(doc_path, &next).with_context(|| format!("writing {}", doc_path.display()))?;
    Ok(report)
}

/// Bring the recordings a document keeps up to date after a run, for the
/// cells the run re-executed. Only cells that already keep a recording are
/// touched: a run never decides what a document keeps.
pub fn refresh_recordings(
    doc_path: &Path,
    refreshed: &[RefreshedRecording],
    today: &str,
) -> Result<usize> {
    if refreshed.is_empty() {
        return Ok(0);
    }
    let source = std::fs::read_to_string(doc_path)
        .with_context(|| format!("reading {}", doc_path.display()))?;
    let doc =
        hick_lang::parse(&source).with_context(|| format!("parsing {}", doc_path.display()))?;
    let prefix = doc.prefix.clone();
    let sites = cells(&doc, &source);
    let mut edits = Vec::new();
    for rec in refreshed {
        let Some(site) = sites.iter().find(|s| s.cell == rec.cell) else {
            continue;
        };
        let Some(span) = site.existing else {
            continue;
        };
        if would_parse_as_markup(&prefix, &rec.output) {
            continue;
        }
        let element = recording_element(&prefix, &rec.key, &rec.output, today);
        if source[span.0..span.1] != element {
            edits.push(Edit {
                span,
                text: element,
            });
        }
    }
    let count = edits.len();
    if count > 0 {
        std::fs::write(doc_path, apply(&source, edits))
            .with_context(|| format!("writing {}", doc_path.display()))?;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recording_element_round_trips_through_the_parser() {
        let element = recording_element("hick", "abc", "one\ntwo\n", "2026-09-03");
        let doc = format!(
            "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             <hick:exec container=\"c\">printf 'x'\n{element}\n</hick:exec>\n</hick:doc>\n"
        );
        let parsed = hick_lang::parse(&doc).unwrap();
        let recs = hick_literate::document_recordings(std::iter::once(&parsed));
        let (cell, list) = recs.into_iter().next().expect("one cell");
        assert_eq!(cell, CellId::exec("c", 2));
        assert_eq!(list[0].key, "abc");
        assert_eq!(
            list[0].output, "one\ntwo\n",
            "the body is the output, verbatim"
        );
    }

    #[test]
    fn an_output_that_reads_as_markup_is_refused() {
        assert!(would_parse_as_markup("hick", "see <hick:exec> for details"));
        assert!(!would_parse_as_markup("hick", "<div>html is fine</div>"));
    }

    #[test]
    fn edits_apply_back_to_front() {
        let out = apply(
            "abcdef",
            vec![
                Edit {
                    span: (1, 2),
                    text: "B".into(),
                },
                Edit {
                    span: (4, 5),
                    text: "E".into(),
                },
            ],
        );
        assert_eq!(out, "aBcdEf");
    }
}
