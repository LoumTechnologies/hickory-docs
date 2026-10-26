<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Rebuild the program from its Markdown source

For: Maintainers moving this project to a fresh checkout or reviewing a source reorganization.

Copy project.md and its declared Markdown inputs into an empty folder. Generating them must recover the same application files without borrowing the old checkout.

## What must happen

Given an entry document and its explicitly declared Markdown closure, pure generation starts from an empty tree and constructs the current project using only those inputs. Missing sources, missing or cyclic predecessors, conflicting unordered writes, and unresolved runtime inputs fail explicitly. Declared fragments and valid retained recordings may supply bytes; ambient caches and earlier Git revisions may not. Completed New combines the independent contributions; a patch sees only its declared predecessors.


## Limits of the evidence

This promises reconstruction, not that the reconstructed application is correct. The full CLI lineage-conversion path has a known scale problem; the successful whole-repository witness uses the existing evaluator and native writer directly.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Fold the later explicit-fragment and recording amendment into the current statement; retain the earlier exclusions as history.

Implementation review: partially verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-9f0933415862"></a>
## `crates/hick-lang/src/project/discovery.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Rebuild the program from its Markdown source](contract.md).

<a id="f-9f0933415862-load_project_sources"></a>
### `load_project_sources`

<storycode:copy id="f-9f0933415862-load_project_sources">
use super::validate_project_path;
use crate::parse;
use std::collections::BTreeMap;

/// Discover only explicitly declared Markdown inputs. The caller provides a
/// read-only revision reader; discovery never consults sibling code or config.
pub fn load_project_sources(
    entry: &str,
    read: &impl Fn(&str) -> Result<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut sources = BTreeMap::new();
    load(entry, read, &mut sources)?;
    Ok(sources)
}
</storycode:copy>
<a id="f-9f0933415862-load"></a>
### `load`

Follow only declared project sources, include edges and upstream edges. Remember loaded paths before recursing so a repeated include does not cause another filesystem read; require included inputs to be Markdown.

<storycode:copy id="f-9f0933415862-load">
fn load(
    path: &str,
    read: &impl Fn(&str) -> Result<String, String>,
    sources: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    validate_project_path(path)?;
    if !path.ends_with(".md") {
        return Err(format!("source closure requires Markdown: {path}"));
    }
    if sources.contains_key(path) {
        return Ok(());
    }
    let source = read(path)?;
    let doc = parse(&source).map_err(|e| e.to_string())?;
    sources.insert(path.into(), source);
    for tag in doc.tags() {
        let names: Vec<_> = match tag.name.as_str() {
            "project" if tag.get_attribute("entry").is_some() => {
                vec![tag.get_attribute("entry").unwrap()]
            }
            "project" => tag
                .get_attribute("sources")
                .unwrap_or("")
                .split_whitespace()
                .collect(),
            "include" | "upstream" => {
                vec![tag.get_attribute("file").ok_or("include requires file")?]
            }
            _ => vec![],
        };
        for name in names {
            validate_project_path(name)?;
            let name = path
                .rsplit_once('/')
                .map_or(name.to_owned(), |(dir, _)| format!("{dir}/{name}"));
            load(&name, read, sources)?;
        }
    }
    Ok(())
}
</storycode:copy>

### Assemble `crates/hick-lang/src/project/discovery.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/discovery.rs"><storycode:paste select="#f-9f0933415862-load_project_sources"/><storycode:paste select="#f-9f0933415862-load"/></storycode:file>

<a id="output-f58a59d2516c"></a>
## `crates/hick-lang/src/project/recorded.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Rebuild the program from its Markdown source](contract.md).

<a id="f-f58a59d2516c-RecordedCell"></a>
### `RecordedCell`

Kept outputs are data from the Markdown closure, never cache lookups.

<storycode:copy id="f-f58a59d2516c-RecordedCell">
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
</storycode:copy>
<a id="f-f58a59d2516c-Recordings"></a>
### `Recordings`

<storycode:copy id="f-f58a59d2516c-Recordings">
pub(super) type Recordings = BTreeMap<(String, usize), RecordedCell>;

</storycode:copy>
<a id="f-f58a59d2516c-load"></a>
### `load`

<storycode:copy id="f-f58a59d2516c-load">
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
</storycode:copy>
<a id="f-f58a59d2516c-command"></a>
### `command`

<storycode:copy id="f-f58a59d2516c-command">
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
    if matches!(
        tag.get_attribute("language"),
        Some("python" | "python3" | "py")
    ) {
        return Ok(crate::notebook::python_command(&text));
    }
    Ok(text)
}
</storycode:copy>
<a id="f-f58a59d2516c-render"></a>
### `render`

<storycode:copy id="f-f58a59d2516c-render">
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
    if matches!(show, ExecShow::All | ExecShow::Output)
        && !record.output.is_empty()
        && crate::notebook::render_output(&record.output) == record.output
    {
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
</storycode:copy>

### Assemble `crates/hick-lang/src/project/recorded.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/project/recorded.rs"><storycode:paste select="#f-f58a59d2516c-RecordedCell"/><storycode:paste select="#f-f58a59d2516c-Recordings"/><storycode:paste select="#f-f58a59d2516c-load"/><storycode:paste select="#f-f58a59d2516c-command"/><storycode:paste select="#f-f58a59d2516c-render"/></storycode:file>

## Test the behavior: `closure_constructs_atomic_endpoints_and_can_be_reorganized`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-closure_constructs_atomic_endpoints_and_can_be_reorganized"></a>
### `closure_constructs_atomic_endpoints_and_can_be_reorganized`

<storycode:copy id="f-005c1e9344c7-closure_constructs_atomic_endpoints_and_can_be_reorganized">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn closure_constructs_atomic_endpoints_and_can_be_reorganized() {
    let sources = BTreeMap::from([
        (
            "project.md".into(),
            include_str!("fixtures/literate-project/project.md").into(),
        ),
        (
            "library.md".into(),
            include_str!("fixtures/literate-project/library.md").into(),
        ),
    ]);
    let project = evaluate_project("project.md", &sources, &ProjectTree::default()).unwrap();
    assert!(project.old.entries().is_empty());
    assert_eq!(
        &*project.new.get("app.ts").unwrap().bytes,
        b"import { welcome } from './library';\nconsole.log(welcome('Hickory'));\n"
    );
    assert_eq!(
        &*project.new.get("library.ts").unwrap().bytes,
        b"export function welcome(name: string) { return 'Hello ' + name; }\n"
    );
    assert_eq!(
        &*project.local_states["caller"]
            .1
            .get("library.ts")
            .unwrap()
            .bytes,
        b"export function greet() { return 'Hello'; }\n"
    );
    let folded = generate("<hick:project/>\n<hick:file path=\"app.ts\">\nimport { welcome } from './library';\nconsole.log(welcome('Hickory'));\n</hick:file>\n<hick:file path=\"library.ts\">\nexport function welcome(name: string) { return 'Hello ' + name; }\n</hick:file>").unwrap();
    assert!(folded.new.compare(&project.new).is_empty());
    let mut incomplete = sources.clone();
    incomplete.remove("library.md");
    assert!(
        evaluate_project("project.md", &incomplete, &ProjectTree::default())
            .unwrap_err()
            .contains("missing library.md")
    );
    assert_eq!(ProjectTree::default().compare(&project.new).len(), 2);
}
</storycode:copy>

## Test the behavior: `refuses_unordered_transforms_hidden_inputs_and_processes`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-refuses_unordered_transforms_hidden_inputs_and_processes"></a>
### `refuses_unordered_transforms_hidden_inputs_and_processes`

<storycode:copy id="f-005c1e9344c7-refuses_unordered_transforms_hidden_inputs_and_processes">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn refuses_unordered_transforms_hidden_inputs_and_processes() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"x\">one\n</hick:file><hick:patch id=\"p\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-one\n+two\n</hick:patch>";
    assert!(generate(source).unwrap_err().contains("unordered"));
    assert!(generate(&source.replace("id=\"p\"", "id=\"p\" after=\"base\"")).is_ok());
    for (extra, diagnostic) in [
        (
            "<hick:exec container=\"c\">echo x</hick:exec>",
            "requires execution",
        ),
        ("<hick:upstream src=\"secret.md\"/>", "unsupported"),
        ("<hick:when test=\"unknown\"/>", "required"),
    ] {
        assert!(
            generate(&format!("<hick:project/>{extra}"))
                .unwrap_err()
                .contains(diagnostic)
        );
    }
    assert!(generate("<hick:project/><hick:file id=\"a\" after=\"b\" path=\"x\">x</hick:file><hick:file id=\"b\" after=\"a\" path=\"y\">y</hick:file>").is_err());
}
</storycode:copy>

## Test the behavior: `contributors_can_route_to_the_completed_project_from_markdown_alone`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-contributors_can_route_to_the_completed_project_from_markdown_alone"></a>
### `contributors_can_route_to_the_completed_project_from_markdown_alone`

<storycode:copy id="f-005c1e9344c7-contributors_can_route_to_the_completed_project_from_markdown_alone">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn contributors_can_route_to_the_completed_project_from_markdown_alone() {
    let sources = BTreeMap::from([
        (
            "project.md".into(),
            include_str!("fixtures/literate-project/project.md").into(),
        ),
        (
            "library.md".into(),
            include_str!("fixtures/literate-project/library.md").into(),
        ),
    ]);
    let root = evaluate_project("project.md", &sources, &ProjectTree::default()).unwrap();
    let member = evaluate_project("library.md", &sources, &ProjectTree::default()).unwrap();
    assert!(root.new.compare(&member.new).is_empty());
    let discovered = load_project_sources("library.md", &|path| {
        sources
            .get(path)
            .cloned()
            .ok_or_else(|| format!("missing {path}"))
    })
    .unwrap();
    assert_eq!(discovered, sources);
    let cycle = BTreeMap::from([
        ("a.md".into(), "<hick:project entry=\"b.md\"/>".into()),
        ("b.md".into(), "<hick:project entry=\"a.md\"/>".into()),
    ]);
    assert!(
        evaluate_project("a.md", &cycle, &ProjectTree::default())
            .unwrap_err()
            .contains("routing cycle")
    );
}

</storycode:copy>

## Test the behavior: `local_predecessor_state_does_not_depend_on_independent_act_names`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-local_predecessor_state_does_not_depend_on_independent_act_names"></a>
### `local_predecessor_state_does_not_depend_on_independent_act_names`

<storycode:copy id="f-005c1e9344c7-local_predecessor_state_does_not_depend_on_independent_act_names">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn local_predecessor_state_does_not_depend_on_independent_act_names() {
    for independent in ["before", "zzz_after"] {
        let source = format!(
            "<hick:project/><hick:file id=\"base\" path=\"x\">a\n</hick:file><hick:file id=\"{independent}\" path=\"y\">independent</hick:file><hick:patch id=\"patch\" after=\"base\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n</hick:patch>"
        );
        let project = generate(&source).unwrap();
        assert_eq!(project.local_states["patch"].0.entries().len(), 1);
        assert!(project.local_states["patch"].0.get("y").is_none());
        assert_eq!(project.new.entries().len(), 2);
        assert_eq!(project.new.get("x").unwrap().bytes.as_ref(), b"b\n");
    }
}
</storycode:copy>

## Test the behavior: `explicit_fragment_closure_is_ordered_private_and_upstream_is_read_only`

This is the maintained test fragment from `crates/hick-lang/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-005c1e9344c7-explicit_fragment_closure_is_ordered_private_and_upstream_is_read_only"></a>
### `explicit_fragment_closure_is_ordered_private_and_upstream_is_read_only`

<storycode:copy id="f-005c1e9344c7-explicit_fragment_closure_is_ordered_private_and_upstream_is_read_only">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[test]
fn explicit_fragment_closure_is_ordered_private_and_upstream_is_read_only() {
    let sources=BTreeMap::from([
        ("project.md".into(),"<hick:project sources=\"member.md private.md\"/><hick:upstream file=\"up.md\"/><hick:copy id=\"first\" class=\"parts\">a\n</hick:copy><hick:file id=\"out\" path=\"app.ts\"><hick:paste select=\".parts\"/></hick:file>".into()),
        ("member.md".into(),"<hick:copy id=\"second\" class=\"parts\">b\n</hick:copy>".into()),
        ("private.md".into(),"<hick:private/><hick:copy id=\"secret\" class=\"parts\">hidden\n</hick:copy>".into()),
        ("up.md".into(),"<hick:upstream file=\"base.md\"/><hick:copy id=\"third\" class=\"parts\">d\n</hick:copy><hick:file path=\"not-imported\">no</hick:file>".into()),
        ("base.md".into(),"<hick:copy id=\"base\" class=\"parts\">c\n</hick:copy>".into()),
    ]);
    let p = evaluate_project("project.md", &sources, &ProjectTree::default()).unwrap();
    assert_eq!(p.new.get("app.ts").unwrap().bytes.as_ref(), b"a\nc\nb\nd\n");
    assert!(p.new.get("not-imported").is_none());
    assert!(p.inventory.read_only.contains("base.md"));
    let origin = p
        .new
        .get("app.ts")
        .unwrap()
        .origins
        .iter()
        .find(|o| o.document == "base.md")
        .unwrap();
    assert!(
        hick_lang::reverse_project_edit(
            "project.md",
            &sources,
            &ProjectTree::default(),
            &p.new,
            "app.ts",
            origin.output,
            "x\n"
        )
        .unwrap_err()
        .contains("base.md is read-only")
    );
    let mut cycle = sources.clone();
    cycle.insert("base.md".into(), "<hick:upstream file=\"up.md\"/>".into());
    assert!(
        evaluate_project("project.md", &cycle, &ProjectTree::default())
            .unwrap_err()
            .contains("cycle")
    );
}

</storycode:copy>

<a id="output-83f7e6f1577e"></a>
## `crates/hick-literate/src/project.rs`

Related contracts: [Rebuild the program from its Markdown source](contract.md).

<a id="f-83f7e6f1577e-file_content_entry"></a>
### `file_content_entry`

Portable project evaluation shared by pipeline callers and native services.

<storycode:copy id="f-83f7e6f1577e-file_content_entry">
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

</storycode:copy>
<a id="f-83f7e6f1577e-pipeline"></a>
### `pipeline`

<storycode:copy id="f-83f7e6f1577e-pipeline">
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

</storycode:copy>
<a id="f-83f7e6f1577e-supplied"></a>
### `supplied`

Supplied-source pipeline seam. Project generation cannot borrow missing
documents from disk. Ordinary pipelines retain their established behavior.

<storycode:copy id="f-83f7e6f1577e-supplied">
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

</storycode:copy>
<a id="f-83f7e6f1577e-freeze_outputs"></a>
### `freeze_outputs`

Pin every output byte in memory. A refactor baseline must not retain a
mutable temporary-file reference that can later change under the comparison.

<storycode:copy id="f-83f7e6f1577e-freeze_outputs">
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

</storycode:copy>
<a id="f-83f7e6f1577e-named_pipeline"></a>
### `named_pipeline`

Host source identities may be absolute. Rebase supplied names without reading
disk, then restore native source destinations on the legacy lineage boundary.

<storycode:copy id="f-83f7e6f1577e-named_pipeline">
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

</storycode:copy>
<a id="f-83f7e6f1577e-origin_kind"></a>
### `origin_kind`

<storycode:copy id="f-83f7e6f1577e-origin_kind">
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
</storycode:copy>

### Assemble `crates/hick-literate/src/project.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/project.rs"><storycode:paste select="#f-83f7e6f1577e-file_content_entry"/><storycode:paste select="#f-83f7e6f1577e-pipeline"/><storycode:paste select="#f-83f7e6f1577e-supplied"/><storycode:paste select="#f-83f7e6f1577e-freeze_outputs"/><storycode:paste select="#f-83f7e6f1577e-named_pipeline"/><storycode:paste select="#f-83f7e6f1577e-origin_kind"/></storycode:file>

## Test the behavior: `pipeline_uses_the_same_pure_complete_evaluator`

This is the maintained test fragment from `crates/hick-literate/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d5caa71df0d7-pipeline_uses_the_same_pure_complete_evaluator"></a>
### `pipeline_uses_the_same_pure_complete_evaluator`

<storycode:copy id="f-d5caa71df0d7-pipeline_uses_the_same_pure_complete_evaluator">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[tokio::test]
async fn pipeline_uses_the_same_pure_complete_evaluator() {
    let source = "<hick:project/><hick:file id=\"base\" path=\"x\">a\n</hick:file><hick:patch after=\"base\">\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n</hick:patch>";
    let run = hick_literate::run_pipeline_weave(&[("project.md", source)], &[], None)
        .await
        .unwrap();
    assert_eq!(run.files["x"].as_text(), Some("b\n"));
    assert_eq!(run.evaluated_project.unwrap().new.entries().len(), 1);
    assert!(run.transcripts.is_empty());
    assert!(
        hick_literate::run_pipeline_weave(&[("p.md", "<hick:patch>diff</hick:patch>")], &[], None)
            .await
            .is_err()
    );
}

</storycode:copy>

## Test the behavior: `kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins`

This is the maintained test fragment from `crates/hick-literate/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d5caa71df0d7-kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins"></a>
### `kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins`

<storycode:copy id="f-d5caa71df0d7-kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[tokio::test]
async fn kept_generator_replay_uses_shared_fingerprint_formatting_expectations_and_origins() {
    let key = hick_lang::exec_cache_key(
        "alpine",
        "",
        "printf hello",
        &[],
        &hick_lang::inputs_digest(&[]),
        &[],
    );
    let source = format!(
        "<hick:project/><hick:container name=\"c\" image=\"alpine\"/><hick:file path=\"out.txt\"><hick:exec container=\"c\" show=\"output\"><hick:cmd>printf hello</hick:cmd><hick:ingested key=\"{key}\">\nhello</hick:ingested><hick:expect match=\"exact\">hello</hick:expect></hick:exec></hick:file>"
    );
    let result = hick_literate::run_pipeline_weave(&[("project.md", &source)], &[], None)
        .await
        .unwrap();
    assert_eq!(result.files["out.txt"].as_text(), Some("hello\n"));
    assert!(result.expectations.iter().all(|e| e.passed));
    assert_eq!(result.expectations.len(), 1);
    assert!(
        result.provenance_maps["out.txt"]
            .spans()
            .iter()
            .any(|s| matches!(s.origin, hick_exec::node::SourceOrigin::Exec { .. }))
    );
    // Literal command wrappers and bare native command text describe the same
    // kept run. A nested paste still needs resolution and must not be ignored.
    let bare = source.replace("<hick:cmd>printf hello</hick:cmd>", "printf hello");
    let bare_result = hick_literate::run_pipeline_weave(&[("project.md", &bare)], &[], None)
        .await
        .unwrap();
    assert!(hick_literate::equiv::compare_outputs(&result.files, &bare_result.files).is_empty());
    let unresolved = source.replace(
        "printf hello</hick:cmd>",
        "<hick:paste select=\"#command\"/></hick:cmd>",
    );
    assert!(
        hick_literate::run_pipeline_weave(&[("project.md", &unresolved)], &[], None)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("requires execution/resolution")
    );
    let bad = source.replace("printf hello", "printf goodbye");
    assert!(
        hick_literate::run_pipeline_weave(&[("project.md", &bad)], &[], None)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("stale")
    );
    let bad = source.replace("match=\"exact\">hello", "match=\"exact\">wrong");
    let result = hick_literate::run_pipeline_weave(&[("project.md", &bad)], &[], None)
        .await
        .unwrap();
    assert!(!result.expectations[0].passed);
    let sources = std::collections::BTreeMap::from([("project.md".into(), source)]);
    let p = hick_lang::evaluate_project("project.md", &sources, &hick_lang::ProjectTree::default())
        .unwrap();
    assert!(
        hick_lang::reverse_project_edit(
            "project.md",
            &sources,
            &hick_lang::ProjectTree::default(),
            &p.new,
            "out.txt",
            (0, 5),
            "other"
        )
        .unwrap_err()
        .contains("synthetic")
    );
}

</storycode:copy>

## Test the behavior: `owned_ingest_is_editable_source_without_claiming_generator_execution`

This is the maintained test fragment from `crates/hick-literate/tests/project.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-d5caa71df0d7-owned_ingest_is_editable_source_without_claiming_generator_execution"></a>
### `owned_ingest_is_editable_source_without_claiming_generator_execution`

<storycode:copy id="f-d5caa71df0d7-owned_ingest_is_editable_source_without_claiming_generator_execution">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[tokio::test]
async fn owned_ingest_is_editable_source_without_claiming_generator_execution() {
    let source = format!(
        "<hick:project/><hick:exec container=\"sdk\"><hick:ingested sha256=\"{}\"><hick:file id=\"owned\" path=\"app.ts\">x\n</hick:file></hick:ingested></hick:exec>",
        "a".repeat(64)
    );
    let result = hick_literate::run_pipeline_weave(&[("project.md", &source)], &[], None)
        .await
        .unwrap();
    assert!(result.transcripts.is_empty());
    assert!(matches!(
        result.provenance_maps["app.ts"].spans()[0].origin,
        hick_exec::node::SourceOrigin::Ingested { .. }
    ));
}
</storycode:copy>

## Test the behavior: `fresh_git_free_folder_runs_the_markdown_closure`

This is the maintained test fragment from `crates/hickory-cli/tests/project_revision.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e0a4f3a47871-fresh_git_free_folder_runs_the_markdown_closure"></a>
### `fresh_git_free_folder_runs_the_markdown_closure`

<storycode:copy id="f-e0a4f3a47871-fresh_git_free_folder_runs_the_markdown_closure">
// Guarantee: docs/guarantees/literate-projects/markdown-constructs-the-complete-project.md
#[tokio::test]
async fn fresh_git_free_folder_runs_the_markdown_closure() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("project.md"),
        include_str!("../../hick-lang/tests/fixtures/literate-project/project.md"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("library.md"),
        include_str!("../../hick-lang/tests/fixtures/literate-project/library.md"),
    )
    .unwrap();
    let run = hickory_cli::run_doc(
        &dir.path().join("project.md"),
        &[],
        hickory_cli::RunMode::Weave,
        hickory_cli::ExecutorChoice::Local,
    )
    .await
    .unwrap();
    assert!(!dir.path().join("app.ts").exists());
    hickory_cli::write_outputs(&run, None).unwrap();
    assert!(hickory_cli::check_failures(&run, None).unwrap().is_empty());
    std::fs::write(dir.path().join("migration.sql"), "required source").unwrap();
    assert_eq!(hickory_cli::check_failures(&run, None).unwrap().len(), 1);
    assert!(!dir.path().join(".git").exists());
    assert!(!dir.path().join(".hick-cache").exists());
}
</storycode:copy>

## Implementation review

Discovery reads the declared closure. The evaluator orders construction acts, builds predecessor states, and combines their outputs. The native writer receives the completed tree only after evaluation. This separation lets a reviewer check equality before any file is written.

### Implementation

- [crates/hick-lang-wasm/src/lib.rs](../../../mechanisms/crates/hick-lang-wasm/src/lib.rs.md#output-9a9e430ed3f3)
- [crates/hick-lang/src/lib.rs](../../../mechanisms/crates/hick-lang/src/lib.rs.md#output-0cf7120750a0)
- [crates/hick-lang/src/project/discovery.rs](contract.md#output-9f0933415862)
- [crates/hick-lang/src/project/evaluate.rs](../../../mechanisms/crates/hick-lang/src/project/evaluate.rs.md#output-4957ad4b2f9a)
- [crates/hick-lang/src/project/recorded.rs](contract.md#output-f58a59d2516c)
- [crates/hick-lang/src/recording.rs](../../../mechanisms/crates/hick-lang/src/recording.rs.md#output-0df1b34a0343)
- [crates/hick-literate/src/project.rs](contract.md#output-83f7e6f1577e)
- [crates/hick-project-env/src/tree.rs](../../../mechanisms/crates/hick-project-env/src/tree.rs.md#output-c594be7b78fc)
- [crates/hickory-cli/src/project_documents.rs](../../../mechanisms/crates/hickory-cli/src/project_documents.rs.md#output-7cd7736e0d32)

### Tests and fixtures

- [crates/hick-lang/tests/project.rs](../../../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7)
- [crates/hick-literate/tests/project.rs](../../../mechanisms/crates/hick-literate/tests/project.rs.md#output-d5caa71df0d7)
- [crates/hickory-cli/tests/project_revision.rs](../../../mechanisms/crates/hickory-cli/tests/project_revision.rs.md#output-e0a4f3a47871)

### Focused witnesses

- `closure_constructs_atomic_endpoints_and_can_be_reorganized` in `crates/hick-lang/tests/project.rs`.
- `refuses_unordered_transforms_hidden_inputs_and_processes` in `crates/hick-lang/tests/project.rs`.

### Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
