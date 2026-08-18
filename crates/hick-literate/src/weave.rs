//! Weave (literate programming) output support.
//!
//! Processes document nodes into markdown-style literate programming output:
//! prose text, code fences for file content, variable interpolation, and
//! exec transcript rendering.

use std::collections::HashMap;
use std::sync::Arc;

use hick_exec::node::{
    InsertionPoint, Node, ProvenanceTransformNode, SourceOrigin, SpanNode, StringNode,
};
use hick_exec::state::MultiDocumentState;
use hick_lang::{HickDocument, HickNode, dedent};

use hick_handlers::{ProcessingContext, ProcessingPhase, TagRegistry, TagResult, TranscriptEntry};

use crate::tag_attr;
use crate::text::interpolate_path;

/// Map file extension to markdown code fence language identifier.
pub(crate) fn extension_to_language(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "rust",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "cs" => "csharp",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "sh" | "bash" => "bash",
        "xml" | "hick" => "xml",
        "html" | "htm" => "html",
        "css" => "css",
        "go" => "go",
        "java" => "java",
        "sql" => "sql",
        "md" | "markdown" => "markdown",
        "rb" => "ruby",
        "php" => "php",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "zig" => "zig",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        "gradle" => "groovy",
        _ => "",
    }
}

/// The file a span's offsets index: the spliced file it was stamped with,
/// else the document being woven. Attributing an included span to the
/// including document is how a reverse edit lands in the wrong file.
fn origin_file(doc_path: &str, span_files: &[Arc<str>], span: &hick_lang::SourceSpan) -> Arc<str> {
    span.file_id
        .and_then(|id| span_files.get(usize::from(id)).cloned())
        .unwrap_or_else(|| Arc::from(doc_path))
}

/// Process document nodes for weave output.
///
/// Iterates through nodes and emits:
/// - `HickNode::Text` → prose (with substitutions applied, dedented)
/// - `<hick:file>` → heading + fenced code block (unless `doc-hidden="true"`)
/// - `<hick:diagram>` → fenced block tagged with its `renderer`
/// - `<hick:claim>` → an attribution line, then the prose unchanged
/// - `<hick:transcript>` → its derived speaker turns
/// - `<hick:said>` → `**Who** (time): what they said`
/// - `<hick:val>` → resolved variable value (via registry)
/// - `<hick:when>` → recursively process children (already filtered)
// The `span_files` threading (include splicing) pushed these over the
// clippy arg limit; a param-struct refactor belongs to that change, not here.
#[allow(clippy::too_many_arguments)]
fn process_weave_content(
    nodes: &[HickNode],
    weave_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    state: &Arc<MultiDocumentState>,
    indent: usize,
    registry: &TagRegistry,
    doc_path: &str,
    span_files: &[Arc<str>],
) {
    for node in nodes {
        match node {
            HickNode::Text(text, span) => {
                let dedented = dedent(text, indent);
                // Prose in the weave IS the document's prose, so it carries
                // the span it came from — which is what makes the woven file
                // editable, the same way a `hick:file` output is. Without it
                // every byte of the weave is "synthetic": no ribbons, and an
                // edit that can only be refused.
                //
                // The lineage layer keeps the origin only when the span's
                // length matches the text's, so a dedent that actually
                // removed something degrades to synthetic on its own rather
                // than claiming a mapping that would put edits in the wrong
                // place.
                match span {
                    Some(span) => weave_ip.add(Arc::new(SpanNode::new(
                        dedented,
                        SourceOrigin::Literal {
                            file: origin_file(doc_path, span_files, span),
                            span: *span,
                        },
                    ))),
                    None => weave_ip.add(Arc::new(StringNode::new(dedented))),
                }
            }
            HickNode::Tag(tag) => {
                process_weave_tag(
                    tag,
                    weave_ip,
                    transcripts,
                    state,
                    registry,
                    doc_path,
                    span_files,
                );
            }
        }
    }
}

/// Process a single tag for weave output.
fn process_weave_tag(
    tag: &hick_lang::HickTag,
    weave_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    state: &Arc<MultiDocumentState>,
    registry: &TagRegistry,
    doc_path: &str,
    span_files: &[Arc<str>],
) {
    match tag.name.as_str() {
        "file" => {
            // Check for doc-hidden attribute
            let doc_hidden = tag_attr(tag, "doc-hidden")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);

            if !doc_hidden {
                let raw_path = tag_attr(tag, "path").unwrap_or_default();
                let path = interpolate_path(&raw_path, state);
                let language = extension_to_language(&path);

                // Emit heading
                weave_ip.add(Arc::new(StringNode::new(format!("\n### `{path}`\n\n"))));

                // Emit opening code fence
                weave_ip.add(Arc::new(StringNode::new(format!("```{language}\n"))));

                // Process file content with dedenting based on file tag's indentation
                process_file_children_to_weave(
                    &tag.children,
                    weave_ip,
                    transcripts,
                    state,
                    tag.source_column,
                    registry,
                    doc_path,
                    span_files,
                );

                // Emit closing code fence
                weave_ip.add(Arc::new(StringNode::new("```\n".to_string())));
            }
        }
        // A diagram is prose that happens to be a picture: it weaves to the
        // fenced block its renderer expects, so the woven markdown renders on
        // GitHub, in an editor preview, and anywhere else a reader opens it —
        // with no hick installed and no notebook.
        //
        // Children go through the same path `hick:file` uses, so a
        // `<hick:paste>` inside a diagram resolves. That is what lets a
        // picture be DERIVED — the edge list can come from the cell that
        // proved it rather than from someone's memory of it.
        "diagram" => {
            let renderer = tag_attr(tag, "renderer").unwrap_or_else(|| "mermaid".to_string());
            weave_ip.add(Arc::new(StringNode::new(format!("\n```{renderer}\n"))));
            process_file_children_to_weave(
                &tag.children,
                weave_ip,
                transcripts,
                state,
                tag.source_column,
                registry,
                doc_path,
                span_files,
            );
            weave_ip.add(Arc::new(StringNode::new("```\n".to_string())));
        }
        // A claim is somebody's assertion about an assertion. It weaves an
        // attribution line — who, on what standing, about what scope — and
        // then the prose UNCHANGED.
        //
        // The prose is deliberately not blockquoted or otherwise rewritten.
        // Prefixing every line would make the woven bytes differ in length
        // from the spans they came from, and the lineage layer drops an origin
        // whose span no longer matches — so the claim's own text would stop
        // being editable and stop carrying ribbons, which is a worse trade than
        // a less emphatic rendering. The visible distinction that matters lives
        // in the app; the woven markdown gets an honest header.
        //
        // See `docs/specs/freeform/provenance-and-standing.md`.
        "claim" => {
            let by = tag_attr(tag, "by").unwrap_or_default();
            let standing = tag_attr(tag, "standing").unwrap_or_default();
            let scope = tag_attr(tag, "scope").unwrap_or_default();

            let who = if by.is_empty() { "unattributed" } else { &by };
            let mut label = String::new();
            if !standing.is_empty() {
                label.push_str(&standing);
            }
            if !scope.is_empty() {
                if !label.is_empty() {
                    label.push_str(" · ");
                }
                label.push_str(&scope);
            }
            let header = if label.is_empty() {
                format!("\n> **{who}** claims:\n\n")
            } else {
                format!("\n> **{who}** — {label}\n\n")
            };
            weave_ip.add(Arc::new(StringNode::new(header)));

            process_weave_content(
                &tag.children,
                weave_ip,
                transcripts,
                state,
                tag.source_column,
                registry,
                doc_path,
                span_files,
            );
        }
        // A transcript weaves as the meeting it is: its derived turns, in
        // order. The raw block stays in the `.hick` file and never reaches the
        // markdown, because what a reader wants is the conversation, not the
        // cue timings a recorder emitted.
        //
        // A transcript whose format was not recognised still has its raw text
        // child, which falls through the same path and weaves as prose — the
        // material survives even when the structure could not be derived.
        "transcript" => {
            process_weave_content(
                &tag.children,
                weave_ip,
                transcripts,
                state,
                tag.source_column,
                registry,
                doc_path,
                span_files,
            );
        }
        // One speaker turn: who, when, and what they said, on one line.
        //
        // Like `hick:claim`, the spoken text passes through unrewritten so it
        // keeps the span it came from and stays editable.
        "said" => {
            let by = tag_attr(tag, "by").unwrap_or_default();
            let at = tag_attr(tag, "at").unwrap_or_default();
            let header = match (by.is_empty(), at.is_empty()) {
                (true, true) => "\n".to_string(),
                (true, false) => format!("\n*{at}* — "),
                (false, true) => format!("\n**{by}**: "),
                (false, false) => format!("\n**{by}** ({at}): "),
            };
            weave_ip.add(Arc::new(StringNode::new(header)));
            process_weave_content(
                &tag.children,
                weave_ip,
                transcripts,
                state,
                tag.source_column,
                registry,
                doc_path,
                span_files,
            );
            weave_ip.add(Arc::new(StringNode::new("\n".to_string())));
        }
        "when" => {
            // When tags have already been filtered, so just process children
            // Use the when tag's indentation for dedenting child content
            process_weave_content(
                &tag.children,
                weave_ip,
                transcripts,
                state,
                tag.source_column,
                registry,
                doc_path,
                span_files,
            );
        }
        _ => {
            // Delegate to registry for content handlers (val, etc.)
            if let Some(handler) = registry.find(&tag.name)
                && handler.phase() == ProcessingPhase::Content
            {
                let ctx = ProcessingContext {
                    state,
                    transcripts,
                    indent: 0,
                    registry: Some(registry),
                    context: None,
                    source_file: None,
                    span_files: &[],
                };
                match handler.process(tag, &ctx) {
                    Ok(TagResult::Node(n)) => weave_ip.add(n),
                    Ok(TagResult::Nodes(ns)) => {
                        for n in ns {
                            weave_ip.add(n);
                        }
                    }
                    _ => {}
                }
            }
            // Skip declaration-phase tags (var, copy, cut, substitute, etc.)
        }
    }
}

/// Process file children for weave output (similar to process_file_children but outputs to weave).
///
/// The `indent` parameter specifies how many leading spaces to strip from each
/// line of text content (typically the source_column of the parent file tag).
// The `span_files` threading (include splicing) pushed these over the
// clippy arg limit; a param-struct refactor belongs to that change, not here.
#[allow(clippy::too_many_arguments)]
fn process_file_children_to_weave(
    children: &[HickNode],
    weave_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    state: &Arc<MultiDocumentState>,
    indent: usize,
    registry: &TagRegistry,
    doc_path: &str,
    span_files: &[Arc<str>],
) {
    let ctx = ProcessingContext {
        state,
        transcripts,
        indent,
        registry: Some(registry),
        context: None,
        source_file: None,
        span_files: &[],
    };

    for child in children {
        match child {
            HickNode::Text(text, span) => {
                let dedented = dedent(text, indent);
                // The fenced copy of a `hick:file` block in the woven markdown
                // is the same text as the block, so it carries the same span.
                // Without this the `### orders.py` section has no ribbon and
                // no edit can be traced out of it, while the file itself —
                // identical bytes — has both.
                //
                // Dedenting changes the text, and the lineage layer keeps an
                // origin only when the span's length matches what was emitted,
                // so an indented block degrades to synthetic on its own rather
                // than claiming a mapping that would land edits elsewhere.
                match span {
                    Some(span) => weave_ip.add(Arc::new(SpanNode::new(
                        dedented,
                        SourceOrigin::Literal {
                            file: origin_file(doc_path, span_files, span),
                            span: *span,
                        },
                    ))),
                    None => weave_ip.add(Arc::new(StringNode::new(dedented))),
                }
            }
            HickNode::Tag(child_tag) => {
                if let Some(handler) = registry.find(&child_tag.name) {
                    match handler.process(child_tag, &ctx) {
                        Ok(TagResult::Node(n)) => weave_ip.add(n),
                        Ok(TagResult::Nodes(ns)) => {
                            for n in ns {
                                weave_ip.add(n);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Process weave output for all documents if weave is enabled.
pub(crate) fn process_weave_output(
    documents: &[(&str, HickDocument)],
    transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    state: &Arc<MultiDocumentState>,
    registry: &TagRegistry,
) {
    // Find the weave path (use the first document that has one)
    let weave_path = documents
        .iter()
        .find_map(|(_, doc)| doc.weave_path.as_ref());

    if let Some(weave_path) = weave_path {
        let raw_ip = Arc::new(InsertionPoint::new());

        // Process all document nodes for weave output
        // Use 0 indent for top-level content (direct children of <hick:doc>)
        for (name, doc) in documents {
            let span_files = crate::span_file_table(doc);
            process_weave_content(
                &doc.nodes,
                &raw_ip,
                transcripts,
                state,
                0,
                registry,
                name,
                &span_files,
            );
        }

        raw_ip.close();

        // The SEGMENTED transform, the same one the `hick:file` path uses.
        //
        // A whole-output transform produces one node with no origin, which
        // erases every span underneath it: that is why the woven markdown was
        // entirely "synthetic" — no ribbons, and an edit the server could only
        // refuse. Segmenting keeps each untouched piece attached to the prose
        // it came from, and marks only what a substitution actually replaced.
        let subs_state = state.clone();
        let transform: Arc<dyn Node> = Arc::new(ProvenanceTransformNode::new(
            raw_ip,
            move |text| crate::apply_substitutions_segmented_to_transform(text, &subs_state),
            "substitute",
        ));

        let weave_ip = Arc::new(InsertionPoint::new());
        weave_ip.add(transform);
        weave_ip.close();
        state.add_file_output(weave_path.clone(), weave_ip);
    }
}
