<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/src/prediction/debugger.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `2b21652dbdac17f0b67445699c881b9b1b9c70c3a75bd3ff4542c19dbe558db9`.

<book:entry path="crates/hick-literate/src/prediction/debugger.rs">
//! Initial, selected-hit Python capture admission. Future-stop interactive
//! predictions require a separate session adapter and are not enabled here.
use super::{ManagedOutput, gate};
use anyhow::{Result, ensure};
use hick_dap::capture::{CaptureSpec, Captured};
use hick_observe::prediction::{Ledger, Target};
use std::path::Path;

pub use super::SealedRequest as DebuggerRequest;

/// Reuse hick:capture parsing, source mapping and DAP watch evaluation. Private
/// results cannot become woven capture tables or public tool/context payloads.
/// Numeric/JSON-rendered values are supported; arbitrary adapter representations
/// are unavailable rather than guessed. Watch context is an adapter hint, not a
/// proof that an arbitrary expression is free of side effects.
///
/// The host must isolate project adapter discovery, ledger and debuggee from the
/// predictor. Scratch launch provides no additional OS execution permissions or
/// sandbox; never expose this as application strict mode on its own.
pub async fn run(
    ledger: &Ledger,
    request: &DebuggerRequest,
    source: &str,
    project: &Path,
) -> Result<ManagedOutput> {
    run_with_environment(ledger, request, source, project, None).await
}
pub async fn run_with_environment(
    ledger: &Ledger,
    request: &DebuggerRequest,
    source: &str,
    project: &Path,
    profile: Option<&super::environment::Profile>,
) -> Result<ManagedOutput> {
    let environment = super::environment::Profile::begin(profile, &request.identity).await?;
    let scratch = tempfile::tempdir()
        .map_err(|_| anyhow::anyhow!("Private debugger workspace unavailable"))?;
    let (spec, document_line) = validate(request, source, scratch.path()).map_err(|_| anyhow::anyhow!(
        "Managed debugger admission unavailable: unsupported capture, frame, span, hit or source identity"
    ))?;
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
    let captures = match hick_dap::capture::run_private(
        Path::new("held-out.md"),
        source,
        project,
        &[spec.clone()],
    )
    .await
    {
        Ok(captures) => captures,
        Err(_) => return gate.unavailable(),
    };
    let actual = match selected(&captures, &spec, &request.target, document_line) {
        Ok(actual) => actual,
        Err(_) => return gate.unavailable(),
    };
    let unchanged =
        match super::environment::Profile::unchanged(&environment, &request.identity).await {
            Ok(value) => value,
            Err(_) => return gate.unavailable(),
        };
    gate.finish(
        actual,
        serde_json::json!({ "adapter": "private_capture_v1", "captures": captures }),
        unchanged,
    )
}

fn validate(request: &DebuggerRequest, source: &str, scratch: &Path) -> Result<(CaptureSpec, u32)> {
    validate_source(request, source, scratch, false)
}
pub(super) fn validate_source(
    request: &DebuggerRequest,
    source: &str,
    scratch: &Path,
    private_volume: bool,
) -> Result<(CaptureSpec, u32)> {
    validate_bounded(request, source, scratch, private_volume, false)
}
pub(super) fn validate_bounded(
    request: &DebuggerRequest,
    source: &str,
    scratch: &Path,
    private_volume: bool,
    interactive: bool,
) -> Result<(CaptureSpec, u32)> {
    use sha2::{Digest, Sha256};
    ensure!(request.identity.source == format!("{:x}", Sha256::digest(source.as_bytes())));
    ensure!(request.identity.dependencies.is_empty());
    let Target::Debugger {
        source: root,
        source_revision,
        start_byte,
        end_byte,
        expression,
        hit,
        frame,
    } = &request.target
    else {
        anyhow::bail!("Debugger target required");
    };
    ensure!(root == "root" && source_revision == &request.identity.source && frame.is_some());
    ensure!((1..=100).contains(hit));
    let document = hick_lang::parse(source)?;
    ensure!(document.all_tags().iter().all(|tag| {
        (private_volume && tag.name == "volume")
            || [
                "doc",
                "file",
                "container",
                "exec",
                "cmd",
                "expect",
                "capture",
            ]
            .contains(&tag.name.as_str())
    }));
    let captures = crate::capture::collect(&document.nodes).map_err(anyhow::Error::msg)?;
    ensure!(captures.len() == 1 && captures[0].specs.len() == 1);
    let spec = captures[0].specs[0].clone();
    ensure!(
        spec.expressions == [expression.clone()]
            && if interactive {
                spec.max as u64 >= *hit && spec.max <= 100
            } else {
                spec.max as u64 == *hit
            }
            && spec.file.ends_with(".py")
    );
    hick_lang::validate_project_path(&spec.file).map_err(anyhow::Error::msg)?;
    let mapping = hick_dap::Mapping::for_document(Path::new("held-out.md"), source, scratch)?;
    let document_line = mapping
        .to_document(
            &scratch.join(&spec.file),
            spec.line
                .checked_sub(1)
                .ok_or_else(|| anyhow::anyhow!("Invalid line"))?,
        )
        .ok_or_else(|| anyhow::anyhow!("Capture source mapping unavailable"))?;
    let (line_start, line_end) = source_span(source, document_line)?;
    ensure!((*start_byte, *end_byte) == (line_start, line_end));
    Ok((spec, document_line))
}

pub(super) fn source_span(source: &str, line_number: u32) -> Result<(u64, u64)> {
    let mut start = 0u64;
    let range = source
        .split_inclusive('\n')
        .enumerate()
        .find_map(|(index, line)| {
            let range = (start, start + line.len() as u64);
            start = range.1;
            (index as u32 == line_number).then_some(range)
        })
        .ok_or_else(|| anyhow::anyhow!("Source line unavailable"))?;
    Ok(range)
}

pub(super) fn selected(
    captures: &[Captured],
    spec: &CaptureSpec,
    target: &Target,
    line: u32,
) -> Result<serde_json::Value> {
    ensure!(captures.len() == 1);
    let capture = &captures[0];
    ensure!(capture.problem.is_none() && capture.moved.is_none());
    ensure!(
        capture.spec.file == spec.file
            && capture.spec.line == spec.line
            && capture.spec.expressions == spec.expressions
    );
    let Target::Debugger {
        hit,
        expression,
        frame: Some(function),
        ..
    } = target
    else {
        anyhow::bail!("Frame required");
    };
    ensure!(capture.hits.len() as u64 == *hit);
    let selected = capture
        .hits
        .last()
        .ok_or_else(|| anyhow::anyhow!("Selected hit never reached"))?;
    let context = selected
        .context
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Frame unavailable"))?;
    ensure!(
        selected.index as u64 == *hit
            && !context.run.is_empty()
            && context.function == *function
            && context.document_line == Some(line)
            && context.frame > 0
            && context.thread > 0
            && context.source_line == spec.line.checked_sub(1)
            && context
                .source
                .as_ref()
                .is_some_and(|source| Path::new(source).ends_with(&spec.file))
            && context.breakpoint_bound
    );
    ensure!(capture.hits.iter().enumerate().all(|(index, prior)| {
        prior.index == index + 1
            && prior.context.as_ref().is_some_and(|prior| {
                prior.thread == context.thread
                    && prior.run == context.run
                    && prior.frame > 0
                    && prior.function == *function
                    && prior.document_line == Some(line)
                    && prior.source == context.source
                    && prior.source_line == context.source_line
                    && prior.breakpoint_bound
            })
    }));
    ensure!(
        selected.values.len() == 1
            && selected.values[0].ok
            && selected.values[0].expression == *expression
    );
    Ok(serde_json::from_str(&selected.values[0].value)?)
}

#[cfg(test)]
mod tests;

#[path = "interactive.rs"]
pub mod interactive;
</book:entry>

</book:doc>
