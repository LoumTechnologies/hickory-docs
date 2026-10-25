<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-dap/src/project.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Native project tools select complete endpoints](../../../../contract.md)

Captured SHA-256: `639fc6a754d59282585eb8b1f15ba31c8e732c41f88e6f74ba2a1e8e134e7f53`.

<book:entry path="crates/hick-dap/src/project.rs">
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
                state
                    .project_maps
                    .get(&file.path)
                    .cloned()
                    .unwrap_or_else(|| {
                        hick_lsp::position_map::PositionMap::build(&file.segments)
                            .without_first_line()
                    }),
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
</book:entry>

</book:doc>
