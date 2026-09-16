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
        // `.csproj` and friends are XML with a different name on them — a
        // document that ingests `dotnet new` gets one whether or not it asked.
        // Kept in step with `ALIASES` in apps/web/src/editor/languages.ts, so
        // the fence in the woven markdown and the colouring in the app agree
        // about what a file is.
        "xml" | "hick" | "csproj" | "props" | "targets" | "xaml" | "xsd" => "xml",
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

/// File extensions a reader's markdown viewer draws as a picture.
///
/// Deliberately the same list as `isPicturePath` in
/// `apps/web/src/editor/hickDoc.ts`, and it has to stay that way: the app
/// decides which file blocks render as a picture, this decides which ones
/// weave as one, and a document where those two disagree shows a chart in one
/// place and a code fence in the other.
const PICTURE_EXTENSIONS: [&str; 7] = ["svg", "png", "jpg", "jpeg", "gif", "webp", "avif"];

/// Whether a `<hick:file path=…>` writes something a reader can look at
/// rather than read.
pub(crate) fn is_picture_path(path: &str) -> bool {
    let ext = path
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    PICTURE_EXTENSIONS.contains(&ext.as_str())
}

/// The file a span's offsets index: the spliced file it was stamped with,
/// else the document being woven. Attributing an included span to the
/// including document is how a reverse edit lands in the wrong file.
fn origin_file(doc_path: &str, span_files: &[Arc<str>], span: &hick_lang::SourceSpan) -> Arc<str> {
    span.file_id
        .and_then(|id| span_files.get(usize::from(id)).cloned())
        .unwrap_or_else(|| Arc::from(doc_path))
}

/// Tags that never contribute a single byte to the weave — every one either
/// has no registered weave handler at all (`container`, `volume`, `feature`,
/// `needs`, `allow`: pipeline/DAG declarations with nothing to render) or a
/// handler whose only job is registering content for a LATER `hick:paste`
/// (`copy`, `cut`: `TagResult::Declaration`, verified by reading
/// `hick-handlers/src/handlers/copy.rs` directly). A closed, verified list
/// rather than a general "did this add anything" check — `InsertionPoint`
/// exposes no count to test that against from outside, and this covers every
/// tag actually observed to cause the problem below.
const SILENT_DECLARATION_TAGS: [&str; 7] = [
    "copy",
    "cut",
    "container",
    "volume",
    "feature",
    "needs",
    "allow",
];

/// Process document nodes for weave output.
///
/// Iterates through nodes and emits:
/// - `HickNode::Text` → prose (with substitutions applied, dedented)
/// - `<hick:file>` → heading + fenced code block (unless `doc-hidden="true"`)
/// - `<hick:diagram>` → fenced block tagged with its `renderer`
/// - `<hick:math>` → a `$$…$$` display-math block
/// - `<hick:table>` → a markdown table (its CSV is also written when it
///   carries a `path`)
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
    // Set when the PREVIOUS node was a silent declaration tag: the next Text
    // node strips up to one blank line's worth of ITS OWN leading newlines
    // before being added. A silent tag contributes zero bytes but the prose
    // around it still carries the paragraph break on both sides — "prose
    // A.\n\n<hick:copy…/>\n\nprose B" weaves as "prose A.\n\n\n\nprose B"
    // (three blank lines) where a human who left the tag out entirely would
    // have written "prose A.\n\nprose B" (one). Stripping is applied to the
    // FOLLOWING text only — the node already added for the PRECEDING text
    // can't be edited after the fact — which is sufficient: one side owning
    // the single blank line the pair should have is enough to remove the
    // redundant three.
    let mut pending_blank_strip = false;
    for node in nodes {
        match node {
            HickNode::Text(text, span) => {
                let (text, span): (&str, Option<hick_lang::SourceSpan>) = if pending_blank_strip {
                    strip_up_to_one_blank_line(text, span.as_ref())
                } else {
                    (text.as_str(), *span)
                };
                pending_blank_strip = false;
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
                            file: origin_file(doc_path, span_files, &span),
                            span,
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
                pending_blank_strip = SILENT_DECLARATION_TAGS.contains(&tag.name.as_str());
            }
        }
    }
}

/// Strip up to one blank line's worth of LEADING newlines from `text` — at
/// most `"\n\n"`, never more, and never a `"\n"` that would remove a real
/// paragraph break rather than a redundant extra one. Same span-adjustment
/// shape as [`crate::strip_opening_break`]; unlike it, this only ever takes
/// from the front, never crosses a `\r\n`, and stops as soon as there is
/// nothing left to take or two newlines have been removed.
fn strip_up_to_one_blank_line<'a>(
    text: &'a str,
    span: Option<&hick_lang::SourceSpan>,
) -> (&'a str, Option<hick_lang::SourceSpan>) {
    let mut rest = text;
    let mut taken = 0usize;
    let mut lines_taken = 0usize;
    while taken < 2 {
        if let Some(r) = rest.strip_prefix('\n') {
            rest = r;
            taken += 1;
            lines_taken += 1;
        } else {
            break;
        }
    }
    if taken == 0 {
        return (text, span.copied());
    }
    let moved = span.map(|s| hick_lang::SourceSpan {
        start: s.start + taken,
        end: s.end,
        start_line: s.start_line + lines_taken,
        start_col: 0,
        file_id: s.file_id,
    });
    (rest, moved)
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

                // A file that writes a PICTURE weaves as the picture.
                //
                // The heading-and-fence below exists to show a file's source,
                // and a chart has no source a reader wants: fencing an SVG
                // puts forty kilobytes of markup in the middle of a document.
                // So an author had to write `doc-hidden="true"` and then a
                // markdown `![…](chart.svg)` by hand — which works, and which
                // means the document names the same picture twice. In the app
                // that shows up as the chart drawn twice, once for the block
                // and once for the line beneath it.
                //
                // Emitting the image here is what lets both go away: no
                // `doc-hidden`, no hand-written line, one picture in the
                // woven markdown and one in the editor. The alt text is the
                // path, which is what a reader needs when the image does not
                // load.
                if is_picture_path(&path) {
                    weave_ip.add(Arc::new(StringNode::new(format!("\n![{path}]({path})\n"))));
                    return;
                }

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
                    None,
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
            // A graph scene is JSON with positions; the weave downgrades it to
            // a mermaid fence so the woven markdown still renders everywhere.
            // Positions are dropped, which is honest — markdown has nowhere to
            // keep them. A body that does not parse weaves as its JSON, so the
            // reader sees what is there instead of nothing; the warning about
            // it is emitted at validation time, beside the asserts warnings.
            if renderer == crate::scene::GRAPH_RENDERER {
                let body = resolved_scene_body(tag, state);
                match crate::scene::parse_scene(&body) {
                    Ok(scene) => {
                        // The woven picture IS the drawn picture: an SVG the
                        // weave itself draws, with the author's positions,
                        // sizes, shapes, and colours — a mermaid downgrade
                        // re-laid the diagram out and looked like a different
                        // drawing. Named by content, so an unchanged scene is
                        // an unchanged file.
                        let svg = crate::scene::to_svg(&scene);
                        let name = format!("diagram-{}.svg", &crate::cache::sha256_hex(&body)[..8]);
                        let file_ip = Arc::new(InsertionPoint::new());
                        file_ip.add(Arc::new(StringNode::new(svg)));
                        file_ip.close();
                        state.add_file_output(name.clone(), file_ip);
                        weave_ip.add(Arc::new(StringNode::new(format!("\n![diagram]({name})\n"))));
                    }
                    Err(_) => {
                        let body = body.trim_end();
                        weave_ip.add(Arc::new(StringNode::new(format!(
                            "\n```json\n{body}\n```\n"
                        ))));
                    }
                }
                return;
            }
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
                None,
            );
            weave_ip.add(Arc::new(StringNode::new("```\n".to_string())));
        }
        // A table is a dataset that is also prose, and both halves matter.
        //
        // The CONTENT is CSV, because that is what a spreadsheet exports,
        // what a query writes, and what a script reads — and when the tag
        // carries a `path`, those exact bytes are written there, the same way
        // a `hick:file` body is (see the file-output pass in lib.rs). The
        // WEAVE is a markdown table, because a reader opening the document
        // wants the data, not the delimiters.
        //
        // Weaving the CSV in a fenced block instead would show a reader the
        // commas; storing markdown instead would leave a file nothing else
        // can read.
        "table" => {
            let delimiter = crate::csv_table::delimiter_of(tag_attr(tag, "delimiter").as_deref());
            let header = crate::csv_table::header_of(tag_attr(tag, "header").as_deref());
            let rows = crate::csv_table::parse_csv(&tag.text_content(), delimiter);
            let table = crate::csv_table::to_markdown(&rows, header);
            if !table.is_empty() {
                weave_ip.add(Arc::new(StringNode::new(format!("\n{table}\n"))));
            }
        }
        // Display maths weaves to `$$…$$`, which is what GitHub, every editor
        // preview, and every static-site generator already renders. The same
        // reasoning as the diagram above: the woven markdown has to be worth
        // reading with no hick installed, and a fenced ```latex block would
        // show a reader the source of an equation instead of the equation.
        //
        // Children go through `hick:file`'s path so a `<hick:paste>` inside
        // the maths resolves — an equation whose coefficients came from the
        // cell that computed them is the whole point of putting one here.
        "math" => {
            weave_ip.add(Arc::new(StringNode::new("\n$$\n".to_string())));
            process_file_children_to_weave(
                &tag.children,
                weave_ip,
                transcripts,
                state,
                tag.source_column,
                registry,
                doc_path,
                span_files,
                None,
            );
            weave_ip.add(Arc::new(StringNode::new("$$\n".to_string())));
        }
        // A window onto what a cell just generated.
        //
        // The point of generating code is not reading it, so a document that
        // pasted the output back in would undo the thing it exists for. But a
        // reader still needs to see WHAT the generator makes, once, to believe
        // the rules — so this shows a few lines and says where they came from.
        //
        // **The bytes live only in the weave.** Nothing is written into the
        // `.hick`, which is why a sample cannot go stale and why it costs the
        // document nothing to keep: the woven markdown is drift-checked, so a
        // sample that stopped matching its file fails `hick test` the same way
        // a changed transcript does.
        //
        // Read from disk rather than from the pipeline's produced files,
        // because the interesting case is a VOLUME output — a program wrote
        // it, and it is on disk beside the document rather than in a
        // `hick:file` the weave holds.
        "sample" => weave_sample(tag, weave_ip, state, doc_path),
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
            // What the author says this rests on — DECLARED provenance, an
            // assertion by whoever wrote the tag, so it weaves as one: a
            // trailing line that names the selectors, never a checkmark.
            if let Some(cites) = tag_attr(tag, "cites")
                && !cites.trim().is_empty()
            {
                weave_ip.add(Arc::new(StringNode::new(format!(
                    "\n*cites: {}*\n",
                    cites.trim()
                ))));
            }
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
        // What a tool outside this document wrote, and this document now
        // owns. The weave names the run before showing the files, because a
        // reader who sees forty files must not have to guess whether somebody
        // typed them. Say what it is — arrived, from that run, on that day —
        // never "generated", which would read as derived and checkable.
        "ingested" => weave_ingested_block(
            tag,
            weave_ip,
            transcripts,
            state,
            registry,
            doc_path,
            span_files,
        ),
        // A pipeline edge renders nothing: what it brought in is selectable,
        // not printed. See `resolve_includes` in `hick-lang`.
        "upstream" => {}
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
                    paste_line_indent: None,
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
            // An exec's transcript is what the registry just rendered; what
            // the run PRODUCED and this document ingested is a child of the
            // cell, and the registry never descends into one. Nesting is
            // `exec > ingested > file`, so this is the only place it weaves.
            if tag.name == "exec" {
                for child in tag.child_tags() {
                    // A sample sits under the cell that generated the file it
                    // shows, which is the whole point of it — so this is the
                    // only place it weaves.
                    if child.name == "sample" {
                        weave_sample(child, weave_ip, state, doc_path);
                    }
                    if child.name == "ingested" {
                        weave_ingested_block(
                            child,
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
            // Skip declaration-phase tags (var, copy, cut, substitute, etc.)
        }
    }
}

/// Weave one `<hick:sample>`: a window onto a few lines of what a cell
/// generated.
fn weave_sample(
    tag: &hick_lang::HickTag,
    weave_ip: &Arc<InsertionPoint>,
    state: &Arc<MultiDocumentState>,
    doc_path: &str,
) {
    let raw_path = tag_attr(tag, "path").unwrap_or_default();
    let path = interpolate_path(&raw_path, state);
    let caption = tag_attr(tag, "caption").unwrap_or_default();
    let from: usize = tag_attr(tag, "from")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let to: usize = tag_attr(tag, "to")
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);

    let base = std::path::Path::new(doc_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let full = base.join(&path);
    let language = extension_to_language(&path);

    // This run's own bytes first: an output volume is not on disk
    // until after the weave, so reading the file would show the
    // PREVIOUS run's output — and, on a first run, nothing at all.
    let produced = state.produced_file(&path);

    let mut block = String::new();
    if !caption.is_empty() {
        block.push_str(&format!("\n*{caption}*\n"));
    }
    match produced
        .ok_or(())
        .or_else(|()| std::fs::read_to_string(&full))
    {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            let last = to.min(lines.len());
            let first = from.max(1);
            if first > last {
                // Named rather than shown empty: a sample whose range
                // has slid off the end of a file it no longer matches
                // is exactly the stale illustration this element
                // exists to make impossible.
                block.push_str(&format!(
                    "\n> `{path}` has {} line(s), so lines {from}–{to} are not there \
                             any more. Re-pick the sample.\n\n",
                    lines.len()
                ));
            } else {
                block.push_str(&format!("\n```{language}\n"));
                block.push_str(&lines[first - 1..last].join("\n"));
                block.push_str(&format!(
                    "\n```\n\n<sub>{path} lines {first}–{last}, generated — \
                             shown here, not stored here.</sub>\n\n"
                ));
            }
        }
        Err(_) => block.push_str(&format!(
            "\n> `{path}` has not been generated yet, so there is nothing to show. \
                     Run the document.\n\n"
        )),
    }
    weave_ip.add(Arc::new(StringNode::new(block)));
}

/// Weave one `<hick:ingested>` block: an attribution line naming the run,
/// then each file the way `hick:file` weaves one.
///
/// `doc-hidden` is honoured per file, exactly as elsewhere — a scaffold's
/// build output is the case that wants it.
#[allow(clippy::too_many_arguments)]
fn weave_ingested_block(
    tag: &hick_lang::HickTag,
    weave_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<TranscriptEntry>>,
    state: &Arc<MultiDocumentState>,
    registry: &TagRegistry,
    doc_path: &str,
    span_files: &[Arc<str>],
) {
    // A recording the document keeps (`key=`) is evidence, not content: the
    // cell's transcript already renders it, so it weaves nothing of its own.
    if tag_attr(tag, "key").is_some() {
        return;
    }
    let Some(run) = tag_attr(tag, "sha256").filter(|v| !v.is_empty()) else {
        return;
    };
    let from = tag_attr(tag, "from").unwrap_or_default();
    let at = tag_attr(tag, "at").unwrap_or_default();
    let files = tag_attr(tag, "files").unwrap_or_default();
    let skipped = tag_attr(tag, "skipped").unwrap_or_default();
    let short: String = run.chars().take(12).collect();

    let mut line = String::from("\n*Ingested");
    if !from.is_empty() {
        line.push_str(&format!(" from `{from}`"));
    }
    if !at.is_empty() {
        line.push_str(&format!(" on {at}"));
    }
    line.push_str(&format!(" — run `{short}`"));
    if !files.is_empty() {
        line.push_str(&format!(", {files} file(s)"));
    }
    if !skipped.is_empty() && skipped != "0" {
        line.push_str(&format!(", {skipped} skipped"));
    }
    line.push_str(". These bytes came from that run, not from this document's author.*\n");
    weave_ip.add(Arc::new(StringNode::new(line)));

    let run: Arc<str> = Arc::from(run.as_str());
    for child in tag.child_tags() {
        if child.name != "file" {
            continue;
        }
        let doc_hidden = tag_attr(child, "doc-hidden")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        if doc_hidden {
            continue;
        }
        let path = interpolate_path(&tag_attr(child, "path").unwrap_or_default(), state);
        let language = extension_to_language(&path);
        weave_ip.add(Arc::new(StringNode::new(format!("\n### `{path}`\n\n"))));
        weave_ip.add(Arc::new(StringNode::new(format!("```{language}\n"))));
        process_file_children_to_weave(
            &child.children,
            weave_ip,
            transcripts,
            state,
            child.source_column,
            registry,
            doc_path,
            span_files,
            Some(&run),
        );
        weave_ip.add(Arc::new(StringNode::new("```\n".to_string())));
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
    ingested: Option<&Arc<str>>,
) {
    let mut pending_aligned_paste_break = false;
    for (position, child) in children.iter().enumerate() {
        match child {
            HickNode::Text(text, span) => {
                // The fence must show the file's OWN bytes, so it drops the
                // line break that ends the open tag's line exactly as the
                // file output does (`strip_opening_break`). Without this the
                // woven markdown depicts a file with a blank first line and
                // the file on disk has none — the fence would be lying about
                // the very thing it exists to show.
                let (text, span) = match (position, pending_aligned_paste_break) {
                    (0, _) | (_, true) => crate::strip_opening_break(text, span.as_ref()),
                    _ => (text.as_str(), *span),
                };
                pending_aligned_paste_break = false;
                let (text, span) = match position {
                    0 => crate::strip_leading_bom(text, span.as_ref()),
                    _ => (text, span),
                };
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
                match &span {
                    // Same rule as the file output: inside an ingested block
                    // these bytes are present and byte-precise but not yours,
                    // so the woven markdown's ribbon says where they came
                    // from rather than colouring them as your prose.
                    Some(span) => weave_ip.add(Arc::new(SpanNode::new(
                        dedented,
                        match ingested {
                            Some(run) => SourceOrigin::Ingested {
                                file: origin_file(doc_path, span_files, span),
                                span: *span,
                                run: run.clone(),
                            },
                            None => SourceOrigin::Literal {
                                file: origin_file(doc_path, span_files, span),
                                span: *span,
                            },
                        },
                    ))),
                    None => weave_ip.add(Arc::new(StringNode::new(dedented))),
                }
            }
            HickNode::Tag(child_tag) => {
                if let Some(handler) = registry.find(&child_tag.name) {
                    let paste_line_indent = (child_tag.name == "paste")
                        .then(|| crate::paste_indent_before(children, position, indent))
                        .flatten();
                    let aligned_paste = paste_line_indent.is_some();
                    let child_ctx = ProcessingContext {
                        state,
                        transcripts,
                        indent,
                        paste_line_indent,
                        registry: Some(registry),
                        context: None,
                        source_file: None,
                        span_files: &[],
                    };
                    match handler.process(child_tag, &child_ctx) {
                        Ok(TagResult::Node(n)) => {
                            pending_aligned_paste_break = aligned_paste;
                            weave_ip.add(n);
                        }
                        Ok(TagResult::Nodes(ns)) => {
                            pending_aligned_paste_break = aligned_paste;
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

/// A graph diagram's body, resolved synchronously: text children dedented,
/// `<hick:paste>` children answered from the run state's fragments — which is
/// what lets a derived scene's topology arrive from the cell that proved it.
/// The streaming path the other renderers use cannot be parsed as JSON before
/// it is woven, and a scene must be parsed to be downgraded.
fn resolved_scene_body(tag: &hick_lang::HickTag, state: &Arc<MultiDocumentState>) -> String {
    let mut body = String::new();
    for child in &tag.children {
        match child {
            HickNode::Text(text, _) => body.push_str(&dedent(text, tag.source_column)),
            HickNode::Tag(child_tag) if child_tag.name == "paste" => {
                if let Some(select) = tag_attr(child_tag, "select")
                    && let Some(resolved) = state.resolve_paste(&select, None, false)
                {
                    body.push_str(&resolved);
                }
            }
            HickNode::Tag(_) => {}
        }
    }
    body
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
        //
        // Two passes, in this order and composed into one segmenter: first
        // the substitutions, then the document-link rewrite. Order matters —
        // link rewriting runs only over passthrough text, so a link that came
        // out of a substituted value is left alone (those bytes are already
        // the weaver's and have no source span to keep intact).
        let subs_state = state.clone();
        let transform: Arc<dyn Node> = Arc::new(ProvenanceTransformNode::new(
            raw_ip,
            move |text| {
                crate::links::rewrite_document_links(
                    crate::apply_substitutions_segmented_to_transform(text, &subs_state),
                )
            },
            "substitute",
        ));

        let weave_ip = Arc::new(InsertionPoint::new());
        weave_ip.add(transform);
        weave_ip.close();
        state.add_file_output(weave_path.clone(), weave_ip);
    }
}

// Protects docs/guarantees/authoring/a-silent-tag-does-not-double-a-blank-line.md
#[cfg(test)]
mod strip_up_to_one_blank_line_tests {
    use super::strip_up_to_one_blank_line;

    #[test]
    fn two_leading_newlines_are_fully_removed() {
        let (text, span) = strip_up_to_one_blank_line("\n\nBetween those…", None);
        assert_eq!(text, "Between those…");
        assert!(span.is_none());
    }

    #[test]
    fn a_single_leading_newline_is_fully_removed_not_left_dangling() {
        // The text between two ADJACENT silent tags is often just one "\n" —
        // there is no real content there to protect.
        let (text, _) = strip_up_to_one_blank_line("\n", None);
        assert_eq!(text, "");
    }

    #[test]
    fn three_or_more_leading_newlines_keep_the_real_paragraph_break() {
        // Never taken: a genuine blank line PLUS whatever came before it in
        // the prose is not this function's to remove — only the redundant
        // pair a silent tag leaves behind.
        let (text, _) = strip_up_to_one_blank_line("\n\n\nreal content", None);
        assert_eq!(text, "\nreal content");
    }

    #[test]
    fn text_with_no_leading_newline_is_untouched() {
        let (text, span) = strip_up_to_one_blank_line("no leading newline", None);
        assert_eq!(text, "no leading newline");
        assert!(span.is_none());
    }

    #[test]
    fn a_span_moves_by_exactly_the_bytes_taken() {
        let span = hick_lang::SourceSpan {
            start: 100,
            end: 120,
            start_line: 5,
            start_col: 0,
            file_id: None,
        };
        let (text, moved) = strip_up_to_one_blank_line("\n\ncontent", Some(&span));
        assert_eq!(text, "content");
        let moved = moved.expect("a span in must be a span out");
        assert_eq!(moved.start, 102);
        assert_eq!(moved.end, 120);
        assert_eq!(moved.start_line, 7);
        assert_eq!(moved.start_col, 0);
    }
}
