//! Store configuration for `hick up` watch mode.
//!
//! Provides [`StoreConfig`] for JSON-based watch-mode settings:
//! version store backend, merge API, branch tracking, and multi-stage limits.
//! Separate from the project-level `_hick.yml` config in [`crate::config`].

use std::path::Path;

use serde::Deserialize;

/// Store backend selection for watch mode.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub enum StoreBackend {
    #[default]
    Auto,
    Git,
    Builtin,
}

/// Watch-mode / store configuration.
///
/// Loaded from `.hick/config.json` if present, otherwise uses defaults.
/// CLI flags in `hick up` can override any field.
#[derive(Debug, Deserialize)]
pub struct StoreConfig {
    /// Version store backend.
    #[serde(default)]
    pub store: StoreBackend,

    /// LLM merge API URL for intelligent conflict resolution.
    pub merge_api: Option<String>,

    /// Branch name for snapshot tracking.
    #[serde(default = "default_branch")]
    pub branch: String,

    /// Maximum number of multi-stage pipeline passes.
    #[serde(default = "default_max_stages")]
    pub max_stages: usize,
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_max_stages() -> usize {
    3
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            store: StoreBackend::Auto,
            merge_api: None,
            branch: default_branch(),
            max_stages: default_max_stages(),
        }
    }
}

impl StoreConfig {
    /// Load store configuration from `.hick/config.json` in the project directory.
    /// Falls back to defaults if the file doesn't exist.
    pub fn load(project_dir: &Path) -> Self {
        let path = project_dir.join(".hick/config.json");
        if path.is_file()
            && let Ok(content) = std::fs::read_to_string(&path)
            && let Ok(config) = serde_json::from_str(&content)
        {
            return config;
        }
        Self::default()
    }

    /// Resolve the effective store backend, applying auto-detection logic.
    pub fn resolve_store_backend(&self, project_dir: &Path) -> StoreBackend {
        match self.store {
            StoreBackend::Auto => {
                if project_dir.join(".git").exists() {
                    StoreBackend::Git
                } else {
                    StoreBackend::Builtin
                }
            }
            ref other => other.clone(),
        }
    }
}
