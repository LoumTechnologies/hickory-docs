//! Render a parsed document plus run results into the block model from
//! `docs/specs/freeform/api.md` (`GET /api/docs/:id/render`).
//!
//! This is the shared code path between the CLI's `--json` output and the
//! server's render endpoint: both call [`build_block_model`].
//!
//! The elements themselves — what an `exec`, a `file`, a `diagram` and the
//! prose between them look like as blocks — are declared here as
//! [`hick_blocks::Element`]s over the run's facts ([`BlockModelInput`]) and
//! registered once in [`registry`]. The walk is the registry's; this module
//! only says what each element means.

use std::collections::HashMap;

use hick_blocks::{
    ActionError, ActionOutcome, AttrSpec, Descend, Element, Registry, attr, span_of,
};
use hick_exec::node::FileContent;
use hick_lang::{HickDocument, HickNode, HickTag};
use hickory_executor::Transcripts;
use serde::Serialize;

use crate::expect::ExpectationOutcome;
use crate::tag_attr;

pub use hick_blocks::Block;

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

/// The facts a run produced, which the elements render from.
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
    registry().blocks(input.doc, input)
}

/// The elements the app draws, over a run's facts.
///
/// Built per call: it is five zero-sized elements and a closure, and a
/// registry generic over a borrowed context cannot be a static.
pub fn registry<'a>() -> Registry<BlockModelInput<'a>> {
    let mut registry = Registry::new();
    registry
        .register(ExecElement)
        .register(FileElement)
        .register(DiagramElement)
        .register(WhenElement)
        .text(Box::new(|text, span, _| {
            (!text.trim().is_empty())
                .then(|| Block::new("prose", span).with("html", markdown_to_html(text)))
        }));
    registry
}

/// The vocabulary the app draws, for anything that needs it as data.
pub fn describe_elements() -> Vec<hick_blocks::ElementDescription> {
    registry().describe()
}

// ---------------------------------------------------------------------------
// The elements
// ---------------------------------------------------------------------------

/// `<hick:exec>`: a cell. Renders as the command, its transcript and its
/// verdict; then shows the files it `ingested`, which are file blocks of
/// their own right after it.
struct ExecElement;

impl<'a> Element<BlockModelInput<'a>> for ExecElement {
    fn name(&self) -> &'static str {
        "exec"
    }

    fn attributes(&self) -> &'static [AttrSpec] {
        const ATTRS: &[AttrSpec] = &[
            AttrSpec::required("container", "the container the command runs in"),
            AttrSpec::optional(
                "image",
                "the image that creates the container, on its first cell",
            ),
            AttrSpec::optional("mount", "volumes mounted into the container, `name:path,…`"),
            AttrSpec::optional("timeout", "how long the cell may run"),
            AttrSpec::optional("show", "which parts of the transcript the weave shows"),
        ];
        ATTRS
    }

    fn descend(&self) -> Descend {
        // What the run produced and this document now owns sits at
        // `exec > ingested > file`. The cell renders as a cell; the files it
        // brought in render as the file blocks they are, right after it.
        Descend::Named(&["ingested"])
    }

    fn render(&self, tag: &HickTag, input: &BlockModelInput<'a>) -> Option<Block> {
        Some(exec_block(tag, input))
    }

    fn actions(&self) -> &'static [&'static str] {
        &["run"]
    }

    /// `run`: ask the host to execute this cell. The element does not run
    /// it — the host does, through the executor and the run record the Run
    /// button already uses — which is what keeps a render endpoint from
    /// ever being a way to execute a document.
    fn act(
        &self,
        action: &str,
        tag: &HickTag,
        _: &BlockModelInput<'a>,
        _: serde_json::Value,
    ) -> Result<ActionOutcome, ActionError> {
        match action {
            "run" => Ok(ActionOutcome::Run {
                cells: vec![cell_id(tag)],
            }),
            other => Err(ActionError::Unknown {
                element: "exec",
                action: other.to_string(),
            }),
        }
    }
}

/// The id an exec block carries: `container:line`.
fn cell_id(tag: &HickTag) -> String {
    format!(
        "{}:{}",
        tag_attr(tag, "container").unwrap_or_default(),
        tag.source_line
    )
}

/// `<hick:file>`: a generated file. Its body is what the run wove when the
/// run is in hand, else the source text; the cells nested in it are blocks
/// after it.
struct FileElement;

impl<'a> Element<BlockModelInput<'a>> for FileElement {
    fn name(&self) -> &'static str {
        "file"
    }

    fn attributes(&self) -> &'static [AttrSpec] {
        const ATTRS: &[AttrSpec] = &[
            AttrSpec::required(
                "path",
                "where the file is written, relative to the document",
            ),
            AttrSpec::optional(
                "language",
                "how the body highlights, when the path does not say",
            ),
        ];
        ATTRS
    }

    fn descend(&self) -> Descend {
        Descend::All
    }

    fn render(&self, tag: &HickTag, input: &BlockModelInput<'a>) -> Option<Block> {
        let path = attr(tag, "path").unwrap_or_default();
        let body = input
            .files
            .and_then(|files| files.get(&path))
            .map(|content| match content {
                FileContent::Text(s) => s.clone(),
                FileContent::Binary(_) => "[binary]".to_string(),
            })
            .unwrap_or_else(|| tag.text_content());
        Some(
            Block::new("file", span_of(tag))
                .with("language", crate::weave::extension_to_language(&path))
                .with("path", path)
                .with("body", body),
        )
    }
}

/// `<hick:diagram>`: a picture drawn from text, with this document's
/// pasted fragments inlined so a notebook can draw it without running
/// anything.
struct DiagramElement;

impl<'a> Element<BlockModelInput<'a>> for DiagramElement {
    fn name(&self) -> &'static str {
        "diagram"
    }

    fn attributes(&self) -> &'static [AttrSpec] {
        const ATTRS: &[AttrSpec] = &[
            AttrSpec::optional("renderer", "`mermaid` (the default) or `graph`"),
            AttrSpec::optional("asserts", "the `#id`s of the cells that prove this picture"),
        ];
        ATTRS
    }

    fn render(&self, tag: &HickTag, input: &BlockModelInput<'a>) -> Option<Block> {
        Some(diagram_block(tag, input.doc))
    }
}

/// `<hick:when>`: a gate. Draws nothing; what it gates is drawn.
struct WhenElement;

impl<'a> Element<BlockModelInput<'a>> for WhenElement {
    fn name(&self) -> &'static str {
        "when"
    }

    fn attributes(&self) -> &'static [AttrSpec] {
        const ATTRS: &[AttrSpec] = &[AttrSpec::optional(
            "feature",
            "the condition under which the content applies",
        )];
        ATTRS
    }

    fn descend(&self) -> Descend {
        Descend::All
    }

    fn render(&self, _: &HickTag, _: &BlockModelInput<'a>) -> Option<Block> {
        None
    }
}

// ---------------------------------------------------------------------------
// How each element reads its tag
// ---------------------------------------------------------------------------

/// Container name → image, from `<hick:container>` declarations anywhere in
/// the document.
fn images_of(doc: &HickDocument) -> HashMap<String, String> {
    fn collect(nodes: &[HickNode], images: &mut HashMap<String, String>) {
        for node in nodes {
            if let HickNode::Tag(tag) = node {
                if tag.name == "container"
                    && let (Some(name), Some(image)) =
                        (tag_attr(tag, "name"), tag_attr(tag, "image"))
                {
                    images.insert(name, image);
                }
                collect(&tag.children, images);
            }
        }
    }
    let mut images = HashMap::new();
    collect(&doc.nodes, &mut images);
    images
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

fn diagram_block(tag: &HickTag, doc: &HickDocument) -> Block {
    let renderer = tag_attr(tag, "renderer").unwrap_or_else(|| "mermaid".to_string());
    let asserts: Vec<String> = tag_attr(tag, "asserts")
        .unwrap_or_default()
        .split_whitespace()
        .map(|s| s.trim_start_matches('#').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut body = String::new();
    resolve_diagram_children(&tag.children, doc, tag.source_column, &mut body);
    Block::new("diagram", span_of(tag))
        .with("renderer", renderer)
        .with("body", body)
        .with("asserts", asserts)
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

fn exec_block(tag: &HickTag, input: &BlockModelInput<'_>) -> Block {
    let container = tag_attr(tag, "container").unwrap_or_default();
    let line = tag.source_line;
    let id = cell_id(tag);

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
        .map(|t| IngestedInfo {
            from: tag_attr(t, "from").unwrap_or_default(),
            at: tag_attr(t, "at").unwrap_or_default(),
            sha256: tag_attr(t, "sha256").unwrap_or_default(),
            files: tag_attr(t, "files").unwrap_or_default(),
            skipped: tag_attr(t, "skipped").unwrap_or_default(),
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

    Block::new("exec", span_of(tag))
        .with("id", id)
        .with("image", images_of(input.doc).get(&container))
        .with("command", command_text(tag).trim())
        .with("container", container)
        .with("transcript", transcript)
        .with("expect", expect)
        .with("ingested", ingested)
        .with("status", status)
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
            .find(|b| b.kind == "diagram")
            .expect("a diagram block");
        assert_eq!(diagram.str_prop("renderer"), Some("mermaid"));
        let body = diagram.str_prop("body").unwrap();
        assert!(body.contains("a --> b"), "paste inlined: {body}");
        assert!(!body.contains("hick:paste"));
        assert_eq!(
            diagram.prop("asserts").unwrap(),
            &serde_json::json!(["row-count", "edge-count"])
        );
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
            .find(|b| b.kind == "exec")
            .and_then(|b| b.prop("ingested"))
            .expect("the exec block carries ingested info");
        assert_eq!(ingested["from"], "#scaffold");
        assert_eq!(ingested["sha256"], "abc123def456");
        assert_eq!(ingested["at"], "2026-09-01");
        assert_eq!(ingested["files"], "2");
        assert_eq!(ingested["skipped"], "5");
    }

    #[test]
    fn an_ordinary_exec_carries_no_ingested_info() {
        let source = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
             <hick:exec container=\"c\">\necho hi\n</hick:exec>\n\
             </hick:doc>\n";
        let blocks = model_of(source);
        let exec = blocks
            .iter()
            .find(|b| b.kind == "exec")
            .expect("an exec block");
        assert_eq!(
            exec.prop("ingested"),
            None,
            "a plain exec block has no ingested info"
        );
    }

    // The wire shape the app has always read: flat, `kind` first, `span` as
    // a pair, an absent prop absent rather than null.
    #[test]
    fn a_block_serialises_to_the_v0_contract() {
        let source = "# Hi\n<hick:exec container=\"c\">\necho hi\n</hick:exec>\n";
        let blocks = model_of(source);
        let json = serde_json::to_value(&blocks).unwrap();
        assert_eq!(json[0]["kind"], "prose");
        assert!(json[0]["html"].as_str().unwrap().contains("<h1>Hi</h1>"));
        let exec = &json[1];
        assert_eq!(exec["kind"], "exec");
        assert_eq!(exec["span"], serde_json::json!([5, 30]));
        assert_eq!(exec["id"], "c:2");
        assert_eq!(exec["container"], "c");
        assert_eq!(exec["command"], "echo hi");
        assert_eq!(exec["status"], "unrecorded");
        for absent in ["image", "transcript", "expect", "ingested"] {
            assert!(
                exec.get(absent).is_none(),
                "{absent} should be absent, not null"
            );
        }
    }

    #[test]
    fn the_registry_describes_the_vocabulary_it_draws() {
        let names: Vec<&str> = describe_elements().iter().map(|d| d.name).collect();
        assert_eq!(names, vec!["diagram", "exec", "file", "when"]);
    }

    #[test]
    fn unresolvable_paste_is_dropped_not_echoed() {
        let source = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\
             <hick:diagram>\n<hick:paste select=\"#gone\" />\n</hick:diagram>\n\
             </hick:doc>\n";
        let blocks = model_of(source);
        let body = blocks
            .iter()
            .find(|b| b.kind == "diagram")
            .and_then(|b| b.str_prop("body"))
            .expect("a diagram block");
        assert!(!body.contains("hick:paste"));
    }
}
