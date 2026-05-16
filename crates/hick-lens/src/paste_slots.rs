//! Language-aware paste slot insertion point computation.
//!
//! Uses Tree-sitter to identify structural boundaries in source content and
//! returns byte offsets where `<hick:paste name="…"/>` markers should be
//! inserted to give agents stable, merge-safe composition points.

use crate::language::{AstElementKindTag, Language};

/// A computed position where a `<hick:paste name="…"/>` element should be inserted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasteInsertion {
    /// Byte offset in the original content string (exclusive: insert before this byte).
    pub byte_offset: usize,
    /// The `name` attribute for the paste slot element.
    pub name: String,
}

/// Compute paste slot insertion positions for the content of a `hick:file` element.
///
/// For known languages (Rust, TypeScript, Python) uses Tree-sitter to find:
/// - `after-imports` — after the last top-level import block
/// - `after-fn-{name}` — after each top-level function/class/impl/struct/enum/trait
/// - `impl-{Name}-body` — inside the body of impl/class/trait/struct/interface (brace languages)
/// - `module-footer` — at the very end of the file content
///
/// For unknown file types returns `file-header` at offset 0 and `module-footer` at the end.
///
/// The insertions are sorted by `byte_offset` (stable sort, preserving relative order
/// of insertions at the same position).
pub fn compute_insertions(file_path: &str, content: &str) -> Vec<PasteInsertion> {
    let Some(lang) = Language::from_path(file_path) else {
        return unknown_insertions(content);
    };

    compute_for_language(content, lang)
}

fn unknown_insertions(content: &str) -> Vec<PasteInsertion> {
    vec![
        PasteInsertion {
            byte_offset: 0,
            name: "file-header".into(),
        },
        PasteInsertion {
            byte_offset: content.len(),
            name: "module-footer".into(),
        },
    ]
}

fn compute_for_language(content: &str, lang: Language) -> Vec<PasteInsertion> {
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&lang.ts_language()).is_err() {
        return unknown_insertions(content);
    }

    let Some(tree) = parser.parse(content, None) else {
        return unknown_insertions(content);
    };

    let structural_kinds = lang.structural_kinds();
    let mut raw: Vec<RawElement> = Vec::new();
    walk_for_elements(tree.root_node(), content.as_bytes(), structural_kinds, &mut raw);

    // Filter to top-level: no other element fully contains this one.
    let top_level_indices: Vec<usize> = (0..raw.len())
        .filter(|&i| {
            !raw.iter().enumerate().any(|(j, other)| {
                j != i
                    && other.start_byte <= raw[i].start_byte
                    && raw[i].end_byte <= other.end_byte
            })
        })
        .collect();

    let mut insertions: Vec<PasteInsertion> = Vec::new();

    // after-imports: after the last top-level import element.
    let last_import_end = top_level_indices
        .iter()
        .filter(|&&i| raw[i].kind == AstElementKindTag::Import)
        .map(|&i| raw[i].end_byte)
        .max();
    if let Some(end) = last_import_end {
        insertions.push(PasteInsertion {
            byte_offset: end,
            name: "after-imports".into(),
        });
    }

    // after-fn-{name} and impl-{Name}-body for each non-import top-level element.
    let uses_braces = !matches!(lang, Language::Python);
    for &i in &top_level_indices {
        let elem = &raw[i];
        if elem.kind == AstElementKindTag::Import {
            continue;
        }

        let slot_name = format!("after-fn-{}", elem.name);
        insertions.push(PasteInsertion {
            byte_offset: elem.end_byte,
            name: slot_name,
        });

        // Body slot for container types in brace-based languages.
        let is_container = matches!(
            elem.kind,
            AstElementKindTag::Impl
                | AstElementKindTag::Class
                | AstElementKindTag::Trait
                | AstElementKindTag::Struct
                | AstElementKindTag::Interface
        );
        if is_container && uses_braces {
            if let Some(brace_pos) =
                find_last_brace(content.as_bytes(), elem.start_byte, elem.end_byte)
            {
                insertions.push(PasteInsertion {
                    byte_offset: brace_pos,
                    name: format!("impl-{}-body", elem.name),
                });
            }
        }
    }

    // module-footer at the very end.
    insertions.push(PasteInsertion {
        byte_offset: content.len(),
        name: "module-footer".into(),
    });

    // Stable sort by byte offset so ordering within the same position is deterministic.
    insertions.sort_by_key(|i| i.byte_offset);
    insertions
}

/// Scan backwards from `end` to find the last `}` in `content[start..end]`.
fn find_last_brace(content: &[u8], start: usize, end: usize) -> Option<usize> {
    let end = end.min(content.len());
    let mut pos = end;
    while pos > start {
        pos -= 1;
        if content[pos] == b'}' {
            return Some(pos);
        }
    }
    None
}

#[derive(Debug)]
struct RawElement {
    kind: AstElementKindTag,
    name: String,
    start_byte: usize,
    end_byte: usize,
}

fn walk_for_elements(
    node: tree_sitter::Node,
    source: &[u8],
    kinds: &[crate::language::StructuralKind],
    out: &mut Vec<RawElement>,
) {
    let node_kind = node.kind();

    if let Some(sk) = kinds.iter().find(|k| k.node_kind == node_kind) {
        let name = node
            .child_by_field_name(sk.name_field)
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("")
            .to_string();

        out.push(RawElement {
            kind: sk.element_kind,
            name,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_elements(child, source, kinds, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_extension_gets_header_and_footer() {
        let insertions = compute_insertions("file.xyz", "some text\n");
        assert_eq!(insertions.len(), 2);
        assert_eq!(insertions[0].name, "file-header");
        assert_eq!(insertions[0].byte_offset, 0);
        assert_eq!(insertions[1].name, "module-footer");
        assert_eq!(insertions[1].byte_offset, 10);
    }

    #[test]
    fn rust_functions_get_after_fn_slots() {
        let content = "fn foo() {}\nfn bar() {}\n";
        let insertions = compute_insertions("src/lib.rs", content);
        let names: Vec<&str> = insertions.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"after-fn-foo"), "missing after-fn-foo: {names:?}");
        assert!(names.contains(&"after-fn-bar"), "missing after-fn-bar: {names:?}");
        assert!(names.contains(&"module-footer"), "missing module-footer: {names:?}");
    }

    #[test]
    fn rust_impl_gets_body_slot() {
        let content = "struct Foo;\nimpl Foo {\n    fn bar(&self) {}\n}\n";
        let insertions = compute_insertions("src/lib.rs", content);
        let names: Vec<&str> = insertions.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"impl-Foo-body"), "missing impl-Foo-body: {names:?}");
        assert!(names.contains(&"after-fn-Foo"), "missing after-fn-Foo: {names:?}");
    }

    #[test]
    fn rust_imports_get_after_imports_slot() {
        let content = "use std::io;\nuse std::fmt;\n\nfn main() {}\n";
        let insertions = compute_insertions("src/main.rs", content);
        let names: Vec<&str> = insertions.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"after-imports"), "missing after-imports: {names:?}");
    }

    #[test]
    fn nested_functions_not_duplicated_as_top_level() {
        // fn bar is nested inside impl Foo — only impl-level slot, not a separate after-fn-bar
        let content = "impl Foo {\n    fn bar(&self) {}\n    fn baz(&self) {}\n}\n";
        let insertions = compute_insertions("src/lib.rs", content);
        let names: Vec<&str> = insertions.iter().map(|i| i.name.as_str()).collect();
        // after-fn-Foo is top-level
        assert!(names.contains(&"after-fn-Foo"), "missing after-fn-Foo: {names:?}");
        // bar and baz are nested — should NOT appear as top-level after-fn slots
        assert!(!names.contains(&"after-fn-bar"), "unexpected after-fn-bar: {names:?}");
        assert!(!names.contains(&"after-fn-baz"), "unexpected after-fn-baz: {names:?}");
    }

    #[test]
    fn python_class_no_body_slot() {
        let content = "class Foo:\n    def bar(self):\n        pass\n";
        let insertions = compute_insertions("app.py", content);
        let names: Vec<&str> = insertions.iter().map(|i| i.name.as_str()).collect();
        // No impl-*-body for Python (no braces)
        assert!(!names.iter().any(|n| n.starts_with("impl-")), "unexpected impl slot: {names:?}");
        assert!(names.contains(&"after-fn-Foo"), "missing after-fn-Foo: {names:?}");
    }

    #[test]
    fn typescript_class_gets_body_slot() {
        let content = "class App {\n    run(): void {}\n}\n";
        let insertions = compute_insertions("src/app.ts", content);
        let names: Vec<&str> = insertions.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"impl-App-body"), "missing impl-App-body: {names:?}");
    }

    #[test]
    fn module_footer_always_last() {
        let content = "fn foo() {}\n";
        let insertions = compute_insertions("a.rs", content);
        assert_eq!(insertions.last().unwrap().name, "module-footer");
    }

    #[test]
    fn insertions_sorted_by_byte_offset() {
        let content = "use std::io;\nfn foo() {}\nfn bar() {}\n";
        let insertions = compute_insertions("a.rs", content);
        let offsets: Vec<usize> = insertions.iter().map(|i| i.byte_offset).collect();
        let mut sorted = offsets.clone();
        sorted.sort();
        assert_eq!(offsets, sorted, "insertions not sorted by byte offset");
    }

    #[test]
    fn slot_names_stable_across_unrelated_renames() {
        // Renaming fn bar should not change the slot for fn foo.
        let content_v1 = "fn foo() {}\nfn bar() {}\n";
        let content_v2 = "fn foo() {}\nfn renamed() {}\n";

        let ins_v1 = compute_insertions("a.rs", content_v1);
        let ins_v2 = compute_insertions("a.rs", content_v2);

        let foo_v1 = ins_v1.iter().find(|i| i.name == "after-fn-foo").unwrap();
        let foo_v2 = ins_v2.iter().find(|i| i.name == "after-fn-foo").unwrap();

        // The slot name is the same. The byte offset may differ if sizes change,
        // but names are symbol-based, not position-based.
        assert_eq!(foo_v1.name, foo_v2.name);
    }
}
