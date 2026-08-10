//! Execution result caching for `--cache` and `--freeze` modes.
//!
//! Cache entries are keyed by a SHA-256 hash of everything that affects
//! execution output (image, capabilities, command text, secret names).
//! Results are stored as JSON files in `.hick-cache/transcripts/<container>/`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use hick_token::ContainerCapabilities;

/// A cached execution result, serialized to JSON.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecCacheEntry {
    /// Individual command lines sent to the container.
    pub commands: Vec<String>,
    /// Captured output from the command execution.
    pub output: String,
    /// SHA-256 of the output, for downstream invalidation.
    pub output_hash: String,
}

/// Configuration for the cache subsystem.
#[derive(Debug)]
pub struct CacheConfig {
    /// Root directory for cached transcripts.
    pub cache_dir: PathBuf,
    /// Whether run-wide caching is enabled: results are recorded, and a
    /// matching recording is reused instead of executing.
    ///
    /// This can be `false` while the cache directory is still consulted — a
    /// cell that declares `freeze="true"` reads its recording regardless.
    pub enabled: bool,
    /// The **run-wide default** for freeze, set by `hick run --freeze`.
    ///
    /// Freeze is a per-cell property: a `freeze=` attribute on an exec cell
    /// overrides this value in either direction. A frozen cell is checked
    /// against its recording and never executed, so a missing recording is an
    /// error rather than a reason to run.
    pub freeze: bool,
}

impl CacheConfig {
    /// Create a new cache config rooted at `project_dir/.hick-cache/transcripts`.
    pub fn new(project_dir: &Path, enabled: bool, freeze: bool) -> Self {
        Self {
            cache_dir: project_dir.join(".hick-cache").join("transcripts"),
            enabled,
            freeze,
        }
    }
}

/// Compute the cache key (SHA-256 hex) for an exec step.
pub fn exec_cache_key(
    container_image: &str,
    capabilities_canonical: &str,
    command: &str,
    secret_names: &[&str],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"image:");
    hasher.update(container_image.as_bytes());
    hasher.update(b"\ncaps:");
    hasher.update(capabilities_canonical.as_bytes());
    hasher.update(b"\ncmd:");
    hasher.update(command.trim().as_bytes());
    for s in secret_names {
        hasher.update(b"\nsecret:");
        hasher.update(s.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Look up a cached exec result.
pub fn cache_lookup(
    config: &CacheConfig,
    container: &str,
    key: &str,
) -> Result<Option<ExecCacheEntry>> {
    let path = config.cache_dir.join(container).join(format!("{key}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read cache entry {}", path.display()))?;
    let entry: ExecCacheEntry = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse cache entry {}", path.display()))?;
    Ok(Some(entry))
}

/// Store a result in the cache.
pub fn cache_store(
    config: &CacheConfig,
    container: &str,
    key: &str,
    entry: &ExecCacheEntry,
) -> Result<()> {
    let dir = config.cache_dir.join(container);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create cache dir {}", dir.display()))?;
    let path = dir.join(format!("{key}.json"));
    let data = serde_json::to_string_pretty(entry)?;
    std::fs::write(&path, data)
        .with_context(|| format!("failed to write cache entry {}", path.display()))?;
    Ok(())
}

/// Delete the entire cache directory.
pub fn cache_clear(config: &CacheConfig) -> Result<()> {
    // Walk up from transcripts/ to .hick-cache/
    if let Some(root) = config.cache_dir.parent()
        && root.exists()
    {
        std::fs::remove_dir_all(root)
            .with_context(|| format!("failed to clear cache at {}", root.display()))?;
    }
    Ok(())
}

/// Produce a canonical string representation of capabilities for hashing.
/// Rules are sorted to ensure deterministic output regardless of definition order.
pub fn canonical_caps(
    container_defs: &HashMap<String, ContainerCapabilities>,
    container: &str,
) -> String {
    let Some(caps) = container_defs.get(container) else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    for rule in &caps.network_rules {
        parts.push(format!("net:{rule:?}"));
    }
    for rule in &caps.file_rules {
        parts.push(format!("file:{rule:?}"));
    }
    for rule in &caps.secret_rules {
        parts.push(format!("secret:{}:{}", rule.env_var, rule.secret_name));
    }
    parts.sort();
    parts.join(";")
}

/// Extract secret names for a container (used in cache key computation).
pub fn secret_names_for(
    container_defs: &HashMap<String, ContainerCapabilities>,
    container: &str,
) -> Vec<String> {
    container_defs
        .get(container)
        .map(|caps| {
            caps.secret_rules
                .iter()
                .map(|r| r.secret_name.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// Compute SHA-256 hex of a string.
pub fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_deterministic() {
        let k1 = exec_cache_key("alpine", "net:deny_all", "echo hi", &[]);
        let k2 = exec_cache_key("alpine", "net:deny_all", "echo hi", &[]);
        assert_eq!(k1, k2);
    }

    #[test]
    fn cache_key_changes_with_command() {
        let k1 = exec_cache_key("alpine", "", "echo a", &[]);
        let k2 = exec_cache_key("alpine", "", "echo b", &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_changes_with_image() {
        let k1 = exec_cache_key("alpine", "", "echo hi", &[]);
        let k2 = exec_cache_key("python:3.12", "", "echo hi", &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_changes_with_caps() {
        let k1 = exec_cache_key("alpine", "net:deny_all", "echo hi", &[]);
        let k2 = exec_cache_key("alpine", "", "echo hi", &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_key_changes_with_secrets() {
        let k1 = exec_cache_key("alpine", "", "echo hi", &[]);
        let k2 = exec_cache_key("alpine", "", "echo hi", &["API_KEY"]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_store_and_lookup() {
        let dir = std::env::temp_dir().join("hick-cache-test-store");
        let _ = std::fs::remove_dir_all(&dir);
        let config = CacheConfig::new(&dir, true, false);

        let entry = ExecCacheEntry {
            commands: vec!["echo hi".into()],
            output: "hi\n".into(),
            output_hash: sha256_hex("hi\n"),
        };
        cache_store(&config, "demo", "key1", &entry).unwrap();

        let found = cache_lookup(&config, "demo", "key1").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.output, "hi\n");
        assert_eq!(found.commands, vec!["echo hi"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cache_lookup_returns_none_for_missing() {
        let dir = std::env::temp_dir().join("hick-cache-test-miss");
        let _ = std::fs::remove_dir_all(&dir);
        let config = CacheConfig::new(&dir, true, false);

        let found = cache_lookup(&config, "demo", "nonexistent").unwrap();
        assert!(found.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cache_clear_removes_directory() {
        let dir = std::env::temp_dir().join("hick-cache-test-clear");
        let _ = std::fs::remove_dir_all(&dir);
        let config = CacheConfig::new(&dir, true, false);

        let entry = ExecCacheEntry {
            commands: vec!["echo hi".into()],
            output: "hi\n".into(),
            output_hash: sha256_hex("hi\n"),
        };
        cache_store(&config, "demo", "key1", &entry).unwrap();
        assert!(config.cache_dir.exists());

        cache_clear(&config).unwrap();
        assert!(!config.cache_dir.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn canonical_caps_sorted() {
        let mut caps = ContainerCapabilities::new();
        caps.secret_rules.push(hick_token::SecretRule {
            env_var: "B_KEY".into(),
            secret_name: "b-secret".into(),
        });
        caps.secret_rules.push(hick_token::SecretRule {
            env_var: "A_KEY".into(),
            secret_name: "a-secret".into(),
        });

        let mut defs = HashMap::new();
        defs.insert("test".to_string(), caps);

        let canonical = canonical_caps(&defs, "test");
        // Should be sorted, so a-secret comes before b-secret
        let parts: Vec<&str> = canonical.split(';').collect();
        assert!(parts[0] < parts[1], "parts should be sorted: {canonical}");
    }
}
