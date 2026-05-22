//! Server state: caches parsed .hick documents, pipeline results, provenance maps,
//! tree-sitter trees, and manages container execution.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use hick_literate::executor::ContainerExecutor;
use hick_exec::node::FileContent;
use hick_flow::provenance::ProvenanceMap;
use hick_lens::language::Language;
use hick_lens::mapping::AstMapping;

/// Configuration for container execution.
#[derive(Clone)]
pub struct ExecConfig {
    pub images_dir: PathBuf,
    pub workdir: PathBuf,
    pub default_image: String,
}

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
    /// Container executor for sandboxed command execution.
    executor: Option<ContainerExecutor>,
    /// Exec configuration (if container execution is enabled).
    exec_config: Option<ExecConfig>,
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
                executor: None,
                exec_config: None,
            })),
        }
    }

    /// Create a new server state with container execution enabled.
    pub fn with_exec_config(config: ExecConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                sources: HashMap::new(),
                files: HashMap::new(),
                provenance_maps: HashMap::new(),
                ast_mappings: HashMap::new(),
                run_summary: None,
                executor: None,
                exec_config: Some(config),
            })),
        }
    }

    /// Get the default container image name.
    pub async fn default_image(&self) -> String {
        let inner = self.inner.lock().await;
        inner
            .exec_config
            .as_ref()
            .map(|c| c.default_image.clone())
            .unwrap_or_else(|| "alpine".to_string())
    }

    /// Whether container execution is configured.
    pub async fn exec_enabled(&self) -> bool {
        let inner = self.inner.lock().await;
        inner.exec_config.is_some()
    }

    /// Ensure a named container is started and booted.
    ///
    /// Creates the `ContainerExecutor` lazily on first call, configuring it
    /// with preopened dirs to mount the working directory at `/workspace`.
    pub async fn ensure_container(&self, name: &str, image: &str) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        let config = inner
            .exec_config
            .as_ref()
            .ok_or_else(|| {
                anyhow::anyhow!("Container execution not configured (missing --images-dir)")
            })?
            .clone();

        if inner.executor.is_none() {
            let mut executor = ContainerExecutor::new(config.images_dir.clone(), HashMap::new())?;
            executor.set_preopened_dirs(vec![(config.workdir.clone(), "/workspace".to_string())]);
            inner.executor = Some(executor);
        }

        let executor = inner.executor.as_mut().unwrap();
        executor.ensure_started(name, image).await
    }

    /// Execute a command in a container and return captured output.
    ///
    /// Ensures the container is booted first. If a timeout is specified,
    /// the command is wrapped with a `timeout` shell command.
    pub async fn exec_command(
        &self,
        container: &str,
        command: &str,
        image: &str,
        timeout: Option<u64>,
    ) -> anyhow::Result<String> {
        self.ensure_container(container, image).await?;

        let mut inner = self.inner.lock().await;
        let executor = inner
            .executor
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("executor not initialized"))?;

        let cmd = if let Some(secs) = timeout {
            format!(
                "cd /workspace && timeout {secs} sh -c {}",
                shell_escape(command)
            )
        } else {
            format!("cd /workspace && {command}")
        };

        executor.execute(container, &cmd).await
    }

    // -----------------------------------------------------------------------
    // Pipeline / lens state methods (unchanged)
    // -----------------------------------------------------------------------

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

        let result = hick_literate::run_pipeline(&source_refs, params).await?;

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

/// Shell-escape a string for use in `sh -c '...'`.
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
