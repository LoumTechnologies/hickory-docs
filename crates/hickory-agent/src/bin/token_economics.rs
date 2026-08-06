//! Token-economics experiment runner + reporter (`just tokens-report`).
//!
//! Subcommands:
//! - `run <spec.json> [--runs-dir DIR]` — execute every (arm x task) of a
//!   committed experiment spec against the real API (needs
//!   `ANTHROPIC_API_KEY`), appending JSONL rows under the runs dir.
//! - `report [--runs-dir DIR] [--out FILE]` — regenerate the Markdown
//!   report from the raw JSONL. Fully offline.
//! - `count-tokens <file...>` — static token measurement of files via
//!   `POST /v1/messages/count_tokens` (the only valid instrument; needs
//!   `ANTHROPIC_API_KEY`). Used by E2 to compare a document against its
//!   woven outputs.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use hickory_agent::harness::{ExperimentSpec, generate_report, run_experiment};
use hickory_agent::{AnthropicClient, Effort, LlmClient, Message, Role};
use hickory_executor::LocalExecutor;

const DEFAULT_RUNS_DIR: &str = "experiments/token-economics/runs";
const DEFAULT_REPORT: &str = "experiments/token-economics/report.md";

fn usage() -> ! {
    eprintln!(
        "usage:\n  token-economics run <spec.json> [--runs-dir DIR]\n  \
         token-economics report [--runs-dir DIR] [--out FILE]\n  \
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
        "run" => {
            let runs_dir = flag_value(&mut args, "--runs-dir")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_RUNS_DIR));
            let [spec_path] = args.as_slice() else {
                usage()
            };
            if std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_default()
                .is_empty()
            {
                bail!(
                    "ANTHROPIC_API_KEY is not set — experiment runs hit the real API. \
                     `report` and the offline tests work without it."
                );
            }
            let spec_path = PathBuf::from(spec_path);
            let spec = ExperimentSpec::load(&spec_path)?;
            let spec_dir = spec_path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            let work_dir = std::env::temp_dir().join(format!(
                "hickory-token-economics-{}-{}",
                std::process::id(),
                chrono::Utc::now().format("%Y%m%d%H%M%S")
            ));
            std::fs::create_dir_all(&work_dir).context("cannot create work dir")?;
            let executor: Arc<dyn hickory_executor::Executor> = Arc::new(LocalExecutor::new()?);
            let out_path = runs_dir.join(format!("{}.jsonl", spec.experiment));
            let factory = |arm: &hickory_agent::harness::ArmSpec| -> Result<Arc<dyn LlmClient>> {
                let mut client = AnthropicClient::new();
                if let Some(model) = &arm.model {
                    client = client.with_model(model.clone());
                }
                if let Some(effort) = &arm.effort {
                    let effort = Effort::parse(effort)
                        .with_context(|| format!("arm {}: bad effort '{effort}'", arm.name))?;
                    client = client.with_effort(effort);
                }
                Ok(Arc::new(client))
            };
            run_experiment(&spec, &spec_dir, &work_dir, &factory, executor, &out_path).await?;
            println!("recorded runs to {}", out_path.display());
        }
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
