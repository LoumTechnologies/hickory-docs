<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-project-env/src/tree.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Verify Covers the Whole Editable Set](../../../../../guarantees/agent/verify-covers-the-whole-editable-set/contract.md)
- [Rebuild the program from its Markdown source](../../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md)

Captured SHA-256: `520ada4ee73433f6b0834807481c481c91e7f82fc90e8ec169a57a923e4d295a`.

<book:entry path="crates/hick-project-env/src/tree.rs">
//! Native publication of an already evaluated project value; no parsing.
use hick_lang::{EntryKind, ProjectTree};
use std::path::{Path, PathBuf};
/// Write an evaluated tree. Validate every destination and supported metadata
/// before any write; never follow a pre-existing symlink in the destination.
pub fn write_tree(root: &Path, tree: &ProjectTree) -> Result<Vec<PathBuf>, String> {
    for (path, entry) in tree.entries() {
        if path.starts_with(hick_lang::EDITOR_BUFFER_PREFIX) {
            return Err(format!(
                "Editor identity has no filesystem destination: {path}"
            ));
        }
        hick_lang::validate_project_path(path)?;
        let mut parent = root.to_path_buf();
        for part in path.split('/') {
            parent.push(part);
            if let Ok(metadata) = std::fs::symlink_metadata(&parent) {
                let destination = parent == root.join(path);
                let replace_link = destination
                    && metadata.file_type().is_symlink()
                    && entry.kind == EntryKind::Symlink;
                if (metadata.file_type().is_symlink() && !replace_link)
                    || (!metadata.is_dir() && !destination)
                {
                    return Err(format!(
                        "refusing materialization through existing non-directory or symlink: {}",
                        parent.display()
                    ));
                }
                if destination && !metadata.is_file() && !replace_link {
                    return Err(format!("occupied project output: {path}"));
                }
            }
        }
        if entry.kind == EntryKind::Symlink {
            let target = std::str::from_utf8(&entry.bytes)
                .map_err(|_| "unsupported non-UTF-8 symlink target")?;
            hick_lang::validate_project_path(target).map_err(|_|format!("native materialization does not support escaping or ambiguous symlink target: {path}"))?;
            #[cfg(not(unix))]
            return Err(format!(
                "symlink materialization is unsupported on this platform: {path}"
            ));
        }
        #[cfg(not(unix))]
        if entry.executable {
            return Err(format!(
                "executable mode materialization is unsupported on this platform: {path}"
            ));
        }
    }
    let mut written = vec![];
    for (path, entry) in tree.entries() {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        if entry.kind == EntryKind::File {
            std::fs::write(&full, &entry.bytes).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(
                    &full,
                    std::fs::Permissions::from_mode(if entry.executable { 0o755 } else { 0o644 }),
                )
                .map_err(|e| e.to_string())?;
            }
        } else {
            #[cfg(unix)]
            {
                if std::fs::symlink_metadata(&full).is_ok() {
                    std::fs::remove_file(&full).map_err(|e| e.to_string())?;
                }
                std::os::unix::fs::symlink(
                    std::str::from_utf8(&entry.bytes).map_err(|e| e.to_string())?,
                    &full,
                )
                .map_err(|e| e.to_string())?;
            }
        }
        written.push(full);
    }
    Ok(written)
}
</book:entry>

</book:doc>
