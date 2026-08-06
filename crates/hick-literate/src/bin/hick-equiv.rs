//! Semantic equivalence checker for hick documents.
//!
//! Verifies that two hick documents produce identical output for all
//! possible parameter/feature combinations.
//!
//! # Usage
//!
//! ```bash
//! hick-equiv original.hick refactored.hick [--max-combinations N]
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "hick-equiv")]
#[command(about = "Verify semantic equivalence of two hick documents")]
struct Args {
    /// First hick document.
    source1: PathBuf,

    /// Second hick document.
    source2: PathBuf,

    /// Maximum number of feature combinations to test.
    #[arg(long, default_value = "1000")]
    max_combinations: usize,

    /// Show detailed diff output.
    #[arg(long, short)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    match run().await {
        Ok(equivalent) => {
            if equivalent {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(2)
        }
    }
}

async fn run() -> Result<bool> {
    let args = Args::parse();

    // Read source files
    let source1 = std::fs::read_to_string(&args.source1)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", args.source1.display(), e))?;

    let source2 = std::fs::read_to_string(&args.source2)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", args.source2.display(), e))?;

    eprintln!(
        "Checking equivalence between {} and {}...",
        args.source1.display(),
        args.source2.display()
    );

    // Run equivalence check
    let result = hick_literate::equiv::check_equivalence(&source1, &source2, args.max_combinations).await?;

    eprintln!("Tested {} feature combinations", result.combinations_tested);

    if result.equivalent {
        eprintln!("Documents are semantically equivalent.");
        Ok(true)
    } else {
        eprintln!(
            "Documents are NOT equivalent. {} failing combinations:",
            result.failures.len()
        );

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
                if args.verbose {
                    eprintln!("{}", hick_literate::equiv::format_diff(diff));
                } else {
                    match &diff.kind {
                        hick_literate::equiv::DiffKind::OnlyInFirst => {
                            eprintln!("    - {} (only in first)", diff.path);
                        }
                        hick_literate::equiv::DiffKind::OnlyInSecond => {
                            eprintln!("    + {} (only in second)", diff.path);
                        }
                        hick_literate::equiv::DiffKind::ContentDiffers { .. } => {
                            eprintln!("    ~ {} (content differs)", diff.path);
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}
