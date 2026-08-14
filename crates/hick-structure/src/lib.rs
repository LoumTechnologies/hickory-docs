//! Structural navigation: definitions and references, without a language
//! server.
//!
//! ## Why this exists
//!
//! Precise navigation is a solved problem — LSP and SCIP both do it properly
//! — and neither can ship in a download. A language server is a per-language
//! program with a per-language runtime; a SCIP index is a per-language build
//! step. Both are right when the user already has the toolchain, and both are
//! absent on the machine of someone who just installed `hick` to read a
//! document.
//!
//! Tree-sitter is the one option that travels: the grammars are C, compiled
//! into this binary, so structure works offline on a machine with nothing
//! installed.
//!
//! ## What it is honest about
//!
//! Resolution here is **by name**. A reference to `load` links to every
//! definition named `load` that this crate can see. That is right often
//! enough to navigate by and wrong in exactly the ways you would expect:
//! overloads, shadowing, methods on different types sharing a name, dynamic
//! dispatch, and anything imported from outside the files given to it.
//!
//! So the links it produces are labelled `structural`, and the layer above
//! draws them differently from links that were computed exactly
//! (`docs/guarantees/authoring/the-lineage-browser-shows-one-column-per-stage.md`).
//! A guess presented as a fact is worse than no answer; a guess labelled as
//! one is a good place to start looking.

use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser, Tree};

/// A named thing a file defines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    pub name: String,
    /// `function`, `class`, `method`, `struct`, `variable`, …
    pub kind: String,
    /// 0-based, inclusive.
    pub start_line: usize,
    pub end_line: usize,
}

/// A place a name is used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub name: String,
    pub line: usize,
}

/// One file's structure.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStructure {
    pub path: String,
    pub language: String,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
}

/// A reference resolved to the definitions that share its name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuralLink {
    pub from_path: String,
    pub from_line: usize,
    pub to_path: String,
    pub to_line: usize,
    pub name: String,
    /// How many definitions share this name across the analysed files.
    ///
    /// One is the ordinary case. More than one means the link is a guess
    /// among several, and the reader deserves to know that before following
    /// it.
    pub candidates: usize,
}

/// Languages this build can parse. Anything else yields no structure rather
/// than a wrong one.
pub fn language_of(path: &str) -> Option<&'static str> {
    let ext = path.rsplit('.').next()?;
    match ext {
        "py" => Some("python"),
        "rs" => Some("rust"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "ts" => Some("typescript"),
        "tsx" => Some("tsx"),
        _ => None,
    }
}

fn parser_for(language: &str) -> Option<Parser> {
    let mut parser = Parser::new();
    let lang = match language {
        "python" => tree_sitter_python::LANGUAGE.into(),
        "rust" => tree_sitter_rust::LANGUAGE.into(),
        "javascript" => tree_sitter_javascript::LANGUAGE.into(),
        "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "tsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        _ => return None,
    };
    parser.set_language(&lang).ok()?;
    Some(parser)
}

/// Node kinds that introduce a name, per language family.
///
/// Kept as a table rather than a query file: the set is small, it is the same
/// shape in every grammar tree-sitter ships, and a `.scm` query file would be
/// one more thing to keep in step with a grammar upgrade.
fn definition_kind(node_kind: &str) -> Option<&'static str> {
    Some(match node_kind {
        "function_definition" | "function_declaration" | "function_item" => "function",
        "class_definition" | "class_declaration" => "class",
        "method_definition" | "method_declaration" => "method",
        "struct_item" => "struct",
        "enum_item" | "enum_declaration" => "enum",
        "trait_item" => "trait",
        "impl_item" => "impl",
        "interface_declaration" => "interface",
        "type_alias_declaration" | "type_item" => "type",
        "const_item" | "static_item" => "const",
        _ => return None,
    })
}

/// The identifier a definition node names.
fn name_of<'a>(node: Node<'a>, source: &'a str) -> Option<String> {
    // Every grammar here puts the declared name in a `name` field; falling
    // back to the first identifier child covers the few that do not.
    if let Some(child) = node.child_by_field_name("name") {
        return Some(source[child.byte_range()].to_string());
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind().contains("identifier") {
            return Some(source[child.byte_range()].to_string());
        }
    }
    None
}

fn walk(node: Node, source: &str, out: &mut FileStructure, depth: usize) {
    // A depth bound rather than recursion the tree could exhaust: a generated
    // file with pathological nesting must not take the process down.
    if depth > 200 {
        return;
    }
    if let Some(kind) = definition_kind(node.kind())
        && let Some(name) = name_of(node, source)
    {
        out.definitions.push(Definition {
            name,
            kind: kind.to_string(),
            start_line: node.start_position().row,
            end_line: node.end_position().row,
        });
    }

    // References: the callee of a call, and field/attribute access. Bare
    // identifiers are deliberately NOT collected — every parameter, local and
    // keyword would become a "reference", and a navigation list where
    // everything is a hit is a list nobody reads.
    if (node.kind() == "call" || node.kind() == "call_expression")
        && let Some(callee) = node.child_by_field_name("function")
    {
        {
            let text = source[callee.byte_range()].to_string();
            // `a.b.c()` refers to `c`.
            let name = text.rsplit(['.', ':']).next().unwrap_or(&text).to_string();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                out.references.push(Reference {
                    name,
                    line: callee.start_position().row,
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, source, out, depth + 1);
    }
}

/// Parse one file. Returns `None` for a language this build cannot read —
/// never a guess.
pub fn analyze(path: &str, source: &str) -> Option<FileStructure> {
    let language = language_of(path)?;
    let mut parser = parser_for(language)?;
    let tree: Tree = parser.parse(source, None)?;
    let mut out = FileStructure {
        path: path.to_string(),
        language: language.to_string(),
        ..Default::default()
    };
    walk(tree.root_node(), source, &mut out, 0);
    Some(out)
}

/// Resolve references to definitions by name across a set of files.
///
/// Self-links are dropped: a recursive function referring to itself is true
/// and useless to draw. Everything else is kept, with the number of
/// candidates carried along so the caller can say "one of three" rather than
/// implying certainty.
pub fn resolve(files: &[FileStructure]) -> Vec<StructuralLink> {
    let mut by_name: std::collections::HashMap<&str, Vec<(&str, &Definition)>> =
        std::collections::HashMap::new();
    for file in files {
        for def in &file.definitions {
            by_name
                .entry(def.name.as_str())
                .or_default()
                .push((file.path.as_str(), def));
        }
    }

    let mut out = Vec::new();
    for file in files {
        for reference in &file.references {
            let Some(targets) = by_name.get(reference.name.as_str()) else {
                continue;
            };
            for (path, def) in targets {
                let same_place = *path == file.path
                    && reference.line >= def.start_line
                    && reference.line <= def.end_line;
                if same_place {
                    continue;
                }
                out.push(StructuralLink {
                    from_path: file.path.clone(),
                    from_line: reference.line,
                    to_path: (*path).to_string(),
                    to_line: def.start_line,
                    name: reference.name.clone(),
                    candidates: targets.len(),
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PY: &str = r#"
def load(path):
    return open(path).read()

def summarise(path):
    text = load(path)
    return len(text)

class Report:
    def render(self):
        return summarise("x")
"#;

    #[test]
    fn finds_python_definitions_with_their_extent() {
        let s = analyze("a.py", PY).unwrap();
        let names: Vec<_> = s.definitions.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"load"), "{names:?}");
        assert!(names.contains(&"summarise"), "{names:?}");
        assert!(names.contains(&"Report"), "{names:?}");
        assert!(names.contains(&"render"), "{names:?}");

        let load = s.definitions.iter().find(|d| d.name == "load").unwrap();
        assert_eq!(load.kind, "function");
        assert!(
            load.end_line > load.start_line,
            "a definition spans its body"
        );
    }

    #[test]
    fn collects_calls_and_not_every_bare_identifier() {
        let s = analyze("a.py", PY).unwrap();
        let names: Vec<_> = s.references.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"load"));
        assert!(names.contains(&"summarise"));
        // `path` and `text` are locals, not references worth navigating to.
        assert!(!names.contains(&"path"), "{names:?}");
        assert!(!names.contains(&"text"), "{names:?}");
    }

    #[test]
    fn resolves_a_call_to_the_definition_that_shares_its_name() {
        let s = analyze("a.py", PY).unwrap();
        let links = resolve(&[s]);
        let to_load: Vec<_> = links.iter().filter(|l| l.name == "load").collect();
        assert_eq!(to_load.len(), 1, "{links:?}");
        assert_eq!(to_load[0].candidates, 1);
        assert_eq!(to_load[0].to_line, 1, "the def line");
    }

    #[test]
    fn resolves_across_files() {
        let a = analyze("lib.py", "def helper():\n    return 1\n").unwrap();
        let b = analyze("main.py", "def main():\n    return helper()\n").unwrap();
        let links = resolve(&[a, b]);
        let link = links.iter().find(|l| l.name == "helper").unwrap();
        assert_eq!(link.from_path, "main.py");
        assert_eq!(link.to_path, "lib.py");
    }

    #[test]
    fn says_how_many_definitions_a_name_could_mean() {
        // The honest failure mode of name matching: two `render`s, and the
        // link cannot tell which. It reports the ambiguity rather than
        // picking one and looking certain.
        let a = analyze("a.py", "def render():\n    return 1\n").unwrap();
        let b = analyze("b.py", "def render():\n    return 2\n").unwrap();
        let c = analyze("c.py", "def go():\n    return render()\n").unwrap();
        let links = resolve(&[a, b, c]);
        let from_c: Vec<_> = links.iter().filter(|l| l.from_path == "c.py").collect();
        assert_eq!(from_c.len(), 2, "both candidates are offered");
        for link in from_c {
            assert_eq!(link.candidates, 2, "and each says the name is ambiguous");
        }
    }

    #[test]
    fn a_recursive_call_does_not_link_to_itself() {
        let s = analyze("r.py", "def loop(n):\n    return loop(n - 1)\n").unwrap();
        let links = resolve(&[s]);
        assert!(links.is_empty(), "{links:?}");
    }

    #[test]
    fn reads_rust_and_typescript_too() {
        let rust = analyze("m.rs", "fn helper() -> u8 { 1 }\nfn main() { helper(); }\n").unwrap();
        assert_eq!(rust.language, "rust");
        let links = resolve(&[rust]);
        assert_eq!(links.iter().filter(|l| l.name == "helper").count(), 1);

        let ts = analyze(
            "m.ts",
            "export function helper(): number { return 1; }\nfunction main() { helper(); }\n",
        )
        .unwrap();
        assert_eq!(ts.language, "typescript");
        assert!(ts.definitions.iter().any(|d| d.name == "helper"));
    }

    #[test]
    fn an_unknown_language_gets_no_structure_rather_than_a_wrong_one() {
        assert!(analyze("notes.md", "# hello\n").is_none());
        assert!(analyze("Makefile", "all:\n\techo hi\n").is_none());
    }

    #[test]
    fn a_file_that_does_not_parse_still_yields_what_it_can() {
        // Tree-sitter recovers from errors, which is the property that makes
        // this usable while somebody is typing.
        let s = analyze("broken.py", "def good():\n    return 1\n\ndef bad(:\n").unwrap();
        assert!(s.definitions.iter().any(|d| d.name == "good"));
    }
}
