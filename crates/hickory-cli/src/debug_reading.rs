//! One producer for UI and MCP debugger readings; no execution policy.
use super::Live;
impl Live {
    pub async fn capture_reading(
        &self,
        session: &str,
        reason: &str,
        frames: &[hick_dap::Frame],
        variables: &[hick_dap::Variable],
        selected_frame: Option<i64>,
    ) -> anyhow::Result<Option<hick_observe::reading::Reading>> {
        let Some(source) = self.session.observation_source.clone() else {
            return Ok(None);
        };
        use hick_observe::reading::{DebugAnnotation as A, Reading, Scope};
        let stopped = self.stopped.lock().await.clone();
        let reason = stopped
            .as_ref()
            .map(|s| s.reason.as_str())
            .unwrap_or(reason);
        let mut held = self.reading.lock().await;
        let mut run = self.observations.lock().await;
        let scope = Scope {
            session: Some(session.into()),
            run: run.id.clone(),
            stop: format!("{}:S{}", session, *self.stop_generation.lock().await),
            frame: selected_frame,
            invocation: None,
        };
        let mut reading = Reading::new(source, scope);
        reading.source_aliases = self.source_aliases.clone();
        if !self.session.capabilities().step_back {
            reading.unsupported.push("reverse execution".into());
        }
        if !self.session.capabilities().restart_frame {
            reading.unsupported.push("restart frame".into());
        }
        let mut add = |line, frame, payload| reading.annotate(&mut run, line, frame, payload);
        let result: anyhow::Result<()> = (|| {
            for bp in &self.session.breakpoint_statuses() {
                let breakpoint = format!("{session}:B{}", bp.line);
                add(
                    Some(bp.line),
                    None,
                    A::BreakpointRequested {
                        breakpoint: breakpoint.clone(),
                        state: match bp.state {
                            hick_dap::BindState::Bound => "bound",
                            hick_dap::BindState::Pending => "pending",
                            hick_dap::BindState::Refused => "refused",
                        }
                        .into(),
                    },
                )?;
                if bp.state.is_bound() {
                    add(
                        Some(bp.moved_to.unwrap_or(bp.line)),
                        None,
                        A::BreakpointBound { breakpoint },
                    )?;
                }
            }
            for (depth, frame) in frames.iter().enumerate() {
                add(
                    frame.line,
                    Some(frame.id),
                    A::Frame {
                        name: frame.name.clone(),
                        depth,
                        reported_source: frame.source.clone(),
                        reported_line: frame.source_line,
                    },
                )?;
            }
            let frame = frames.first().map(|f| f.id);
            let line = frames.first().and_then(|f| f.line);
            add(
                line,
                frame,
                A::Paused {
                    reason: reason.into(),
                },
            )?;
            add(
                frames
                    .iter()
                    .find(|f| Some(f.id) == selected_frame)
                    .and_then(|f| f.line),
                selected_frame,
                A::Values {
                    values: variables
                        .iter()
                        .map(|v| (v.name.clone(), v.value.clone()))
                        .collect(),
                },
            )?;
            if reason == "exception" {
                add(
                    line,
                    frame,
                    A::Exception {
                        description: stopped
                            .as_ref()
                            .and_then(|s| s.description.clone().or_else(|| s.text.clone())),
                    },
                )?;
            }
            Ok(())
        })();
        result?;
        // Re-reading one stop does not erase explicit evaluation receipts.
        if let Some(previous) = held.as_ref().filter(|p| p.scope.stop == reading.scope.stop) {
            for annotation in previous
                .annotations
                .iter()
                .filter(|a| matches!(a.payload, A::Expression { .. }))
            {
                if let Some(location) = annotation
                    .location
                    .as_ref()
                    .and_then(|id| previous.locations.iter().find(|l| &l.id == id))
                    && !reading.locations.iter().any(|l| l.id == location.id)
                {
                    reading.locations.push(location.clone());
                }
                reading.annotations.push(annotation.clone());
            }
        }
        *held = Some(reading.clone());
        Ok(Some(reading))
    }
    pub async fn record_expression(
        &self,
        expression: &str,
        value: &str,
        frame: Option<i64>,
    ) -> anyhow::Result<Option<hick_observe::reading::Reading>> {
        use hick_observe::reading::DebugAnnotation;
        let mut held = self.reading.lock().await;
        let Some(reading) = held.as_mut() else {
            return Ok(None);
        };
        let mut run = self.observations.lock().await;
        let location = reading
            .annotations
            .iter()
            .find(|a| matches!(a.payload, DebugAnnotation::Frame { .. }) && a.scope.frame == frame)
            .and_then(|a| a.location.as_ref());
        let line = location
            .and_then(|id| reading.locations.iter().find(|l| &l.id == id))
            .map(|l| l.line);
        let receipt = format!("{}:evaluation:{}", run.id, run.events.len());
        reading.annotate(
            &mut run,
            line,
            frame,
            DebugAnnotation::Expression {
                expression: expression.into(),
                value: value.into(),
                receipt,
            },
        )?;
        Ok(Some(reading.clone()))
    }
    pub async fn resolve_location(
        &self,
        location: &str,
        scope: &hick_observe::reading::Scope,
    ) -> anyhow::Result<u32> {
        let held = self.reading.lock().await;
        let reading = held
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no current stop"))?;
        let current = if reading.source.uri.starts_with("hick-project:///") {
            reading.source.bytes.clone()
        } else {
            std::fs::read(&reading.source.uri)?
        };
        let thread = self
            .thread_id
            .lock()
            .await
            .ok_or_else(|| anyhow::anyhow!("no current stopped thread"))?;
        let actual_frame = self.session.stack(thread).await?.first().map(|f| f.id);
        reading.resolve(location, scope, &current, actual_frame)
    }
}
