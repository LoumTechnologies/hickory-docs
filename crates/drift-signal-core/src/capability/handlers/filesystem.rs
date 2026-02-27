//! Filesystem capability handler — read, write, and list files.
//!
//! Available on any native platform (macOS, iOS sandbox, Android, desktop).
//!
//! Params:
//!   { "action": "read"|"write"|"list", "path": "...", "content": "..." }
//!
//! Returns:
//!   read  → { "content": "..." }
//!   write → { "bytes_written": N }
//!   list  → { "entries": ["file1", "file2", ...] }

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::capability::types::AppCapabilityHandler;

/// Reference capability handler for local filesystem operations.
pub struct FileSystemCapabilityHandler {
    /// Root directory to scope operations within (sandbox).
    root: std::path::PathBuf,
}

impl FileSystemCapabilityHandler {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Resolve a relative path within the root, rejecting traversal attempts.
    fn resolve(&self, path: &str) -> anyhow::Result<std::path::PathBuf> {
        let resolved = self.root.join(path);
        let canonical_root = self.root.canonicalize().unwrap_or_else(|_| self.root.clone());
        let canonical_path = resolved
            .canonicalize()
            .unwrap_or_else(|_| resolved.clone());

        if !canonical_path.starts_with(&canonical_root) {
            anyhow::bail!("Path traversal rejected: {}", path);
        }
        Ok(resolved)
    }
}

#[async_trait]
impl AppCapabilityHandler for FileSystemCapabilityHandler {
    fn cap_type(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "Read, write, and list files within a sandboxed directory"
    }

    fn is_available(&self) -> bool {
        self.root.exists()
    }

    async fn execute(&self, params: Value) -> anyhow::Result<Value> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("read");
        let path = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let resolved = self.resolve(path)?;

        match action {
            "read" => {
                let content = tokio::fs::read_to_string(&resolved).await?;
                Ok(json!({ "content": content }))
            }
            "write" => {
                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter for write"))?;
                if let Some(parent) = resolved.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                let bytes = content.len();
                tokio::fs::write(&resolved, content).await?;
                Ok(json!({ "bytes_written": bytes }))
            }
            "list" => {
                let mut entries = Vec::new();
                let mut dir = tokio::fs::read_dir(&resolved).await?;
                while let Some(entry) = dir.next_entry().await? {
                    if let Some(name) = entry.file_name().to_str() {
                        entries.push(name.to_string());
                    }
                }
                Ok(json!({ "entries": entries }))
            }
            other => anyhow::bail!("Unknown action: {other}"),
        }
    }
}
