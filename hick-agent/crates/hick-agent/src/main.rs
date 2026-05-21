//! hick: universal AI coding agent and pipeline runner.
//!
//! With no arguments, launches the AI agent TUI (ratatui interface) with
//! extensions determined by `.hick/agent.toml` (or `--config <path>`).
//! All pipeline subcommands (run, up, repl, init, validate, generate-matrix)
//! are available as first-class subcommands.

mod config;
mod judge_task;

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

#[cfg(feature = "local")]
use hick_agent_engine::LocalLlmClient;
use hick_agent_script_first::{
    AgentCommand, AgentEvent, ScriptBlockIntentParser, ScriptBlockPayload,
    correction_message, run_ui_event_fanout_task, ReplayRequest,
    CodeBlock, Language, ScriptResult,
    SANDBOX_EXECUTION_CONTEXT, SANDBOX_RUNTIME_PROBE,
};
use hick_agent_tui::{run_tui, TabSource, TuiConfig};
use hick_agent_hick_edit::{PipelineWatcher, insert_paste_slots};
use hick_agent_sdk::{
    AgentContext, AgentHistory, AgentIo, AgentSecurityConfig, AnthropicClient,
    FilteredLlmClient, IntentRouter, LlmClient, Message, NetworkApprovalRequest,
    NetworkFilter, OpenAiCompatClient, Role, SinkGuard,
    ToolData, ToolExecutor, XmlDiagnosticLogger, XmlSessionLog, NullSessionLog,
    run_agent_engine,
};
use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
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
  hick.query(selector, file=path)           — find elements (fn, struct, class, #id, .class)
  hick.structure(path)                       — structural summary of a generated file
  hick.provenance(path, start, end)         — which .hick source generated these lines
  hick.edit(selector, new_code, file=path)  — compute source edits
  hick.apply(edits)                          — apply edits to .hick source files

  # Pipeline-first writes (PREFERRED over direct file writes)
  hick.is_pipeline_owned(file)              — True if file is declared in a hick:file element
  hick.list_paste_slots(file)               — list named paste slots for a pipeline file
  hick.pipeline_write(file, content, slot)  — modify pipeline file via hick:copy (merge-safe)
  hick.pipeline_new_file(file, content)     — create new file via hick:file element

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
- Paste slot: `<hick:paste name="after-fn-foo"/>` — composition point for hick:copy
- Copy edit: `<hick:copy target="after-fn-foo">new code</hick:copy>` — inserts at slot

## Key concepts

1. **Output files are generated** from `.hick` XML source files. The pipeline runs transparently when `.hick` files change — you never need to run it manually.

2. **Pipeline-first edits (REQUIRED for pipeline-owned files):**
   - ALWAYS check `hick.is_pipeline_owned(file)` before writing a file.
   - If owned: use `hick.pipeline_write(file, content)` to append a `hick:copy` element.
     List available slots first with `hick.list_paste_slots(file)` and pass `slot=` to target one.
   - If creating a new file: use `hick.pipeline_new_file(file, content)` instead of `open()`.
   - If NOT owned (external/binary/config files): write directly with `open()` or shell commands.
   - Fallback: `pipeline_write` prints a warning and does a direct write when the file is not owned.

3. **Edit .hick source, not output files.** Use the hickory SDK to map output locations back to .hick source, or edit .hick files directly.

4. **`<hick:exec>` vs one-off commands:**
   - Use one-off scripts for: running tests, installing deps, checking status
   - Use `<hick:exec container="...">` in .hick files for recurring code generation
   - Rule: if the command's output becomes part of generated code, put it in a `<hick:exec>` block

5. **Print your results.** Your script's stdout is what you see as observation. Use print/Console.WriteLine/console.log in typed languages, or echo/printf for shell fences, to report findings.
"#;

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "hick")]
#[command(about = "Universal hick AI coding agent and pipeline runner")]
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
        /// Path to the agent config file.
        /// Resolution order when omitted: .hick/agent.toml → ~/.hick/agent.toml → built-in default.
        #[arg(long)]
        config: Option<PathBuf>,
        /// Additional config files for multi-config/eval mode.  Each path is a separate
        /// agent.toml — all configs run as independent Tokio tasks and appear as tabs.
        #[arg(long = "also-config", value_name = "PATH")]
        extra_configs: Vec<PathBuf>,
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
        /// Disable local Web UI (`HICK_NO_WEB=1` also disables).
        #[arg(long)]
        no_web: bool,
    },

    /// Open a completed session bundle in read-only replay mode.
    Replay {
        /// Path to the session bundle directory (contains session.xml).
        bundle_dir: PathBuf,
        /// Replay speed multiplier: 1.0 = real-time, 0.0 = instant.
        #[arg(long, default_value = "0.0")]
        replay_speed: f64,
        /// Project directory (default: current dir).
        #[arg(short, long)]
        project_dir: Option<PathBuf>,
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
    GenerateMatrix(hick_literate::generate_matrix::GenerateMatrixArgs),

    /// Insert paste slots into all hick:file elements in a .hick file.
    PasteSlots(PasteSlotsArgs),

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
    images_dir: Option<PathBuf>,
    /// Default container image name (used with --images-dir).
    #[arg(long, default_value = "alpine")]
    default_image: String,
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

#[derive(clap::Args)]
struct PipelineRunArgs {
    files: Vec<PathBuf>,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    key_file: Option<PathBuf>,
    #[arg(long)]
    secrets_dir: Option<PathBuf>,
    #[arg(long, env = "HICK_IMAGES_DIR")]
    images_dir: Option<PathBuf>,
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
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
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
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

#[allow(dead_code)]
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

fn detect_providers() -> Vec<DetectedProvider> {
    let mut found = vec![];
    let candidates: &[(&str, Provider, &str)] = &[
        ("ANTHROPIC_API_KEY", Provider::Anthropic, "claude-sonnet-4-5-20250929"),
        ("OPENAI_API_KEY", Provider::Openai, "gpt-4o"),
        ("XAI_API_KEY", Provider::Xai, "grok-4.3"),
    ];
    for &(env_var, provider, default_model) in candidates {
        if std::env::var(env_var).ok().filter(|k| !k.is_empty()).is_some() {
            found.push(DetectedProvider { provider, env_var, default_model });
        }
    }
    found
}

fn prompt_provider_selection(detected: Vec<DetectedProvider>) -> Result<Provider> {
    if detected.is_empty() {
        anyhow::bail!(
            "No LLM provider specified and no API keys found in environment.\n\n\
             Set one of the following environment variables:\n\
             \x20 ANTHROPIC_API_KEY  — Anthropic Claude\n\
             \x20 OPENAI_API_KEY     — OpenAI\n\
             \x20 XAI_API_KEY        — xAI / Grok\n\n\
             Or specify a provider explicitly:\n\
             \x20 hick agent --provider anthropic\n\
             \x20 hick agent --provider local --model-path ./model.gguf"
        );
    }

    if detected.len() == 1 {
        return Ok(detected[0].provider);
    }

    println!("Multiple API keys detected:");
    for (i, d) in detected.iter().enumerate() {
        println!("  [{}] {} — {} ({})", i + 1, d.provider, d.env_var, d.default_model);
    }
    print!("Select provider [1-{}]: ", detected.len());
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        anyhow::bail!("No selection made. Use --provider to specify a provider explicitly.");
    }

    let idx: usize = input.parse().map_err(|_| anyhow::anyhow!("Invalid selection: {input}"))?;
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
                let key = api_key.unwrap_or_else(|| std::env::var("OPENAI_API_KEY").unwrap_or_default());
                Ok(Box::new(OpenAiCompatClient::custom(base, key, model, "openai")))
            } else {
                Ok(Box::new(OpenAiCompatClient::openai(model)))
            }
        }
        Provider::Anthropic => {
            let model = model.unwrap_or_else(|| "claude-sonnet-4-5-20250929".into());
            let key = api_key.unwrap_or_else(|| std::env::var("ANTHROPIC_API_KEY").unwrap_or_default());
            Ok(Box::new(AnthropicClient::with_config(model, key)))
        }
        Provider::Xai => {
            let model = model.unwrap_or_else(|| "grok-4.3".into());
            if let Some(key) = api_key {
                Ok(Box::new(OpenAiCompatClient::custom("https://api.x.ai/v1", key, model, "xai")))
            } else {
                Ok(Box::new(OpenAiCompatClient::xai(model)))
            }
        }
        Provider::OpenaiCompat => {
            let base = api_base.ok_or_else(|| anyhow::anyhow!("--api-base is required for openai-compat provider"))?;
            let key = api_key.unwrap_or_default();
            let model = model.unwrap_or_else(|| "default".into());
            Ok(Box::new(OpenAiCompatClient::custom(base, key, model, "openai-compat")))
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
                    Some(layers) => Ok(Box::new(LocalLlmClient::load(path, layers, local_opts.n_ctx, local_opts.max_tokens)?)),
                    None => Ok(Box::new(LocalLlmClient::load_auto(path, local_opts.n_ctx, local_opts.max_tokens, &local_opts.project_dir)?)),
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

// ── Config-driven extension loading ──────────────────────────────────────────

/// Load `[extensions]` from config resolution order.
/// Returns default (iron-peak profile: script_first + hick_edit enabled) if no config found.
fn load_extensions(
    explicit_config: Option<&std::path::Path>,
    project_dir: &std::path::Path,
) -> config::ExtensionsSection {
    let path = explicit_config.map(|p| p.to_path_buf()).or_else(|| {
        let proj = project_dir.join(".hick/agent.toml");
        if proj.exists() {
            return Some(proj);
        }
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".hick/agent.toml"))
            .filter(|p| p.exists())
    });

    if let Some(path) = path {
        if let Ok(config::AgentConfig::Point(pc)) = config::AgentConfig::load(Some(&path), project_dir) {
            return pc.extensions;
        }
    }

    // Built-in default: full hick-edit profile
    config::ExtensionsSection {
        script_first: true,
        hick_edit: true,
        ..Default::default()
    }
}

/// Load `[eval.judge]` from config resolution order.
/// Returns `None` if no config is found or the judge section is absent.
fn load_eval_judge(
    explicit_config: Option<&std::path::Path>,
    project_dir: &std::path::Path,
) -> Option<config::JudgeSection> {
    let path = explicit_config.map(|p| p.to_path_buf()).or_else(|| {
        let proj = project_dir.join(".hick/agent.toml");
        if proj.exists() {
            return Some(proj);
        }
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".hick/agent.toml"))
            .filter(|p| p.exists())
    });

    if let Some(path) = path {
        if let Ok(config::AgentConfig::Point(pc)) =
            config::AgentConfig::load(Some(&path), project_dir)
        {
            return pc.eval.judge;
        }
    }
    None
}

// ── HickAgentIo ───────────────────────────────────────────────────────────────

/// Possible side-effects signalled from [`HickAgentIo::await_next_input_packet`].
enum PendingCommand {
    None,
    Quit,
    Rewind { system_messages: Vec<Message>, user_assistant_history: Vec<Message> },
}

struct HickAgentIo {
    /// Receives all `AgentCommand`s from the TUI / UI.
    cmd_rx: tokio::sync::Mutex<mpsc::UnboundedReceiver<AgentCommand>>,
    /// Sends `AgentEvent`s to the UI fanout task.
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    /// Cleared to signal the engine loop to exit.
    running: Arc<AtomicBool>,
    /// Side-channel for non-message commands detected during `await_next_input_packet`.
    pending: tokio::sync::Mutex<PendingCommand>,
    /// Snapshot of system messages for rewind reconstruction.
    system_messages: tokio::sync::Mutex<Vec<Message>>,
    /// WASM executor reference (for DirectCommand handling).
    executor: Arc<tokio::sync::Mutex<dyn CommandExecutor>>,
    /// Container image name for the WASM sandbox.
    sandbox_image: String,
    /// Agent scratch directory (`.hick`).
    agent_dir: String,
    /// Project directory.
    project_dir: std::path::PathBuf,
}

#[async_trait::async_trait]
impl AgentIo for HickAgentIo {
    async fn await_next_input_packet(&self) -> String {
        loop {
            let cmd = {
                let mut rx = self.cmd_rx.lock().await;
                rx.recv().await
            };
            match cmd {
                Some(AgentCommand::Message(msg)) => {
                    self.event_tx.send(AgentEvent::UserMessage(msg.clone())).ok();
                    return msg;
                }
                Some(AgentCommand::DirectCommand(cmd)) => {
                    // Execute the command in the sandbox and return an observation.
                    self.event_tx
                        .send(AgentEvent::ScriptStarted {
                            code: cmd.clone(),
                            lang: "sh".to_string(),
                        })
                        .ok();
                    let result = self.execute_shell(&cmd).await;
                    self.event_tx.send(AgentEvent::ScriptFinished(result.clone())).ok();
                    let obs = format!(
                        "User ran directly (! command): {cmd}\nOutput:\n{}",
                        result.as_observation()
                    );
                    self.event_tx.send(AgentEvent::Done(String::new())).ok();
                    return obs;
                }
                Some(AgentCommand::RewindTo { fork_node_id, user_assistant_history }) => {
                    let system_msgs = self.system_messages.lock().await.clone();
                    *self.pending.lock().await = PendingCommand::Rewind {
                        system_messages: system_msgs,
                        user_assistant_history,
                    };
                    self.event_tx
                        .send(AgentEvent::Info(format!(
                            "Context rewound to branch point #{fork_node_id}."
                        )))
                        .ok();
                    self.running.store(false, Ordering::SeqCst);
                    return String::new();
                }
                Some(AgentCommand::Quit) | None => {
                    *self.pending.lock().await = PendingCommand::Quit;
                    self.running.store(false, Ordering::SeqCst);
                    return String::new();
                }
            }
        }
    }

    fn stream_chunk(&self, message: &Message) {
        if !message.content.is_empty() {
            self.event_tx.send(AgentEvent::Token(message.content.clone())).ok();
        }
    }

    fn print_status_update(&self, text: &str) {
        self.event_tx.send(AgentEvent::Done(text.to_string())).ok();
    }
}

impl HickAgentIo {
    async fn execute_shell(&self, cmd: &str) -> ScriptResult {
        let sandbox_cmd = format!("cd /workspace && {cmd}");
        let mut exec = self.executor.lock().await;
        match exec
            .execute("agent-sandbox", &sandbox_cmd, &self.sandbox_image, Some(60))
            .await
        {
            Ok(result) => ScriptResult {
                stdout: result.output,
                stderr: String::new(),
                exit_code: result.exit_code,
            },
            Err(e) => ScriptResult {
                stdout: String::new(),
                stderr: format!("Command failed: {e}"),
                exit_code: None,
            },
        }
    }
}

// ── HickToolExecutor ──────────────────────────────────────────────────────────

struct HickToolExecutor {
    executor: Arc<tokio::sync::Mutex<dyn CommandExecutor>>,
    sandbox_image: String,
    agent_dir: String,
    project_dir: std::path::PathBuf,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    session_log: Arc<dyn hick_agent_sdk::SessionLog>,
    policy_evaluator: Option<hick_policy::PolicyEvaluator>,
    inject_sdk_pythonpath: bool,
}

#[async_trait::async_trait]
impl ToolExecutor for HickToolExecutor {
    async fn run(&self, _tool: ToolData) -> anyhow::Result<Message> {
        // The script-first strategy uses run_custom; standard tool calls are not used.
        Ok(Message {
            role: Role::User,
            content: "<tool-error>No MCP tools configured</tool-error>".to_string(),
        })
    }

    async fn run_custom(
        &self,
        name: &str,
        payload: Box<dyn Any + Send>,
    ) -> anyhow::Result<Message> {
        if name != "script_first" {
            return Ok(Message {
                role: Role::User,
                content: format!("<tool-error>Unknown custom intent: {name}</tool-error>"),
            });
        }

        let script_payload = match payload.downcast::<ScriptBlockPayload>() {
            Ok(p) => p,
            Err(_) => {
                return Ok(Message {
                    role: Role::User,
                    content: "<tool-error>Invalid script_first payload type</tool-error>".to_string(),
                });
            }
        };

        match *script_payload {
            ScriptBlockPayload::Invalid { reason } => {
                // Inject re-prompt correction — LLM will self-correct on next turn.
                Ok(Message {
                    role: Role::User,
                    content: correction_message(&reason),
                })
            }
            ScriptBlockPayload::Code { blocks, .. } => {
                let results = self.execute_blocks(&blocks).await;
                let observation = results
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        if results.len() == 1 {
                            format!("Script output:\n{}", r.as_observation())
                        } else {
                            format!("Script {} output:\n{}", i + 1, r.as_observation())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n");

                Ok(Message { role: Role::User, content: observation })
            }
        }
    }
}

impl HickToolExecutor {
    async fn execute_blocks(&self, blocks: &[CodeBlock]) -> Vec<ScriptResult> {
        let mut results = Vec::with_capacity(blocks.len());

        for (i, block) in blocks.iter().enumerate() {
            // Policy check.
            if let Some(ref evaluator) = self.policy_evaluator {
                let input = hick_policy::FlowPolicyInput {
                    column_classifications: serde_json::json!({}),
                    sink_type: "script-execution".to_string(),
                    slot_name: format!("script_{i}"),
                    data_classification: serde_json::json!(["agent-generated"]),
                    token_caveats: vec![],
                };
                match evaluator.evaluate_flow(&input).await {
                    Ok(hick_policy::PolicyDecision::Allow) => {}
                    Ok(hick_policy::PolicyDecision::Deny { reason }) => {
                        let desc = format!("Policy denied script execution: {reason}");
                        self.event_tx.send(AgentEvent::SecurityViolation { description: desc.clone() }).ok();
                        self.session_log.record(hick_agent_sdk::session_log::LogEvent::SecurityViolation { description: &desc });
                        results.push(ScriptResult {
                            stdout: String::new(),
                            stderr: format!("Policy denied execution: {reason}"),
                            exit_code: Some(1),
                        });
                        continue;
                    }
                    Ok(hick_policy::PolicyDecision::Prompt { reason }) => {
                        let desc = format!("Policy requires approval for script execution: {reason}");
                        self.event_tx.send(AgentEvent::SecurityViolation { description: desc.clone() }).ok();
                        self.session_log.record(hick_agent_sdk::session_log::LogEvent::SecurityViolation { description: &desc });
                        results.push(ScriptResult {
                            stdout: String::new(),
                            stderr: format!("Policy requires approval: {reason}"),
                            exit_code: Some(1),
                        });
                        continue;
                    }
                    Err(e) => {
                        let desc = format!("Policy evaluation error: {e}");
                        self.event_tx.send(AgentEvent::SecurityViolation { description: desc.clone() }).ok();
                        self.session_log.record(hick_agent_sdk::session_log::LogEvent::SecurityViolation { description: &desc });
                        results.push(ScriptResult {
                            stdout: String::new(),
                            stderr: format!("Policy evaluation failed: {e}"),
                            exit_code: Some(1),
                        });
                        continue;
                    }
                }
            }

            let lang = block.language.extension().to_string();
            self.session_log.record(hick_agent_sdk::session_log::LogEvent::ActionStart { lang: &lang, code: &block.code });
            self.event_tx
                .send(AgentEvent::ScriptStarted {
                    code: block.code.clone(),
                    lang: lang.clone(),
                })
                .ok();
            let result = self.execute_block(block, i).await;
            self.session_log.record(hick_agent_sdk::session_log::LogEvent::ActionEnd {
                stdout: &result.stdout,
                stderr: &result.stderr,
                exit_code: result.exit_code,
            });
            self.event_tx.send(AgentEvent::ScriptFinished(result.clone())).ok();
            results.push(result);
        }

        results
    }

    async fn execute_block(&self, block: &CodeBlock, index: usize) -> ScriptResult {
        let tmp_dir = self.project_dir.join(&self.agent_dir).join("tmp");
        if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
            return ScriptResult {
                stdout: String::new(),
                stderr: format!("Failed to create tmp dir: {e}"),
                exit_code: None,
            };
        }

        let filename = format!("script_{index}.{}", block.language.extension());
        let script_path = tmp_dir.join(&filename);
        if let Err(e) = std::fs::write(&script_path, &block.code) {
            return ScriptResult {
                stdout: String::new(),
                stderr: format!("Failed to write script: {e}"),
                exit_code: None,
            };
        }

        let container_script_path =
            format!("/workspace/{}/tmp/{filename}", self.agent_dir);

        let run_cmd = block.language.run_command(&container_script_path);
        let cmd = match block.language {
            Language::Python => {
                if self.inject_sdk_pythonpath {
                    format!(
                        "PYTHONPATH=/workspace/{}/sdk WORKSPACE=/workspace {run_cmd}",
                        self.agent_dir
                    )
                } else {
                    format!("WORKSPACE=/workspace {run_cmd}")
                }
            }
            Language::Shell
            | Language::JavaScript
            | Language::TypeScript
            | Language::CSharp => format!("cd /workspace && {run_cmd}"),
        };

        let mut exec = self.executor.lock().await;

        if let Err(e) = exec
            .inject_file(
                "agent-sandbox",
                &self.sandbox_image,
                block.code.as_bytes(),
                &container_script_path,
            )
            .await
        {
            return ScriptResult {
                stdout: String::new(),
                stderr: format!("Failed to inject script into sandbox: {e}"),
                exit_code: None,
            };
        }

        match exec
            .execute("agent-sandbox", &cmd, &self.sandbox_image, Some(300))
            .await
        {
            Ok(result) => ScriptResult {
                stdout: result.output.clone(),
                stderr: String::new(),
                exit_code: result.exit_code,
            },
            Err(e) => ScriptResult {
                stdout: String::new(),
                stderr: format!("Sandbox execution failed: {e}"),
                exit_code: None,
            },
        }
    }
}

// ── Agent task ────────────────────────────────────────────────────────────────

async fn agent_task(
    ctx: AgentContext,
    io: Arc<HickAgentIo>,
    mut history: AgentHistory,
) {
    let running = Arc::clone(&io.running);
    loop {
        running.store(true, Ordering::SeqCst);
        run_agent_engine(&ctx, &mut history, Arc::clone(&running)).await;

        // Check what caused the engine to exit.
        let pending = {
            let mut lock = io.pending.lock().await;
            std::mem::replace(&mut *lock, PendingCommand::None)
        };
        match pending {
            PendingCommand::Quit | PendingCommand::None => break,
            PendingCommand::Rewind { system_messages, user_assistant_history } => {
                let mut new_msgs = system_messages;
                new_msgs.extend(user_assistant_history);
                history.set_messages(new_msgs);
                history.mark_idle();
                // Update the system_messages snapshot for future rewinds.
                // (system messages don't change, so no action needed here.)
            }
        }
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let command = cli.command.unwrap_or(Commands::Agent {
        config: None,
        extra_configs: Vec::new(),
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
        no_web: false,
    });

    match command {
        Commands::Agent {
            config,
            extra_configs,
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
            no_web,
        } => {
            let project_dir = project_dir.unwrap_or_else(|| std::env::current_dir().unwrap());
            let (images_dir, default_image) = match images_dir {
                Some(d) => (d, default_image),
                None => match detect_images_dir(&project_dir) {
                    Some((d, img)) => {
                        println!("Auto-detected images: {} (image: {})", d.display(), img);
                        (d, img)
                    }
                    None => (PathBuf::from("."), default_image),
                },
            };

            // Load extensions from config (resolution: --config > .hick/agent.toml > ~/.hick/agent.toml > built-in default)
            let extensions = load_extensions(config.as_deref(), &project_dir);
            let hick_edit = extensions.hick_edit;

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

            let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<AgentCommand>();
            let interrupt = Arc::new(Notify::new());

            let security_config_path = project_dir.join(".hick").join("agent-security.toml");
            let security_config = AgentSecurityConfig::load_or_default(&security_config_path)?;
            let sandbox_caps = security_config.to_capabilities();

            let auth_rules_path = project_dir.join(".hick").join("auth-rules.toml");
            let age_key_path = std::env::var("HICK_AGE_KEY")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/Users/ai/Documents/src/key.txt"));
            let age_secrets_dir = project_dir.join(".hick").join("secrets");
            let auth_stack = match hick_literate::auth::AuthStack::from_config(
                &auth_rules_path,
                &age_key_path,
                &age_secrets_dir,
            ) {
                Ok(Some(stack)) => {
                    let stack = Arc::new(stack);
                    event_tx.send(AgentEvent::Info(format!(
                        "Auth injection: active ({} env vars)",
                        stack.container_env.len(),
                    ))).ok();
                    Some(stack)
                }
                Ok(None) => None,
                Err(e) => {
                    event_tx.send(AgentEvent::Info(format!("Auth injection: failed to load ({e})"))).ok();
                    None
                }
            };

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
                event_tx.send(AgentEvent::Info(format!(
                    "Network filter: active ({} rules, default {})",
                    filter.rule_count(),
                    filter.default_action_name(),
                ))).ok();
                Box::new(FilteredLlmClient::new(llm, filter, host))
            } else if security_config.deny_all_network {
                let filter = Arc::new(NetworkFilter::from_capabilities(
                    &sandbox_caps,
                    Some(&host),
                    approval_tx,
                ));
                event_tx.send(AgentEvent::Info(format!(
                    "Network filter: derived from security config ({} rules, default {})",
                    filter.rule_count(),
                    filter.default_action_name(),
                ))).ok();
                Box::new(FilteredLlmClient::new(llm, filter, host))
            } else {
                drop(approval_tx);
                llm
            };

            let provider_name = llm.provider_name().to_string();
            let model_name = llm.model_name().to_string();

            let output_dir = std::env::var("HICK_OUTPUT_DIR").ok();

            let sink_defs = security_config.to_sink_defs();
            let sink_guard = Arc::new(SinkGuard::new(sandbox_caps.clone(), sink_defs));

            let caps_summary = if security_config.deny_all_network {
                format!("deny-all + {} allowed hosts", security_config.allowed_hosts.len())
            } else {
                "permissive (no deny-all)".to_string()
            };

            let sandbox_guest_network = !matches!(
                std::env::var("HICK_AGENT_DISABLE_SANDBOX_GUEST_NET").as_deref(),
                Ok("1") | Ok("true") | Ok("yes")
            );

            let wasm_image_path = images_dir.join(format!("{}.wasm", default_image));
            hick_literate::executor::ensure_loadable_wasm_file(&wasm_image_path).map_err(|e| {
                anyhow::anyhow!(
                    "{e:#}\n\n\
                     Set --images-dir to the directory containing your .wasm images\n\
                     and --default-image to the image name (without the .wasm extension).\n\n\
                     If you have a `*.wasm.zip` next to the image, it will be extracted automatically when the `.wasm` is missing or corrupt.\n\n\
                     Example:\n\
                     \x20 hick agent \\\n\
                     \x20   --images-dir /path/to/converted_images/alpine \\\n\
                     \x20   --default-image 3.23.2",
                )
            })?;
            let sandbox_config = WasmSandboxConfig {
                images_dir: images_dir.clone(),
                workdir: project_dir.clone(),
                default_image: default_image.clone(),
                capabilities: sandbox_caps.clone(),
                auth: auth_stack,
                guest_network_enabled: sandbox_guest_network,
            };
            let sandbox_executor: Arc<Mutex<dyn CommandExecutor>> =
                Arc::new(Mutex::new(WasmSandboxExecutor::new(sandbox_config)));
            event_tx.send(AgentEvent::Info(format!(
                "Sandbox: WASM container enabled ({caps_summary})"
            ))).ok();

            // Build system prompt — include hick pipeline section only when hick_edit is enabled
            let system_prompt = if hick_edit {
                format!(
                    "{}\n\n---\n\n{}",
                    HICK_SYSTEM_PROMPT,
                    hick_agent_hick_edit::system_prompt_fragment()
                )
            } else {
                HICK_SYSTEM_PROMPT.to_string()
            };

            // Build sandbox probe text to append to the system prompt
            let probe_text: Option<String> = {
                let mut exec = sandbox_executor.lock().await;
                match exec
                    .execute("agent-sandbox", SANDBOX_RUNTIME_PROBE, &default_image, Some(120))
                    .await
                {
                    Ok(cmd_out) => Some(cmd_out.output),
                    Err(e) => Some(format!("Sandbox runtime probe failed: {e}")),
                }
            };

            // Enable hick pipeline integration when configured
            let inject_sdk = hick_edit;
            if hick_edit {
                // Copy the Python SDK into .hick/sdk/
                setup_sdk(&project_dir);

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

                event_tx.send(AgentEvent::Info("Pipeline: watching .hick files".into())).ok();
            }

            // Wire up optional policy evaluator.
            let policy_evaluator: Option<hick_policy::PolicyEvaluator> =
                if let Some(ref policy_path) = security_config.policy_script {
                    let resolved = project_dir.join(policy_path);
                    let canonical_project = project_dir.canonicalize().unwrap_or_default();
                    let is_valid_policy = resolved.exists()
                        && resolved.is_file()
                        && resolved.extension().is_some_and(|ext| ext == "py")
                        && resolved.canonicalize().ok().is_some_and(|p| p.starts_with(&canonical_project));

                    if is_valid_policy {
                        event_tx.send(AgentEvent::Info(format!("Policy: active ({})", resolved.display()))).ok();
                        Some(hick_policy::PolicyEvaluator::new(
                            resolved.to_str().unwrap_or(policy_path),
                        ))
                    } else if resolved.exists() {
                        event_tx.send(AgentEvent::Info(format!(
                            "Policy: rejected ({}) — must be a .py file within project directory",
                            resolved.display()
                        ))).ok();
                        None
                    } else {
                        event_tx.send(AgentEvent::Info(format!(
                            "Policy: script not found ({}), skipping",
                            resolved.display()
                        ))).ok();
                        None
                    }
                } else {
                    None
                };

            let _no_stream = no_stream; // kept for future streaming support

            event_tx.send(AgentEvent::Info(format!("Project: {}", project_dir.display()))).ok();

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

            let (ui_cmd_tx, ui_cmd_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiCommand>();

            let (replay_tx, replay_rx) = mpsc::unbounded_channel::<ReplayRequest>();

            tokio::spawn(run_ui_event_fanout_task(
                event_rx,
                approval_rx,
                ui_fanout_tx.clone(),
                Some(model_name.clone()),
                Some(default_image.clone()),
                Some(provider_info.clone()),
                ui_cmd_rx,
                cmd_tx.clone(),
                Arc::clone(&interrupt),
                Some(replay_tx.clone()),
            ));

            // Start the pane watcher
            {
                let pw = hick_agent_sdk::PaneWatcher::new(&project_dir, ui_fanout_tx.clone());
                tokio::spawn(pw.run(std::time::Duration::from_secs(2)));
            }

            // Connect to relay if HICK_RELAY_URL is set
            if let Ok(relay_url) = std::env::var("HICK_RELAY_URL") {
                let event_rx = ui_fanout_tx.subscribe();
                let cmd_tx_relay = ui_cmd_tx.clone();
                match hick_agent_sdk::RelayClient::connect(&relay_url, event_rx, cmd_tx_relay).await {
                    Ok(client) => {
                        event_tx.send(AgentEvent::Info(format!("Relay session: {}", client.session_url()))).ok();
                    }
                    Err(e) => {
                        event_tx.send(AgentEvent::Info(format!("Relay connect failed: {e}"))).ok();
                    }
                }
            }

            let session_log: Arc<dyn hick_agent_sdk::SessionLog> = {
                let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
                let bundle_dir = project_dir.join(".hick").join(format!("session-{ts}"));
                match XmlSessionLog::new(bundle_dir) {
                    Ok(log) => Arc::new(log),
                    Err(_) => Arc::new(NullSessionLog),
                }
            };
            log::set_boxed_logger(Box::new(XmlDiagnosticLogger::new(Arc::clone(&session_log)))).ok();
            log::set_max_level(log::LevelFilter::Info);

            // Build the initial conversation history with system prompt.
            let mut history = AgentHistory::new();
            let full_system_prompt: Arc<String> = {
                let mut full_system = system_prompt.clone();
                full_system.push_str(SANDBOX_EXECUTION_CONTEXT);
                if let Some(ref probe) = probe_text {
                    full_system.push_str(
                        "\n\n## Detected sandbox runtimes (automatic probe)\n\n```text\n",
                    );
                    full_system.push_str(probe.trim_end());
                    full_system.push_str("\n```\n");
                }
                history.push_system(full_system.clone());
                Arc::new(full_system)
            };
            let system_messages_snapshot: Vec<Message> = history.messages().to_vec();

            let running = Arc::new(AtomicBool::new(true));

            let hick_io = Arc::new(HickAgentIo {
                cmd_rx: tokio::sync::Mutex::new(cmd_rx),
                event_tx: event_tx.clone(),
                running: Arc::clone(&running),
                pending: tokio::sync::Mutex::new(PendingCommand::None),
                system_messages: tokio::sync::Mutex::new(system_messages_snapshot),
                executor: Arc::clone(&sandbox_executor),
                sandbox_image: default_image.clone(),
                agent_dir: ".hick".to_string(),
                project_dir: project_dir.clone(),
            });

            let hick_executor = Arc::new(HickToolExecutor {
                executor: Arc::clone(&sandbox_executor),
                sandbox_image: default_image.clone(),
                agent_dir: ".hick".to_string(),
                project_dir: project_dir.clone(),
                event_tx: event_tx.clone(),
                session_log: Arc::clone(&session_log),
                policy_evaluator,
                inject_sdk_pythonpath: inject_sdk,
            });

            let mut router = IntentRouter::new();
            router.register(ScriptBlockIntentParser);

            let ctx = AgentContext {
                llm,
                router,
                executor: hick_executor,
                io: Arc::clone(&hick_io) as Arc<dyn AgentIo>,
            };

            // Suppress unused warning — sink_guard is captured for future use
            let _ = sink_guard;

            tokio::spawn(agent_task(ctx, hick_io, history.clone()));

            // Per-tab channel pair (also used for web server fanout).
            let primary_tab_label = format!("{provider_name} / {model_name}");

            // Spawn judge task for primary config if configured (AC #2).
            if let Some(judge) = load_eval_judge(config.as_deref(), &project_dir) {
                let sub = ui_fanout_tx.subscribe();
                let task_cfg = judge_task::JudgeTaskConfig {
                    judge_section: judge,
                    config_name: primary_tab_label.clone(),
                };
                tokio::spawn(judge_task::run_judge_task(sub, task_cfg, ui_fanout_tx.clone()));
            }

            // ── Multi-config extra tabs ───────────────────────────────────────────
            // Build a TabSource for the primary config first, then one per extra config.
            // The add_tab sender stays in main for mid-session AddConfigPath commands.
            let (add_tab_tx, add_tab_rx) = mpsc::unbounded_channel::<TabSource>();

            let mut all_tab_sources: Vec<TabSource> = vec![TabSource {
                label: primary_tab_label.clone(),
                event_rx: ui_rx,
                cmd_tx: ui_cmd_tx.clone(),
            }];

            // Web server multi-tab attachment list (starts with primary).
            let mut web_tab_attachments = vec![hick_agent_sdk::WebServerTabAttachment {
                label: primary_tab_label.clone(),
                event_tx: ui_fanout_tx.clone(),
                cmd_tx: ui_cmd_tx.clone(),
            }];

            // Spawn extra config agents (multi-config mode, AC #1).
            for extra_cfg_path in &extra_configs {
                let extra_cfg_path = extra_cfg_path.clone();
                let extra_label = extra_cfg_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("config")
                    .to_string();

                // Channels for this extra agent instance.
                let (ex_event_tx, ex_event_rx) = mpsc::unbounded_channel::<AgentEvent>();
                let (ex_cmd_tx, ex_cmd_rx) = mpsc::unbounded_channel::<AgentCommand>();
                let (ex_approval_tx, ex_approval_rx) = mpsc::unbounded_channel::<NetworkApprovalRequest>();
                let ex_interrupt = Arc::new(Notify::new());

                let ex_extensions = load_extensions(Some(&extra_cfg_path), &project_dir);
                let ex_hick_edit = ex_extensions.hick_edit;

                // Build LLM for this config (detect from the same env vars for now).
                let ex_detected = detect_providers();
                let ex_provider = match prompt_provider_selection(ex_detected) {
                    Ok(p) => p,
                    Err(e) => {
                        event_tx.send(AgentEvent::Info(format!("extra config {extra_label}: provider error: {e}"))).ok();
                        continue;
                    }
                };
                let ex_local_opts = LocalOpts {
                    model_path: None,
                    n_gpu_layers: None,
                    n_ctx: 4096,
                    max_tokens: 4096,
                    project_dir: project_dir.clone(),
                };
                let ex_llm = match build_llm_client(ex_provider, None, None, None, ex_local_opts) {
                    Ok(l) => l,
                    Err(e) => {
                        event_tx.send(AgentEvent::Info(format!("extra config {extra_label}: llm error: {e}"))).ok();
                        continue;
                    }
                };

                let ex_provider_name = ex_llm.provider_name().to_string();
                let ex_model_name = ex_llm.model_name().to_string();
                let ex_tab_label = format!("{ex_provider_name} / {ex_model_name} [{extra_label}]");

                const EX_BROADCAST_CAP: usize = 2048;
                let (ex_ui_fanout_tx, _) = broadcast::channel::<hick_agent_sdk::UiEvent>(EX_BROADCAST_CAP);
                let (ex_ui_mpsc_tx, ex_ui_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiEvent>();
                let (ex_ui_cmd_tx, ex_ui_cmd_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiCommand>();

                // Subscribe to the per-config broadcast for the mpsc bridge.
                tokio::spawn({
                    let mut sub = ex_ui_fanout_tx.subscribe();
                    let tx = ex_ui_mpsc_tx.clone();
                    async move {
                        loop {
                            match sub.recv().await {
                                Ok(ev) => { if tx.send(ev).is_err() { break; } }
                                Err(broadcast::error::RecvError::Lagged(_)) => {}
                                Err(broadcast::error::RecvError::Closed) => break,
                            }
                        }
                    }
                });

                let ex_fanout_tx_clone = ex_ui_fanout_tx.clone();
                tokio::spawn(run_ui_event_fanout_task(
                    ex_event_rx,
                    ex_approval_rx,
                    ex_fanout_tx_clone,
                    Some(ex_model_name.clone()),
                    Some(default_image.clone()),
                    Some(format!("{ex_provider_name} / {ex_model_name}")),
                    ex_ui_cmd_rx,
                    ex_cmd_tx.clone(),
                    Arc::clone(&ex_interrupt),
                    Some(replay_tx.clone()),
                ));

                let ex_system = if ex_hick_edit {
                    format!("{}\n\n---\n\n{}", HICK_SYSTEM_PROMPT, hick_agent_hick_edit::system_prompt_fragment())
                } else {
                    HICK_SYSTEM_PROMPT.to_string()
                };

                // Share history context: inject primary agent's non-system messages so all
                // configs start with the same context (AC #6 — shared history on mode switch).
                let mut ex_history = AgentHistory::new();
                let ex_system_with_context = format!("{ex_system}{SANDBOX_EXECUTION_CONTEXT}");
                ex_history.push_system(ex_system_with_context);
                for msg in history.messages() {
                    if msg.role != hick_agent_sdk::Role::System {
                        ex_history.push(msg.clone());
                    }
                }

                let ex_running = Arc::new(AtomicBool::new(true));
                let ex_sandbox_config = WasmSandboxConfig {
                    images_dir: images_dir.clone(),
                    workdir: project_dir.clone(),
                    default_image: default_image.clone(),
                    capabilities: security_config.to_capabilities(),
                    auth: None,
                    guest_network_enabled: sandbox_guest_network,
                };
                let ex_sandbox_executor: Arc<Mutex<dyn CommandExecutor>> =
                    Arc::new(Mutex::new(WasmSandboxExecutor::new(ex_sandbox_config)));

                let ex_session_log: Arc<dyn hick_agent_sdk::SessionLog> = {
                    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
                    let bundle_dir = project_dir.join(".hick").join(format!("session-{ts}-{extra_label}"));
                    match XmlSessionLog::new(bundle_dir) {
                        Ok(log) => Arc::new(log),
                        Err(_) => Arc::new(NullSessionLog),
                    }
                };

                let ex_hick_io = Arc::new(HickAgentIo {
                    cmd_rx: tokio::sync::Mutex::new(ex_cmd_rx),
                    event_tx: ex_event_tx.clone(),
                    running: Arc::clone(&ex_running),
                    pending: tokio::sync::Mutex::new(PendingCommand::None),
                    system_messages: tokio::sync::Mutex::new(ex_history.messages().to_vec()),
                    executor: Arc::clone(&ex_sandbox_executor),
                    sandbox_image: default_image.clone(),
                    agent_dir: ".hick".to_string(),
                    project_dir: project_dir.clone(),
                });

                let ex_hick_executor = Arc::new(HickToolExecutor {
                    executor: Arc::clone(&ex_sandbox_executor),
                    sandbox_image: default_image.clone(),
                    agent_dir: ".hick".to_string(),
                    project_dir: project_dir.clone(),
                    event_tx: ex_event_tx,
                    session_log: Arc::clone(&ex_session_log),
                    policy_evaluator: None,
                    inject_sdk_pythonpath: ex_hick_edit,
                });

                let mut ex_router = IntentRouter::new();
                ex_router.register(ScriptBlockIntentParser);

                let ex_ctx = AgentContext {
                    llm: ex_llm,
                    router: ex_router,
                    executor: ex_hick_executor,
                    io: Arc::clone(&ex_hick_io) as Arc<dyn AgentIo>,
                };

                tokio::spawn(agent_task(ex_ctx, ex_hick_io, ex_history));

                // Spawn judge task for this extra config if configured (AC #2).
                if let Ok(config::AgentConfig::Point(pc)) =
                    config::AgentConfig::load(Some(&extra_cfg_path), &project_dir)
                {
                    if let Some(judge) = pc.eval.judge {
                        let sub = ex_ui_fanout_tx.subscribe();
                        let task_cfg = judge_task::JudgeTaskConfig {
                            judge_section: judge,
                            config_name: ex_tab_label.clone(),
                        };
                        tokio::spawn(judge_task::run_judge_task(
                            sub,
                            task_cfg,
                            ex_ui_fanout_tx.clone(),
                        ));
                    }
                }

                all_tab_sources.push(TabSource {
                    label: ex_tab_label.clone(),
                    event_rx: ex_ui_rx,
                    cmd_tx: ex_ui_cmd_tx.clone(),
                });

                web_tab_attachments.push(hick_agent_sdk::WebServerTabAttachment {
                    label: ex_tab_label,
                    event_tx: ex_ui_fanout_tx,
                    cmd_tx: ex_ui_cmd_tx,
                });
            }

            // ── Replay request handler (TASK-47) ─────────────────────────────────
            {
                let replay_images_dir = images_dir.clone();
                let replay_default_image = default_image.clone();
                let replay_project_dir = project_dir.clone();
                let replay_sandbox_guest_network = sandbox_guest_network;
                let replay_security_config = sandbox_caps.clone();
                let replay_full_system = Arc::clone(&full_system_prompt);
                let replay_add_tab_tx = add_tab_tx.clone();
                let replay_event_tx = event_tx.clone();
                tokio::spawn(async move {
                    handle_replay_requests(
                        replay_rx,
                        replay_images_dir,
                        replay_default_image,
                        replay_project_dir,
                        replay_sandbox_guest_network,
                        replay_security_config,
                        replay_full_system,
                        replay_add_tab_tx,
                        replay_event_tx,
                    ).await;
                });
            }

            // ── Web server — multi-tab attach ─────────────────────────────────────
            let web_ui_url = if web_ui_enabled(no_web) {
                let security_cfg = hick_agent_sdk::AgentSecurityConfig::load_or_default(&project_dir)
                    .unwrap_or_default();
                let cfg = hick_agent_sdk::WebServerConfig::from_env()
                    .with_project(project_dir.clone(), security_cfg.allowed_hosts.clone());
                match hick_agent_sdk::WebServer::bind(cfg).await {
                    Ok(ws) => {
                        if web_tab_attachments.len() == 1 {
                            // Single-config: use legacy single attach.
                            ws.attach(ui_fanout_tx.clone(), ui_cmd_tx.clone());
                        } else {
                            ws.attach_tabs(web_tab_attachments);
                        }
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

            // ── Eval store: write results when multi-config session completes (AC #8) ──
            if !extra_configs.is_empty() {
                let project_dir_eval = project_dir.clone();
                let config_names: Vec<String> = std::iter::once(primary_tab_label.clone())
                    .chain(extra_configs.iter().map(|p| {
                        p.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("config")
                            .to_string()
                    }))
                    .collect();
                tokio::spawn(async move {
                    write_eval_session_records(&project_dir_eval, &config_names).await;
                });
            }

            // ── Handle mid-session AddConfigPath commands from TUI/web ────────────
            // (forward them back to the main loop via add_tab_tx)
            // We intercept AddConfigPath from the shared ui_cmd_tx stream.
            // For simplicity, ignore them for now — dynamic adding is wired via add_tab_rx.
            let _ = add_tab_tx; // kept for future mid-session addition

            // Rating channel: TUI writes RatingSubmission; handler persists HumanRatingRecord.
            let (rating_tx, rating_rx) = mpsc::unbounded_channel::<hick_agent_tui::RatingSubmission>();
            tokio::spawn(handle_rating_submissions(rating_rx));

            let interrupt_tui = Arc::clone(&interrupt);
            tokio::task::spawn_blocking(move || {
                run_tui(
                    all_tab_sources,
                    TuiConfig {
                        agent_name: "hick".to_string(),
                        provider_info,
                        project_dir,
                        interrupt: interrupt_tui,
                        ui_fanout_tx: ui_fanout_tx.clone(),
                        web_ui_url,
                        replay_label: None,
                        add_tab_rx: Some(add_tab_rx),
                        rating_tx: Some(rating_tx),
                    },
                )
            })
            .await??;
            Ok(())
        }

        Commands::Run(args) => {
            hick_literate::run_pipeline_cmd(hick_literate::PipelineRunOpts {
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
            use hick_literate::store_config::{StoreBackend, StoreConfig};
            let key_path = hick_literate::expand_tilde(&args.key_file);
            let secrets_dir_path = hick_literate::expand_tilde(&args.secrets_dir);
            let _secrets_provider = hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir_path);

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
                .map(|p| hick_literate::expand_tilde(&p))
                .unwrap_or_else(|| std::path::PathBuf::from("."));

            let mut params: Vec<(String, String)> = args.params.clone();
            if let Some(ref features) = args.features {
                params.retain(|(pk, _)| pk != "features");
                params.push(("features".to_string(), features.clone()));
            }

            hick_literate::watch::run_watch(&args.files, &config, &params, args.dry_run, images_dir).await
        }

        Commands::Repl(args) => {
            let images_dir = args
                .images_dir
                .map(|p| hick_literate::expand_tilde(&p))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            hick_literate::repl::run_repl(images_dir, args.output).await
        }

        Commands::Init(args) => {
            let project_dir = args
                .project_dir
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let config = hick_literate::agents::InitConfig {
                project_dir: project_dir.clone(),
                no_agents: args.no_agents,
            };
            let result = hick_literate::agents::init_agents(&config)?;
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
            let config = hick_literate::agents::ValidateConfig { project_dir, fix: args.fix };
            let issues = hick_literate::agents::validate_agents(&config)?;
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

        Commands::GenerateMatrix(args) => hick_literate::generate_matrix::run(args).await,

        Commands::PasteSlots(args) => paste_slots_cmd(args),

        Commands::Mcp { cmd } => mcp_cmd(cmd).await,

        Commands::Replay { bundle_dir, replay_speed, project_dir } => {
            let project_dir = project_dir.unwrap_or_else(|| std::env::current_dir().unwrap());
            run_replay_tui(&bundle_dir, replay_speed, project_dir).await
        }
    }
}

// ── Replay requests handler (TASK-47) ────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn handle_replay_requests(
    mut replay_rx: mpsc::UnboundedReceiver<ReplayRequest>,
    images_dir: PathBuf,
    default_image: String,
    project_dir: PathBuf,
    sandbox_guest_network: bool,
    sandbox_caps: hick_token::ContainerCapabilities,
    full_system_prompt: Arc<String>,
    add_tab_tx: mpsc::UnboundedSender<TabSource>,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
) {
    while let Some(req) = replay_rx.recv().await {
        let turn_id = req.turn_node_id.as_string();
        for config_path_str in &req.config_paths {
            let config_path = PathBuf::from(config_path_str);
            let config_label = config_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("replay")
                .to_string();

            let extensions = load_extensions(Some(&config_path), &project_dir);

            let detected = detect_providers();
            let provider = match prompt_provider_selection(detected) {
                Ok(p) => p,
                Err(e) => {
                    event_tx.send(AgentEvent::Info(format!(
                        "[replay] {config_label}: provider error: {e}"
                    ))).ok();
                    continue;
                }
            };
            let local_opts = LocalOpts {
                model_path: None,
                n_gpu_layers: None,
                n_ctx: 4096,
                max_tokens: 4096,
                project_dir: project_dir.clone(),
            };
            let llm = match build_llm_client(provider, None, None, None, local_opts) {
                Ok(l) => l,
                Err(e) => {
                    event_tx.send(AgentEvent::Info(format!(
                        "[replay] {config_label}: llm error: {e}"
                    ))).ok();
                    continue;
                }
            };

            let provider_name = llm.provider_name().to_string();
            let model_name = llm.model_name().to_string();
            let tab_label = format!("{provider_name} / {model_name} [replay:{config_label}]");

            let (rep_event_tx, rep_event_rx) = mpsc::unbounded_channel::<AgentEvent>();
            let (rep_cmd_tx, rep_cmd_rx) = mpsc::unbounded_channel::<AgentCommand>();
            let (rep_approval_tx, rep_approval_rx) = mpsc::unbounded_channel::<NetworkApprovalRequest>();
            let rep_interrupt = Arc::new(tokio::sync::Notify::new());

            const REP_BROADCAST_CAP: usize = 2048;
            let (rep_ui_fanout_tx, _) = broadcast::channel::<hick_agent_sdk::UiEvent>(REP_BROADCAST_CAP);
            let (rep_ui_mpsc_tx, rep_ui_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiEvent>();
            let (rep_ui_cmd_tx, rep_ui_cmd_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiCommand>();

            tokio::spawn({
                let mut sub = rep_ui_fanout_tx.subscribe();
                let tx = rep_ui_mpsc_tx.clone();
                async move {
                    loop {
                        match sub.recv().await {
                            Ok(ev) => { if tx.send(ev).is_err() { break; } }
                            Err(broadcast::error::RecvError::Lagged(_)) => {}
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            });

            let rep_provider_info = format!("{provider_name} / {model_name}");
            tokio::spawn(run_ui_event_fanout_task(
                rep_event_rx,
                rep_approval_rx,
                rep_ui_fanout_tx.clone(),
                Some(model_name.clone()),
                Some(default_image.clone()),
                Some(rep_provider_info.clone()),
                rep_ui_cmd_rx,
                rep_cmd_tx.clone(),
                Arc::clone(&rep_interrupt),
                None, // replay agents do not themselves spawn further replays
            ));

            // Seed history with context up to (not including) the selected turn.
            let mut rep_history = AgentHistory::new();
            rep_history.push_system((*full_system_prompt).clone());
            for msg in &req.context_history {
                rep_history.push(msg.clone());
            }

            let rep_running = Arc::new(AtomicBool::new(true));
            let rep_sandbox_config = WasmSandboxConfig {
                images_dir: images_dir.clone(),
                workdir: project_dir.clone(),
                default_image: default_image.clone(),
                capabilities: sandbox_caps.clone(),
                auth: None,
                guest_network_enabled: sandbox_guest_network,
            };
            let rep_sandbox_executor: Arc<Mutex<dyn CommandExecutor>> =
                Arc::new(Mutex::new(WasmSandboxExecutor::new(rep_sandbox_config)));

            let rep_session_log: Arc<dyn hick_agent_sdk::SessionLog> = {
                let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
                let bundle_dir = project_dir.join(".hick").join(format!("session-{ts}-replay-{config_label}"));
                match XmlSessionLog::new(bundle_dir) {
                    Ok(log) => Arc::new(log),
                    Err(_) => Arc::new(NullSessionLog),
                }
            };

            let rep_hick_io = Arc::new(HickAgentIo {
                cmd_rx: tokio::sync::Mutex::new(rep_cmd_rx),
                event_tx: rep_event_tx.clone(),
                running: Arc::clone(&rep_running),
                pending: tokio::sync::Mutex::new(PendingCommand::None),
                system_messages: tokio::sync::Mutex::new(rep_history.messages().to_vec()),
                executor: Arc::clone(&rep_sandbox_executor),
                sandbox_image: default_image.clone(),
                agent_dir: ".hick".to_string(),
                project_dir: project_dir.clone(),
            });

            let rep_hick_executor = Arc::new(HickToolExecutor {
                executor: Arc::clone(&rep_sandbox_executor),
                sandbox_image: default_image.clone(),
                agent_dir: ".hick".to_string(),
                project_dir: project_dir.clone(),
                event_tx: rep_event_tx,
                session_log: Arc::clone(&rep_session_log),
                policy_evaluator: None,
                inject_sdk_pythonpath: extensions.hick_edit,
            });

            let mut rep_router = IntentRouter::new();
            rep_router.register(ScriptBlockIntentParser);

            let rep_ctx = AgentContext {
                llm,
                router: rep_router,
                executor: rep_hick_executor,
                io: Arc::clone(&rep_hick_io) as Arc<dyn AgentIo>,
            };

            tokio::spawn(agent_task(rep_ctx, rep_hick_io, rep_history));

            // Send the selected turn's user message as the first prompt.
            rep_cmd_tx.send(AgentCommand::Message(req.user_message.clone())).ok();

            // Register the tab so TUI shows it.
            add_tab_tx.send(TabSource {
                label: tab_label.clone(),
                event_rx: rep_ui_rx,
                cmd_tx: rep_ui_cmd_tx,
            }).ok();

            event_tx.send(AgentEvent::Info(format!(
                "[replay] Tab added: {tab_label}"
            ))).ok();

            // Write eval record with reference to the original turn.
            write_replay_eval_record(&project_dir, config_path_str, &req.user_message, &turn_id).await;
        }
    }
}

async fn write_replay_eval_record(
    project_dir: &std::path::Path,
    config_name: &str,
    query: &str,
    original_turn_id: &str,
) {
    use hick_eval_store::{EvalRecord, EvalResultRecord, GitBackend, StorageBackend};

    let backend = match GitBackend::new() {
        Ok(b) => b,
        Err(e) => {
            log::warn!("eval store: failed to open git backend for replay record: {e}");
            return;
        }
    };

    let machine_id = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let record = EvalResultRecord {
        machine_id,
        timestamp: chrono::Utc::now(),
        config_name: config_name.to_string(),
        query: Some(query.to_string()),
        response: None,
        scores: None,
        token_cost: None,
        original_turn_id: Some(original_turn_id.to_string()),
        wall_secs: None,
        rework_count: None,
    };

    if let Err(e) = backend.append(&EvalRecord::EvalResult(record)) {
        log::warn!("eval store: failed to append replay record for {config_name}: {e}");
    }
    backend.sync_session();
}

// ── Eval store — multi-config session write (AC #8) ──────────────────────────

async fn write_eval_session_records(project_dir: &std::path::Path, config_names: &[String]) {
    use hick_eval_store::{EvalRecord, EvalResultRecord, GitBackend, StorageBackend};

    let backend = match GitBackend::new() {
        Ok(b) => b,
        Err(e) => {
            log::warn!("eval store: failed to open git backend: {e}");
            return;
        }
    };

    let machine_id = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    for config_name in config_names {
        let record = EvalResultRecord {
            machine_id: machine_id.clone(),
            timestamp: chrono::Utc::now(),
            config_name: config_name.clone(),
            query: None,
            response: None,
            scores: None,
            token_cost: None,
            original_turn_id: None,
            wall_secs: None,
            rework_count: None,
        };
        if let Err(e) = backend.append(&EvalRecord::EvalResult(record)) {
            log::warn!("eval store: failed to append record for {config_name}: {e}");
        }
    }

    backend.sync_session();
}

// ── Rating handler (TASK-48 AC #4) ───────────────────────────────────────────

async fn handle_rating_submissions(
    mut rating_rx: mpsc::UnboundedReceiver<hick_agent_tui::RatingSubmission>,
) {
    use hick_eval_store::{EvalRecord, GitBackend, HumanRatingRecord, MachineId, StorageBackend};

    while let Some(sub) = rating_rx.recv().await {
        let backend = match GitBackend::new() {
            Ok(b) => b,
            Err(e) => {
                log::warn!("eval store: failed to open git backend for rating: {e}");
                continue;
            }
        };

        let machine_id = MachineId::load()
            .map(|m| m.uuid().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let judge_ts = sub
            .judge_timestamp
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|t| t.with_timezone(&chrono::Utc));

        let record = HumanRatingRecord {
            machine_id,
            timestamp: chrono::Utc::now(),
            config_name: sub.config_name,
            query: sub.query,
            scores: sub.scores,
            judge_timestamp: judge_ts,
        };

        if let Err(e) = backend.append(&EvalRecord::HumanRating(record)) {
            log::warn!("eval store: failed to append human rating: {e}");
        } else {
            backend.sync_session();
        }
    }
}

// ── Replay ────────────────────────────────────────────────────────────────────

async fn run_replay_tui(
    bundle_dir: &std::path::Path,
    replay_speed: f64,
    project_dir: PathBuf,
) -> Result<()> {
    use hick_agent_sdk::SessionBundleReader;

    let label = bundle_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("session")
        .to_string();

    let events = SessionBundleReader::read(bundle_dir)
        .map_err(|e| anyhow::anyhow!("failed to load session bundle: {e}"))?;

    const UI_BROADCAST_CAP: usize = 2048;
    let (ui_fanout_tx, _) = broadcast::channel::<hick_agent_sdk::UiEvent>(UI_BROADCAST_CAP);
    let (ui_mpsc_tx, ui_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiEvent>();
    let (ui_cmd_tx, _ui_cmd_rx) = mpsc::unbounded_channel::<hick_agent_sdk::UiCommand>();
    let interrupt = Arc::new(Notify::new());

    let fanout_clone = ui_fanout_tx.clone();
    tokio::spawn(async move {
        for ev in events {
            if replay_speed > 0.0 {
                let delay_ms = (50.0 / replay_speed) as u64;
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }
            let _ = fanout_clone.send(ev);
        }
    });

    tokio::spawn({
        let mut sub = ui_fanout_tx.subscribe();
        let ui_mpsc_tx = ui_mpsc_tx.clone();
        async move {
            loop {
                match sub.recv().await {
                    Ok(ev) => { if ui_mpsc_tx.send(ev).is_err() { break; } }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    });

    let interrupt_tui = Arc::clone(&interrupt);
    tokio::task::spawn_blocking(move || {
        run_tui(
            vec![TabSource {
                label: label.clone(),
                event_rx: ui_rx,
                cmd_tx: ui_cmd_tx,
            }],
            TuiConfig {
                agent_name: "hick".to_string(),
                provider_info: String::new(),
                project_dir,
                interrupt: interrupt_tui,
                ui_fanout_tx,
                web_ui_url: None,
                replay_label: Some(label),
                add_tab_rx: None,
                rating_tx: None,
            },
        )
    })
    .await??;

    Ok(())
}

// ── MCP subcommand ────────────────────────────────────────────────────────────

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

            if let Some(images_dir) = args.images_dir {
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
                    default_image: args.default_image,
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
                    "Skipping WASM package install (use --images-dir to enable). \
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

// ── Paste slots subcommand ────────────────────────────────────────────────────

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

    let mut regions: Vec<(usize, usize, String)> = Vec::new();
    for file_tag in doc.find_tags("file") {
        let Some(file_path) = file_tag.get_attribute("path") else { continue; };
        let Some(span) = file_tag.source_span else { continue; };
        let content_start = span.end;
        let Some(close_rel) = source[content_start..].find(&close_marker) else { continue; };
        let content_end = content_start + close_rel;
        regions.push((content_start, content_end, file_path.to_string()));
    }

    if regions.is_empty() {
        println!("No hick:file elements found in '{}'.", args.file.display());
        return Ok(());
    }

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
        println!("Updated {} hick:file element(s) in '{}'.", modified_count, args.file.display());
    } else {
        println!(
            "Would update {} hick:file element(s) in '{}'. Use --write to apply.",
            modified_count,
            args.file.display()
        );
    }

    let _ = open_marker_prefix;
    Ok(())
}

// ── SDK setup ─────────────────────────────────────────────────────────────────

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

// ── SdkExecutorBridge ─────────────────────────────────────────────────────────

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
