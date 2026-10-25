<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang/src/project/debug.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Debug uses the latest project containing the displayed code](../../../../../contract.md)

Captured SHA-256: `1f8398daf7f13224745dfe048ce789ac264c566efda71103f9734fdb15f159c7`.

<book:entry path="crates/hick-lang/src/project/debug.rs">
//! Debug actions select the newest evaluated tree containing the displayed bytes.
use super::{EntryKind, ProjectTree, project_byte_location};

/// Versions are complete project prefixes in evaluation order, oldest first.
/// Never synthesize a tree by mixing files from different versions. A rename is
/// followed only through unique byte provenance, never by searching equal text.
pub fn latest_debug_project(
    versions: &[ProjectTree],
    displayed: &ProjectTree,
    path: &str,
) -> Result<(usize, String), String> {
    let file = displayed
        .get(path)
        .filter(|entry| entry.kind == EntryKind::File)
        .ok_or_else(|| {
            format!("No source file is displayed for {path}. Open its code before debugging.")
        })?;
    for (index, tree) in versions.iter().enumerate().rev() {
        let renamed = if tree.get(path).is_none() && !file.bytes.is_empty() {
            project_byte_location(displayed, tree, path, 0)
                .ok()
                .flatten()
                .filter(|(_, offset)| *offset == 0)
                .map(|(path, _)| path)
        } else {
            None
        };
        let target = renamed.as_deref().unwrap_or(path);
        if tree
            .get(target)
            .is_some_and(|entry| entry.kind == EntryKind::File && entry.bytes == file.bytes)
        {
            return Ok((index, target.to_owned()));
        }
    }
    Err(format!(
        "No evaluated project contains the exact displayed code for {path}. Save or reload this code before debugging."
    ))
}
</book:entry>

</book:doc>
