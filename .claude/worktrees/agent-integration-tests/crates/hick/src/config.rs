//! Project configuration via `_hick.yml`.
//!
//! Provides [`HickConfig`] for YAML-based project settings and
//! [`find_config`] for automatic discovery by walking up directories.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Top-level project configuration loaded from `_hick.yml`.
#[derive(Debug, Deserialize, Default)]
pub struct HickConfig {
    /// Ordered list of `.hick` files to process. Supports globs.
    #[serde(default)]
    pub files: Vec<String>,

    /// Default variable values (lowest priority).
    #[serde(default)]
    pub vars: HashMap<String, String>,

    /// Default settings for containers and paths.
    #[serde(default)]
    pub defaults: Defaults,

    /// Base directory for file outputs.
    #[serde(rename = "output-dir")]
    pub output_dir: Option<String>,

    /// Secrets provider configuration.
    #[serde(default)]
    pub secrets: SecretsConfig,
}

/// Default settings for container execution.
#[derive(Debug, Deserialize, Default)]
pub struct Defaults {
    /// Default container image when none specified on a container/exec tag.
    pub image: Option<String>,

    /// Directory containing pre-converted `.wasm` images.
    #[serde(rename = "images-dir")]
    pub images_dir: Option<String>,
}

/// Secrets provider configuration.
#[derive(Debug, Deserialize, Default)]
pub struct SecretsConfig {
    /// Path to the age identity file.
    #[serde(rename = "key-file")]
    pub key_file: Option<String>,

    /// Directory containing age-encrypted secret files.
    #[serde(rename = "secrets-dir")]
    pub secrets_dir: Option<String>,
}

impl HickConfig {
    /// Load config from a YAML file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let config: HickConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(config)
    }

    /// Resolve file globs relative to `base_dir`.
    ///
    /// Within each glob pattern, matches are sorted alphabetically.
    /// The order between different patterns is preserved.
    pub fn resolve_files(&self, base_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut result = Vec::new();
        for pattern in &self.files {
            let full_pattern = base_dir.join(pattern);
            let pattern_str = full_pattern.to_string_lossy();
            let mut matches: Vec<PathBuf> = glob::glob(&pattern_str)
                .with_context(|| format!("invalid glob pattern: {pattern}"))?
                .filter_map(|r| r.ok())
                .collect();
            matches.sort();
            if matches.is_empty() {
                log::warn!("Glob pattern '{pattern}' matched no files");
            }
            result.extend(matches);
        }
        Ok(result)
    }
}

/// Search for `_hick.yml` starting from `start_dir`, walking up to a `.git`
/// boundary or the filesystem root.
pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join("_hick.yml");
        if candidate.is_file() {
            return Some(candidate);
        }
        // Stop at .git boundary
        if dir.join(".git").exists() {
            return None;
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let yaml = "files:\n  - guide.hick\n";
        let config: HickConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.files, vec!["guide.hick"]);
        assert!(config.vars.is_empty());
        assert!(config.defaults.image.is_none());
        assert!(config.output_dir.is_none());
    }

    #[test]
    fn parse_full_config() {
        let yaml = r#"
files:
  - docs/*.hick
  - reference.hick

vars:
  version: "2.0.0"
  env: staging

defaults:
  image: alpine:3.20
  images-dir: /opt/hick/images

output-dir: dist/

secrets:
  key-file: /etc/hick/key.txt
  secrets-dir: /etc/hick/secrets
"#;
        let config: HickConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.files.len(), 2);
        assert_eq!(config.vars.get("version").unwrap(), "2.0.0");
        assert_eq!(config.vars.get("env").unwrap(), "staging");
        assert_eq!(config.defaults.image.as_deref(), Some("alpine:3.20"));
        assert_eq!(
            config.defaults.images_dir.as_deref(),
            Some("/opt/hick/images")
        );
        assert_eq!(config.output_dir.as_deref(), Some("dist/"));
        assert_eq!(
            config.secrets.key_file.as_deref(),
            Some("/etc/hick/key.txt")
        );
        assert_eq!(
            config.secrets.secrets_dir.as_deref(),
            Some("/etc/hick/secrets")
        );
    }

    #[test]
    fn parse_empty_config() {
        let config: HickConfig = serde_yaml::from_str("{}").unwrap();
        assert!(config.files.is_empty());
        assert!(config.vars.is_empty());
    }

    #[test]
    fn find_config_in_current_dir() {
        let dir = std::env::temp_dir().join("hick-config-test-find");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("_hick.yml"), "files: []\n").unwrap();

        let found = find_config(&dir);
        assert_eq!(found, Some(dir.join("_hick.yml")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_config_walks_up() {
        let root = std::env::temp_dir().join("hick-config-test-walk");
        let _ = std::fs::remove_dir_all(&root);
        let sub = root.join("a").join("b");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(root.join("_hick.yml"), "files: []\n").unwrap();

        let found = find_config(&sub);
        assert_eq!(found, Some(root.join("_hick.yml")));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn find_config_stops_at_git() {
        let root = std::env::temp_dir().join("hick-config-test-git");
        let _ = std::fs::remove_dir_all(&root);
        let sub = root.join("project");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::create_dir_all(sub.join(".git")).unwrap();
        // Config is above the .git boundary
        std::fs::write(root.join("_hick.yml"), "files: []\n").unwrap();

        let found = find_config(&sub);
        assert!(found.is_none(), "should not find config above .git");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn find_config_returns_none_when_absent() {
        let dir = std::env::temp_dir().join("hick-config-test-absent");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();

        let found = find_config(&dir);
        assert!(found.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_files_glob() {
        let dir = std::env::temp_dir().join("hick-config-test-glob");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.hick"), "").unwrap();
        std::fs::write(dir.join("b.hick"), "").unwrap();
        std::fs::write(dir.join("c.txt"), "").unwrap();

        let config = HickConfig {
            files: vec!["*.hick".to_string()],
            ..Default::default()
        };
        let files = config.resolve_files(&dir).unwrap();
        assert_eq!(files.len(), 2);
        // Should be sorted alphabetically
        assert!(files[0].ends_with("a.hick"));
        assert!(files[1].ends_with("b.hick"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_files_explicit_order_preserved() {
        let dir = std::env::temp_dir().join("hick-config-test-order");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("first.hick"), "").unwrap();
        std::fs::write(dir.join("second.hick"), "").unwrap();

        let config = HickConfig {
            files: vec!["second.hick".to_string(), "first.hick".to_string()],
            ..Default::default()
        };
        let files = config.resolve_files(&dir).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files[0].ends_with("second.hick"));
        assert!(files[1].ends_with("first.hick"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
