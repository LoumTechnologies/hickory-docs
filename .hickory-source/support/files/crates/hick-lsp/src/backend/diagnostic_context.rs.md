<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lsp/src/backend/diagnostic_context.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ccbf8bd75ece601681a64edd8ef3df2267a70806ccec8f237a6fefbefdca0950`.

<book:entry path="crates/hick-lsp/src/backend/diagnostic_context.rs">
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
</book:entry>

</book:doc>
