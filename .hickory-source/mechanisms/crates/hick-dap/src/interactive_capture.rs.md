<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/interactive_capture.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c26b35fd706d"></a>
## `crates/hick-dap/src/interactive_capture.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../guarantees/verification/predictions-precede-managed-observation/contract.md).

<a id="f-c26b35fd706d-PrivateCaptureSession"></a>
### `PrivateCaptureSession`

Retained host-private capture; no public variables, output or evaluator handle.

<storycode:copy id="f-c26b35fd706d-PrivateCaptureSession">
//! Retained host-private capture; no public variables, output or evaluator handle.
use super::*;
use anyhow::ensure;

pub struct PrivateCaptureSession {
    _scratch: tempfile::TempDir,
    session: Session,
    capture: Captured,
    line: u32,
    run: String,
    paused: Option<crate::session::Stopped>,
    deadline: tokio::time::Instant,
    stopped: bool,
}

</storycode:copy>
<a id="f-c26b35fd706d-impl-PrivateCaptureSession"></a>
### `impl-PrivateCaptureSession`

<storycode:copy id="f-c26b35fd706d-impl-PrivateCaptureSession">
impl PrivateCaptureSession {
    /// The host must admit an initial prediction BEFORE calling this method.
    /// Retains one capture location, expression set and original hit bound.
    pub async fn start(
        document: &Path,
        source: &str,
        project: &Path,
        spec: CaptureSpec,
    ) -> Result<Self> {
        ensure!(
            (1..=100).contains(&spec.max),
            "Retained capture needs a bounded hit count"
        );
        let (scratch, mapping, launch) = prepare(document, source, project, true).await?;
        let line = mapping
            .to_document(
                &scratch.path().join(&spec.file),
                spec.line.checked_sub(1).context("Invalid capture line")?,
            )
            .context("Capture location is outside the source mapping")?;
        let run = hick_observe::Run::new(
            mapping.observation_source().into_iter().collect(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        )
        .id;
        let breakpoint = Breakpoint {
            line,
            condition: spec.condition.clone(),
            hit_condition: None,
            log_message: None,
        };
        let (session, statuses) = Session::start_private(launch, mapping, &[breakpoint]).await?;
        let bad = statuses
            .iter()
            .any(|status| status.state.is_refused() || status.moved_to.is_some());
        if bad {
            session.shutdown().await;
            anyhow::bail!("Retained capture breakpoint is refused or moved");
        }
        Ok(Self {
            _scratch: scratch,
            session,
            capture: Captured {
                spec,
                hits: vec![],
                truncated: false,
                problem: None,
                moved: None,
            },
            line,
            run,
            paused: None,
            deadline: tokio::time::Instant::now() + RUN_BUDGET,
            stopped: false,
        })
    }

    /// Host-only observation. No resume may be requested until its future seal
    /// has been admitted. The enclosing prediction adapter owns that ordering.
    pub async fn observe_next(&mut self) -> Result<()> {
        ensure!(
            !self.stopped && self.capture.hits.len() < self.capture.spec.max,
            "Retained capture has ended or reached its hit bound"
        );
        ensure!(
            tokio::time::Instant::now() < self.deadline,
            "Retained capture time budget expired"
        );
        if let Some(prior) = self.paused.take() {
            self.session
                .step(Step::Continue, prior.thread_id, None)
                .await?;
        }
        let remaining = self
            .deadline
            .saturating_duration_since(tokio::time::Instant::now());
        ensure!(!remaining.is_zero(), "Retained capture time budget expired");
        let stopped = self
            .session
            .wait_for_stop(remaining)
            .await?
            .context("Future capture hit was not reached")?;
        let frames = self.session.stack(stopped.thread_id).await?;
        let frame = frames
            .first()
            .context("Retained capture frame unavailable")?;
        ensure!(
            frame.line == Some(self.line),
            "Retained capture stopped outside its selected location"
        );
        let context = CaptureContext {
            run: self.run.clone(),
            thread: stopped.thread_id,
            frame: frame.id,
            function: frame.name.clone(),
            document_line: frame.line,
            source: frame.adapter_source.clone(),
            source_line: frame.adapter_line,
            breakpoint_bound: self
                .session
                .breakpoint_statuses()
                .iter()
                .any(|status| status.line == self.line && status.state.is_bound()),
        };
        let spec = self.capture.spec.clone();
        record(&self.session, frame.id, &spec, &mut self.capture, context).await;
        self.paused = Some(stopped);
        Ok(())
    }

    /// Private values stay inside the host adapter. Never serialize this as a
    /// tool response; only the ledger's bounded comparison feedback may leave.
    pub fn private_capture(&self) -> Captured {
        self.capture.clone()
    }

    pub fn paused_context(&self) -> Option<&CaptureContext> {
        if self.stopped || self.paused.is_none() {
            return None;
        }
        self.capture
            .hits
            .last()
            .and_then(|hit| hit.context.as_ref())
    }

    pub async fn shutdown(&mut self) {
        self.stopped = true;
        self.paused = None;
        self.session.shutdown().await;
    }
}
</storycode:copy>

### Assemble `crates/hick-dap/src/interactive_capture.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/interactive_capture.rs"><storycode:paste select="#f-c26b35fd706d-PrivateCaptureSession"/><storycode:paste select="#f-c26b35fd706d-impl-PrivateCaptureSession"/></storycode:file>

</storycode:doc>
