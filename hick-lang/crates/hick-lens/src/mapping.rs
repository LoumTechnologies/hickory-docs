//! AST mapping: parse output with tree-sitter and correlate structural elements
//! with provenance spans.

use std::sync::Arc;

use hick_flow::node::SourceOrigin;
use hick_flow::provenance::ProvenanceMap;

use crate::language::{AstElementKindTag, Language};

/// A structural element found in the output AST.
#[derive(Debug, Clone)]
pub struct AstElement {
    /// What kind of element this is.
    pub kind: AstElementKind,
    /// Element name (e.g. function name, struct name).
    pub name: String,
    /// Byte range in the output file (start inclusive, end exclusive).
    pub output_start: usize,
    pub output_end: usize,
    /// Line range in the output file (0-based).
    pub start_line: usize,
    pub end_line: usize,
}

/// The kind of structural element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstElementKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Class,
    Interface,
    Import,
    Const,
    TypeAlias,
    Module,
}

impl From<AstElementKindTag> for AstElementKind {
    fn from(tag: AstElementKindTag) -> Self {
        match tag {
            AstElementKindTag::Function => AstElementKind::Function,
            AstElementKindTag::Struct => AstElementKind::Struct,
            AstElementKindTag::Enum => AstElementKind::Enum,
            AstElementKindTag::Trait => AstElementKind::Trait,
            AstElementKindTag::Impl => AstElementKind::Impl,
            AstElementKindTag::Class => AstElementKind::Class,
            AstElementKindTag::Interface => AstElementKind::Interface,
            AstElementKindTag::Import => AstElementKind::Import,
            AstElementKindTag::Const => AstElementKind::Const,
            AstElementKindTag::TypeAlias => AstElementKind::TypeAlias,
            AstElementKindTag::Module => AstElementKind::Module,
        }
    }
}

/// Result of mapping an output file's AST to provenance origins.
#[derive(Debug, Clone)]
pub struct AstMapping {
    /// Structural elements found in the output.
    pub elements: Vec<AstElement>,
    /// The provenance map used for this mapping.
    provenance: ProvenanceMap,
    /// Source text of the output file.
    source: Arc<str>,
}

impl AstMapping {
    /// Build an AST mapping from output text and its provenance map.
    ///
    /// Parses the output with tree-sitter for the given language, walks the
    /// AST to find structural elements, and records their positions.
    pub fn build(
        output: &str,
        provenance: ProvenanceMap,
        language: Language,
    ) -> Result<Self, AstMappingError> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language.ts_language())
            .map_err(|e| AstMappingError::LanguageError(e.to_string()))?;

        let tree = parser
            .parse(output, None)
            .ok_or(AstMappingError::ParseFailed)?;

        let structural_kinds = language.structural_kinds();
        let mut elements = Vec::new();

        walk_tree(tree.root_node(), output, structural_kinds, &mut elements);

        Ok(AstMapping {
            elements,
            provenance,
            source: Arc::from(output),
        })
    }

    /// Get all structural elements.
    pub fn elements(&self) -> &[AstElement] {
        &self.elements
    }

    /// Get the provenance map.
    pub fn provenance(&self) -> &ProvenanceMap {
        &self.provenance
    }

    /// Get the output source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Find elements by name.
    pub fn find_by_name(&self, name: &str) -> Vec<&AstElement> {
        self.elements.iter().filter(|e| e.name == name).collect()
    }

    /// Find elements by kind.
    pub fn find_by_kind(&self, kind: &AstElementKind) -> Vec<&AstElement> {
        self.elements.iter().filter(|e| &e.kind == kind).collect()
    }

    /// Find the provenance origins that overlap an element's byte range.
    pub fn origins_for_element(&self, element: &AstElement) -> Vec<&SourceOrigin> {
        self.provenance
            .spans_in_range(element.output_start, element.output_end)
            .into_iter()
            .map(|s| &s.origin)
            .collect()
    }
}

/// Errors during AST mapping.
#[derive(Debug, thiserror::Error)]
pub enum AstMappingError {
    #[error("failed to set tree-sitter language: {0}")]
    LanguageError(String),
    #[error("tree-sitter parse failed")]
    ParseFailed,
}

/// Recursively walk the tree-sitter AST looking for structural elements.
fn walk_tree(
    node: tree_sitter::Node,
    source: &str,
    kinds: &[crate::language::StructuralKind],
    elements: &mut Vec<AstElement>,
) {
    let node_kind = node.kind();

    if let Some(sk) = kinds.iter().find(|k| k.node_kind == node_kind) {
        let name = node
            .child_by_field_name(sk.name_field)
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("")
            .to_string();

        elements.push(AstElement {
            kind: sk.element_kind.into(),
            name,
            output_start: node.start_byte(),
            output_end: node.end_byte(),
            start_line: node.start_position().row,
            end_line: node.end_position().row,
        });
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_tree(child, source, kinds, elements);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_flow::node::SourceOrigin;
    use hick_flow::provenance::{ProvenanceMap, ProvenanceSpan};
    use hick_lang::SourceSpan;

    #[test]
    fn parse_rust_finds_functions_and_structs() {
        let source = r#"
fn greet() {
    println!("hello");
}

struct Config {
    name: String,
}

fn main() {
    let c = Config { name: "test".into() };
    greet();
}
"#;
        let map = ProvenanceMap::new();
        let mapping = AstMapping::build(source, map, Language::Rust).unwrap();

        let fns: Vec<&AstElement> = mapping.find_by_kind(&AstElementKind::Function);
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].name, "greet");
        assert_eq!(fns[1].name, "main");

        let structs = mapping.find_by_kind(&AstElementKind::Struct);
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].name, "Config");
    }

    #[test]
    fn parse_typescript_finds_functions_and_classes() {
        let source = r#"
function greet(): void {
    console.log("hello");
}

class App {
    run() {}
}
"#;
        let map = ProvenanceMap::new();
        let mapping = AstMapping::build(source, map, Language::TypeScript).unwrap();

        let fns = mapping.find_by_kind(&AstElementKind::Function);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "greet");

        let classes = mapping.find_by_kind(&AstElementKind::Class);
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "App");
    }

    #[test]
    fn parse_python_finds_functions_and_classes() {
        let source = "def greet():\n    pass\n\nclass Config:\n    pass\n";
        let map = ProvenanceMap::new();
        let mapping = AstMapping::build(source, map, Language::Python).unwrap();

        let fns = mapping.find_by_kind(&AstElementKind::Function);
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].name, "greet");

        let classes = mapping.find_by_kind(&AstElementKind::Class);
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "Config");
    }

    #[test]
    fn find_by_name() {
        let source = "fn foo() {}\nfn bar() {}\nfn foo_bar() {}\n";
        let mapping = AstMapping::build(source, ProvenanceMap::new(), Language::Rust).unwrap();

        let results = mapping.find_by_name("foo");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "foo");
    }

    #[test]
    fn origins_for_element_with_provenance() {
        let source = "fn greet() {}\n";
        let mut prov = ProvenanceMap::new();
        prov.push(ProvenanceSpan {
            output_start: 0,
            output_end: 14,
            origin: SourceOrigin::Literal {
                file: Arc::from("test.hick"),
                span: SourceSpan::new(10, 24, 3, 0),
            },
        });

        let mapping = AstMapping::build(source, prov, Language::Rust).unwrap();
        let fns = mapping.find_by_kind(&AstElementKind::Function);
        assert_eq!(fns.len(), 1);

        let origins = mapping.origins_for_element(fns[0]);
        assert_eq!(origins.len(), 1);
        match origins[0] {
            SourceOrigin::Literal { file, .. } => assert_eq!(&**file, "test.hick"),
            other => panic!("Expected Literal, got {:?}", other),
        }
    }
}
