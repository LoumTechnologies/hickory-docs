<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/backend/diagnostic_context.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-54babc1a0113"></a>
## `crates/hick-lsp/src/backend/diagnostic_context.rs`

<a id="f-54babc1a0113-provider_action_diagnostics"></a>
### `provider_action_diagnostics`

<storycode:copy id="f-54babc1a0113-provider_action_diagnostics">
use super::*;

/// Keep provider diagnostics in the child's coordinate system and retain
/// provider-specific code/data used to identify fixes. An empty selection
/// asks for fixes on the cursor's line, including an insertion beside it.
pub(super) fn provider_action_diagnostics(
    diagnostics: &[serde_json::Value],
    requested: Range,
) -> Vec<serde_json::Value> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            let Some(range) = diagnostic
                .get("range")
                .and_then(|value| serde_json::from_value::<Range>(value.clone()).ok())
            else {
                return false;
            };
            if requested.start == requested.end {
                range.start.line <= requested.start.line && range.end.line >= requested.start.line
            } else {
                range.start <= requested.end && range.end >= requested.start
            }
        })
        .cloned()
        .collect()
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/backend/diagnostic_context.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/backend/diagnostic_context.rs"><storycode:paste select="#f-54babc1a0113-provider_action_diagnostics"/></storycode:file>

</storycode:doc>
