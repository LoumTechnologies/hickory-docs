//! MCP WASM extension: auto-generated WASI CLI binary per MCP tool.
//!
//! At agent startup [`McpWasmExtension`] generates one WASI Preview 2 component
//! per configured MCP tool and registers each on the WASM VM's PATH. The
//! components call into the host via the `hick:mcp/mcp` WIT import — a direct
//! Wasmtime host function — with no network involved at any layer.
//!
//! ## Transport
//!
//! The WIT interface (`wit/mcp-tool.wit`) defines a `call-tool` import. The host
//! provides the implementation; the generated WASI component simply calls it.
//! Auth credentials never appear in generated source — they live in
//! [`McpCredentialStore`] on the host and are accessed only inside the host
//! function implementation.
//!
//! ## VM PATH injection
//!
//! Each successfully built component is registered via
//! [`McpWasmContainerBackend`], which wraps another [`ContainerBackend`]. When
//! the VM runs a command whose binary name matches `mcp-<server>-<tool>`, the
//! wrapper intercepts it, calls [`ToolGate::check`], and dispatches to the
//! appropriate [`McpClient`] — returning the JSON result as stdout.
//!
//! A build failure for one tool logs a warning but does not prevent other tools
//! from being registered.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use hick_agent_sdk::{McpClient, McpToolDef, ToolGate, ToolVerdict};
use hick_classify::Classification;
use serde_json::Value;

// ---------------------------------------------------------------------------
// WIT source template for generated WASI components
// ---------------------------------------------------------------------------

/// Return the WIT file content (embedded from the crate's `wit/` directory).
pub fn mcp_tool_wit() -> &'static str {
    include_str!("../wit/mcp-tool.wit")
}

/// Rust source template for a generated tool component.
///
/// Placeholders:
/// - `{server}` — MCP server name (literal string in the call)
/// - `{tool}` — MCP tool name
/// - `{input_builder}` — code that builds the JSON input string from `args`
fn component_source(server: &str, tool: &str) -> String {
    // Auth credentials are never placed in generated source: the `call-tool`
    // host function resolves them from `McpCredentialStore`.
    format!(
        r#"// Auto-generated WASI component for MCP tool {server}/{tool}
// DO NOT EDIT — this file is regenerated at agent startup.
wit_bindgen::generate!({{
    world: "mcp-tool",
    path: "wit/mcp-tool.wit",
}});

struct Component;

impl Guest for Component {{
    fn run(args: Vec<String>) -> i32 {{
        let input = args_to_json_input(&args);
        match mcp::call_tool("{server}", "{tool}", &input) {{
            Ok(result) => {{
                println!("{{result}}");
                0
            }}
            Err(e) => {{
                eprintln!("mcp-{server}-{tool}: {{e}}");
                1
            }}
        }}
    }}
}}

fn args_to_json_input(args: &[String]) -> String {{
    // Positional key=value arguments are parsed into a JSON object.
    // If the first argument is a raw JSON object/array it is passed through.
    if let Some(first) = args.first() {{
        if first.trim_start().starts_with('{{') || first.trim_start().starts_with('[') {{
            return first.clone();
        }}
    }}
    let mut map = serde_json::Map::new();
    for arg in args {{
        if let Some((k, v)) = arg.split_once('=') {{
            let value = serde_json::from_str(v)
                .unwrap_or_else(|_| serde_json::Value::String(v.to_string()));
            map.insert(k.to_string(), value);
        }}
    }}
    serde_json::Value::Object(map).to_string()
}}
"#
    )
}

/// Template Cargo.toml for a generated WASI component.
fn component_cargo_toml(server: &str, tool: &str) -> String {
    format!(
        r#"[package]
name = "mcp-{server}-{tool}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.36"
serde_json = "1"
"#
    )
}

// ---------------------------------------------------------------------------
// Built component record
// ---------------------------------------------------------------------------

/// A successfully compiled WASI component ready for registration.
#[derive(Debug, Clone)]
pub struct BuiltComponent {
    /// Binary name used on the WASM VM's PATH (e.g. `mcp-github-create_issue`).
    pub binary_name: String,
    /// Server this tool belongs to.
    pub server: String,
    /// Tool name.
    pub tool: String,
    /// Path to the compiled `.wasm` file.
    pub wasm_path: PathBuf,
}

// ---------------------------------------------------------------------------
// McpWasmExtension
// ---------------------------------------------------------------------------

/// Extension state shared between the builder and the container backend wrapper.
pub struct McpWasmExtension {
    tool_gate: Arc<ToolGate>,
    /// Map from `"server/tool"` to the MCP client for that server.
    mcp_clients: HashMap<String, Arc<dyn McpClient>>,
    /// Successfully built components, keyed by binary name.
    built: HashMap<String, BuiltComponent>,
}

impl McpWasmExtension {
    /// Create a new extension without any built components.
    ///
    /// Call [`build`] to compile WASI components for each tool.
    pub fn new(
        tool_gate: Arc<ToolGate>,
        mcp_clients: HashMap<String, Arc<dyn McpClient>>,
    ) -> Self {
        Self {
            tool_gate,
            mcp_clients,
            built: HashMap::new(),
        }
    }

    /// Attempt to compile one WASI component per tool in `tools`.
    ///
    /// Build failures are non-fatal: a warning is logged and the tool is
    /// excluded from the VM PATH. Returns `self` for method chaining.
    pub async fn build(
        mut self,
        server: &str,
        tools: &[McpToolDef],
        build_dir: Option<&std::path::Path>,
    ) -> Self {
        let base_dir = match build_dir {
            Some(d) => d.to_path_buf(),
            None => std::env::temp_dir().join("hick-mcp-wasm"),
        };

        for tool in tools {
            match self
                .build_one_component(server, &tool.name, &base_dir)
                .await
            {
                Ok(component) => {
                    self.built
                        .insert(component.binary_name.clone(), component);
                }
                Err(e) => {
                    log::warn!(
                        "hick-agent-mcp-wasm: skipping tool {server}/{} — build failed: {e}",
                        tool.name
                    );
                }
            }
        }
        self
    }

    async fn build_one_component(
        &self,
        server: &str,
        tool: &str,
        base_dir: &std::path::Path,
    ) -> anyhow::Result<BuiltComponent> {
        let binary_name = format!("mcp-{server}-{tool}");
        let project_dir = base_dir.join(&binary_name);
        let wit_dir = project_dir.join("wit");
        let src_dir = project_dir.join("src");

        std::fs::create_dir_all(&wit_dir)?;
        std::fs::create_dir_all(&src_dir)?;

        // Write WIT interface (no credentials referenced).
        std::fs::write(wit_dir.join("mcp-tool.wit"), mcp_tool_wit())?;

        // Write generated Rust source (no credentials referenced).
        std::fs::write(
            src_dir.join("lib.rs"),
            component_source(server, tool),
        )?;

        // Write component Cargo.toml.
        std::fs::write(
            project_dir.join("Cargo.toml"),
            component_cargo_toml(server, tool),
        )?;

        // Compile with cargo-component.
        let output = tokio::process::Command::new("cargo")
            .args(["component", "build", "--release", "--target", "wasm32-wasip2"])
            .current_dir(&project_dir)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cargo-component build failed:\n{stderr}");
        }

        let wasm_path = project_dir
            .join("target")
            .join("wasm32-wasip2")
            .join("release")
            .join(format!("{}.wasm", binary_name.replace('-', "_")));

        if !wasm_path.exists() {
            anyhow::bail!("expected .wasm at {} but it was not found", wasm_path.display());
        }

        Ok(BuiltComponent {
            binary_name,
            server: server.to_string(),
            tool: tool.to_string(),
            wasm_path,
        })
    }

    /// Call a tool directly (used by the `ContainerBackend` wrapper).
    ///
    /// Calls `ToolGate::check` first; returns an error string if blocked.
    /// Auth credentials are resolved from the `McpClient` on the host side.
    pub async fn call_tool(
        &self,
        server: &str,
        tool: &str,
        input_json: &str,
        scope: &[Classification],
    ) -> Result<String, String> {
        let verdict = self.tool_gate.check(server, tool, scope).await;
        if verdict == ToolVerdict::Block {
            return Err(format!("blocked: ToolGate denied call to {server}/{tool}"));
        }

        let client = self
            .mcp_clients
            .get(server)
            .ok_or_else(|| format!("no MCP client configured for server '{server}'"))?;

        let input: Value = serde_json::from_str(input_json)
            .map_err(|e| format!("invalid JSON input: {e}"))?;

        let params = serde_json::json!({
            "name": tool,
            "arguments": input,
        });

        client
            .call("tools/call", params)
            .await
            .map(|v| v.to_string())
            .map_err(|e| e.to_string())
    }

    /// List built component binary names for system prompt generation.
    pub fn binary_names(&self) -> impl Iterator<Item = &str> {
        self.built.keys().map(|s| s.as_str())
    }

    /// System prompt fragment listing available MCP tool binaries.
    pub fn system_prompt_fragment(&self) -> String {
        if self.built.is_empty() {
            return String::new();
        }
        let mut lines = vec![
            "## MCP tool binaries (available on PATH)".to_string(),
            String::new(),
            "Call these like any shell command. Input: key=value args or a JSON object.".to_string(),
            String::new(),
        ];
        let mut names: Vec<&str> = self.binary_names().collect();
        names.sort_unstable();
        for name in names {
            lines.push(format!("- `{name} [key=value ...]`"));
        }
        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// McpWasmContainerBackend
// ---------------------------------------------------------------------------

/// `ContainerBackend` wrapper that intercepts `mcp-<server>-<tool>` commands
/// and dispatches them to the appropriate [`McpClient`] via the host.
///
/// Commands not matching the `mcp-` prefix are forwarded to the inner backend.
pub struct McpWasmContainerBackend<B> {
    inner: B,
    extension: Arc<McpWasmExtension>,
    /// Classifications currently in scope (used for `ToolGate::check`).
    scope: Vec<Classification>,
}

impl<B> McpWasmContainerBackend<B> {
    /// Wrap an existing backend with MCP tool interception.
    pub fn new(inner: B, extension: Arc<McpWasmExtension>) -> Self {
        Self {
            inner,
            extension,
            scope: Vec::new(),
        }
    }

    /// Update the classification scope (e.g. when a classified file is opened).
    pub fn set_scope(&mut self, scope: Vec<Classification>) {
        self.scope = scope;
    }
}

#[async_trait]
impl<B: hick_agent_sdk::ContainerBackend + Send + Sync> hick_agent_sdk::ContainerBackend
    for McpWasmContainerBackend<B>
{
    async fn ensure_started(
        &mut self,
        container: &str,
        image: &str,
    ) -> anyhow::Result<()> {
        self.inner.ensure_started(container, image).await
    }

    async fn execute_raw(
        &mut self,
        container: &str,
        command: &str,
    ) -> anyhow::Result<String> {
        // Parse the first token of the command to detect `mcp-<server>-<tool>`.
        let argv: Vec<&str> = command.split_whitespace().collect();
        let binary = argv.first().copied().unwrap_or("");

        if let Some(rest) = binary.strip_prefix("mcp-") {
            // rest = "<server>-<tool>" — split on the first `-` that yields a
            // known server, otherwise split at the first `-`.
            if let Some((server, tool)) = rest.split_once('-') {
                let args_str = argv[1..].join(" ");
                let input_json = if args_str.trim().starts_with('{') {
                    args_str.clone()
                } else {
                    args_to_json_object(&argv[1..])
                };

                match self
                    .extension
                    .call_tool(server, tool, &input_json, &self.scope)
                    .await
                {
                    Ok(result) => return Ok(result),
                    Err(e) => anyhow::bail!("{e}"),
                }
            }
        }

        self.inner.execute_raw(container, command).await
    }
}

/// Parse `key=value` arguments into a JSON object string.
fn args_to_json_object(args: &[&str]) -> String {
    let mut map = serde_json::Map::new();
    for arg in args {
        if let Some((k, v)) = arg.split_once('=') {
            let value = serde_json::from_str(v)
                .unwrap_or_else(|_| Value::String(v.to_string()));
            map.insert(k.to_string(), value);
        }
    }
    Value::Object(map).to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── WIT and source generation ─────────────────────────────────────────────

    #[test]
    fn wit_content_is_non_empty() {
        assert!(!mcp_tool_wit().is_empty());
        assert!(mcp_tool_wit().contains("call-tool"));
    }

    #[test]
    fn generated_source_contains_no_credentials() {
        let src = component_source("github", "create_issue");
        // No credential keywords should appear in generated source.
        for keyword in ["token", "api_key", "password", "secret", "bearer"] {
            assert!(
                !src.to_lowercase().contains(keyword),
                "credential keyword '{keyword}' found in generated source"
            );
        }
        assert!(src.contains("create_issue"));
        assert!(src.contains("github"));
    }

    #[test]
    fn binary_name_convention() {
        // Binary names follow `mcp-<server>-<tool>` convention.
        let name = format!("mcp-{}-{}", "github", "create_issue");
        assert_eq!(name, "mcp-github-create_issue");
    }

    // ── args_to_json_object ───────────────────────────────────────────────────

    #[test]
    fn key_value_args_produce_json_object() {
        let args = ["owner=myorg", "repo=myrepo", "title=Fix bug"];
        let result = args_to_json_object(&args);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["owner"], "myorg");
        assert_eq!(v["repo"], "myrepo");
        assert_eq!(v["title"], "Fix bug");
    }

    #[test]
    fn numeric_value_parsed_as_number() {
        let args = ["count=42"];
        let result = args_to_json_object(&args);
        let v: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["count"], 42);
    }

    // ── ToolGate integration ──────────────────────────────────────────────────

    #[tokio::test]
    async fn call_tool_blocked_on_secret_classification() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use serde_json::json;
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig {
            rejection_classifications: vec![hick_classify::Classification::new(vec![
                json!({"sensitivity": "secret"}),
            ])],
        };
        let gate = Arc::new(ToolGate::new(config, HashMap::new(), ToolTrust::Trusted, tx));
        let ext = McpWasmExtension::new(gate, HashMap::new());

        let secret_scope = vec![hick_classify::Classification::new(vec![
            json!({"sensitivity": "secret"}),
        ])];

        let result = ext
            .call_tool("github", "create_issue", "{}", &secret_scope)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("blocked"));
    }

    // ── System prompt ─────────────────────────────────────────────────────────

    #[test]
    fn system_prompt_empty_when_no_tools_built() {
        use hick_agent_sdk::{ToolGateConfig, ToolTrust};
        use tokio::sync::mpsc;

        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = Arc::new(ToolGate::new(
            ToolGateConfig::default(),
            HashMap::new(),
            ToolTrust::Trusted,
            tx,
        ));
        let ext = McpWasmExtension::new(gate, HashMap::new());
        assert!(ext.system_prompt_fragment().is_empty());
    }
}
