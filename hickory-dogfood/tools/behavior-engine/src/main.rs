//! Thin native adapter. Discovery, construction and safe writing belong to Hickory.
use hick_lang::{EntryKind, ProjectTree, evaluate_project, load_project_sources};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let action = args.next().ok_or(
        "expected generate, baseline, check, inspect, capture, edits, accept or new-patch",
    )?;
    if !matches!(
        action.as_str(),
        "generate"
            | "baseline"
            | "check"
            | "inspect"
            | "capture"
            | "edits"
            | "accept"
            | "new-patch"
    ) {
        return Err(
            "expected generate, baseline, check, inspect, capture, edits, accept or new-patch"
                .into(),
        );
    }
    let root = PathBuf::from(args.next().ok_or("expected project root")?).canonicalize()?;
    let rest: Vec<String> = args.collect();
    use hick_project_env::pending;
    match action.as_str() {
        "capture" => {
            if let Some(c) = pending::capture(&root, "project.md", &rest)? {
                println!("Pending {}: {}", c.id, c.paths.join(", "));
            } else {
                println!("No external output edits.");
            }
            return Ok(());
        }
        "edits" => {
            if rest.len() > 1 {
                return Err("expected at most one capture id".into());
            }
            let captures = pending::status(&root, "project.md")?;
            if let Some(id) = rest.first() {
                let capture = captures
                    .iter()
                    .find(|c| &c.id == id)
                    .ok_or("unknown pending capture")?;
                println!(
                    "{}",
                    capture
                        .patch
                        .as_deref()
                        .or(capture.patch_error.as_deref())
                        .unwrap_or("No textual patch")
                );
            } else {
                for c in &captures {
                    println!("{}: {}", c.id, c.paths.join(", "));
                }
                if captures.is_empty() {
                    println!("No pending captures.");
                }
            }
            return Ok(());
        }
        "accept" if rest.len() == 1 => {
            pending::accept(&root, "project.md", &rest[0])?;
            println!("Capture incorporated; observations archived.");
            return Ok(());
        }
        "new-patch" if rest.len() == 4 => {
            pending::new_patch(&root, "project.md", &rest[0], &rest[1], &rest[2], &rest[3])?;
            println!(
                "Authored {}; review then accept the pending capture.",
                rest[1]
            );
            return Ok(());
        }
        "accept" | "new-patch" => return Err("wrong capture incorporation arguments".into()),
        _ if !rest.is_empty() => return Err("unexpected arguments".into()),
        _ => (),
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
        let written = pending::materialize(&root, "project.md", &sources, &project.new)?;
        println!(
            "Constructed {} files from {} Markdown sources.",
            written.len(),
            sources.len()
        );
    } else if action == "baseline" {
        let captures = pending::prepare_generation(&root, "project.md", &sources, &project.new)?;
        if !captures.is_empty() {
            return Err(
                "Output captures remain pending; inspect them before recording a baseline".into(),
            );
        }
        pending::checkpoint(&root, "project.md", &sources, &project.new)?;
        println!("Recorded an exact source/output baseline without rewriting products.");
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
