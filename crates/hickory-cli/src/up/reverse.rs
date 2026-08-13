//! Carrying an edit made in a woven output file back into the document.
//!
//! The mapping itself is `hickory_lineage`: byte ranges in the output, mapped
//! through provenance onto byte ranges in the `.hick` source. What this module
//! adds is the two things a filesystem loop needs and an API call does not —
//! turning "the file on disk is different now" into a set of byte ranges, and
//! deciding what to tell the user when a range has nowhere to go.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use hickory_lineage::{LineageError, OutputEdit, Provenance, SourceEdit};
use similar::{ChangeTag, TextDiff};

/// Diff the bytes we wrote against the bytes now on disk, as a set of
/// output edits.
///
/// Line-level rather than whole-file: a save that touches two functions with
/// generated text between them has to arrive as two edits, because a single
/// span wide enough to cover both would also cover the generated text and be
/// refused. Leading and trailing unchanged lines cost nothing to skip.
pub fn diff_to_edits(old: &str, new: &str) -> Vec<OutputEdit> {
    let diff = TextDiff::from_lines(old, new);

    // Walk the change list accumulating runs. A replacement arrives as a
    // Delete run immediately followed by an Insert run; grouping them into
    // one edit keeps the replaced range tight instead of emitting a deletion
    // and an insertion that `map_edits` would see as two separate spans.
    let mut edits: Vec<OutputEdit> = Vec::new();
    let mut offset = 0usize; // byte offset into `old`
    let mut pending: Option<OutputEdit> = None;

    for change in diff.iter_all_changes() {
        let value = change.value();
        match change.tag() {
            ChangeTag::Equal => {
                if let Some(edit) = pending.take() {
                    edits.push(edit);
                }
                offset += value.len();
            }
            ChangeTag::Delete => {
                let edit = pending.get_or_insert(OutputEdit {
                    start: offset,
                    end: offset,
                    text: String::new(),
                });
                edit.end = offset + value.len();
                offset += value.len();
            }
            ChangeTag::Insert => {
                let edit = pending.get_or_insert(OutputEdit {
                    start: offset,
                    end: offset,
                    text: String::new(),
                });
                edit.text.push_str(value);
            }
        }
    }
    if let Some(edit) = pending.take() {
        edits.push(edit);
    }
    edits
}

/// The document edits one saved output file implies, or the reason there are
/// none.
pub fn source_edits_for_save(
    woven: &str,
    on_disk: &str,
    provenance: &[Provenance],
) -> Result<Vec<SourceEdit>, LineageError> {
    let edits = diff_to_edits(woven, on_disk);
    if edits.is_empty() {
        return Ok(Vec::new());
    }
    hickory_lineage::map_edits(woven, &edits, provenance)
}

/// Apply document edits to the `.hick` files on disk.
///
/// Every document named by an edit is re-read at this moment rather than
/// taken from the loop's memory: the user may have edited the document
/// directly since the last weave, and writing a remembered copy back would
/// silently discard that. If a document has moved on, the edit is refused
/// here and the loop re-weaves instead — the same staleness rule the hashline
/// anchors in `hick doc edit` enforce.
pub fn apply_to_documents(
    edits: &[SourceEdit],
    expected: &HashMap<String, String>,
) -> Result<Vec<String>> {
    let mut sources = HashMap::new();
    for edit in edits {
        if sources.contains_key(&edit.doc_path) {
            continue;
        }
        let current = std::fs::read_to_string(&edit.doc_path)
            .with_context(|| format!("failed to read {}", edit.doc_path))?;
        if let Some(known) = expected.get(&edit.doc_path)
            && *known != current
        {
            anyhow::bail!(
                "{} changed since it was last woven, so this edit was computed \
                 against stale text and was not applied.\n\
                 Nothing was lost: save the file again and the edit will be \
                 recomputed against the current document.",
                edit.doc_path
            );
        }
        sources.insert(edit.doc_path.clone(), current);
    }

    let updated = hickory_lineage::apply_source_edits(&sources, edits)?;
    let mut written = Vec::new();
    for (doc_path, content) in updated {
        super::state::write_atomic(Path::new(&doc_path), &content)?;
        written.push(doc_path);
    }
    written.sort();
    Ok(written)
}

/// What to print when an edit cannot be carried back.
///
/// The user is looking at a file they just typed into and a change that
/// vanished, so the message has to say which bytes had nowhere to go, why,
/// and where to make the edit so that it sticks.
pub fn refusal_message(path: &Path, doc: &Path, woven: &str, error: &LineageError) -> String {
    let where_ = match error {
        LineageError::SyntheticOverlap { start, end } => {
            let (line, _) = line_col(woven, *start);
            let excerpt = woven
                .get(*start..*end)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| {
                    let first = s.lines().next().unwrap_or(s);
                    if first.chars().count() > 60 {
                        format!("{}…", first.chars().take(60).collect::<String>())
                    } else {
                        first.to_string()
                    }
                })
                .unwrap_or_default();
            format!(
                "line {line} is generated text, not text from the document.\n  \
                 Command output, transcripts, interpolated values, and the \
                 separators between blocks\n  have no source to carry an edit \
                 back to.{}",
                if excerpt.is_empty() {
                    String::new()
                } else {
                    format!("\n  The refused text began: {excerpt}")
                }
            )
        }
        LineageError::Conflict(detail) => format!(
            "the edit maps onto the document in more than one place ({detail}).\n  \
             This happens when one copy block is pasted twice and the two copies\n  \
             were edited differently."
        ),
        LineageError::InvalidEdit(detail) => format!("the edit could not be read ({detail})."),
    };

    format!(
        "refused an edit to {}\n  {}\n  \
         The file has been restored. To make this change, edit\n  {}\n  \
         directly — the document is where generated text comes from.",
        path.display(),
        where_,
        doc.display()
    )
}

/// 1-based line and column of a byte offset.
fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let upto = text.get(..offset).unwrap_or(text);
    let line = upto.matches('\n').count() + 1;
    let col = upto.rsplit('\n').next().map(str::len).unwrap_or(0) + 1;
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change_is_no_edits() {
        assert!(diff_to_edits("a\nb\n", "a\nb\n").is_empty());
    }

    #[test]
    fn single_line_replacement_is_one_edit() {
        let edits = diff_to_edits("a\nb\nc\n", "a\nB\nc\n");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].start, 2);
        assert_eq!(edits[0].end, 4);
        assert_eq!(edits[0].text, "B\n");
    }

    /// Two regions edited in one save must arrive as two edits. One span
    /// covering both would also cover the untouched middle line, which is
    /// exactly what gets refused when that middle line is generated.
    #[test]
    fn two_regions_are_two_edits() {
        let edits = diff_to_edits("a\nb\nc\n", "A\nb\nC\n");
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].text, "A\n");
        assert_eq!(edits[1].text, "C\n");
    }

    #[test]
    fn pure_insertion_has_empty_range() {
        let edits = diff_to_edits("a\nc\n", "a\nb\nc\n");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].start, edits[0].end);
        assert_eq!(edits[0].text, "b\n");
    }

    #[test]
    fn pure_deletion_has_empty_text() {
        let edits = diff_to_edits("a\nb\nc\n", "a\nc\n");
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].start, 2);
        assert_eq!(edits[0].end, 4);
        assert!(edits[0].text.is_empty());
    }

    #[test]
    fn line_col_counts_from_one() {
        assert_eq!(line_col("abc\ndef", 0), (1, 1));
        assert_eq!(line_col("abc\ndef", 4), (2, 1));
        assert_eq!(line_col("abc\ndef", 6), (2, 3));
    }
}
