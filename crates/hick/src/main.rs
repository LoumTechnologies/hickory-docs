//! Hick: Secure container orchestration shell.
//!
//! Usage:
//!   `hick [file.hick ...] [--config _hick.yml]` — single-shot pipeline run
//!   `hick up <file.hick> [file2.hick ...]`       — watch mode with merge-aware snapshots
//!
//! Parses `.hick` files, builds an information-flow DAG, validates it,
//! and executes containers with capability-based security.

mod generate_matrix;
mod output_cleanup;

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use log::{debug, info};

use hick::config::{HickConfig, find_config};
use hick::store_config::{StoreBackend, StoreConfig};

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

    /// Initialize AI agent instructions for the project.
    ///
    /// Creates `.agents/hick.md` with embedded instructions and wires a reference
    /// to CLAUDE.md or AGENTS.md if present.
    Init(InitArgs),

    /// Validate AI agent configuration.
    ///
    /// Checks that `.agents/hick.md` exists, content matches, and references are wired.
    Validate(ValidateArgs),

    /// Generate all combinations of features to separate folders.
    ///
    /// Creates a matrix of all feature combinations and generates each to a
    /// separate output directory. Optionally runs verification commands.
    GenerateMatrix(generate_matrix::GenerateMatrixArgs),
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

    /// Enable features (comma-separated list).
    #[arg(long)]
    features: Option<String>,

    /// Output directory for generated files (overrides config `output-dir`).
    #[arg(long)]
    output_dir: Option<PathBuf>,

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

    /// Set a variable: --param key=value
    #[arg(long = "param", value_parser = parse_param)]
    params: Vec<(String, String)>,

    /// Enable features (comma-separated list).
    #[arg(long)]
    features: Option<String>,

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

#[derive(clap::Args)]
struct InitArgs {
    /// Project directory (defaults to current directory).
    #[arg(long)]
    project_dir: Option<PathBuf>,

    /// Skip creating agent instruction files.
    #[arg(long)]
    no_agents: bool,
}

#[derive(clap::Args)]
struct ValidateArgs {
    /// Project directory (defaults to current directory).
    #[arg(long)]
    project_dir: Option<PathBuf>,

    /// Automatically fix validation issues.
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
    let first_positional = raw_args.iter().skip(1).find(|a| !a.starts_with('-'));

    let is_subcommand =
        first_positional.is_some_and(|a| matches!(a.as_str(), "run" | "up" | "repl" | "init" | "validate" | "generate-matrix" | "help"));

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
            let _secrets_provider = hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir);

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

            // Build params from --param and --features flags
            let mut params: Vec<(String, String)> = args.params.clone();
            if let Some(ref features) = args.features {
                params.retain(|(pk, _)| pk != "features");
                params.push(("features".to_string(), features.clone()));
            }

            hick::watch::run_watch(&args.files, &config, &params, args.dry_run, images_dir).await
        }
        Command::Init(args) => {
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
        Command::Validate(args) => {
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
                bail!("Validation failed with {} issue(s)", issues.len())
            }
        }
        Command::GenerateMatrix(args) => {
            init_logger(args.verbose);
            generate_matrix::run(args).await
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

/// Expand a CLI argument into a list of `.hick` files.
///
/// - If `path` is a file, return it directly.
/// - If `path` is a directory with `_hick.yml`, use that config to resolve files.
/// - If `path` is a directory without config, expand `**/*.hick`.
pub(crate) fn expand_path_arg(path: &Path) -> Result<(Vec<PathBuf>, Option<PathBuf>)> {
    if path.is_file() {
        return Ok((vec![path.to_path_buf()], None));
    }

    if path.is_dir() {
        let config_path = path.join("_hick.yml");
        if config_path.is_file() {
            let config = HickConfig::load(&config_path)?;
            let files = config.resolve_files(path)?;
            return Ok((files, Some(config_path)));
        }

        // No config file: expand **/*.hick
        let pattern = path.join("**/*.hick");
        let pattern_str = pattern.to_string_lossy();
        let mut matches: Vec<PathBuf> = glob::glob(&pattern_str)
            .with_context(|| format!("invalid glob pattern: {}", pattern_str))?
            .filter_map(|r| r.ok())
            .collect();
        matches.sort();
        if matches.is_empty() {
            bail!(
                "No .hick files found in '{}'\n\n\
                 To fix this, either:\n  \
                 1. Create a _hick.yml config file listing your .hick files\n  \
                 2. Add .hick files to the directory\n  \
                 3. Specify files directly: hick run file1.hick file2.hick",
                path.display()
            );
        }
        return Ok((matches, None));
    }

    bail!(
        "Path '{}' does not exist\n\n\
         Check that the path is correct and try again.\n\
         Use 'hick --help' to see usage information.",
        path.display()
    );
}

/// Run the pipeline once and write outputs.
async fn run_single_shot(cli: RunArgs) -> Result<()> {
    let pipeline_start = Instant::now();

    // Expand CLI file arguments (supporting directories)
    let (expanded_files, dir_config) = if !cli.files.is_empty() {
        let mut all_files = Vec::new();
        let mut found_config = None;
        for path in &cli.files {
            let (files, cfg) = expand_path_arg(path)?;
            all_files.extend(files);
            if cfg.is_some() {
                found_config = cfg;
            }
        }
        (all_files, found_config)
    } else {
        (Vec::new(), None)
    };

    // Discover config file: CLI --config > directory config > auto-discover
    let config_path = cli
        .config
        .clone()
        .or(dir_config)
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

    // Resolve files: expanded CLI args take precedence, then config
    let files = if !expanded_files.is_empty() {
        expanded_files
    } else {
        config.resolve_files(config_dir)?
    };

    if files.is_empty() {
        bail!(
            "No .hick files specified\n\n\
             Usage:\n  \
             hick run <file.hick>         Run a single file\n  \
             hick run <directory>         Run all .hick files in directory\n  \
             hick run                     Use _hick.yml config if present\n\n\
             Create a _hick.yml file to define default files and variables."
        );
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

    // Add features as a special param
    if let Some(ref features) = cli.features {
        params.retain(|(pk, _)| pk != "features");
        params.push(("features".to_string(), features.clone()));
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

    // Resolve output directory: CLI flag > config value > current directory
    let output_dir = cli
        .output_dir
        .as_ref()
        .cloned()
        .or_else(|| config.output_dir.as_ref().map(|d| config_dir.join(d)))
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
        hick::run_pipeline_live(&sources, &pipeline_config, &params, cc.as_ref(), None).await?
    };

    // Write file outputs to disk (prepend output-dir) and track expected paths
    let mut expected_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    let mut files_written = 0usize;
    let mut files_unchanged = 0usize;

    for (path, content) in &result.files {
        let full_path = if output_dir == Path::new(".") {
            PathBuf::from(path)
        } else {
            // Track the relative path for cleanup
            expected_paths.insert(PathBuf::from(path));
            output_dir.join(path)
        };

        // Check if file already exists with same content
        let needs_write = if full_path.is_file() {
            match std::fs::read_to_string(&full_path) {
                Ok(existing) => existing != *content,
                Err(_) => true, // Can't read, so write it
            }
        } else {
            true // File doesn't exist
        };

        if needs_write {
            info!("Writing output: {}", full_path.display());
            if let Some(parent) = full_path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, content)?;
            debug!("  {} bytes written", content.len());
            files_written += 1;
        } else {
            debug!("Unchanged: {}", full_path.display());
            files_unchanged += 1;
        }
    }

    // Clean up stale files that are no longer in the output set
    if output_dir != Path::new(".") {
        output_cleanup::clean_stale_outputs(&output_dir, &expected_paths)?;
    }

    if result.files.is_empty() {
        info!("No file outputs produced");
    }

    // Print summary to stderr so it's always visible.
    let wall_time = pipeline_start.elapsed();
    let num_containers = result.resource_stats.len();
    let num_files = result.files.len();
    let total_output_bytes: usize = result.files.values().map(|c| c.len()).sum();

    let total_boot: std::time::Duration = result
        .resource_stats
        .values()
        .map(|s| s.boot_duration)
        .sum();
    let total_exec: std::time::Duration = result
        .resource_stats
        .values()
        .map(|s| s.total_exec_duration)
        .sum();
    let total_commands: usize = result
        .resource_stats
        .values()
        .map(|s| s.command_durations.len())
        .sum();

    let pool_info = if result.pool_hits > 0 {
        format!(" ({} from pool)", result.pool_hits)
    } else {
        String::new()
    };

    // When pool was used, show only the boot time actually paid by this pipeline
    let pipeline_boot = total_boot.saturating_sub(result.pool_boot_time_saved);

    // Show file write stats: "3 files (2 written, 1 unchanged)"
    let file_stats = if files_unchanged > 0 {
        format!(
            "{} file{} ({} written, {} unchanged)",
            num_files,
            if num_files == 1 { "" } else { "s" },
            files_written,
            files_unchanged
        )
    } else {
        format!(
            "{} file{} written",
            files_written,
            if files_written == 1 { "" } else { "s" }
        )
    };

    eprintln!(
        "Done: {} ({} bytes), {} container{}{}, {} command{} — {} total (boot {}, exec {})",
        file_stats,
        total_output_bytes,
        num_containers,
        if num_containers == 1 { "" } else { "s" },
        pool_info,
        total_commands,
        if total_commands == 1 { "" } else { "s" },
        hick::fmt_duration(wall_time),
        hick::fmt_duration(pipeline_boot),
        hick::fmt_duration(total_exec),
    );

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
