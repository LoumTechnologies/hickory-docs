//! Project-local MCP server discovery via `mcp.json` / `.mcp.json`.
//!
//! Augments any user-level MCP configuration with servers declared in the
//! project directory. This follows the de-facto cross-agent convention used
//! by Claude Code, Cursor, and others.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::mcp_config::{McpServerConfig, McpServerTrust, McpTransportKind};

// ---------------------------------------------------------------------------
// Raw JSON schema
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawProjectMcpServer {
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawProjectMcpConfig {
    #[serde(rename = "mcpServers", default)]
    mcp_servers: HashMap<String, RawProjectMcpServer>,
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single MCP server entry discovered from `mcp.json` / `.mcp.json`.
pub struct ProjectMcpServerEntry {
    /// Server name (key in `mcpServers` object).
    pub name: String,
    /// Full command vector `[command, ...args]` for stdio servers.
    pub command: Option<Vec<String>>,
    /// Resolved environment variables (after `${VAR}` expansion).
    pub env: HashMap<String, String>,
    /// Server URL for HTTP servers.
    pub url: Option<String>,
}

/// MCP server configuration loaded from a project's `mcp.json` / `.mcp.json`.
pub struct ProjectMcpConfig {
    /// Servers declared in the project file.
    pub servers: Vec<ProjectMcpServerEntry>,
}

impl ProjectMcpConfig {
    /// Detect and load `mcp.json` / `.mcp.json` at `project_dir`.
    ///
    /// Checks `.mcp.json` first, then `mcp.json`. Returns `Ok(None)` if
    /// neither file is present. Logs a warning and returns `Ok(None)` on
    /// malformed JSON rather than propagating the error.
    pub fn load(project_dir: &Path) -> anyhow::Result<Option<Self>> {
        let candidates = [".mcp.json", "mcp.json"];
        for name in candidates {
            let path = project_dir.join(name);
            if !path.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            match serde_json::from_str::<RawProjectMcpConfig>(&content) {
                Ok(raw) => return Ok(Some(Self::from_raw(raw))),
                Err(e) => {
                    log::warn!(
                        "mcp_project_config: malformed JSON in '{}': {e} — skipping",
                        path.display()
                    );
                    return Ok(None);
                }
            }
        }
        Ok(None)
    }

    fn from_raw(raw: RawProjectMcpConfig) -> Self {
        let mut servers: Vec<ProjectMcpServerEntry> = raw
            .mcp_servers
            .into_iter()
            .map(|(name, entry)| {
                let command = entry.command.map(|cmd| {
                    let mut full = vec![cmd];
                    full.extend(entry.args);
                    full
                });
                let env = entry
                    .env
                    .into_iter()
                    .map(|(k, v)| (k, interpolate_env(&v)))
                    .collect();
                ProjectMcpServerEntry {
                    name,
                    command,
                    env,
                    url: entry.url,
                }
            })
            .collect();
        // Deterministic ordering.
        servers.sort_by(|a, b| a.name.cmp(&b.name));
        Self { servers }
    }

    /// Convert to [`McpServerConfig`] entries suitable for merging into
    /// [`crate::mcp_config::McpServersConfig`].
    ///
    /// Entries with neither a `command` nor a `url` are skipped.
    pub fn to_server_configs(&self) -> Vec<McpServerConfig> {
        self.servers
            .iter()
            .filter_map(|entry| {
                if entry.command.is_some() {
                    Some(McpServerConfig {
                        name: entry.name.clone(),
                        transport: McpTransportKind::Stdio,
                        command: entry.command.clone(),
                        url: None,
                        trust: McpServerTrust::Prompt,
                        sandbox: None,
                    })
                } else if entry.url.is_some() {
                    Some(McpServerConfig {
                        name: entry.name.clone(),
                        transport: McpTransportKind::Http,
                        command: None,
                        url: entry.url.clone(),
                        trust: McpServerTrust::Prompt,
                        sandbox: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Replace `${VAR}` placeholders with the corresponding environment variable.
/// Unknown variables expand to an empty string.
fn interpolate_env(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find("${") {
        result.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        if let Some(end) = rest.find('}') {
            let var_name = &rest[..end];
            result.push_str(&std::env::var(var_name).unwrap_or_default());
            rest = &rest[end + 1..];
        } else {
            // Malformed placeholder — emit literally.
            result.push_str("${");
        }
    }
    result.push_str(rest);
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let result = ProjectMcpConfig::load(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn discovers_mcp_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("mcp.json"),
            r#"{"mcpServers":{"my-server":{"command":"npx","args":["-y","@some/pkg"]}}}"#,
        )
        .unwrap();

        let config = ProjectMcpConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(config.servers.len(), 1);
        let server = &config.servers[0];
        assert_eq!(server.name, "my-server");
        assert_eq!(
            server.command.as_deref().unwrap(),
            ["npx", "-y", "@some/pkg"]
        );
    }

    #[test]
    fn prefers_dotmcp_json_over_mcp_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".mcp.json"),
            r#"{"mcpServers":{"dot-server":{"command":"node","args":[]}}}"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("mcp.json"),
            r#"{"mcpServers":{"plain-server":{"command":"python","args":[]}}}"#,
        )
        .unwrap();

        let config = ProjectMcpConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(config.servers[0].name, "dot-server");
    }

    #[test]
    fn env_interpolation() {
        // SAFETY: test-only, single-threaded.
        unsafe { std::env::set_var("TEST_MCP_VAR_39", "resolved-value") };

        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("mcp.json"),
            r#"{"mcpServers":{"s":{"command":"cmd","args":[],"env":{"KEY":"${TEST_MCP_VAR_39}"}}}}"#,
        )
        .unwrap();

        let config = ProjectMcpConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(
            config.servers[0].env.get("KEY").map(|s| s.as_str()),
            Some("resolved-value")
        );

        unsafe { std::env::remove_var("TEST_MCP_VAR_39") };
    }

    #[test]
    fn malformed_json_returns_none_with_warning() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("mcp.json"), "not json at all").unwrap();
        let result = ProjectMcpConfig::load(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn to_server_configs_stdio() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("mcp.json"),
            r#"{"mcpServers":{"srv":{"command":"npx","args":["pkg"]}}}"#,
        )
        .unwrap();
        let config = ProjectMcpConfig::load(dir.path()).unwrap().unwrap();
        let server_configs = config.to_server_configs();
        assert_eq!(server_configs.len(), 1);
        assert_eq!(server_configs[0].transport, McpTransportKind::Stdio);
        assert_eq!(
            server_configs[0].command.as_deref().unwrap(),
            ["npx", "pkg"]
        );
    }

    #[test]
    fn to_server_configs_http() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("mcp.json"),
            r#"{"mcpServers":{"web":{"url":"http://localhost:3001/mcp"}}}"#,
        )
        .unwrap();
        let config = ProjectMcpConfig::load(dir.path()).unwrap().unwrap();
        let server_configs = config.to_server_configs();
        assert_eq!(server_configs[0].transport, McpTransportKind::Http);
        assert_eq!(
            server_configs[0].url.as_deref().unwrap(),
            "http://localhost:3001/mcp"
        );
    }

    #[test]
    fn entry_without_command_or_url_skipped() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("mcp.json"),
            r#"{"mcpServers":{"empty":{}}}"#,
        )
        .unwrap();
        let config = ProjectMcpConfig::load(dir.path()).unwrap().unwrap();
        assert!(config.to_server_configs().is_empty());
    }
}
