//! Kept outputs are data from the Markdown closure, never cache lookups.
use crate::{ByteOrigin, ExecShow, HickNode, HickTag};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct RecordedCell {
    pub kind: String,
    pub document: String,
    pub line: usize,
    pub container: String,
    pub commands: Vec<String>,
    pub key: String,
    pub output: String,
    pub source_offset: usize,
}
pub(super) type Recordings = BTreeMap<(String, usize), RecordedCell>;

pub(super) fn load(
    sources: &BTreeMap<String, String>,
    documents: &BTreeSet<String>,
) -> Result<Recordings, String> {
    let mut containers = BTreeMap::new();
    let mut unavailable = BTreeSet::new();
    for document in documents {
        let doc = crate::parse(&sources[document]).map_err(|e| e.to_string())?;
        for tag in doc.all_tags() {
            if tag.name == "container" {
                let name = tag.get_attribute("name").ok_or("Container requires name")?;
                let image = tag.get_attribute("image").unwrap_or("alpine");
                if containers
                    .insert(name.to_owned(), image.to_owned())
                    .is_some_and(|old| old != image)
                {
                    return Err(format!("Conflicting container images: {name}"));
                }
                if tag
                    .attributes
                    .iter()
                    .any(|(name, _)| !matches!(name.as_str(), "name" | "image"))
                {
                    unavailable.insert(name.to_owned());
                }
            }
            if tag.name == "allow" {
                unavailable.insert(tag.get_attribute("container").unwrap_or("").to_owned());
            }
        }
    }
    let mut recordings = BTreeMap::new();
    for document in documents {
        let doc = crate::parse(&sources[document]).map_err(|e| e.to_string())?;
        let bom = usize::from(sources[document].starts_with('\u{feff}')) * 3;
        let mut previous: BTreeMap<String, String> = BTreeMap::new();
        for tag in doc
            .all_tags()
            .into_iter()
            .filter(|t| matches!(t.name.as_str(), "exec" | "agent"))
        {
            // Owned ingested files are maintained source; no stdout recording
            // is needed to construct their bytes. They make no execution claim.
            if tag
                .child_tags()
                .any(|t| t.name == "ingested" && t.get_attribute("key").is_none())
            {
                continue;
            }
            let records: Vec<_> = tag
                .child_tags()
                .filter(|t| t.name == "ingested" && t.get_attribute("key").is_some())
                .collect();
            if records.is_empty() {
                return Err(format!(
                    "{document}: {} at line {} requires execution; keep its generator result in the document before pure verification",
                    tag.name, tag.source_line
                ));
            }
            let container = tag.get_attribute("container").unwrap_or("agent").to_owned();
            if tag.get_attribute("mount").is_some() || unavailable.contains(&container) {
                return Err(format!(
                    "{document}: recorded cell at line {} requires execution/input resolution for its mounts or capabilities; pure replay cannot borrow those inputs",
                    tag.source_line
                ));
            }
            let command = command(tag)?;
            let key = if tag.name == "agent" {
                crate::agent_cache_key(
                    tag.get_attribute("model")
                        .ok_or("Recorded agent cell requires a declared model")?,
                    &command,
                )
            } else {
                let upstream: Vec<_> = previous
                    .get(&container)
                    .into_iter()
                    .map(String::as_str)
                    .collect();
                crate::exec_cache_key(
                    tag.get_attribute("image")
                        .or_else(|| containers.get(&container).map(String::as_str))
                        .unwrap_or("alpine"),
                    "",
                    &command,
                    &[],
                    &crate::inputs_digest(&[]),
                    &upstream,
                )
            };
            let matches: Vec<_> = records
                .into_iter()
                .filter(|r| r.get_attribute("key") == Some(key.as_str()))
                .collect();
            let [record] = matches.as_slice() else {
                return Err(format!(
                    "{document}: stale or ambiguous kept recording at line {}; execution required",
                    tag.source_line
                ));
            };
            let raw = record.text_content();
            let output = raw.strip_prefix('\n').unwrap_or(&raw).to_owned();
            if let Some(hash) = record.get_attribute("sha256")
                && format!("{:x}", Sha256::digest(output.as_bytes())) != hash
            {
                return Err(format!(
                    "{document}: kept output hash differs at line {}; execution required",
                    tag.source_line
                ));
            }
            let offset = record
                .children
                .iter()
                .find_map(|n| {
                    if let HickNode::Text(_, Some(span)) = n {
                        Some(span.start)
                    } else {
                        None
                    }
                })
                .ok_or("Kept recording requires a verbatim source span")?
                + bom
                + usize::from(raw.starts_with('\n'));
            recordings.insert(
                (
                    document.clone(),
                    tag.source_span.ok_or("Cell needs a source span")?.start,
                ),
                RecordedCell {
                    kind: tag.name.clone(),
                    document: document.clone(),
                    line: tag.source_line,
                    container: container.clone(),
                    commands: vec![command.trim().to_owned()],
                    key: key.clone(),
                    output,
                    source_offset: offset,
                },
            );
            previous.insert(container, key);
        }
    }
    Ok(recordings)
}
fn command(tag: &HickTag) -> Result<String, String> {
    if let Some(command) = tag.get_attribute("cmd") {
        return Ok(command.into());
    }
    if let Some(prompt) = tag.child_tags().find(|t| t.name == "prompt") {
        return Ok(prompt.text_content());
    }
    let mut text = String::new();
    for node in &tag.children {
        match node {
            HickNode::Text(source, _) => text.push_str(source),
            // Native command collection visits literal command wrappers. They
            // need no external resolution, but nested operations still do.
            HickNode::Tag(t)
                if t.name == "cmd"
                    && t.children.iter().all(|n| matches!(n, HickNode::Text(_, _))) =>
            {
                text.push_str(&t.text_content());
            }
            HickNode::Tag(t)
                if matches!(
                    t.name.as_str(),
                    "ingested" | "expect" | "capture" | "sample"
                ) => {}
            HickNode::Tag(t) => {
                return Err(format!(
                    "Cell command {} requires execution/resolution before portable replay",
                    t.name
                ));
            }
        }
    }
    Ok(text)
}
pub(super) fn render(
    tag: &HickTag,
    document: &str,
    recordings: &Recordings,
) -> Result<(String, Vec<ByteOrigin>), String> {
    let record = recordings
        .get(&(
            document.into(),
            tag.source_span.ok_or("Cell needs a source span")?.start,
        ))
        .ok_or_else(|| format!("{document}: cell output requires execution"))?;
    let show = match tag.get_attribute("show") {
        Some("output") => ExecShow::Output,
        Some("command") => ExecShow::Command,
        Some("none") => ExecShow::None,
        _ => ExecShow::All,
    };
    let text = crate::render_transcript_entry(&record.commands, &record.output, show);
    let mut origins = vec![];
    if matches!(show, ExecShow::All | ExecShow::Output) && !record.output.is_empty() {
        let start = crate::render_transcript_entry(&record.commands, "", show).len();
        origins.push(ByteOrigin {
            output: (start, start + record.output.len()),
            document: document.into(),
            source: (
                record.source_offset,
                record.source_offset + record.output.len(),
            ),
        });
    }
    Ok((text, origins))
}
