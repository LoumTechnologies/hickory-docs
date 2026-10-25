<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/coverage/mapping.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Coverage retains its measured scope](../../../../../../guarantees/verification/coverage-retains-its-measured-scope/contract.md)

Captured SHA-256: `3bd6a655056d915ab19e3fc0ea32f203fceb05c999db693d123bd362e7802ebd`.

<book:entry path="crates/hickory-cli/src/coverage/mapping.rs">
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
</book:entry>

</book:doc>
