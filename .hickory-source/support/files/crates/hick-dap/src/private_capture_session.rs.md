<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-dap/src/private_capture_session.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c615a48e59abf8873a1da37daa7980440e2480464530433753d43a71d81ae6db`.

<book:entry path="crates/hick-dap/src/private_capture_session.rs">
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
</book:entry>

</book:doc>
