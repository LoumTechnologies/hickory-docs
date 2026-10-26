<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/store_config.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d0fc216ac7b5"></a>
## `crates/hick-literate/src/store_config.rs`

<a id="f-d0fc216ac7b5-StoreBackend"></a>
### `StoreBackend`

Store configuration for `hick up` watch mode.
Provides [`StoreConfig`] for JSON-based watch-mode settings:
version store backend, merge API, branch tracking, and multi-stage limits.
Separate from the project-level `_hick.yml` config in [`crate::config`].

<storycode:copy id="f-d0fc216ac7b5-StoreBackend">
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

</storycode:copy>
<a id="f-d0fc216ac7b5-StoreConfig"></a>
### `StoreConfig`

Watch-mode / store configuration.
Loaded from `.md/config.json` if present, otherwise uses defaults.
CLI flags in `hick up` can override any field.

<storycode:copy id="f-d0fc216ac7b5-StoreConfig">
/// Watch-mode / store configuration.
///
/// Loaded from `.md/config.json` if present, otherwise uses defaults.
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

</storycode:copy>
<a id="f-d0fc216ac7b5-default_branch"></a>
### `default_branch`

<storycode:copy id="f-d0fc216ac7b5-default_branch">
fn default_branch() -> String {
    "main".to_string()
}

</storycode:copy>
<a id="f-d0fc216ac7b5-default_max_stages"></a>
### `default_max_stages`

<storycode:copy id="f-d0fc216ac7b5-default_max_stages">
fn default_max_stages() -> usize {
    3
}

</storycode:copy>
<a id="f-d0fc216ac7b5-impl-Default-for-StoreConfig"></a>
### `impl-Default for StoreConfig`

<storycode:copy id="f-d0fc216ac7b5-impl-Default-for-StoreConfig">
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

</storycode:copy>
<a id="f-d0fc216ac7b5-impl-StoreConfig"></a>
### `impl-StoreConfig`

<storycode:copy id="f-d0fc216ac7b5-impl-StoreConfig">
impl StoreConfig {
    /// Load store configuration from `.md/config.json` in the project directory.
    /// Falls back to defaults if the file doesn't exist.
    pub fn load(project_dir: &Path) -> Self {
        let path = project_dir.join(".md/config.json");
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
</storycode:copy>

### Assemble `crates/hick-literate/src/store_config.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/store_config.rs"><storycode:paste select="#f-d0fc216ac7b5-StoreBackend"/><storycode:paste select="#f-d0fc216ac7b5-StoreConfig"/><storycode:paste select="#f-d0fc216ac7b5-default_branch"/><storycode:paste select="#f-d0fc216ac7b5-default_max_stages"/><storycode:paste select="#f-d0fc216ac7b5-impl-Default-for-StoreConfig"/><storycode:paste select="#f-d0fc216ac7b5-impl-StoreConfig"/></storycode:file>

</storycode:doc>
