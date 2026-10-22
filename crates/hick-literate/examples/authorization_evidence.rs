//! Explicit host witness/export using the existing combined pipeline and DAP.
//! No upload, actual-value disclosure, new evaluator or application enable flag.
#[path = "../tests/support/authorization.rs"]
#[allow(dead_code)] // The fixture also exposes the declared-identity test entry point.
mod fixture;
use anyhow::{Result, ensure};
use hick_literate::prediction::combined;
use hick_literate::prediction::environment::capture_python;
use hick_observe::prediction::{Classification, Ledger};
use hickory_executor::LocalExecutor;
use std::{io::Write, path::PathBuf, sync::Arc};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let destination = PathBuf::from(
        args.next()
            .ok_or_else(|| anyhow::anyhow!("Pass an explicit new JSON output path"))?,
    );
    ensure!(args.next().is_none(), "Unexpected export arguments");
    ensure!(
        !destination.exists(),
        "Review existing export before replacing it"
    );
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    ensure!(
        hick_dap::discover("python", &project).is_some(),
        "A configured Python DAP adapter is required"
    );
    let private = tempfile::tempdir()?;
    let ledger = Ledger::open(private.path())?;
    let captured_environment = capture_python(&project)?;
    let mut scenarios = Vec::new();
    for (source, key, classification) in [
        (
            fixture::VULNERABLE,
            "vulnerable",
            Classification::ExecutionRejected,
        ),
        (
            fixture::PATCHED,
            "patched",
            Classification::VerifiedAfterCorrection,
        ),
    ] {
        ensure!(
            capture_python(&project)? == captured_environment,
            "Runtime changed before dispatch; no export written"
        );
        let request = fixture::request_with_environment(
            &ledger,
            source,
            key,
            0,
            captured_environment.clone(),
        );
        let result = combined::run(
            &ledger,
            &request,
            source,
            &project,
            Arc::new(LocalExecutor::new()?),
        )
        .await?;
        ensure!(
            result.summary.classification == classification,
            "Required fixture evidence is unavailable or differs; no export written"
        );
        ensure!(
            capture_python(&project)? == captured_environment,
            "Runtime changed during execution; no export written"
        );
        if classification == Classification::VerifiedAfterCorrection {
            ledger.promote(&result.summary.attempt, &request.identity, fixture::actor())?;
        }
        scenarios.push(result.summary.scenario);
    }
    let selected = ledger.select_history(&scenarios)?;
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)?;
    serde_json::to_writer_pretty(&mut output, &selected)?;
    output.write_all(b"\n")?;
    println!(
        "Wrote selected historical prediction metadata; actuals and diagnostics withheld. Review before publication."
    );
    Ok(())
}
