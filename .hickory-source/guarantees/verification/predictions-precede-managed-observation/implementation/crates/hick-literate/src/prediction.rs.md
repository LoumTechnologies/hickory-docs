<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/prediction.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-72f0d919b421"></a>
## `crates/hick-literate/src/prediction.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-72f0d919b421-combined"></a>
### `combined`

Host-only output admission around the existing live pipeline. No application
enable flag: debugger and cross-interface isolation are still prerequisites.
The caller must own a fresh, private executor and durable ledger inaccessible
to the predictor. The adapter never returns pipeline output, files, errors,
recordings, or diagnostics, including on success. Execution permissions are
those of the supplied executor; prediction admission grants no capabilities.

<storycode:copy id="f-72f0d919b421-combined">
//! Host-only output admission around the existing live pipeline. No application
//! enable flag: debugger and cross-interface isolation are still prerequisites.
//!
//! The caller must own a fresh, private executor and durable ledger inaccessible
//! to the predictor. The adapter never returns pipeline output, files, errors,
//! recordings, or diagnostics, including on success. Execution permissions are
//! those of the supplied executor; prediction admission grants no capabilities.
use crate::{PipelineConfig, expect::ExpectationOutcome};
use anyhow::{Result, ensure};
use hick_observe::prediction::{Actor, Identity, Ledger, Summary, Target};
use hickory_executor::Executor;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod combined;
</storycode:copy>
<a id="f-72f0d919b421-debugger"></a>
### `debugger`

<storycode:copy id="f-72f0d919b421-debugger">
pub mod debugger;
</storycode:copy>
<a id="f-72f0d919b421-environment"></a>
### `environment`

<storycode:copy id="f-72f0d919b421-environment">
pub mod environment;
</storycode:copy>
<a id="f-72f0d919b421-gate"></a>
### `gate`

<storycode:copy id="f-72f0d919b421-gate">
mod gate;
</storycode:copy>
<a id="f-72f0d919b421-source_question"></a>
### `source_question`

<storycode:copy id="f-72f0d919b421-source_question">
mod source_question;
</storycode:copy>
<a id="f-72f0d919b421-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-72f0d919b421-use_declaration">
pub use source_question::{SourceInputs, SourceQuestion};

</storycode:copy>
<a id="f-72f0d919b421-log_expectation"></a>
### `log_expectation`

<storycode:copy id="f-72f0d919b421-log_expectation">
pub(crate) fn log_expectation(outcome: &ExpectationOutcome, withheld: bool) {
    if !outcome.passed {
        if withheld {
            log::warn!("Managed expectation mismatch; actual values withheld");
        } else {
            log::warn!(
                "expectation failed in '{}' at {}:{}: {}",
                outcome.container.as_deref().unwrap_or("agent cell"),
                outcome.doc,
                outcome.line,
                outcome.detail
            );
        }
    }
}

</storycode:copy>
<a id="f-72f0d919b421-OutputEncoding"></a>
### `OutputEncoding`

<storycode:copy id="f-72f0d919b421-OutputEncoding">
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputEncoding {
    Text,
    Json,
}

</storycode:copy>
<a id="f-72f0d919b421-SealedRequest"></a>
### `SealedRequest`

Host-captured target request shared by debugger and combined observations.
Adapters validate source/target correspondence before the common gate admits it.

<storycode:copy id="f-72f0d919b421-SealedRequest">
/// Host-captured target request shared by debugger and combined observations.
/// Adapters validate source/target correspondence before the common gate admits it.
#[derive(Clone)]
pub struct SealedRequest {
    pub seal: String,
    pub request_id: String,
    pub actor: Actor,
    pub identity: Identity,
    pub target: Target,
}

</storycode:copy>
<a id="f-72f0d919b421-OutputRequest"></a>
### `OutputRequest`

Host-captured admission; none of these identities come from tool arguments.
Source identity is independently recomputed here. The host must capture the
concrete inputs and executor/toolchain environment and recognize the scenario
family. Unestablished correspondence must have identity_known=false in the seal.

<storycode:copy id="f-72f0d919b421-OutputRequest">
/// Host-captured admission; none of these identities come from tool arguments.
/// Source identity is independently recomputed here. The host must capture the
/// concrete inputs and executor/toolchain environment and recognize the scenario
/// family. Unestablished correspondence must have identity_known=false in the seal.
pub struct OutputRequest {
    pub seal: String,
    pub request_id: String,
    pub actor: Actor,
    pub identity: Identity,
    pub encoding: OutputEncoding,
}

</storycode:copy>
<a id="f-72f0d919b421-ManagedOutput"></a>
### `ManagedOutput`

<storycode:copy id="f-72f0d919b421-ManagedOutput">
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedOutput {
    pub summary: Summary,
    /// False for an idempotent retry, whether running, settled or interrupted.
    pub dispatched: bool,
}

</storycode:copy>
<a id="f-72f0d919b421-run_output"></a>
### `run_output`

A bounded output adapter, not universal application strict mode. Only one
standalone exec is supported. Unsupported capture/project/volume/agent/cache
paths fail before admission or execution; they cannot silently fall back.
No hooks or caller-selected pipeline configuration are accepted.

<storycode:copy id="f-72f0d919b421-run_output">
/// A bounded output adapter, not universal application strict mode. Only one
/// standalone exec is supported. Unsupported capture/project/volume/agent/cache
/// paths fail before admission or execution; they cannot silently fall back.
/// No hooks or caller-selected pipeline configuration are accepted.
pub async fn run_output(
    ledger: &Ledger,
    request: &OutputRequest,
    source: &str,
    executor: Arc<dyn Executor>,
) -> Result<ManagedOutput> {
    run_output_with_environment(ledger, request, source, executor, None).await
}
</storycode:copy>
<a id="f-72f0d919b421-run_output_with_environment"></a>
### `run_output_with_environment`

Revalidate a host-selected runtime before dispatch and before private settlement.

<storycode:copy id="f-72f0d919b421-run_output_with_environment">
/// Revalidate a host-selected runtime before dispatch and before private settlement.
pub async fn run_output_with_environment(
    ledger: &Ledger,
    request: &OutputRequest,
    source: &str,
    executor: Arc<dyn Executor>,
    profile: Option<&environment::Profile>,
) -> Result<ManagedOutput> {
    let environment = environment::Profile::begin(profile, &request.identity).await?;
    // Parse failures can quote source text. Keep all refusal feedback bounded.
    let target = validate(request, source).map_err(|_| {
        anyhow::anyhow!(
            "Managed output admission unavailable: unsupported document or changed source identity"
        )
    })?;
    let gate = match gate::admit(
        ledger,
        &request.seal,
        &request.request_id,
        &request.identity,
        &request.actor,
        &target,
    )? {
        gate::Admission::Existing(result) => return Ok(result),
        gate::Admission::Fresh(gate) => gate,
    };
    // An executor previously used for another run may retain answer-bearing
    // transcripts/state. Never mix it with this target's evidence.
    if !executor.transcripts().is_empty() {
        return gate.unavailable();
    }
    let scratch = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(_) => return gate.unavailable(),
    };
    let config = PipelineConfig {
        working_dir: Some(scratch.path().to_owned()),
        withhold_actual_diagnostics: true,
        max_rounds: 1,
        ..Default::default()
    };
    let result =
        match crate::run_pipeline_live(&[("held-out.md", source)], &config, &[], None, executor)
            .await
        {
            Ok(result) => result,
            Err(_) => return gate.unavailable(),
        };
    let actual = match gate::decode_output(&result, &target, request.encoding) {
        Ok(actual) => actual,
        Err(_) => return gate.unavailable(),
    };
    // Store diagnostics only inside the private ledger. In particular there is
    // no PipelineResult or publishable output capability in the return value.
    let unchanged = match environment::Profile::unchanged(&environment, &request.identity).await {
        Ok(value) => value,
        Err(_) => return gate.unavailable(),
    };
    let execution_accepted = unchanged && result.expectations.iter().all(|outcome| outcome.passed);
    gate.finish(
        actual,
        serde_json::json!({ "transcripts": result.transcripts,
        "expectations": result.expectations, "adapter": "live_output_v1" }),
        execution_accepted,
    )
}

</storycode:copy>
<a id="f-72f0d919b421-output_target"></a>
### `output_target`

Exact target ID includes the output decoding semantics. Renaming the cell
cannot reset the scenario's ledger history.

<storycode:copy id="f-72f0d919b421-output_target">
/// Exact target ID includes the output decoding semantics. Renaming the cell
/// cannot reset the scenario's ledger history.
pub fn output_target(container: &str, line: usize, encoding: OutputEncoding) -> Target {
    let encoding = match encoding {
        OutputEncoding::Text => "text",
        OutputEncoding::Json => "json",
    };
    Target::Output {
        execution: format!("{container}:{line}:{encoding}"),
    }
}

</storycode:copy>
<a id="f-72f0d919b421-validate"></a>
### `validate`

<storycode:copy id="f-72f0d919b421-validate">
fn validate(request: &OutputRequest, source: &str) -> Result<Target> {
    use sha2::{Digest, Sha256};
    ensure!(request.identity.source == format!("{:x}", Sha256::digest(source.as_bytes())));
    ensure!(request.identity.dependencies.is_empty());
    let document = hick_lang::parse(source)?;
    fn supported(nodes: &[hick_lang::HickNode]) -> bool {
        nodes.iter().all(|node| match node {
            hick_lang::HickNode::Tag(tag) => {
                ["doc", "container", "exec", "expect", "cmd", "file"].contains(&tag.name.as_str())
                    && supported(&tag.children)
            }
            _ => true,
        })
    }
    ensure!(supported(&document.nodes));
    let cells: Vec<_> = document
        .all_tags()
        .into_iter()
        .filter(|tag| tag.name == "exec")
        .collect();
    ensure!(cells.len() == 1);
    let cell = cells[0];
    // Even frozen-without-cache execution is unsupported: admission never
    // presents a replay declaration as an independent fresh runtime witness.
    ensure!(crate::tag_attr(cell, "freeze").is_none());
    let container = crate::tag_attr(cell, "container").unwrap_or_default();
    ensure!(!container.is_empty());
    Ok(output_target(
        &container,
        cell.source_line,
        request.encoding,
    ))
}
</storycode:copy>

### Assemble `crates/hick-literate/src/prediction.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/prediction.rs"><storycode:paste select="#f-72f0d919b421-combined"/><storycode:paste select="#f-72f0d919b421-debugger"/><storycode:paste select="#f-72f0d919b421-environment"/><storycode:paste select="#f-72f0d919b421-gate"/><storycode:paste select="#f-72f0d919b421-source_question"/><storycode:paste select="#f-72f0d919b421-use_declaration"/><storycode:paste select="#f-72f0d919b421-log_expectation"/><storycode:paste select="#f-72f0d919b421-OutputEncoding"/><storycode:paste select="#f-72f0d919b421-SealedRequest"/><storycode:paste select="#f-72f0d919b421-OutputRequest"/><storycode:paste select="#f-72f0d919b421-ManagedOutput"/><storycode:paste select="#f-72f0d919b421-run_output"/><storycode:paste select="#f-72f0d919b421-run_output_with_environment"/><storycode:paste select="#f-72f0d919b421-output_target"/><storycode:paste select="#f-72f0d919b421-validate"/></storycode:file>

</storycode:doc>
