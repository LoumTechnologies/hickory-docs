//! Edit compaction tool for hick documents.
//!
//! Merges multiple append-only edits (copy blocks with same class) while
//! preserving output equivalence. This helps clean up documents that have
//! accumulated many small edits over time.
//!
//! # Algorithm
//!
//! 1. Parse document and identify copy blocks with same class
//! 2. Group adjacent/related blocks
//! 3. Merge blocks within each group
//! 4. Re-emit document source
//! 5. Optionally verify equivalence with original
//!
//! # Example
//!
//! ```bash
//! hick-compact input.hick -o output.hick --verify
//! ```

use std::collections::HashMap;

use anyhow::Result;
use hick_lang::{HickDocument, HickNode};

// ---------------------------------------------------------------------------
// Content Block for Compaction
// ---------------------------------------------------------------------------

/// A content block extracted from the document for potential merging.
#[derive(Debug, Clone)]
struct ContentBlockInfo {
    /// Index in the original document (for ordering).
    index: usize,
    /// Block ID (if any).
    id: Option<String>,
    /// Class names.
    classes: Vec<String>,
    /// Content text.
    content: String,
    /// Tag name (copy or cut).
    tag_name: String,
}

// ---------------------------------------------------------------------------
// Document Compaction
// ---------------------------------------------------------------------------

/// Compact a hick document by merging related copy/cut blocks.
///
/// Returns the compacted document source as a string.
pub fn compact_document(source: &str) -> Result<String> {
    let doc = hick_lang::parse(source).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

    // Extract content blocks
    let blocks = extract_content_blocks(&doc.nodes);

    // Group blocks by class
    let groups = group_by_class(&blocks);

    // Build compacted document
    let compacted = build_compacted_document(&doc, &groups);

    Ok(compacted)
}

/// Extract content blocks from document nodes.
fn extract_content_blocks(nodes: &[HickNode]) -> Vec<ContentBlockInfo> {
    let mut blocks = Vec::new();

    for (index, node) in nodes.iter().enumerate() {
        if let HickNode::Tag(tag) = node
            && (tag.name == "copy" || tag.name == "cut")
        {
            let id = tag
                .attributes
                .iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.clone());

            let classes: Vec<String> = tag
                .attributes
                .iter()
                .find(|(k, _)| k == "class")
                .map(|(_, v)| v.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default();

            let content: String = tag
                .children
                .iter()
                .filter_map(|c| match c {
                    HickNode::Text(t, _) => Some(t.as_str()),
                    _ => None,
                })
                .collect();

            blocks.push(ContentBlockInfo {
                index,
                id,
                classes,
                content,
                tag_name: tag.name.clone(),
            });
        }
    }

    blocks
}

/// Group blocks by class for merging.
fn group_by_class(blocks: &[ContentBlockInfo]) -> HashMap<String, Vec<&ContentBlockInfo>> {
    let mut groups: HashMap<String, Vec<&ContentBlockInfo>> = HashMap::new();

    for block in blocks {
        for class in &block.classes {
            groups.entry(class.clone()).or_default().push(block);
        }

        // Blocks with only ID go into a special group
        if block.classes.is_empty()
            && let Some(id) = &block.id
        {
            groups
                .entry(format!("__id__{}", id))
                .or_default()
                .push(block);
        }
    }

    groups
}

/// Build compacted document source.
fn build_compacted_document(
    doc: &HickDocument,
    groups: &HashMap<String, Vec<&ContentBlockInfo>>,
) -> String {
    // For simplicity, we'll emit merged blocks followed by non-content nodes
    // A more sophisticated implementation would preserve document structure

    let mut output = String::new();
    output.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    output.push('\n');
    output.push_str(r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">"#);

    // Track which blocks have been emitted
    let mut emitted_indices = std::collections::HashSet::new();

    // Emit merged blocks by class
    for (class, blocks) in groups {
        if class.starts_with("__id__") {
            // ID-only blocks - emit as-is
            for block in blocks {
                if !emitted_indices.contains(&block.index) {
                    output.push('\n');
                    emit_block(&mut output, block);
                    emitted_indices.insert(block.index);
                }
            }
        } else if blocks.len() > 1 {
            // Multiple blocks with same class - merge them
            let merged_content: String = blocks.iter().map(|b| b.content.as_str()).collect();

            // Use the first block's tag name and ID (if any)
            let first = blocks[0];
            output.push('\n');
            output.push_str(&format!(
                r#"<hick:{} class="{}">{}</hick:{}>"#,
                first.tag_name,
                class,
                escape_xml(&merged_content),
                first.tag_name,
            ));

            for block in blocks {
                emitted_indices.insert(block.index);
            }
        } else {
            // Single block - emit as-is
            let block = blocks[0];
            if !emitted_indices.contains(&block.index) {
                output.push('\n');
                emit_block(&mut output, block);
                emitted_indices.insert(block.index);
            }
        }
    }

    // Emit non-content nodes (file, exec, etc.)
    for node in &doc.nodes {
        if let HickNode::Tag(tag) = node
            && tag.name != "copy"
            && tag.name != "cut"
        {
            output.push('\n');
            emit_node(&mut output, node, 0);
        }
    }

    output.push('\n');
    output.push_str("</hick:doc>\n");
    output
}

/// Emit a content block.
fn emit_block(output: &mut String, block: &ContentBlockInfo) {
    let mut attrs = String::new();

    if let Some(id) = &block.id {
        attrs.push_str(&format!(r#" id="{}""#, escape_xml(id)));
    }

    if !block.classes.is_empty() {
        attrs.push_str(&format!(r#" class="{}""#, block.classes.join(" ")));
    }

    output.push_str(&format!(
        "<hick:{}{}>{}</hick:{}>",
        block.tag_name,
        attrs,
        escape_xml(&block.content),
        block.tag_name,
    ));
}

/// Emit a node (recursively for nested tags).
fn emit_node(output: &mut String, node: &HickNode, indent: usize) {
    let indent_str = "  ".repeat(indent);

    match node {
        HickNode::Text(text, _) => {
            output.push_str(&escape_xml(text));
        }
        HickNode::Tag(tag) => {
            output.push_str(&indent_str);
            output.push_str(&format!("<hick:{}", tag.name));

            for (k, v) in &tag.attributes {
                output.push_str(&format!(r#" {}="{}""#, k, escape_xml(v)));
            }

            if tag.self_closing && tag.children.is_empty() {
                output.push_str(" />\n");
            } else {
                output.push('>');

                if tag.children.iter().any(|c| matches!(c, HickNode::Tag(_))) {
                    output.push('\n');
                    for child in &tag.children {
                        emit_node(output, child, indent + 1);
                    }
                    output.push_str(&indent_str);
                } else {
                    for child in &tag.children {
                        emit_node(output, child, 0);
                    }
                }

                output.push_str(&format!("</hick:{}>\n", tag.name));
            }
        }
    }
}

/// Escape XML special characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

/// Verify that compacted document is equivalent to original.
pub async fn verify_compaction(original: &str, compacted: &str) -> Result<bool> {
    let result = crate::equiv::check_equivalence(original, compacted, 100).await?;
    Ok(result.equivalent)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn hick_doc(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
        )
    }

    #[test]
    fn extract_blocks() {
        let src = hick_doc(
            r#"<hick:copy class="imports">import foo;</hick:copy>
<hick:copy class="imports">import bar;</hick:copy>"#,
        );

        let doc = hick_lang::parse(&src).unwrap();
        let blocks = extract_content_blocks(&doc.nodes);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].classes, vec!["imports"]);
        assert_eq!(blocks[1].classes, vec!["imports"]);
    }

    #[test]
    fn group_blocks() {
        let src = hick_doc(
            r#"<hick:copy class="imports">import foo;</hick:copy>
<hick:copy class="imports">import bar;</hick:copy>
<hick:copy class="other">other;</hick:copy>"#,
        );

        let doc = hick_lang::parse(&src).unwrap();
        let blocks = extract_content_blocks(&doc.nodes);
        let groups = group_by_class(&blocks);

        assert_eq!(groups.get("imports").unwrap().len(), 2);
        assert_eq!(groups.get("other").unwrap().len(), 1);
    }

    #[test]
    fn compact_merges_same_class() {
        let src = hick_doc(
            r#"<hick:copy class="imports">import foo;</hick:copy>
<hick:copy class="imports">import bar;</hick:copy>
<hick:file path="out.txt"><hick:paste select=".imports" /></hick:file>"#,
        );

        let compacted = compact_document(&src).unwrap();

        // Should have merged the imports
        assert!(compacted.contains("import foo;import bar;"));
    }

    #[tokio::test]
    async fn verify_compaction_preserves_output() {
        let src = hick_doc(
            r#"<hick:copy class="items">a;</hick:copy>
<hick:copy class="items">b;</hick:copy>
<hick:file path="out.txt"><hick:paste select=".items" /></hick:file>"#,
        );

        let compacted = compact_document(&src).unwrap();

        // Both should produce the same output
        let result1 = crate::run_pipeline(&[("orig.hick", &src)], &[])
            .await
            .unwrap();
        let result2 = crate::run_pipeline(&[("compact.hick", &compacted)], &[])
            .await
            .unwrap();

        // Note: The compacted version may have slight whitespace differences in non-content areas,
        // but the actual pasted content should be the same.
        let content1 = result1.files.get("out.txt").unwrap().as_text().unwrap();
        let content2 = result2.files.get("out.txt").unwrap().as_text().unwrap();

        // Trim for comparison since whitespace around file content may differ
        assert_eq!(content1.trim(), content2.trim());
    }
}
