<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/output_format.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-226a8bef814b"></a>
## `crates/hickory-cli/src/output_format.rs`

<a id="f-226a8bef814b-indent"></a>
### `indent`

<storycode:copy id="f-226a8bef814b-indent">
pub(super) fn indent(s: &str) -> String {
    s.lines()
        .map(|l| format!("    | {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/output_format.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/output_format.rs"><storycode:paste select="#f-226a8bef814b-indent"/></storycode:file>

</storycode:doc>
