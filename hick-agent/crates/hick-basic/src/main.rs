//! hick-basic: baseline AI coding agent.
//!
//! Launches the script-first ReAct agent TUI with no hick pipeline features.
//! Useful as an evaluation baseline: does the agent add value before any
//! hick-XML-aware extensions are added?

use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use hick_agent_core::{CommandExecutor, WasmSandboxConfig, WasmSandboxExecutor};
use tokio::sync::{Mutex, Notify, mpsc};

use hick_basic::tui::run_tui;
use hick_agent_script_first::{AgentCommand, AgentEvent, ReactLoop};
use hick_agent_sdk::{
    AgentSecurityConfig, AnthropicClient, FilteredLlmClient, LlmClient,
    NetworkApprovalRequest, NetworkFilter, OpenAiCompatClient, SinkGuard,
};
use hick_agent_sdk::network_filter::extract_host_from_url;

#[derive(Parser)]
#[command(name = "hick-basic")]
#[command(about = "Baseline AI coding agent — script-first ReAct loop, works on any repo")]
struct Cli {
    /// Project directory (default: current dir)
    #[arg(short, long)]
    project_dir: Option<PathBuf>,
    /// Directory containing .wasm container images
    #[arg(long, env = "HICK_IMAGES_DIR")]
    images_dir: Option<PathBuf>,
    /// Default container image
    #[arg(long, default_value = "alpine")]
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
    /// Disable streaming output
    #[arg(long)]
    no_stream: bool,
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
             \x20 hick-basic --provider anthropic"
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
    env_logger::init();

    let cli = Cli::parse();

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
        auth: None, // no auth injection in hick-basic
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
        .with_sink_guard(sink_guard);

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

    setup_sdk(&project_dir);

    event_tx
        .send(AgentEvent::Info(format!(
            "Project: {}",
            project_dir.display()
        )))
        .ok();

    let provider_info = format!("{provider_name} / {model_name}");

    let log_path = {
        let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let log_dir = project_dir.join(".hick");
        std::fs::create_dir_all(&log_dir).ok();
        log_dir.join(format!("basic-{ts}.hick"))
    };

    let interrupt_agent = Arc::clone(&interrupt);
    tokio::spawn(agent_task(react_loop, event_tx, cmd_rx, interrupt_agent));

    let interrupt_tui = Arc::clone(&interrupt);
    tokio::task::spawn_blocking(move || {
        run_tui(
            event_rx,
            cmd_tx,
            interrupt_tui,
            provider_info,
            approval_rx,
            Some(log_path),
        )
    })
    .await??;

    Ok(())
}

fn setup_sdk(project_dir: &std::path::Path) {
    let sdk_dest = project_dir.join(".hick").join("sdk").join("hickory");
    if sdk_dest.join("__init__.py").exists() {
        return;
    }

    let sdk_source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("sdk")
        .join("hickory");

    if !sdk_source.exists() {
        return;
    }

    if let Err(e) = std::fs::create_dir_all(&sdk_dest) {
        eprintln!("Warning: could not create SDK dir: {e}");
        return;
    }

    for entry in ["__init__.py", "client.py"] {
        let src = sdk_source.join(entry);
        let dst = sdk_dest.join(entry);
        if src.exists()
            && let Err(e) = std::fs::copy(&src, &dst)
        {
            eprintln!("Warning: could not copy SDK file {entry}: {e}");
        }
    }
}
