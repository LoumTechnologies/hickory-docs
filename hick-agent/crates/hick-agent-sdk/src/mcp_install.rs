//! Shared MCP server installation logic.
//!
//! Drives the install sequence used by both CLI subcommands and TUI slash
//! commands. Does not perform any I/O prompting — callers handle that.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::command::CommandExecutor;
use crate::mcp_client::{HttpMcpClient, McpClient, McpToolDef, StdioMcpClient};
use crate::mcp_config::{
    McpCredentialStore, McpSandboxAllowedHost, McpServersConfig, McpServerTrust,
    McpTransportKind,
};

// ---------------------------------------------------------------------------
// Install spec
// ---------------------------------------------------------------------------

/// Parsed MCP server install specification.
#[derive(Debug, Clone)]
pub enum McpInstallSpec {
    /// Child-process server; communicate over stdio.
    Stdio { command: Vec<String> },
    /// Remote server; communicate over HTTP JSON-RPC.
    Http { url: String },
}

/// Parse a raw spec string into a [`McpInstallSpec`].
///
/// Strings starting with `http://` or `https://` become `Http`; everything
/// else is shell-split into a `Stdio` command.
pub fn parse_spec(raw: &str) -> McpInstallSpec {
    let raw = raw.trim();
    if raw.starts_with("http://") || raw.starts_with("https://") {
        McpInstallSpec::Http { url: raw.to_string() }
    } else {
        McpInstallSpec::Stdio { command: shell_split(raw) }
    }
}

/// Whitespace-aware shell splitter that handles single- and double-quoted tokens.
fn shell_split(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        match (in_quote, ch) {
            (None, '"') | (None, '\'') => in_quote = Some(ch),
            (Some(q), c) if c == q => in_quote = None,
            (None, c) if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

// ---------------------------------------------------------------------------
// Name inference
// ---------------------------------------------------------------------------

/// Infer a server name from the install spec.
///
/// Returns `None` when the name is ambiguous and the caller should prompt.
pub fn infer_name(spec: &McpInstallSpec) -> Option<String> {
    match spec {
        McpInstallSpec::Http { url } => {
            // Extract meaningful label from URL host: "api.example.com" → "example".
            let without_scheme = url
                .strip_prefix("https://")
                .or_else(|| url.strip_prefix("http://"))
                .unwrap_or(url);
            let host = without_scheme.split('/').next().unwrap_or("");
            let host = host.split(':').next().unwrap_or(host); // strip port
            let parts: Vec<&str> = host.split('.').collect();
            if parts.len() >= 2 {
                Some(parts[parts.len() - 2].to_string())
            } else if !host.is_empty() {
                Some(host.to_string())
            } else {
                None
            }
        }
        McpInstallSpec::Stdio { command } => {
            // Skip binary name and flags; take the first non-flag token as the package.
            // e.g. ["npx", "-y", "@scope/server-github"] → "github"
            // e.g. ["uvx", "mcp-server-fetch"] → "fetch"
            let pkg = command.iter().skip(1).find(|t| !t.starts_with('-'))?;
            let last_seg = pkg.split('/').next_back().unwrap_or(pkg.as_str());
            let name = last_seg
                .strip_prefix("mcp-server-")
                .or_else(|| last_seg.strip_prefix("server-"))
                .unwrap_or(last_seg);
            if name.is_empty() || name == "-" { None } else { Some(name.to_string()) }
        }
    }
}

// ---------------------------------------------------------------------------
// Package manager detection
// ---------------------------------------------------------------------------

/// Package manager inferred from the first token of a stdio command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    /// `npx` or `npm` — Node package manager.
    Npm,
    /// `uvx` — Python uv executor.
    Uvx,
    /// `pip` or `pip3` — Python pip.
    Pip,
    /// `cargo` — Rust package manager.
    Cargo,
    /// Unrecognised first token.
    Unknown,
}

/// Detect which package manager is used from the first token of `command`.
pub fn detect_package_manager(command: &str) -> PackageManager {
    match command.split_whitespace().next().unwrap_or("") {
        "npx" | "npm" => PackageManager::Npm,
        "uvx" => PackageManager::Uvx,
        "pip" | "pip3" => PackageManager::Pip,
        "cargo" => PackageManager::Cargo,
        _ => PackageManager::Unknown,
    }
}

// ---------------------------------------------------------------------------
// Package manager network policy
// ---------------------------------------------------------------------------

/// Allowed hosts for the package-registry network policy used during install.
pub struct PackageManagerNetworkPolicy;

impl PackageManagerNetworkPolicy {
    /// All host entries covering npm, PyPI, crates.io, and GitHub releases.
    pub fn allowed_hosts() -> Vec<McpSandboxAllowedHost> {
        vec![
            McpSandboxAllowedHost { host: "registry.npmjs.org".into(), port: "443".into() },
            McpSandboxAllowedHost { host: "registry.npmjs.org".into(), port: "80".into() },
            McpSandboxAllowedHost { host: "pypi.org".into(), port: "443".into() },
            McpSandboxAllowedHost { host: "files.pythonhosted.org".into(), port: "443".into() },
            McpSandboxAllowedHost { host: "static.crates.io".into(), port: "443".into() },
            McpSandboxAllowedHost { host: "crates.io".into(), port: "443".into() },
            McpSandboxAllowedHost { host: "github.com".into(), port: "443".into() },
            McpSandboxAllowedHost {
                host: "objects.githubusercontent.com".into(),
                port: "443".into(),
            },
        ]
    }
}

// ---------------------------------------------------------------------------
// Credential prompting (channel-based, for TUI)
// ---------------------------------------------------------------------------

/// Sent by the install task to request credential input from the UI.
pub struct McpCredentialRequest {
    /// Name of the server being installed.
    pub server_name: String,
    /// Suggested environment variable name shown to the user.
    pub suggested_env_var: String,
    /// Send `Some((env_var, value))` to continue, or `None` to skip.
    pub response_tx: tokio::sync::oneshot::Sender<Option<(String, String)>>,
}

// ---------------------------------------------------------------------------
// McpInstaller
// ---------------------------------------------------------------------------

/// Drives the MCP server install sequence.
pub struct McpInstaller;

impl McpInstaller {
    /// Run the package install step in the WASM sandbox.
    ///
    /// Pass `Some(executor)` to run the install in the WASM VM; pass `None`
    /// to skip (e.g. when invoked without a WASM image available). No-op for
    /// `Http` specs regardless of whether an executor is present.
    pub async fn run_package_install(
        spec: &McpInstallSpec,
        executor: Option<&mut dyn CommandExecutor>,
    ) -> Result<()> {
        let McpInstallSpec::Stdio { command } = spec else {
            return Ok(());
        };
        let Some(exec) = executor else {
            return Ok(());
        };
        if command.is_empty() {
            return Ok(());
        }

        let pm = detect_package_manager(&command[0]);
        let Some(install_cmd) = build_install_command(pm, command) else {
            return Ok(());
        };

        let output = exec
            .execute("mcp-install", &install_cmd, "alpine", Some(120))
            .await
            .context("package install in WASM sandbox failed")?;

        if output.exit_code != Some(0) {
            anyhow::bail!(
                "package install failed (exit {:?}):\n{}",
                output.exit_code,
                output.output
            );
        }
        Ok(())
    }

    /// Connect to the server and call `tools/list` to verify it responds.
    ///
    /// Fails if the server does not respond. Uses an empty credential store
    /// so the server must accept unauthenticated `tools/list` requests for
    /// verification to succeed.
    pub async fn verify_connection(spec: &McpInstallSpec) -> Result<Vec<McpToolDef>> {
        match spec {
            McpInstallSpec::Stdio { command } => {
                let client = StdioMcpClient::new(command.clone());
                client
                    .list_tools()
                    .await
                    .context("stdio MCP server did not respond to tools/list")
            }
            McpInstallSpec::Http { url } => {
                let client = HttpMcpClient::new(
                    url.clone(),
                    "install-verify".to_string(),
                    Arc::new(McpCredentialStore::empty()),
                )?;
                client
                    .list_tools()
                    .await
                    .context("HTTP MCP server did not respond to tools/list")
            }
        }
    }

    /// Append a `[[servers]]` entry to `.hick/mcp-servers.toml`.
    ///
    /// Creates the directory and file if absent. Returns an error if a server
    /// with `name` is already present.
    pub fn write_config(
        spec: &McpInstallSpec,
        name: &str,
        trust: McpServerTrust,
        project_dir: &Path,
    ) -> Result<()> {
        let config_path = project_dir.join(".hick").join("mcp-servers.toml");
        std::fs::create_dir_all(
            config_path.parent().expect("config path always has a parent"),
        )?;

        // Duplicate-name guard.
        if config_path.exists() {
            let existing = std::fs::read_to_string(&config_path)?;
            let config: McpServersConfig = toml::from_str(&existing)?;
            if config.find(name).is_some() {
                anyhow::bail!("MCP server '{}' is already configured", name);
            }
        }

        let trust_str = match trust {
            McpServerTrust::Trusted => "trusted",
            McpServerTrust::Untrusted => "untrusted",
            McpServerTrust::Prompt => "prompt",
        };

        let entry = match spec {
            McpInstallSpec::Stdio { command } => format!(
                "\n[[servers]]\nname = {}\ntransport = \"stdio\"\ncommand = {}\ntrust = \"{}\"\n",
                toml_quote(name),
                toml_string_array(command),
                trust_str,
            ),
            McpInstallSpec::Http { url } => format!(
                "\n[[servers]]\nname = {}\ntransport = \"http\"\nurl = {}\ntrust = \"{}\"\n",
                toml_quote(name),
                toml_quote(url),
                trust_str,
            ),
        };

        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config_path)?;
        file.write_all(entry.as_bytes())?;

        // Verify the result parses correctly.
        let written = std::fs::read_to_string(&config_path)?;
        toml::from_str::<McpServersConfig>(&written)
            .context("written mcp-servers.toml failed to re-parse")?;
        Ok(())
    }

    /// Append credential entries to `.hick/mcp-secrets.toml`.
    ///
    /// Each pair is `(env_var_name, value)`. If `value` is empty, writes a
    /// `type = "env"` reference (user must set the env var); otherwise writes
    /// a `type = "bearer"` entry with the literal token.
    pub fn write_credentials(
        name: &str,
        creds: Vec<(String, String)>,
        project_dir: &Path,
    ) -> Result<()> {
        if creds.is_empty() {
            return Ok(());
        }
        let secrets_path = project_dir.join(".hick").join("mcp-secrets.toml");
        std::fs::create_dir_all(
            secrets_path.parent().expect("secrets path always has a parent"),
        )?;

        let mut entries = String::new();
        for (env_var, value) in &creds {
            if env_var.is_empty() {
                continue;
            }
            let entry = if value.is_empty() {
                format!(
                    "\n[[credentials]]\nserver = {}\ntype = \"env\"\nvar = {}\n",
                    toml_quote(name),
                    toml_quote(env_var),
                )
            } else {
                format!(
                    "\n[[credentials]]\nserver = {}\ntype = \"bearer\"\ntoken = {}\n",
                    toml_quote(name),
                    toml_quote(value),
                )
            };
            entries.push_str(&entry);
        }

        if entries.is_empty() {
            return Ok(());
        }

        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&secrets_path)?;
        file.write_all(entries.as_bytes())?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Config management helpers
// ---------------------------------------------------------------------------

/// Read all configured MCP servers and return a formatted table.
pub fn list_servers(project_dir: &Path) -> Result<Vec<String>> {
    let config_path = project_dir.join(".hick").join("mcp-servers.toml");
    let config = McpServersConfig::load_or_default(&config_path)?;
    if config.servers.is_empty() {
        return Ok(vec!["No MCP servers configured.".into()]);
    }
    let mut lines = vec![format!(
        "{:<20} {:<8} {:<10} {}",
        "NAME", "TRANSPORT", "TRUST", "COMMAND/URL"
    )];
    lines.push("-".repeat(60));
    for s in &config.servers {
        let transport = match s.transport {
            McpTransportKind::Stdio => "stdio",
            McpTransportKind::Http => "http",
        };
        let trust = match s.trust {
            McpServerTrust::Trusted => "trusted",
            McpServerTrust::Untrusted => "untrusted",
            McpServerTrust::Prompt => "prompt",
        };
        let detail = s
            .command
            .as_ref()
            .map(|c| c.join(" "))
            .or_else(|| s.url.clone())
            .unwrap_or_default();
        lines.push(format!(
            "{:<20} {:<8} {:<10} {}",
            s.name, transport, trust, detail
        ));
    }
    Ok(lines)
}

/// Remove the server with `name` from `.hick/mcp-servers.toml`.
///
/// Returns an error if `name` is not found.
pub fn remove_server(name: &str, project_dir: &Path) -> Result<()> {
    let config_path = project_dir.join(".hick").join("mcp-servers.toml");
    let content = std::fs::read_to_string(&config_path)
        .context("mcp-servers.toml not found or unreadable")?;

    let mut config: McpServersConfig = toml::from_str(&content)?;
    let before = config.servers.len();
    config.servers.retain(|s| s.name != name);
    if config.servers.len() == before {
        anyhow::bail!("MCP server '{}' not found", name);
    }

    std::fs::write(&config_path, serialize_servers_config(&config.servers))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn build_install_command(pm: PackageManager, command: &[String]) -> Option<String> {
    match pm {
        PackageManager::Npm => {
            let pkg = command.iter().skip(1).find(|t| !t.starts_with('-'))?;
            Some(format!("npm install -g {pkg}"))
        }
        PackageManager::Uvx => {
            let pkg = command.iter().skip(1).find(|t| !t.starts_with('-'))?;
            Some(format!("pip install {pkg}"))
        }
        PackageManager::Pip => Some(command.join(" ")),
        PackageManager::Cargo => {
            let pkg = command.iter().skip(1).find(|t| !t.starts_with('-'))?;
            Some(format!("cargo install {pkg}"))
        }
        PackageManager::Unknown => None,
    }
}

pub(crate) fn toml_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

pub(crate) fn toml_string_array(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| toml_quote(s)).collect();
    format!("[{}]", quoted.join(", "))
}

/// Manually re-serialise an `McpServerConfig` slice to TOML.
///
/// `McpServerConfig` has no `Serialize` impl; this function produces output
/// round-trip-compatible with `McpServersConfig::load_or_default`.
fn serialize_servers_config(
    servers: &[crate::mcp_config::McpServerConfig],
) -> String {
    let mut out = String::new();
    for s in servers {
        out.push_str("\n[[servers]]\n");
        out.push_str(&format!("name = {}\n", toml_quote(&s.name)));
        let transport = match s.transport {
            McpTransportKind::Stdio => "stdio",
            McpTransportKind::Http => "http",
        };
        out.push_str(&format!("transport = \"{transport}\"\n"));
        if let Some(cmd) = &s.command {
            out.push_str(&format!("command = {}\n", toml_string_array(cmd)));
        }
        if let Some(url) = &s.url {
            out.push_str(&format!("url = {}\n", toml_quote(url)));
        }
        let trust = match s.trust {
            McpServerTrust::Trusted => "trusted",
            McpServerTrust::Untrusted => "untrusted",
            McpServerTrust::Prompt => "prompt",
        };
        out.push_str(&format!("trust = \"{trust}\"\n"));
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_spec ---

    #[test]
    fn parse_spec_https_url() {
        let spec = parse_spec("https://api.example.com/mcp");
        assert!(matches!(spec, McpInstallSpec::Http { url } if url == "https://api.example.com/mcp"));
    }

    #[test]
    fn parse_spec_http_url() {
        let spec = parse_spec("http://localhost:3001/mcp");
        assert!(matches!(spec, McpInstallSpec::Http { .. }));
    }

    #[test]
    fn parse_spec_stdio_bare() {
        let spec = parse_spec("npx -y @modelcontextprotocol/server-github");
        match spec {
            McpInstallSpec::Stdio { command } => {
                assert_eq!(
                    command,
                    ["npx", "-y", "@modelcontextprotocol/server-github"]
                );
            }
            _ => panic!("expected Stdio"),
        }
    }

    #[test]
    fn parse_spec_stdio_quoted_arg() {
        let spec = parse_spec(r#"uvx "mcp-server-fetch""#);
        match spec {
            McpInstallSpec::Stdio { command } => {
                assert_eq!(command, ["uvx", "mcp-server-fetch"]);
            }
            _ => panic!("expected Stdio"),
        }
    }

    #[test]
    fn parse_spec_trims_whitespace() {
        let spec = parse_spec("  uvx mcp-server-fetch  ");
        assert!(matches!(spec, McpInstallSpec::Stdio { .. }));
    }

    // --- infer_name ---

    #[test]
    fn infer_name_npm_scoped_package() {
        let spec = McpInstallSpec::Stdio {
            command: vec![
                "npx".into(),
                "-y".into(),
                "@modelcontextprotocol/server-github".into(),
            ],
        };
        assert_eq!(infer_name(&spec), Some("github".into()));
    }

    #[test]
    fn infer_name_uvx_mcp_server() {
        let spec = McpInstallSpec::Stdio {
            command: vec!["uvx".into(), "mcp-server-fetch".into()],
        };
        assert_eq!(infer_name(&spec), Some("fetch".into()));
    }

    #[test]
    fn infer_name_http_url() {
        let spec = McpInstallSpec::Http {
            url: "https://api.example.com/mcp".into(),
        };
        assert_eq!(infer_name(&spec), Some("example".into()));
    }

    #[test]
    fn infer_name_http_localhost() {
        let spec = McpInstallSpec::Http {
            url: "http://localhost:3001/mcp".into(),
        };
        assert_eq!(infer_name(&spec), Some("localhost".into()));
    }

    // --- detect_package_manager ---

    #[test]
    fn detect_npm_npx() {
        assert_eq!(
            detect_package_manager("npx -y @modelcontextprotocol/server-github"),
            PackageManager::Npm
        );
    }

    #[test]
    fn detect_npm_npm() {
        assert_eq!(
            detect_package_manager("npm install -g pkg"),
            PackageManager::Npm
        );
    }

    #[test]
    fn detect_uvx() {
        assert_eq!(
            detect_package_manager("uvx mcp-server-fetch"),
            PackageManager::Uvx
        );
    }

    #[test]
    fn detect_pip() {
        assert_eq!(detect_package_manager("pip install pkg"), PackageManager::Pip);
        assert_eq!(
            detect_package_manager("pip3 install pkg"),
            PackageManager::Pip
        );
    }

    #[test]
    fn detect_cargo() {
        assert_eq!(
            detect_package_manager("cargo install pkg"),
            PackageManager::Cargo
        );
    }

    #[test]
    fn detect_unknown() {
        assert_eq!(
            detect_package_manager("docker run something"),
            PackageManager::Unknown
        );
        assert_eq!(detect_package_manager(""), PackageManager::Unknown);
    }

    // --- write_config ---

    #[test]
    fn write_config_stdio_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let spec = McpInstallSpec::Stdio {
            command: vec![
                "npx".into(),
                "-y".into(),
                "@modelcontextprotocol/server-github".into(),
            ],
        };
        McpInstaller::write_config(&spec, "github", McpServerTrust::Prompt, dir.path()).unwrap();

        let config_path = dir.path().join(".hick").join("mcp-servers.toml");
        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: McpServersConfig = toml::from_str(&content).unwrap();
        assert_eq!(config.servers.len(), 1);
        let s = &config.servers[0];
        assert_eq!(s.name, "github");
        assert_eq!(s.transport, McpTransportKind::Stdio);
        assert_eq!(
            s.command.as_deref().unwrap(),
            ["npx", "-y", "@modelcontextprotocol/server-github"]
        );
        assert_eq!(s.trust, McpServerTrust::Prompt);
    }

    #[test]
    fn write_config_http_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let spec = McpInstallSpec::Http {
            url: "https://api.example.com/mcp".into(),
        };
        McpInstaller::write_config(&spec, "example", McpServerTrust::Trusted, dir.path())
            .unwrap();

        let config_path = dir.path().join(".hick").join("mcp-servers.toml");
        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: McpServersConfig = toml::from_str(&content).unwrap();
        let s = &config.servers[0];
        assert_eq!(s.name, "example");
        assert_eq!(s.transport, McpTransportKind::Http);
        assert_eq!(s.url.as_deref().unwrap(), "https://api.example.com/mcp");
        assert_eq!(s.trust, McpServerTrust::Trusted);
    }

    #[test]
    fn write_config_duplicate_name_errors() {
        let dir = tempfile::tempdir().unwrap();
        let spec = McpInstallSpec::Http {
            url: "https://api.example.com/mcp".into(),
        };
        McpInstaller::write_config(&spec, "example", McpServerTrust::Prompt, dir.path()).unwrap();
        let result =
            McpInstaller::write_config(&spec, "example", McpServerTrust::Prompt, dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already configured"));
    }

    #[test]
    fn write_config_multiple_servers() {
        let dir = tempfile::tempdir().unwrap();
        McpInstaller::write_config(
            &McpInstallSpec::Stdio {
                command: vec!["npx".into(), "-y".into(), "pkg-a".into()],
            },
            "a",
            McpServerTrust::Prompt,
            dir.path(),
        )
        .unwrap();
        McpInstaller::write_config(
            &McpInstallSpec::Http {
                url: "https://b.example.com/mcp".into(),
            },
            "b",
            McpServerTrust::Trusted,
            dir.path(),
        )
        .unwrap();

        let config_path = dir.path().join(".hick").join("mcp-servers.toml");
        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: McpServersConfig = toml::from_str(&content).unwrap();
        assert_eq!(config.servers.len(), 2);
        assert_eq!(config.servers[0].name, "a");
        assert_eq!(config.servers[1].name, "b");
    }
}
