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
