//! hick-agent: AI command-line coding agent.
//!
//! Script-first model: the LLM writes Python scripts, which are executed
//! and their output returned as observations. The Hick pipeline runs
//! transparently via a file watcher.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
#[cfg(feature = "local")]
use hick_agent::LocalLlmClient;
use hick_agent::{
    Agent, AgentConfig, AnthropicClient, LlmClient, OpenAiCompatClient, PipelineWatcher, ReactLoop,
};

#[derive(Parser)]
#[command(name = "hick-agent")]
#[command(about = "AI coding agent using Hick XML — script-first model")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum Provider {
    /// OpenAI API (OPENAI_API_KEY)
    #[default]
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
        /// LLM provider
        #[arg(
            long,
            value_enum,
            default_value = "openai",
            env = "HICK_AGENT_PROVIDER"
        )]
        provider: Provider,
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
            let model = model.unwrap_or_else(|| "grok-2".into());
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
            let config = AgentConfig {
                project_dir: project_dir.unwrap_or_else(|| std::env::current_dir().unwrap()),
                images_dir: images_dir.unwrap_or_else(|| PathBuf::from(".")),
                default_image,
            };

            let local_opts = LocalOpts {
                model_path,
                n_gpu_layers,
                n_ctx,
                max_tokens,
                project_dir: config.project_dir.clone(),
            };
            let llm = build_llm_client(provider, model, api_base, api_key, local_opts)?;
            let agent = Agent::new(config.clone(), llm);
            let output_dir = std::env::var("HICK_OUTPUT_DIR").ok();
            let mut loop_ = ReactLoop::new(agent, config.project_dir.clone());
            loop_.stream_output = !no_stream;

            // Start the transparent pipeline watcher in the background
            let watcher =
                std::sync::Arc::new(PipelineWatcher::new(config.project_dir.clone(), output_dir));
            let watcher_handle = {
                let watcher = std::sync::Arc::clone(&watcher);
                tokio::spawn(async move {
                    if let Err(e) = watcher.watch_loop().await {
                        eprintln!("Pipeline watcher error: {e}");
                    }
                })
            };

            // Copy the Python SDK into .hick/sdk/ so scripts can import it
            setup_sdk(&config.project_dir);

            println!("hick-agent: AI coding agent (script-first)");
            println!("  Project:  {}", config.project_dir.display());
            println!("  Provider: {}", loop_.agent.provider_name());
            println!("  Model:    {}", loop_.agent.model_name());
            println!("  Stream:   {}", !no_stream);
            println!("  Pipeline: watching .hick files");
            println!();
            println!("Enter your request (or 'quit' to exit):");

            let mut buffer = String::new();
            loop {
                buffer.clear();
                if std::io::stdin().read_line(&mut buffer).is_err() || buffer.is_empty() {
                    break;
                }
                let input = buffer.trim();
                if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
                    break;
                }

                match loop_.run(input).await {
                    Ok(response) => println!("{}\n", response),
                    Err(e) => eprintln!("Error: {e}"),
                }
            }

            watcher_handle.abort();
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
