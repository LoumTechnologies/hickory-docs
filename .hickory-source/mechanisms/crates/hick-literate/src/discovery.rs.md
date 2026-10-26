<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/src/discovery.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0745e3a567b7"></a>
## `crates/hick-literate/src/discovery.rs`

<a id="f-0745e3a567b7-default_document_directory_allowed"></a>
### `default_document_directory_allowed`

<storycode:copy id="f-0745e3a567b7-default_document_directory_allowed">
use anyhow::Result;
use std::path::{Path, PathBuf};

fn default_document_directory_allowed(name: &std::ffi::OsStr) -> bool {
    !matches!(
        name.to_str(),
        Some(
            ".git"
                | ".hick-cache"
                | "node_modules"
                | ".venv"
                | "__pycache__"
                | "target"
                | "dist"
                | "dist-site"
                | "build"
                | "bin"
                | "obj"
                | ".dev"
        )
    )
}

</storycode:copy>
<a id="f-0745e3a567b7-is_default_document_path"></a>
### `is_default_document_path`

Whether a discovered Markdown path belongs to the default folder scope.
Only descendants are filtered, so explicitly selecting a cache/build root
still discovers its own documents. Explicit file/config choices bypass this.

<storycode:copy id="f-0745e3a567b7-is_default_document_path">
/// Whether a discovered Markdown path belongs to the default folder scope.
/// Only descendants are filtered, so explicitly selecting a cache/build root
/// still discovers its own documents. Explicit file/config choices bypass this.
pub fn is_default_document_path(root: &Path, path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
        && path
            .strip_prefix(root)
            .ok()
            .and_then(Path::parent)
            .is_some_and(|parent| {
                parent
                    .components()
                    .all(|part| default_document_directory_allowed(part.as_os_str()))
            })
}

</storycode:copy>
<a id="f-0745e3a567b7-discover_default_documents"></a>
### `discover_default_documents`

Discover project Markdown without entering dependency, runtime or build trees.
This is only the default folder policy: a selected file, selected root or
`_hick.yml` file list is an explicit choice and is not filtered here.

<storycode:copy id="f-0745e3a567b7-discover_default_documents">
/// Discover project Markdown without entering dependency, runtime or build trees.
/// This is only the default folder policy: a selected file, selected root or
/// `_hick.yml` file list is an explicit choice and is not filtered here.
pub fn discover_default_documents(root: &Path) -> Result<Vec<PathBuf>> {
    use anyhow::Context as _;
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| {
            entry.depth() == 0
                || !entry.file_type().is_dir()
                || default_document_directory_allowed(entry.file_name())
        })
    {
        let entry =
            entry.with_context(|| format!("discovering documents in {}", root.display()))?;
        // Follow a deliberately linked document, but never recurse through a
        // directory symlink into an unrelated dependency tree or a cycle.
        if entry.path().extension().is_some_and(|ext| ext == "md") && entry.path().is_file() {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(files)
}
</storycode:copy>

### Assemble `crates/hick-literate/src/discovery.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/discovery.rs"><storycode:paste select="#f-0745e3a567b7-default_document_directory_allowed"/><storycode:paste select="#f-0745e3a567b7-is_default_document_path"/><storycode:paste select="#f-0745e3a567b7-discover_default_documents"/></storycode:file>

</storycode:doc>
