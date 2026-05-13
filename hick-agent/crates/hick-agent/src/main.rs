//! hick-agent: AI command-line coding agent.
//!
//! Script-first model: the LLM writes Python scripts, which are executed
//! and their output returned as observations. The Hick pipeline runs
//! transparently via a file watcher.
//!
//! The binary wires up the core [`ReactLoop`] to a ratatui TUI via
//! event channels, keeping all rendering in the TUI and all logic in
//! the core.

use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use hick_agent_core::{CommandExecutor, WasmSandboxConfig, WasmSandboxExecutor};
use hick_token::TokenAuthority;
use tokio::sync::{Mutex, Notify, mpsc};

#[cfg(feature = "local")]
use hick_agent_engine::LocalLlmClient;
use hick_agent::tui::run_tui;
use hick_agent_engine::{
    Agent, AgentCommand, AgentConfig, AgentEvent, AgentSecurityConfig, AnthropicClient,
    FilteredLlmClient, LlmClient, NetworkApprovalRequest, NetworkFilter, OpenAiCompatClient,
    PipelineWatcher, ReactLoop, SinkGuard,
};
use hick_agent_engine::network_filter::extract_host_from_url;

#[derive(Parser)]
#[command(name = "hick-agent")]
#[command(about = "AI coding agent using Hick XML — script-first model")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    /// Local GGUF model via llama.cpp (requires --model-path, build with --features local)
    Local,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the agent interactively
    Run {
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
        /// Path to a local GGUF model file (for --provider local)
        #[arg(long, env = "HICK_AGENT_MODEL_PATH")]
        model_path: Option<PathBuf>,
        /// GPU layers to offload (auto-calibrates if omitted; 0 = CPU only, 1000 = all)
        #[arg(long, env = "HICK_AGENT_GPU_LAYERS")]
        n_gpu_layers: Option<u32>,
        /// Context window size in tokens (for --provider local)
        #[arg(long, default_value = "4096", env = "HICK_AGENT_CTX")]
        n_ctx: u32,
        /// Maximum tokens to generate per response (for --provider local)
        #[arg(long, default_value = "4096", env = "HICK_AGENT_MAX_TOKENS")]
        max_tokens: u32,
        /// Disable streaming output
        #[arg(long)]
        no_stream: bool,
    },
}

#[allow(dead_code)] // Fields read only when `local` feature is enabled
struct LocalOpts {
    model_path: Option<PathBuf>,
    n_gpu_layers: Option<u32>,
    n_ctx: u32,
    max_tokens: u32,
    project_dir: PathBuf,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Openai => write!(f, "OpenAI"),
            Provider::Anthropic => write!(f, "Anthropic"),
            Provider::Xai => write!(f, "xAI"),
            Provider::OpenaiCompat => write!(f, "OpenAI-compatible"),
            Provider::Local => write!(f, "Local"),
        }
    }
}

struct DetectedProvider {
    provider: Provider,
    env_var: &'static str,
    default_model: &'static str,
}

/// Scan environment for known API keys and return all detected providers.
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

/// Prompt the user to confirm or select a provider from detected API keys.
/// Called before the TUI starts, so stdin/stdout are available.
fn prompt_provider_selection(detected: Vec<DetectedProvider>) -> Result<Provider> {
    if detected.is_empty() {
        anyhow::bail!(
            "No LLM provider specified and no API keys found in environment.\n\n\
             Set one of the following environment variables:\n\
             \x20 ANTHROPIC_API_KEY  — Anthropic Claude\n\
             \x20 OPENAI_API_KEY     — OpenAI\n\
             \x20 XAI_API_KEY        — xAI / Grok\n\n\
             Or specify a provider explicitly:\n\
             \x20 hick-agent run --provider anthropic\n\
             \x20 hick-agent run --provider local --model-path ./model.gguf"
        );
    }

    if detected.len() == 1 {
        let d = &detected[0];
        print!(
            "Detected {} ({}). Use {} / {}? [Y/n] ",
            d.env_var, d.provider, d.provider, d.default_model,
        );
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        if input.is_empty() || input == "y" || input == "yes" {
            return Ok(d.provider);
        }
        anyhow::bail!("Cancelled. Use --provider to specify a provider explicitly.");
    }

    // Multiple providers detected — let the user choose
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
    local_opts: LocalOpts,
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
        Provider::Local => {
            #[cfg(feature = "local")]
            {
                let path = local_opts.model_path.ok_or_else(|| {
                    anyhow::anyhow!(
                        "--model-path is required for the local provider\n\
                         Example: --provider local --model-path ./model.gguf"
                    )
                })?;
                match local_opts.n_gpu_layers {
                    Some(layers) => Ok(Box::new(LocalLlmClient::load(
                        path,
                        layers,
                        local_opts.n_ctx,
                        local_opts.max_tokens,
                    )?)),
                    None => Ok(Box::new(LocalLlmClient::load_auto(
                        path,
                        local_opts.n_ctx,
                        local_opts.max_tokens,
                        &local_opts.project_dir,
                    )?)),
                }
            }
            #[cfg(not(feature = "local"))]
            {
                let _ = local_opts;
                anyhow::bail!(
                    "Local provider requires the 'local' feature.\n\
                     Build with: cargo build -p hick-agent --features local\n\
                     For NVIDIA GPU: --features local-cuda"
                )
            }
        }
    }
}

/// Determine the API endpoint host for a given provider and optional base URL.
fn endpoint_host_for_provider(provider: Provider, api_base: &Option<String>) -> String {
    if let Some(base) = api_base {
        extract_host_from_url(base).unwrap_or_else(|| "unknown".to_string())
    } else {
        match provider {
            Provider::Anthropic => "api.anthropic.com".to_string(),
            Provider::Openai => "api.openai.com".to_string(),
            Provider::Xai => "api.x.ai".to_string(),
            Provider::OpenaiCompat => "unknown".to_string(),
            Provider::Local => "localhost".to_string(),
        }
    }
}

/// Walk up from `start` looking for `native2wasm/crates/c2w/converted_images`.
/// Returns `(images_dir, default_image)` for the first image found, or `None`.
fn detect_images_dir(start: &std::path::Path) -> Option<(PathBuf, String)> {
    let rel = std::path::Path::new("native2wasm/crates/c2w/converted_images");
    let mut dir = start.to_path_buf();
    loop {
        let candidate = dir.join(rel);
        if candidate.is_dir() {
            // Find the first subdirectory (e.g. "alpine") that contains a .wasm file
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

/// Background task that waits for user messages and drives the react loop.
async fn agent_task(
    react_loop: ReactLoop,
    events: mpsc::UnboundedSender<AgentEvent>,
    mut commands: mpsc::UnboundedReceiver<AgentCommand>,
    interrupt: Arc<Notify>,
) {
    let mut history = Vec::new();
    while let Some(cmd) = commands.recv().await {
        match cmd {
            AgentCommand::Message(msg) => {
                if let Err(e) = react_loop.run(&msg, &mut history, &events, &interrupt).await {
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

    match cli.command {
        Commands::Run {
            project_dir,
            images_dir,
            default_image,
            provider,
            model,
            api_base,
            api_key,
            model_path,
            n_gpu_layers,
            n_ctx,
            max_tokens,
            no_stream,
        } => {
            let project_dir = project_dir.unwrap_or_else(|| std::env::current_dir().unwrap());
            let (images_dir, default_image) = match images_dir {
                Some(d) => (d, default_image),
                None => {
                    match detect_images_dir(&project_dir) {
                        Some((d, img)) => {
                            println!("Auto-detected images: {} (image: {})", d.display(), img);
                            (d, img)
                        }
                        None => (PathBuf::from("."), default_image),
                    }
                }
            };
            let config = AgentConfig {
                project_dir,
                images_dir,
                default_image,
            };

            // Resolve provider: explicit flag/env, or auto-detect from API keys
            let provider = match provider {
                Some(p) => p,
                None => {
                    let detected = detect_providers();
                    prompt_provider_selection(detected)?
                }
            };

            let local_opts = LocalOpts {
                model_path,
                n_gpu_layers,
                n_ctx,
                max_tokens,
                project_dir: config.project_dir.clone(),
            };
            let llm = build_llm_client(provider, model, api_base.clone(), api_key, local_opts)?;

            // Channels between core and TUI
            let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<AgentCommand>();
            let interrupt = Arc::new(Notify::new());

            // Load agent security configuration (used for both sandbox and network filter).
            let security_config_path = config.project_dir.join(".hick").join("agent-security.toml");
            let security_config = AgentSecurityConfig::load_or_default(&security_config_path)?;
            let sandbox_caps = security_config.to_capabilities();

            // Load auth injection configuration.
            let auth_rules_path = config.project_dir.join(".hick").join("auth-rules.toml");
            let age_key_path = std::env::var("HICK_AGE_KEY")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/Users/ai/Documents/src/key.txt"));
            let age_secrets_dir = config.project_dir.join(".hick").join("secrets");
            let auth_stack = match hick::auth::AuthStack::from_config(
                &auth_rules_path,
                &age_key_path,
                &age_secrets_dir,
            ) {
                Ok(Some(stack)) => {
                    let stack = Arc::new(stack);
                    event_tx
                        .send(AgentEvent::Info(format!(
                            "Auth injection: active ({} env vars)",
                            stack.container_env.len(),
                        )))
                        .ok();
                    Some(stack)
                }
                Ok(None) => None,
                Err(e) => {
                    event_tx
                        .send(AgentEvent::Info(format!("Auth injection: failed to load ({e})")))
                        .ok();
                    None
                }
            };

            // Network filter for LLM API calls.
            // Priority: explicit network-rules.toml > derived from agent-security.toml > none.
            let (approval_tx, approval_rx) = mpsc::unbounded_channel::<NetworkApprovalRequest>();
            let filter_config_path = config.project_dir.join(".hick").join("network-rules.toml");
            let host = endpoint_host_for_provider(provider, &api_base);
            let llm: Box<dyn LlmClient> = if matches!(provider, Provider::Local) {
                drop(approval_tx);
                llm
            } else if filter_config_path.exists() {
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
                // Derive filter from agent-security.toml capabilities.
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

            // Create token authority with a random per-session key and mint
            // a root capability token for this session.
            let session_key: [u8; 32] = rand::random();
            let token_authority = Arc::new(TokenAuthority::new(&session_key));
            let session_token = Arc::new(
                token_authority
                    .mint("agent-session", &sandbox_caps)
                    .map_err(|e| anyhow::anyhow!("Failed to mint session token: {e}"))?,
            );

            let agent =
                Agent::new(config.clone(), llm).with_security(token_authority, session_token);
            let output_dir = std::env::var("HICK_OUTPUT_DIR").ok();

            // Create the sink guard for classification-based API call gating.
            // Uses a clone of sandbox_caps since the original is moved into WasmSandboxConfig.
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

            // Guest networking is on by default so apk/pip/git work inside the VM.
            // The previous FD-3 collision (preopened_socket overwriting preopened_dir at FD 3)
            // that caused an empty /workspace with networking on has been fixed in container.rs.
            // HICK_AGENT_DISABLE_SANDBOX_GUEST_NET is kept for backward compatibility but is no
            // longer needed for workspace visibility.
            let sandbox_guest_network = !matches!(
                std::env::var("HICK_AGENT_DISABLE_SANDBOX_GUEST_NET").as_deref(),
                Ok("1") | Ok("true") | Ok("yes")
            );

            // Build the WASM sandbox executor for running scripts.
            let wasm_image_path = config
                .images_dir
                .join(format!("{}.wasm", config.default_image));
            hick::executor::ensure_loadable_wasm_file(&wasm_image_path).map_err(|e| {
                anyhow::anyhow!(
                    "{e:#}\n\n\
                     Set --images-dir to the directory containing your .wasm images\n\
                     and --default-image to the image name (without the .wasm extension).\n\n\
                     If you have a `*.wasm.zip` next to the image, it will be extracted automatically when the `.wasm` is missing or corrupt.\n\n\
                     Example:\n\
                     \x20 hick-agent run \\\n\
                     \x20   --images-dir /path/to/converted_images/alpine \\\n\
                     \x20   --default-image 3.23.2",
                )
            })?;
            let sandbox_config = WasmSandboxConfig {
                images_dir: config.images_dir.clone(),
                workdir: config.project_dir.clone(),
                default_image: config.default_image.clone(),
                capabilities: sandbox_caps,
                auth: auth_stack,
                guest_network_enabled: sandbox_guest_network,
            };
            let sandbox_executor: Arc<Mutex<dyn CommandExecutor>> =
                Arc::new(Mutex::new(WasmSandboxExecutor::new(sandbox_config)));
            event_tx
                .send(AgentEvent::Info(format!(
                    "Sandbox: WASM container enabled ({caps_summary})"
                )))
                .ok();

            let mut react_loop = ReactLoop::new(agent, config.project_dir.clone())
                .with_executor(sandbox_executor, config.default_image.clone())
                .with_sink_guard(sink_guard);

            // Wire up policy evaluator if configured.
            if let Some(ref policy_path) = security_config.policy_script {
                let resolved = config.project_dir.join(policy_path);
                // Validate the policy script path before wiring it up:
                // - Must exist and be a regular file
                // - Must end in .py
                // - Must resolve to within the project directory (prevent traversal)
                let canonical_project = config.project_dir.canonicalize().unwrap_or_default();
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
                    react_loop = react_loop.with_policy(hick_agent_engine::hick_policy::PolicyEvaluator::new(
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

            react_loop.stream_output = !no_stream;

            // Start the transparent pipeline watcher
            let watcher = Arc::new(PipelineWatcher::new(config.project_dir.clone(), output_dir));
            let event_tx_watcher = event_tx.clone();
            tokio::spawn({
                let watcher = Arc::clone(&watcher);
                async move {
                    if let Err(e) = watcher.watch_loop().await {
                        event_tx_watcher
                            .send(AgentEvent::Error(format!("Pipeline watcher: {e}")))
                            .ok();
                    }
                }
            });

            // Copy the Python SDK into .hick/sdk/
            setup_sdk(&config.project_dir);

            // Send startup info as events
            event_tx
                .send(AgentEvent::Info(format!(
                    "Project: {}",
                    config.project_dir.display()
                )))
                .ok();
            event_tx
                .send(AgentEvent::Info("Pipeline: watching .hick files".into()))
                .ok();

            let provider_info = format!("{provider_name} / {model_name}");

            // Determine log path: .hick/agent-YYYYMMDD-HHMMSS.log
            let log_path = {
                let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
                let log_dir = config.project_dir.join(".hick");
                std::fs::create_dir_all(&log_dir).ok();
                log_dir.join(format!("agent-{ts}.log"))
            };

            // Spawn the agent task
            let interrupt_agent = Arc::clone(&interrupt);
            tokio::spawn(agent_task(react_loop, event_tx, cmd_rx, interrupt_agent));

            // Run the TUI on a blocking thread (the agent runs on tokio workers)
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
        }
    }

    Ok(())
}

/// Copy the bundled Python SDK into the project's `.hick/sdk/` directory
/// so that Python scripts can `from hickory import hick`.
fn setup_sdk(project_dir: &std::path::Path) {
    let sdk_dest = project_dir.join(".hick").join("sdk").join("hickory");
    if sdk_dest.join("__init__.py").exists() {
        return; // Already set up
    }

    // The SDK source is bundled alongside the hick-agent binary
    let sdk_source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("sdk")
        .join("hickory");

    if !sdk_source.exists() {
        // Fallback: try relative to project dir (development mode)
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
