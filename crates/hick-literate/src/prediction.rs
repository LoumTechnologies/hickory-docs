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
pub mod debugger;
pub mod environment;
mod gate;
mod source_question;
pub use source_question::{SourceInputs, SourceQuestion};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputEncoding {
    Text,
    Json,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedOutput {
    pub summary: Summary,
    /// False for an idempotent retry, whether running, settled or interrupted.
    pub dispatched: bool,
}

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
