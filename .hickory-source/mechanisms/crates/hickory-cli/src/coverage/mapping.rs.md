<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/coverage/mapping.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9c1a7ccb215d"></a>
## `crates/hickory-cli/src/coverage/mapping.rs`

Related contracts: [Coverage retains its measured scope](../../../../../guarantees/verification/coverage-retains-its-measured-scope/contract.md).

<a id="f-9c1a7ccb215d-capture"></a>
### `capture`

<storycode:copy id="f-9c1a7ccb215d-capture">
use super::*;
pub(super) fn capture(
    root: &Path,
    record: &Record,
) -> Result<BTreeMap<String, BTreeMap<u32, Value>>> {
    let locations = record
        .evidence
        .as_ref()
        .map(|e| {
            e.report
                .files
                .iter()
                .map(|(path, file)| (path.clone(), file.lines.keys().copied().collect()))
                .collect()
        })
        .unwrap_or_default();
    crate::source_mapping::capture(root, &record.identity.source_hashes, &locations)
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/coverage/mapping.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/coverage/mapping.rs"><storycode:paste select="#f-9c1a7ccb215d-capture"/></storycode:file>

</storycode:doc>
