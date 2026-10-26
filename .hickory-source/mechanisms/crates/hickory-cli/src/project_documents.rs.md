<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/project_documents.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7cd7736e0d32"></a>
## `crates/hickory-cli/src/project_documents.rs`

Related contracts: [Debug the latest atomic project that still contains the displayed bytes](../../../../guarantees/debugging/debug-uses-the-latest-project-containing-the-displayed-code/contract.md), [Rebuild the program from its Markdown source](../../../../guarantees/literate-projects/markdown-constructs-the-complete-project/contract.md), [Keep language tools and debug launches tied to their captured project](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md).

<a id="f-7cd7736e0d32-run"></a>
### `run`

<storycode:copy id="f-7cd7736e0d32-run">
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
</storycode:copy>
<a id="f-7cd7736e0d32-check"></a>
### `check`

<storycode:copy id="f-7cd7736e0d32-check">
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
</storycode:copy>
<a id="f-7cd7736e0d32-write"></a>
### `write`

<storycode:copy id="f-7cd7736e0d32-write">
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

</storycode:copy>
<a id="f-7cd7736e0d32-debug_state_at"></a>
### `debug_state_at`

<storycode:copy id="f-7cd7736e0d32-debug_state_at">
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

</storycode:copy>
<a id="f-7cd7736e0d32-state_at"></a>
### `state_at`

Capture/evaluate the live closure without writing files or starting tools.

<storycode:copy id="f-7cd7736e0d32-state_at">
/// Capture/evaluate the live closure without writing files or starting tools.
pub(crate) fn state_at(
    document: &Path,
    source: &str,
    endpoint: &hick_lsp::ProjectEndpoint,
) -> Result<hick_lsp::document::HickDocumentState> {
    if hick_lang::parse(source)?.find_tags("project").is_empty() {
        return hick_lsp::document::HickDocumentState::from_source(source).map_err(Into::into);
    }
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
    hick_lsp::document::HickDocumentState::from_project_at(
        entry,
        &sources,
        &hick_lang::ProjectTree::default(),
        endpoint,
    )
    .map_err(Into::into)
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/project_documents.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/project_documents.rs"><storycode:paste select="#f-7cd7736e0d32-run"/><storycode:paste select="#f-7cd7736e0d32-check"/><storycode:paste select="#f-7cd7736e0d32-write"/><storycode:paste select="#f-7cd7736e0d32-debug_state_at"/><storycode:paste select="#f-7cd7736e0d32-state_at"/></storycode:file>

</storycode:doc>
