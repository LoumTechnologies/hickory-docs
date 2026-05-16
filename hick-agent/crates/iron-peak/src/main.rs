//! iron-peak: hick-native AI coding agent and pipeline runner.
//!
//! With no arguments, launches the AI agent TUI (ratatui interface).
//! All pipeline subcommands (run, up, repl, init, validate, generate-matrix)
//! are available as first-class subcommands.

use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use hick_agent_core::{CommandExecutor, WasmSandboxConfig, WasmSandboxExecutor};
use tokio::sync::{Mutex, Notify, mpsc};

#[cfg(feature = "local")]
use hick_agent_engine::LocalLlmClient;
use iron_peak::tui::run_tui;
use hick_agent_hick_edit::{PipelineWatcher, insert_paste_slots};
use hick_lang;
use hick_agent_script_first::{AgentCommand, AgentEvent, ReactLoop};
use hick_agent_sdk::{
    AgentSecurityConfig, AnthropicClient, FilteredLlmClient, LlmClient,
    NetworkApprovalRequest, NetworkFilter, OpenAiCompatClient, SinkGuard,
};
use hick_agent_sdk::network_filter::extract_host_from_url;

// ── System prompt ─────────────────────────────────────────────────────────────

const HICK_SYSTEM_PROMPT: &str = r#"You are an AI coding agent for projects using Hick (hickory docs) — an XML format for defining containers, file outputs, and code generation.

## Response format (REQUIRED)

Every response MUST begin with one of these two XML tags on the very first line:

  <hick:next>code</hick:next>
  <hick:next>done</hick:next>

Rules:
- `<hick:next>code</hick:next>` — you are going to run code. You MUST include exactly one fenced code block in this response.
- `<hick:next>done</hick:next>` — you are finished. Write your final answer as plain text; do NOT include any code blocks.

The system validates these tags. If you write `<hick:next>code</hick:next>` without a code block, or `<hick:next>done</hick:next>` with a code block, you will be asked to correct your response.

Write ONE fenced code block per response, then STOP and wait for the real execution output. Do NOT predict or guess what the output will be — the system runs your code and shows you the actual stdout/stderr.

## Executable fenced blocks (runner map)

Pick a fence whose runner exists in your environment (see the runtime probe in the system prompt when using the sandbox).

- Shell: ```bash, ```sh, ```shell, ```posix-sh — saved as POSIX `sh`; use for `apk add`, `ls`, pipelines, POSIX scripts, installers, compilers, etc.
- Python: ```python — `python3` + Hickory SDK
- JavaScript (Node): ```javascript / ```js — `node`
- C#: ```csharp / ```cs — `dotnet-script`
- TypeScript: ```typescript / ```ts — `tsx`

You may install language runtimes or packages yourself inside the sandbox (e.g. `apk add …`, `npm install -g …`, pip, curling binaries) whenever you need them. No separate permission step—go ahead and fix the environment if something is missing (subject to Hick resource limits you may hit as normal errors).

## Available languages / patterns

### Shell (recommended for discovery and installs)
- Prefer ```bash when you want simple commands: `find`, `ls`, `apk add python3 py3-pip`, `wget`, etc.
- Scripts run as `sh /workspace/.hick/tmp/…` — the interpreter is POSIX `sh` for these fences (shebang lines are ignored). Call another interpreter explicitly (`python3 …`, `bash …`) once installed.

### Python
- Read/write files with `open()`, `os`, `pathlib`
- Run shell commands with `subprocess.run()`
- Install packages: `subprocess.run(['pip', 'install', ...])` (after installing pip if needed)
- Hickory SDK: `from hickory import hick` (Python only — SDK under `/workspace/.hick/sdk`)

### JavaScript (Node)
- Runs with `node`. Use `.mjs` style or plain CommonJS depending on what's available after install.

### C# (dotnet-script)
- Write C# script files (.csx) — no Main() or class wrapper needed
- Install NuGet packages: `#r "nuget: Newtonsoft.Json, 13.0.3"`
- Requires `dotnet` / `dotnet-script` when you choose this fence.

### TypeScript (tsx)
- Write TypeScript; requires `tsx` (install globally or via npm/npx depending on probe output).

## Hickory SDK (Python only)
  from hickory import hick
  hick.query(selector, file=path)      — find elements (fn, struct, class, #id, .class)
  hick.structure(path)                  — structural summary of a generated file
  hick.provenance(path, start, end)    — which .hick source generated these lines
  hick.edit(selector, new_code, file=path) — compute source edits
  hick.apply(edits)                     — apply edits to .hick source files

## Hick XML format

.hick files use this format:
  <hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
    <hick:file path="output/path.py">
  file content here
    </hick:file>
  </hick:doc>

Key elements:
- Root element: `<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">`
- File output: `<hick:file path="relative/path">content</hick:file>`
- Variables: `<hick:var name="x">value</hick:var>` and `<hick:paste select="$x"/>`
- Shell exec: `<hick:exec container="default">command</hick:exec>`

## Key concepts

1. **Output files are generated** from `.hick` XML source files. The pipeline runs transparently when `.hick` files change — you never need to run it manually.

2. **Edit .hick source, not output files.** Use the hickory SDK to map output locations back to .hick source, or edit .hick files directly.

3. **`<hick:exec>` vs one-off commands:**
   - Use one-off scripts for: running tests, installing deps, checking status
   - Use `<hick:exec container="...">` in .hick files for recurring code generation
   - Rule: if the command's output becomes part of generated code, put it in a `<hick:exec>` block

4. **Print your results.** Your script's stdout is what you see as observation. Use print/Console.WriteLine/console.log in typed languages, or echo/printf for shell fences, to report findings.
"#;

#[derive(Parser)]
#[command(name = "iron-peak")]
#[command(about = "Hick-native AI coding agent — script-first ReAct loop with hick pipeline integration")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
    /// Launch the AI agent TUI (default when no subcommand given).
    Agent {
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

    /// Run the pipeline once (single-shot mode).
    Run(PipelineRunArgs),

    /// Watch mode: re-run pipeline on file changes with merge-aware snapshots.
    Up(PipelineUpArgs),

    /// Interactive REPL: explore containers and record a session transcript.
    Repl(PipelineReplArgs),

    /// Initialize AI agent instructions for the project.
    Init(PipelineInitArgs),

    /// Validate AI agent configuration.
    Validate(PipelineValidateArgs),

    /// Generate all combinations of features to separate folders.
    GenerateMatrix(hick::generate_matrix::GenerateMatrixArgs),

    /// Insert paste slots into all hick:file elements in a .hick file.
    PasteSlots(PasteSlotsArgs),
}

#[derive(clap::Args)]
struct PipelineRunArgs {
    /// `.hick` files to process.
    files: Vec<PathBuf>,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    key_file: Option<PathBuf>,
    #[arg(long)]
    secrets_dir: Option<PathBuf>,
    #[arg(long, env = "HICK_IMAGES_DIR")]
    images_dir: Option<PathBuf>,
    #[arg(long = "param", value_parser = hick::parse_param)]
    params: Vec<(String, String)>,
    #[arg(long)]
    features: Option<String>,
    #[arg(long)]
    output_dir: Option<PathBuf>,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    cache: bool,
    #[arg(long)]
    freeze: bool,
    #[arg(long)]
    clear_cache: bool,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::Args)]
struct PipelineUpArgs {
    #[arg(required = true)]
    files: Vec<PathBuf>,
    #[arg(long, default_value = "~/.config/hick/key.txt")]
    key_file: PathBuf,
    #[arg(long, default_value = "~/.config/hick/secrets")]
    secrets_dir: PathBuf,
    #[arg(long, env = "HICK_IMAGES_DIR")]
    images_dir: Option<PathBuf>,
    #[arg(long = "param", value_parser = hick::parse_param)]
    params: Vec<(String, String)>,
    #[arg(long)]
    features: Option<String>,
    #[arg(long)]
    dry_run: bool,
    #[arg(short, long)]
    verbose: bool,
    #[arg(long, default_value = "auto")]
    store: String,
    #[arg(long)]
    merge_api: Option<String>,
    #[arg(long, default_value = "main")]
    branch: String,
    #[arg(long, default_value = "3")]
    max_stages: usize,
}

#[derive(clap::Args)]
struct PipelineReplArgs {
    #[arg(long, env = "HICK_IMAGES_DIR")]
    images_dir: Option<PathBuf>,
    #[arg(long, default_value = "session.hick")]
    output: PathBuf,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::Args)]
struct PipelineInitArgs {
    #[arg(long)]
    project_dir: Option<PathBuf>,
    #[arg(long)]
    no_agents: bool,
}

#[derive(clap::Args)]
struct PipelineValidateArgs {
    #[arg(long)]
    project_dir: Option<PathBuf>,
    #[arg(long)]
    fix: bool,
}

#[derive(clap::Args)]
struct PasteSlotsArgs {
    /// Path to the `.hick` file whose `hick:file` elements should receive paste slots.
    file: PathBuf,
    /// Write modified content back to disk (default: print diff summary to stdout).
    #[arg(long)]
    write: bool,
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

/// Select a provider from detected API keys, prompting only when multiple are found.
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
        return Ok(detected[0].provider);
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

    // Unwrap or default to Agent subcommand
    let command = cli.command.unwrap_or(Commands::Agent {
        project_dir: None,
        images_dir: None,
        default_image: "alpine".to_string(),
        provider: None,
        model: None,
        api_base: None,
        api_key: None,
        model_path: None,
        n_gpu_layers: None,
        n_ctx: 4096,
        max_tokens: 4096,
        no_stream: false,
    });

    match command {
        Commands::Agent {
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
                project_dir: project_dir.clone(),
            };
            let llm = build_llm_client(provider, model, api_base.clone(), api_key, local_opts)?;

            // Channels between core and TUI
            let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<AgentCommand>();
            let interrupt = Arc::new(Notify::new());

            // Load agent security configuration (used for both sandbox and network filter).
            let security_config_path = project_dir.join(".hick").join("agent-security.toml");
            let security_config = AgentSecurityConfig::load_or_default(&security_config_path)?;
            let sandbox_caps = security_config.to_capabilities();

            // Load auth injection configuration.
            let auth_rules_path = project_dir.join(".hick").join("auth-rules.toml");
            let age_key_path = std::env::var("HICK_AGE_KEY")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/Users/ai/Documents/src/key.txt"));
            let age_secrets_dir = project_dir.join(".hick").join("secrets");
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
            let filter_config_path = project_dir.join(".hick").join("network-rules.toml");
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
            // container.rs registers dirs first (FD 3+) then places the socket after them,
            // using --net=socket=listenfd=N so both networking and /workspace visibility work.
            // HICK_AGENT_DISABLE_SANDBOX_GUEST_NET is kept for backward compatibility.
            let sandbox_guest_network = !matches!(
                std::env::var("HICK_AGENT_DISABLE_SANDBOX_GUEST_NET").as_deref(),
                Ok("1") | Ok("true") | Ok("yes")
            );

            // Build the WASM sandbox executor for running scripts.
            let wasm_image_path = images_dir.join(format!("{}.wasm", default_image));
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
                images_dir: images_dir.clone(),
                workdir: project_dir.clone(),
                default_image: default_image.clone(),
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

            let mut react_loop = ReactLoop::new(llm, project_dir.clone())
                .with_executor(sandbox_executor, default_image.clone())
                .with_sink_guard(sink_guard)
                .with_agent_dir(".hick")
                .with_sdk()
                .with_system_prompt(HICK_SYSTEM_PROMPT);

            // Wire up policy evaluator if configured.
            if let Some(ref policy_path) = security_config.policy_script {
                let resolved = project_dir.join(policy_path);
                // Validate the policy script path before wiring it up:
                // - Must exist and be a regular file
                // - Must end in .py
                // - Must resolve to within the project directory (prevent traversal)
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

            react_loop.stream_output = !no_stream;

            // Start the transparent pipeline watcher
            let watcher = Arc::new(PipelineWatcher::new(project_dir.clone(), output_dir));
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
            setup_sdk(&project_dir);

            // Send startup info as events
            event_tx
                .send(AgentEvent::Info(format!(
                    "Project: {}",
                    project_dir.display()
                )))
                .ok();
            event_tx
                .send(AgentEvent::Info("Pipeline: watching .hick files".into()))
                .ok();

            let provider_info = format!("{provider_name} / {model_name}");

            // Session file: .hick/agent-YYYYMMDD-HHMMSS.hick
            let log_path = {
                let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
                let log_dir = project_dir.join(".hick");
                std::fs::create_dir_all(&log_dir).ok();
                log_dir.join(format!("agent-{ts}.hick"))
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
            Ok(())
        }

        Commands::Run(args) => {
            hick::run_pipeline_cmd(hick::PipelineRunOpts {
                files: args.files,
                config_path: args.config,
                key_file: args.key_file,
                secrets_dir: args.secrets_dir,
                images_dir: args.images_dir,
                params: args.params,
                features: args.features,
                output_dir: args.output_dir,
                dry_run: args.dry_run,
                cache: args.cache,
                freeze: args.freeze,
                clear_cache: args.clear_cache,
                verbose: args.verbose,
            })
            .await
        }

        Commands::Up(args) => {
            use hick::store_config::{StoreBackend, StoreConfig};
            let key_path = hick::expand_tilde(&args.key_file);
            let secrets_dir_path = hick::expand_tilde(&args.secrets_dir);
            let _secrets_provider =
                hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir_path);

            let mut config = StoreConfig::load(&std::env::current_dir()?);
            config.store = match args.store.as_str() {
                "git" => StoreBackend::Git,
                "builtin" => StoreBackend::Builtin,
                _ => StoreBackend::Auto,
            };
            if let Some(url) = args.merge_api {
                config.merge_api = Some(url);
            }
            config.branch = args.branch;
            config.max_stages = args.max_stages;

            let images_dir = args
                .images_dir
                .map(|p| hick::expand_tilde(&p))
                .unwrap_or_else(|| std::path::PathBuf::from("."));

            let mut params: Vec<(String, String)> = args.params.clone();
            if let Some(ref features) = args.features {
                params.retain(|(pk, _)| pk != "features");
                params.push(("features".to_string(), features.clone()));
            }

            hick::watch::run_watch(&args.files, &config, &params, args.dry_run, images_dir).await
        }

        Commands::Repl(args) => {
            let images_dir = args
                .images_dir
                .map(|p| hick::expand_tilde(&p))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            hick::repl::run_repl(images_dir, args.output).await
        }

        Commands::Init(args) => {
            let project_dir = args
                .project_dir
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let config = hick::agents::InitConfig {
                project_dir: project_dir.clone(),
                no_agents: args.no_agents,
            };
            let result = hick::agents::init_agents(&config)?;
            if result.agents_file_created {
                println!("Created .agents/hick.md");
            }
            if let Some(ref path) = result.reference_added_to {
                println!("Added reference to {}", path.display());
            }
            if !result.agents_file_created && result.reference_added_to.is_none() {
                println!("Nothing to do (--no-agents specified or already initialized)");
            }
            Ok(())
        }

        Commands::Validate(args) => {
            let project_dir = args
                .project_dir
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let config = hick::agents::ValidateConfig {
                project_dir,
                fix: args.fix,
            };
            let issues = hick::agents::validate_agents(&config)?;
            if issues.is_empty() {
                println!("Agent configuration is valid");
                Ok(())
            } else {
                for issue in &issues {
                    eprintln!("Error: {issue}");
                }
                if !args.fix {
                    eprintln!("\nRun with --fix to automatically resolve these issues");
                }
                anyhow::bail!("Validation failed with {} issue(s)", issues.len())
            }
        }

        Commands::GenerateMatrix(args) => hick::generate_matrix::run(args).await,

        Commands::PasteSlots(args) => paste_slots_cmd(args),
    }
}

fn paste_slots_cmd(args: PasteSlotsArgs) -> Result<()> {
    let source = std::fs::read_to_string(&args.file).map_err(|e| {
        anyhow::anyhow!("Cannot read '{}': {e}", args.file.display())
    })?;

    let doc = hick_lang::parse(&source).map_err(|e| {
        anyhow::anyhow!("Parse error in '{}': {e}", args.file.display())
    })?;

    let prefix = &doc.prefix;
    let close_marker = format!("</{prefix}:file>");
    let open_marker_prefix = format!("<{prefix}:file");

    // Collect (content_start, content_end, file_path) for each hick:file tag.
    // content_start = byte after the opening tag's closing `>`
    // content_end   = byte of the `<` that starts the closing tag
    let mut regions: Vec<(usize, usize, String)> = Vec::new();
    for file_tag in doc.find_tags("file") {
        let Some(file_path) = file_tag.get_attribute("path") else {
            continue;
        };
        let Some(span) = file_tag.source_span else {
            continue;
        };
        // Content starts right after the opening tag ends.
        let content_start = span.end;
        // Find the matching closing tag in the source from content_start onward.
        let Some(close_rel) = source[content_start..].find(&close_marker) else {
            continue;
        };
        let content_end = content_start + close_rel;
        regions.push((content_start, content_end, file_path.to_string()));
    }

    if regions.is_empty() {
        println!("No hick:file elements found in '{}'.", args.file.display());
        return Ok(());
    }

    // Apply insertions in reverse order to preserve byte offsets.
    let mut result = source.clone();
    let mut modified_count = 0usize;

    for (content_start, content_end, file_path) in regions.iter().rev() {
        let content = &source[*content_start..*content_end];
        let new_content = insert_paste_slots(file_path, content);
        if new_content != content {
            result.replace_range(*content_start..*content_end, &new_content);
            modified_count += 1;
        }
    }

    if modified_count == 0 {
        println!(
            "All hick:file elements in '{}' already have paste slots.",
            args.file.display()
        );
        return Ok(());
    }

    if args.write {
        std::fs::write(&args.file, &result).map_err(|e| {
            anyhow::anyhow!("Cannot write '{}': {e}", args.file.display())
        })?;
        println!(
            "Updated {} hick:file element(s) in '{}'.",
            modified_count,
            args.file.display()
        );
    } else {
        println!(
            "Would update {} hick:file element(s) in '{}'. Use --write to apply.",
            modified_count,
            args.file.display()
        );
    }

    // Suppress unused-variable warning on open_marker_prefix (used for documentation clarity).
    let _ = open_marker_prefix;

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
