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
        status: String,
    },
    File {
        path: String,
        language: String,
        body: String,
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

/// Command text of an exec tag: all text content excluding expect subtrees.
fn command_text(tag: &HickTag) -> String {
    fn collect(nodes: &[HickNode], out: &mut String) {
        for node in nodes {
            match node {
                HickNode::Text(t, _) => out.push_str(t),
                HickNode::Tag(t) if t.name == "expect" => {}
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
                "exec" => blocks.push(exec_block(tag, input, images)),
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
                "when" => walk(&tag.children, input, images, blocks),
                _ => {}
            },
        }
    }
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
        .find(|o| o.container == container && o.line == line);

    let status = if input
        .never_run
        .contains_key(&crate::CellId::exec(&container, line))
        || entry.is_none()
    {
        "never-run"
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
        status: status.to_string(),
    }
}
