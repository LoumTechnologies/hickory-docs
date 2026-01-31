//! Hick: Secure container orchestration shell.
//!
//! Usage: `hick [file.hick ...] [--config _hick.yml]`
//!
//! Parses `.hick` files, builds an information-flow DAG, validates it,
//! and executes containers with capability-based security.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use log::{debug, info};

use hick::config::{HickConfig, find_config};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "hick", about = "Secure container orchestration shell")]
struct Cli {
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

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_default_env()
        .filter_level(if cli.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

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

        let pipeline_config = hick::PipelineConfig { images_dir };
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
