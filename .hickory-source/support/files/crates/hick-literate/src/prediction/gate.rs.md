<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/src/prediction/gate.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `b030001231938ecdf43a37f5bef4628dc213f6c6e4ae0c1eedf0e66404ee76b8`.

<book:entry path="crates/hick-literate/src/prediction/gate.rs">
//! One private admission/retry/settlement boundary for every observation adapter.
use super::{ManagedOutput, OutputEncoding, output_target};
use anyhow::{Result, ensure};
use hick_observe::prediction::{Actor, FutureStop, Identity, Ledger, ObservedResult, Target};
use serde_json::Value;

pub(super) enum Admission<'a> {
    Existing(ManagedOutput),
    Fresh(Gate<'a>),
}

pub(super) struct Gate<'a> {
    ledger: &'a Ledger,
    attempt: String,
    identity: Identity,
    target: Target,
    future: Option<FutureStop>,
}

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
</book:entry>

</book:doc>
