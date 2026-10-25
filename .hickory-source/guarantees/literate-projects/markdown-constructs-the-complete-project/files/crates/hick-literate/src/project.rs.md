<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/src/project.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Rebuild the program from its Markdown source](../../../../contract.md)

Captured SHA-256: `fae313292476050db6b2d208bb18375402118f8900eefeaab1aa6661a1d8c858`.

<book:entry path="crates/hick-literate/src/project.rs">
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
    if evaluated
        .new
        .entries()
        .keys()
        .any(|path| path.starts_with(hick_lang::EDITOR_BUFFER_PREFIX))
    {
        bail!(
            "Editor-only entries cannot be run or saved as filesystem outputs; inspect them with the captured changeset evaluator."
        );
    }
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
                origin: origin_kind(
                    sources,
                    &origin.document,
                    origin.source,
                    SourceSpan::new(origin.source.0, origin.source.1, line, col),
                )?,
            });
        }
        provenance_maps.insert(path.clone(), map);
    }
    let mut transcripts: hickory_executor::Transcripts = HashMap::new();
    let mut expectations = vec![];
    let mut keys = BTreeMap::new();
    let mut from_document = std::collections::BTreeSet::new();
    for record in &evaluated.recordings {
        let cell = if record.kind == "agent" {
            crate::CellId::containerless(record.line)
        } else {
            crate::CellId::exec(&record.container, record.line)
        };
        keys.insert(cell.clone(), record.key.clone());
        from_document.insert(cell.clone());
        transcripts
            .entry(record.container.clone())
            .or_default()
            .push(hickory_executor::ExecTranscriptEntry {
                commands: record.commands.clone(),
                output: record.output.clone(),
                source_line: Some(record.line),
                events: vec![],
            });
        let doc = hick_lang::parse(&sources[&record.document])?;
        for spec in crate::expect::collect_expectations(&doc.nodes).map_err(anyhow::Error::msg)? {
            if spec.cell == cell {
                expectations.push(crate::expect::evaluate(
                    &spec,
                    &record.document,
                    &record.output,
                ));
            }
        }
    }
    Ok(PipelineResult {
        evaluated_project: Some(evaluated),
        files,
        provenance_maps,
        containers: HashMap::new(),
        volume_provenance: HashMap::new(),
        resource_stats: HashMap::new(),
        transcripts,
        expectations,
        never_run: Default::default(),
        stale: Default::default(),
        keys,
        refreshed: vec![],
        from_document,
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
    let entry = entry.or_else(|| {
        sources
            .iter()
            .find(|(_, source)| {
                hick_lang::parse(source).is_ok_and(|doc| !doc.find_tags("project").is_empty())
            })
            .map(|(name, _)| *name)
    });
    if let Some(entry) = entry {
        return named_pipeline(entry, sources).map(Some);
    }
    for (_, source) in sources {
        if !hick_lang::parse(source)?.find_tags("patch").is_empty() {
            bail!("patch construction requires an explicit project entry and inventory");
        }
    }
    Ok(None)
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

/// Host source identities may be absolute. Rebase supplied names without reading
/// disk, then restore native source destinations on the legacy lineage boundary.
fn named_pipeline(entry: &str, sources: &[(&str, &str)]) -> Result<PipelineResult> {
    let path = std::path::Path::new(entry);
    if !path.is_absolute() {
        return pipeline(
            entry,
            &sources
                .iter()
                .map(|(n, s)| ((*n).into(), (*s).into()))
                .collect(),
        );
    }
    let root = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Project entry needs a directory"))?;
    let entry = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 project entry"))?;
    let mut relative = BTreeMap::new();
    for (name, source) in sources {
        let path = std::path::Path::new(name);
        let path = if path.is_absolute() {
            path.strip_prefix(root).map_err(|_| {
                anyhow::anyhow!("Supplied project source is outside its root: {name}")
            })?
        } else {
            path
        };
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 project source"))?
            .replace('\\', "/");
        if relative
            .insert(path.clone(), (*source).to_owned())
            .is_some()
        {
            bail!("Duplicate supplied project source: {path}");
        }
    }
    let mut result = pipeline(entry, &relative)?;
    result.span_files = relative
        .keys()
        .filter(|n| n.as_str() != entry)
        .map(|n| root.join(n).display().to_string())
        .collect();
    for map in result.provenance_maps.values_mut() {
        let mut updated = ProvenanceMap::new();
        for span in map.spans() {
            let mut span = span.clone();
            if let SourceOrigin::Literal { file, .. } | SourceOrigin::Ingested { file, .. } =
                &mut span.origin
            {
                *file = Arc::from(root.join(file.as_ref()).to_string_lossy().as_ref());
            }
            updated.push(span);
        }
        *map = updated;
    }
    Ok(result)
}

fn origin_kind(
    sources: &BTreeMap<String, String>,
    document: &str,
    range: (usize, usize),
    span: SourceSpan,
) -> Result<SourceOrigin> {
    let source = &sources[document];
    let doc = hick_lang::parse(source)?;
    let bom = usize::from(source.starts_with('\u{feff}')) * 3;
    for tag in doc.all_tags() {
        if tag.name != "ingested"
            || tag.source_span.is_none_or(|s| s.end + bom > range.0)
            || tag.close_span.is_none_or(|s| range.1 > s.start + bom)
        {
            continue;
        }
        if tag.get_attribute("key").is_none() {
            return Ok(SourceOrigin::Ingested {
                file: Arc::from(document),
                span,
                run: Arc::from(tag.get_attribute("sha256").unwrap_or("")),
            });
        }
        if let Some(cell) = doc
            .all_tags()
            .into_iter()
            .find(|cell| cell.child_tags().any(|child| std::ptr::eq(child, tag)))
        {
            if cell.name == "exec" {
                return Ok(SourceOrigin::Exec {
                    container: Arc::from(cell.get_attribute("container").unwrap_or("")),
                    tag_line: cell.source_line,
                });
            }
            return Ok(SourceOrigin::Synthetic);
        }
    }
    Ok(SourceOrigin::Literal {
        file: Arc::from(document),
        span,
    })
}
</book:entry>

</book:doc>
