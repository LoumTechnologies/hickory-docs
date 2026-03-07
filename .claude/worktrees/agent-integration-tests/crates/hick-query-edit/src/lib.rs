//! Core logic for hick-query-edit CLI.
//!
//! Provides stateless query, structure, provenance, and edit operations
//! on hick documents and their generated output files. All I/O is JSON
//! for easy consumption from the Python SDK.

pub mod serialize;

use std::fs;
use std::path::Path;

use hick_flow::provenance::ProvenanceMap;
use hick_lens::Language;
use hick_lens::edit::compute_edits;
use hick_lens::mapping::AstMapping;
use hick_lens::selector::Selector;

use serialize::{
    ParseOutput, QueryMatchOutput, SourceEditDto, convert_hick_node, provenance_span_dto,
    query_match_output, source_edit_dto,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("selector error: {0}")]
    Selector(String),
    #[error("language detection failed for {0}")]
    UnknownLanguage(String),
    #[error("AST mapping error: {0}")]
    Mapping(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Parse a .hick file and return JSON-serializable AST.
pub fn parse_hick(path: &str) -> Result<ParseOutput, Error> {
    let source = fs::read_to_string(path)?;
    let doc = hick_lang::parse(&source).map_err(|e| Error::Parse(e.to_string()))?;

    Ok(ParseOutput {
        prefix: doc.prefix.clone(),
        weave_path: doc.weave_path.clone(),
        nodes: doc.nodes.iter().map(convert_hick_node).collect(),
    })
}

/// Query structural elements in a generated output file.
pub fn query(
    selector_str: &str,
    output_path: &str,
    provenance_path: &str,
) -> Result<Vec<QueryMatchOutput>, Error> {
    let provenance = load_provenance(provenance_path)?;
    let output = fs::read_to_string(output_path)?;
    let language = Language::from_path(output_path)
        .ok_or_else(|| Error::UnknownLanguage(output_path.to_string()))?;
    let mapping = AstMapping::build(&output, provenance, language)
        .map_err(|e| Error::Mapping(e.to_string()))?;

    let selector = Selector::parse(selector_str).map_err(|e| Error::Selector(e.to_string()))?;
    let matches = selector.query(&mapping);

    let results: Vec<QueryMatchOutput> = matches
        .iter()
        .map(|m| {
            let origins = mapping.origins_for_element(m.element);
            query_match_output(m.element, m.output_text, origins, Some(output_path))
        })
        .collect();

    Ok(results)
}

/// Get a structural summary of a generated output file.
pub fn structure(output_path: &str, provenance_path: &str) -> Result<String, Error> {
    let provenance = load_provenance(provenance_path)?;
    let output = fs::read_to_string(output_path)?;
    let language = Language::from_path(output_path)
        .ok_or_else(|| Error::UnknownLanguage(output_path.to_string()))?;
    let mapping = AstMapping::build(&output, provenance, language)
        .map_err(|e| Error::Mapping(e.to_string()))?;

    Ok(hick_lens::annotate::structure_summary(&mapping, None))
}

/// Get provenance spans for a line range in a generated output file.
pub fn provenance(
    output_path: &str,
    start_line: usize,
    end_line: usize,
    provenance_path: &str,
) -> Result<Vec<serde_json::Value>, Error> {
    let provenance = load_provenance(provenance_path)?;
    let output = fs::read_to_string(output_path)?;

    let (byte_start, byte_end) = line_range_to_bytes(&output, start_line, end_line);
    let spans = provenance.spans_in_range(byte_start, byte_end);

    Ok(spans.iter().map(|s| provenance_span_dto(s)).collect())
}

/// Compute source edits to replace a selector match with new content.
pub fn compute_edits_cmd(
    output_path: &str,
    selector_str: &str,
    new_content: &str,
    provenance_path: &str,
) -> Result<Vec<SourceEditDto>, Error> {
    let provenance = load_provenance(provenance_path)?;
    let output = fs::read_to_string(output_path)?;
    let language = Language::from_path(output_path)
        .ok_or_else(|| Error::UnknownLanguage(output_path.to_string()))?;
    let mapping = AstMapping::build(&output, provenance.clone(), language)
        .map_err(|e| Error::Mapping(e.to_string()))?;

    let selector = Selector::parse(selector_str).map_err(|e| Error::Selector(e.to_string()))?;
    let matches = selector.query(&mapping);

    let mut all_edits = Vec::new();
    for m in &matches {
        let edits = compute_edits(m.element, new_content, &provenance);
        all_edits.extend(edits.iter().map(source_edit_dto));
    }

    Ok(all_edits)
}

/// Apply computed edits to .hick source files on disk.
pub fn apply_edits(edits_path: &str, hick_dir: &str) -> Result<(), Error> {
    let edits_json = fs::read_to_string(edits_path)?;
    let edits: Vec<SourceEditDto> = serde_json::from_str(&edits_json)?;

    // Group edits by file, sort by source_start descending (apply from end to preserve offsets)
    let mut by_file: std::collections::HashMap<String, Vec<&SourceEditDto>> =
        std::collections::HashMap::new();
    for edit in &edits {
        by_file.entry(edit.file.clone()).or_default().push(edit);
    }

    for (file, mut file_edits) in by_file {
        let file_path = Path::new(hick_dir).join(&file);
        let mut content = fs::read_to_string(&file_path)?;

        // Sort by start position descending so later edits don't shift earlier offsets
        file_edits.sort_by(|a, b| b.source_start.cmp(&a.source_start));

        for edit in file_edits {
            if edit.source_start <= content.len() && edit.source_end <= content.len() {
                content.replace_range(edit.source_start..edit.source_end, &edit.new_content);
            }
        }

        fs::write(&file_path, content)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_provenance(path: &str) -> Result<ProvenanceMap, Error> {
    let json = fs::read_to_string(path)?;
    let provenance: ProvenanceMap = serde_json::from_str(&json)?;
    Ok(provenance)
}

/// Convert 1-based line range to byte offsets.
fn line_range_to_bytes(text: &str, start_line: usize, end_line: usize) -> (usize, usize) {
    let mut current_line = 1usize;
    let mut byte_start = 0usize;
    let mut byte_end = text.len();

    for (i, ch) in text.char_indices() {
        if current_line == start_line && i == byte_start {
            // Already found start, but need to handle first line
        }
        if ch == '\n' {
            current_line += 1;
            if current_line == start_line {
                byte_start = i + 1;
            }
            if current_line == end_line + 1 {
                byte_end = i + 1;
                break;
            }
        }
    }

    if start_line == 1 {
        byte_start = 0;
    }

    (byte_start, byte_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_range_to_bytes_first_line() {
        let text = "line1\nline2\nline3\n";
        let (start, end) = line_range_to_bytes(text, 1, 1);
        assert_eq!(&text[start..end], "line1\n");
    }

    #[test]
    fn line_range_to_bytes_middle() {
        let text = "line1\nline2\nline3\n";
        let (start, end) = line_range_to_bytes(text, 2, 2);
        assert_eq!(&text[start..end], "line2\n");
    }

    #[test]
    fn line_range_to_bytes_multi_line() {
        let text = "line1\nline2\nline3\n";
        let (start, end) = line_range_to_bytes(text, 1, 3);
        assert_eq!(&text[start..end], "line1\nline2\nline3\n");
    }

    #[test]
    fn line_range_to_bytes_last_line_no_newline() {
        let text = "line1\nline2\nline3";
        let (start, end) = line_range_to_bytes(text, 3, 3);
        assert_eq!(&text[start..end], "line3");
    }
}
