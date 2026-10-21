//! Private capture keeps adapter process output out of host diagnostics.
use super::*;

impl Session {
    /// Start an adapter, launch the program, and stop at the first
    /// breakpoint.
    pub async fn start(
        launch: Launch,
        mapping: Arc<Mapping>,
        breakpoints: &[Breakpoint],
    ) -> Result<(Self, Vec<BreakpointStatus>)> {
        Self::start_with_sources(launch, mapping, breakpoints, &[]).await
    }

    /// Initial private capture only. Attached or multiplexed adapters have
    /// external output channels this admission does not control, so refuse them.
    pub async fn start_private(
        launch: Launch,
        mapping: Arc<Mapping>,
        breakpoints: &[Breakpoint],
    ) -> Result<(Self, Vec<BreakpointStatus>)> {
        anyhow::ensure!(
            !launch.multi_session,
            "Private multi-session capture is unavailable"
        );
        let adapter = Adapter::start_private(&launch.adapter, launch.transport).await?;
        let extra = launch.extra.clone();
        Self::start_on(adapter, None, &launch, extra, mapping, breakpoints, &[]).await
    }
}
