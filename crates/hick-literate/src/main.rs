//! Hick: Secure container orchestration shell.
//!
//! Usage:
//!   `hick [file.hick ...] [--config _hick.yml]` — single-shot pipeline run
//!   `hick up <file.hick> [file2.hick ...]`       — watch mode with merge-aware snapshots

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use hick_literate::store_config::{StoreBackend, StoreConfig};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "hick", about = "Secure container orchestration shell")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the pipeline once (default mode).
    Run(RunArgs),

    /// Watch mode: re-run pipeline on file changes with merge-aware snapshots.
    Up(UpArgs),

    /// Interactive REPL: explore containers interactively and record a session transcript.
    Repl(ReplArgs),

    /// Initialize AI agent instructions for the project.
    Init(InitArgs),

    /// Validate AI agent configuration.
    Validate(ValidateArgs),

    /// Generate all combinations of features to separate folders.
    GenerateMatrix(hick_literate::generate_matrix::GenerateMatrixArgs),

    /// Inspect the pipeline DAG and file ownership.
    Pipeline(PipelineArgs),
}

#[derive(clap::Args)]
struct PipelineArgs {
    #[command(subcommand)]
    command: PipelineCommand,
}

#[derive(Subcommand)]
enum PipelineCommand {
    /// Print the pipeline DAG with file ownership and paste slots.
    Show(PipelineShowArgs),
    /// Show which output files are pipeline-owned vs untracked on disk.
    Status(PipelineStatusArgs),
}

#[derive(clap::Args)]
struct PipelineShowArgs {
    /// Project directory (default: current dir).
    #[arg(long)]
    project_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
struct PipelineStatusArgs {
    /// Project directory (default: current dir).
    #[arg(long)]
    project_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
struct RunArgs {
    /// `.hick` files to process (overrides `_hick.yml` files list).
    files: Vec<PathBuf>,

    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long)]
    key_file: Option<PathBuf>,

    #[arg(long)]
    secrets_dir: Option<PathBuf>,

    #[arg(long)]
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
struct UpArgs {
    #[arg(required = true)]
    files: Vec<PathBuf>,

    #[arg(long, default_value = "~/.config/hick/key.txt")]
    key_file: PathBuf,

    #[arg(long, default_value = "~/.config/hick/secrets")]
    secrets_dir: PathBuf,

    #[arg(long)]
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
struct ReplArgs {
    #[arg(long)]
    images_dir: Option<PathBuf>,

    #[arg(long, default_value = "session.hick")]
    output: PathBuf,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::Args)]
struct InitArgs {
    #[arg(long)]
    project_dir: Option<PathBuf>,

    #[arg(long)]
    no_agents: bool,
}

#[derive(clap::Args)]
struct ValidateArgs {
    #[arg(long)]
    project_dir: Option<PathBuf>,

    #[arg(long)]
    fix: bool,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

/// Parse CLI, allowing bare `hick <files>` as sugar for `hick run <files>`.
///
/// When invoked with no arguments at all, enters REPL mode.
fn parse_cli() -> Cli {
    let raw_args: Vec<String> = std::env::args().collect();

    if raw_args.len() == 1 {
        return Cli {
            command: Command::Repl(ReplArgs {
                images_dir: None,
                output: PathBuf::from("session.hick"),
                verbose: false,
            }),
        };
    }

    let first_positional = raw_args.iter().skip(1).find(|a| !a.starts_with('-'));

    let is_subcommand = first_positional.is_some_and(|a| {
        matches!(
            a.as_str(),
            "run" | "up" | "repl" | "init" | "validate" | "generate-matrix" | "pipeline" | "help"
        )
    });

    if is_subcommand {
        Cli::parse()
    } else {
        let mut args = raw_args;
        args.insert(1, "run".to_string());
        Cli::parse_from(args)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = parse_cli();

    match cli.command {
        Command::Run(args) => {
            init_logger(args.verbose);
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
        Command::Repl(args) => {
            init_logger(args.verbose);
            let images_dir = args
                .images_dir
                .map(|p| hick_literate::expand_tilde(&p))
                .unwrap_or_else(default_images_dir);
            hick_literate::repl::run_repl(images_dir, args.output).await
        }
        Command::Up(args) => {
            init_logger(args.verbose);
            let key_path = hick_literate::expand_tilde(&args.key_file);
            let secrets_dir = hick_literate::expand_tilde(&args.secrets_dir);
            let _secrets_provider = hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir);

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
                .unwrap_or_else(default_images_dir);

            let mut params: Vec<(String, String)> = args.params.clone();
            if let Some(ref features) = args.features {
                params.retain(|(pk, _)| pk != "features");
                params.push(("features".to_string(), features.clone()));
            }

            hick_literate::watch::run_watch(&args.files, &config, &params, args.dry_run, images_dir).await
        }
        Command::Init(args) => {
            let project_dir = args
                .project_dir
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let config = hick_literate::agents::InitConfig {
                project_dir: project_dir.clone(),
                no_agents: args.no_agents,
            };
            let result = hick_literate::agents::init_agents(&config)?;
            if result.pipeline_initialized {
                println!("Created _hick.yml");
            }
            if result.agents_file_created {
                println!("Created .agents/hick.md");
            }
            if let Some(ref path) = result.reference_added_to {
                println!("Added reference to {}", path.display());
            }
            if !result.pipeline_initialized
                && !result.agents_file_created
                && result.reference_added_to.is_none()
            {
                println!("Nothing to do (already initialized)");
            }
            Ok(())
        }
        Command::Validate(args) => {
            let project_dir = args
                .project_dir
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let config = hick_literate::agents::ValidateConfig {
                project_dir,
                fix: args.fix,
            };
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
                bail!("Validation failed with {} issue(s)", issues.len())
            }
        }
        Command::GenerateMatrix(args) => {
            init_logger(args.verbose);
            hick_literate::generate_matrix::run(args).await
        }
        Command::Pipeline(args) => {
            let project_dir = match &args.command {
                PipelineCommand::Show(a) => a
                    .project_dir
                    .clone()
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()),
                PipelineCommand::Status(a) => a
                    .project_dir
                    .clone()
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default()),
            };
            match args.command {
                PipelineCommand::Show(_) => hick_literate::pipeline::pipeline_show(&project_dir),
                PipelineCommand::Status(_) => hick_literate::pipeline::pipeline_status(&project_dir),
            }
        }
    }
}

fn init_logger(verbose: bool) {
    env_logger::Builder::from_default_env()
        .filter_level(if verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        })
        .init();
}

fn default_images_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("crates/c2w/converted_images"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("c2w/converted_images"),
    ];
    for path in &candidates {
        if path.is_dir() {
            return path.clone();
        }
    }
    candidates[0].clone()
}
