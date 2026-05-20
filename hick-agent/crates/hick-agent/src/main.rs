//! Universal hick agent binary.
//!
//! Reads `.hick/agent.toml` (or `--config <path>`), wires the enabled
//! extension crates, and runs an agent loop.

mod config;

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context as _;
use clap::Parser;
use config::{AgentConfig, ModelSection, PointConfig, Provider, SearchSpaceConfig};
use hick_agent_mcp_graphql::McpGraphqlExtension;
use hick_agent_mcp_sdk::McpSdkExtension;
use hick_agent_mcp_wasm::McpWasmExtension;
use hick_agent_mcp_xml::McpXmlExtension;
use futures::StreamExt as _;
use hick_agent_sdk::{
    AgentRuntime, AnthropicClient, HttpMcpClient, LlmClient, McpClient, McpCredentialStore,
    McpServersConfig, McpTransportKind, Message, OpenAiCompatClient, Role, ShapeFormatConfig,
    ApprovalResponse, SinkGuard, StdioMcpClient, ToolGate, ToolGateConfig, ToolTrust,
    XmlDiagnosticLogger, NullSessionLog,
    network_filter::NetworkVerdict,
    tool_gate::ToolApprovalRequest,
};
use hick_classify::Classification;
use tokio::sync::{Mutex, mpsc};

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
    /// Path to the agent config file.
    /// Resolution order when omitted: .hick/agent.toml → ~/.hick/agent.toml.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Project directory (defaults to current working directory).
    #[arg(long)]
    project_dir: Option<PathBuf>,

    /// Print all concrete trial configs as TOML and exit (requires a search-
    /// space config; errors on a point config).
    #[arg(long)]
    expand: bool,

    /// Task to execute. Reads from stdin if not provided.
    #[arg(long)]
    task: Option<String>,

    /// Override the model ID specified in the config.
    #[arg(long)]
    model: Option<String>,

    /// Maximum number of agent turns.
    #[arg(long, default_value_t = 50)]
    max_turns: usize,
}

// ---------------------------------------------------------------------------
// Provider / LLM client construction
// ---------------------------------------------------------------------------

fn build_llm_client(model: &ModelSection, model_override: Option<&str>) -> Box<dyn LlmClient> {
    let model_id = model_override.unwrap_or(&model.model_id);

    let api_key_env = model.api_key_env.as_deref().unwrap_or_else(|| match model.provider {
        Provider::Anthropic => "ANTHROPIC_API_KEY",
        Provider::OpenAI => "OPENAI_API_KEY",
        Provider::XAI => "XAI_API_KEY",
    });

    match model.provider {
        Provider::Anthropic => {
            let key = std::env::var(api_key_env).unwrap_or_default();
            Box::new(AnthropicClient::with_config(model_id.to_string(), key))
        }
        Provider::OpenAI => {
            Box::new(OpenAiCompatClient::openai(model_id.to_string()))
        }
        Provider::XAI => {
            Box::new(OpenAiCompatClient::xai(model_id.to_string()))
        }
    }
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

fn build_format_config(mcp: &config::McpSection) -> ShapeFormatConfig {
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
        nested: mcp.response_format.by_shape.nested.as_deref().map(parse_fmt),
        scalar: mcp.response_format.by_shape.scalar.as_deref().map(parse_fmt),
        object: mcp.response_format.by_shape.object.as_deref().map(parse_fmt),
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

        if let Some(xml) = &mcp_xml {
            if let Some(obs) = xml.process_response(&response, &scope_snapshot).await {
                println!("\n[xml-tool-result]\n{obs}");
                runtime.add_user_message(obs);
                continue;
            }
        }

        if let Some(gql) = &mcp_graphql {
            if let Some(obs) = gql.process_response(&response).await {
                println!("\n[graphql-result]\n{obs}");
                runtime.add_user_message(obs);
                continue;
            }
        }

        println!();
        break;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// run_point_config — wire a PointConfig and run the agent
// ---------------------------------------------------------------------------

async fn run_point_config(
    point: PointConfig,
    model_override: Option<&str>,
    task: String,
    project_dir: PathBuf,
    max_turns: usize,
) -> anyhow::Result<()> {
    log::info!("model: {}", model_override.unwrap_or(&point.model.model_id));

    // Security config (inline in [security] section).
    let security = &point.security;
    let capabilities = security.to_capabilities();
    let sink_defs = security.to_sink_defs();
    let gate_config = security.to_tool_gate_config();

    // Extra system prompt.
    let system_prompt_extra: Option<String> = point
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

    // MCP config.
    let servers_path = project_dir.join(&point.mcp.servers_file);
    let secrets_path = project_dir.join(&point.mcp.secrets_file);
    let servers = McpServersConfig::load_or_default(&servers_path)?;
    let creds = Arc::new(McpCredentialStore::load(&secrets_path)?);
    let mcp_clients = build_mcp_clients(&servers, creds)?;
    let trust_map = server_trust_map(&servers);

    // Tool listings.
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
    let tool_gate = build_tool_gate(gate_config, trust_map);
    let format_config = build_format_config(&point.mcp);
    let scope: Arc<Mutex<Vec<Classification>>> = Arc::new(Mutex::new(Vec::new()));

    // Build enabled extensions.
    let mcp_xml: Option<Arc<McpXmlExtension>> = if point.extensions.mcp_xml {
        log::info!("extension enabled: mcp_xml");
        Some(Arc::new(McpXmlExtension::new(
            tool_gate.clone(),
            mcp_clients_arc.clone(),
            format_config.clone(),
        )))
    } else {
        None
    };

    let mcp_graphql: Option<Arc<McpGraphqlExtension>> = if point.extensions.mcp_graphql {
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

    let mcp_sdk: Option<McpSdkExtension> = if point.extensions.mcp_sdk {
        log::info!("extension enabled: mcp_sdk");
        Some(McpSdkExtension::new(server_tools.clone()))
    } else {
        None
    };

    let mcp_wasm: Option<McpWasmExtension> = if point.extensions.mcp_wasm {
        log::info!("extension enabled: mcp_wasm");
        Some(McpWasmExtension::new(tool_gate.clone(), mcp_clients_arc.clone()))
    } else {
        None
    };

    if point.extensions.script_first {
        log::info!("extension enabled: script_first");
    }
    if point.extensions.hick_edit {
        log::info!("extension enabled: hick_edit");
    }

    let _sink_guard = Arc::new(SinkGuard::new(capabilities, sink_defs));

    let system_prompt = build_system_prompt(
        system_prompt_extra.as_deref(),
        mcp_xml.as_deref(),
        mcp_graphql.as_deref(),
        mcp_sdk.as_ref(),
        mcp_wasm.as_ref(),
        &server_tools,
        point.extensions.script_first,
        point.extensions.hick_edit,
    );

    let llm = build_llm_client(&point.model, model_override);
    let runtime = AgentRuntime::new(llm, Box::new(NoopSandbox), project_dir);

    run_agent(runtime, system_prompt, task, mcp_xml, mcp_graphql, scope, max_turns).await
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    log::set_boxed_logger(Box::new(XmlDiagnosticLogger::new(Arc::new(NullSessionLog)))).ok();
    log::set_max_level(log::LevelFilter::Info);

    let cli = Cli::parse();

    let project_dir = cli
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().expect("cannot read cwd"));

    // Resolve config path: explicit --config resolves relative to cwd.
    let explicit_path: Option<PathBuf> = cli.config.map(|p| {
        if p.is_absolute() { p } else { std::env::current_dir().unwrap().join(p) }
    });

    let agent_config =
        AgentConfig::load(explicit_path.as_deref(), &project_dir)?;

    // Handle --expand (AC #6): print trial configs as TOML and exit.
    if cli.expand {
        match agent_config {
            AgentConfig::Search(sc) => {
                let trials = sc.expand_trials();
                let total = trials.len();
                for (i, trial) in trials.iter().enumerate() {
                    println!("# --- Trial {} of {} ---", i + 1, total);
                    print!("{}", SearchSpaceConfig::format_trial_toml(trial));
                    if i + 1 < total {
                        println!();
                    }
                }
                return Ok(());
            }
            AgentConfig::Point(_) => {
                anyhow::bail!(
                    "--expand requires a search-space config (one with a [search] section)"
                );
            }
        }
    }

    // AC #7: reject search-space configs without --expand.
    match agent_config {
        AgentConfig::Search(_) => {
            anyhow::bail!(
                "config contains a [search] section; run with --expand to list \
                 trial configs, or remove [search] to run a single agent"
            );
        }
        AgentConfig::Point(point) => {
            // Task input.
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

            run_point_config(
                point,
                cli.model.as_deref(),
                task,
                project_dir,
                cli.max_turns,
            )
            .await
        }
    }
}

// ---------------------------------------------------------------------------
// Noop sandbox
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
