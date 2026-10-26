<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/prediction/gate.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bcac6949feb6"></a>
## `crates/hick-literate/src/prediction/gate.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-bcac6949feb6-Admission"></a>
### `Admission`

One private admission/retry/settlement boundary for every observation adapter.

<storycode:copy id="f-bcac6949feb6-Admission">
//! One private admission/retry/settlement boundary for every observation adapter.
use super::{ManagedOutput, OutputEncoding, output_target};
use anyhow::{Result, ensure};
use hick_observe::prediction::{Actor, FutureStop, Identity, Ledger, ObservedResult, Target};
use serde_json::Value;

pub(super) enum Admission<'a> {
    Existing(ManagedOutput),
    Fresh(Gate<'a>),
}

</storycode:copy>
<a id="f-bcac6949feb6-Gate"></a>
### `Gate`

<storycode:copy id="f-bcac6949feb6-Gate">
pub(super) struct Gate<'a> {
    ledger: &'a Ledger,
    attempt: String,
    identity: Identity,
    target: Target,
    future: Option<FutureStop>,
}

</storycode:copy>
<a id="f-bcac6949feb6-admit"></a>
### `admit`

<storycode:copy id="f-bcac6949feb6-admit">
pub(super) fn admit<'a>(
    ledger: &'a Ledger,
    seal: &str,
    key: &str,
    identity: &Identity,
    actor: &Actor,
    target: &Target,
) -> Result<Admission<'a>> {
    admit_inner(ledger, seal, key, identity, actor, target, None)
}
</storycode:copy>
<a id="f-bcac6949feb6-admit_future"></a>
### `admit_future`

<storycode:copy id="f-bcac6949feb6-admit_future">
pub(super) fn admit_future<'a>(
    ledger: &'a Ledger,
    seal: &str,
    key: &str,
    identity: &Identity,
    actor: &Actor,
    target: &Target,
    future: &FutureStop,
) -> Result<Admission<'a>> {
    admit_inner(ledger, seal, key, identity, actor, target, Some(future))
}
</storycode:copy>
<a id="f-bcac6949feb6-admit_inner"></a>
### `admit_inner`

<storycode:copy id="f-bcac6949feb6-admit_inner">
fn admit_inner<'a>(
    ledger: &'a Ledger,
    seal: &str,
    key: &str,
    identity: &Identity,
    actor: &Actor,
    target: &Target,
    future: Option<&FutureStop>,
) -> Result<Admission<'a>> {
    let admission = match future {
        Some(future) => {
            ledger.admit_future_target(seal, key, identity, actor.clone(), target, future)
        }
        None => ledger.admit_target(seal, key, identity, false, actor.clone(), target),
    }
    .map_err(|_| {
        anyhow::anyhow!("Managed prediction admission refused; inspect current sealed history")
    })?;
    let attempt = admission.receipt.id;
    if !admission.dispatch {
        let summary = ledger
            .history(Some(identity))?
            .attempts
            .into_iter()
            .find(|entry| entry.attempt == attempt)
            .ok_or_else(|| anyhow::anyhow!("Managed prediction history unavailable"))?;
        return Ok(Admission::Existing(ManagedOutput {
            summary,
            dispatched: false,
        }));
    }
    Ok(Admission::Fresh(Gate {
        ledger,
        attempt,
        identity: identity.clone(),
        target: target.clone(),
        future: future.cloned(),
    }))
}

</storycode:copy>
<a id="f-bcac6949feb6-impl-Gate-_"></a>
### `impl-Gate<'_>`

<storycode:copy id="f-bcac6949feb6-impl-Gate-_">
impl Gate<'_> {
    /// A missing observation never releases the execution claim. The adapter
    /// does not know whether a remote job/debuggee stopped merely from an error.
    pub fn unavailable(self) -> Result<ManagedOutput> {
        let summary = self.ledger.mark_unavailable(
            &self.attempt,
            Actor {
                kind: "host".into(),
                id: "private_prediction_adapter".into(),
            },
        )?;
        Ok(ManagedOutput {
            summary,
            dispatched: true,
        })
    }

    /// Adapters supply observations and existing runner-check verdicts. Only
    /// the ledger decides eligibility from prediction + requirement + checks for
    /// output-only, debugger-only and combined observations identically.
    pub fn finish(
        self,
        actual: Value,
        evidence: Value,
        execution_accepted: bool,
    ) -> Result<ManagedOutput> {
        let observation = ObservedResult {
            identity: self.identity.clone(),
            target: self.target.clone(),
            actual,
            evidence,
        };
        let result = match &self.future {
            Some(future) => self.ledger.finish_future_checked(
                &self.attempt,
                observation,
                execution_accepted,
                future,
            ),
            None => self
                .ledger
                .finish_checked(&self.attempt, observation, execution_accepted),
        };
        match result {
            Ok(summary) => Ok(ManagedOutput {
                summary,
                dispatched: true,
            }),
            Err(_) => self.unavailable(),
        }
    }
}

</storycode:copy>
<a id="f-bcac6949feb6-decode_output"></a>
### `decode_output`

Shared output intake: one fresh, exactly identified cell, no missing/stale/
kept project evidence. Raw output never escapes the calling private adapter.

<storycode:copy id="f-bcac6949feb6-decode_output">
/// Shared output intake: one fresh, exactly identified cell, no missing/stale/
/// kept project evidence. Raw output never escapes the calling private adapter.
pub(super) fn decode_output(
    result: &crate::PipelineResult,
    target: &Target,
    encoding: OutputEncoding,
) -> Result<Value> {
    ensure!(
        result.never_run.is_empty()
            && result.stale.is_empty()
            && result.from_document.is_empty()
            && result.evaluated_project.is_none()
    );
    let entries: Vec<_> = result
        .transcripts
        .iter()
        .flat_map(|(container, entries)| entries.iter().map(move |entry| (container, entry)))
        .collect();
    ensure!(entries.len() == 1);
    let (container, entry) = entries[0];
    let line = entry
        .source_line
        .ok_or_else(|| anyhow::anyhow!("Output source unavailable"))?;
    ensure!(&output_target(container, line, encoding) == target);
    Ok(match encoding {
        OutputEncoding::Text => Value::String(entry.output.clone()),
        OutputEncoding::Json => serde_json::from_str(&entry.output)?,
    })
}
</storycode:copy>

### Assemble `crates/hick-literate/src/prediction/gate.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/prediction/gate.rs"><storycode:paste select="#f-bcac6949feb6-Admission"/><storycode:paste select="#f-bcac6949feb6-Gate"/><storycode:paste select="#f-bcac6949feb6-admit"/><storycode:paste select="#f-bcac6949feb6-admit_future"/><storycode:paste select="#f-bcac6949feb6-admit_inner"/><storycode:paste select="#f-bcac6949feb6-impl-Gate-_"/><storycode:paste select="#f-bcac6949feb6-decode_output"/></storycode:file>

</storycode:doc>
