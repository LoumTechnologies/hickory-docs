//! Structural selector engine for querying output elements.
//!
//! Extends the existing `#id` / `.class` pattern with structural queries:
//! - `fn greet` — function named "greet"
//! - `struct Config` — struct named "Config"
//! - `class App` — class named "App"
//! - `#my-id` — existing ID selector (delegates to paste resolution)
//! - `.my-class` — existing class selector

use crate::mapping::{AstElement, AstElementKind, AstMapping};

/// A parsed selector for querying elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// Match a function by name: `fn name`
    Function(String),
    /// Match a struct by name: `struct Name`
    Struct(String),
    /// Match an enum by name: `enum Name`
    Enum(String),
    /// Match a trait by name: `trait Name`
    Trait(String),
    /// Match an impl by type: `impl Name`
    Impl(String),
    /// Match a class by name: `class Name`
    Class(String),
    /// Match an interface by name: `interface Name`
    Interface(String),
    /// Match a const by name: `const NAME`
    Const(String),
    /// Match a type alias by name: `type Name`
    TypeAlias(String),
    /// Match a module by name: `mod name`
    Module(String),
    /// Match by element name regardless of kind.
    ByName(String),
    /// CSS ID selector: `#id`
    Id(String),
    /// CSS class selector: `.class`
    CssClass(String),
}

impl Selector {
    /// Parse a selector string.
    ///
    /// Supported formats:
    /// - `fn name` / `struct Name` / `class Name` / etc.
    /// - `#id` — ID selector
    /// - `.class` — class selector
    /// - `name` — match by name regardless of kind
    pub fn parse(input: &str) -> Result<Self, SelectorParseError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(SelectorParseError::Empty);
        }

        if let Some(id) = input.strip_prefix('#') {
            return Ok(Selector::Id(id.to_string()));
        }
        if let Some(class) = input.strip_prefix('.') {
            return Ok(Selector::CssClass(class.to_string()));
        }

        if let Some((keyword, name)) = input.split_once(char::is_whitespace) {
            let name = name.trim().to_string();
            if name.is_empty() {
                return Err(SelectorParseError::MissingName(keyword.to_string()));
            }
            return match keyword {
                "fn" => Ok(Selector::Function(name)),
                "struct" => Ok(Selector::Struct(name)),
                "enum" => Ok(Selector::Enum(name)),
                "trait" => Ok(Selector::Trait(name)),
                "impl" => Ok(Selector::Impl(name)),
                "class" => Ok(Selector::Class(name)),
                "interface" => Ok(Selector::Interface(name)),
                "const" => Ok(Selector::Const(name)),
                "type" => Ok(Selector::TypeAlias(name)),
                "mod" => Ok(Selector::Module(name)),
                _ => Err(SelectorParseError::UnknownKeyword(keyword.to_string())),
            };
        }

        // Single word: match by name
        Ok(Selector::ByName(input.to_string()))
    }

    /// Check if an AST element matches this selector.
    pub fn matches(&self, element: &AstElement) -> bool {
        match self {
            Selector::Function(name) => {
                element.kind == AstElementKind::Function && element.name == *name
            }
            Selector::Struct(name) => {
                element.kind == AstElementKind::Struct && element.name == *name
            }
            Selector::Enum(name) => element.kind == AstElementKind::Enum && element.name == *name,
            Selector::Trait(name) => element.kind == AstElementKind::Trait && element.name == *name,
            Selector::Impl(name) => element.kind == AstElementKind::Impl && element.name == *name,
            Selector::Class(name) => element.kind == AstElementKind::Class && element.name == *name,
            Selector::Const(name) => element.kind == AstElementKind::Const && element.name == *name,
            Selector::TypeAlias(name) => {
                element.kind == AstElementKind::TypeAlias && element.name == *name
            }
            Selector::Module(name) => {
                element.kind == AstElementKind::Module && element.name == *name
            }
            Selector::Interface(name) => {
                element.kind == AstElementKind::Interface && element.name == *name
            }
            Selector::ByName(name) => element.name == *name,
            // ID and class selectors don't match AST elements — they match provenance
            Selector::Id(_) | Selector::CssClass(_) => false,
        }
    }

    /// Query an AST mapping for elements matching this selector.
    pub fn query<'a>(&self, mapping: &'a AstMapping) -> Vec<SelectorMatch<'a>> {
        mapping
            .elements()
            .iter()
            .filter(|e| self.matches(e))
            .map(|element| SelectorMatch {
                element,
                output_text: &mapping.source()[element.output_start..element.output_end],
            })
            .collect()
    }
}

/// A match result from a selector query.
#[derive(Debug, Clone)]
pub struct SelectorMatch<'a> {
    /// The matched AST element.
    pub element: &'a AstElement,
    /// The output text slice for this element.
    pub output_text: &'a str,
}

/// Errors during selector parsing.
#[derive(Debug, thiserror::Error)]
pub enum SelectorParseError {
    #[error("empty selector")]
    Empty,
    #[error("missing name after keyword '{0}'")]
    MissingName(String),
    #[error("unknown selector keyword '{0}'")]
    UnknownKeyword(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use hick_flow::provenance::ProvenanceMap;

    #[test]
    fn parse_function_selector() {
        assert_eq!(
            Selector::parse("fn greet").unwrap(),
            Selector::Function("greet".to_string())
        );
    }

    #[test]
    fn parse_struct_selector() {
        assert_eq!(
            Selector::parse("struct Config").unwrap(),
            Selector::Struct("Config".to_string())
        );
    }

    #[test]
    fn parse_class_selector() {
        assert_eq!(
            Selector::parse("class App").unwrap(),
            Selector::Class("App".to_string())
        );
    }

    #[test]
    fn parse_id_selector() {
        assert_eq!(
            Selector::parse("#version").unwrap(),
            Selector::Id("version".to_string())
        );
    }

    #[test]
    fn parse_css_class_selector() {
        assert_eq!(
            Selector::parse(".imports").unwrap(),
            Selector::CssClass("imports".to_string())
        );
    }

    #[test]
    fn parse_bare_name() {
        assert_eq!(
            Selector::parse("greet").unwrap(),
            Selector::ByName("greet".to_string())
        );
    }

    #[test]
    fn parse_empty_errors() {
        assert!(Selector::parse("").is_err());
    }

    #[test]
    fn parse_unknown_keyword_errors() {
        assert!(Selector::parse("widget Foo").is_err());
    }

    #[test]
    fn query_finds_function() {
        let source = "fn greet() {}\nfn main() {}\n";
        let mapping = AstMapping::build(source, ProvenanceMap::new(), Language::Rust).unwrap();

        let selector = Selector::parse("fn greet").unwrap();
        let matches = selector.query(&mapping);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].element.name, "greet");
        assert!(matches[0].output_text.contains("fn greet"));
    }

    #[test]
    fn query_by_name_finds_all_kinds() {
        let source = "fn greet() {}\n";
        let mapping = AstMapping::build(source, ProvenanceMap::new(), Language::Rust).unwrap();

        let selector = Selector::parse("greet").unwrap();
        let matches = selector.query(&mapping);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn query_struct_in_rust() {
        let source = "struct Foo {\n    x: i32,\n}\n";
        let mapping = AstMapping::build(source, ProvenanceMap::new(), Language::Rust).unwrap();

        let selector = Selector::parse("struct Foo").unwrap();
        let matches = selector.query(&mapping);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].output_text.contains("struct Foo"));
    }
}
