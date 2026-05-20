//! Session → pipeline promotion tool.
//!
//! Extracts a clean, minimal pipeline document from an agent session file,
//! keeping only the surviving (last-written) side effects.
//!
//! # Usage
//!
//! ```bash
//! hick-promote session.hick [-o pipeline.hick] [--append _hick.yml] [--dry-run]
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "hick-promote")]
#[command(about = "Promote surviving writes from a session file into a pipeline document")]
struct Args {
    /// Session file to promote (hick:session format).
    session: PathBuf,

    /// Output pipeline file (defaults to stdout).
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Append promoted elements to an existing pipeline file (e.g. _hick.yml).
    #[arg(long)]
    append: Option<PathBuf>,

    /// Print the promoted document without writing any files.
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    let session_source = std::fs::read_to_string(&args.session)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", args.session.display(), e))?;

    let project_dir = args
        .session
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    let session_name = args
        .session
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("session.hick");

    eprintln!("Promoting {}...", args.session.display());

    let result = hick_literate::promote::promote(&hick_literate::promote::PromoteOpts {
        session_source: &session_source,
        project_dir: &project_dir,
        session_name,
    })?;

    eprintln!(
        "  {} write(s) found, {} surviving",
        result.total_writes, result.surviving_writes
    );

    if args.dry_run {
        print!("{}", result.promoted_source);
        return Ok(());
    }

    if let Some(append_path) = &args.append {
        let existing = std::fs::read_to_string(append_path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", append_path.display(), e))?;
        let combined = format!("{}\n{}", existing.trim_end(), result.promoted_source);
        std::fs::write(append_path, combined)
            .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", append_path.display(), e))?;
        eprintln!("Appended to {}", append_path.display());
        return Ok(());
    }

    if let Some(output_path) = &args.output {
        std::fs::write(output_path, &result.promoted_source)
            .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", output_path.display(), e))?;
        eprintln!("Wrote promoted pipeline to {}", output_path.display());
    } else {
        print!("{}", result.promoted_source);
    }

    Ok(())
}
