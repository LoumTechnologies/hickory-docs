<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/private_capture_session.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8691d45b608e"></a>
## `crates/hick-dap/src/private_capture_session.rs`

<a id="f-8691d45b608e-impl-Session"></a>
### `impl-Session`

Private capture keeps adapter process output out of host diagnostics.

<storycode:copy id="f-8691d45b608e-impl-Session">
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
</storycode:copy>

### Assemble `crates/hick-dap/src/private_capture_session.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/src/private_capture_session.rs"><storycode:paste select="#f-8691d45b608e-impl-Session"/></storycode:file>

</storycode:doc>
