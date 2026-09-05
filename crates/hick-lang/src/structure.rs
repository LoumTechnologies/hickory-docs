//! A document's structure as a reader draws it: every tag and every block,
//! with byte spans, from a parse that never fails.
//!
//! This is the shape an editor needs on every keystroke. It is computed by
//! [`parse_lenient`], so what the editor draws and what `hick run` parses are
//! the same parser reading the same bytes — a rebound prefix
//! (`<h:doc xmlns:h="…">`), a verbatim element whose content quotes tags, and
//! a document about hick's own syntax all read the same in both places. The
//! editor used to carry a second parser of its own, and those were exactly
//! the three places it disagreed.
//!
//! Offsets are **byte** offsets into the source as given, including a leading
//! byte-order mark if there is one, so they index the same bytes every other
//! span in this crate does.

use crate::{HickNode, HickTag, SourceSpan, parse_lenient};

/// One tag as written: an opening tag, a self-closing tag, or a closing tag.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructureTag {
    /// Local name without the prefix, e.g. `"exec"`.
    pub name: String,
    /// Byte offset of the `<`.
    pub from: usize,
    /// Byte offset past the `>`.
    pub to: usize,
    /// A closing tag (`</prefix:name>`).
    pub closing: bool,
    /// A self-closing tag (`<prefix:name … />`).
    pub self_closing: bool,
    /// Attributes in document order; empty for a closing tag.
    pub attrs: Vec<(String, String)>,
}

/// One element: its opening tag, its content, and its closing tag if any.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructureBlock {
    /// Local name without the prefix.
    pub name: String,
    /// Byte offset of the opening tag's `<`.
    pub from: usize,
    /// Byte offset past the closing tag's `>` — or past the end of the
    /// document for an element nobody closed, or past the opening tag for a
    /// self-closing one.
    pub to: usize,
    /// Byte offset where the content starts (past the opening tag).
    pub content_from: usize,
    /// Byte offset where the content ends (at the closing tag's `<`).
    pub content_to: usize,
    /// Whether a closing tag was found. `false` for a self-closing element
    /// and for one that runs to the end of the document.
    pub closed: bool,
    /// Attributes in document order.
    pub attrs: Vec<(String, String)>,
}

/// Everything a reader needs to draw a document's structure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Structure {
    /// The namespace prefix the document binds (`"hick"` when bare).
    pub prefix: String,
    /// Every tag in document order.
    pub tags: Vec<StructureTag>,
    /// Every element, sorted by start and then by length, longest first — so
    /// a parent precedes its children.
    pub blocks: Vec<StructureBlock>,
    /// The first thing a strict parse would refuse, or `None` when the
    /// document is well formed. A reader draws the structure either way and
    /// shows this beside it.
    pub error: Option<String>,
}

/// The structure of `source`, from a parse that never fails.
pub fn structure(source: &str) -> Structure {
    let (doc, error) = parse_lenient(source);
    // The parser strips a leading BOM before it counts bytes; a reader that
    // holds the source as given must not.
    let shift = if source.starts_with('\u{feff}') { 3 } else { 0 };
    let end = source.len();
    let mut out = Structure {
        prefix: doc.prefix.clone(),
        tags: Vec::new(),
        blocks: Vec::new(),
        error: error.map(|e| e.to_string()),
    };
    if let Some(root) = &doc.root_tag {
        collect_tag(root, shift, end, &mut out);
    }
    collect(&doc.nodes, shift, end, &mut out);
    out.tags.sort_by_key(|t| t.from);
    out.blocks
        .sort_by(|a, b| a.from.cmp(&b.from).then(b.to.cmp(&a.to)));
    out
}

fn collect(nodes: &[HickNode], shift: usize, end: usize, out: &mut Structure) {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            collect_tag(tag, shift, end, out);
            collect(&tag.children, shift, end, out);
        }
    }
}

fn collect_tag(tag: &HickTag, shift: usize, end: usize, out: &mut Structure) {
    let Some(open) = tag.source_span else {
        // A tag without a span was built by hand, not read from this
        // source, and has no place a reader could draw it.
        return;
    };
    let at = |span: SourceSpan| (span.start + shift, span.end + shift);
    let (from, open_to) = at(open);
    out.tags.push(StructureTag {
        name: tag.name.clone(),
        from,
        to: open_to,
        closing: false,
        self_closing: tag.self_closing,
        attrs: tag.attributes.clone(),
    });
    let (to, content_to, closed) = match (tag.self_closing, tag.close_span) {
        (true, _) => (open_to, open_to, false),
        (false, Some(close)) => {
            let (close_from, close_to) = at(close);
            out.tags.push(StructureTag {
                name: tag.name.clone(),
                from: close_from,
                to: close_to,
                closing: true,
                self_closing: false,
                attrs: Vec::new(),
            });
            (close_to, close_from, true)
        }
        (false, None) => (end, end, false),
    };
    out.blocks.push(StructureBlock {
        name: tag.name.clone(),
        from,
        to,
        content_from: open_to,
        content_to,
        closed,
        attrs: tag.attributes.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(s: &Structure) -> Vec<(&str, usize, usize, bool)> {
        s.tags
            .iter()
            .map(|t| (t.name.as_str(), t.from, t.to, t.closing))
            .collect()
    }

    #[test]
    fn a_well_formed_bare_document_has_no_error() {
        let src = "# Hi\n<hick:exec container=\"c\">echo 1</hick:exec>\n";
        let s = structure(src);
        assert_eq!(s.error, None);
        assert_eq!(s.prefix, "hick");
        assert_eq!(
            names(&s),
            vec![("exec", 5, 30, false), ("exec", 36, 48, true)]
        );
        let b = &s.blocks[0];
        assert_eq!(
            (b.from, b.to, b.content_from, b.content_to, b.closed),
            (5, 48, 30, 36, true)
        );
        assert_eq!(&src[b.content_from..b.content_to], "echo 1");
    }

    #[test]
    fn a_rebound_prefix_is_honoured_and_literal_hick_tags_are_prose() {
        let src = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\nWrite `<hick:exec>` to run.\n<h:exec container=\"c\">true</h:exec>\n</h:doc>";
        let s = structure(src);
        assert_eq!(s.error, None);
        assert_eq!(s.prefix, "h");
        let tag_names: Vec<&str> = s.tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(tag_names, vec!["doc", "exec", "exec", "doc"]);
        assert_eq!(s.blocks[0].name, "doc");
        assert_eq!(s.blocks[0].to, src.len());
    }

    #[test]
    fn verbatim_content_is_not_structure() {
        let src = "<hick:tool-result name=\"x\">saw <hick:copy id=\"a\"> here</hick:tool-result>";
        let s = structure(src);
        assert_eq!(s.error, None);
        let tag_names: Vec<&str> = s.tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(tag_names, vec!["tool-result", "tool-result"]);
    }

    #[test]
    fn an_unclosed_element_runs_to_the_end_and_is_reported() {
        let src = "before\n<hick:exec container=\"c\">echo";
        let s = structure(src);
        assert!(
            s.error
                .as_deref()
                .unwrap()
                .contains("unclosed tag <hick:exec>"),
            "{:?}",
            s.error
        );
        assert_eq!(s.tags.len(), 1);
        let b = &s.blocks[0];
        assert_eq!(
            (b.to, b.content_to, b.closed),
            (src.len(), src.len(), false)
        );
    }

    #[test]
    fn a_stray_closer_is_text_and_reported() {
        let src = "a</hick:exec>b<hick:copy id=\"x\">c</hick:copy>";
        let s = structure(src);
        assert!(
            s.error
                .as_deref()
                .unwrap()
                .contains("unexpected closing tag")
        );
        let tag_names: Vec<&str> = s.tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(tag_names, vec!["copy", "copy"]);
    }

    #[test]
    fn a_tag_that_does_not_parse_is_text_and_what_follows_still_is_structure() {
        // An unquoted attribute value is a syntax error. The NEXT tag must
        // still be found.
        let src = "<hick:exec container=c>\nmore\n<hick:copy id=\"x\" />";
        let s = structure(src);
        assert!(s.error.is_some());
        let tag_names: Vec<&str> = s.tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(tag_names, vec!["copy"]);
        assert!(!s.blocks[0].closed && s.tags[0].self_closing);
    }

    #[test]
    fn an_unclosed_comment_is_text() {
        let src = "<!-- never closed <hick:copy id=\"x\" />";
        let s = structure(src);
        assert!(s.error.as_deref().unwrap().contains("unclosed comment"));
        assert_eq!(s.tags.len(), 1);
        assert_eq!(s.tags[0].name, "copy");
    }

    #[test]
    fn a_broken_root_reads_as_a_bare_document() {
        let src = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\n<hick:copy id=\"x\" />";
        let s = structure(src);
        assert!(s.error.is_some());
        let tag_names: Vec<&str> = s.tags.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(tag_names, vec!["copy"]);
    }

    #[test]
    fn offsets_include_a_leading_bom() {
        let src = "\u{feff}<hick:copy id=\"x\" />";
        let s = structure(src);
        assert_eq!(s.tags[0].from, 3);
        assert_eq!(&src[s.tags[0].from..s.tags[0].to], "<hick:copy id=\"x\" />");
    }

    #[test]
    fn nested_blocks_sort_parent_first() {
        let src = "<hick:file path=\"a\"><hick:paste select=\"#x\" /></hick:file>";
        let s = structure(src);
        let block_names: Vec<&str> = s.blocks.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(block_names, vec!["file", "paste"]);
    }
}
