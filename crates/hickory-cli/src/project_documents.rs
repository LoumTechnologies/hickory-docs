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
