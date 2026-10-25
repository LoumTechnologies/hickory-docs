//! Thin native adapter. Discovery, construction and safe writing belong to Hickory.
use hick_lang::{EntryKind, ProjectTree, evaluate_project, load_project_sources};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let action = args.next().ok_or("expected generate, check or inspect")?;
    if !matches!(action.as_str(), "generate" | "check" | "inspect") {
        return Err("expected generate, check or inspect".into());
    }
    let root = PathBuf::from(args.next().ok_or("expected project root")?).canonicalize()?;
    if args.next().is_some() {
        return Err("unexpected arguments".into());
    }
    let sources = load_project_sources("project.md", &|name| {
        let path = root.join(name);
        if !std::fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .is_file()
        {
            return Err(format!("Markdown input is not a regular file: {name}"));
        }
        let actual = path.canonicalize().map_err(|e| e.to_string())?;
        if !actual.starts_with(&root) {
            return Err(format!("Markdown input escapes root: {name}"));
        }
        std::fs::read_to_string(actual).map_err(|e| e.to_string())
    })?;
    let project = evaluate_project("project.md", &sources, &ProjectTree::default())?;
    if action == "generate" {
        let written = hick_project_env::write_tree(&root, &project.new)?;
        println!(
            "Constructed {} files from {} Markdown sources.",
            written.len(),
            sources.len()
        );
    } else if action == "check" {
        for (path, entry) in project.new.entries() {
            let full = root.join(path);
            let actual = full.canonicalize()?;
            if !actual.starts_with(&root) {
                return Err(format!("output escapes root: {path}").into());
            }
            let metadata = std::fs::symlink_metadata(&full)?;
            let bytes = match entry.kind {
                EntryKind::File if metadata.is_file() => std::fs::read(&full)?,
                EntryKind::Symlink if metadata.file_type().is_symlink() => {
                    std::fs::read_link(&full)?
                        .to_str()
                        .ok_or("non-UTF-8 link")?
                        .as_bytes()
                        .to_vec()
                }
                _ => return Err(format!("output kind differs: {path}").into()),
            };
            if bytes.as_slice() != entry.bytes.as_ref() {
                return Err(format!("output bytes differ: {path}").into());
            }
            #[cfg(unix)]
            if entry.kind == EntryKind::File {
                use std::os::unix::fs::PermissionsExt;
                if (metadata.permissions().mode() & 0o111 != 0) != entry.executable {
                    return Err(format!("output executable mode differs: {path}").into());
                }
            }
        }
        println!(
            "Checked {} constructed outputs against {} Markdown sources.",
            project.new.entries().len(),
            sources.len()
        );
    } else {
        println!(
            "Evaluated {} files from {} Markdown sources without writing.",
            project.new.entries().len(),
            sources.len()
        );
    }
    Ok(())
}
