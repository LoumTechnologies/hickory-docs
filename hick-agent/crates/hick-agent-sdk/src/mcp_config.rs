//! MCP server configuration and credential management.
//!
//! Server configuration lives in `.hick/mcp-servers.toml` and is safe to
//! commit. Auth credentials live in `.hick/mcp-secrets.toml` (gitignored) or
//! come from environment variables. The two files are separate by design —
//! no inline secrets in `mcp-servers.toml`.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Server configuration (safe to commit)
// ---------------------------------------------------------------------------

/// Transport kind for an MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpTransportKind {
    /// Spawn a child process; communicate over its stdin/stdout.
    Stdio,
    /// Send HTTP POST requests to a URL.
    Http,
}

/// Trust level assigned to an MCP server.
///
/// Controls how [`crate::ToolGate`] handles calls to tools from this server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpServerTrust {
    /// All non-blocked tool calls are allowed without prompting.
    Trusted,
    /// The server is untrusted; any classified data in scope blocks calls.
    Untrusted,
    /// The user is prompted for every tool call from this server.
    #[default]
    Prompt,
}

/// Configuration for a single MCP server. Contains no secrets — safe to commit.
///
/// ```toml
/// # Stdio transport
/// [[servers]]
/// name = "filesystem"
/// transport = "stdio"
/// command = ["mcp-server-filesystem", "--root", "/workspace"]
///
/// # HTTP transport
/// [[servers]]
/// name = "github"
/// transport = "http"
/// url = "http://localhost:3001/mcp"
/// trust = "untrusted"
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerConfig {
    /// Unique name used to reference this server from other config stanzas.
    pub name: String,
    /// How to connect to the server (`"stdio"` or `"http"`).
    pub transport: McpTransportKind,
    /// Command to spawn (stdio transport only).
    pub command: Option<Vec<String>>,
    /// Server URL (HTTP transport only).
    pub url: Option<String>,
    /// Trust level for tools provided by this server.
    #[serde(default)]
    pub trust: McpServerTrust,
}

impl McpServerConfig {
    /// Return the stdio command or an error if the transport is not stdio.
    pub fn stdio_command(&self) -> anyhow::Result<&[String]> {
        self.command
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("server '{}' is stdio but has no command", self.name))
    }

    /// Return the HTTP URL or an error if the transport is not HTTP.
    pub fn http_url(&self) -> anyhow::Result<&str> {
        self.url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("server '{}' is http but has no url", self.name))
    }
}

/// Top-level server configuration file (`.hick/mcp-servers.toml`).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct McpServersConfig {
    /// List of MCP server configurations.
    #[serde(default)]
    pub servers: Vec<McpServerConfig>,
}

impl McpServersConfig {
    /// Load from a TOML file, or return an empty config if the file does not exist.
    pub fn load_or_default(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Look up a server by name.
    pub fn find(&self, name: &str) -> Option<&McpServerConfig> {
        self.servers.iter().find(|s| s.name == name)
    }
}

// ---------------------------------------------------------------------------
// Credential types  — never serialized, never written to disk
// ---------------------------------------------------------------------------

/// Raw credential entry as loaded from the secrets file (before env-var resolution).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RawCredential {
    Bearer { token: String },
    ApiKey { header: String, key: String },
    Basic { username: String, password: String },
    /// Resolve the value from an environment variable at load time.
    Env { var: String },
}

/// A resolved credential ready to inject as HTTP headers at call time.
///
/// Never serialized, never written to disk.
#[derive(Debug, Clone)]
pub enum ResolvedCredential {
    /// `Authorization: Bearer <token>`
    Bearer(String),
    /// `<header>: <key>`
    ApiKey {
        /// HTTP header name.
        header: String,
        /// Header value.
        key: String,
    },
    /// `Authorization: Basic <base64(username:password)>`
    Basic {
        /// Basic auth username.
        username: String,
        /// Basic auth password.
        password: String,
    },
}

impl ResolvedCredential {
    /// Inject this credential as an HTTP header into the given request builder.
    pub fn apply(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self {
            ResolvedCredential::Bearer(token) => {
                builder.header("Authorization", format!("Bearer {token}"))
            }
            ResolvedCredential::ApiKey { header, key } => builder.header(header.as_str(), key.as_str()),
            ResolvedCredential::Basic { username, password } => {
                builder.basic_auth(username, Some(password))
            }
        }
    }
}

/// Raw per-server secret entry in the secrets file.
#[derive(Debug, Clone, Deserialize)]
struct RawServerSecret {
    server: String,
    #[serde(flatten)]
    credential: RawCredential,
}

/// Top-level structure of `.hick/mcp-secrets.toml`.
#[derive(Debug, Clone, Deserialize)]
struct RawSecretsConfig {
    #[serde(default)]
    credentials: Vec<RawServerSecret>,
}

/// Holds resolved credentials keyed by server name.
///
/// Credentials are loaded once at startup and held in memory. This type
/// deliberately has **no** `Serialize`, `Deserialize`, or `Display` impl to
/// prevent accidental leakage into logs, API calls, or pipeline output.
///
/// Extension crates receive credentials via `Arc<McpCredentialStore>` — they
/// never cross a serialization boundary.
pub struct McpCredentialStore {
    credentials: HashMap<String, ResolvedCredential>,
}

impl McpCredentialStore {
    /// Load credentials from a TOML secrets file with env-var resolution.
    ///
    /// Returns an empty store if the file does not exist.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::empty());
        }
        let content = std::fs::read_to_string(path)?;
        let raw: RawSecretsConfig = toml::from_str(&content)?;

        let mut credentials = HashMap::new();
        for entry in raw.credentials {
            let resolved = resolve_credential(entry.credential)?;
            credentials.insert(entry.server, resolved);
        }

        Ok(Self { credentials })
    }

    /// Create an empty credential store (no credentials configured).
    pub fn empty() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }

    /// Look up the resolved credential for a named server.
    pub fn get(&self, server_name: &str) -> Option<&ResolvedCredential> {
        self.credentials.get(server_name)
    }
}

fn resolve_credential(raw: RawCredential) -> anyhow::Result<ResolvedCredential> {
    match raw {
        RawCredential::Bearer { token } => {
            Ok(ResolvedCredential::Bearer(resolve_value(token)?))
        }
        RawCredential::ApiKey { header, key } => Ok(ResolvedCredential::ApiKey {
            header,
            key: resolve_value(key)?,
        }),
        RawCredential::Basic { username, password } => Ok(ResolvedCredential::Basic {
            username: resolve_value(username)?,
            password: resolve_value(password)?,
        }),
        RawCredential::Env { var } => {
            let token = std::env::var(&var)
                .map_err(|_| anyhow::anyhow!("env var '{var}' not set for MCP credential"))?;
            Ok(ResolvedCredential::Bearer(token))
        }
    }
}

/// Resolve a string value that may contain an `env:VAR_NAME` indirection.
fn resolve_value(value: String) -> anyhow::Result<String> {
    if let Some(var) = value.strip_prefix("env:") {
        std::env::var(var)
            .map_err(|_| anyhow::anyhow!("env var '{var}' not set for MCP credential"))
    } else {
        Ok(value)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_store_empty_when_no_file() {
        let store = McpCredentialStore::load(Path::new("/nonexistent/path/mcp-secrets.toml"))
            .unwrap();
        assert!(store.get("any-server").is_none());
    }

    #[test]
    fn credential_env_var_indirection() {
        // SAFETY: test-only; no other threads modify this env var concurrently.
        unsafe { std::env::set_var("TEST_MCP_TOKEN_CFG_241", "supersecret") };

        let toml = r#"
[[credentials]]
server = "github"
type = "env"
var = "TEST_MCP_TOKEN_CFG_241"
"#;
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(tmp.path(), toml).unwrap();

        let store = McpCredentialStore::load(tmp.path()).unwrap();
        let cred = store.get("github").unwrap();
        assert!(matches!(cred, ResolvedCredential::Bearer(t) if t == "supersecret"));

        // SAFETY: test-only; no other threads modify this env var concurrently.
        unsafe { std::env::remove_var("TEST_MCP_TOKEN_CFG_241") };
    }

    #[test]
    fn credential_literal_bearer() {
        let toml = r#"
[[credentials]]
server = "myserver"
type = "bearer"
token = "literal-token"
"#;
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(tmp.path(), toml).unwrap();

        let store = McpCredentialStore::load(tmp.path()).unwrap();
        let cred = store.get("myserver").unwrap();
        assert!(matches!(cred, ResolvedCredential::Bearer(t) if t == "literal-token"));
    }

    #[test]
    fn credential_api_key() {
        let toml = r#"
[[credentials]]
server = "stripe"
type = "api_key"
header = "X-Api-Key"
key = "sk-test-12345"
"#;
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(tmp.path(), toml).unwrap();

        let store = McpCredentialStore::load(tmp.path()).unwrap();
        let cred = store.get("stripe").unwrap();
        assert!(
            matches!(cred, ResolvedCredential::ApiKey { header, key } if header == "X-Api-Key" && key == "sk-test-12345")
        );
    }

    #[test]
    fn servers_config_loads_correctly() {
        let toml = r#"
[[servers]]
name = "filesystem"
transport = "stdio"
command = ["mcp-server-filesystem", "--root", "/workspace"]

[[servers]]
name = "github"
transport = "http"
url = "http://localhost:3001/mcp"
trust = "untrusted"
"#;
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(tmp.path(), toml).unwrap();

        let config = McpServersConfig::load_or_default(tmp.path()).unwrap();
        assert_eq!(config.servers.len(), 2);

        let fs = &config.servers[0];
        assert_eq!(fs.name, "filesystem");
        assert_eq!(fs.transport, McpTransportKind::Stdio);
        assert_eq!(
            fs.command.as_deref().unwrap(),
            ["mcp-server-filesystem", "--root", "/workspace"]
        );
        assert_eq!(fs.trust, McpServerTrust::Prompt); // default

        let gh = &config.servers[1];
        assert_eq!(gh.name, "github");
        assert_eq!(gh.transport, McpTransportKind::Http);
        assert_eq!(gh.url.as_deref().unwrap(), "http://localhost:3001/mcp");
        assert_eq!(gh.trust, McpServerTrust::Untrusted);
    }

    #[test]
    fn servers_config_default_when_missing() {
        let config =
            McpServersConfig::load_or_default(Path::new("/no/such/mcp-servers.toml")).unwrap();
        assert!(config.servers.is_empty());
    }

    #[test]
    fn servers_config_find() {
        let config = McpServersConfig {
            servers: vec![McpServerConfig {
                name: "foo".into(),
                transport: McpTransportKind::Stdio,
                command: Some(vec!["cmd".into()]),
                url: None,
                trust: McpServerTrust::Trusted,
            }],
        };
        assert!(config.find("foo").is_some());
        assert!(config.find("bar").is_none());
    }
}
