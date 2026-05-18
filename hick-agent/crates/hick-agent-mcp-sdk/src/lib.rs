//! MCP SDK extension: auto-generated Python/TypeScript stub SDK injected into LLM scripts.
//!
//! This extension exposes MCP tools to LLM-written scripts via generated stub
//! functions in `_mcp.py` (Python) and `_mcp.ts` (TypeScript). The stubs call
//! a magic gateway IP (`169.254.1.2:80`) that is intercepted by
//! [`McpSdkInterceptor`] inside the smoltcp network loop — no OS TCP port is
//! bound, no loopback socket is opened.
//!
//! ## Magic IP transport
//!
//! The interceptor is registered in the [`InterceptorRegistry`] at a distinct
//! address from the GraphQL interceptor (`169.254.1.2` vs `169.254.1.3`) so
//! both can coexist in the same agent binary.
//!
//! ## Preamble injection
//!
//! Before a Python or TypeScript script is written to the WASM container, the
//! extension prepends import lines that bring all generated functions into scope.
//! The original LLM response stored in conversation history is not modified —
//! only the copy written to the container receives the preamble.
//!
//! ## Auth isolation
//!
//! Auth credentials never appear in generated stub files. They live in
//! [`McpCredentialStore`] on the host and are accessed inside the interceptor.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use hick_agent_sdk::{McpClient, McpToolDef, ToolGate, ToolVerdict};
use hick_classify::Classification;
use network::interceptor::Interceptor;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Magic IP for the SDK interceptor (distinct from the GraphQL interceptor at 169.254.1.3).
pub const SDK_INTERCEPTOR_IP: Ipv4Addr = Ipv4Addr::new(169, 254, 1, 2);
/// Port used by both the SDK stubs and interceptor.
pub const SDK_INTERCEPTOR_PORT: u16 = 80;

// ---------------------------------------------------------------------------
// Wire protocol
// ---------------------------------------------------------------------------

/// JSON body sent by stub functions to the magic gateway.
#[derive(Debug, Deserialize)]
struct ToolCallRequest {
    server: String,
    tool: String,
    input: serde_json::Value,
}

// ---------------------------------------------------------------------------
// McpSdkInterceptor
// ---------------------------------------------------------------------------

/// Intercepts HTTP POST requests to [`SDK_INTERCEPTOR_IP`] and dispatches
/// them to the appropriate MCP tool via `ToolGate` + `McpClient`.
///
/// No OS port is bound — this interceptor runs entirely within the smoltcp
/// event loop. Auth credentials stay in the host process.
pub struct McpSdkInterceptor {
    tool_gate: Arc<ToolGate>,
    mcp_clients: HashMap<String, Arc<dyn McpClient>>,
    scope: Arc<tokio::sync::Mutex<Vec<Classification>>>,
}

impl McpSdkInterceptor {
    pub fn new(
        tool_gate: Arc<ToolGate>,
        mcp_clients: HashMap<String, Arc<dyn McpClient>>,
        scope: Arc<tokio::sync::Mutex<Vec<Classification>>>,
    ) -> Self {
        Self {
            tool_gate,
            mcp_clients,
            scope,
        }
    }

    /// Dispatch a parsed tool call request.
    ///
    /// Calls `ToolGate::check` before forwarding to `McpClient`. Returns a
    /// (status_code, body) tuple; `403` when blocked.
    async fn dispatch(&self, req: ToolCallRequest) -> (u16, String) {
        let scope = self.scope.lock().await.clone();
        let verdict = self.tool_gate.check(&req.server, &req.tool, &scope).await;
        if verdict == ToolVerdict::Block {
            return (
                403,
                serde_json::json!({
                    "error": format!("blocked: ToolGate denied call to {}/{}", req.server, req.tool)
                })
                .to_string(),
            );
        }

        let client = match self.mcp_clients.get(&req.server) {
            Some(c) => c.clone(),
            None => {
                return (
                    404,
                    serde_json::json!({
                        "error": format!("no MCP client configured for server '{}'", req.server)
                    })
                    .to_string(),
                );
            }
        };

        let params = serde_json::json!({
            "name": req.tool,
            "arguments": req.input,
        });

        match client.call("tools/call", params).await {
            Ok(result) => (200, result.to_string()),
            Err(e) => (
                502,
                serde_json::json!({"error": e.to_string()}).to_string(),
            ),
        }
    }
}

impl Interceptor for McpSdkInterceptor {
    fn matches(&self, dest: SocketAddr) -> bool {
        dest.ip() == IpAddr::V4(SDK_INTERCEPTOR_IP) && dest.port() == SDK_INTERCEPTOR_PORT
    }

    fn handle(
        &self,
        _dest: SocketAddr,
        reader: Pin<Box<dyn AsyncRead + Send>>,
        writer: Pin<Box<dyn AsyncWrite + Send>>,
    ) -> Pin<Box<dyn std::future::Future<Output = network::interceptor::InterceptResult> + Send>>
    {
        // Clone the Arc fields to move into the async block.
        let tool_gate = self.tool_gate.clone();
        let mcp_clients = self.mcp_clients.clone();
        let scope_ref = self.scope.clone();

        Box::pin(async move {
            let interceptor = McpSdkInterceptor {
                tool_gate,
                mcp_clients,
                scope: scope_ref,
            };
            handle_http(reader, writer, &interceptor).await
        })
    }
}

/// Minimal HTTP/1.1 request handler for the interceptor.
///
/// Reads request headers + body, dispatches to [`McpSdkInterceptor::dispatch`],
/// and writes a response. Only POST /call is supported.
async fn handle_http(
    mut reader: Pin<Box<dyn AsyncRead + Send>>,
    mut writer: Pin<Box<dyn AsyncWrite + Send>>,
    interceptor: &McpSdkInterceptor,
) -> network::interceptor::InterceptResult {
    // Read headers.
    let mut header_buf = Vec::with_capacity(1024);
    let mut content_length: usize = 0;
    let mut buf = [0u8; 1];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        header_buf.push(buf[0]);
        // Detect end of headers (\r\n\r\n).
        if header_buf.ends_with(b"\r\n\r\n") || header_buf.ends_with(b"\n\n") {
            break;
        }
    }

    // Parse Content-Length from headers.
    let header_str = String::from_utf8_lossy(&header_buf);
    for line in header_str.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(n) = lower.split(':').nth(1).and_then(|s| s.trim().parse().ok()) {
                content_length = n;
            }
        }
    }

    // Read body.
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).await?;
    }

    // Dispatch.
    let (status, response_body) = match serde_json::from_slice::<ToolCallRequest>(&body) {
        Ok(req) => interceptor.dispatch(req).await,
        Err(e) => (
            400,
            serde_json::json!({"error": format!("invalid request body: {e}")}).to_string(),
        ),
    };

    // Write HTTP response.
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{response_body}",
        status = status,
        reason = if status == 200 { "OK" } else { "Error" },
        len = response_body.len(),
    );
    writer.write_all(response.as_bytes()).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Stub generation
// ---------------------------------------------------------------------------

/// Convert a `server_name/tool_name` to a Python function name.
fn python_fn_name(server: &str, tool: &str) -> String {
    format!(
        "{}_{tool}",
        server.replace('-', "_").replace('.', "_")
    )
}

/// Convert a `server_name/tool_name` to a camelCase TypeScript function name.
fn ts_fn_name(server: &str, tool: &str) -> String {
    let mut name = python_fn_name(server, tool);
    // Convert snake_case to camelCase.
    let mut result = String::with_capacity(name.len());
    let mut capitalize_next = false;
    for ch in name.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Extract parameter names from an MCP tool's input JSON Schema.
fn schema_params(schema: &serde_json::Value) -> Vec<(String, bool)> {
    let mut params = Vec::new();
    let required: Vec<&str> = schema
        .get("required")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    if let Some(props) = schema
        .get("properties")
        .and_then(|v| v.as_object())
    {
        for (k, _) in props {
            let is_required = required.contains(&k.as_str());
            params.push((k.clone(), is_required));
        }
    }
    params
}

/// Generate a `_mcp.py` stub file for a list of server+tools.
///
/// No credentials are included — only the magic gateway IP.
pub fn generate_python_stub(server: &str, tools: &[McpToolDef]) -> String {
    let mut lines = vec![
        "# auto-generated — do not edit".to_string(),
        "import urllib.request, json".to_string(),
        String::new(),
        format!("_MCP_URL = 'http://{SDK_INTERCEPTOR_IP}:{SDK_INTERCEPTOR_PORT}/call'"),
        String::new(),
    ];

    for tool in tools {
        let fn_name = python_fn_name(server, &tool.name);
        let params = schema_params(&tool.input_schema);

        let mut param_strs: Vec<String> = params
            .iter()
            .map(|(k, required)| {
                if *required {
                    k.clone()
                } else {
                    format!("{k}=None")
                }
            })
            .collect();

        let input_keys: Vec<String> = params
            .iter()
            .map(|(k, _)| format!("'{k}': {k}"))
            .collect();

        lines.push(format!(
            "def {fn_name}({params}) -> dict:",
            params = param_strs.join(", ")
        ));
        lines.push(format!("    \"\"\"{}\"\"\"", tool.description.replace('"', "'")));
        lines.push(format!(
            "    _req = {{'server': '{server}', 'tool': '{}', 'input': {{{input}}}}}",
            tool.name,
            input = input_keys.join(", ")
        ));
        lines.push(format!(
            "    _resp = urllib.request.urlopen(urllib.request.Request(_MCP_URL, json.dumps(_req).encode(), method='POST'))"
        ));
        lines.push("    return json.loads(_resp.read())".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Generate a `_mcp.ts` stub file for a list of server+tools.
///
/// No credentials are included — only the magic gateway IP.
pub fn generate_typescript_stub(server: &str, tools: &[McpToolDef]) -> String {
    let mut lines = vec![
        "// auto-generated — do not edit".to_string(),
        format!(
            "const MCP_URL = 'http://{SDK_INTERCEPTOR_IP}:{SDK_INTERCEPTOR_PORT}/call';"
        ),
        String::new(),
    ];

    for tool in tools {
        let fn_name = ts_fn_name(server, &tool.name);
        let params = schema_params(&tool.input_schema);

        let param_strs: Vec<String> = params
            .iter()
            .map(|(k, required)| {
                if *required {
                    format!("{k}: unknown")
                } else {
                    format!("{k}?: unknown")
                }
            })
            .collect();

        let input_keys: Vec<String> = params.iter().map(|(k, _)| k.clone()).collect();

        lines.push(format!(
            "export async function {fn_name}({params}): Promise<Record<string, unknown>> {{",
            params = param_strs.join(", ")
        ));
        lines.push(format!("  // {}", tool.description));
        lines.push("    const res = await fetch(MCP_URL, {".to_string());
        lines.push("        method: 'POST',".to_string());
        lines.push(format!(
            "        body: JSON.stringify({{server: '{server}', tool: '{}', input: {{{input}}}}}),",
            tool.name,
            input = input_keys.join(", ")
        ));
        lines.push("    });".to_string());
        lines.push("    if (!res.ok) throw new Error(await res.text());".to_string());
        lines.push("    return res.json();".to_string());
        lines.push("}".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

// ---------------------------------------------------------------------------
// McpSdkExtension
// ---------------------------------------------------------------------------

/// High-level extension coordinator: generates stubs and provides preamble injection.
pub struct McpSdkExtension {
    server_tools: Vec<(String, Vec<McpToolDef>)>,
}

impl McpSdkExtension {
    pub fn new(server_tools: Vec<(String, Vec<McpToolDef>)>) -> Self {
        Self { server_tools }
    }

    /// Write `_mcp.py` and `_mcp.ts` to `dir`.
    ///
    /// The files contain no credentials — only the magic gateway IP.
    pub fn write_stubs(&self, dir: &std::path::Path) -> anyhow::Result<()> {
        let mut py_sections = Vec::new();
        let mut ts_sections = Vec::new();

        for (server, tools) in &self.server_tools {
            py_sections.push(generate_python_stub(server, tools));
            ts_sections.push(generate_typescript_stub(server, tools));
        }

        std::fs::write(dir.join("_mcp.py"), py_sections.join("\n"))?;
        std::fs::write(dir.join("_mcp.ts"), ts_sections.join("\n"))?;
        Ok(())
    }

    /// Return Python import preamble lines for all generated functions.
    pub fn python_preamble(&self) -> String {
        let mut imports = Vec::new();
        for (server, tools) in &self.server_tools {
            let fns: Vec<String> = tools
                .iter()
                .map(|t| python_fn_name(server, &t.name))
                .collect();
            if !fns.is_empty() {
                imports.push(format!("from _mcp import {}", fns.join(", ")));
            }
        }
        if imports.is_empty() {
            return String::new();
        }
        format!("# auto-generated — do not edit\n{}\n", imports.join("\n"))
    }

    /// Return TypeScript import preamble lines for all generated functions.
    pub fn typescript_preamble(&self) -> String {
        let mut imports = Vec::new();
        for (server, tools) in &self.server_tools {
            let fns: Vec<String> = tools
                .iter()
                .map(|t| ts_fn_name(server, &t.name))
                .collect();
            if !fns.is_empty() {
                imports.push(format!(
                    "import {{ {} }} from './_mcp';",
                    fns.join(", ")
                ));
            }
        }
        if imports.is_empty() {
            return String::new();
        }
        format!("// auto-generated — do not edit\n{}\n", imports.join("\n"))
    }

    /// Prepend the MCP preamble to a Python script (for a copy written to the container).
    ///
    /// The original script content (stored in conversation history) is not modified.
    pub fn inject_python_preamble(&self, script: &str) -> String {
        let preamble = self.python_preamble();
        if preamble.is_empty() {
            return script.to_string();
        }
        format!("{preamble}\n{script}")
    }

    /// Prepend the MCP preamble to a TypeScript script (for a copy written to the container).
    ///
    /// The original script content (stored in conversation history) is not modified.
    pub fn inject_typescript_preamble(&self, script: &str) -> String {
        let preamble = self.typescript_preamble();
        if preamble.is_empty() {
            return script.to_string();
        }
        format!("{preamble}\n{script}")
    }

    /// System prompt fragment listing all available functions with Python signatures.
    pub fn system_prompt_fragment(&self) -> String {
        if self.server_tools.is_empty() {
            return String::new();
        }

        let mut lines = vec![
            "## MCP tool functions (auto-imported in Python and TypeScript scripts)".to_string(),
            String::new(),
            "These functions are available in every Python script via `from _mcp import ...`.".to_string(),
            "Call them directly — no import needed if preamble injection is active.".to_string(),
            String::new(),
        ];

        for (server, tools) in &self.server_tools {
            for tool in tools {
                let fn_name = python_fn_name(server, &tool.name);
                let params = schema_params(&tool.input_schema);
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|(k, required)| if *required { k.clone() } else { format!("{k}=None") })
                    .collect();
                lines.push(format!(
                    "- `{fn_name}({})` — {}",
                    param_strs.join(", "),
                    tool.description
                ));
            }
        }

        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_tools() -> Vec<McpToolDef> {
        vec![
            McpToolDef {
                name: "create_issue".to_string(),
                description: "Create a GitHub issue".to_string(),
                input_schema: json!({
                    "type": "object",
                    "required": ["owner", "repo", "title"],
                    "properties": {
                        "owner": {"type": "string"},
                        "repo": {"type": "string"},
                        "title": {"type": "string"},
                        "body": {"type": "string"}
                    }
                }),
                output_schema: None,
            },
            McpToolDef {
                name: "list_repos".to_string(),
                description: "List repositories".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {"org": {"type": "string"}}
                }),
                output_schema: None,
            },
        ]
    }

    // ── naming ────────────────────────────────────────────────────────────────

    #[test]
    fn python_fn_name_correct() {
        assert_eq!(python_fn_name("github", "create_issue"), "github_create_issue");
        assert_eq!(python_fn_name("my-server", "read_file"), "my_server_read_file");
    }

    #[test]
    fn ts_fn_name_correct() {
        assert_eq!(ts_fn_name("github", "create_issue"), "githubCreateIssue");
        assert_eq!(ts_fn_name("filesystem", "read_file"), "filesystemReadFile");
    }

    // ── stub generation ───────────────────────────────────────────────────────

    #[test]
    fn python_stub_contains_no_credentials() {
        let tools = sample_tools();
        let stub = generate_python_stub("github", &tools);
        for keyword in ["token", "api_key", "password", "secret", "bearer"] {
            assert!(
                !stub.to_lowercase().contains(keyword),
                "credential keyword '{keyword}' found in Python stub"
            );
        }
    }

    #[test]
    fn python_stub_uses_only_magic_ip() {
        let tools = sample_tools();
        let stub = generate_python_stub("github", &tools);
        assert!(stub.contains("169.254.1.2"), "magic IP not in stub");
        assert!(!stub.contains("github.com"), "real server URL in stub");
    }

    #[test]
    fn python_stub_has_function_for_each_tool() {
        let tools = sample_tools();
        let stub = generate_python_stub("github", &tools);
        assert!(stub.contains("def github_create_issue("));
        assert!(stub.contains("def github_list_repos("));
    }

    #[test]
    fn typescript_stub_contains_no_credentials() {
        let tools = sample_tools();
        let stub = generate_typescript_stub("github", &tools);
        for keyword in ["token", "api_key", "password", "secret", "bearer"] {
            assert!(
                !stub.to_lowercase().contains(keyword),
                "credential keyword '{keyword}' found in TypeScript stub"
            );
        }
    }

    #[test]
    fn typescript_stub_uses_only_magic_ip() {
        let tools = sample_tools();
        let stub = generate_typescript_stub("github", &tools);
        assert!(stub.contains("169.254.1.2"), "magic IP not in stub");
    }

    #[test]
    fn typescript_stub_has_function_for_each_tool() {
        let tools = sample_tools();
        let stub = generate_typescript_stub("github", &tools);
        assert!(stub.contains("githubCreateIssue"));
        assert!(stub.contains("githubListRepos"));
    }

    // ── McpSdkExtension ───────────────────────────────────────────────────────

    #[test]
    fn write_stubs_creates_both_files() {
        let tools = sample_tools();
        let ext = McpSdkExtension::new(vec![("github".to_string(), tools)]);
        let tmp = tempfile::tempdir().unwrap();

        ext.write_stubs(tmp.path()).unwrap();

        assert!(tmp.path().join("_mcp.py").exists());
        assert!(tmp.path().join("_mcp.ts").exists());
    }

    #[test]
    fn preamble_injection_prepends_imports() {
        let tools = sample_tools();
        let ext = McpSdkExtension::new(vec![("github".to_string(), tools)]);

        let script = "print('hello')";
        let injected = ext.inject_python_preamble(script);
        assert!(injected.starts_with("# auto-generated"));
        assert!(injected.contains("from _mcp import"));
        assert!(injected.contains(script));
    }

    #[test]
    fn original_script_not_modified() {
        let tools = sample_tools();
        let ext = McpSdkExtension::new(vec![("github".to_string(), tools)]);

        let original = "result = github_create_issue(owner='a', repo='b', title='c')";
        let injected = ext.inject_python_preamble(original);
        // Original is preserved verbatim in the injected version.
        assert!(injected.contains(original));
        // And the injected string is strictly longer.
        assert!(injected.len() > original.len());
    }

    #[test]
    fn system_prompt_lists_python_signatures() {
        let tools = sample_tools();
        let ext = McpSdkExtension::new(vec![("github".to_string(), tools)]);
        let fragment = ext.system_prompt_fragment();
        assert!(fragment.contains("github_create_issue"));
        assert!(fragment.contains("github_list_repos"));
    }

    // ── interceptor matching ──────────────────────────────────────────────────

    #[test]
    fn interceptor_matches_magic_ip_and_port() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let scope = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let interceptor = McpSdkInterceptor::new(gate, HashMap::new(), scope);

        let magic: SocketAddr =
            format!("{SDK_INTERCEPTOR_IP}:{SDK_INTERCEPTOR_PORT}").parse().unwrap();
        assert!(interceptor.matches(magic));

        let other: SocketAddr = "1.2.3.4:80".parse().unwrap();
        assert!(!interceptor.matches(other));
    }

    #[test]
    fn interceptor_does_not_match_graphql_ip() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let scope = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let interceptor = McpSdkInterceptor::new(gate, HashMap::new(), scope);

        // GraphQL interceptor uses 169.254.1.3 — must not match SDK interceptor.
        let graphql_ip: SocketAddr = "169.254.1.3:80".parse().unwrap();
        assert!(!interceptor.matches(graphql_ip));
    }
}
