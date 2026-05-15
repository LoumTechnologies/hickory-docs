//! Reverse provenance: map output file edits back to `.hick` source.
//!
//! When a generated output file is modified (by the agent or a user in their
//! IDE), this module:
//! 1. Diffs old vs new output content → changed byte ranges
//! 2. Looks up provenance spans for each changed range
//! 3. For `Literal` origins: computes corresponding `.hick` source edits
//! 4. For non-reversible origins: produces actionable error messages
//! 5. Applies reversible edits to `.hick` source files
//!
//! After applying, the pipeline is rerun and the output verified against the
//! intended edit (round-trip check).

use std::collections::HashMap;
use std::sync::Arc;

use hick_flow::node::SourceOrigin;
use hick_flow::provenance::{ProvenanceMap, ProvenanceSpan};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A byte range that changed in an output file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputChange {
    /// Byte offset in the *old* output where the change starts.
    pub old_start: usize,
    /// Byte offset in the *old* output where the change ends (exclusive).
    pub old_end: usize,
    /// The new content that replaces `old[old_start..old_end]`.
    pub new_content: String,
}

/// A computed edit to a `.hick` source file.
#[derive(Debug, Clone)]
pub struct ReverseEdit {
    /// The `.hick` source file to edit.
    pub file: Arc<str>,
    /// Start byte offset in the source (inclusive).
    pub source_start: usize,
    /// End byte offset in the source (exclusive).
    pub source_end: usize,
    /// Replacement content.
    pub new_content: String,
}

/// Why a particular output change cannot be reverse-mapped.
#[derive(Debug, Clone)]
pub struct ReverseError {
    /// The output file that was edited.
    pub output_file: String,
    /// Byte range in the old output.
    pub old_start: usize,
    pub old_end: usize,
    /// Human-readable explanation with provenance context.
    pub message: String,
}

/// Result of attempting to reverse-map output changes.
#[derive(Debug, Clone)]
pub struct ReverseResult {
    /// Edits that can be applied to `.hick` source files.
    pub edits: Vec<ReverseEdit>,
    /// Changes that could not be reversed, with explanations.
    pub errors: Vec<ReverseError>,
}

impl ReverseResult {
    /// Whether all changes were successfully reversed (no errors).
    pub fn is_fully_reversible(&self) -> bool {
        self.errors.is_empty() && !self.edits.is_empty()
    }
}

/// Outcome of a round-trip verification after applying reverse edits.
#[derive(Debug, Clone)]
pub enum VerifyResult {
    /// Pipeline output matches the intended edit exactly.
    Match,
    /// Pipeline output differs from intended edit.
    Mismatch {
        expected_len: usize,
        actual_len: usize,
        first_diff_byte: Option<usize>,
    },
    /// Pipeline failed to run after applying edits.
    PipelineError(String),
}

// ---------------------------------------------------------------------------
// Diff: find changed byte ranges between old and new output
// ---------------------------------------------------------------------------

/// Compute the changed byte ranges between two versions of an output file.
///
/// Uses a simple approach: find the common prefix and suffix, then treat
/// the middle as a single change. This works well for typical edits (insert,
/// delete, replace a contiguous region). Multiple disjoint edits are merged
/// into one range.
pub fn diff_output(old: &str, new: &str) -> Vec<OutputChange> {
    if old == new {
        return Vec::new();
    }

    let old_bytes = old.as_bytes();
    let new_bytes = new.as_bytes();

    let prefix_len = old_bytes
        .iter()
        .zip(new_bytes.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let old_remaining = old_bytes.len() - prefix_len;
    let new_remaining = new_bytes.len() - prefix_len;
    let suffix_len = old_bytes[prefix_len..]
        .iter()
        .rev()
        .zip(new_bytes[prefix_len..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(old_remaining)
        .min(new_remaining);

    let old_end = old_bytes.len() - suffix_len;
    let new_end = new_bytes.len() - suffix_len;

    if prefix_len == old_end && prefix_len == new_end {
        return Vec::new();
    }

    vec![OutputChange {
        old_start: prefix_len,
        old_end,
        new_content: new[prefix_len..new_end].to_string(),
    }]
}

// ---------------------------------------------------------------------------
// Reverse mapping: output changes → source edits
// ---------------------------------------------------------------------------

/// Map output changes back to `.hick` source edits using provenance.
///
/// For each `OutputChange`, looks up the overlapping provenance spans.
/// - `Literal` origins produce `ReverseEdit`s (byte ranges in `.hick` source)
/// - `Exec`, `Paste`, `Variable`, `Synthetic` origins produce `ReverseError`s
///   with actionable messages.
pub fn reverse_map(
    output_file: &str,
    changes: &[OutputChange],
    provenance: &ProvenanceMap,
) -> ReverseResult {
    let mut edits = Vec::new();
    let mut errors = Vec::new();

    for change in changes {
        let spans = provenance.spans_in_range(change.old_start, change.old_end);

        if spans.is_empty() {
            errors.push(ReverseError {
                output_file: output_file.to_string(),
                old_start: change.old_start,
                old_end: change.old_end,
                message: format!(
                    "No provenance data for bytes {}..{} in {}. \
                     This region may have been added after the last pipeline run.",
                    change.old_start, change.old_end, output_file
                ),
            });
            continue;
        }

        let mut all_literal = true;
        let mut literal_file: Option<Arc<str>> = None;

        for span in &spans {
            match &span.origin {
                SourceOrigin::Literal { file, .. } => {
                    if let Some(ref prev_file) = literal_file {
                        if prev_file.as_ref() != file.as_ref() {
                            errors.push(ReverseError {
                                output_file: output_file.to_string(),
                                old_start: change.old_start,
                                old_end: change.old_end,
                                message: format!(
                                    "Cannot reverse-apply edit to {} bytes {}..{}.\n\
                                     These bytes span multiple source files ({} and {}).\n\
                                     Edit each source file individually instead.",
                                    output_file, change.old_start, change.old_end, prev_file, file
                                ),
                            });
                            all_literal = false;
                            break;
                        }
                    } else {
                        literal_file = Some(Arc::clone(file));
                    }
                }
                other => {
                    errors.push(non_reversible_error(output_file, change, span, other));
                    all_literal = false;
                }
            }
        }

        if !all_literal {
            continue;
        }

        if let Some(file) = literal_file
            && let Some(edit) = compute_literal_reverse_edit(&file, change, &spans)
        {
            edits.push(edit);
        }
    }

    edits.sort_by(|a, b| b.source_start.cmp(&a.source_start));

    ReverseResult { edits, errors }
}

/// Compute a `ReverseEdit` for a change that maps entirely to `Literal` spans
/// from a single source file.
fn compute_literal_reverse_edit(
    file: &Arc<str>,
    change: &OutputChange,
    spans: &[&ProvenanceSpan],
) -> Option<ReverseEdit> {
    let mut source_start = usize::MAX;
    let mut source_end = 0;

    for span in spans {
        if let SourceOrigin::Literal { span: src_span, .. } = &span.origin {
            let overlap_start = change.old_start.max(span.output_start);
            let overlap_end = change.old_end.min(span.output_end);

            let offset_in_span_start = overlap_start - span.output_start;
            let offset_in_span_end = overlap_end - span.output_start;

            let mapped_start = src_span.start + offset_in_span_start;
            let mapped_end = src_span.start + offset_in_span_end;

            source_start = source_start.min(mapped_start);
            source_end = source_end.max(mapped_end);
        }
    }

    if source_start == usize::MAX {
        return None;
    }

    Some(ReverseEdit {
        file: Arc::clone(file),
        source_start,
        source_end,
        new_content: change.new_content.clone(),
    })
}

/// Generate an actionable error for a non-reversible provenance origin.
fn non_reversible_error(
    output_file: &str,
    change: &OutputChange,
    _span: &ProvenanceSpan,
    origin: &SourceOrigin,
) -> ReverseError {
    let message = match origin {
        SourceOrigin::Exec {
            container,
            tag_line,
        } => {
            format!(
                "Cannot reverse-apply edit to {} bytes {}..{}.\n\
                 These bytes are generated by <hick:exec container=\"{}\"> (source line {}).\n\
                 To change this output, modify the code generator input or template.",
                output_file, change.old_start, change.old_end, container, tag_line
            )
        }
        SourceOrigin::Paste { selector } => {
            format!(
                "Cannot reverse-apply edit to {} bytes {}..{}.\n\
                 These bytes come from <hick:paste select=\"{}\">.\n\
                 To modify this content, edit the source <hick:cut> or <hick:copy> block.",
                output_file, change.old_start, change.old_end, selector
            )
        }
        SourceOrigin::Variable { name } => {
            format!(
                "Cannot reverse-apply edit to {} bytes {}..{}.\n\
                 These bytes come from variable \"{}\".\n\
                 To change this value, update the <hick:var name=\"{}\"> definition.",
                output_file, change.old_start, change.old_end, name, name
            )
        }
        SourceOrigin::Synthetic => {
            format!(
                "Cannot reverse-apply edit to {} bytes {}..{}.\n\
                 These bytes were programmatically generated (no source location).\n\
                 Edit the .hick pipeline logic that produces this content.",
                output_file, change.old_start, change.old_end
            )
        }
        SourceOrigin::Script { tag_line } => {
            format!(
                "Cannot reverse-apply edit to {} bytes {}..{}.\n\
                 These bytes are generated by <hick:script> (source line {}).\n\
                 To change this output, modify the script source.",
                output_file, change.old_start, change.old_end, tag_line
            )
        }
        SourceOrigin::Literal { .. } => {
            format!(
                "Unexpected: Literal origin treated as non-reversible for {} bytes {}..{}.",
                output_file, change.old_start, change.old_end
            )
        }
    };

    ReverseError {
        output_file: output_file.to_string(),
        old_start: change.old_start,
        old_end: change.old_end,
        message,
    }
}

// ---------------------------------------------------------------------------
// Apply reverse edits to source content
// ---------------------------------------------------------------------------

/// Apply reverse edits to in-memory `.hick` source content.
///
/// Edits must be sorted by `source_start` descending (as returned by
/// `reverse_map`) so that applying from end to start preserves offsets.
///
/// Returns the updated source map.
pub fn apply_reverse_edits(
    sources: &HashMap<String, String>,
    edits: &[ReverseEdit],
) -> HashMap<String, String> {
    let mut result = sources.clone();

    for edit in edits {
        let file_key = edit.file.as_ref();
        if let Some(content) = result.get_mut(file_key)
            && edit.source_start <= content.len()
            && edit.source_end <= content.len()
        {
            let mut new = String::with_capacity(
                content.len() - (edit.source_end - edit.source_start) + edit.new_content.len(),
            );
            new.push_str(&content[..edit.source_start]);
            new.push_str(&edit.new_content);
            new.push_str(&content[edit.source_end..]);
            *content = new;
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Round-trip verification
// ---------------------------------------------------------------------------

/// Verify that re-running the pipeline after applying reverse edits produces
/// the expected output for the given file.
pub fn verify_round_trip(
    output_file: &str,
    expected_content: &str,
    actual_pipeline_output: &HashMap<String, String>,
) -> VerifyResult {
    match actual_pipeline_output.get(output_file) {
        None => VerifyResult::PipelineError(format!(
            "Pipeline did not produce output file: {output_file}"
        )),
        Some(actual) => {
            if actual == expected_content {
                VerifyResult::Match
            } else {
                let first_diff = actual
                    .bytes()
                    .zip(expected_content.bytes())
                    .position(|(a, b)| a != b);
                VerifyResult::Mismatch {
                    expected_len: expected_content.len(),
                    actual_len: actual.len(),
                    first_diff_byte: first_diff,
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use hick_flow::provenance::ProvenanceSpan;
    use std::sync::Arc;

    fn literal_origin(file: &str, start: usize, end: usize, line: usize) -> SourceOrigin {
        SourceOrigin::Literal {
            file: Arc::from(file),
            span: hick_lang::SourceSpan::new(start, end, line, 0),
        }
    }

    #[test]
    fn diff_identical() {
        let changes = diff_output("hello world", "hello world");
        assert!(changes.is_empty());
    }

    #[test]
    fn diff_simple_replacement() {
        let changes = diff_output("hello world", "hello rust!");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_start, 6);
        assert_eq!(changes[0].old_end, 11);
        assert_eq!(changes[0].new_content, "rust!");
    }

    #[test]
    fn diff_insertion() {
        let changes = diff_output("ab", "aXb");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_start, 1);
        assert_eq!(changes[0].old_end, 1);
        assert_eq!(changes[0].new_content, "X");
    }

    #[test]
    fn diff_deletion() {
        let changes = diff_output("aXb", "ab");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_start, 1);
        assert_eq!(changes[0].old_end, 2);
        assert_eq!(changes[0].new_content, "");
    }

    #[test]
    fn diff_prefix_change() {
        let changes = diff_output("hello", "jello");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_start, 0);
        assert_eq!(changes[0].old_end, 1);
        assert_eq!(changes[0].new_content, "j");
    }

    #[test]
    fn diff_complete_replacement() {
        let changes = diff_output("abc", "xyz");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_start, 0);
        assert_eq!(changes[0].old_end, 3);
        assert_eq!(changes[0].new_content, "xyz");
    }

    #[test]
    fn reverse_literal_single_span() {
        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 11,
            origin: literal_origin("main.hick", 100, 111, 5),
        });

        let changes = vec![OutputChange {
            old_start: 6,
            old_end: 11,
            new_content: "rust!".into(),
        }];

        let result = reverse_map("src/lib.rs", &changes, &prov);
        assert!(result.errors.is_empty());
        assert_eq!(result.edits.len(), 1);
        assert_eq!(result.edits[0].file.as_ref(), "main.hick");
        assert_eq!(result.edits[0].source_start, 106);
        assert_eq!(result.edits[0].source_end, 111);
        assert_eq!(result.edits[0].new_content, "rust!");
    }

    #[test]
    fn reverse_literal_multiple_spans_same_file() {
        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: literal_origin("main.hick", 100, 105, 5),
        });
        prov.push(ProvenanceSpan {
            output_start: 5,
            output_end: 10,
            origin: literal_origin("main.hick", 105, 110, 5),
        });

        let changes = vec![OutputChange {
            old_start: 3,
            old_end: 8,
            new_content: "XY".into(),
        }];

        let result = reverse_map("out.rs", &changes, &prov);
        assert!(result.errors.is_empty());
        assert_eq!(result.edits.len(), 1);
        assert_eq!(result.edits[0].source_start, 103);
        assert_eq!(result.edits[0].source_end, 108);
        assert_eq!(result.edits[0].new_content, "XY");
    }

    #[test]
    fn reverse_exec_produces_error() {
        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 10,
            origin: SourceOrigin::Exec {
                container: Arc::from("codegen"),
                tag_line: 8,
            },
        });

        let changes = vec![OutputChange {
            old_start: 2,
            old_end: 5,
            new_content: "new".into(),
        }];

        let result = reverse_map("src/lib.rs", &changes, &prov);
        assert!(result.edits.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("hick:exec"));
        assert!(result.errors[0].message.contains("codegen"));
    }

    #[test]
    fn reverse_paste_produces_error() {
        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 10,
            origin: SourceOrigin::Paste {
                selector: Arc::from("#utils"),
            },
        });

        let changes = vec![OutputChange {
            old_start: 0,
            old_end: 5,
            new_content: "xxx".into(),
        }];

        let result = reverse_map("src/lib.rs", &changes, &prov);
        assert!(result.edits.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("hick:paste"));
        assert!(result.errors[0].message.contains("#utils"));
    }

    #[test]
    fn reverse_variable_produces_error() {
        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: SourceOrigin::Variable {
                name: Arc::from("version"),
            },
        });

        let changes = vec![OutputChange {
            old_start: 0,
            old_end: 5,
            new_content: "2.0".into(),
        }];

        let result = reverse_map("out.rs", &changes, &prov);
        assert!(result.edits.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("version"));
        assert!(result.errors[0].message.contains("hick:var"));
    }

    #[test]
    fn reverse_synthetic_produces_error() {
        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: SourceOrigin::Synthetic,
        });

        let changes = vec![OutputChange {
            old_start: 0,
            old_end: 3,
            new_content: "abc".into(),
        }];

        let result = reverse_map("out.rs", &changes, &prov);
        assert!(result.edits.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(
            result.errors[0]
                .message
                .contains("programmatically generated")
        );
    }

    #[test]
    fn reverse_no_provenance_data() {
        let prov = ProvenanceMap::new();

        let changes = vec![OutputChange {
            old_start: 0,
            old_end: 5,
            new_content: "test".into(),
        }];

        let result = reverse_map("out.rs", &changes, &prov);
        assert!(result.edits.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("No provenance data"));
    }

    #[test]
    fn reverse_mixed_literal_and_exec() {
        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 5,
            origin: literal_origin("main.hick", 100, 105, 5),
        });
        prov.push(ProvenanceSpan {
            output_start: 5,
            output_end: 10,
            origin: SourceOrigin::Exec {
                container: Arc::from("codegen"),
                tag_line: 8,
            },
        });

        let changes = vec![OutputChange {
            old_start: 3,
            old_end: 8,
            new_content: "xxxxx".into(),
        }];

        let result = reverse_map("src/lib.rs", &changes, &prov);
        assert!(result.edits.is_empty());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn apply_single_edit() {
        let mut sources = HashMap::new();
        sources.insert("main.hick".to_string(), "hello world!".to_string());

        let edits = vec![ReverseEdit {
            file: Arc::from("main.hick"),
            source_start: 6,
            source_end: 11,
            new_content: "rust".into(),
        }];

        let result = apply_reverse_edits(&sources, &edits);
        assert_eq!(result["main.hick"], "hello rust!");
    }

    #[test]
    fn apply_multiple_edits_same_file() {
        let mut sources = HashMap::new();
        sources.insert("main.hick".to_string(), "aaabbbccc".to_string());

        let edits = vec![
            ReverseEdit {
                file: Arc::from("main.hick"),
                source_start: 6,
                source_end: 9,
                new_content: "CCC".into(),
            },
            ReverseEdit {
                file: Arc::from("main.hick"),
                source_start: 0,
                source_end: 3,
                new_content: "AAA".into(),
            },
        ];

        let result = apply_reverse_edits(&sources, &edits);
        assert_eq!(result["main.hick"], "AAAbbbCCC");
    }

    #[test]
    fn verify_match() {
        let mut output = HashMap::new();
        output.insert("src/lib.rs".to_string(), "fn main() {}".to_string());
        let result = verify_round_trip("src/lib.rs", "fn main() {}", &output);
        assert!(matches!(result, VerifyResult::Match));
    }

    #[test]
    fn verify_mismatch() {
        let mut output = HashMap::new();
        output.insert("src/lib.rs".to_string(), "fn main() { x }".to_string());
        let result = verify_round_trip("src/lib.rs", "fn main() { y }", &output);
        match result {
            VerifyResult::Mismatch {
                first_diff_byte, ..
            } => {
                assert_eq!(first_diff_byte, Some(12));
            }
            other => panic!("Expected Mismatch, got {:?}", other),
        }
    }

    #[test]
    fn verify_missing_file() {
        let output = HashMap::new();
        let result = verify_round_trip("src/lib.rs", "content", &output);
        assert!(matches!(result, VerifyResult::PipelineError(_)));
    }

    #[test]
    fn end_to_end_literal_edit() {
        let prefix = "<hick:doc><hick:file path=\"src/lib.rs\">";
        let old_output = "fn greet() { println!(\"Hello\"); }";
        let new_output = "fn greet() { println!(\"Hi\"); }";
        let source_offset = prefix.len();

        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: old_output.len(),
            origin: literal_origin(
                "main.hick",
                source_offset,
                source_offset + old_output.len(),
                3,
            ),
        });

        let changes = diff_output(old_output, new_output);
        assert_eq!(changes.len(), 1);

        let result = reverse_map("src/lib.rs", &changes, &prov);
        assert!(result.is_fully_reversible());
        assert_eq!(result.edits.len(), 1);

        let mut sources = HashMap::new();
        let source = format!("{prefix}{old_output}</hick:file></hick:doc>",);
        sources.insert("main.hick".to_string(), source);

        let updated = apply_reverse_edits(&sources, &result.edits);
        assert!(updated["main.hick"].contains("Hi"));
        assert!(!updated["main.hick"].contains("Hello"));
    }
}
