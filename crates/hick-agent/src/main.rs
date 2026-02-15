//! hick-agent: AI command-line coding agent.
//!
//! Uses Hick XML as the edit format and Hick WASM containers for sandboxed
//! command execution. All operations run in-process.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use hick_agent::{Agent, AgentConfig, LlmClient, OpenAiClient, ReactLoop, XaiClient};

#[derive(Parser)]
#[command(name = "hick-agent")]
#[command(about = "AI coding agent using Hick XML edit format and WASM sandbox")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum LlmProvider {
    /// OpenAI API (OPENAI_API_KEY)
    #[default]
    Openai,
    /// xAI API (XAI_API_KEY)
    Xai,
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
        /// LLM provider: openai (default) or xai
        #[arg(long, value_enum, default_value = "openai")]
        llm: LlmProvider,
    },
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
            llm,
        } => {
            let config = AgentConfig {
                project_dir: project_dir.unwrap_or_else(|| std::env::current_dir().unwrap()),
                images_dir: images_dir.unwrap_or_else(|| PathBuf::from(".")),
                default_image,
            };

            let llm: Box<dyn LlmClient> = match llm {
                LlmProvider::Openai => Box::new(OpenAiClient::new()),
                LlmProvider::Xai => Box::new(XaiClient::new()),
            };
            let agent = Agent::new(config.clone(), llm);
            let output_dir = std::env::var("HICK_OUTPUT_DIR").ok();
            let mut loop_ = ReactLoop::new(agent, config.project_dir.clone(), output_dir);

            println!("hick-agent: AI coding agent (ReAct loop)");
            println!("  Project: {}", config.project_dir.display());
            println!("  Images:  {}", loop_.agent.config().images_dir.display());
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
        }
    }

    Ok(())
}
