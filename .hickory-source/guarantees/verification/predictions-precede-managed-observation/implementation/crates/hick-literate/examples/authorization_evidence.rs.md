<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/examples/authorization_evidence.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f34cc4987708"></a>
## `crates/hick-literate/examples/authorization_evidence.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-f34cc4987708-fixture"></a>
### `fixture`

Explicit host witness/export using the existing combined pipeline and DAP.
No upload, actual-value disclosure, new evaluator or application enable flag.

<storycode:copy id="f-f34cc4987708-fixture">
//! Explicit host witness/export using the existing combined pipeline and DAP.
//! No upload, actual-value disclosure, new evaluator or application enable flag.
#[path = "../tests/support/authorization.rs"]
#[allow(dead_code)] // The fixture also exposes the declared-identity test entry point.
mod fixture;
</storycode:copy>
<a id="f-f34cc4987708-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-f34cc4987708-use_declaration">
use anyhow::{Result, ensure};
</storycode:copy>
<a id="f-f34cc4987708-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-f34cc4987708-use_declaration-2">
use hick_literate::prediction::combined;
</storycode:copy>
<a id="f-f34cc4987708-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-f34cc4987708-use_declaration-3">
use hick_literate::prediction::environment::capture_python;
</storycode:copy>
<a id="f-f34cc4987708-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-f34cc4987708-use_declaration-4">
use hick_observe::prediction::{Classification, Ledger};
</storycode:copy>
<a id="f-f34cc4987708-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-f34cc4987708-use_declaration-5">
use hickory_executor::LocalExecutor;
</storycode:copy>
<a id="f-f34cc4987708-use_declaration-6"></a>
### `use_declaration`

<storycode:copy id="f-f34cc4987708-use_declaration-6">
use std::{io::Write, path::PathBuf, sync::Arc};

</storycode:copy>
<a id="f-f34cc4987708-main"></a>
### `main`

<storycode:copy id="f-f34cc4987708-main">
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
</storycode:copy>

### Assemble `crates/hick-literate/examples/authorization_evidence.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/examples/authorization_evidence.rs"><storycode:paste select="#f-f34cc4987708-fixture"/><storycode:paste select="#f-f34cc4987708-use_declaration"/><storycode:paste select="#f-f34cc4987708-use_declaration-2"/><storycode:paste select="#f-f34cc4987708-use_declaration-3"/><storycode:paste select="#f-f34cc4987708-use_declaration-4"/><storycode:paste select="#f-f34cc4987708-use_declaration-5"/><storycode:paste select="#f-f34cc4987708-use_declaration-6"/><storycode:paste select="#f-f34cc4987708-main"/></storycode:file>

</storycode:doc>
