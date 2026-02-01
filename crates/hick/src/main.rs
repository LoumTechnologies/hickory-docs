//! Hick: Secure container orchestration shell.
//!
//! Usage:
//!   `hick [file.hick ...] [--config _hick.yml]` — single-shot pipeline run
//!   `hick up <file.hick> [file2.hick ...]`       — watch mode with merge-aware snapshots
//!
//! Parses `.hick` files, builds an information-flow DAG, validates it,
//! and executes containers with capability-based security.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use log::{debug, info};

use hick::config::{HickConfig, find_config};
use hick::store_config::{StoreConfig, StoreBackend};

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
    ///
    /// This is also invoked when no subcommand is given.
    Run(RunArgs),

    /// Watch mode: re-run pipeline on file changes with merge-aware snapshots.
    Up(UpArgs),

    /// Interactive REPL: explore containers interactively and record a session transcript.
    Repl(ReplArgs),
}

#[derive(clap::Args)]
struct RunArgs {
    /// `.hick` files to process (overrides `_hick.yml` files list).
    files: Vec<PathBuf>,

    /// Path to config file (default: auto-discover `_hick.yml`).
    #[arg(long)]
    config: Option<PathBuf>,

    /// Age identity file for decrypting secrets.
    #[arg(long)]
    key_file: Option<PathBuf>,

    /// Directory containing age-encrypted secret files.
    #[arg(long)]
    secrets_dir: Option<PathBuf>,

    /// Directory containing pre-converted `.wasm` container images.
    #[arg(long)]
    images_dir: Option<PathBuf>,

    /// Set a variable: --param key=value
    #[arg(long = "param", value_parser = parse_param)]
    params: Vec<(String, String)>,

    /// Dry-run mode: produce placeholder strings instead of running containers.
    #[arg(long)]
    dry_run: bool,

    /// Cache execution results; reuse on subsequent runs when inputs match.
    #[arg(long)]
    cache: bool,

    /// Freeze mode: require all exec results to be cached, never re-execute.
    #[arg(long)]
    freeze: bool,

    /// Clear the cache directory before running.
    #[arg(long)]
    clear_cache: bool,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,
}

fn parse_param(s: &str) -> Result<(String, String), String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| format!("invalid param format '{s}', expected key=value"))?;
    Ok((key.to_string(), value.to_string()))
}

#[derive(clap::Args)]
struct UpArgs {
    /// One or more `.hick` files to watch and process.
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Age identity file for decrypting secrets.
    #[arg(long, default_value = "~/.config/hick/key.txt")]
    key_file: PathBuf,

    /// Directory containing age-encrypted secret files.
    #[arg(long, default_value = "~/.config/hick/secrets")]
    secrets_dir: PathBuf,

    /// Directory containing pre-converted `.wasm` container images.
    #[arg(long)]
    images_dir: Option<PathBuf>,

    /// Dry-run mode: produce placeholder strings instead of running containers.
    #[arg(long)]
    dry_run: bool,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,

    /// Store backend: auto, git, or builtin.
    #[arg(long, default_value = "auto")]
    store: String,

    /// LLM merge API URL for intelligent conflict resolution.
    #[arg(long)]
    merge_api: Option<String>,

    /// Branch name for snapshot tracking.
    #[arg(long, default_value = "main")]
    branch: String,

    /// Maximum number of multi-stage pipeline passes.
    #[arg(long, default_value = "3")]
    max_stages: usize,
}

#[derive(clap::Args)]
struct ReplArgs {
    /// Directory containing pre-converted `.wasm` container images.
    #[arg(long)]
    images_dir: Option<PathBuf>,

    /// Output transcript file (default: session.hick).
    #[arg(long, default_value = "session.hick")]
    output: PathBuf,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

/// Parse CLI, allowing bare `hick <files>` as sugar for `hick run <files>`.
///
/// When invoked with no arguments at all, enters REPL mode.
fn parse_cli() -> Cli {
    let raw_args: Vec<String> = std::env::args().collect();

    // No arguments at all → REPL mode
    if raw_args.len() == 1 {
        return Cli {
            command: Command::Repl(ReplArgs {
                images_dir: None,
                output: PathBuf::from("session.hick"),
                verbose: false,
            }),
        };
    }

    // Check if the first non-flag argument is a known subcommand
    let first_positional = raw_args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with('-'));

    let is_subcommand = first_positional
        .is_some_and(|a| matches!(a.as_str(), "run" | "up" | "repl" | "help"));

    if is_subcommand {
        Cli::parse()
    } else {
        // Inject "run" as the subcommand
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
            run_single_shot(args).await
        }
        Command::Repl(args) => {
            init_logger(args.verbose);

            let images_dir = args
                .images_dir
                .map(|p| hick::expand_tilde(&p))
                .unwrap_or_else(default_images_dir);

            hick::repl::run_repl(images_dir, args.output).await
        }
        Command::Up(args) => {
            init_logger(args.verbose);

            let key_path = hick::expand_tilde(&args.key_file);
            let secrets_dir = hick::expand_tilde(&args.secrets_dir);
            let _secrets_provider =
                hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir);

            let mut config = StoreConfig::load(&std::env::current_dir()?);

            // CLI flags override config file
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
                .unwrap_or_else(default_images_dir);

            hick::watch::run_watch(&args.files, &config, args.dry_run, images_dir).await
        }
    }
}

fn init_logger(verbose: bool) {
    env_logger::Builder::from_default_env()
        .filter_level(if verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
}

/// Run the pipeline once and write outputs.
async fn run_single_shot(cli: RunArgs) -> Result<()> {
    // Discover config file
    let config_path = cli
        .config
        .clone()
        .or_else(|| find_config(&std::env::current_dir().unwrap_or_default()));

    let config = if let Some(ref path) = config_path {
        info!("Using config: {}", path.display());
        HickConfig::load(path)?
    } else {
        HickConfig::default()
    };

    let config_dir = config_path
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or(Path::new("."));

    // Resolve files: CLI args take precedence, then config
    let files = if !cli.files.is_empty() {
        cli.files.clone()
    } else {
        config.resolve_files(config_dir)?
    };

    if files.is_empty() {
        bail!("No .hick files specified. Provide files as arguments or create a _hick.yml.");
    }

    // Merge config vars with CLI params (CLI wins)
    let mut params: Vec<(String, String)> = config
        .vars
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (k, v) in &cli.params {
        params.retain(|(pk, _)| pk != k);
        params.push((k.clone(), v.clone()));
    }

    // Resolve paths: CLI flag > config value > default
    let key_path = cli
        .key_file
        .as_ref()
        .cloned()
        .or_else(|| config.secrets.key_file.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("~/.config/hick/key.txt"));
    let key_path = hick::expand_tilde(&key_path);

    let secrets_dir = cli
        .secrets_dir
        .as_ref()
        .cloned()
        .or_else(|| config.secrets.secrets_dir.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("~/.config/hick/secrets"));
    let secrets_dir = hick::expand_tilde(&secrets_dir);

    let output_dir = config
        .output_dir
        .as_ref()
        .map(|d| config_dir.join(d))
        .unwrap_or_else(|| PathBuf::from("."));

    // Set up secrets provider
    let _secrets_provider = hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir);

    // Read all input files
    let mut file_contents = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        file_contents.push((path.display().to_string(), source));
    }

    // Build sources slice for the pipeline
    let sources: Vec<(&str, &str)> = file_contents
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();

    // Set up cache config
    let cache_enabled = cli.cache || cli.freeze;
    let cc = if cache_enabled {
        let project_dir = config_dir.to_path_buf();
        let cc = hick::cache::CacheConfig::new(&project_dir, true, cli.freeze);
        if cli.clear_cache {
            hick::cache::cache_clear(&cc)?;
            info!("Cache cleared");
        }
        Some(cc)
    } else if cli.clear_cache {
        let project_dir = config_dir.to_path_buf();
        let cc = hick::cache::CacheConfig::new(&project_dir, false, false);
        hick::cache::cache_clear(&cc)?;
        info!("Cache cleared");
        None
    } else {
        None
    };

    // Run the pipeline
    let result = if cli.dry_run {
        hick::run_pipeline(&sources, &params).await?
    } else {
        let images_dir = cli
            .images_dir
            .map(|p| hick::expand_tilde(&p))
            .or_else(|| {
                config
                    .defaults
                    .images_dir
                    .as_ref()
                    .map(|d| hick::expand_tilde(Path::new(d)))
            })
            .unwrap_or_else(default_images_dir);

        let pipeline_config = hick::PipelineConfig {
            images_dir,
            working_dir: None,
        };
        hick::run_pipeline_live(&sources, &pipeline_config, &params, cc.as_ref()).await?
    };

    // Write file outputs to disk (prepend output-dir)
    for (path, content) in &result.files {
        let full_path = if output_dir == Path::new(".") {
            PathBuf::from(path)
        } else {
            output_dir.join(path)
        };
        info!("Writing output: {}", full_path.display());
        if let Some(parent) = full_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, content)?;
        debug!("  {} bytes written", content.len());
    }

    if result.files.is_empty() {
        info!("No file outputs produced");
    }

    info!("Done");
    Ok(())
}

/// Default images directory: `crates/c2w/converted_images` relative to the
/// workspace root (detected via `CARGO_MANIFEST_DIR` at build time, then
/// walking up to the workspace).
fn default_images_dir() -> PathBuf {
    // At runtime, try relative to current dir first
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

    // Fall back to the first candidate (will produce a clear error downstream)
    candidates[0].clone()
}
