<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/prediction/combined.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9b7bf060a506"></a>
## `crates/hick-literate/src/prediction/combined.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../contract.md).

<a id="f-9b7bf060a506-run"></a>
### `run`

One sealed output + selected-hit comparison for the same source revision.
This is still a host library seam, not application-wide strict mode.

<storycode:copy id="f-9b7bf060a506-run">
//! One sealed output + selected-hit comparison for the same source revision.
//! This is still a host library seam, not application-wide strict mode.
use super::{ManagedOutput, OutputEncoding, debugger, gate, output_target};
use crate::{PipelineConfig, capture::PrivateCapture};
use anyhow::{Result, ensure};
use hick_observe::prediction::{Ledger, Target};
use hickory_executor::Executor;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

pub use super::SealedRequest as Request;

/// Both observations are compared/preserved/promoted together. Ordinary exec and
/// scratch DAP replay are the existing two-run capture architecture; this does
/// not claim the scratch debuggee is the authoritative output-producing run.
/// Only a single generated Python file in an empty input volume is supported.
pub async fn run(
    ledger: &Ledger,
    request: &Request,
    source: &str,
    project: &Path,
    executor: Arc<dyn Executor>,
) -> Result<ManagedOutput> {
    run_with_environment(ledger, request, source, project, executor, None).await
}
</storycode:copy>
<a id="f-9b7bf060a506-run_with_environment"></a>
### `run_with_environment`

<storycode:copy id="f-9b7bf060a506-run_with_environment">
pub async fn run_with_environment(
    ledger: &Ledger,
    request: &Request,
    source: &str,
    project: &Path,
    executor: Arc<dyn Executor>,
    profile: Option<&super::environment::Profile>,
) -> Result<ManagedOutput> {
    let environment = super::environment::Profile::begin(profile, &request.identity).await?;
    let scratch = tempfile::tempdir()
        .map_err(|_| anyhow::anyhow!("Private scenario workspace unavailable"))?;
    let (spec, line, debugger_target) =
        validate(request, source, scratch.path()).map_err(|_| {
            anyhow::anyhow!(
                "Combined prediction admission unavailable: unsupported or changed scenario"
            )
        })?;
    let gate = match gate::admit(
        ledger,
        &request.seal,
        &request.request_id,
        &request.identity,
        &request.actor,
        &request.target,
    )? {
        gate::Admission::Existing(result) => return Ok(result),
        gate::Admission::Fresh(gate) => gate,
    };
    if !executor.transcripts().is_empty() {
        return gate.unavailable();
    }
    let private = Arc::new(Mutex::new(Vec::new()));
    let sink = private.clone();
    let config = PipelineConfig {
        working_dir: Some(scratch.path().to_owned()),
        max_rounds: 1,
        withhold_actual_diagnostics: true,
        private_capture: Some(PrivateCapture {
            project: project.to_owned(),
            receive: Arc::new(move |captures| {
                let mut sink = sink
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Private capture storage unavailable"))?;
                ensure!(
                    sink.is_empty(),
                    "Only one private capture delivery is supported"
                );
                sink.extend_from_slice(captures);
                Ok(())
            }),
        }),
        ..Default::default()
    };
    let result =
        match crate::run_pipeline_live(&[("held-out.md", source)], &config, &[], None, executor)
            .await
        {
            Ok(result) => result,
            Err(_) => return gate.unavailable(),
        };
    let captures = match private.lock() {
        Ok(value) => value.clone(),
        Err(_) => return gate.unavailable(),
    };
    let debug_value = match debugger::selected(&captures, &spec, &debugger_target, line) {
        Ok(value) => value,
        Err(_) => return gate.unavailable(),
    };
    let Target::Combined {
        output: output_target,
        ..
    } = &request.target
    else {
        return gate.unavailable();
    };
    let output = match gate::decode_output(&result, output_target, OutputEncoding::Json) {
        Ok(value) => value,
        Err(_) => return gate.unavailable(),
    };
    let unchanged =
        match super::environment::Profile::unchanged(&environment, &request.identity).await {
            Ok(value) => value,
            Err(_) => return gate.unavailable(),
        };
    let accepted = unchanged && result.expectations.iter().all(|outcome| outcome.passed);
    gate.finish(
        serde_json::json!({"output":output, "debugger":debug_value}),
        serde_json::json!({"adapter":"combined_live_v1", "captures":captures,
            "transcripts":result.transcripts, "expectations":result.expectations}),
        accepted,
    )
}

</storycode:copy>
<a id="f-9b7bf060a506-validate"></a>
### `validate`

<storycode:copy id="f-9b7bf060a506-validate">
pub(super) fn validate(
    request: &Request,
    source: &str,
    scratch: &Path,
) -> Result<(hick_dap::CaptureSpec, u32, Target)> {
    let Target::Combined { output, debugger } = &request.target else {
        anyhow::bail!("Combined target required");
    };
    // Validate the debugger arm while retaining the combined target for the
    // single ledger admission below. The debugger validator deliberately
    // refuses combinations rather than silently selecting an observation.
    let debugger_request = Request {
        seal: request.seal.clone(),
        request_id: request.request_id.clone(),
        actor: request.actor.clone(),
        identity: request.identity.clone(),
        target: debugger.as_ref().clone(),
    };
    let (spec, line) = debugger::validate_source(&debugger_request, source, scratch, true)?;
    // A fixed command and literal file bind both paths to the same program;
    // no independent hand-written stdout program may stand in for the debuggee.
    ensure!(
        spec.file.len() <= 128
            && spec
                .file
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))
    );
    let document = hick_lang::parse(source)?;
    let all = document.all_tags();
    let files: Vec<_> = all.iter().filter(|tag| tag.name == "file").collect();
    let volumes: Vec<_> = all.iter().filter(|tag| tag.name == "volume").collect();
    let execs: Vec<_> = all.iter().filter(|tag| tag.name == "exec").collect();
    ensure!(files.len() == 1 && volumes.len() == 1 && execs.len() == 1);
    ensure!(
        files[0].get_attribute("path") == Some(spec.file.as_str())
            && files[0].child_tags().next().is_none()
    );
    ensure!(
        volumes[0].get_attribute("name") == Some("code")
            && volumes[0].get_attribute("input") == Some(".")
            && volumes[0].get_attribute("output").is_none()
    );
    let exec = execs[0];
    ensure!(
        exec.get_attribute("container") == Some("c")
            && exec.get_attribute("mount") == Some("code:/work")
            && exec.get_attribute("freeze").is_none()
    );
    let command: String = exec
        .children
        .iter()
        .filter_map(|node| match node {
            hick_lang::HickNode::Text(text, _) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    ensure!(command.trim() == format!("python3 work/{}", spec.file));
    ensure!(
        exec.child_tags()
            .all(|tag| ["capture", "expect"].contains(&tag.name.as_str()))
    );
    ensure!(output.as_ref() == &output_target("c", exec.source_line, OutputEncoding::Json));
    Ok((spec, line, debugger.as_ref().clone()))
}
</storycode:copy>

### Assemble `crates/hick-literate/src/prediction/combined.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/prediction/combined.rs"><storycode:paste select="#f-9b7bf060a506-run"/><storycode:paste select="#f-9b7bf060a506-run_with_environment"/><storycode:paste select="#f-9b7bf060a506-validate"/></storycode:file>

</storycode:doc>
