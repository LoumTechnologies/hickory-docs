//! Language detection and tree-sitter grammar selection.

use tree_sitter::Language as TsLanguage;

/// Supported output languages for tree-sitter parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    TypeScript,
    Tsx,
    Python,
}

impl Language {
    /// Get the tree-sitter language grammar.
    pub fn ts_language(&self) -> TsLanguage {
        match self {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
        }
    }

    /// Detect language from a file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            "ts" => Some(Language::TypeScript),
            "tsx" => Some(Language::Tsx),
            "py" => Some(Language::Python),
            _ => None,
        }
    }

    /// Detect language from a file path.
    pub fn from_path(path: &str) -> Option<Self> {
        let ext = path.rsplit('.').next()?;
        Self::from_extension(ext)
    }

    /// Tree-sitter node kinds that represent structural elements (functions, structs, etc.).
    pub fn structural_kinds(&self) -> &[StructuralKind] {
        match self {
            Language::Rust => RUST_KINDS,
            Language::TypeScript | Language::Tsx => TS_KINDS,
            Language::Python => PYTHON_KINDS,
        }
    }
}

/// Maps a tree-sitter node kind to an `AstElementKind` and specifies
/// which child field holds the element's name.
#[derive(Debug, Clone)]
pub struct StructuralKind {
    /// Tree-sitter node kind string (e.g. "function_item").
    pub node_kind: &'static str,
    /// The `AstElementKind` this maps to.
    pub element_kind: AstElementKindTag,
    /// Field name for the identifier child (e.g. "name").
    pub name_field: &'static str,
}

/// Tags for structural element kinds (mirrors `AstElementKind` but without data).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstElementKindTag {
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

// ---------------------------------------------------------------------------
// Language-specific structural kind tables
// ---------------------------------------------------------------------------

static RUST_KINDS: &[StructuralKind] = &[
    StructuralKind {
        node_kind: "function_item",
        element_kind: AstElementKindTag::Function,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "struct_item",
        element_kind: AstElementKindTag::Struct,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "enum_item",
        element_kind: AstElementKindTag::Enum,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "trait_item",
        element_kind: AstElementKindTag::Trait,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "impl_item",
        element_kind: AstElementKindTag::Impl,
        name_field: "type",
    },
    StructuralKind {
        node_kind: "const_item",
        element_kind: AstElementKindTag::Const,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "type_item",
        element_kind: AstElementKindTag::TypeAlias,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "mod_item",
        element_kind: AstElementKindTag::Module,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "use_declaration",
        element_kind: AstElementKindTag::Import,
        name_field: "argument",
    },
];

static TS_KINDS: &[StructuralKind] = &[
    StructuralKind {
        node_kind: "function_declaration",
        element_kind: AstElementKindTag::Function,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "class_declaration",
        element_kind: AstElementKindTag::Class,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "interface_declaration",
        element_kind: AstElementKindTag::Interface,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "type_alias_declaration",
        element_kind: AstElementKindTag::TypeAlias,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "enum_declaration",
        element_kind: AstElementKindTag::Enum,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "import_statement",
        element_kind: AstElementKindTag::Import,
        name_field: "source",
    },
];

static PYTHON_KINDS: &[StructuralKind] = &[
    StructuralKind {
        node_kind: "function_definition",
        element_kind: AstElementKindTag::Function,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "class_definition",
        element_kind: AstElementKindTag::Class,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "import_statement",
        element_kind: AstElementKindTag::Import,
        name_field: "name",
    },
    StructuralKind {
        node_kind: "import_from_statement",
        element_kind: AstElementKindTag::Import,
        name_field: "module_name",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::Tsx));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("java"), None);
    }

    #[test]
    fn detect_language_from_path() {
        assert_eq!(Language::from_path("src/main.rs"), Some(Language::Rust));
        assert_eq!(Language::from_path("app/index.tsx"), Some(Language::Tsx));
        assert_eq!(Language::from_path("script.py"), Some(Language::Python));
    }

    #[test]
    fn ts_language_parses_rust() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&Language::Rust.ts_language()).unwrap();
        let tree = parser.parse("fn main() {}", None).unwrap();
        assert_eq!(tree.root_node().kind(), "source_file");
    }

    #[test]
    fn ts_language_parses_typescript() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&Language::TypeScript.ts_language())
            .unwrap();
        let tree = parser.parse("function greet(): void {}", None).unwrap();
        assert_eq!(tree.root_node().kind(), "program");
    }

    #[test]
    fn ts_language_parses_python() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&Language::Python.ts_language())
            .unwrap();
        let tree = parser.parse("def greet():\n    pass\n", None).unwrap();
        assert_eq!(tree.root_node().kind(), "module");
    }
}
