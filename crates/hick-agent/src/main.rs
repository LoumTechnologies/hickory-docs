//! hick-agent: AI command-line coding agent.
//!
//! Uses Hick XML as the edit format and Hick WASM containers for sandboxed
//! command execution. All operations run in-process.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use hick_agent::{
    Agent, AgentConfig, AnthropicClient, LlmClient, OpenAiCompatClient, PipelineWatcher, ReactLoop,
};

#[derive(Parser)]
#[command(name = "hick-agent")]
#[command(about = "AI coding agent using Hick XML edit format and WASM sandbox")]
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
        #[arg(long, value_enum, default_value = "openai", env = "HICK_AGENT_PROVIDER")]
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
        /// Disable streaming output
        #[arg(long)]
        no_stream: bool,
    },
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
                let key = api_key
                    .unwrap_or_else(|| std::env::var("OPENAI_API_KEY").unwrap_or_default());
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
            let base = api_base
                .ok_or_else(|| anyhow::anyhow!("--api-base is required for openai-compat provider"))?;
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

#[tokio::main]
async fn main() -> Result<()> {
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
            no_stream,
        } => {
            let config = AgentConfig {
                project_dir: project_dir.unwrap_or_else(|| std::env::current_dir().unwrap()),
                images_dir: images_dir.unwrap_or_else(|| PathBuf::from(".")),
                default_image,
            };

            let llm = build_llm_client(provider, model, api_base, api_key)?;
            let agent = Agent::new(config.clone(), llm);
            let output_dir = std::env::var("HICK_OUTPUT_DIR").ok();
            let mut loop_ = ReactLoop::new(agent, config.project_dir.clone(), output_dir.clone());
            loop_.stream_output = !no_stream;

            // Start the transparent pipeline watcher in the background
            let watcher = std::sync::Arc::new(PipelineWatcher::new(
                config.project_dir.clone(),
                output_dir,
            ));
            let watcher_handle = {
                let watcher = std::sync::Arc::clone(&watcher);
                tokio::spawn(async move {
                    if let Err(e) = watcher.watch_loop().await {
                        eprintln!("Pipeline watcher error: {e}");
                    }
                })
            };

            println!("hick-agent: AI coding agent (ReAct loop)");
            println!("  Project:  {}", config.project_dir.display());
            println!("  Provider: {}", loop_.agent.provider_name());
            println!("  Model:    {}", loop_.agent.model_name());
            println!("  Images:   {}", loop_.agent.config().images_dir.display());
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
