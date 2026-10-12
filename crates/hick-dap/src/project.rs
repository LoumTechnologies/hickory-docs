//! Explicit revision-aware debugger preparation. Native builds and adapters use
//! this complete selected tree; none of these operations run during rendering.
use anyhow::Result;
use hick_lang::EvaluatedProject;
use std::collections::BTreeMap;

pub struct ProjectLaunch {
    pub materialized: hick_lsp::MaterializedProject,
    pub program: std::path::PathBuf,
    pub language: &'static str,
}
pub fn prepare(
    project: &EvaluatedProject,
    side: hick_lsp::ProjectEndpoint,
) -> Result<ProjectLaunch> {
    let materialized = hick_lsp::materialize(project, side).map_err(anyhow::Error::msg)?;
    let files = materialized.files();
    let program = crate::entry_point(&files)?;
    let language = crate::language_of(&program)
        .ok_or_else(|| anyhow::anyhow!("selected project entry has no supported debug language"))?;
    // Adapter discovery does not certify successful building or launching.
    Ok(ProjectLaunch {
        materialized,
        program,
        language,
    })
}
pub fn prepare_closure(
    entry: &str,
    sources: &BTreeMap<String, String>,
    input: &hick_lang::ProjectTree,
    side: hick_lsp::ProjectEndpoint,
) -> Result<ProjectLaunch> {
    let project = hick_lang::evaluate_project(entry, sources, input).map_err(anyhow::Error::msg)?;
    prepare(&project, side)
}

pub(crate) fn document_maps(
    state: &hick_lsp::document::HickDocumentState,
    workdir: &std::path::Path,
) -> std::collections::HashMap<std::path::PathBuf, hick_lsp::position_map::PositionMap> {
    let workdir = workdir
        .canonicalize()
        .unwrap_or_else(|_| workdir.to_path_buf());
    state
        .virtual_files
        .iter()
        .map(|file| {
            (
                workdir.join(&file.path),
                hick_lsp::position_map::PositionMap::build(&file.segments).without_first_line(),
            )
        })
        .collect()
}

pub(crate) fn generated_line(
    files: &std::collections::HashMap<std::path::PathBuf, hick_lsp::position_map::PositionMap>,
    line: u32,
) -> Option<(std::path::PathBuf, u32)> {
    let mut found = None;
    for (path, map) in files {
        for mapping in &map.mappings {
            if mapping.source_line != line {
                continue;
            }
            if found.is_some() {
                return None;
            } // A line breakpoint must have one destination.
            found = Some((path.clone(), mapping.virtual_line));
        }
    }
    found
}
