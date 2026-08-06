//! Virtual file generation from `hick:file` blocks with copy/paste resolution.

use std::collections::HashMap;

use hick_lang::{HickDocument, HickNode, HickTag, SourceSpan};

use crate::lang_detect;

/// A segment of a virtual file, tracking its origin in the .hick source.
#[derive(Debug, Clone)]
pub struct VirtualSegment {
    /// The text content of this segment.
    pub text: String,
    /// Source span in the .hick file where this text originated.
    /// None for synthesized content (e.g., newlines between segments).
    pub source_span: Option<SourceSpan>,
    /// 1-based line in the .hick file where this segment's tag starts.
    pub source_line: usize,
    /// 0-based column for dedenting.
    pub source_column: usize,
}

/// A virtual file generated from a hick:file block.
#[derive(Debug, Clone)]
pub struct VirtualFile {
    /// Output file path (from the path="" attribute).
    pub path: String,
    /// Language identifier for this file.
    pub language_id: Option<&'static str>,
    /// Ordered segments that compose this file's content.
    pub segments: Vec<VirtualSegment>,
}

impl VirtualFile {
    /// Get the full text content of the virtual file.
    pub fn content(&self) -> String {
        let mut out = String::new();
        for seg in &self.segments {
            out.push_str(&seg.text);
        }
        out
    }

    /// Total number of lines.
    pub fn line_count(&self) -> usize {
        let content = self.content();
        if content.is_empty() {
            return 0;
        }
        content.lines().count()
    }
}

/// Build virtual files from all `hick:file` tags in the document.
///
/// `copy_registry` maps copy/paste IDs to their resolved text content.
pub fn build_virtual_files(
    doc: &HickDocument,
    copy_registry: &HashMap<String, String>,
) -> Vec<VirtualFile> {
    let mut files = Vec::new();

    for tag in doc.find_tags("file") {
        let path = match tag.get_attribute("path") {
            Some(p) => p.to_string(),
            None => continue,
        };

        let language_id = lang_detect::language_id(&path);
        let mut segments = Vec::new();

        collect_segments(tag, copy_registry, &mut segments);

        files.push(VirtualFile {
            path,
            language_id,
            segments,
        });
    }

    files
}

fn collect_segments(
    tag: &HickTag,
    copy_registry: &HashMap<String, String>,
    segments: &mut Vec<VirtualSegment>,
) {
    for child in &tag.children {
        match child {
            HickNode::Text(text, span) => {
                let dedented = hick_lang::dedent(text, tag.source_column);
                segments.push(VirtualSegment {
                    text: dedented,
                    source_span: *span,
                    source_line: tag.source_line,
                    source_column: tag.source_column,
                });
            }
            HickNode::Tag(child_tag) => {
                if child_tag.name == "copy" || child_tag.name == "paste" {
                    if let Some(select) = child_tag.get_attribute("select") {
                        let id = select.strip_prefix('#').unwrap_or(select);
                        if let Some(content) = copy_registry.get(id) {
                            segments.push(VirtualSegment {
                                text: content.clone(),
                                source_span: child_tag.source_span,
                                source_line: child_tag.source_line,
                                source_column: child_tag.source_column,
                            });
                        }
                    }
                } else {
                    // Recurse into other tags
                    collect_segments(child_tag, copy_registry, segments);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_lang::{HickDocument, HickNode, HickTag, SourceSpan};

    fn make_doc(nodes: Vec<HickNode>) -> HickDocument {
        HickDocument {
            nodes,
            source: String::new(),
            prefix: "hick".to_string(),
            weave_path: None,
        }
    }

    fn make_file_tag(path: &str, children: Vec<HickNode>) -> HickTag {
        HickTag {
            name: "file".to_string(),
            attributes: vec![("path".to_string(), path.to_string())],
            children,
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: Some(SourceSpan::new(0, 10, 1, 0)),
        }
    }

    #[test]
    fn simple_text_file() {
        let doc = make_doc(vec![HickNode::Tag(make_file_tag(
            "src/main.rs",
            vec![HickNode::Text(
                "fn main() {}\n".to_string(),
                Some(SourceSpan::new(20, 33, 2, 0)),
            )],
        ))]);

        let registry = HashMap::new();
        let files = build_virtual_files(&doc, &registry);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[0].language_id, Some("rust"));
        assert_eq!(files[0].content(), "fn main() {}\n");
        assert_eq!(files[0].line_count(), 1);
    }

    #[test]
    fn file_with_copy_paste() {
        let copy_tag = HickTag {
            name: "paste".to_string(),
            attributes: vec![("select".to_string(), "#imports".to_string())],
            children: vec![],
            self_closing: true,
            source_line: 3,
            source_column: 4,
            source_span: Some(SourceSpan::new(40, 70, 3, 4)),
        };

        let doc = make_doc(vec![HickNode::Tag(make_file_tag(
            "app.py",
            vec![
                HickNode::Tag(copy_tag),
                HickNode::Text(
                    "\ndef main():\n    pass\n".to_string(),
                    Some(SourceSpan::new(70, 92, 4, 0)),
                ),
            ],
        ))]);

        let mut registry = HashMap::new();
        registry.insert("imports".to_string(), "import os\n".to_string());

        let files = build_virtual_files(&doc, &registry);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].content(), "import os\n\ndef main():\n    pass\n");
        assert_eq!(files[0].language_id, Some("python"));
    }

    #[test]
    fn file_without_path_is_skipped() {
        let tag = HickTag {
            name: "file".to_string(),
            attributes: vec![],
            children: vec![HickNode::Text("content".to_string(), None)],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
        };
        let doc = make_doc(vec![HickNode::Tag(tag)]);
        let files = build_virtual_files(&doc, &HashMap::new());
        assert!(files.is_empty());
    }

    #[test]
    fn multiple_files() {
        let doc = make_doc(vec![
            HickNode::Tag(make_file_tag(
                "a.rs",
                vec![HickNode::Text("a\n".to_string(), None)],
            )),
            HickNode::Tag(make_file_tag(
                "b.py",
                vec![HickNode::Text("b\n".to_string(), None)],
            )),
        ]);

        let files = build_virtual_files(&doc, &HashMap::new());
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "a.rs");
        assert_eq!(files[1].path, "b.py");
    }

    #[test]
    fn empty_file_line_count() {
        let doc = make_doc(vec![HickNode::Tag(make_file_tag("empty.txt", vec![]))]);
        let files = build_virtual_files(&doc, &HashMap::new());
        assert_eq!(files[0].line_count(), 0);
    }

    #[test]
    fn copy_select_without_hash() {
        let copy_tag = HickTag {
            name: "copy".to_string(),
            attributes: vec![("select".to_string(), "no_hash".to_string())],
            children: vec![],
            self_closing: true,
            source_line: 2,
            source_column: 0,
            source_span: None,
        };

        let doc = make_doc(vec![HickNode::Tag(make_file_tag(
            "out.txt",
            vec![HickNode::Tag(copy_tag)],
        ))]);

        let mut registry = HashMap::new();
        registry.insert("no_hash".to_string(), "resolved\n".to_string());

        let files = build_virtual_files(&doc, &registry);
        assert_eq!(files[0].content(), "resolved\n");
    }
}
