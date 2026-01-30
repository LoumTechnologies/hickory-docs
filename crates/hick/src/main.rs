//! Hick: Secure container orchestration shell.
//!
//! Usage: `hick <file.hick> [file2.hick ...]`
//!
//! Parses `.hick` files, builds an information-flow DAG, validates it,
//! and executes containers with capability-based security.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "hick", about = "Secure container orchestration shell")]
struct Cli {
    /// One or more `.hick` files to process.
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

    // Set up secrets provider
    let key_path = hick::expand_tilde(&cli.key_file);
    let secrets_dir = hick::expand_tilde(&cli.secrets_dir);
    let _secrets_provider = hick_secrets::AgeSecretsProvider::new(&key_path, &secrets_dir);

    // Read all input files
    let mut file_contents = Vec::new();
    for path in &cli.files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        file_contents.push((path.display().to_string(), source));
    }

    // Build sources slice for the pipeline
    let sources: Vec<(&str, &str)> = file_contents
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str()))
        .collect();

    // Run the pipeline
    let result = if cli.dry_run {
        hick::run_pipeline(&sources).await?
    } else {
        let images_dir = cli
            .images_dir
            .map(|p| hick::expand_tilde(&p))
            .unwrap_or_else(default_images_dir);

        let config = hick::PipelineConfig { images_dir };
        hick::run_pipeline_live(&sources, &config).await?
    };

    // Write file outputs to disk
    for (path, content) in &result.files {
        info!("Writing output: {path}");
        if let Some(parent) = PathBuf::from(path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
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
