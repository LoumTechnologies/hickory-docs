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
