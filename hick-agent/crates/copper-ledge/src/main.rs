//! copper-ledge: standard AI coding agent.
//!
//! Launches the script-first ReAct agent TUI with no hick pipeline features.
//! Works on any codebase. Useful as an evaluation baseline against iron-peak.

use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use hick_agent_core::{CommandExecutor, WasmSandboxConfig, WasmSandboxExecutor};
use hick_agent_sdk::mcp_install::{
    McpInstaller, infer_name, list_servers, parse_spec, remove_server,
};
use hick_agent_sdk::mcp_config::McpServerTrust;
use tokio::sync::{Mutex, Notify, broadcast, mpsc};

use hick_agent_script_first::{
    AgentCommand, AgentEvent, ReactLoop,
    run_ui_event_fanout_task, ui_command_to_agent,
};
use hick_agent_tui::{run_tui, TuiConfig};
use hick_agent_sdk::{
    AgentSecurityConfig, AnthropicClient, FilteredLlmClient, LlmClient,
    NetworkApprovalRequest, NetworkFilter, OpenAiCompatClient, SinkGuard,
    XmlDiagnosticLogger, XmlSessionLog, NullSessionLog,
};
use hick_agent_sdk::network_filter::extract_host_from_url;

#[derive(Parser)]
#[command(name = "copper-ledge")]
#[command(about = "Standard AI coding agent — script-first ReAct loop, works on any repo")]
struct Cli {
    #[command(subcommand)]
    command: Option<TopCommand>,

    /// Project directory (default: current dir)
    #[arg(short, long, global = true)]
    project_dir: Option<PathBuf>,
    /// Directory containing .wasm container images
    #[arg(long, env = "HICK_IMAGES_DIR", global = true)]
    images_dir: Option<PathBuf>,
    /// Default container image
    #[arg(long, default_value = "alpine", global = true)]
    default_image: String,
    /// LLM provider (auto-detected from API keys if omitted)
    #[arg(long, value_enum, env = "HICK_AGENT_PROVIDER")]
    provider: Option<Provider>,
    /// Model name (defaults to provider's recommended model)
    #[arg(long, env = "HICK_AGENT_MODEL")]
    model: Option<String>,
    /// API base URL (for openai-compat, or to override a provider's default)
    #[arg(long, env = "HICK_AGENT_API_BASE")]
    api_base: Option<String>,
    /// API key (overrides provider-specific env vars like OPENAI_API_KEY)
    #[arg(long, env = "HICK_AGENT_API_KEY")]
    api_key: Option<String>,
    /// Disable local Web UI (also set `HICK_NO_WEB=1`).
    #[arg(long)]
    no_web: bool,
    /// Disable streaming output
    #[arg(long)]
    no_stream: bool,
}

#[derive(Subcommand)]
enum TopCommand {
    /// Manage MCP server installations.
    Mcp {
        #[command(subcommand)]
        cmd: McpCommand,
    },
}

#[derive(Subcommand)]
enum McpCommand {
    /// Install an MCP server and add it to .hick/mcp-servers.toml.
    Install(McpInstallArgs),
    /// List all configured MCP servers.
    List {
        /// Project directory (default: current dir).
        #[arg(long)]
        project_dir: Option<PathBuf>,
    },
    /// Remove a configured MCP server.
    Remove {
        /// Server name to remove.
        name: String,
        /// Project directory (default: current dir).
        #[arg(long)]
        project_dir: Option<PathBuf>,
    },
}

#[derive(clap::Args)]
struct McpInstallArgs {
    /// Stdio command (e.g. "npx -y @scope/server-github") or HTTP URL.
    spec: String,
    /// Server name (inferred from spec if omitted).
    #[arg(long)]
    name: Option<String>,
    /// Trust level: trusted, untrusted, or prompt (default).
    #[arg(long, default_value = "prompt")]
    trust: McpTrustArg,
    /// Project directory (default: current dir).
    #[arg(long)]
    project_dir: Option<PathBuf>,
    /// Directory containing .wasm container images (enables WASM package install).
    #[arg(long, env = "HICK_IMAGES_DIR")]
    install_images_dir: Option<PathBuf>,
    /// Default container image name (used with --install-images-dir).
    #[arg(long, default_value = "alpine")]
    install_default_image: String,
}

#[derive(Clone, Copy, ValueEnum)]
enum McpTrustArg {
    Trusted,
    Untrusted,
    Prompt,
}

impl From<McpTrustArg> for McpServerTrust {
    fn from(t: McpTrustArg) -> Self {
        match t {
            McpTrustArg::Trusted => McpServerTrust::Trusted,
            McpTrustArg::Untrusted => McpServerTrust::Untrusted,
            McpTrustArg::Prompt => McpServerTrust::Prompt,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum Provider {
    /// OpenAI API (OPENAI_API_KEY)
    Openai,
    /// Anthropic Claude API (ANTHROPIC_API_KEY)
    Anthropic,
    /// xAI/Grok API (XAI_API_KEY)
    Xai,
    /// Custom OpenAI-compatible endpoint (requires --api-base)
    #[value(name = "openai-compat")]
    OpenaiCompat,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Openai => write!(f, "OpenAI"),
            Provider::Anthropic => write!(f, "Anthropic"),
            Provider::Xai => write!(f, "xAI"),
            Provider::OpenaiCompat => write!(f, "OpenAI-compatible"),
        }
    }
}

struct DetectedProvider {
    provider: Provider,
    env_var: &'static str,
    default_model: &'static str,
}

fn detect_providers() -> Vec<DetectedProvider> {
    let mut found = vec![];
    let candidates: &[(&str, Provider, &str)] = &[
        (
            "ANTHROPIC_API_KEY",
            Provider::Anthropic,
            "claude-sonnet-4-5-20250929",
        ),
        ("OPENAI_API_KEY", Provider::Openai, "gpt-4o"),
        ("XAI_API_KEY", Provider::Xai, "grok-4.3"),
    ];
    for &(env_var, provider, default_model) in candidates {
        if std::env::var(env_var)
            .ok()
            .filter(|k| !k.is_empty())
            .is_some()
        {
            found.push(DetectedProvider {
                provider,
                env_var,
                default_model,
            });
        }
    }
    found
}

fn select_provider(detected: Vec<DetectedProvider>) -> Result<Provider> {
    if detected.is_empty() {
        anyhow::bail!(
            "No LLM provider specified and no API keys found in environment.\n\n\
             Set one of the following environment variables:\n\
             \x20 ANTHROPIC_API_KEY  — Anthropic Claude\n\
             \x20 OPENAI_API_KEY     — OpenAI\n\
             \x20 XAI_API_KEY        — xAI / Grok\n\n\
             Or specify a provider explicitly:\n\
             \x20 copper-ledge --provider anthropic"
        );
    }

    if detected.len() == 1 {
        return Ok(detected[0].provider);
    }

    println!("Multiple API keys detected:");
    for (i, d) in detected.iter().enumerate() {
        println!(
            "  [{}] {} — {} ({})",
            i + 1,
            d.provider,
            d.env_var,
            d.default_model,
        );
    }
    print!("Select provider [1-{}]: ", detected.len());
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        anyhow::bail!("No selection made. Use --provider to specify a provider explicitly.");
    }

    let idx: usize = input
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid selection: {input}"))?;
    if idx < 1 || idx > detected.len() {
        anyhow::bail!("Selection out of range: {idx}");
    }

    Ok(detected[idx - 1].provider)
}

fn build_llm_client(
    provider: Provider,
    model: Option<String>,
    api_base: Option<String>,
    api_key: Option<String>,
) -> Result<Box<dyn LlmClient>> {
    match provider {
        Provider::Openai => {
            let model = model.unwrap_or_else(|| "gpt-4o".into());
            if let Some(base) = api_base {
                let key =
                    api_key.unwrap_or_else(|| std::env::var("OPENAI_API_KEY").unwrap_or_default());
                Ok(Box::new(OpenAiCompatClient::custom(
                    base, key, model, "openai",
                )))
            } else {
                Ok(Box::new(OpenAiCompatClient::openai(model)))
            }
        }
        Provider::Anthropic => {
            let model = model.unwrap_or_else(|| "claude-sonnet-4-5-20250929".into());
            let key =
                api_key.unwrap_or_else(|| std::env::var("ANTHROPIC_API_KEY").unwrap_or_default());
            Ok(Box::new(AnthropicClient::with_config(model, key)))
        }
        Provider::Xai => {
            let model = model.unwrap_or_else(|| "grok-4.3".into());
            if let Some(key) = api_key {
                Ok(Box::new(OpenAiCompatClient::custom(
                    "https://api.x.ai/v1",
                    key,
                    model,
                    "xai",
                )))
            } else {
                Ok(Box::new(OpenAiCompatClient::xai(model)))
            }
        }
        Provider::OpenaiCompat => {
            let base = api_base.ok_or_else(|| {
                anyhow::anyhow!("--api-base is required for openai-compat provider")
            })?;
            let key = api_key.unwrap_or_default();
            let model = model.unwrap_or_else(|| "default".into());
            Ok(Box::new(OpenAiCompatClient::custom(
                base,
                key,
                model,
                "openai-compat",
            )))
        }
    }
}

fn endpoint_host_for_provider(provider: Provider, api_base: &Option<String>) -> String {
    if let Some(base) = api_base {
        extract_host_from_url(base).unwrap_or_else(|| "unknown".to_string())
    } else {
        match provider {
            Provider::Anthropic => "api.anthropic.com".to_string(),
            Provider::Openai => "api.openai.com".to_string(),
            Provider::Xai => "api.x.ai".to_string(),
            Provider::OpenaiCompat => "unknown".to_string(),
        }
    }
}

fn detect_images_dir(start: &std::path::Path) -> Option<(PathBuf, String)> {
    let rel = std::path::Path::new("native2wasm/crates/c2w/converted_images");
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join(rel);
        if candidate.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&candidate) {
                for entry in entries.flatten() {
                    let image_dir = entry.path();
                    if !image_dir.is_dir() {
                        continue;
                    }
                    if let Ok(files) = std::fs::read_dir(&image_dir) {
                        for file in files.flatten() {
                            let p = file.path();
                            if p.extension().and_then(|e| e.to_str()) == Some("wasm") {
                                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                    return Some((image_dir, stem.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn web_ui_enabled(no_web_cli: bool) -> bool {
    !no_web_cli
        && !matches!(
            std::env::var("HICK_NO_WEB").as_deref(),
            Ok("1") | Ok("true") | Ok("yes")
        )
}

async fn agent_task(
    mut react_loop: ReactLoop,
    events: mpsc::UnboundedSender<AgentEvent>,
    mut commands: mpsc::UnboundedReceiver<AgentCommand>,
    interrupt: Arc<Notify>,
) {
    while let Some(cmd) = commands.recv().await {
        match cmd {
            AgentCommand::Message(msg) => {
                if let Err(e) = react_loop.run(&msg, &events, &interrupt).await {
                    events.send(AgentEvent::Error(e.to_string())).ok();
                }
            }
            AgentCommand::DirectCommand(cmd) => {
                if let Err(e) = react_loop.run_direct_command(&cmd, &events).await {
                    events.send(AgentEvent::Error(e.to_string())).ok();
                }
            }
            AgentCommand::Quit => break,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Dispatch Mcp subcommand before the agent setup.
    if let Some(TopCommand::Mcp { cmd }) = cli.command {
        return mcp_cmd(cmd).await;
    }

    let project_dir = cli.project_dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    let (images_dir, default_image) = match cli.images_dir {
        Some(d) => (d, cli.default_image),
        None => match detect_images_dir(&project_dir) {
            Some((d, img)) => {
                println!("Auto-detected images: {} (image: {})", d.display(), img);
                (d, img)
            }
            None => (PathBuf::from("."), cli.default_image),
        },
    };

    let provider = match cli.provider {
        Some(p) => p,
        None => {
            let detected = detect_providers();
            select_provider(detected)?
        }
    };

    let llm = build_llm_client(provider, cli.model, cli.api_base.clone(), cli.api_key)?;

    let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<AgentCommand>();
    let interrupt = Arc::new(Notify::new());

    let security_config_path = project_dir.join(".hick").join("agent-security.toml");
    let security_config = AgentSecurityConfig::load_or_default(&security_config_path)?;
    let sandbox_caps = security_config.to_capabilities();

    let (approval_tx, approval_rx) = mpsc::unbounded_channel::<NetworkApprovalRequest>();
    let filter_config_path = project_dir.join(".hick").join("network-rules.toml");
    let host = endpoint_host_for_provider(provider, &cli.api_base);
    let llm: Box<dyn LlmClient> = if filter_config_path.exists() {
        let filter = Arc::new(
            NetworkFilter::load(&filter_config_path, approval_tx).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to load network rules from {}: {e}",
                    filter_config_path.display()
                )
            })?,
        );
        event_tx
            .send(AgentEvent::Info(format!(
                "Network filter: active ({} rules, default {})",
                filter.rule_count(),
                filter.default_action_name(),
            )))
            .ok();
        Box::new(FilteredLlmClient::new(llm, filter, host))
    } else if security_config.deny_all_network {
        let filter = Arc::new(NetworkFilter::from_capabilities(
            &sandbox_caps,
            Some(&host),
            approval_tx,
        ));
        event_tx
            .send(AgentEvent::Info(format!(
                "Network filter: derived from security config ({} rules, default {})",
                filter.rule_count(),
                filter.default_action_name(),
            )))
            .ok();
        Box::new(FilteredLlmClient::new(llm, filter, host))
    } else {
        drop(approval_tx);
        llm
    };

    let provider_name = llm.provider_name().to_string();
    let model_name = llm.model_name().to_string();

    let sink_defs = security_config.to_sink_defs();
    let sink_guard = Arc::new(SinkGuard::new(sandbox_caps.clone(), sink_defs));

    let caps_summary = if security_config.deny_all_network {
        format!(
            "deny-all + {} allowed hosts",
            security_config.allowed_hosts.len()
        )
    } else {
        "permissive (no deny-all)".to_string()
    };

    let sandbox_guest_network = !matches!(
        std::env::var("HICK_AGENT_DISABLE_SANDBOX_GUEST_NET").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    );

    // Warn if the WASM image file is missing but don't hard-fail — the sandbox
    // will report a clear error on first use.
    let wasm_image_path = images_dir.join(format!("{}.wasm", default_image));
    if !wasm_image_path.exists() {
        event_tx
            .send(AgentEvent::Info(format!(
                "Warning: WASM image not found at {} — sandbox will fail on first script execution",
                wasm_image_path.display()
            )))
            .ok();
    }

    let sandbox_config = WasmSandboxConfig {
        images_dir: images_dir.clone(),
        workdir: project_dir.clone(),
        default_image: default_image.clone(),
        capabilities: sandbox_caps,
        auth: None,
        guest_network_enabled: sandbox_guest_network,
    };
    let sandbox_executor: Arc<Mutex<dyn CommandExecutor>> =
        Arc::new(Mutex::new(WasmSandboxExecutor::new(sandbox_config)));
    event_tx
        .send(AgentEvent::Info(format!(
            "Sandbox: WASM container enabled ({caps_summary})"
        )))
        .ok();

    let mut react_loop = ReactLoop::new(llm, project_dir.clone())
        .with_executor(sandbox_executor, default_image.clone())
        .with_sink_guard(sink_guard)
        .with_agent_dir(".copper-ledge");

    // Wire up policy evaluator if configured.
    if let Some(ref policy_path) = security_config.policy_script {
        let resolved = project_dir.join(policy_path);
        let canonical_project = project_dir.canonicalize().unwrap_or_default();
        let is_valid_policy = resolved.exists()
            && resolved.is_file()
            && resolved
                .extension()
                .is_some_and(|ext| ext == "py")
            && resolved
                .canonicalize()
                .ok()
                .is_some_and(|p| p.starts_with(&canonical_project));

        if is_valid_policy {
            react_loop = react_loop.with_policy(hick_policy::PolicyEvaluator::new(
                resolved.to_str().unwrap_or(policy_path),
            ));
            event_tx
                .send(AgentEvent::Info(format!(
                    "Policy: active ({})",
                    resolved.display()
                )))
                .ok();
        } else if resolved.exists() {
            event_tx
                .send(AgentEvent::Info(format!(
                    "Policy: rejected ({}) — must be a .py file within project directory",
                    resolved.display()
                )))
                .ok();
        } else {
            event_tx
                .send(AgentEvent::Info(format!(
                    "Policy: script not found ({}), skipping",
                    resolved.display()
                )))
                .ok();
        }
    }

    react_loop.stream_output = !cli.no_stream;

    event_tx
        .send(AgentEvent::Info(format!(
            "Project: {}",
            project_dir.display()
        )))
        .ok();

    let provider_info = format!("{provider_name} / {model_name}");

    const UI_BROADCAST_CAP: usize = 2048;
    let (ui_fanout_tx, _) = broadcast::channel::<hick_agent_sdk::UiEvent>(UI_BROADCAST_CAP);
    let (ui_mpsc_tx, ui_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiEvent>();

    tokio::spawn({
        let mut sub = ui_fanout_tx.subscribe();
        let ui_mpsc_tx = ui_mpsc_tx.clone();
        async move {
            loop {
                match sub.recv().await {
                    Ok(ev) => {
                        if ui_mpsc_tx.send(ev).is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    });

    let (ui_cmd_tx, mut ui_cmd_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiCommand>();

    tokio::spawn(run_ui_event_fanout_task(
        event_rx,
        approval_rx,
        ui_fanout_tx.clone(),
        Some(model_name.clone()),
        Some(default_image.clone()),
        Some(provider_info.clone()),
    ));

    let web_ui_url = if web_ui_enabled(cli.no_web) {
        match hick_agent_sdk::WebServer::bind(hick_agent_sdk::WebServerConfig::from_env()).await {
            Ok(ws) => {
                ws.attach(ui_fanout_tx.clone(), ui_cmd_tx.clone());
                match ws.start().await {
                    Ok(url) => {
                        event_tx.send(AgentEvent::Info(format!("Web UI: {url}"))).ok();
                        Some(url)
                    }
                    Err(e) => {
                        event_tx.send(AgentEvent::Info(format!("Web UI failed to start: {e}"))).ok();
                        None
                    }
                }
            }
            Err(e) => {
                event_tx.send(AgentEvent::Info(format!("Web UI unavailable (bind failed): {e}"))).ok();
                None
            }
        }
    } else {
        None
    };

    let cmd_bridge_to_agent = cmd_tx.clone();
    let interrupt_ui = Arc::clone(&interrupt);
    tokio::spawn(async move {
        while let Some(u) = ui_cmd_rx.recv().await {
            if matches!(u, hick_agent_sdk::UiCommand::Cancel) {
                interrupt_ui.notify_one();
            }
            if let Some(c) = ui_command_to_agent(u) {
                cmd_bridge_to_agent.send(c).ok();
            }
        }
    });

    let session_log: Arc<dyn hick_agent_sdk::SessionLog> = {
        let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let bundle_dir = project_dir.join(".copper-ledge").join(format!("session-{ts}"));
        match XmlSessionLog::new(bundle_dir) {
            Ok(log) => Arc::new(log),
            Err(_) => Arc::new(NullSessionLog),
        }
    };
    log::set_boxed_logger(Box::new(XmlDiagnosticLogger::new(Arc::clone(&session_log))))
        .ok();
    log::set_max_level(log::LevelFilter::Info);

    let react_loop = react_loop.with_session_log(Arc::clone(&session_log));

    let interrupt_agent = Arc::clone(&interrupt);
    tokio::spawn(agent_task(react_loop, event_tx.clone(), cmd_rx, interrupt_agent));

    let interrupt_tui = Arc::clone(&interrupt);
    tokio::task::spawn_blocking(move || {
        run_tui(
            ui_rx,
            ui_cmd_tx,
            TuiConfig {
                agent_name: "copper-ledge".to_string(),
                provider_info,
                project_dir,
                interrupt: interrupt_tui,
                ui_fanout_tx: ui_fanout_tx.clone(),
                web_ui_url,
            },
        )
    })
    .await??;

    Ok(())
}

async fn mcp_cmd(cmd: McpCommand) -> Result<()> {
    match cmd {
        McpCommand::List { project_dir } => {
            let project_dir = project_dir.unwrap_or_else(|| std::env::current_dir().unwrap());
            let lines = list_servers(&project_dir)?;
            for l in lines {
                println!("{l}");
            }
            Ok(())
        }

        McpCommand::Remove { name, project_dir } => {
            let project_dir = project_dir.unwrap_or_else(|| std::env::current_dir().unwrap());
            remove_server(&name, &project_dir)?;
            println!("Removed MCP server '{name}'.");
            Ok(())
        }

        McpCommand::Install(args) => {
            let project_dir = args
                .project_dir
                .unwrap_or_else(|| std::env::current_dir().unwrap());
            let spec = parse_spec(&args.spec);
            let trust: McpServerTrust = args.trust.into();

            let name = match args.name {
                Some(n) => n,
                None => match infer_name(&spec) {
                    Some(n) => {
                        println!("Inferred server name: {n}");
                        n
                    }
                    None => {
                        print!("Server name: ");
                        std::io::stdout().flush()?;
                        let mut buf = String::new();
                        std::io::stdin().read_line(&mut buf)?;
                        let n = buf.trim().to_string();
                        if n.is_empty() {
                            anyhow::bail!("Server name is required.");
                        }
                        n
                    }
                },
            };

            if let Some(images_dir) = args.install_images_dir {
                println!("Installing package in WASM sandbox...");
                use hick_agent_core::WasmSandboxConfig;
                use hick_token::ContainerCapabilities;
                let caps = hick_agent_sdk::mcp_install::PackageManagerNetworkPolicy::allowed_hosts()
                    .iter()
                    .fold(
                        ContainerCapabilities::new().deny_all_network(),
                        |c, h| c.allow_network(&h.host, &h.port),
                    );
                let sandbox_config = WasmSandboxConfig {
                    images_dir,
                    workdir: project_dir.clone(),
                    default_image: args.install_default_image,
                    capabilities: caps,
                    auth: None,
                    guest_network_enabled: true,
                };
                let mut executor = WasmSandboxExecutor::new(sandbox_config);
                McpInstaller::run_package_install(
                    &spec,
                    Some(&mut SdkExecutorBridge(&mut executor)),
                )
                .await?;
                println!("Package install complete.");
            } else {
                println!(
                    "Skipping WASM package install (use --install-images-dir to enable). \
                     Server must be available on PATH."
                );
            }

            println!("Verifying server connection...");
            let tools = McpInstaller::verify_connection(&spec).await?;
            println!("Server responded with {} tool(s):", tools.len());
            for t in &tools {
                println!("  • {} — {}", t.name, t.description);
            }

            McpInstaller::write_config(&spec, &name, trust, &project_dir)?;
            println!("Wrote entry to .hick/mcp-servers.toml");

            let mut creds: Vec<(String, String)> = Vec::new();
            loop {
                print!("Credential env var name (blank to skip): ");
                std::io::stdout().flush()?;
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf)?;
                let env_var = buf.trim().to_string();
                if env_var.is_empty() {
                    break;
                }
                creds.push((env_var, String::new()));
            }
            McpInstaller::write_credentials(&name, creds, &project_dir)?;
            if project_dir.join(".hick").join("mcp-secrets.toml").exists() {
                println!("Wrote credentials to .hick/mcp-secrets.toml");
            }

            println!("\nMCP server '{}' installed successfully.", name);
            Ok(())
        }
    }
}

struct SdkExecutorBridge<'a>(&'a mut WasmSandboxExecutor);

#[async_trait::async_trait]
impl hick_agent_sdk::CommandExecutor for SdkExecutorBridge<'_> {
    async fn execute(
        &mut self,
        container: &str,
        command: &str,
        image: &str,
        timeout: Option<u64>,
    ) -> anyhow::Result<hick_agent_sdk::CommandOutput> {
        let out = self.0.execute(container, command, image, timeout).await?;
        Ok(hick_agent_sdk::CommandOutput {
            output: out.output,
            exit_code: out.exit_code,
        })
    }
}
