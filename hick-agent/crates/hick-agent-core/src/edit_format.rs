//! Hick XML edit format abstraction.
//!
//! Provides pipeline execution, structure querying, provenance lookup, and
//! source edit computation. Other agents can use this to manipulate `.hick`
//! files structurally.

use std::collections::HashMap;
use std::sync::Arc;

use hick_exec::node::FileContent;
use hick_flow::provenance::ProvenanceMap;
use hick_lens::edit::{self, EditConfidence};
use hick_lens::language::Language;
use hick_lens::mapping::AstMapping;
use hick_lens::selector::Selector;

use hick_literate::run_pipeline;

/// A computed source edit (before application).
#[derive(Debug, Clone)]
pub struct SourceEdit {
    /// Path to the `.hick` source file.
    pub file: Arc<str>,
    /// Byte range to replace (start, end).
    pub source_start: usize,
    pub source_end: usize,
    /// New content to insert.
    pub new_content: String,
    /// Confidence level.
    pub confidence: EditConfidence,
}

/// Abstraction for Hick XML edit operations.
///
/// Enables agents to run pipelines, query output structure, and compute
/// source edits without depending on implementation details.
#[async_trait::async_trait]
pub trait HickEditFormat: Send + Sync {
    /// Run the pipeline on in-memory `.hick` sources.
    ///
    /// Returns generated file paths and their content.
    async fn run_pipeline(
        &mut self,
        sources: &[(String, String)],
        params: &[(String, String)],
    ) -> anyhow::Result<PipelineOutput>;

    /// Get structural summary of an output file.
    async fn structure(&self, file: &str) -> anyhow::Result<String>;

    /// Query output files for elements matching a selector.
    async fn query(&self, selector: &str, file: Option<&str>) -> anyhow::Result<Vec<QueryMatch>>;

    /// Get provenance for a line range in an output file.
    async fn provenance(
        &self,
        file: &str,
        start_line: usize,
        end_line: usize,
    ) -> anyhow::Result<String>;

    /// Compute source edits to replace a selector match with new content.
    async fn compute_edits(
        &self,
        file: &str,
        selector: &str,
        new_content: &str,
    ) -> anyhow::Result<Vec<SourceEdit>>;

    /// Apply edits to in-memory source files.
    ///
    /// Takes a map of file path -> content, applies edits (in reverse byte order
    /// per file to preserve indices), returns updated content per file.
    fn apply_edits(
        sources: &HashMap<String, String>,
        edits: &[SourceEdit],
    ) -> HashMap<String, String>;
}

/// Output from a pipeline run.
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    /// Generated file path -> content.
    pub files: HashMap<String, String>,
    /// File path -> provenance map.
    pub provenance_maps: HashMap<String, ProvenanceMap>,
    /// File path -> AST mapping (for supported languages).
    pub ast_mappings: HashMap<String, AstMapping>,
}

/// A single query match.
#[derive(Debug, Clone)]
pub struct QueryMatch {
    pub element_kind: String,
    pub element_name: String,
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub output_text: String,
    pub origins: Vec<String>,
}

/// Concrete implementation of [`HickEditFormat`] using hick and hick-lens.
pub struct HickEditService {
    sources: HashMap<String, String>,
    files: HashMap<String, FileContent>,
    provenance_maps: HashMap<String, ProvenanceMap>,
    ast_mappings: HashMap<String, AstMapping>,
}

impl HickEditService {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            files: HashMap::new(),
            provenance_maps: HashMap::new(),
            ast_mappings: HashMap::new(),
        }
    }

    fn build_ast_mappings(
        files: &HashMap<String, FileContent>,
        provenance_maps: &HashMap<String, ProvenanceMap>,
    ) -> HashMap<String, AstMapping> {
        let mut mappings = HashMap::new();
        for (path, content) in files {
            if let FileContent::Text(text) = content
                && let Some(lang) = Language::from_path(path)
            {
                let prov = provenance_maps.get(path).cloned().unwrap_or_default();
                if let Ok(mapping) = AstMapping::build(text, prov, lang) {
                    mappings.insert(path.clone(), mapping);
                }
            }
        }
        mappings
    }
}

impl Default for HickEditService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl HickEditFormat for HickEditService {
    async fn run_pipeline(
        &mut self,
        sources: &[(String, String)],
        params: &[(String, String)],
    ) -> anyhow::Result<PipelineOutput> {
        let source_refs: Vec<(&str, &str)> = sources
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let result = run_pipeline(&source_refs, params).await?;

        let mut files = HashMap::new();
        for (path, content) in &result.files {
            if let FileContent::Text(s) = content {
                files.insert(path.clone(), s.clone());
            }
        }

        let ast_mappings = Self::build_ast_mappings(&result.files, &result.provenance_maps);

        self.sources = sources
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        self.files = result.files;
        self.provenance_maps = result.provenance_maps.clone();
        self.ast_mappings = ast_mappings.clone();

        Ok(PipelineOutput {
            files,
            provenance_maps: result.provenance_maps,
            ast_mappings,
        })
    }

    async fn structure(&self, file: &str) -> anyhow::Result<String> {
        let mapping = self
            .ast_mappings
            .get(file)
            .ok_or_else(|| anyhow::anyhow!("No AST mapping for '{}'. Run pipeline first.", file))?;
        Ok(hick_lens::annotate::structure_summary(mapping, None))
    }

    async fn query(
        &self,
        selector: &str,
        file_filter: Option<&str>,
    ) -> anyhow::Result<Vec<QueryMatch>> {
        let sel =
            Selector::parse(selector).map_err(|e| anyhow::anyhow!("Invalid selector: {e}"))?;

        let mut results = Vec::new();
        for (path, mapping) in &self.ast_mappings {
            if let Some(f) = file_filter
                && path != f
            {
                continue;
            }

            let matches = sel.query(mapping);
            for m in matches {
                let origins = mapping.origins_for_element(m.element);
                let origin_info: Vec<String> = origins.iter().map(|o| format!("{:?}", o)).collect();
                results.push(QueryMatch {
                    element_kind: format!("{:?}", m.element.kind),
                    element_name: m.element.name.clone(),
                    file: path.clone(),
                    start_line: m.element.start_line + 1,
                    end_line: m.element.end_line + 1,
                    output_text: m.output_text.to_string(),
                    origins: origin_info,
                });
            }
        }

        Ok(results)
    }

    async fn provenance(
        &self,
        file: &str,
        start_line: usize,
        end_line: usize,
    ) -> anyhow::Result<String> {
        let content = self
            .files
            .get(file)
            .and_then(|c| match c {
                FileContent::Text(s) => Some(s.as_str()),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("File '{}' not found", file))?;

        let prov = self.provenance_maps.get(file).cloned().unwrap_or_default();

        let mut line_starts = vec![0];
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }

        let start_line_0 = start_line.saturating_sub(1);
        let end_byte = line_starts.get(end_line).copied().unwrap_or(content.len());
        let start_byte = line_starts.get(start_line_0).copied().unwrap_or(0);

        let spans = prov.spans_in_range(start_byte, end_byte);

        if spans.is_empty() {
            return Ok(format!(
                "No provenance data for lines {}-{} in '{}'",
                start_line, end_line, file
            ));
        }

        let mut lines = Vec::new();
        for span in &spans {
            let text_slice =
                &content[span.output_start.min(content.len())..span.output_end.min(content.len())];
            let preview = if text_slice.len() > 80 {
                format!("{}...", &text_slice[..77])
            } else {
                text_slice.to_string()
            };
            lines.push(format!(
                "[{}-{}] {:?}\n  Text: {:?}",
                span.output_start, span.output_end, span.origin, preview
            ));
        }
        Ok(lines.join("\n"))
    }

    async fn compute_edits(
        &self,
        file: &str,
        selector: &str,
        new_content: &str,
    ) -> anyhow::Result<Vec<SourceEdit>> {
        let mapping = self
            .ast_mappings
            .get(file)
            .ok_or_else(|| anyhow::anyhow!("No AST mapping for '{}'", file))?;

        let sel =
            Selector::parse(selector).map_err(|e| anyhow::anyhow!("Invalid selector: {e}"))?;

        let matches = sel.query(mapping);
        if matches.is_empty() {
            return Ok(vec![]);
        }

        let mut all_edits = Vec::new();
        for m in &matches {
            let edits = edit::compute_edits(m.element, new_content, mapping.provenance());
            for e in edits {
                all_edits.push(SourceEdit {
                    file: e.file,
                    source_start: e.source_start,
                    source_end: e.source_end,
                    new_content: e.new_content,
                    confidence: e.confidence,
                });
            }
        }
        Ok(all_edits)
    }

    fn apply_edits(
        sources: &HashMap<String, String>,
        edits: &[SourceEdit],
    ) -> HashMap<String, String> {
        let mut result = sources.clone();
        for (file, file_edits) in &group_edits_by_file(edits) {
            let Some(content) = result.get(file) else {
                continue;
            };
            let mut content = content.clone();
            // Sort descending by start so indices stay valid
            let mut sorted: Vec<_> = file_edits.iter().collect();
            sorted.sort_by_key(|e| std::cmp::Reverse(e.source_start));
            for e in sorted {
                if e.source_start >= content.len() || e.source_end > content.len() {
                    continue;
                }
                content.replace_range(e.source_start..e.source_end, &e.new_content);
            }
            result.insert(file.clone(), content);
        }
        result
    }
}

fn group_edits_by_file(edits: &[SourceEdit]) -> HashMap<String, Vec<&SourceEdit>> {
    let mut by_file: HashMap<String, Vec<&SourceEdit>> = HashMap::new();
    for e in edits {
        by_file.entry(e.file.to_string()).or_default().push(e);
    }
    by_file
}
