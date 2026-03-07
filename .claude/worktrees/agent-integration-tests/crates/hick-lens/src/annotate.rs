//! Virtual structural annotations for `.hick` files.
//!
//! Takes an AST mapping and produces a virtual enriched view where structural
//! elements appear as `hick:fn`, `hick:class`, etc. These are never written
//! to disk — they exist only in query responses (e.g. via hick-mcp).

use crate::mapping::{AstElement, AstElementKind, AstMapping};

/// A virtual annotation marking a structural element's boundaries.
#[derive(Debug, Clone)]
pub struct Annotation {
    /// Virtual tag name (e.g. "hick:fn", "hick:struct").
    pub tag: String,
    /// Element name.
    pub name: String,
    /// Byte offset in the output where this element starts.
    pub output_start: usize,
    /// Byte offset in the output where this element ends.
    pub output_end: usize,
    /// Line range (0-based).
    pub start_line: usize,
    pub end_line: usize,
}

/// Generate virtual annotations from an AST mapping.
///
/// Returns annotations sorted by output position. These represent structural
/// elements found by tree-sitter, tagged with hick-style names.
pub fn annotate(mapping: &AstMapping) -> Vec<Annotation> {
    mapping
        .elements()
        .iter()
        .map(|e| Annotation {
            tag: virtual_tag_name(&e.kind),
            name: e.name.clone(),
            output_start: e.output_start,
            output_end: e.output_end,
            start_line: e.start_line,
            end_line: e.end_line,
        })
        .collect()
}

/// Build a tree-structured summary of the output file.
///
/// Returns a multi-line string showing the structural elements with their
/// line ranges, suitable for tool responses.
pub fn structure_summary(mapping: &AstMapping, depth: Option<usize>) -> String {
    let depth = depth.unwrap_or(usize::MAX);
    let mut lines = Vec::new();

    for element in mapping.elements() {
        format_element(&mut lines, element, 0, depth);
    }

    lines.join("\n")
}

fn format_element(lines: &mut Vec<String>, element: &AstElement, indent: usize, _max_depth: usize) {
    let prefix = "  ".repeat(indent);
    let kind_str = kind_label(&element.kind);
    lines.push(format!(
        "{}{} {} [L{}-L{}]",
        prefix,
        kind_str,
        element.name,
        element.start_line + 1,
        element.end_line + 1,
    ));
}

fn virtual_tag_name(kind: &AstElementKind) -> String {
    match kind {
        AstElementKind::Function => "hick:fn".to_string(),
        AstElementKind::Struct => "hick:struct".to_string(),
        AstElementKind::Enum => "hick:enum".to_string(),
        AstElementKind::Trait => "hick:trait".to_string(),
        AstElementKind::Impl => "hick:impl".to_string(),
        AstElementKind::Class => "hick:class".to_string(),
        AstElementKind::Interface => "hick:interface".to_string(),
        AstElementKind::Import => "hick:import".to_string(),
        AstElementKind::Const => "hick:const".to_string(),
        AstElementKind::TypeAlias => "hick:type".to_string(),
        AstElementKind::Module => "hick:mod".to_string(),
    }
}

fn kind_label(kind: &AstElementKind) -> &'static str {
    match kind {
        AstElementKind::Function => "fn",
        AstElementKind::Struct => "struct",
        AstElementKind::Enum => "enum",
        AstElementKind::Trait => "trait",
        AstElementKind::Impl => "impl",
        AstElementKind::Class => "class",
        AstElementKind::Interface => "interface",
        AstElementKind::Import => "import",
        AstElementKind::Const => "const",
        AstElementKind::TypeAlias => "type",
        AstElementKind::Module => "mod",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use hick_flow::provenance::ProvenanceMap;

    #[test]
    fn annotate_rust_source() {
        let source = "fn greet() {}\nstruct Config {\n    x: i32,\n}\n";
        let mapping = AstMapping::build(source, ProvenanceMap::new(), Language::Rust).unwrap();
        let annotations = annotate(&mapping);

        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].tag, "hick:fn");
        assert_eq!(annotations[0].name, "greet");
        assert_eq!(annotations[1].tag, "hick:struct");
        assert_eq!(annotations[1].name, "Config");
    }

    #[test]
    fn structure_summary_output() {
        let source = "fn main() {}\nfn helper() {}\n";
        let mapping = AstMapping::build(source, ProvenanceMap::new(), Language::Rust).unwrap();
        let summary = structure_summary(&mapping, None);

        assert!(summary.contains("fn main"));
        assert!(summary.contains("fn helper"));
    }
}
