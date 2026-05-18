//! Universal hick agent binary.
//!
//! Reads `.hick/agent.toml`, wires the enabled extension crates, and runs an
//! agent loop backed by [`hick_agent_sdk::AgentRuntime`].

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser;
use hick_agent_mcp_graphql::McpGraphqlExtension;
use hick_agent_mcp_sdk::McpSdkExtension;
use hick_agent_mcp_wasm::McpWasmExtension;
use hick_agent_mcp_xml::McpXmlExtension;
use futures::StreamExt as _;
use hick_agent_sdk::{
    AgentRuntime, AnthropicClient, HttpMcpClient, LlmClient, McpClient, McpCredentialStore,
    McpServersConfig, McpTransportKind, Message, OpenAiCompatClient, Role, ShapeFormatConfig,
    ApprovalResponse, SinkGuard, StdioMcpClient, ToolGate, ToolGateConfig, ToolTrust,
    network_filter::NetworkVerdict,
    tool_gate::ToolApprovalRequest,
};
use hick_classify::Classification;
use serde::Deserialize;
use tokio::sync::{Mutex, mpsc};

// ---------------------------------------------------------------------------
// Config schema
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct AgentConfig {
    agent: AgentSection,
    #[serde(default)]
    extensions: ExtensionsSection,
    #[serde(default)]
    mcp: McpSection,
    #[serde(default)]
    security: SecuritySection,
}

#[derive(Debug, Deserialize)]
struct AgentSection {
    model: String,
    #[serde(default)]
    system_prompt_extra: Option<String>,
    config_version: u32,
}

#[derive(Debug, Default, Deserialize)]
struct ExtensionsSection {
    #[serde(default)]
    script_first: bool,
    #[serde(default)]
    hick_edit: bool,
    #[serde(default)]
    mcp_wasm: bool,
    #[serde(default)]
    mcp_sdk: bool,
    #[serde(default)]
    mcp_xml: bool,
    #[serde(default)]
    mcp_graphql: bool,
}

#[derive(Debug, Default, Deserialize)]
struct McpSection {
    #[serde(default = "default_servers_file")]
    servers_file: String,
    #[serde(default = "default_secrets_file")]
    secrets_file: String,
    #[serde(default)]
    response_format: ResponseFormatSection,
}

fn default_servers_file() -> String {
    ".hick/mcp-servers.toml".into()
}
fn default_secrets_file() -> String {
    ".hick/mcp-secrets.toml".into()
}

#[derive(Debug, Deserialize)]
struct ResponseFormatSection {
    #[serde(default = "default_fmt")]
    default: String,
    #[serde(default)]
    by_shape: ByShapeSection,
}

impl Default for ResponseFormatSection {
    fn default() -> Self {
        Self {
            default: default_fmt(),
            by_shape: ByShapeSection::default(),
        }
    }
}

fn default_fmt() -> String {
    "json".into()
}

#[derive(Debug, Default, Deserialize)]
struct ByShapeSection {
    #[serde(default)]
    list: Option<String>,
    #[serde(default)]
    nested: Option<String>,
    #[serde(default)]
    scalar: Option<String>,
    #[serde(default)]
    object: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct SecuritySection {
    #[serde(default = "default_security_file")]
    config_file: String,
}

fn default_security_file() -> String {
    ".hick/agent-security.toml".into()
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    name = "hick-agent",
    about = "Universal hick AI coding agent",
    version
)]
struct Cli {
    /// Path to the agent config file (default: .hick/agent.toml).
    #[arg(long, default_value = ".hick/agent.toml")]
    config: PathBuf,

    /// Project directory (defaults to current working directory).
    #[arg(long)]
    project_dir: Option<PathBuf>,

    /// Task to execute. Reads from stdin if not provided.
    #[arg(long)]
    task: Option<String>,

    /// Override the model specified in the config.
    #[arg(long)]
    model: Option<String>,

    /// Maximum number of agent turns.
    #[arg(long, default_value_t = 50)]
    max_turns: usize,
}

// ---------------------------------------------------------------------------
// Provider detection
// ---------------------------------------------------------------------------

fn build_llm_client(model: &str) -> Box<dyn LlmClient> {
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        let key = std::env::var("ANTHROPIC_API_KEY").unwrap();
        return Box::new(AnthropicClient::with_config(model.to_string(), key));
    }
    if std::env::var("OPENAI_API_KEY").is_ok() {
        return Box::new(OpenAiCompatClient::openai(model.to_string()));
    }
    if std::env::var("XAI_API_KEY").is_ok() {
        return Box::new(OpenAiCompatClient::xai(model.to_string()));
    }
    log::warn!("no API key found; defaulting to Anthropic (key will be empty)");
    let key = String::new();
    Box::new(AnthropicClient::with_config(model.to_string(), key))
}

// ---------------------------------------------------------------------------
// MCP client construction
// ---------------------------------------------------------------------------

fn build_mcp_clients(
    servers: &McpServersConfig,
    creds: Arc<McpCredentialStore>,
) -> anyhow::Result<HashMap<String, Arc<dyn McpClient>>> {
    let mut clients: HashMap<String, Arc<dyn McpClient>> = HashMap::new();
    for srv in &servers.servers {
        let client: Arc<dyn McpClient> = match srv.transport {
            McpTransportKind::Stdio => {
                let cmd = srv.stdio_command()?.to_vec();
                Arc::new(StdioMcpClient::new(cmd))
            }
            McpTransportKind::Http => {
                let url = srv.http_url()?.to_string();
                Arc::new(HttpMcpClient::new(url, srv.name.clone(), creds.clone())?)
            }
        };
        clients.insert(srv.name.clone(), client);
    }
    Ok(clients)
}

fn server_trust_map(servers: &McpServersConfig) -> HashMap<String, ToolTrust> {
    servers
        .servers
        .iter()
        .map(|s| {
            let trust = match s.trust {
                hick_agent_sdk::McpServerTrust::Trusted => ToolTrust::Trusted,
                hick_agent_sdk::McpServerTrust::Untrusted => ToolTrust::Untrusted,
                hick_agent_sdk::McpServerTrust::Prompt => ToolTrust::Prompt,
            };
            (s.name.clone(), trust)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Format config construction
// ---------------------------------------------------------------------------

fn build_format_config(mcp: &McpSection) -> ShapeFormatConfig {
    use hick_agent_sdk::{ByShapeConfig, McpResponseFormat};

    let parse_fmt = |s: &str| match s {
        "json" => McpResponseFormat::Json,
        "markdown" => McpResponseFormat::Markdown,
        "xml" => McpResponseFormat::Xml,
        other => {
            log::warn!("unknown mcp.response_format value '{}', using json", other);
            McpResponseFormat::Json
        }
    };

    let default_fmt = parse_fmt(&mcp.response_format.default);
    let by_shape = ByShapeConfig {
        list: mcp.response_format.by_shape.list.as_deref().map(parse_fmt),
        nested: mcp
            .response_format
            .by_shape
            .nested
            .as_deref()
            .map(parse_fmt),
        scalar: mcp
            .response_format
            .by_shape
            .scalar
            .as_deref()
            .map(parse_fmt),
        object: mcp
            .response_format
            .by_shape
            .object
            .as_deref()
            .map(parse_fmt),
    };

    ShapeFormatConfig { default: default_fmt, by_shape }
}

// ---------------------------------------------------------------------------
// ToolGate with terminal approval handler
// ---------------------------------------------------------------------------

fn build_tool_gate(
    gate_config: ToolGateConfig,
    trust_map: HashMap<String, ToolTrust>,
) -> Arc<ToolGate> {
    let (approval_tx, mut approval_rx) = mpsc::unbounded_channel::<ToolApprovalRequest>();

    // Spawn a task that handles approval prompts by asking the user on stderr.
    tokio::spawn(async move {
        while let Some(req) = approval_rx.recv().await {
            eprint!(
                "[hick-agent] Allow tool call: {}/{} ? [y/N] ",
                req.server_name, req.tool_name
            );
            let _ = io::stderr().flush();
            let mut line = String::new();
            let _ = io::stdin().read_line(&mut line);
            let network_verdict = if line.trim().eq_ignore_ascii_case("y") {
                NetworkVerdict::Allow
            } else {
                NetworkVerdict::Block
            };
            let _ = req.response_tx.send(ApprovalResponse {
                verdict: network_verdict,
                always_allow: None,
            });
        }
    });

    Arc::new(ToolGate::new(
        gate_config,
        trust_map,
        ToolTrust::Prompt,
        approval_tx,
    ))
}

// ---------------------------------------------------------------------------
// System prompt assembly
// ---------------------------------------------------------------------------

fn build_system_prompt(
    base_extra: Option<&str>,
    mcp_xml: Option<&McpXmlExtension>,
    mcp_graphql: Option<&McpGraphqlExtension>,
    mcp_sdk: Option<&McpSdkExtension>,
    mcp_wasm: Option<&McpWasmExtension>,
    server_tools: &[(String, Vec<hick_agent_sdk::McpToolDef>)],
    script_first: bool,
    hick_edit: bool,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    parts.push(
        "You are an AI coding agent powered by hick.\n\n\
         Work step-by-step. When you use a tool, stop and wait for the result before continuing."
            .to_string(),
    );

    if script_first {
        parts.push(
            "## Code execution\n\
             Write fenced code blocks (```python, ```bash, etc.) to run code in the sandbox.\n\
             One block per response. Wait for the output before continuing."
                .to_string(),
        );
    }

    if hick_edit {
        parts.push(
            "## Hick pipeline editing\n\
             You have access to hick XML pipeline editing operations."
                .to_string(),
        );
    }

    if let Some(xml) = mcp_xml {
        parts.push(xml.system_prompt_fragment(server_tools));
    }
    if let Some(gql) = mcp_graphql {
        parts.push(gql.system_prompt_fragment());
    }
    if let Some(sdk) = mcp_sdk {
        parts.push(sdk.system_prompt_fragment());
    }
    if let Some(wasm) = mcp_wasm {
        parts.push(wasm.system_prompt_fragment());
    }

    if let Some(extra) = base_extra {
        parts.push(extra.to_string());
    }

    parts.join("\n\n---\n\n")
}

// ---------------------------------------------------------------------------
// Agent loop
// ---------------------------------------------------------------------------

async fn run_agent(
    mut runtime: AgentRuntime,
    system_prompt: String,
    task: String,
    mcp_xml: Option<Arc<McpXmlExtension>>,
    mcp_graphql: Option<Arc<McpGraphqlExtension>>,
    scope: Arc<Mutex<Vec<Classification>>>,
    max_turns: usize,
) -> anyhow::Result<()> {
    use hick_agent_sdk::runtime::Step;

    runtime.add_system_message(system_prompt);
    runtime.add_user_message(task);

    for turn in 0..max_turns {
        let mut stream = runtime.complete_stream().await?;
        let mut response = String::new();

        while let Some(step) = stream.next().await {
            match step? {
                Step::AssistantChunk(chunk) => {
                    print!("{}", chunk.delta);
                    let _ = io::stdout().flush();
                    response.push_str(&chunk.delta);
                }
                Step::AssistantDone(msg) => {
                    if response.is_empty() {
                        response = msg.content.clone();
                        print!("{}", response);
                        let _ = io::stdout().flush();
                    }
                    break;
                }
                Step::ContextWindowFull => {
                    runtime.compact_history(1, 20);
                    log::info!("context window compacted");
                    break;
                }
                _ => {}
            }
        }
        drop(stream);

        if response.is_empty() {
            log::warn!("empty response on turn {turn}, stopping");
            break;
        }

        runtime.add_message(Message {
            role: Role::Assistant,
            content: response.clone(),
        });

        let scope_snapshot = scope.lock().await.clone();

        // MCP XML extension — takes priority.
        if let Some(xml) = &mcp_xml {
            if let Some(obs) = xml.process_response(&response, &scope_snapshot).await {
                println!("\n[xml-tool-result]\n{obs}");
                runtime.add_user_message(obs);
                continue;
            }
        }

        // MCP GraphQL extension.
        if let Some(gql) = &mcp_graphql {
            if let Some(obs) = gql.process_response(&response).await {
                println!("\n[graphql-result]\n{obs}");
                runtime.add_user_message(obs);
                continue;
            }
        }

        // No extension consumed the response — we're done.
        println!();
        break;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();

    let project_dir = cli
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().expect("cannot read cwd"));

    // --- Load config --------------------------------------------------------

    let config_path = if cli.config.is_absolute() {
        cli.config.clone()
    } else {
        project_dir.join(&cli.config)
    };

    let config: AgentConfig = {
        let raw = std::fs::read_to_string(&config_path)
            .with_context(|| format!("reading config: {}", config_path.display()))?;
        toml::from_str(&raw)
            .with_context(|| format!("parsing config: {}", config_path.display()))?
    };

    if config.agent.config_version != 1 {
        anyhow::bail!(
            "unsupported config_version {}; only version 1 is supported",
            config.agent.config_version
        );
    }

    let model = cli.model.as_deref().unwrap_or(&config.agent.model);
    log::info!("model: {model}");

    // --- Load extra system prompt -------------------------------------------

    let system_prompt_extra: Option<String> = config
        .agent
        .system_prompt_extra
        .as_deref()
        .map(|p| {
            let full = if Path::new(p).is_absolute() {
                PathBuf::from(p)
            } else {
                project_dir.join(p)
            };
            std::fs::read_to_string(&full)
                .with_context(|| format!("reading system_prompt_extra: {}", full.display()))
        })
        .transpose()?;

    // --- Security config ----------------------------------------------------

    let security_path = project_dir.join(&config.security.config_file);
    let security = hick_agent_sdk::AgentSecurityConfig::load_or_default(&security_path)?;
    let capabilities = security.to_capabilities();
    let sink_defs = security.to_sink_defs();
    let gate_config = security.to_tool_gate_config();

    // --- MCP config ---------------------------------------------------------

    let servers_path = project_dir.join(&config.mcp.servers_file);
    let secrets_path = project_dir.join(&config.mcp.secrets_file);

    let servers = McpServersConfig::load_or_default(&servers_path)?;
    let creds = Arc::new(McpCredentialStore::load(&secrets_path)?);

    let mcp_clients = build_mcp_clients(&servers, creds)?;
    let trust_map = server_trust_map(&servers);

    // --- Fetch tool listings ------------------------------------------------

    let mut server_tools: Vec<(String, Vec<hick_agent_sdk::McpToolDef>)> = Vec::new();
    for (name, client) in &mcp_clients {
        match client.list_tools().await {
            Ok(tools) => {
                log::info!("server '{}': {} tools", name, tools.len());
                server_tools.push((name.clone(), tools));
            }
            Err(e) => {
                log::warn!("could not list tools from '{}': {e}", name);
            }
        }
    }

    let mcp_clients_arc: HashMap<String, Arc<dyn McpClient>> = mcp_clients;

    // --- ToolGate -----------------------------------------------------------

    let tool_gate = build_tool_gate(gate_config, trust_map);

    // --- Format config ------------------------------------------------------

    let format_config = build_format_config(&config.mcp);

    // --- Scope (shared, mutable) --------------------------------------------

    let scope: Arc<Mutex<Vec<Classification>>> = Arc::new(Mutex::new(Vec::new()));

    // --- Build enabled extensions -------------------------------------------

    let mcp_xml: Option<Arc<McpXmlExtension>> = if config.extensions.mcp_xml {
        log::info!("extension enabled: mcp_xml");
        Some(Arc::new(McpXmlExtension::new(
            tool_gate.clone(),
            mcp_clients_arc.clone(),
            format_config.clone(),
        )))
    } else {
        None
    };

    let mcp_graphql: Option<Arc<McpGraphqlExtension>> = if config.extensions.mcp_graphql {
        log::info!("extension enabled: mcp_graphql");
        Some(Arc::new(McpGraphqlExtension::new(
            server_tools.clone(),
            tool_gate.clone(),
            mcp_clients_arc.clone(),
            scope.clone(),
            format_config.clone(),
        )?))
    } else {
        None
    };

    let mcp_sdk: Option<McpSdkExtension> = if config.extensions.mcp_sdk {
        log::info!("extension enabled: mcp_sdk");
        Some(McpSdkExtension::new(server_tools.clone()))
    } else {
        None
    };

    let mcp_wasm: Option<McpWasmExtension> = if config.extensions.mcp_wasm {
        log::info!("extension enabled: mcp_wasm");
        Some(McpWasmExtension::new(tool_gate.clone(), mcp_clients_arc.clone()))
    } else {
        None
    };

    if config.extensions.script_first {
        log::info!("extension enabled: script_first");
    }
    if config.extensions.hick_edit {
        log::info!("extension enabled: hick_edit");
    }

    // --- Sink guard ---------------------------------------------------------

    let _sink_guard = Arc::new(SinkGuard::new(capabilities, sink_defs));

    // --- System prompt ------------------------------------------------------

    let system_prompt = build_system_prompt(
        system_prompt_extra.as_deref(),
        mcp_xml.as_deref(),
        mcp_graphql.as_deref(),
        mcp_sdk.as_ref(),
        mcp_wasm.as_ref(),
        &server_tools,
        config.extensions.script_first,
        config.extensions.hick_edit,
    );

    // --- Task ---------------------------------------------------------------

    let task = match cli.task {
        Some(t) => t,
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf.trim().to_string()
        }
    };

    if task.is_empty() {
        anyhow::bail!("no task provided (use --task or pipe to stdin)");
    }

    // --- Build LLM client and runtime ---------------------------------------

    let llm = build_llm_client(model);
    let runtime = AgentRuntime::new(llm, Box::new(NoopSandbox), project_dir.clone());

    // --- Run ----------------------------------------------------------------

    run_agent(
        runtime,
        system_prompt,
        task,
        mcp_xml,
        mcp_graphql,
        scope,
        cli.max_turns,
    )
    .await
}

// ---------------------------------------------------------------------------
// Noop sandbox — the hick-agent binary delegates real execution to extension
// crates (McpWasmContainerBackend etc.) or skips sandboxing entirely.
// ---------------------------------------------------------------------------

struct NoopSandbox;

#[async_trait::async_trait]
impl hick_agent_sdk::CommandExecutor for NoopSandbox {
    async fn execute(
        &mut self,
        _container: &str,
        _command: &str,
        _image: &str,
        _timeout: Option<u64>,
    ) -> anyhow::Result<hick_agent_sdk::CommandOutput> {
        Ok(hick_agent_sdk::CommandOutput {
            output: String::new(),
            exit_code: Some(0),
        })
    }
}
