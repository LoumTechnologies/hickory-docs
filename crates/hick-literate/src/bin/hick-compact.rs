//! Edit compaction tool for hick documents.
//!
//! Merges multiple append-only edits (copy blocks with same class) while
//! preserving output equivalence.
//!
//! # Usage
//!
//! ```bash
//! hick-compact input.hick [-o output.hick] [--verify]
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "hick-compact")]
#[command(about = "Compact a hick document by merging related copy blocks")]
struct Args {
    /// Input hick document.
    input: PathBuf,

    /// Output file (defaults to stdout).
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Verify that compacted document is equivalent to original.
    #[arg(long)]
    verify: bool,

    /// Maximum combinations to test during verification.
    #[arg(long, default_value = "100")]
    max_verify_combinations: usize,
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

    // Read input file
    let source = std::fs::read_to_string(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", args.input.display(), e))?;

    eprintln!("Compacting {}...", args.input.display());

    // Compact the document
    let compacted = hick_literate::compact::compact_document(&source)?;

    // Verify if requested
    if args.verify {
        eprintln!("Verifying equivalence...");

        let result =
            hick_literate::equiv::check_equivalence(&source, &compacted, args.max_verify_combinations)
                .await?;

        if !result.equivalent {
            eprintln!("ERROR: Compacted document is NOT equivalent to original!");
            eprintln!("Failing combinations: {}", result.failures.len());

            for (params, diffs) in &result.failures {
                eprintln!(
                    "\n  Features: {}",
                    if params.features.is_empty() {
                        "(none)"
                    } else {
                        &params.features
                    }
                );
                for diff in diffs {
                    eprintln!("    {}", hick_literate::equiv::format_diff(diff));
                }
            }

            return Err(anyhow::anyhow!("Compaction verification failed"));
        }

        eprintln!(
            "Verified equivalent ({} combinations tested)",
            result.combinations_tested
        );
    }

    // Output
    if let Some(output_path) = &args.output {
        std::fs::write(output_path, &compacted)
            .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", output_path.display(), e))?;
        eprintln!("Wrote compacted document to {}", output_path.display());
    } else {
        print!("{}", compacted);
    }

    Ok(())
}
