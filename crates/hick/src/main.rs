//! Hick: Secure container orchestration shell.
//!
//! Usage:
//!   `hick [file.hick ...] [--config _hick.yml]` — single-shot pipeline run
//!   `hick up <file.hick> [file2.hick ...]`       — watch mode with merge-aware snapshots
//!
//! Parses `.hick` files, builds an information-flow DAG, validates it,
//! and executes containers with capability-based security.

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
    GenerateMatrix(GenerateMatrixArgs),
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

#[derive(clap::Args)]
struct GenerateMatrixArgs {
    /// `.hick` files or directory to process.
    files: Vec<PathBuf>,

    /// Path to config file (default: auto-discover `_hick.yml`).
    #[arg(long)]
    config: Option<PathBuf>,

    /// Set a variable: --param key=value
    #[arg(long = "param", value_parser = parse_param)]
    params: Vec<(String, String)>,

    /// Output base directory for generated feature combinations.
    /// Each combination gets a subdirectory named after its features.
    #[arg(long, default_value = "matrix")]
    output_dir: PathBuf,

    /// Only include these features (comma-separated).
    /// If not specified, uses all leaf features (non-meta features).
    #[arg(long)]
    include_features: Option<String>,

    /// Exclude these features from combinations (comma-separated).
    #[arg(long)]
    exclude_features: Option<String>,

    /// Commands to run for verification after each combination.
    /// Commands run in the output directory. Use {{features}} for the
    /// comma-separated list of enabled features, {{dir}} for the directory.
    /// Prefix with [feature] to run only when feature is enabled.
    /// Use [f1,f2] for OR (any), [f1+f2] for AND (all).
    #[arg(long)]
    verify: Vec<String>,

    /// Timeout for verification commands in seconds.
    #[arg(long, default_value = "300")]
    verify_timeout: u64,

    /// Continue on verification failure instead of stopping.
    #[arg(long)]
    continue_on_error: bool,

    /// Only generate combinations that include at least one of these features.
    /// Useful to avoid generating the empty combination.
    #[arg(long)]
    require_any: Option<String>,

    /// Maximum number of features in a combination (0 = unlimited).
    #[arg(long, default_value = "0")]
    max_combination_size: usize,

    /// Generate only single-feature combinations (shortcut for --max-combination-size 1).
    #[arg(long)]
    singles_only: bool,

    /// Dry run: show what would be generated without actually generating.
    #[arg(long)]
    dry_run: bool,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,

    /// Number of parallel jobs (0 = auto-detect from CPU count).
    #[arg(short = 'j', long, default_value = "0")]
    jobs: usize,
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
            run_generate_matrix(args).await
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
fn expand_path_arg(path: &Path) -> Result<(Vec<PathBuf>, Option<PathBuf>)> {
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
        clean_stale_outputs(&output_dir, &expected_paths)?;
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

// ---------------------------------------------------------------------------
// Verify Command Parsing
// ---------------------------------------------------------------------------

/// A verify command with optional feature condition.
struct VerifyCommand {
    command: String,
    condition: VerifyCondition,
}

/// Condition for when a verify command should run.
enum VerifyCondition {
    /// Always run this command.
    Always,
    /// Run if ANY of these features are present (OR logic).
    AnyOf(Vec<String>),
    /// Run if ALL of these features are present (AND logic).
    AllOf(Vec<String>),
}

impl VerifyCommand {
    /// Parse "[condition] command" syntax.
    ///
    /// Examples:
    /// - `"cargo build"` → Always run
    /// - `"[rust-frontend-lib] cargo build"` → Run if rust-frontend-lib enabled
    /// - `"[spa,extension] npm test"` → Run if spa OR extension enabled (comma = OR)
    /// - `"[spa|extension] npm test"` → Run if spa OR extension enabled (pipe = OR)
    /// - `"[spa+backend] npm run e2e"` → Run if spa AND backend enabled
    fn parse(input: &str) -> Self {
        let input = input.trim();
        if input.starts_with('[') {
            if let Some(end) = input.find(']') {
                let condition_str = &input[1..end];
                let command = input[end + 1..].trim().to_string();

                if condition_str.contains('+') {
                    // AND logic
                    let features: Vec<String> = condition_str
                        .split('+')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    return Self {
                        command,
                        condition: VerifyCondition::AllOf(features),
                    };
                } else {
                    // OR logic (comma or pipe separated)
                    let features: Vec<String> = condition_str
                        .split(|c| c == ',' || c == '|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    return Self {
                        command,
                        condition: VerifyCondition::AnyOf(features),
                    };
                }
            }
        }
        Self {
            command: input.to_string(),
            condition: VerifyCondition::Always,
        }
    }

    /// Check if this command should run for the given feature combination.
    fn should_run(&self, enabled_features: &[String]) -> bool {
        match &self.condition {
            VerifyCondition::Always => true,
            VerifyCondition::AnyOf(required) => {
                required.iter().any(|f| enabled_features.contains(f))
            }
            VerifyCondition::AllOf(required) => {
                required.iter().all(|f| enabled_features.contains(f))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Generate Matrix
// ---------------------------------------------------------------------------

/// Data for tracking timing of each combination (for linear regression).
#[derive(Clone)]
struct ComboTiming {
    features: Vec<String>,
    duration_secs: f64,
}

/// Simple linear regression to identify which features contribute most to runtime.
/// Uses ordinary least squares with feature indicator variables.
fn analyze_feature_timing(
    all_features: &[String],
    timings: &[ComboTiming],
) -> Vec<(String, f64)> {
    if timings.is_empty() || all_features.is_empty() {
        return vec![];
    }

    let n = timings.len();
    let m = all_features.len();

    // Build design matrix X (n x m+1) with intercept column
    // X[i][0] = 1 (intercept)
    // X[i][j+1] = 1 if feature j is present in combo i, else 0
    let mut x: Vec<Vec<f64>> = vec![vec![0.0; m + 1]; n];
    let mut y: Vec<f64> = vec![0.0; n];

    for (i, timing) in timings.iter().enumerate() {
        x[i][0] = 1.0; // intercept
        for (j, feat) in all_features.iter().enumerate() {
            if timing.features.contains(feat) {
                x[i][j + 1] = 1.0;
            }
        }
        y[i] = timing.duration_secs;
    }

    // Compute X^T * X
    let mut xtx: Vec<Vec<f64>> = vec![vec![0.0; m + 1]; m + 1];
    for i in 0..=m {
        for j in 0..=m {
            for k in 0..n {
                xtx[i][j] += x[k][i] * x[k][j];
            }
        }
    }

    // Compute X^T * y
    let mut xty: Vec<f64> = vec![0.0; m + 1];
    for i in 0..=m {
        for k in 0..n {
            xty[i] += x[k][i] * y[k];
        }
    }

    // Solve (X^T * X) * beta = X^T * y using Gaussian elimination
    // Augment matrix [xtx | xty]
    let mut aug: Vec<Vec<f64>> = vec![vec![0.0; m + 2]; m + 1];
    for i in 0..=m {
        for j in 0..=m {
            aug[i][j] = xtx[i][j];
        }
        aug[i][m + 1] = xty[i];
    }

    // Forward elimination with partial pivoting
    for col in 0..=m {
        // Find pivot
        let mut max_row = col;
        for row in (col + 1)..=m {
            if aug[row][col].abs() > aug[max_row][col].abs() {
                max_row = row;
            }
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        if pivot.abs() < 1e-10 {
            continue; // Skip singular column
        }

        // Eliminate below
        for row in (col + 1)..=m {
            let factor = aug[row][col] / pivot;
            for j in col..=(m + 1) {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Back substitution
    let mut beta: Vec<f64> = vec![0.0; m + 1];
    for i in (0..=m).rev() {
        let mut sum = aug[i][m + 1];
        for j in (i + 1)..=m {
            sum -= aug[i][j] * beta[j];
        }
        if aug[i][i].abs() > 1e-10 {
            beta[i] = sum / aug[i][i];
        }
    }

    // Return feature coefficients (skip intercept at index 0)
    let mut results: Vec<(String, f64)> = all_features
        .iter()
        .enumerate()
        .map(|(i, feat)| (feat.clone(), beta[i + 1]))
        .collect();

    // Sort by coefficient (descending) to show slowest features first
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Shared state for real-time progress display.
struct MatrixProgress {
    completed: std::sync::atomic::AtomicUsize,
    total: usize,
    failed: std::sync::atomic::AtomicUsize,
    cumulative_ms: std::sync::atomic::AtomicU64,
    timings: tokio::sync::Mutex<Vec<ComboTiming>>,
    all_features: Vec<String>,
    current_combo: tokio::sync::Mutex<String>,
}

impl MatrixProgress {
    fn new(total: usize, all_features: Vec<String>) -> Self {
        Self {
            completed: std::sync::atomic::AtomicUsize::new(0),
            total,
            failed: std::sync::atomic::AtomicUsize::new(0),
            cumulative_ms: std::sync::atomic::AtomicU64::new(0),
            timings: tokio::sync::Mutex::new(Vec::new()),
            all_features,
            current_combo: tokio::sync::Mutex::new(String::new()),
        }
    }

    async fn record_success(&self, combo: &[String], duration: std::time::Duration) {
        use std::sync::atomic::Ordering;
        self.completed.fetch_add(1, Ordering::Relaxed);
        self.cumulative_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        self.timings.lock().await.push(ComboTiming {
            features: combo.to_vec(),
            duration_secs: duration.as_secs_f64(),
        });
    }

    fn record_failure(&self) {
        use std::sync::atomic::Ordering;
        self.completed.fetch_add(1, Ordering::Relaxed);
        self.failed.fetch_add(1, Ordering::Relaxed);
    }

    async fn set_current(&self, name: &str) {
        *self.current_combo.lock().await = name.to_string();
    }

    async fn get_display_state(&self) -> (usize, usize, usize, u64, String, Vec<ComboTiming>) {
        use std::sync::atomic::Ordering;
        let completed = self.completed.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let cumulative_ms = self.cumulative_ms.load(Ordering::Relaxed);
        let current = self.current_combo.lock().await.clone();
        let timings = self.timings.lock().await.clone();
        (completed, self.total, failed, cumulative_ms, current, timings)
    }
}

/// Format bytes as human-readable size.
fn fmt_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Render the real-time progress display.
fn render_progress(
    completed: usize,
    total: usize,
    failed: usize,
    cumulative_ms: u64,
    current: &str,
    timings: &[ComboTiming],
    all_features: &[String],
    cpu_usage: f32,
    mem_bytes: u64,
    is_tty: bool,
) {
    use std::io::Write;
    use std::time::Duration;

    if total == 0 {
        return;
    }

    let pct = (completed as f64 / total as f64) * 100.0;
    let avg_ms = if completed > 0 {
        cumulative_ms / completed as u64
    } else {
        0
    };
    let remaining = total.saturating_sub(completed);
    let eta_ms = avg_ms * remaining as u64;
    let eta = hick::fmt_duration(Duration::from_millis(eta_ms));

    // Progress bar (20 chars wide)
    let bar_width = 20;
    let filled = ((pct / 100.0) * bar_width as f64) as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

    // Build the status line
    let status = if failed > 0 {
        format!(
            "\x1b[2K\rMatrix: {:>5.1}% {} {}/{} ({} failed)  cpu:{:>3.0}%  mem:{}  eta:{}  {}",
            pct,
            bar,
            completed,
            total,
            failed,
            cpu_usage,
            fmt_bytes(mem_bytes),
            eta,
            if current.len() > 20 {
                format!("{}...", &current[..17])
            } else {
                current.to_string()
            }
        )
    } else {
        format!(
            "\x1b[2K\rMatrix: {:>5.1}% {} {}/{}  cpu:{:>3.0}%  mem:{}  eta:{}  {}",
            pct,
            bar,
            completed,
            total,
            cpu_usage,
            fmt_bytes(mem_bytes),
            eta,
            if current.len() > 20 {
                format!("{}...", &current[..17])
            } else {
                current.to_string()
            }
        )
    };

    // Feature impact line (only if we have enough data)
    let impact_line = if timings.len() >= all_features.len().max(3) && !all_features.is_empty() {
        let impacts = analyze_feature_timing(all_features, timings);
        let top_impacts: Vec<String> = impacts
            .iter()
            .take(3)
            .filter(|(_, coef)| *coef > 0.01) // Only show significant impacts
            .map(|(feat, coef)| {
                format!(
                    "{} +{}",
                    if feat.len() > 12 {
                        format!("{}…", &feat[..11])
                    } else {
                        feat.clone()
                    },
                    hick::fmt_duration(Duration::from_secs_f64(*coef))
                )
            })
            .collect();
        if !top_impacts.is_empty() {
            format!("\n\x1b[2K  slowest: {}", top_impacts.join(", "))
        } else {
            String::new()
        }
    } else if !timings.is_empty() && timings.len() < all_features.len().max(3) {
        format!(
            "\n\x1b[2K  gathering data... ({}/{})",
            timings.len(),
            all_features.len().max(3)
        )
    } else {
        String::new()
    };

    if is_tty {
        // Move cursor up if we have a second line
        let line_count = if impact_line.is_empty() { 1 } else { 2 };
        if line_count == 2 {
            print!("\x1b[2A"); // Move up 2 lines
        } else {
            print!("\x1b[1A"); // Move up 1 line
        }
        print!("{}{}", status, impact_line);
        if line_count == 2 {
            println!(); // Ensure we're on the line after the second line
        }
        println!();
        let _ = std::io::stdout().flush();
    } else {
        // Non-TTY: just print completion updates
        if completed == total || completed % 10 == 0 {
            println!(
                "Matrix: {:.1}% {}/{}  eta:{}",
                pct, completed, total, eta
            );
        }
    }
}

/// Run the generate-matrix command: generate all feature combinations.
async fn run_generate_matrix(cli: GenerateMatrixArgs) -> Result<()> {
    use futures::stream::{self, StreamExt};
    use std::collections::HashSet;
    use std::io::IsTerminal;
    use std::process::Command as ProcessCommand;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use sysinfo::System;

    let start_time = Instant::now();
    let is_tty = std::io::stdout().is_terminal();

    // Suppress INFO logs during matrix generation unless --verbose
    // This prevents noisy DAG validation messages from cluttering the UI
    if !cli.verbose {
        log::set_max_level(log::LevelFilter::Warn);
    }

    // Expand CLI file arguments
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

    // Discover config file
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

    // Resolve files
    let files = if !expanded_files.is_empty() {
        expanded_files
    } else {
        config.resolve_files(config_dir)?
    };

    if files.is_empty() {
        bail!("No .hick files specified");
    }

    // Read all input files
    let mut file_contents = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        file_contents.push((path.display().to_string(), source));
    }

    // Build sources slice for feature extraction
    let sources: Vec<(&str, &str)> = file_contents
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();

    // Extract feature definitions from the hick files
    let feature_defs = hick::extract_feature_definitions(&sources)?;

    // Identify leaf features (those that are not meta-features)
    // A leaf feature is one that doesn't have any `requires` attributes
    let leaf_features: Vec<String> = feature_defs
        .iter()
        .filter(|(_, def)| def.requires.is_empty())
        .map(|(name, _)| name.clone())
        .collect();

    let meta_features: Vec<String> = feature_defs
        .iter()
        .filter(|(_, def)| !def.requires.is_empty())
        .map(|(name, _)| name.clone())
        .collect();

    info!(
        "Found {} features: {} leaf, {} meta",
        feature_defs.len(),
        leaf_features.len(),
        meta_features.len()
    );

    // Determine which features to use for combinations
    let mut features_to_combine: Vec<String> = if let Some(ref include) = cli.include_features {
        include
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        leaf_features.clone()
    };

    // Apply exclusions
    if let Some(ref exclude) = cli.exclude_features {
        let exclusions: HashSet<String> = exclude
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        features_to_combine.retain(|f| !exclusions.contains(f));
    }

    // Sort for consistent ordering
    features_to_combine.sort();

    info!("Generating combinations for features: {:?}", features_to_combine);

    // Generate power set of feature combinations
    let max_size = if cli.singles_only {
        1
    } else if cli.max_combination_size > 0 {
        cli.max_combination_size
    } else {
        features_to_combine.len()
    };

    let mut combinations: Vec<Vec<String>> = Vec::new();
    generate_combinations(&features_to_combine, max_size, &mut combinations);

    // Filter out empty combination if require_any is specified
    if let Some(ref require_any) = cli.require_any {
        let required: HashSet<String> = require_any
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        combinations.retain(|combo| combo.iter().any(|f| required.contains(f)));
    } else {
        // By default, skip the empty combination
        combinations.retain(|combo| !combo.is_empty());
    }

    // Filter out combinations with conflicting features
    let pre_filter_count = combinations.len();
    combinations.retain(|combo| {
        // Check for pairwise conflicts
        for (i, a) in combo.iter().enumerate() {
            for b in combo.iter().skip(i + 1) {
                // Check explicit conflicts_with (either direction)
                if let Some(def_a) = feature_defs.get(a) {
                    if def_a.conflicts_with.iter().any(|c| c == b) {
                        return false;
                    }
                }
                if let Some(def_b) = feature_defs.get(b) {
                    if def_b.conflicts_with.iter().any(|c| c == a) {
                        return false;
                    }
                }

                // Check exclusive groups
                if let (Some(def_a), Some(def_b)) = (feature_defs.get(a), feature_defs.get(b)) {
                    if let (Some(group_a), Some(group_b)) =
                        (&def_a.exclusive_group, &def_b.exclusive_group)
                    {
                        if group_a == group_b {
                            return false;
                        }
                    }
                }
            }
        }
        true
    });

    let filtered_count = pre_filter_count - combinations.len();
    if filtered_count > 0 {
        info!(
            "Filtered out {} combinations due to feature conflicts",
            filtered_count
        );
    }

    info!(
        "Generated {} feature combinations (max size: {})",
        combinations.len(),
        max_size
    );

    if cli.dry_run {
        println!("Feature combinations (dry run):\n");
        for combo in &combinations {
            let name = if combo.is_empty() {
                "none".to_string()
            } else {
                combo.join("-")
            };
            println!("  {}/", name);
        }
        println!("\nTotal: {} combinations", combinations.len());
        return Ok(());
    }

    // Create output directory
    std::fs::create_dir_all(&cli.output_dir)
        .with_context(|| format!("failed to create output directory: {}", cli.output_dir.display()))?;

    // Merge config vars with CLI params
    let mut base_params: Vec<(String, String)> = config
        .vars
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (k, v) in &cli.params {
        base_params.retain(|(pk, _)| pk != k);
        base_params.push((k.clone(), v.clone()));
    }

    // Determine parallelism
    let jobs = if cli.jobs == 0 {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    } else {
        cli.jobs
    };

    info!("Running with {} parallel job(s)", jobs);

    // Parse verify commands once before the loop
    let verify_cmds: Vec<VerifyCommand> = cli.verify
        .iter()
        .map(|s| VerifyCommand::parse(s))
        .collect();

    // Wrap shared data in Arc for parallel access
    let file_contents = Arc::new(file_contents);
    let base_params = Arc::new(base_params);
    let output_dir = Arc::new(cli.output_dir.clone());
    let verify_cmds = Arc::new(verify_cmds);
    let continue_on_error = cli.continue_on_error;
    let total_combinations = combinations.len();

    // Flag to signal cancellation on first failure (when !continue_on_error)
    let cancelled = Arc::new(AtomicBool::new(false));

    // Collect failures for final report
    let failures = Arc::new(tokio::sync::Mutex::new(Vec::<(String, String)>::new()));

    // Clone features list for regression analysis
    let all_features_for_analysis: Vec<String> = features_to_combine.clone();

    // Create shared progress state
    let progress = Arc::new(MatrixProgress::new(
        total_combinations,
        all_features_for_analysis.clone(),
    ));

    // Spawn background task to update progress display with CPU/RAM info
    let progress_for_display = progress.clone();
    let display_cancelled = cancelled.clone();
    let display_handle = tokio::spawn(async move {
        let mut sys = System::new();
        let pid = sysinfo::get_current_pid().ok();

        // Print initial blank lines for progress display
        if is_tty {
            println!();
            println!();
        }

        loop {
            // Refresh system info
            sys.refresh_all();

            // Get process-specific CPU and memory
            let (cpu_usage, mem_bytes) = if let Some(pid) = pid {
                if let Some(proc) = sys.process(pid) {
                    (proc.cpu_usage(), proc.memory())
                } else {
                    (0.0, 0)
                }
            } else {
                (0.0, 0)
            };

            let (completed, total, failed, cumulative_ms, current, timings) =
                progress_for_display.get_display_state().await;

            render_progress(
                completed,
                total,
                failed,
                cumulative_ms,
                &current,
                &timings,
                &progress_for_display.all_features,
                cpu_usage,
                mem_bytes,
                is_tty,
            );

            if completed >= total || display_cancelled.load(Ordering::Relaxed) {
                break;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    // Process combinations in parallel
    let results: Vec<Result<String, ()>> = stream::iter(combinations.into_iter().enumerate())
        .map(|(_i, combo)| {
            let file_contents = file_contents.clone();
            let base_params = base_params.clone();
            let output_dir = output_dir.clone();
            let verify_cmds = verify_cmds.clone();
            let cancelled = cancelled.clone();
            let failures = failures.clone();
            let progress = progress.clone();

            async move {
                // Check if we should skip due to earlier failure
                if cancelled.load(Ordering::Relaxed) {
                    return Err(());
                }

                // Start timing this combination
                let combo_start = Instant::now();

                let combo_name = if combo.is_empty() {
                    "none".to_string()
                } else {
                    combo.join("-")
                };

                // Update current combo for display
                progress.set_current(&combo_name).await;

                let combo_dir = output_dir.join(&combo_name);
                let features_str = combo.join(",");

                // Build sources from file_contents
                let sources: Vec<(&str, &str)> = file_contents
                    .iter()
                    .map(|(name, content)| (name.as_str(), content.as_str()))
                    .collect();

                // Build params for this combination
                let mut params: Vec<(String, String)> = base_params
                    .iter()
                    .filter(|(pk, _)| pk != "features")
                    .cloned()
                    .collect();
                if !combo.is_empty() {
                    params.push(("features".to_string(), features_str.clone()));
                }

                // Run the pipeline for this combination
                let result = hick::run_pipeline(&sources, &params).await;
                let combo_elapsed = combo_start.elapsed();

                match result {
                    Ok(pipeline_result) => {
                        // Write outputs
                        if let Err(e) = std::fs::create_dir_all(&combo_dir) {
                            let error_msg = format!("failed to create directory: {}", e);
                            progress.record_failure();
                            failures.lock().await.push((combo_name.clone(), error_msg));
                            if !continue_on_error {
                                cancelled.store(true, Ordering::Relaxed);
                            }
                            return Err(());
                        }

                        for (path, content) in &pipeline_result.files {
                            let full_path = combo_dir.join(path);
                            if let Some(parent) = full_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            if let Err(e) = std::fs::write(&full_path, content) {
                                let error_msg = format!("failed to write {}: {}", path, e);
                                progress.record_failure();
                                failures.lock().await.push((combo_name.clone(), error_msg));
                                if !continue_on_error {
                                    cancelled.store(true, Ordering::Relaxed);
                                }
                                return Err(());
                            }
                        }

                        // Run verification commands
                        for verify in verify_cmds.iter() {
                            // Skip commands whose conditions don't match this combo
                            if !verify.should_run(&combo) {
                                continue;
                            }

                            let cmd_with_features = verify.command.replace("{{features}}", &features_str);
                            let cmd_with_dir =
                                cmd_with_features.replace("{{dir}}", &combo_dir.to_string_lossy());

                            let combo_dir_for_verify = combo_dir.clone();
                            let verify_result = tokio::task::spawn_blocking(move || {
                                ProcessCommand::new("sh")
                                    .arg("-c")
                                    .arg(&cmd_with_dir)
                                    .current_dir(&combo_dir_for_verify)
                                    .output()
                            })
                            .await;

                            match verify_result {
                                Ok(Ok(output)) => {
                                    if !output.status.success() {
                                        let stderr = String::from_utf8_lossy(&output.stderr);
                                        let stdout = String::from_utf8_lossy(&output.stdout);
                                        let error_msg = format!(
                                            "exit code: {:?}\nstdout: {}\nstderr: {}",
                                            output.status.code(),
                                            stdout,
                                            stderr
                                        );
                                        progress.record_failure();
                                        failures.lock().await.push((combo_name.clone(), error_msg));
                                        if !continue_on_error {
                                            cancelled.store(true, Ordering::Relaxed);
                                        }
                                        return Err(());
                                    }
                                }
                                Ok(Err(e)) => {
                                    let error_msg = format!("failed to run command: {}", e);
                                    progress.record_failure();
                                    failures.lock().await.push((combo_name.clone(), error_msg));
                                    if !continue_on_error {
                                        cancelled.store(true, Ordering::Relaxed);
                                    }
                                    return Err(());
                                }
                                Err(e) => {
                                    let error_msg = format!("task join error: {}", e);
                                    progress.record_failure();
                                    failures.lock().await.push((combo_name.clone(), error_msg));
                                    if !continue_on_error {
                                        cancelled.store(true, Ordering::Relaxed);
                                    }
                                    return Err(());
                                }
                            }
                        }

                        // Record success with timing
                        progress.record_success(&combo, combo_elapsed).await;
                        Ok(combo_name)
                    }
                    Err(e) => {
                        let error_msg = format!("{}", e);
                        progress.record_failure();
                        failures.lock().await.push((combo_name, error_msg));
                        if !continue_on_error {
                            cancelled.store(true, Ordering::Relaxed);
                        }
                        Err(())
                    }
                }
            }
        })
        .buffer_unordered(jobs)
        .collect()
        .await;

    // Wait for display task to finish
    let _ = display_handle.await;

    // Count results
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.iter().filter(|r| r.is_err()).count();
    let failures = Arc::try_unwrap(failures)
        .map(|mutex| mutex.into_inner())
        .unwrap_or_else(|arc| {
            // This shouldn't happen since we're the only owner after collect()
            tokio::runtime::Handle::current().block_on(async { arc.lock().await.clone() })
        });

    // Extract timings for analysis from progress state
    let timings_data = Arc::try_unwrap(progress)
        .map(|p| {
            tokio::runtime::Handle::current().block_on(async { p.timings.into_inner() })
        })
        .unwrap_or_else(|arc| {
            tokio::runtime::Handle::current().block_on(async { arc.timings.lock().await.clone() })
        });

    // Clear progress lines and print summary
    let elapsed = start_time.elapsed();
    if is_tty {
        print!("\x1b[2K\r"); // Clear the progress line
    }
    println!("\n--- Summary ---");
    println!("Total combinations: {}", total_combinations);
    println!("Parallel jobs: {}", jobs);
    println!("Successful: {}", successful);
    println!("Failed: {}", failed);
    println!("Total time: {}", hick::fmt_duration(elapsed));

    // Calculate timing statistics
    if !timings_data.is_empty() {
        let total_combo_time: f64 = timings_data.iter().map(|t| t.duration_secs).sum();
        let avg_time = total_combo_time / timings_data.len() as f64;
        let min_time = timings_data
            .iter()
            .map(|t| t.duration_secs)
            .fold(f64::INFINITY, f64::min);
        let max_time = timings_data
            .iter()
            .map(|t| t.duration_secs)
            .fold(f64::NEG_INFINITY, f64::max);

        println!("\n--- Timing Statistics ---");
        println!(
            "Average time per combination: {}",
            hick::fmt_duration(Duration::from_secs_f64(avg_time))
        );
        println!(
            "Min time: {}",
            hick::fmt_duration(Duration::from_secs_f64(min_time))
        );
        println!(
            "Max time: {}",
            hick::fmt_duration(Duration::from_secs_f64(max_time))
        );
        println!(
            "Sum of all iteration times: {}",
            hick::fmt_duration(Duration::from_secs_f64(total_combo_time))
        );

        // Run linear regression to identify slow features
        if !all_features_for_analysis.is_empty() && timings_data.len() >= 2 {
            let feature_impacts = analyze_feature_timing(&all_features_for_analysis, &timings_data);

            if !feature_impacts.is_empty() {
                println!("\n--- Feature Time Impact (Linear Regression) ---");
                println!("Estimated additional time per feature (sorted by impact):");
                for (feat, coef) in &feature_impacts {
                    let sign = if *coef >= 0.0 { "+" } else { "" };
                    println!(
                        "  {:20} {}{}",
                        feat,
                        sign,
                        hick::fmt_duration(Duration::from_secs_f64(coef.abs()))
                    );
                }
                // Show baseline (intercept would be at index 0 in beta, but we don't expose it)
                // The regression already filters and sorts by impact
            }
        }
    }

    if !failures.is_empty() {
        println!("\nFailures:");
        for (name, error) in &failures {
            println!("  {} - {}", name, error.lines().next().unwrap_or("unknown error"));
        }
    }

    if failed > 0 {
        bail!("{} combination(s) failed", failed);
    }

    Ok(())
}

/// Generate all combinations of features up to max_size.
fn generate_combinations(
    features: &[String],
    max_size: usize,
    result: &mut Vec<Vec<String>>,
) {
    let n = features.len();
    // Power set: iterate through all 2^n combinations
    for mask in 0..(1usize << n) {
        let mut combo: Vec<String> = Vec::new();
        for (i, feature) in features.iter().enumerate() {
            if mask & (1 << i) != 0 {
                combo.push(feature.clone());
            }
        }
        if combo.len() <= max_size {
            result.push(combo);
        }
    }
}

/// Remove files and directories from `output_dir` that are not in `expected_paths`.
///
/// This is called after writing output files to clean up stale artifacts from
/// previous runs. Only operates when `output_dir` is a non-trivial path (not ".").
///
/// Only cleans files in directories that hick is actively writing to (i.e.,
/// directories that contain at least one expected file). This prevents deletion
/// of files generated by other tools (like xcodegen) in sibling directories.
///
/// `expected_paths` should contain paths relative to `output_dir`.
fn clean_stale_outputs(output_dir: &Path, expected_paths: &std::collections::HashSet<PathBuf>) -> Result<()> {
    if output_dir == Path::new(".") || !output_dir.exists() {
        return Ok(());
    }

    // Build set of "managed directories" - directories that directly contain expected files
    let managed_dirs: std::collections::HashSet<PathBuf> = expected_paths
        .iter()
        .filter_map(|p| p.parent().map(|parent| parent.to_path_buf()))
        .collect();

    // Collect all files currently in the output directory
    let mut stale_files: Vec<PathBuf> = Vec::new();
    let mut stale_dirs: Vec<PathBuf> = Vec::new();

    collect_files_recursive(output_dir, output_dir, &mut stale_files, expected_paths, &managed_dirs)?;

    // Delete stale files
    for file in &stale_files {
        info!("Removing stale output: {}", file.display());
        if let Err(e) = std::fs::remove_file(file) {
            log::warn!("Failed to remove stale file {}: {}", file.display(), e);
        }
    }

    // Collect empty directories after file deletion
    collect_empty_dirs_recursive(output_dir, &mut stale_dirs)?;

    // Delete empty directories (deepest first)
    stale_dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));
    for dir in &stale_dirs {
        // Double-check it's still empty and within output_dir
        if dir.starts_with(output_dir)
            && dir != output_dir
            && dir.is_dir()
            && std::fs::read_dir(dir).map(|mut d| d.next().is_none()).unwrap_or(false)
        {
            info!("Removing empty directory: {}", dir.display());
            if let Err(e) = std::fs::remove_dir(dir) {
                log::warn!("Failed to remove empty directory {}: {}", dir.display(), e);
            }
        }
    }

    if !stale_files.is_empty() || !stale_dirs.is_empty() {
        info!(
            "Cleaned {} stale file(s) and {} empty director{}",
            stale_files.len(),
            stale_dirs.len(),
            if stale_dirs.len() == 1 { "y" } else { "ies" }
        );
    }

    Ok(())
}

/// Recursively collect files that are not in the expected set.
///
/// Only considers files in "managed directories" - directories that contain
/// at least one expected file. This prevents cleaning files in directories
/// that hick doesn't write to (e.g., xcodegen-generated .xcodeproj).
fn collect_files_recursive(
    dir: &Path,
    base: &Path,
    stale: &mut Vec<PathBuf>,
    expected: &std::collections::HashSet<PathBuf>,
    managed_dirs: &std::collections::HashSet<PathBuf>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    // Get directory path relative to base
    let rel_dir = dir.strip_prefix(base).unwrap_or(Path::new(""));

    // Only process this directory if it's a managed directory
    let is_managed = rel_dir.as_os_str().is_empty() || managed_dirs.contains(rel_dir);

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files_recursive(&path, base, stale, expected, managed_dirs)?;
        } else if path.is_file() && is_managed {
            // Only consider files in managed directories
            if let Ok(rel) = path.strip_prefix(base) {
                if !expected.contains(rel) {
                    stale.push(path);
                }
            }
        }
    }

    Ok(())
}

/// Recursively collect empty directories.
fn collect_empty_dirs_recursive(dir: &Path, empty_dirs: &mut Vec<PathBuf>) -> Result<bool> {
    if !dir.is_dir() {
        return Ok(false);
    }

    let mut has_content = false;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let subdir_empty = collect_empty_dirs_recursive(&path, empty_dirs)?;
            if !subdir_empty {
                has_content = true;
            }
        } else {
            has_content = true;
        }
    }

    if !has_content {
        empty_dirs.push(dir.to_path_buf());
    }

    Ok(!has_content)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_stale_outputs_removes_extra_files() {
        let dir = std::env::temp_dir().join("hick-clean-stale-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Create some files: one expected, one stale
        std::fs::write(dir.join("expected.txt"), "keep").unwrap();
        std::fs::write(dir.join("stale.txt"), "remove").unwrap();

        let mut expected = std::collections::HashSet::new();
        expected.insert(PathBuf::from("expected.txt"));

        clean_stale_outputs(&dir, &expected).unwrap();

        // Expected file should still exist
        assert!(dir.join("expected.txt").exists());
        // Stale file should be removed
        assert!(!dir.join("stale.txt").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clean_stale_outputs_removes_empty_dirs() {
        let dir = std::env::temp_dir().join("hick-clean-dirs-test");
        let _ = std::fs::remove_dir_all(&dir);

        // Create nested structure: output/subdir/file.txt and output/empty/
        std::fs::create_dir_all(dir.join("subdir")).unwrap();
        std::fs::create_dir_all(dir.join("empty")).unwrap();
        std::fs::write(dir.join("subdir").join("file.txt"), "keep").unwrap();

        let mut expected = std::collections::HashSet::new();
        expected.insert(PathBuf::from("subdir/file.txt"));

        clean_stale_outputs(&dir, &expected).unwrap();

        // Subdir and file should still exist
        assert!(dir.join("subdir").join("file.txt").exists());
        // Empty dir should be removed
        assert!(!dir.join("empty").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clean_stale_outputs_only_cleans_managed_dirs() {
        let dir = std::env::temp_dir().join("hick-clean-managed-test");
        let _ = std::fs::remove_dir_all(&dir);

        // Simulate xcodegen scenario:
        // - hick writes to apple/MyProject/ContentView.swift
        // - xcodegen creates apple/MyProject.xcodeproj/project.pbxproj
        // The xcodeproj should NOT be cleaned because hick doesn't write there
        std::fs::create_dir_all(dir.join("apple/MyProject")).unwrap();
        std::fs::create_dir_all(dir.join("apple/MyProject.xcodeproj")).unwrap();
        std::fs::write(dir.join("apple/MyProject/ContentView.swift"), "keep").unwrap();
        std::fs::write(dir.join("apple/MyProject/OldFile.swift"), "stale").unwrap();
        std::fs::write(dir.join("apple/MyProject.xcodeproj/project.pbxproj"), "xcodegen").unwrap();

        let mut expected = std::collections::HashSet::new();
        expected.insert(PathBuf::from("apple/MyProject/ContentView.swift"));

        clean_stale_outputs(&dir, &expected).unwrap();

        // Expected file should exist
        assert!(dir.join("apple/MyProject/ContentView.swift").exists());
        // Stale file in same directory should be removed
        assert!(!dir.join("apple/MyProject/OldFile.swift").exists());
        // xcodegen file should NOT be removed (different directory, not managed by hick)
        assert!(dir.join("apple/MyProject.xcodeproj/project.pbxproj").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clean_stale_outputs_cleans_stale_in_same_dir() {
        let dir = std::env::temp_dir().join("hick-clean-same-dir-test");
        let _ = std::fs::remove_dir_all(&dir);

        // Create: output/src/keep.txt and output/src/stale.txt
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src").join("keep.txt"), "keep").unwrap();
        std::fs::write(dir.join("src").join("stale.txt"), "remove").unwrap();

        let mut expected = std::collections::HashSet::new();
        expected.insert(PathBuf::from("src/keep.txt"));

        clean_stale_outputs(&dir, &expected).unwrap();

        // Keep file should exist
        assert!(dir.join("src").join("keep.txt").exists());
        // Stale file in same directory should be removed
        assert!(!dir.join("src").join("stale.txt").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clean_stale_outputs_noop_for_current_dir() {
        // Should not do anything when output_dir is "."
        let expected = std::collections::HashSet::new();
        clean_stale_outputs(Path::new("."), &expected).unwrap();
        // No assertions needed - just ensure it doesn't panic or delete anything
    }

    #[test]
    fn clean_stale_outputs_noop_for_nonexistent_dir() {
        let dir = std::env::temp_dir().join("hick-clean-nonexistent-test");
        let _ = std::fs::remove_dir_all(&dir);
        // Don't create the directory

        let expected = std::collections::HashSet::new();
        clean_stale_outputs(&dir, &expected).unwrap();
        // Should not panic
    }

    // VerifyCommand tests

    #[test]
    fn verify_command_parse_unconditional() {
        let cmd = VerifyCommand::parse("cargo build");
        assert_eq!(cmd.command, "cargo build");
        assert!(matches!(cmd.condition, VerifyCondition::Always));
        assert!(cmd.should_run(&[]));
        assert!(cmd.should_run(&["spa".to_string()]));
    }

    #[test]
    fn verify_command_parse_single_feature() {
        let cmd = VerifyCommand::parse("[rust-frontend-lib] cargo check");
        assert_eq!(cmd.command, "cargo check");
        assert!(matches!(cmd.condition, VerifyCondition::AnyOf(_)));

        // Should run when feature is enabled
        assert!(cmd.should_run(&["rust-frontend-lib".to_string()]));
        assert!(cmd.should_run(&["other".to_string(), "rust-frontend-lib".to_string()]));

        // Should not run when feature is disabled
        assert!(!cmd.should_run(&[]));
        assert!(!cmd.should_run(&["spa".to_string()]));
    }

    #[test]
    fn verify_command_parse_or_comma() {
        let cmd = VerifyCommand::parse("[spa,extension] npm test");
        assert_eq!(cmd.command, "npm test");
        if let VerifyCondition::AnyOf(features) = &cmd.condition {
            assert_eq!(features, &["spa", "extension"]);
        } else {
            panic!("Expected AnyOf condition");
        }

        // Should run if ANY of the features is enabled
        assert!(cmd.should_run(&["spa".to_string()]));
        assert!(cmd.should_run(&["extension".to_string()]));
        assert!(cmd.should_run(&["spa".to_string(), "extension".to_string()]));

        // Should not run if none are enabled
        assert!(!cmd.should_run(&[]));
        assert!(!cmd.should_run(&["backend".to_string()]));
    }

    #[test]
    fn verify_command_parse_or_pipe() {
        let cmd = VerifyCommand::parse("[spa|extension] npm test");
        assert_eq!(cmd.command, "npm test");
        if let VerifyCondition::AnyOf(features) = &cmd.condition {
            assert_eq!(features, &["spa", "extension"]);
        } else {
            panic!("Expected AnyOf condition");
        }
    }

    #[test]
    fn verify_command_parse_and() {
        let cmd = VerifyCommand::parse("[spa+backend] npm run e2e");
        assert_eq!(cmd.command, "npm run e2e");
        if let VerifyCondition::AllOf(features) = &cmd.condition {
            assert_eq!(features, &["spa", "backend"]);
        } else {
            panic!("Expected AllOf condition");
        }

        // Should run only if ALL features are enabled
        assert!(cmd.should_run(&["spa".to_string(), "backend".to_string()]));
        assert!(cmd.should_run(&["spa".to_string(), "backend".to_string(), "other".to_string()]));

        // Should not run if any are missing
        assert!(!cmd.should_run(&["spa".to_string()]));
        assert!(!cmd.should_run(&["backend".to_string()]));
        assert!(!cmd.should_run(&[]));
    }

    #[test]
    fn verify_command_parse_whitespace_handling() {
        // Spaces around features
        let cmd = VerifyCommand::parse("[ spa , extension ] npm test");
        assert_eq!(cmd.command, "npm test");
        if let VerifyCondition::AnyOf(features) = &cmd.condition {
            assert_eq!(features, &["spa", "extension"]);
        } else {
            panic!("Expected AnyOf condition");
        }

        // Leading/trailing whitespace
        let cmd2 = VerifyCommand::parse("  cargo build  ");
        assert_eq!(cmd2.command, "cargo build");
    }

    #[test]
    fn verify_command_parse_empty_brackets() {
        // Edge case: empty brackets should result in empty AnyOf
        let cmd = VerifyCommand::parse("[] cargo build");
        assert_eq!(cmd.command, "cargo build");
        // Empty AnyOf should never match
        assert!(!cmd.should_run(&["spa".to_string()]));
    }
}
