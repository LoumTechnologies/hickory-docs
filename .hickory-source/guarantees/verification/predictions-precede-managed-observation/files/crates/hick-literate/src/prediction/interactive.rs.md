<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/src/prediction/interactive.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../contract.md)

Captured SHA-256: `883693670d15ce8942b8dcb03e48f823aa11c463bb154d3861cfdca660ec50c0`.

<book:entry path="crates/hick-literate/src/prediction/interactive.rs">
//! Host-owned retained debugger session. All outward feedback uses the shared gate.
use super::{DebuggerRequest, selected, validate_bounded};
use crate::prediction::{ManagedOutput, gate};
use anyhow::{Result, ensure};
use hick_dap::capture::{CaptureSpec, PrivateCaptureSession};
use hick_observe::prediction::{Actor, FutureStop, Identity, Ledger, Seal, Target, digest};
use serde_json::Value;
use std::{collections::HashMap, path::Path};

pub struct RetainedDebugger {
    ledger: Ledger,
    environment: Option<crate::prediction::environment::Profile>,
    private_volume: bool,
    live: PrivateCaptureSession,
    identity: Identity,
    actor: Actor,
    source: String,
    spec: CaptureSpec,
    line: u32,
    hit: u64,
    receipts: HashMap<String, ResumeReceipt>,
}
struct ResumeReceipt {
    seal: String,
    target: Target,
    visible_digest: String,
    result: ManagedOutput,
}
impl RetainedDebugger {
    /// Initial seal precedes launch. An idempotent retry cannot re-create a lost
    /// process. Its receipt is returned without a usable session.
    pub async fn start(
        ledger: &Ledger,
        request: &DebuggerRequest,
        source: &str,
        project: &Path,
    ) -> Result<(Option<Self>, ManagedOutput)> {
        Self::start_with_environment(ledger, request, source, project, None).await
    }
    /// A configured profile is revalidated before launch, before each Continue,
    /// and before settling either stop. This does not freeze the operating system.
    pub async fn start_with_environment(
        ledger: &Ledger,
        request: &DebuggerRequest,
        source: &str,
        project: &Path,
        profile: Option<&crate::prediction::environment::Profile>,
    ) -> Result<(Option<Self>, ManagedOutput)> {
        Self::start_inner(ledger, request, source, project, profile, false).await
    }
    /// Retain the existing scratch debuggee for a validated combined document.
    /// The authoritative output check remains a separate combined Verify run.
    pub async fn start_in_combined_source(
        ledger: &Ledger,
        request: &DebuggerRequest,
        whole: &DebuggerRequest,
        source: &str,
        project: &Path,
        profile: Option<&crate::prediction::environment::Profile>,
    ) -> Result<(Option<Self>, ManagedOutput)> {
        ensure!(
            request.identity == whole.identity && request.actor == whole.actor,
            "Combined retained identity differs"
        );
        let scratch = tempfile::tempdir()?;
        crate::prediction::combined::validate(whole, source, scratch.path())?;
        let Target::Combined { debugger, .. } = &whole.target else {
            anyhow::bail!("Combined source question required");
        };
        let mut first = debugger.as_ref().clone();
        if let Target::Debugger { hit, .. } = &mut first {
            *hit = 1;
        }
        ensure!(
            first == request.target,
            "Retained capture differs from the host combined question"
        );
        Self::start_inner(ledger, request, source, project, profile, true).await
    }
    async fn start_inner(
        ledger: &Ledger,
        request: &DebuggerRequest,
        source: &str,
        project: &Path,
        profile: Option<&crate::prediction::environment::Profile>,
        private_volume: bool,
    ) -> Result<(Option<Self>, ManagedOutput)> {
        let environment =
            crate::prediction::environment::Profile::begin(profile, &request.identity).await?;
        let scratch = tempfile::tempdir()?;
        let (spec, line) = validate_bounded(request, source, scratch.path(), private_volume, true)
            .map_err(|_| anyhow::anyhow!("Unsupported retained debugger source or target"))?;
        ensure!(
            matches!(&request.target, Target::Debugger { hit: 1, .. }),
            "Initial retained prediction must select the first hit"
        );
        let gate = match gate::admit(
            ledger,
            &request.seal,
            &request.request_id,
            &request.identity,
            &request.actor,
            &request.target,
        )? {
            gate::Admission::Existing(result) => return Ok((None, result)),
            gate::Admission::Fresh(gate) => gate,
        };
        let mut live = match PrivateCaptureSession::start(
            Path::new("held-out.md"),
            source,
            project,
            spec.clone(),
        )
        .await
        {
            Ok(live) => live,
            Err(_) => return Ok((None, gate.unavailable()?)),
        };
        if live.observe_next().await.is_err() {
            live.shutdown().await;
            return Ok((None, gate.unavailable()?));
        }
        let captures = vec![live.private_capture()];
        let actual = match selected(&captures, &spec, &request.target, line) {
            Ok(actual) => actual,
            Err(_) => {
                live.shutdown().await;
                return Ok((None, gate.unavailable()?));
            }
        };
        let unchanged = match crate::prediction::environment::Profile::unchanged(
            &environment,
            &request.identity,
        )
        .await
        {
            Ok(value) => value,
            Err(_) => {
                live.shutdown().await;
                return Ok((None, gate.unavailable()?));
            }
        };
        if !unchanged {
            live.shutdown().await;
            return Ok((
                None,
                gate.finish(
                    actual,
                    serde_json::json!({"adapter":"private_capture_v1", "captures":captures}),
                    false,
                )?,
            ));
        }
        let result = gate.finish(
            actual,
            serde_json::json!({"adapter":"private_capture_v1", "captures":captures}),
            true,
        )?;
        Ok((
            Some(Self {
                ledger: ledger.clone(),
                environment,
                private_volume,
                live,
                identity: request.identity.clone(),
                actor: request.actor.clone(),
                source: source.into(),
                spec,
                line,
                hit: 1,
                receipts: HashMap::new(),
            }),
            result,
        ))
    }
    fn stop(&self, prior_visible: &Value) -> Result<FutureStop> {
        let context = self
            .live
            .paused_context()
            .ok_or_else(|| anyhow::anyhow!("Retained debugger is not paused"))?;
        Ok(FutureStop {
            run: context.run.clone(),
            thread: context.thread,
            frame: context.frame,
            hit: self.hit,
            prior_visible_digest: digest(prior_visible)?,
        })
    }
    /// The host supplies only context actually visible to the predictor. Target
    /// values remain private. Scenario admission must recognize the next question;
    /// a renamed previously exposed question cannot claim independent evidence.
    pub fn seal_next(
        &self,
        seal: Seal,
        request_id: String,
        prior_visible: Value,
    ) -> Result<DebuggerRequest> {
        ensure!(
            seal.identity == self.identity,
            "Retained debugger source or environment changed"
        );
        let request = DebuggerRequest {
            seal: String::new(),
            request_id,
            actor: self.actor.clone(),
            identity: self.identity.clone(),
            target: seal.target.clone(),
        };
        let scratch = tempfile::tempdir()?;
        validate_bounded(
            &request,
            &self.source,
            scratch.path(),
            self.private_volume,
            true,
        )
        .map_err(|_| anyhow::anyhow!("Future capture does not match the retained source"))?;
        let future = self.stop(&prior_visible)?;
        let receipt = self.ledger.seal_future(
            self.ledger.history(None)?.generation,
            seal,
            self.actor.clone(),
            future,
            prior_visible,
        )?;
        Ok(DebuggerRequest {
            seal: receipt.id,
            ..request
        })
    }
    /// The exact prior visible context is supplied again by the host so both
    /// context and paused identity must match before the process can continue.
    pub async fn resume(
        &mut self,
        request: &DebuggerRequest,
        prior_visible: &Value,
    ) -> Result<ManagedOutput> {
        ensure!(
            request.identity == self.identity && request.actor == self.actor,
            "Retained debugger owner or identity changed"
        );
        if let Some(receipt) = self.receipts.get(&request.request_id) {
            ensure!(
                receipt.seal == request.seal
                    && receipt.target == request.target
                    && receipt.visible_digest == digest(prior_visible)?,
                "Retained request id was reused for a different operation"
            );
            let mut result = receipt.result.clone();
            result.dispatched = false;
            return Ok(result);
        }
        if !matches!(
            crate::prediction::environment::Profile::unchanged(&self.environment, &self.identity)
                .await,
            Ok(true)
        ) {
            self.live.shutdown().await;
            anyhow::bail!(
                "Retained runtime identity changed or is unavailable; continuation refused"
            );
        }
        let future = self.stop(prior_visible)?;
        let gate = match gate::admit_future(
            &self.ledger,
            &request.seal,
            &request.request_id,
            &self.identity,
            &self.actor,
            &request.target,
            &future,
        )? {
            gate::Admission::Existing(result) => return Ok(result),
            gate::Admission::Fresh(gate) => gate,
        };
        if self.live.observe_next().await.is_err() {
            self.live.shutdown().await;
            return gate.unavailable();
        }
        let captures = vec![self.live.private_capture()];
        let actual = match selected(&captures, &self.spec, &request.target, self.line) {
            Ok(actual) => actual,
            Err(_) => {
                self.live.shutdown().await;
                return gate.unavailable();
            }
        };
        let context = self
            .live
            .paused_context()
            .ok_or_else(|| anyhow::anyhow!("Future stop metadata unavailable"))?;
        if context.run != future.run || context.thread != future.thread {
            self.live.shutdown().await;
            return gate.unavailable();
        }
        let unchanged = match crate::prediction::environment::Profile::unchanged(
            &self.environment,
            &self.identity,
        )
        .await
        {
            Ok(value) => value,
            Err(_) => {
                self.live.shutdown().await;
                return gate.unavailable();
            }
        };
        if !unchanged {
            self.live.shutdown().await;
        }
        self.hit += 1;
        let result = gate.finish(
            actual,
            serde_json::json!({"adapter":"private_capture_v1", "captures":captures}),
            unchanged,
        )?;
        self.receipts.insert(
            request.request_id.clone(),
            ResumeReceipt {
                seal: request.seal.clone(),
                target: request.target.clone(),
                visible_digest: future.prior_visible_digest,
                result: result.clone(),
            },
        );
        Ok(result)
    }
    pub async fn shutdown(&mut self) {
        self.live.shutdown().await;
    }
}
</book:entry>

</book:doc>
