//! Portable project evaluation shared by pipeline callers and native services.
use crate::PipelineResult;
use anyhow::{Result, bail};
use hick_exec::node::{BinaryData, FileContent, ProvenanceMap, SourceOrigin};
use hick_lang::{EntryKind, ProjectEntry, ProjectTree, SourceSpan, evaluate_project};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

pub fn file_content_entry(content: &FileContent) -> Result<ProjectEntry> {
    let bytes = match content {
        FileContent::Text(text) => text.as_bytes().to_vec(),
        FileContent::Binary(data) => data.to_bytes()?,
    };
    Ok(ProjectEntry::file(bytes))
}

pub fn pipeline(entry: &str, sources: &BTreeMap<String, String>) -> Result<PipelineResult> {
    let evaluated =
        evaluate_project(entry, sources, &ProjectTree::default()).map_err(anyhow::Error::msg)?;
    let mut files = HashMap::new();
    let mut provenance_maps = HashMap::new();
    for (path, entry) in evaluated.new.entries() {
        // Symlinks have metadata rather than file content; callers must use
        // evaluated_project for full inventory and materialization.
        if entry.kind == EntryKind::File {
            let content = match std::str::from_utf8(&entry.bytes) {
                Ok(text) => FileContent::Text(text.to_owned()),
                Err(_) => FileContent::Binary(BinaryData::Inline(entry.bytes.to_vec())),
            };
            files.insert(path.clone(), content);
        }
        let mut map = ProvenanceMap::new();
        for origin in &entry.origins {
            let source = sources
                .get(&origin.document)
                .ok_or_else(|| anyhow::anyhow!("missing provenance source {}", origin.document))?;
            let prefix = source
                .get(..origin.source.0)
                .ok_or_else(|| anyhow::anyhow!("invalid provenance coordinate"))?;
            let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
            let col = prefix.rsplit('\n').next().unwrap_or("").chars().count();
            map.push(hick_exec::node::ProvenanceSpan {
                output_start: origin.output.0,
                output_end: origin.output.1,
                origin: SourceOrigin::Literal {
                    file: Arc::from(origin.document.as_str()),
                    span: SourceSpan::new(origin.source.0, origin.source.1, line, col),
                },
            });
        }
        provenance_maps.insert(path.clone(), map);
    }
    Ok(PipelineResult {
        evaluated_project: Some(evaluated),
        files,
        provenance_maps,
        containers: HashMap::new(),
        volume_provenance: HashMap::new(),
        resource_stats: HashMap::new(),
        transcripts: HashMap::new(),
        expectations: vec![],
        never_run: Default::default(),
        stale: Default::default(),
        keys: Default::default(),
        refreshed: vec![],
        from_document: Default::default(),
        volume_outputs: HashMap::new(),
        span_files: vec![],
    })
}

/// Supplied-source pipeline seam. Project generation cannot borrow missing
/// documents from disk. Ordinary pipelines retain their established behavior.
pub(crate) fn supplied(sources: &[(&str, &str)]) -> Result<Option<PipelineResult>> {
    let mut entry = None;
    for (name, source) in sources {
        let doc = hick_lang::parse(source)?;
        if doc
            .find_tags("project")
            .iter()
            .any(|p| p.get_attribute("entry").is_none())
        {
            if entry.replace(*name).is_some() {
                bail!("one evaluated project requires one entry Markdown");
            }
        } else if !doc.find_tags("patch").is_empty() {
            // It may be a contributing source below the declared entry.
        }
    }
    match entry {
        Some(entry) => pipeline(
            entry,
            &sources
                .iter()
                .map(|(n, s)| ((*n).into(), (*s).into()))
                .collect(),
        )
        .map(Some),
        None => {
            if let Some((name, _)) = sources.iter().find(|(_, source)| {
                hick_lang::parse(source).is_ok_and(|doc| !doc.find_tags("project").is_empty())
            }) {
                return pipeline(
                    name,
                    &sources
                        .iter()
                        .map(|(n, s)| ((*n).into(), (*s).into()))
                        .collect(),
                )
                .map(Some);
            }
            for (_, source) in sources {
                if !hick_lang::parse(source)?.find_tags("patch").is_empty() {
                    bail!("patch construction requires an explicit project entry and inventory");
                }
            }
            Ok(None)
        }
    }
}

/// Pin every output byte in memory. A refactor baseline must not retain a
/// mutable temporary-file reference that can later change under the comparison.
pub fn freeze_outputs(
    files: &HashMap<String, FileContent>,
) -> Result<HashMap<String, FileContent>> {
    files
        .iter()
        .map(|(path, content)| {
            let frozen = match content {
                FileContent::Text(text) => FileContent::Text(text.clone()),
                FileContent::Binary(data) => {
                    FileContent::Binary(BinaryData::Inline(data.to_bytes()?))
                }
            };
            Ok((path.clone(), frozen))
        })
        .collect()
}
