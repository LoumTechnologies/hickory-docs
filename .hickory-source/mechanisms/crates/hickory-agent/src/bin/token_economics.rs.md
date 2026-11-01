<recovered:doc xmlns:recovered="http://www.hickorydocs.com/1.0">
# Restore the maintained source for crates/hickory-agent/src/bin/token_economics.rs

<a id="output-1f148c2f470b"></a>

For: maintainers reconstructing this repository from its committed Markdown.

This page was referenced by the project but omitted from Git by broad ignore
rules. Its payload is recovered from the tracked output after checking the
captured SHA-256, `58e11cbfa94b1247b4000a1dcabb23fdac41c5bf366d2b725b2b0e06ca812007`. This restores the
existing construction; it makes no behavior change or new verification claim.

<a id="f-1f148c2f470b-DEFAULT_RUNS_DIR"></a>
## DEFAULT_RUNS_DIR

Keep the offline report defaults next to the imports and command overview.

<recovered:copy id="f-1f148c2f470b-DEFAULT_RUNS_DIR">//! Historical token-economics reporter (`just tokens-report`).
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
</recovered:copy>
<a id="f-1f148c2f470b-DEFAULT_REPORT"></a>
## DEFAULT_REPORT

The default report destination is separate from the input run directory.

<recovered:copy id="f-1f148c2f470b-DEFAULT_REPORT">const DEFAULT_REPORT: &str = "experiments/token-economics/report.md";

</recovered:copy>
<a id="f-1f148c2f470b-usage"></a>
## usage

Reject unknown commands and incomplete arguments with a usage error.

<recovered:copy id="f-1f148c2f470b-usage">fn usage() -> ! {
    eprintln!(
        "usage:\n  token-economics report [--runs-dir DIR] [--out FILE]\n  \
         token-economics count-tokens <file...>"
    );
    std::process::exit(2);
}

</recovered:copy>
<a id="f-1f148c2f470b-flag_value"></a>
## flag_value

Remove an option and its required value together so leftover positional arguments remain meaningful.

<recovered:copy id="f-1f148c2f470b-flag_value">fn flag_value(args: &mut Vec<String>, flag: &str) -> Option<String> {
    let idx = args.iter().position(|a| a == flag)?;
    if idx + 1 >= args.len() {
        usage();
    }
    let value = args.remove(idx + 1);
    args.remove(idx);
    Some(value)
}

</recovered:copy>
<a id="f-1f148c2f470b-main"></a>
## main

Choose offline report generation or the explicit authenticated token-count operation; reject missing inputs before processing files.

<recovered:copy id="f-1f148c2f470b-main">#[tokio::main]
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
</recovered:copy>
<recovered:file path="crates/hickory-agent/src/bin/token_economics.rs"><recovered:paste select="#f-1f148c2f470b-DEFAULT_RUNS_DIR"/><recovered:paste select="#f-1f148c2f470b-DEFAULT_REPORT"/><recovered:paste select="#f-1f148c2f470b-usage"/><recovered:paste select="#f-1f148c2f470b-flag_value"/><recovered:paste select="#f-1f148c2f470b-main"/></recovered:file>
</recovered:doc>
