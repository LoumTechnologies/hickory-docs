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
