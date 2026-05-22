//! Core logic for hick-query-edit CLI.
//!
//! Provides stateless query, structure, provenance, and edit operations
//! on hick documents and their generated output files. All I/O is JSON
//! for easy consumption from the Python SDK.

pub mod serialize;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use hick_flow::provenance::ProvenanceMap;
use hick_lens::Language;
use hick_lens::edit::compute_edits;
use hick_lens::mapping::AstMapping;
use hick_lens::selector::Selector;
use serde::Serialize;

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
    #[error("pipeline error: {0}")]
    Pipeline(String),
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
// Pipeline registry helpers
// ---------------------------------------------------------------------------

/// Metadata about one pipeline-owned output file.
#[derive(Debug, Serialize)]
pub struct PipelineFileInfo {
    /// Relative output path declared in `<hick:file path="…">`.
    pub output_path: String,
    /// Relative path of the `.hick` source that declares it.
    pub hick_source: String,
    /// Named paste slots (`<hick:paste name="…"/>`) found in that source.
    pub paste_slots: Vec<String>,
}

/// Result of a pipeline write operation.
#[derive(Debug, Serialize)]
pub struct PipelineWriteResult {
    /// The hick:copy element appended to the pipeline.
    pub element: String,
    /// The `.hick` source file that was modified.
    pub hick_source: String,
    /// Paste slot targeted by the copy element.
    pub slot: String,
}

/// Load all `.hick` source content from `hick_dir`.
///
/// Respects `_hick.yml` if present; otherwise discovers `*.hick` recursively.
fn load_hick_sources_from_dir(hick_dir: &str) -> Result<HashMap<String, String>, Error> {
    let base = Path::new(hick_dir);
    let mut sources = HashMap::new();

    let config_path = base.join("_hick.yml");
    let patterns: Vec<String> = if config_path.is_file() {
        #[derive(serde::Deserialize)]
        struct HickConfig {
            #[serde(default)]
            files: Option<Vec<String>>,
        }
        let raw = fs::read_to_string(&config_path)?;
        let cfg: HickConfig = serde_yaml::from_str(&raw)
            .map_err(|e| Error::Parse(e.to_string()))?;
        cfg.files.unwrap_or_else(|| vec!["*.hick".to_string()])
    } else {
        vec!["**/*.hick".to_string()]
    };

    for pattern in patterns {
        let full = base.join(&pattern);
        let pat_str = full.to_string_lossy();
        if let Ok(entries) = glob::glob(&pat_str) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.is_file() {
                    let rel = entry
                        .strip_prefix(base)
                        .unwrap_or(&entry)
                        .to_string_lossy()
                        .replace('\\', "/");
                    sources.insert(rel, fs::read_to_string(&entry)?);
                }
            }
        }
    }

    if sources.is_empty() {
        for entry in walkdir::WalkDir::new(base)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.is_file() && p.extension().is_some_and(|e| e == "hick") {
                let rel = p
                    .strip_prefix(base)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .replace('\\', "/");
                sources.insert(rel, fs::read_to_string(p)?);
            }
        }
    }

    Ok(sources)
}

/// Scan a `.hick` source for `<hick:paste name="…"/>` slots.
fn extract_named_slots(content: &str) -> Vec<String> {
    let mut slots = Vec::new();
    let mut remaining = content;
    while let Some(idx) = remaining.find("hick:paste") {
        remaining = &remaining[idx + "hick:paste".len()..];
        if let Some(name_idx) = remaining.find("name=\"") {
            let after = &remaining[name_idx + "name=\"".len()..];
            if let Some(end) = after.find('"') {
                slots.push(after[..end].to_string());
            }
        }
    }
    slots
}

/// Append a hick XML element before the closing `</…:doc>` tag.
fn append_to_hick_file(hick_path: &Path, element: &str) -> Result<(), Error> {
    let content = fs::read_to_string(hick_path)?;

    let insert_pos = if let Some(pos) = content.rfind(":doc>") {
        let before = &content[..pos];
        before.rfind("</").ok_or_else(|| {
            Error::Pipeline(format!(
                "no closing doc tag found in {}",
                hick_path.display()
            ))
        })?
    } else {
        return Err(Error::Pipeline(format!(
            "no closing doc tag found in {}",
            hick_path.display()
        )));
    };

    let mut new_content = content[..insert_pos].to_string();
    new_content.push('\n');
    new_content.push_str(element);
    new_content.push('\n');
    new_content.push_str(&content[insert_pos..]);

    fs::write(hick_path, new_content)?;
    Ok(())
}

/// List all output files owned by the pipeline.
pub fn pipeline_list_owned(hick_dir: &str) -> Result<Vec<PipelineFileInfo>, Error> {
    let sources = load_hick_sources_from_dir(hick_dir)?;
    let mut result = Vec::new();

    for (hick_path, content) in &sources {
        let slots = extract_named_slots(content);
        if let Ok(doc) = hick_lang::parse(content) {
            for file_tag in doc.find_tags("file") {
                if let Some(output_path) = file_tag.get_attribute("path") {
                    result.push(PipelineFileInfo {
                        output_path: output_path.to_string(),
                        hick_source: hick_path.clone(),
                        paste_slots: slots.clone(),
                    });
                }
            }
        }
    }

    Ok(result)
}

/// List named paste slots available for a pipeline-owned output file.
pub fn pipeline_list_slots(output_path: &str, hick_dir: &str) -> Result<Vec<String>, Error> {
    let owned = pipeline_list_owned(hick_dir)?;
    for info in &owned {
        if info.output_path == output_path {
            return Ok(info.paste_slots.clone());
        }
    }
    Err(Error::Pipeline(format!(
        "output file '{}' is not owned by the pipeline",
        output_path
    )))
}

/// Append a `hick:copy target="<slot>"` element to the pipeline for an existing
/// pipeline-owned file.
///
/// If `slot` is `None`, the first available named slot is used. Returns an
/// error if the file is not pipeline-owned or no slots are available.
pub fn pipeline_write(
    output_path: &str,
    content: &str,
    slot: Option<&str>,
    hick_dir: &str,
) -> Result<PipelineWriteResult, Error> {
    let owned = pipeline_list_owned(hick_dir)?;
    let info = owned
        .iter()
        .find(|i| i.output_path == output_path)
        .ok_or_else(|| {
            Error::Pipeline(format!(
                "output file '{}' is not owned by the pipeline — use a direct write instead",
                output_path
            ))
        })?;

    let slot_name = match slot {
        Some(s) => s.to_string(),
        None => info
            .paste_slots
            .first()
            .cloned()
            .unwrap_or_else(|| "module-footer".to_string()),
    };

    let element = format!(
        "<hick:copy target=\"{slot_name}\">{content}</hick:copy>"
    );

    let hick_path = Path::new(hick_dir).join(&info.hick_source);
    append_to_hick_file(&hick_path, &element)?;

    Ok(PipelineWriteResult {
        element,
        hick_source: info.hick_source.clone(),
        slot: slot_name,
    })
}

/// Append a `hick:file path="<output_path>">` element to the pipeline.
///
/// Appends to the alphabetically first `.hick` source (or the one specified by
/// `_hick.yml`). Returns an error if no `.hick` sources are found.
pub fn pipeline_new_file(
    output_path: &str,
    content: &str,
    hick_dir: &str,
) -> Result<(), Error> {
    let sources = load_hick_sources_from_dir(hick_dir)?;
    if sources.is_empty() {
        return Err(Error::Pipeline(format!(
            "no .hick sources found in '{hick_dir}'"
        )));
    }

    let hick_path_rel = sources
        .keys()
        .min()
        .unwrap()
        .clone();
    let hick_path = Path::new(hick_dir).join(&hick_path_rel);

    let element = format!("<hick:file path=\"{output_path}\">\n{content}\n</hick:file>");
    append_to_hick_file(&hick_path, &element)?;
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
