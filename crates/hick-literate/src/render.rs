//! Render a parsed document plus run results into the block model from
//! `docs/specs/freeform/api.md` (`GET /api/docs/:id/render`).
//!
//! This is the shared code path between the CLI's `--json` output and the
//! server's render endpoint: both call [`build_block_model`].

use std::collections::HashMap;

use hick_exec::node::FileContent;
use hick_lang::{HickDocument, HickNode, HickTag};
use hickory_executor::{TranscriptEvent, Transcripts};
use serde::Serialize;

use crate::expect::ExpectationOutcome;
use crate::tag_attr;

/// Expectation metadata attached to an exec block.
#[derive(Debug, Clone, Serialize)]
pub struct ExpectInfo {
    #[serde(rename = "match")]
    pub match_mode: String,
    pub body: String,
}

/// A `<hick:ingested>` child's own attributes, when an exec cell owns one —
/// the same fingerprint `weave_ingested_block` reads to write the woven
/// markdown's "Ingested from …" caption, surfaced here so the Document
/// view's live card can show the same fact instead of only the raw
/// `<hick:ingested>` tag sitting as unstyled text after the card.
/// See docs/guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view.md
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IngestedInfo {
    pub from: String,
    pub at: String,
    pub sha256: String,
    pub files: String,
    pub skipped: String,
}

/// One block of the document, per the v0 API contract.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Block {
    Prose {
        html: String,
        span: (usize, usize),
    },
    Exec {
        id: String,
        container: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
        command: String,
        span: (usize, usize),
        #[serde(skip_serializing_if = "Option::is_none")]
        transcript: Option<Vec<TranscriptEvent>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        expect: Option<ExpectInfo>,
        #[serde(skip_serializing_if = "Option::is_none")]
        // Boxed: `IngestedInfo`'s five `String` fields would otherwise make
        // this variant the largest in `Block` by a wide margin for the
        // common (no ingest) case every OTHER exec cell hits.
        ingested: Option<Box<IngestedInfo>>,
        status: String,
    },
    File {
        path: String,
        language: String,
        body: String,
        span: (usize, usize),
    },
    Diagram {
        renderer: String,
        /// The body with `<hick:paste>` fragments from THIS document inlined
        /// — what a notebook draws. The raw source, paste tags and all, stays
        /// in the document; a derived diagram is unreadable without this.
        body: String,
        /// Ids named by `asserts`, `#` stripped.
        asserts: Vec<String>,
        span: (usize, usize),
    },
}

/// Inputs for [`build_block_model`].
pub struct BlockModelInput<'a> {
    /// Parsed document (spans intact).
    pub doc: &'a HickDocument,
    /// Per-container transcripts, entries carrying `source_line` provenance.
    pub transcripts: &'a Transcripts,
    /// Expectation outcomes from the run (empty when not evaluated).
    pub expectations: &'a [ExpectationOutcome],
    /// Final rendered file outputs, when available (`<hick:file>` bodies).
    pub files: Option<&'a HashMap<String, FileContent>>,
    /// Cells known to have no baseline — they neither executed nor were
    /// answered from a recording — keyed by [`crate::CellId`] so a cell with
    /// no container can be named too.
    pub never_run: &'a crate::NeverRun,
    /// Cells answered from a recording whose inputs have since changed.
    pub stale: &'a std::collections::BTreeMap<crate::CellId, String>,
}

/// Build the block model for one document.
pub fn build_block_model(input: &BlockModelInput<'_>) -> Vec<Block> {
    // container name -> image, from <hick:container> declarations.
    let mut images: HashMap<String, String> = HashMap::new();
    collect_images(&input.doc.nodes, &mut images);

    let mut blocks = Vec::new();
    walk(&input.doc.nodes, input, &images, &mut blocks);
    blocks
}

fn collect_images(nodes: &[HickNode], images: &mut HashMap<String, String>) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.name == "container"
                && let (Some(name), Some(image)) = (tag_attr(tag, "name"), tag_attr(tag, "image"))
            {
                images.insert(name, image);
            }
            collect_images(&tag.children, images);
        }
    }
}

fn span_of_tag(tag: &HickTag) -> (usize, usize) {
    tag.source_span.map(|s| (s.start, s.end)).unwrap_or((0, 0))
}

/// Command text of an exec tag: all text content excluding the expect and
/// capture subtrees, which are metadata rather than command.
fn command_text(tag: &HickTag) -> String {
    fn collect(nodes: &[HickNode], out: &mut String) {
        for node in nodes {
            match node {
                HickNode::Text(t, _) => out.push_str(t),
                // `ingested` is what the cell PRODUCED, not what it runs —
                // the same exclusion `hick_exec::dag::command_text` makes,
                // and for the same reason: without it the app would show a
                // scaffolder's forty files as the command line.
                HickNode::Tag(t)
                    if t.name == "expect" || t.name == "capture" || t.name == "ingested" => {}
                HickNode::Tag(t) => collect(&t.children, out),
            }
        }
    }
    let mut out = String::new();
    collect(&tag.children, &mut out);
    out
}

fn markdown_to_html(md: &str) -> String {
    let parser = pulldown_cmark::Parser::new(md);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

fn walk(
    nodes: &[HickNode],
    input: &BlockModelInput<'_>,
    images: &HashMap<String, String>,
    blocks: &mut Vec<Block>,
) {
    for node in nodes {
        match node {
            HickNode::Text(text, span) => {
                if text.trim().is_empty() {
                    continue;
                }
                blocks.push(Block::Prose {
                    html: markdown_to_html(text),
                    span: span.map(|s| (s.start, s.end)).unwrap_or((0, 0)),
                });
            }
            HickNode::Tag(tag) => match tag.name.as_str() {
                "exec" => {
                    blocks.push(exec_block(tag, input, images));
                    // What the run produced and this document now owns sits
                    // at `exec > ingested > file`. The cell renders as a
                    // cell; the files it brought in render as the file blocks
                    // they are, right after it.
                    for child in tag.child_tags() {
                        if child.name == "ingested" {
                            walk(&child.children, input, images, blocks);
                        }
                    }
                }
                "file" => {
                    let path = tag_attr(tag, "path").unwrap_or_default();
                    let body = input
                        .files
                        .and_then(|files| files.get(&path))
                        .map(|content| match content {
                            FileContent::Text(s) => s.clone(),
                            FileContent::Binary(_) => "[binary]".to_string(),
                        })
                        .unwrap_or_else(|| tag.text_content());
                    blocks.push(Block::File {
                        language: crate::weave::extension_to_language(&path).to_string(),
                        path,
                        body,
                        span: span_of_tag(tag),
                    });
                    // Execs nested in the file still appear as blocks after it.
                    walk(&tag.children, input, images, blocks);
                }
                "diagram" => blocks.push(diagram_block(tag, input.doc)),
                "when" => walk(&tag.children, input, images, blocks),
                _ => {}
            },
        }
    }
}

fn diagram_block(tag: &HickTag, doc: &HickDocument) -> Block {
    let renderer = tag_attr(tag, "renderer").unwrap_or_else(|| "mermaid".to_string());
    let asserts = tag_attr(tag, "asserts")
        .unwrap_or_default()
        .split_whitespace()
        .map(|s| s.trim_start_matches('#').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut body = String::new();
    resolve_diagram_children(&tag.children, doc, tag.source_column, &mut body);
    Block::Diagram {
        renderer,
        body,
        asserts,
        span: span_of_tag(tag),
    }
}

/// Inline `<hick:paste>` fragments from this document into a diagram body.
///
/// The weave resolves pastes through the whole pipeline; the block model only
/// needs what a notebook can draw without running anything, so this resolves
/// the document-local `copy`/`cut` fragments and drops a paste it cannot find
/// — the picture then fails to parse in the panel, which shows the source,
/// the existing posture for a diagram that cannot draw.
fn resolve_diagram_children(
    nodes: &[HickNode],
    doc: &HickDocument,
    indent: usize,
    out: &mut String,
) {
    for node in nodes {
        match node {
            HickNode::Text(text, _) => out.push_str(&hick_lang::dedent(text, indent)),
            HickNode::Tag(tag) if tag.name == "paste" => {
                if let Some(select) = tag_attr(tag, "select")
                    && let Some(fragment) = find_fragment(&doc.nodes, &select)
                {
                    out.push_str(&fragment.text_content());
                }
            }
            HickNode::Tag(_) => {}
        }
    }
}

/// The first `copy`/`cut` fragment a paste selector (`#id`, `.class`, or a
/// comma list of those) refers to, anywhere in the document.
fn find_fragment<'a>(nodes: &'a [HickNode], selector: &str) -> Option<&'a HickTag> {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if (tag.name == "copy" || tag.name == "cut") && selector_matches(selector, tag) {
                return Some(tag);
            }
            if let Some(found) = find_fragment(&tag.children, selector) {
                return Some(found);
            }
        }
    }
    None
}

fn selector_matches(selector: &str, fragment: &HickTag) -> bool {
    let id = tag_attr(fragment, "id");
    let classes = tag_attr(fragment, "class").unwrap_or_default();
    let classes: Vec<&str> = classes.split_whitespace().collect();
    selector.split(',').map(str::trim).any(|part| {
        if let Some(rest) = part.strip_prefix('#') {
            id.as_deref() == Some(rest)
        } else if let Some(rest) = part.strip_prefix('.') {
            classes.contains(&rest)
        } else {
            false
        }
    })
}

fn exec_block(
    tag: &HickTag,
    input: &BlockModelInput<'_>,
    images: &HashMap<String, String>,
) -> Block {
    let container = tag_attr(tag, "container").unwrap_or_default();
    let line = tag.source_line;
    let id = format!("{container}:{line}");

    let expect = tag
        .child_tags()
        .find(|t| t.name == "expect")
        .map(|t| ExpectInfo {
            match_mode: tag_attr(t, "match").unwrap_or_else(|| "exact".to_string()),
            body: t.text_content(),
        });

    // Same attributes `weave_ingested_block` reads to write the woven
    // markdown's caption — only surfaced here for the live card instead.
    let ingested = tag
        .child_tags()
        .find(|t| t.name == "ingested" && t.get_attribute("key").is_none())
        .map(|t| {
            Box::new(IngestedInfo {
                from: tag_attr(t, "from").unwrap_or_default(),
                at: tag_attr(t, "at").unwrap_or_default(),
                sha256: tag_attr(t, "sha256").unwrap_or_default(),
                files: tag_attr(t, "files").unwrap_or_default(),
                skipped: tag_attr(t, "skipped").unwrap_or_default(),
            })
        });

    let entry = input
        .transcripts
        .get(&container)
        .and_then(|entries| entries.iter().find(|e| e.source_line == Some(line)));
    let transcript = entry
        .map(|e| e.events.clone())
        .filter(|events| !events.is_empty());

    let outcome = input
        .expectations
        .iter()
        .find(|o| o.container.as_deref() == Some(container.as_str()) && o.line == line);

    // Axis 1 of docs/specs/freeform/three-axes.md: recorded, stale, or
    // unrecorded — and a person must be able to tell the last two apart.
    let cell = crate::CellId::exec(&container, line);
    let status = if input.never_run.contains_key(&cell) || entry.is_none() {
        "unrecorded"
    } else if input.stale.contains_key(&cell) {
        "stale"
    } else if let Some(o) = outcome {
        if o.passed { "ok" } else { "failed" }
    } else {
        "ok"
    };

    Block::Exec {
        id,
        image: images.get(&container).cloned(),
        command: command_text(tag).trim().to_string(),
        container,
        span: span_of_tag(tag),
        transcript,
        expect,
        ingested,
        status: status.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model_of(source: &str) -> Vec<Block> {
        let doc = hick_lang::parse(source).expect("parse");
        let transcripts = Transcripts::new();
        let never_run = crate::NeverRun::new();
        let stale = std::collections::BTreeMap::new();
        build_block_model(&BlockModelInput {
            doc: &doc,
            transcripts: &transcripts,
            expectations: &[],
            files: None,
            never_run: &never_run,
            stale: &stale,
        })
    }

    // Protects docs/guarantees/authoring/a-diagram-names-what-proves-it.md:
    // the app's block model carries the diagram, ids stripped of their `#`,
    // and a derived diagram's body arrives with its paste resolved — the raw
    // source alone is a paste tag, which no renderer can draw.
    #[test]
    fn diagram_block_resolves_document_local_pastes() {
        let source = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
             <hick:copy id=\"edges\">\nflowchart TD\n  a --> b\n</hick:copy>\n\
             <hick:diagram renderer=\"mermaid\" asserts=\"#row-count #edge-count\">\n\
             <hick:paste select=\"#edges\" />\n\
             </hick:diagram>\n\
             </hick:doc>\n";
        let blocks = model_of(source);
        let diagram = blocks
            .iter()
            .find_map(|b| match b {
                Block::Diagram {
                    renderer,
                    body,
                    asserts,
                    ..
                } => Some((renderer, body, asserts)),
                _ => None,
            })
            .expect("a diagram block");
        assert_eq!(diagram.0, "mermaid");
        assert!(
            diagram.1.contains("a --> b"),
            "paste inlined: {}",
            diagram.1
        );
        assert!(!diagram.1.contains("hick:paste"));
        assert_eq!(diagram.2, &["row-count", "edge-count"]);
    }

    // Protects docs/guarantees/editor-intelligence/an-ingested-cell-names-itself-in-the-document-view.md
    #[test]
    fn an_exec_owning_an_ingested_block_carries_its_fingerprint() {
        let source = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="c">
scaffold
<hick:ingested from="#scaffold" sha256="abc123def456" at="2026-09-01" files="2" skipped="5">
<hick:file path="app/Program.cs">content</hick:file>
</hick:ingested>
</hick:exec>
</hick:doc>
"##;
        let blocks = model_of(source);
        let ingested = blocks
            .iter()
            .find_map(|b| match b {
                Block::Exec { ingested, .. } => ingested.as_ref(),
                _ => None,
            })
            .expect("the exec block carries ingested info");
        assert_eq!(ingested.from, "#scaffold");
        assert_eq!(ingested.sha256, "abc123def456");
        assert_eq!(ingested.at, "2026-09-01");
        assert_eq!(ingested.files, "2");
        assert_eq!(ingested.skipped, "5");
    }

    #[test]
    fn an_ordinary_exec_carries_no_ingested_info() {
        let source = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
             <hick:exec container=\"c\">\necho hi\n</hick:exec>\n\
             </hick:doc>\n";
        let blocks = model_of(source);
        let ingested = blocks.iter().find_map(|b| match b {
            Block::Exec { ingested, .. } => Some(ingested.clone()),
            _ => None,
        });
        assert_eq!(
            ingested,
            Some(None),
            "a plain exec block has no ingested info"
        );
    }

    #[test]
    fn unresolvable_paste_is_dropped_not_echoed() {
        let source = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
             <hick:diagram>\n<hick:paste select=\"#gone\" />\n</hick:diagram>\n\
             </hick:doc>\n";
        let blocks = model_of(source);
        let body = blocks
            .iter()
            .find_map(|b| match b {
                Block::Diagram { body, .. } => Some(body),
                _ => None,
            })
            .expect("a diagram block");
        assert!(!body.contains("hick:paste"));
    }
}
