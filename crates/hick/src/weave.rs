//! Weave (literate programming) output support.
//!
//! Processes document nodes into markdown-style literate programming output:
//! prose text, code fences for file content, variable interpolation, and
//! exec transcript rendering.

use std::collections::HashMap;
use std::sync::Arc;

use hick_exec::node::{InsertionPoint, StringNode};
use hick_exec::state::MultiDocumentState;
use hick_lang::{HickDocument, HickNode, dedent};

use crate::executor::ExecTranscriptEntry;
use crate::text::{apply_substitutions, interpolate_path};
use crate::{ExecShow, parse_exec_show, render_transcript, tag_attr};

/// Map file extension to markdown code fence language identifier.
fn extension_to_language(path: &str) -> &'static str {
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

/// Process document nodes for weave output.
///
/// Iterates through nodes and emits:
/// - `HickNode::Text` → prose (with substitutions applied, dedented)
/// - `<hick:file>` → heading + fenced code block (unless `doc-hidden="true"`)
/// - `<hick:val>` → resolved variable value
/// - `<hick:when>` → recursively process children (already filtered)
///
/// The `indent` parameter specifies how many leading spaces to strip from prose
/// text (typically 0 for top-level content, or the parent tag's source_column).
fn process_weave_content(
    nodes: &[HickNode],
    weave_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<ExecTranscriptEntry>>,
    state: &MultiDocumentState,
    indent: usize,
) {
    for node in nodes {
        match node {
            HickNode::Text(text) => {
                // Dedent and apply substitutions to prose text
                let dedented = dedent(text, indent);
                let substituted = apply_substitutions(&dedented, state);
                weave_ip.add(Arc::new(StringNode::new(substituted)));
            }
            HickNode::Tag(tag) => {
                process_weave_tag(tag, weave_ip, transcripts, state);
            }
        }
    }
}

/// Process a single tag for weave output.
fn process_weave_tag(
    tag: &hick_lang::HickTag,
    weave_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<ExecTranscriptEntry>>,
    state: &MultiDocumentState,
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
                process_file_children_to_weave(&tag.children, weave_ip, transcripts, state, tag.source_column);

                // Emit closing code fence
                weave_ip.add(Arc::new(StringNode::new("```\n".to_string())));
            }
        }
        "val" => {
            let var_name = tag_attr(tag, "name").unwrap_or_default();
            if let Some(value) = state.resolve_var(&var_name) {
                weave_ip.add(Arc::new(StringNode::new(value)));
            }
        }
        "when" => {
            // When tags have already been filtered, so just process children
            // Use the when tag's indentation for dedenting child content
            process_weave_content(&tag.children, weave_ip, transcripts, state, tag.source_column);
        }
        // Skip other tags (var, copy, cut, substitute, feature, exclude, etc.)
        // These are structural and don't produce weave output
        _ => {}
    }
}

/// Process file children for weave output (similar to process_file_children but outputs to weave).
///
/// The `indent` parameter specifies how many leading spaces to strip from each
/// line of text content (typically the source_column of the parent file tag).
fn process_file_children_to_weave(
    children: &[HickNode],
    weave_ip: &Arc<InsertionPoint>,
    transcripts: &HashMap<String, Vec<ExecTranscriptEntry>>,
    state: &MultiDocumentState,
    indent: usize,
) {
    for child in children {
        match child {
            HickNode::Text(text) => {
                // Dedent and apply substitutions to text content
                let dedented = dedent(text, indent);
                let substituted = apply_substitutions(&dedented, state);
                weave_ip.add(Arc::new(StringNode::new(substituted)));
            }
            HickNode::Tag(child_tag) if child_tag.name == "exec" => {
                let container_name = tag_attr(child_tag, "container").unwrap_or_default();
                let show = parse_exec_show(child_tag);

                if show != ExecShow::None {
                    if let Some(entries) = transcripts.get(&container_name) {
                        let rendered = render_transcript(entries, show);
                        weave_ip.add(Arc::new(StringNode::new(rendered)));
                    }
                }
            }
            HickNode::Tag(child_tag) if child_tag.name == "paste" => {
                let selector = tag_attr(child_tag, "select").unwrap_or_default();
                if let Some(content) = state.resolve_paste(&selector) {
                    // Dedent and apply substitutions to pasted content
                    let dedented = dedent(&content, indent);
                    let substituted = apply_substitutions(&dedented, state);
                    weave_ip.add(Arc::new(StringNode::new(substituted)));
                }
            }
            HickNode::Tag(child_tag) if child_tag.name == "val" => {
                let var_name = tag_attr(child_tag, "name").unwrap_or_default();
                if let Some(value) = state.resolve_var(&var_name) {
                    weave_ip.add(Arc::new(StringNode::new(value)));
                }
            }
            _ => {}
        }
    }
}

/// Process weave output for all documents if weave is enabled.
pub(crate) fn process_weave_output(
    documents: &[(&str, HickDocument)],
    transcripts: &HashMap<String, Vec<ExecTranscriptEntry>>,
    state: &MultiDocumentState,
) {
    // Find the weave path (use the first document that has one)
    let weave_path = documents
        .iter()
        .find_map(|(_, doc)| doc.weave_path.as_ref());

    if let Some(weave_path) = weave_path {
        let weave_ip = Arc::new(InsertionPoint::new());

        // Process all document nodes for weave output
        // Use 0 indent for top-level content (direct children of <hick:doc>)
        for (_, doc) in documents {
            process_weave_content(&doc.nodes, &weave_ip, transcripts, state, 0);
        }

        weave_ip.close();
        state.add_file_output(weave_path.clone(), weave_ip);
    }
}
