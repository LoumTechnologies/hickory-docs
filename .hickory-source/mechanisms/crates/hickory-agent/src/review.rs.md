<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/review.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-cea59e7c79b9"></a>
## `crates/hickory-agent/src/review.rs`

<a id="f-cea59e7c79b9-whole_edit"></a>
### `whole_edit`

Guard for complete replacements approved by the shared tool review UI.

<storycode:copy id="f-cea59e7c79b9-whole_edit">
//! Guard for complete replacements approved by the shared tool review UI.
pub(crate) fn whole_edit(
    index: &crate::tools::hashline::LineIndex,
    inv: &crate::ToolInvocation,
) -> Option<Result<(usize, String), String>> {
    if inv.arg("whole") != Some("true") {
        return None;
    }
    Some(if inv.arg("snapshot") != Some(index.snapshot.as_str()) {
        Err("A complete reviewed replacement requires the exact surface snapshot.".into())
    } else {
        Ok((
            index.line_end(index.lines.len() - 1),
            inv.input.clone().unwrap_or_default(),
        ))
    })
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/review.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/review.rs"><storycode:paste select="#f-cea59e7c79b9-whole_edit"/></storycode:file>

</storycode:doc>
