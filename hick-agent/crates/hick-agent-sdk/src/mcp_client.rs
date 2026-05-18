//! JSON-RPC 2.0 MCP client with stdio and HTTP transport support.
//!
//! Provides the [`McpClient`] trait and concrete implementations for
//! stdio ([`StdioMcpClient`]) and HTTP ([`HttpMcpClient`]) transports.
//! Both implementations cache the tool manifest and support refresh.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::mcp_config::{McpCredentialStore, McpSandboxConfig};

// ---------------------------------------------------------------------------
// Tool manifest
// ---------------------------------------------------------------------------

/// Description of a single tool exposed by an MCP server.
#[derive(Debug, Clone, Deserialize)]
pub struct McpToolDef {
    /// Tool name as reported by the server.
    pub name: String,
    /// Human-readable description of what the tool does.
    #[serde(default)]
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    #[serde(rename = "inputSchema", default)]
    pub input_schema: serde_json::Value,
    /// JSON Schema for the tool's output (optional in MCP spec).
    #[serde(rename = "outputSchema", default)]
    pub output_schema: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// McpClient trait
// ---------------------------------------------------------------------------

/// Trait for a connected MCP server client.
///
/// Concrete implementations are [`StdioMcpClient`] (child-process transport)
/// and [`HttpMcpClient`] (HTTP transport). Both serialize all calls through a
/// lock, so requests are never interleaved on the same connection.
#[async_trait]
pub trait McpClient: Send + Sync {
    /// Invoke a JSON-RPC method on the server and return the `result` value.
    async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value>;

    /// Return the cached tool manifest, fetching it from the server if not
    /// yet loaded.
    async fn list_tools(&self) -> anyhow::Result<Vec<McpToolDef>>;

    /// Re-fetch the tool manifest from the server and update the cache.
    async fn refresh_tools(&self) -> anyhow::Result<Vec<McpToolDef>>;
}

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 types (crate-private)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: &'a serde_json::Value,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

fn parse_tools_list(result: serde_json::Value) -> anyhow::Result<Vec<McpToolDef>> {
    let tools = result
        .get("tools")
        .ok_or_else(|| anyhow::anyhow!("tools/list response missing 'tools' array"))?;
    serde_json::from_value(tools.clone()).map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Stdio transport
// ---------------------------------------------------------------------------

struct StdioConnection {
    // Held to keep the process alive; dropped when the connection is discarded.
    #[allow(dead_code)]
    child: tokio::process::Child,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::io::BufReader<tokio::process::ChildStdout>,
}

/// MCP client that communicates with a server process over stdin/stdout.
///
/// The child process is spawned lazily on the first call. If the process exits
/// unexpectedly, the next call automatically respawns it.
pub struct StdioMcpClient {
    command: Vec<String>,
    state: tokio::sync::Mutex<Option<StdioConnection>>,
    cached_tools: Mutex<Option<Vec<McpToolDef>>>,
    next_id: AtomicU64,
}

impl StdioMcpClient {
    /// Create a new stdio client that will spawn `command` on first use.
    pub fn new(command: Vec<String>) -> Self {
        Self {
            command,
            state: tokio::sync::Mutex::new(None),
            cached_tools: Mutex::new(None),
            next_id: AtomicU64::new(1),
        }
    }

    fn spawn_connection(&self) -> anyhow::Result<StdioConnection> {
        use tokio::process::Command;
        use std::process::Stdio as StdStdio;

        let (cmd, args) = self
            .command
            .split_first()
            .ok_or_else(|| anyhow::anyhow!("stdio MCP command is empty"))?;

        let mut child = Command::new(cmd)
            .args(args)
            .stdin(StdStdio::piped())
            .stdout(StdStdio::piped())
            .stderr(StdStdio::null())
            .spawn()?;

        let stdin = child.stdin.take().expect("stdin piped");
        let stdout = child.stdout.take().expect("stdout piped");

        Ok(StdioConnection {
            child,
            stdin,
            stdout: tokio::io::BufReader::new(stdout),
        })
    }

    async fn do_call(
        &self,
        state: &mut Option<StdioConnection>,
        id: u64,
        method: &str,
        params: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        if state.is_none() {
            *state = Some(self.spawn_connection()?);
        }
        let conn = state.as_mut().unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };
        let mut bytes = serde_json::to_vec(&request)?;
        bytes.push(b'\n');

        conn.stdin.write_all(&bytes).await?;
        conn.stdin.flush().await?;

        let mut line = String::new();
        let n = conn.stdout.read_line(&mut line).await?;
        if n == 0 {
            anyhow::bail!("stdio MCP server closed stdout unexpectedly");
        }

        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        match response.result {
            Some(r) => Ok(r),
            None => anyhow::bail!(
                "JSON-RPC error from stdio MCP server: {}",
                response
                    .error
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            ),
        }
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut state = self.state.lock().await;

        // Attempt the call; on any I/O error, discard the connection and retry
        // once with a fresh process (automatic reconnect on process exit).
        match self.do_call(&mut state, id, method, &params).await {
            Ok(result) => Ok(result),
            Err(_) => {
                // Force reconnect: drop the dead connection.
                *state = None;
                self.do_call(&mut state, id, method, &params).await
            }
        }
    }

    async fn list_tools(&self) -> anyhow::Result<Vec<McpToolDef>> {
        {
            let cache = self.cached_tools.lock().unwrap();
            if let Some(tools) = cache.as_ref() {
                return Ok(tools.clone());
            }
        }
        self.refresh_tools().await
    }

    async fn refresh_tools(&self) -> anyhow::Result<Vec<McpToolDef>> {
        let result = self.call("tools/list", serde_json::Value::Null).await?;
        let tools = parse_tools_list(result)?;
        *self.cached_tools.lock().unwrap() = Some(tools.clone());
        Ok(tools)
    }
}

// ---------------------------------------------------------------------------
// Sentinel nonce generation
// ---------------------------------------------------------------------------

fn generate_nonce() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::SystemTime;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    // 16-char hex timestamp + 8-char counter gives a unique, hard-to-guess nonce.
    format!("{:016x}{:08x}", time, count)
}

// ---------------------------------------------------------------------------
// Sandboxed stdio transport
// ---------------------------------------------------------------------------

/// MCP client that injects a sentinel credential token into the server's
/// environment instead of the real credential.
///
/// The sentinel has the format `hick-proxy:<server_name>:<nonce>`. The real
/// credential never enters the server process's environment; it is held by
/// the host and injected by the smoltcp interceptor when the server makes
/// outbound HTTP calls carrying the sentinel in its `Authorization` header.
///
/// The process lifecycle is identical to [`StdioMcpClient`]: the server is
/// spawned on first use and kept alive across JSON-RPC calls for the session.
/// On process exit, the client automatically respawns.
///
/// Use [`StdioMcpClient`] for servers without a sandbox config.
pub struct SandboxedStdioMcpClient {
    server_name: String,
    command: Vec<String>,
    sandbox: McpSandboxConfig,
    sentinel_token: String,
    state: tokio::sync::Mutex<Option<StdioConnection>>,
    cached_tools: Mutex<Option<Vec<McpToolDef>>>,
    next_id: AtomicU64,
}

impl SandboxedStdioMcpClient {
    /// Create a sandboxed stdio client.
    ///
    /// Generates a unique sentinel token `hick-proxy:<server_name>:<nonce>` on
    /// construction. The token is injected into the server process via
    /// `sandbox.credential_env` at spawn time. Register `sentinel_token()` with
    /// [`crate::McpSentinelInterceptor`] so the interceptor can resolve it.
    pub fn new(
        server_name: String,
        command: Vec<String>,
        sandbox: McpSandboxConfig,
    ) -> Self {
        let nonce = generate_nonce();
        let sentinel_token = format!("hick-proxy:{}:{}", server_name, nonce);
        Self {
            server_name,
            command,
            sandbox,
            sentinel_token,
            state: tokio::sync::Mutex::new(None),
            cached_tools: Mutex::new(None),
            next_id: AtomicU64::new(1),
        }
    }

    /// The sentinel token injected into the server's `credential_env`.
    ///
    /// Register this with the interceptor so it can substitute the real credential.
    pub fn sentinel_token(&self) -> &str {
        &self.sentinel_token
    }

    /// The server name this client was created for.
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// The sandbox configuration for this server.
    pub fn sandbox_config(&self) -> &McpSandboxConfig {
        &self.sandbox
    }

    fn spawn_connection(&self) -> anyhow::Result<StdioConnection> {
        use std::process::Stdio as StdStdio;
        use tokio::process::Command;

        let (cmd, args) = self
            .command
            .split_first()
            .ok_or_else(|| anyhow::anyhow!("stdio MCP command is empty"))?;

        let mut child = Command::new(cmd)
            .args(args)
            // Inject sentinel token under the declared env var name.
            // The real credential is NOT present in the environment.
            .env(&self.sandbox.credential_env, &self.sentinel_token)
            .stdin(StdStdio::piped())
            .stdout(StdStdio::piped())
            .stderr(StdStdio::null())
            .spawn()?;

        let stdin = child.stdin.take().expect("stdin piped");
        let stdout = child.stdout.take().expect("stdout piped");

        Ok(StdioConnection {
            child,
            stdin,
            stdout: tokio::io::BufReader::new(stdout),
        })
    }

    async fn do_call(
        &self,
        state: &mut Option<StdioConnection>,
        id: u64,
        method: &str,
        params: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        if state.is_none() {
            *state = Some(self.spawn_connection()?);
        }
        let conn = state.as_mut().unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params,
        };
        let mut bytes = serde_json::to_vec(&request)?;
        bytes.push(b'\n');

        conn.stdin.write_all(&bytes).await?;
        conn.stdin.flush().await?;

        let mut line = String::new();
        let n = conn.stdout.read_line(&mut line).await?;
        if n == 0 {
            anyhow::bail!("sandboxed stdio MCP server '{}' closed stdout unexpectedly", self.server_name);
        }

        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        match response.result {
            Some(r) => Ok(r),
            None => anyhow::bail!(
                "JSON-RPC error from sandboxed stdio MCP server '{}': {}",
                self.server_name,
                response
                    .error
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            ),
        }
    }
}

#[async_trait]
impl McpClient for SandboxedStdioMcpClient {
    async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut state = self.state.lock().await;

        match self.do_call(&mut state, id, method, &params).await {
            Ok(result) => Ok(result),
            Err(_) => {
                *state = None;
                self.do_call(&mut state, id, method, &params).await
            }
        }
    }

    async fn list_tools(&self) -> anyhow::Result<Vec<McpToolDef>> {
        {
            let cache = self.cached_tools.lock().unwrap();
            if let Some(tools) = cache.as_ref() {
                return Ok(tools.clone());
            }
        }
        self.refresh_tools().await
    }

    async fn refresh_tools(&self) -> anyhow::Result<Vec<McpToolDef>> {
        let result = self.call("tools/list", serde_json::Value::Null).await?;
        let tools = parse_tools_list(result)?;
        *self.cached_tools.lock().unwrap() = Some(tools.clone());
        Ok(tools)
    }
}

// ---------------------------------------------------------------------------
// HTTP transport
// ---------------------------------------------------------------------------

/// MCP client that communicates with a server over HTTP (JSON-RPC 2.0 POST).
///
/// Auth credentials are looked up from the [`McpCredentialStore`] at call
/// time — they are never serialized or stored in the client's fields.
pub struct HttpMcpClient {
    url: String,
    server_name: String,
    credentials: Arc<McpCredentialStore>,
    http: reqwest::Client,
    cached_tools: Mutex<Option<Vec<McpToolDef>>>,
    next_id: AtomicU64,
}

impl HttpMcpClient {
    /// Create a new HTTP client for the server at `url`.
    ///
    /// Auth credentials for `server_name` are looked up from `credentials` at
    /// each call — no credential is stored in the client struct itself.
    pub fn new(
        url: String,
        server_name: String,
        credentials: Arc<McpCredentialStore>,
    ) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder().build()?;
        Ok(Self {
            url,
            server_name,
            credentials,
            http,
            cached_tools: Mutex::new(None),
            next_id: AtomicU64::new(1),
        })
    }
}

#[async_trait]
impl McpClient for HttpMcpClient {
    async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method,
            params: &params,
        };

        // Inject auth headers at call time from the credential store.
        let builder = self.http.post(&self.url).json(&request);
        let builder = match self.credentials.get(&self.server_name) {
            Some(cred) => cred.apply(builder),
            None => builder,
        };

        let response: JsonRpcResponse = builder.send().await?.json().await?;
        match response.result {
            Some(r) => Ok(r),
            None => anyhow::bail!(
                "JSON-RPC error from HTTP MCP server '{}': {}",
                self.server_name,
                response
                    .error
                    .as_ref()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            ),
        }
    }

    async fn list_tools(&self) -> anyhow::Result<Vec<McpToolDef>> {
        {
            let cache = self.cached_tools.lock().unwrap();
            if let Some(tools) = cache.as_ref() {
                return Ok(tools.clone());
            }
        }
        self.refresh_tools().await
    }

    async fn refresh_tools(&self) -> anyhow::Result<Vec<McpToolDef>> {
        let result = self.call("tools/list", serde_json::Value::Null).await?;
        let tools = parse_tools_list(result)?;
        *self.cached_tools.lock().unwrap() = Some(tools.clone());
        Ok(tools)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

    #[test]
    fn parse_tools_list_valid() {
        let result = serde_json::json!({
            "tools": [
                {
                    "name": "read_file",
                    "description": "Read a file from the filesystem",
                    "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}},
                    "outputSchema": {"type": "object"}
                },
                {
                    "name": "list_dir",
                    "description": "List directory contents"
                }
            ]
        });
        let tools = parse_tools_list(result).unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "read_file");
        assert_eq!(tools[0].description, "Read a file from the filesystem");
        assert!(tools[0].output_schema.is_some());
        assert_eq!(tools[1].name, "list_dir");
        assert!(tools[1].output_schema.is_none());
    }

    #[test]
    fn parse_tools_list_empty() {
        let result = serde_json::json!({"tools": []});
        let tools = parse_tools_list(result).unwrap();
        assert!(tools.is_empty());
    }

    #[test]
    fn parse_tools_list_missing_key() {
        let result = serde_json::json!({"notTools": []});
        assert!(parse_tools_list(result).is_err());
    }

    /// Spawn a mock stdio MCP server that responds to one request then exits.
    ///
    /// The mock ignores the request ID and always responds with an empty tools
    /// list. Calls are serialized by the Mutex so ID mismatch is not an issue.
    fn mock_stdio_command() -> Vec<String> {
        // Read one line from stdin (the JSON-RPC request), write one response,
        // then exit. The StdioMcpClient should reconnect for the next call.
        vec![
            "sh".into(),
            "-c".into(),
            r#"IFS= read -r _line && printf '{"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"test_tool","description":"A test tool","inputSchema":{}}]}}\n'"#.into(),
        ]
    }

    #[tokio::test]
    async fn stdio_client_basic_call() {
        let client = StdioMcpClient::new(mock_stdio_command());
        let result = client
            .call("tools/list", serde_json::Value::Null)
            .await
            .unwrap();
        assert!(result.get("tools").is_some());
    }

    #[tokio::test]
    async fn stdio_client_list_tools_parsed() {
        let client = StdioMcpClient::new(mock_stdio_command());
        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");
    }

    #[tokio::test]
    async fn stdio_client_reconnects_after_process_exit() {
        // The mock handles exactly one request then exits.
        // The second call must reconnect (spawn a new process) and succeed.
        let client = StdioMcpClient::new(mock_stdio_command());

        let first = client.call("tools/list", serde_json::Value::Null).await;
        assert!(first.is_ok(), "first call failed: {first:?}");

        // Process has now exited. The next call must auto-reconnect.
        let second = client.call("tools/list", serde_json::Value::Null).await;
        assert!(second.is_ok(), "second call after reconnect failed: {second:?}");
    }

    // -----------------------------------------------------------------------
    // HTTP client tests
    // -----------------------------------------------------------------------

    /// Start a minimal HTTP/1.1 mock server that validates the Authorization
    /// header and responds with an empty tools list.
    async fn start_mock_http_server(
        expected_auth_prefix: &'static str,
    ) -> (u16, tokio::task::JoinHandle<Option<String>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = tokio::io::BufReader::new(stream);

            // Read HTTP request headers
            let mut auth_header: Option<String> = None;
            let mut content_length: usize = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
                if line == "\r\n" || line == "\n" {
                    break;
                }
                let lower = line.to_lowercase();
                if lower.starts_with("authorization:") {
                    auth_header = Some(line.trim().to_string());
                }
                if lower.starts_with("content-length:") {
                    content_length = line
                        .split(':')
                        .nth(1)
                        .unwrap_or("0")
                        .trim()
                        .parse()
                        .unwrap_or(0);
                }
            }

            // Drain the request body so reqwest doesn't get a connection reset.
            if content_length > 0 {
                let mut body = vec![0u8; content_length];
                let _ = reader.read_exact(&mut body).await;
            }

            // Send a valid JSON-RPC response.
            let body = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[]}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            reader
                .get_mut()
                .write_all(response.as_bytes())
                .await
                .unwrap();

            // Validate the expected prefix
            auth_header.filter(|h| {
                h.to_lowercase()
                    .contains(&expected_auth_prefix.to_lowercase())
            })
        });

        (port, handle)
    }

    #[tokio::test]
    async fn http_client_injects_bearer_auth_header() {
        let (port, server) =
            start_mock_http_server("authorization: bearer test-token-http-241").await;

        let toml = r#"
[[credentials]]
server = "testserver"
type = "bearer"
token = "test-token-http-241"
"#;
        let tmp = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(tmp.path(), toml).unwrap();
        let store = Arc::new(crate::mcp_config::McpCredentialStore::load(tmp.path()).unwrap());

        let client = HttpMcpClient::new(
            format!("http://127.0.0.1:{port}/mcp"),
            "testserver".into(),
            store,
        )
        .unwrap();

        let result = client.call("tools/list", serde_json::Value::Null).await;
        assert!(result.is_ok(), "HTTP call failed: {result:?}");

        let auth_found = server.await.unwrap();
        assert!(
            auth_found.is_some(),
            "Authorization header was not injected or did not match expected value"
        );
    }

    // -----------------------------------------------------------------------
    // SandboxedStdioMcpClient tests
    // -----------------------------------------------------------------------

    fn sample_sandbox_config() -> crate::mcp_config::McpSandboxConfig {
        crate::mcp_config::McpSandboxConfig {
            credential_env: "TEST_MCP_TOKEN".into(),
            auth_header: "Authorization".into(),
            auth_scheme: "Bearer".into(),
            deny_all_network: true,
            allowed_hosts: vec![],
        }
    }

    #[test]
    fn sentinel_token_format() {
        let client = SandboxedStdioMcpClient::new(
            "myserver".into(),
            vec!["echo".into()],
            sample_sandbox_config(),
        );
        let token = client.sentinel_token();
        assert!(token.starts_with("hick-proxy:myserver:"), "token: {token}");
        // nonce is non-empty
        let nonce = token.strip_prefix("hick-proxy:myserver:").unwrap();
        assert!(!nonce.is_empty());
    }

    #[test]
    fn sentinel_tokens_are_unique() {
        let c1 = SandboxedStdioMcpClient::new("s".into(), vec!["cmd".into()], sample_sandbox_config());
        let c2 = SandboxedStdioMcpClient::new("s".into(), vec!["cmd".into()], sample_sandbox_config());
        assert_ne!(c1.sentinel_token(), c2.sentinel_token(), "nonces must be unique");
    }

    #[tokio::test]
    async fn sandboxed_client_injects_sentinel_env_var() {
        // Command that prints the value of TEST_MCP_TOKEN to stdout as JSON-RPC.
        // We echo a valid JSON-RPC response so the client can parse it.
        let client = SandboxedStdioMcpClient::new(
            "github".into(),
            vec![
                "sh".into(),
                "-c".into(),
                // Read stdin (the JSON-RPC request), print the env var value in result.
                r#"IFS= read -r _req && printf '{"jsonrpc":"2.0","id":1,"result":{"env":"%s"}}\n' "$TEST_MCP_TOKEN""#.into(),
            ],
            sample_sandbox_config(),
        );

        let sentinel = client.sentinel_token().to_string();
        let result = client.call("tools/list", serde_json::Value::Null).await.unwrap();
        let env_value = result["env"].as_str().unwrap_or("");
        assert_eq!(env_value, sentinel, "sentinel token should be injected into TEST_MCP_TOKEN");
    }

    #[test]
    fn sandboxed_client_real_credential_not_in_sentinel() {
        // The sentinel token must not contain any substring that could be a real token.
        // It should be "hick-proxy:<server>:<nonce>" with no secret embedded.
        let client = SandboxedStdioMcpClient::new(
            "github".into(),
            vec!["cmd".into()],
            sample_sandbox_config(),
        );
        let token = client.sentinel_token();
        // Token must follow the sentinel prefix format exactly.
        assert!(token.starts_with("hick-proxy:github:"), "token: {token}");
        // No spaces, no real secret-looking content.
        assert!(!token.contains(' '), "sentinel must not contain spaces");
    }

    #[tokio::test]
    async fn http_client_works_without_credentials() {
        let (port, server) = start_mock_http_server("").await;

        let store = Arc::new(McpCredentialStore::empty());
        let client = HttpMcpClient::new(
            format!("http://127.0.0.1:{port}/mcp"),
            "noauth".into(),
            store,
        )
        .unwrap();

        let result = client.call("tools/list", serde_json::Value::Null).await;
        assert!(result.is_ok(), "HTTP call without auth failed: {result:?}");

        // Server accepted the request regardless of auth header presence.
        let _ = server.await;
    }
}
