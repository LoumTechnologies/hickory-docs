//! Server state: caches parsed .hick documents, pipeline results, provenance maps,
//! and tree-sitter trees.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use hick_exec::node::FileContent;
use hick_flow::provenance::ProvenanceMap;
use hick_lens::language::Language;
use hick_lens::mapping::AstMapping;

/// Cached state for the MCP server.
#[derive(Clone)]
pub struct ServerState {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    /// Source .hick files: name -> content.
    sources: HashMap<String, String>,
    /// Cached file outputs from last pipeline run.
    files: HashMap<String, FileContent>,
    /// Cached provenance maps from last pipeline run.
    provenance_maps: HashMap<String, ProvenanceMap>,
    /// Cached AST mappings per output file.
    ast_mappings: HashMap<String, AstMapping>,
    /// Summary of last run (file count, etc.).
    run_summary: Option<RunSummary>,
}

/// Summary info from a pipeline run.
#[derive(Clone)]
pub struct RunSummary {
    pub file_count: usize,
    pub file_details: Vec<(String, String, usize)>, // (path, size_str, prov_count)
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                sources: HashMap::new(),
                files: HashMap::new(),
                provenance_maps: HashMap::new(),
                ast_mappings: HashMap::new(),
                run_summary: None,
            })),
        }
    }

    /// Load .hick source files.
    pub async fn load_sources(&self, sources: Vec<(String, String)>) {
        let mut inner = self.inner.lock().await;
        inner.sources.clear();
        for (name, content) in sources {
            inner.sources.insert(name, content);
        }
        // Invalidate cached results
        inner.files.clear();
        inner.provenance_maps.clear();
        inner.ast_mappings.clear();
        inner.run_summary = None;
    }

    /// Run the pipeline and cache results.
    pub async fn run_pipeline(&self, params: &[(String, String)]) -> anyhow::Result<RunSummary> {
        let sources = {
            let inner = self.inner.lock().await;
            inner
                .sources
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>()
        };

        let source_refs: Vec<(&str, &str)> = sources
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let result = hick::run_pipeline(&source_refs, params).await?;

        // Build AST mappings and summary
        let mut mappings = HashMap::new();
        let mut file_details = Vec::new();

        for (path, content) in &result.files {
            let size_str = match content {
                FileContent::Text(s) => format!("{} bytes", s.len()),
                FileContent::Binary(b) => {
                    format!(
                        "{} bytes (binary)",
                        b.to_bytes().map(|v| v.len()).unwrap_or(0)
                    )
                }
            };
            let prov_count = result
                .provenance_maps
                .get(path)
                .map(|p| p.len())
                .unwrap_or(0);
            file_details.push((path.clone(), size_str, prov_count));

            if let FileContent::Text(text) = content
                && let Some(lang) = Language::from_path(path)
            {
                let prov = result
                    .provenance_maps
                    .get(path)
                    .cloned()
                    .unwrap_or_default();
                if let Ok(mapping) = AstMapping::build(text, prov, lang) {
                    mappings.insert(path.clone(), mapping);
                }
            }
        }

        let summary = RunSummary {
            file_count: result.files.len(),
            file_details,
        };

        let mut inner = self.inner.lock().await;
        inner.files = result.files;
        inner.provenance_maps = result.provenance_maps;
        inner.ast_mappings = mappings;
        inner.run_summary = Some(summary.clone());

        Ok(summary)
    }

    /// Get last run summary.
    pub async fn get_run_summary(&self) -> Option<RunSummary> {
        let inner = self.inner.lock().await;
        inner.run_summary.clone()
    }

    /// Get a cached AST mapping for a specific output file.
    pub async fn get_ast_mapping(&self, file: &str) -> Option<AstMapping> {
        let inner = self.inner.lock().await;
        inner.ast_mappings.get(file).cloned()
    }

    /// Get all cached AST mappings.
    pub async fn get_all_ast_mappings(&self) -> HashMap<String, AstMapping> {
        let inner = self.inner.lock().await;
        inner.ast_mappings.clone()
    }

    /// Get provenance map for a specific file.
    pub async fn get_provenance_map(&self, file: &str) -> Option<ProvenanceMap> {
        let inner = self.inner.lock().await;
        inner.provenance_maps.get(file).cloned()
    }

    /// Get file content.
    pub async fn get_file_content(&self, file: &str) -> Option<String> {
        let inner = self.inner.lock().await;
        inner.files.get(file).and_then(|c| match c {
            FileContent::Text(s) => Some(s.clone()),
            _ => None,
        })
    }
}
