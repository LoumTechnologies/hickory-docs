//! Structural (parser-level) navigation over a hick document.
//!
//! These answers need no child language server: `hick:copy id="x"` defines a
//! named block, `hick:paste select="#x"` (or `hick:copy select="#x"`) weaves
//! it. Definition at a paste site jumps to the copy definition; references at
//! either end list every paste site (optionally plus the definition).
//!
//! Also home to the LSP-position ↔ byte-offset helpers used by the proxy
//! handlers (LSP positions count UTF-16 code units per line).

use hick_lang::{HickDocument, HickNode, HickTag};

/// Convert an LSP position (0-based line, UTF-16 character) to a byte offset.
pub fn position_to_byte(source: &str, line: u32, character: u32) -> Option<usize> {
    let mut current_line = 0u32;
    let mut line_start = 0usize;
    if line > 0 {
        let mut found = false;
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                current_line += 1;
                if current_line == line {
                    line_start = i + 1;
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return None;
        }
    }
    let rest = &source[line_start..];
    let line_end = rest
        .find('\n')
        .map(|i| line_start + i)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];

    let mut units = 0u32;
    for (byte_idx, ch) in line_text.char_indices() {
        if units >= character {
            return Some(line_start + byte_idx);
        }
        units += ch.len_utf16() as u32;
    }
    // Position at (or past) end of line clamps to line end.
    Some(line_end)
}

/// Convert a byte offset to an LSP position (0-based line, UTF-16 character).
/// Offsets past the end clamp to the final position.
pub fn byte_to_position(source: &str, offset: usize) -> (u32, u32) {
    let offset = offset.min(source.len());
    let before = &source[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32;
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character: u32 = source[line_start..offset]
        .chars()
        .map(|c| c.len_utf16() as u32)
        .sum();
    (line, character)
}

/// A byte span in the .hick source.
pub type Span = (usize, usize);

#[derive(Debug, Clone)]
struct CopyDef {
    id: String,
    /// Span of the opening `<hick:copy …>` tag (the definition location).
    tag_span: Span,
    /// Extent of the whole element (opening tag through last child span).
    extent: Span,
}

#[derive(Debug, Clone)]
struct PasteSite {
    id: String,
    /// Span of the `<hick:paste select="#id" …>` tag.
    span: Span,
}

#[derive(Debug, Default)]
struct CopyGraph {
    defs: Vec<CopyDef>,
    sites: Vec<PasteSite>,
}

fn max_descendant_end(tag: &HickTag, mut end: usize) -> usize {
    for child in &tag.children {
        match child {
            HickNode::Text(_, Some(span)) => end = end.max(span.end),
            HickNode::Text(_, None) => {}
            HickNode::Tag(t) => {
                if let Some(span) = t.source_span {
                    end = end.max(span.end);
                }
                end = max_descendant_end(t, end);
            }
        }
    }
    end
}

fn walk(nodes: &[HickNode], graph: &mut CopyGraph) {
    for node in nodes {
        let HickNode::Tag(tag) = node else { continue };
        let span = tag.source_span.map(|s| (s.start, s.end));
        if tag.name == "copy" || tag.name == "paste" {
            if let Some(select) = tag.get_attribute("select") {
                if let Some(span) = span {
                    graph.sites.push(PasteSite {
                        id: select.strip_prefix('#').unwrap_or(select).to_string(),
                        span,
                    });
                }
            } else if tag.name == "copy"
                && let Some(id) = tag.get_attribute("id")
                && let Some(span) = span
            {
                graph.defs.push(CopyDef {
                    id: id.to_string(),
                    tag_span: span,
                    extent: (span.0, max_descendant_end(tag, span.1)),
                });
            }
        }
        walk(&tag.children, graph);
    }
}

fn build_graph(doc: &HickDocument) -> CopyGraph {
    let mut graph = CopyGraph::default();
    walk(&doc.nodes, &mut graph);
    graph
}

/// Which copy id is under the cursor (a paste site tag, a copy definition tag,
/// or anywhere inside a copy definition's body)?
fn id_at(graph: &CopyGraph, offset: usize) -> Option<String> {
    if let Some(site) = graph
        .sites
        .iter()
        .find(|s| s.span.0 <= offset && offset < s.span.1)
    {
        return Some(site.id.clone());
    }
    graph
        .defs
        .iter()
        .find(|d| d.extent.0 <= offset && offset < d.extent.1)
        .map(|d| d.id.clone())
}

/// Structural go-to-definition: the copy definition's opening-tag span for the
/// copy id under the cursor.
pub fn definition(doc: &HickDocument, offset: usize) -> Option<Span> {
    let graph = build_graph(doc);
    let id = id_at(&graph, offset)?;
    graph.defs.iter().find(|d| d.id == id).map(|d| d.tag_span)
}

/// Structural references: every paste site weaving the copy id under the
/// cursor, plus (when `include_declaration`) the definition's opening tag.
pub fn references(doc: &HickDocument, offset: usize, include_declaration: bool) -> Vec<Span> {
    let graph = build_graph(doc);
    let Some(id) = id_at(&graph, offset) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if include_declaration && let Some(def) = graph.defs.iter().find(|d| d.id == id) {
        out.push(def.tag_span);
    }
    out.extend(graph.sites.iter().filter(|s| s.id == id).map(|s| s.span));
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="imports">
use std::io;
</hick:copy>
<hick:file path="main.rs">
<hick:paste select="#imports" />
fn main() {}
</hick:file>
<hick:file path="lib.rs">
<hick:paste select="#imports" />
</hick:file>
</hick:doc>"##;

    fn parse() -> HickDocument {
        hick_lang::parse(DOC).unwrap()
    }

    #[test]
    fn position_byte_round_trip_ascii() {
        let src = "abc\ndef\nghi";
        assert_eq!(position_to_byte(src, 0, 0), Some(0));
        assert_eq!(position_to_byte(src, 1, 2), Some(6));
        assert_eq!(byte_to_position(src, 6), (1, 2));
        assert_eq!(byte_to_position(src, 0), (0, 0));
    }

    #[test]
    fn position_byte_utf16_units() {
        // '€' is 3 UTF-8 bytes, 1 UTF-16 unit; '𝄞' is 4 UTF-8 bytes, 2 units.
        let src = "€x\n𝄞y";
        assert_eq!(position_to_byte(src, 0, 1), Some(3)); // after €
        assert_eq!(position_to_byte(src, 1, 2), Some(9)); // after 𝄞 (5 + 4)
        assert_eq!(byte_to_position(src, 3), (0, 1));
        assert_eq!(byte_to_position(src, 9), (1, 2));
    }

    #[test]
    fn position_past_line_end_clamps() {
        let src = "ab\ncd";
        assert_eq!(position_to_byte(src, 0, 99), Some(2));
        assert_eq!(position_to_byte(src, 9, 0), None);
    }

    #[test]
    fn definition_from_paste_site() {
        let doc = parse();
        let paste_off = DOC.find(r##"<hick:paste select="#imports" />"##).unwrap();
        let span = definition(&doc, paste_off + 5).unwrap();
        assert!(DOC[span.0..span.1].starts_with(r#"<hick:copy id="imports""#));
    }

    #[test]
    fn definition_from_inside_copy_body() {
        let doc = parse();
        let body_off = DOC.find("use std::io;").unwrap();
        let span = definition(&doc, body_off).unwrap();
        assert!(DOC[span.0..span.1].starts_with(r#"<hick:copy id="imports""#));
    }

    #[test]
    fn references_lists_both_paste_sites() {
        let doc = parse();
        let body_off = DOC.find("use std::io;").unwrap();
        let refs = references(&doc, body_off, false);
        assert_eq!(refs.len(), 2);
        for span in &refs {
            assert!(DOC[span.0..span.1].starts_with(r##"<hick:paste select="#imports""##));
        }
        let with_decl = references(&doc, body_off, true);
        assert_eq!(with_decl.len(), 3);
    }

    #[test]
    fn no_copy_under_cursor_is_empty() {
        let doc = parse();
        let off = DOC.find("fn main() {}").unwrap();
        assert_eq!(definition(&doc, off), None);
        assert!(references(&doc, off, true).is_empty());
    }
}
