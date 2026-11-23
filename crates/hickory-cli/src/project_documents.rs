use crate::{CheckFailure, DocRun, WrittenOutputs};
use anyhow::{Context, Result};
use hick_lang::HickDocument;
use std::{collections::BTreeMap, path::Path};

pub(crate) fn run(path: &Path, source: String, doc: HickDocument) -> Result<DocRun> {
    let root = path.parent().unwrap_or(Path::new("."));
    let entry = path
        .file_name()
        .context("entry Markdown requires a filename")?
        .to_str()
        .context("non-UTF-8 entry path")?;
    let mut sources = BTreeMap::new();
    crate::project_command::load_closure(
        entry,
        &|name| {
            let full = root.join(name);
            if !std::fs::symlink_metadata(&full)?.is_file() {
                anyhow::bail!("Markdown source is not a regular file: {name}");
            }
            let actual = full.canonicalize()?;
            if !actual.starts_with(root.canonicalize()?) {
                anyhow::bail!("Markdown input escapes project root: {name}");
            }
            if name == entry {
                Ok(source.clone())
            } else {
                Ok(std::fs::read_to_string(actual)?)
            }
        },
        &mut sources,
    )?;
    let mut result = hick_literate::project::pipeline(entry, &sources)?;
    result.span_files = sources
        .keys()
        .filter(|name| name.as_str() != entry)
        .map(|name| root.join(name).to_string_lossy().into_owned())
        .collect();
    // Native lineage routes need real contributing source destinations.
    for map in result.provenance_maps.values_mut() {
        let mut adjusted = hick_exec::node::ProvenanceMap::new();
        for original in map.spans() {
            let mut span = original.clone();
            if let hick_exec::node::SourceOrigin::Literal { file, .. } = &mut span.origin {
                *file = std::sync::Arc::from(root.join(file.as_ref()).to_string_lossy().as_ref());
            }
            adjusted.push(span);
        }
        *map = adjusted;
    }
    Ok(DocRun {
        doc_path: path.to_path_buf(),
        source,
        doc,
        result,
    })
}
pub(crate) fn check(run: &DocRun, out_dir: Option<&Path>) -> Result<Vec<CheckFailure>> {
    let project = run
        .result
        .evaluated_project
        .as_ref()
        .context("missing evaluated project")?;
    let root = out_dir.unwrap_or_else(|| run.doc_path.parent().unwrap_or(Path::new(".")));
    let actual = crate::project_revision::directory(root, Some(&project.inventory))?;
    Ok(project
        .new
        .compare(&actual)
        .into_iter()
        .map(|difference| CheckFailure::Drift {
            doc: run.doc_path.clone(),
            output_path: root.join(difference.path),
            detail: format!("complete project inventory: {:?}", difference.kind),
        })
        .collect())
}
pub(crate) fn write(run: &DocRun, out_dir: Option<&Path>) -> Result<WrittenOutputs> {
    let project = run
        .result
        .evaluated_project
        .as_ref()
        .context("missing evaluated project")?;
    let root = out_dir.unwrap_or_else(|| run.doc_path.parent().unwrap_or(Path::new(".")));
    let written = hick_lsp::write_tree(root, &project.new).map_err(anyhow::Error::msg)?;
    Ok(WrittenOutputs {
        written,
        preserved: vec![],
        ignored: vec![],
    })
}

pub(crate) fn debug_state_at(
    document: &Path,
    source: &str,
    scratch: &Path,
    endpoint: &hick_lsp::ProjectEndpoint,
) -> Result<(hick_dap::Mapping, Vec<std::path::PathBuf>, Option<String>)> {
    let doc = hick_lang::parse(source)?;
    if doc.find_tags("project").is_empty() {
        return Ok((
            hick_dap::Mapping::for_document(document, source, scratch)?,
            hick_dap::weave_into(source, scratch)?,
            None,
        ));
    }
    let state = state_at(document, source, endpoint)?;
    let tree = state
        .selected_project
        .as_ref()
        .context("Missing selected project")?;
    anyhow::ensure!(
        !tree.entries().is_empty(),
        "Selected Old project is empty; no executable input exists."
    );
    let files = hick_lsp::write_tree(scratch, tree).map_err(anyhow::Error::msg)?;
    let mapping =
        hick_dap::Mapping::for_state(document, &state, scratch)?.with_observation_source(source);
    let target = match endpoint {
        hick_lsp::ProjectEndpoint::LatestFile { .. }
        | hick_lsp::ProjectEndpoint::LatestCode { .. } => {
            let project = state
                .evaluated_project
                .as_ref()
                .context("Missing evaluated project")?;
            let (_, target) =
                hick_lsp::debug_selection(project, endpoint).map_err(anyhow::Error::msg)?;
            Some(target)
        }
        _ => None,
    };
    Ok((mapping, files, target))
}

/// Capture/evaluate the live closure without writing files or starting tools.
pub(crate) fn state_at(
    document: &Path,
    source: &str,
    endpoint: &hick_lsp::ProjectEndpoint,
) -> Result<hick_lsp::document::HickDocumentState> {
    if hick_lang::parse(source)?.find_tags("project").is_empty() {
        return hick_lsp::document::HickDocumentState::from_source(source).map_err(Into::into);
    }
    let (entry, sources) = project_sources(document, source)?;
    hick_lsp::document::HickDocumentState::from_project_at(
        &entry,
        &sources,
        &hick_lang::ProjectTree::default(),
        endpoint,
    )
    .map_err(Into::into)
}

fn project_sources(
    document: &Path,
    source: &str,
) -> Result<(String, BTreeMap<String, String>)> {
    let root = document.parent().unwrap_or(Path::new("."));
    let entry = document
        .file_name()
        .and_then(|s| s.to_str())
        .context("project entry needs a UTF-8 filename")?;
    let sources = hick_lang::load_project_sources(entry, &|name| {
        if name == entry {
            Ok(source.into())
        } else {
            let source_path = root.join(name);
            if !std::fs::symlink_metadata(&source_path)
                .map_err(|e| e.to_string())?
                .is_file()
            {
                return Err(format!("Markdown source is not a regular file: {name}"));
            }
            let full = source_path.canonicalize().map_err(|e| e.to_string())?;
            if !full.starts_with(root.canonicalize().map_err(|e| e.to_string())?) {
                return Err(format!("Markdown input escapes root: {name}"));
            }
            std::fs::read_to_string(full).map_err(|e| e.to_string())
        }
    })
    .map_err(anyhow::Error::msg)?;
    Ok((entry.to_owned(), sources))
}

pub(crate) fn debug_launch_state(
    workspace: &Path,
    document: &Path,
    source: &str,
    endpoint: &hick_lsp::ProjectEndpoint,
) -> Result<(std::path::PathBuf, hick_lsp::document::HickDocumentState)> {
    if hick_lang::parse(source)?.find_tags("project").is_empty() {
        let workspace = workspace.canonicalize()?;
        for directory in document.ancestors().skip(1).take_while(|dir| dir.starts_with(&workspace)) {
            let candidate = directory.join("project.md");
            if !std::fs::symlink_metadata(&candidate).is_ok_and(|metadata| metadata.is_file()) {
                continue;
            }
            let candidate_source = std::fs::read_to_string(&candidate)?;
            if hick_lang::parse(&candidate_source)?.find_tags("project").is_empty() {
                continue;
            }
            let (entry, mut sources) = project_sources(&candidate, &candidate_source)?;
            let name = document.strip_prefix(directory)?.to_str().context("Invalid Markdown path")?.replace('\\', "/");
            if !sources.contains_key(&name) {
                continue;
            }
            anyhow::ensure!(sources[&name] == source, "The contributor changed while reading the project. Save and try again.");
            sources.insert(name, source.into());
            let state = hick_lsp::document::HickDocumentState::from_project_at(
                &entry, &sources, &hick_lang::ProjectTree::default(), endpoint,
            )?;
            return Ok((directory.into(), state));
        }
    }
    Ok((document.parent().unwrap_or(workspace).into(), state_at(document, source, endpoint)?))
}

#[cfg(test)]
mod workbook_debug_tests {
    use super::*;

    #[test]
    fn ordinary_fence_uses_its_own_saved_bytes_and_source_coordinates() {
        let dir = tempfile::tempdir().unwrap();
        let saved = "Intro\n\n  ```python\n  print(1)\n  ```\n";
        let path = dir.path().join("workbook.md");
        std::fs::write(&path, saved).unwrap();
        let registry = crate::debug_sessions::Registry::with_projects(Default::default(), dir.path().into());
        let launch = registry.launch_context(dir.path(), "hick:///workbook.md", "new", Some("fence:2")).unwrap();
        assert_eq!(launch.viewed, "fence.py");
        assert!(launch.check_expected(None, Some(saved)).is_ok());
        assert!(launch.check_expected(None, Some("changed")).is_err());
        assert_eq!(launch.profiles().unwrap().len(), 2);
        let (scratch, _cleanup) = launch.prepare().unwrap();
        assert_eq!(std::fs::read_to_string(scratch.join("fence.py")).unwrap(), "print(1)\n");
        assert!(!path.with_file_name("fence.py").exists());
        assert!(registry.launch_context(dir.path(), "hick:///workbook.md", "new", Some("fence:3")).is_err());
    }

    #[test]
    fn launch_uses_saved_declared_workbook_closure() {
        let dir = tempfile::tempdir().unwrap();
        let entry = include_str!("../../../examples/simple-ui/project.md");
        let workbook = include_str!("../../../examples/simple-ui/workbook.md");
        std::fs::write(dir.path().join("project.md"), entry).unwrap();
        let path = dir.path().join("workbook.md");
        std::fs::write(&path, workbook).unwrap();
        let path = path.canonicalize().unwrap();
        let (root, state) = debug_launch_state(dir.path(), &path, workbook, &hick_lsp::ProjectEndpoint::New).unwrap();
        let tree = state.selected_project.as_ref().unwrap();
        let code = std::str::from_utf8(&tree.get("generated/coupon.py").unwrap().bytes).unwrap();
        assert!(code.contains("Coupon receipt:"));
        let mapping = hick_dap::Mapping::for_contributor(&path, &root, &state, dir.path(), workbook).unwrap();
        let added = workbook.lines().position(|line| line.starts_with("+    print")).unwrap() as u32;
        let context = workbook.lines().position(|line| line.starts_with("     total =")).unwrap() as u32;
        let removed = workbook.lines().position(|line| line.starts_with("-    print")).unwrap() as u32;
        let (program, line) = mapping.to_generated(added).unwrap();
        assert_eq!(line, 7);
        assert_eq!(mapping.to_document(&program, line), Some(added));
        assert_eq!(mapping.to_generated(context), Some((program.clone(), 6)));
        assert_eq!(mapping.to_generated(removed), None);
        assert!(!dir.path().join("generated").exists());
    }
}
