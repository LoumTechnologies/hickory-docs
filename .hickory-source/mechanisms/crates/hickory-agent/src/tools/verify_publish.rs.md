<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/tools/verify_publish.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4f425a9f3483"></a>
## `crates/hickory-agent/src/tools/verify_publish.rs`

Related contracts: [Verify Covers the Whole Editable Set](../../../../../guarantees/agent/verify-covers-the-whole-editable-set/contract.md).

<a id="f-4f425a9f3483-publish"></a>
### `publish`

Publication is separate from execution. The host selects its acceptance policy.

<storycode:copy id="f-4f425a9f3483-publish">
//! Publication is separate from execution. The host selects its acceptance policy.
use super::*;

pub(super) fn publish(
    root: &Path,
    result: &hick_literate::PipelineResult,
) -> Result<Vec<String>, String> {
    let mut tree = hick_lang::ProjectTree::default();
    let tree = if let Some(project) = &result.evaluated_project {
        &project.new
    } else {
        // Pin binaries and validate the complete inventory before writing any
        // destination. Reuse the native writer's path/symlink safety boundary.
        for (path, content) in &result.files {
            tree.insert(
                path,
                hick_literate::project::file_content_entry(content).map_err(|e| e.to_string())?,
            )?;
        }
        &tree
    };
    let mut paths = hick_project_env::write_tree(root, tree)?
        .into_iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/verify_publish.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/verify_publish.rs"><storycode:paste select="#f-4f425a9f3483-publish"/></storycode:file>

</storycode:doc>
