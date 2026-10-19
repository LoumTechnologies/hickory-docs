//! Historical token-economics reporter (`just tokens-report`).
//!
//! Subcommands:
//! - `report [--runs-dir DIR] [--out FILE]` — regenerate the Markdown
//!   report from the raw JSONL. Fully offline.
//! - `count-tokens <file...>` — static token measurement of files via
//!   `POST /v1/messages/count_tokens` (the only valid instrument; needs
//!   `ANTHROPIC_API_KEY`). Used by E2 to compare a document against its
//!   woven outputs.

use std::path::PathBuf;

use anyhow::{Context as _, Result, bail};
use hickory_agent::harness::generate_report;
use hickory_agent::{AnthropicClient, Message, Role};

const DEFAULT_RUNS_DIR: &str = "experiments/token-economics/runs";
const DEFAULT_REPORT: &str = "experiments/token-economics/report.md";

fn usage() -> ! {
    eprintln!(
        "usage:\n  token-economics report [--runs-dir DIR] [--out FILE]\n  \
         token-economics count-tokens <file...>"
    );
    std::process::exit(2);
}

fn flag_value(args: &mut Vec<String>, flag: &str) -> Option<String> {
    let idx = args.iter().position(|a| a == flag)?;
    if idx + 1 >= args.len() {
        usage();
    }
    let value = args.remove(idx + 1);
    args.remove(idx);
    Some(value)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    let cmd = args.remove(0);
    match cmd.as_str() {
        "report" => {
            let runs_dir = flag_value(&mut args, "--runs-dir")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_RUNS_DIR));
            let out = flag_value(&mut args, "--out")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_REPORT));
            if !args.is_empty() {
                usage();
            }
            let report = generate_report(&runs_dir)?;
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out, report)?;
            println!("wrote {}", out.display());
        }
        "count-tokens" => {
            if args.is_empty() {
                usage();
            }
            if std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_default()
                .is_empty()
            {
                bail!(
                    "ANTHROPIC_API_KEY is not set — count_tokens is an API call (and the \
                     only valid token measurement; we never estimate with heuristics)."
                );
            }
            let client = AnthropicClient::new();
            println!("| file | tokens |");
            println!("|---|---:|");
            for path in &args {
                let text =
                    std::fs::read_to_string(path).with_context(|| format!("cannot read {path}"))?;
                let tokens = client
                    .count_tokens(&[Message::new(Role::User, text)])
                    .await?;
                println!("| {path} | {tokens} |");
            }
        }
        _ => usage(),
    }
    Ok(())
}
