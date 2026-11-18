<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Prove which Rust declaration a compiled test will run

This reading is for a maintainer changing selected-test Debug admission. A unique test-name suffix is not source ownership: a disabled declaration can share its leaf name with a compiled sibling. The issuer retains the selected compiler source point. Preparation joins that point to a bounded, known-active module graph and then to the exact emitted image’s qualified function and source lines. Unknown mappings refuse before launching a substitute.

## Retain the selected declaration and its module route

The pure model accepts the existing Rust detector’s exact identifier point, including UTF16 columns. It keeps inline namespaces and declaration lines. Default external modules use Rust’s ordinary file rules; unknown cfg, path overrides, includes and ambiguous files are not guessed. Parsing, syntax nodes, depth, source files and total bytes are bounded. All target graphs share a read-count/byte/time budget, so repeated roots cannot multiply a per-graph limit. These checks prove a selected package source route, not a hermetic dependency or build-script closure.

<ui:copy id="rust-test-source-declaration">
use tree_sitter::{Node, Parser};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustTestDeclaration {
    pub name: String,
    pub namespace: Vec<String>,
    pub first_line: u32,
    pub last_line: u32,
    pub ignored: bool,
}
fn parse(source: &str) -> Result<tree_sitter::Tree, String> {
    if source.len() > 1048576 {
        return Err("Rust declaration proof exceeds its source budget".into());
    }
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|e| e.to_string())?;
    #[allow(deprecated)]
    parser.set_timeout_micros(1_000_000);
    let tree = parser
        .parse(source, None)
        .ok_or("Rust declaration proof timed out")?;
    if tree.root_node().has_error() {
        return Err("Rust declaration proof requires complete syntax".into());
    }
    Ok(tree)
}
fn text<'a>(node: Node<'_>, source: &'a str) -> Result<&'a str, String> {
    source
        .get(node.byte_range())
        .ok_or_else(|| "Rust declaration byte range changed".into())
}
pub fn builtin_crate_harness(source: &str) -> Result<(), String> {
    let tree = parse(source)?;
    let root = tree.root_node();
    let mut cursor = root.walk();
    for node in root.named_children(&mut cursor) {
        let value = text(node, source)?.trim();
        if value.starts_with("#![") {
            let body = value
                .strip_prefix("#![")
                .and_then(|v| v.strip_suffix(']'))
                .ok_or("Malformed Rust crate attribute")?
                .trim();
            let name = body
                .split(|c: char| c == '(' || c == '=' || c.is_whitespace())
                .next()
                .unwrap_or("");
            if !matches!(
                name,
                "allow"
                    | "warn"
                    | "deny"
                    | "forbid"
                    | "doc"
                    | "cfg"
                    | "no_std"
                    | "recursion_limit"
                    | "type_length_limit"
            ) {
                return Err(format!(
                    "Rust crate attribute {name} does not prove the default built-in test harness"
                ));
            }
        }
    }
    Ok(())
}
pub fn selected_declaration(
    source: &str,
    line: u32,
    character: u32,
    name: &str,
) -> Result<RustTestDeclaration, String> {
    let scan = super::candidates::rust("rust", source)?;
    if !scan.syntax_complete
        || !scan
            .candidates
            .iter()
            .any(|c| c.line == line && c.character == character && c.name == name)
    {
        return Err(
            "The selected Rust test declaration no longer matches the admitted source point".into(),
        );
    }
    let tree = parse(source)?;
    let mut stack = vec![tree.root_node()];
    let mut visits = 0;
    let mut found = None;
    let mut matching_ranges = Vec::new();
    while let Some(node) = stack.pop() {
        visits += 1;
        if visits > 200000 {
            return Err("Rust declaration proof node budget exceeded".into());
        }
        if node.kind() == "function_item" {
            if let Some(identifier) = node.child_by_field_name("name") {
                let position = identifier.start_position();
                let start = source[..identifier.start_byte()]
                    .rfind('\n')
                    .map_or(0, |n| n + 1);
                let column = source[start..identifier.start_byte()]
                    .encode_utf16()
                    .count();
                if text(identifier, source)?.trim_start_matches("r#")
                    == name.trim_start_matches("r#")
                {
                    matching_ranges.push((
                        node.start_position().row as u32 + 1,
                        node.end_position().row as u32 + 1,
                    ))
                }
                if position.row + 1 == line as usize
                    && column == character as usize
                    && text(identifier, source)? == name
                {
                    let mut namespace = Vec::new();
                    let mut parent = node.parent();
                    while let Some(at) = parent {
                        if at.kind() == "function_item" {
                            return Err(
                                "Nested local Rust functions are not proven test declarations"
                                    .into(),
                            );
                        }
                        if at.kind() == "mod_item" {
                            let id = at
                                .child_by_field_name("name")
                                .ok_or("Rust module identity is missing")?;
                            namespace.push(text(id, source)?.trim_start_matches("r#").to_owned())
                        }
                        parent = at.parent()
                    }
                    namespace.reverse();
                    let mut ignored = false;
                    let mut first_line = node.start_position().row as u32 + 1;
                    let mut before = node.prev_named_sibling();
                    while let Some(at) = before {
                        if at.kind() == "attribute_item" {
                            first_line = at.start_position().row as u32 + 1;
                            let value = text(at, source)?
                                .trim()
                                .trim_start_matches("#[")
                                .trim_end_matches(']')
                                .trim();
                            if value
                                .split(|c| c == '(' || c == '=')
                                .next()
                                .unwrap_or("")
                                .trim()
                                == "ignore"
                            {
                                ignored = true
                            }
                        } else if !matches!(at.kind(), "line_comment" | "block_comment") {
                            break;
                        }
                        before = at.prev_named_sibling()
                    }
                    let value = RustTestDeclaration {
                        name: name.trim_start_matches("r#").into(),
                        namespace,
                        first_line,
                        last_line: node.end_position().row as u32 + 1,
                        ignored,
                    };
                    if found.replace(value).is_some() {
                        return Err("Selected Rust declaration is ambiguous".into());
                    }
                }
            }
        }
        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }
    let found =
        found.ok_or("Selected Rust declaration was not found at its admitted source point")?;
    if matching_ranges
        .iter()
        .filter(|(first, last)| *first <= found.last_line && *last >= found.first_line)
        .count()
        != 1
    {
        return Err("Selected Rust declaration shares source lines with another matching function; compiled line metadata is insufficient".into());
    }
    Ok(found)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn declaration_identity_keeps_inline_namespace_and_exact_point() {
        let source = "#[cfg(test)]\nmod cases {\n #[test]\n fn chosen(){assert!(true)}\n}\n";
        let value = selected_declaration(source, 4, 4, "chosen").unwrap();
        assert_eq!(value.namespace, vec!["cases"]);
        assert_eq!((value.first_line, value.last_line), (3, 4));
        assert!(selected_declaration(source, 3, 4, "chosen").is_err());
        assert!(selected_declaration(source, 4, 5, "chosen").is_err())
    }
    #[test]
    fn ignored_and_unicode_points_are_data_not_name_substitutions() {
        let source = "mod tests {#[test] #[ignore] fn r#type() {}}";
        let scan = super::super::candidates::rust("rust", source).unwrap();
        let candidate = &scan.candidates[0];
        let value =
            selected_declaration(source, candidate.line, candidate.character, &candidate.name)
                .unwrap();
        assert_eq!(value.name, "type");
        assert!(value.ignored);
        assert_eq!(value.namespace, vec!["tests"])
    }
    #[test]
    fn custom_crate_runners_are_refused_before_a_binary_is_listed() {
        for source in [
            "#![feature(custom_test_frameworks)]\nfn main(){}",
            "#![test_runner(custom)]\nfn main(){}",
            "#![cfg_attr(test,test_runner(custom))]\nfn main(){}",
            "#![no_main]\nfn main(){}",
        ] {
            assert!(builtin_crate_harness(source).is_err(), "{source}")
        }
        builtin_crate_harness("#![allow(dead_code)]\n#![cfg(test)]\n#[test]fn chosen(){}").unwrap()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustModuleTest {
    pub file: std::path::PathBuf,
    pub line: u32,
    pub character: u32,
    pub name: String,
}
#[derive(Debug, Default)]
pub struct RustModuleTests {
    pub tests: std::collections::BTreeMap<String, RustModuleTest>,
}
fn cfg_value(value: &str, depth: usize) -> Option<bool> {
    if depth > 32 || value.len() > 8192 {
        return None;
    }
    let value = value.trim();
    match value {
        "test" | "true" => return Some(true),
        "false" => return Some(false),
        _ => {}
    }
    let (open, close) = (value.find('(')?, value.strip_suffix(')')?);
    let name = value[..open].trim();
    if !matches!(name, "all" | "any" | "not") {
        return None;
    }
    let body = &close[open + 1..];
    let mut parts = Vec::new();
    let (mut start, mut nesting) = (0, 0isize);
    for (i, ch) in body.char_indices() {
        match ch {
            '(' => nesting += 1,
            ')' => {
                nesting -= 1;
                if nesting < 0 {
                    return None;
                }
            }
            ',' if nesting == 0 => {
                parts.push(&body[start..i]);
                start = i + 1
            }
            _ => {}
        }
    }
    if nesting != 0 {
        return None;
    }
    if !body[start..].trim().is_empty() {
        parts.push(&body[start..])
    }
    match name {
        "not" if parts.len() == 1 => cfg_value(parts[0], depth + 1).map(|v| !v),
        "all" => {
            let mut unknown = false;
            for part in parts {
                match cfg_value(part, depth + 1) {
                    Some(false) => return Some(false),
                    None => unknown = true,
                    _ => {}
                }
            }
            (!unknown).then_some(true)
        }
        "any" => {
            let mut unknown = false;
            for part in parts {
                match cfg_value(part, depth + 1) {
                    Some(true) => return Some(true),
                    None => unknown = true,
                    _ => {}
                }
            }
            (!unknown).then_some(false)
        }
        _ => None,
    }
}
fn and_cfg(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}
fn attribute_active(value: &str, function: bool) -> Option<bool> {
    let value = value
        .trim()
        .trim_start_matches("#![")
        .trim_start_matches("#[")
        .trim_end_matches(']')
        .trim();
    let name = value
        .split(|c: char| c == '(' || c == '=' || c.is_whitespace())
        .next()
        .unwrap_or("");
    if name == "cfg" {
        return value
            .strip_prefix("cfg")
            .and_then(|v| v.trim().strip_prefix('('))
            .and_then(|v| v.strip_suffix(')'))
            .and_then(|v| cfg_value(v, 0));
    }
    if matches!(
        name,
        "allow"
            | "warn"
            | "deny"
            | "forbid"
            | "expect"
            | "doc"
            | "rustfmt::skip"
            | "recursion_limit"
            | "type_length_limit"
            | "no_std"
    ) {
        return Some(true);
    }
    if function
        && matches!(
            name,
            "test"
                | "tokio::test"
                | "async_std::test"
                | "ignore"
                | "should_panic"
                | "cold"
                | "inline"
                | "track_caller"
                | "must_use"
                | "deprecated"
        )
    {
        return Some(true);
    }
    None
}
fn item_active(node: Node<'_>, source: &str, function: bool) -> Result<Option<bool>, String> {
    let mut state = Some(true);
    let mut previous = node.prev_named_sibling();
    while let Some(at) = previous {
        match at.kind() {
            "attribute_item" => {
                state = and_cfg(state, attribute_active(text(at, source)?, function))
            }
            "line_comment" | "block_comment" => {}
            _ => break,
        };
        previous = at.prev_named_sibling()
    }
    Ok(state)
}
pub fn module_test_graph(
    root: &std::path::Path,
    read: &mut dyn FnMut(&std::path::Path) -> Result<Option<String>, String>,
) -> Result<RustModuleTests, String> {
    struct Walk<'a> {
        read: &'a mut dyn FnMut(&std::path::Path) -> Result<Option<String>, String>,
        cache: std::collections::BTreeMap<std::path::PathBuf, String>,
        active: Vec<std::path::PathBuf>,
        bytes: usize,
        nodes: usize,
        result: RustModuleTests,
    }
    impl Walk<'_> {
        fn source(&mut self, path: &std::path::Path) -> Result<Option<String>, String> {
            if let Some(text) = self.cache.get(path) {
                return Ok(Some(text.clone()));
            }
            let value = (self.read)(path)?;
            if let Some(text) = &value {
                self.bytes = self
                    .bytes
                    .checked_add(text.len())
                    .ok_or("Rust module proof size overflow")?;
                if self.bytes > 8 * 1024 * 1024 || self.cache.len() >= 256 {
                    return Err("Rust module proof file/byte budget exceeded".into());
                }
                self.cache.insert(path.to_owned(), text.clone());
            }
            Ok(value)
        }
        fn file(
            &mut self,
            file: &std::path::Path,
            source: String,
            prefix: &[String],
            directory: &std::path::Path,
        ) -> Result<(), String> {
            if self.active.len() >= 32 || self.active.iter().any(|p| p == file) {
                return Err("Rust module proof has a cycle or exceeds its depth budget".into());
            }
            self.active.push(file.to_owned());
            let tree = parse(&source)?;
            let scan = super::candidates::rust("rust", &source)?;
            if !scan.syntax_complete {
                return Err("Rust module proof requires complete source syntax".into());
            }
            let candidates = scan
                .candidates
                .into_iter()
                .map(|c| (c.byte, c))
                .collect::<std::collections::BTreeMap<_, _>>();
            self.items(
                tree.root_node(),
                &source,
                file,
                prefix,
                directory,
                &candidates,
            )?;
            self.active.pop();
            Ok(())
        }
        fn items(
            &mut self,
            body: Node<'_>,
            source: &str,
            file: &std::path::Path,
            prefix: &[String],
            directory: &std::path::Path,
            candidates: &std::collections::BTreeMap<usize, super::candidates::TestCandidate>,
        ) -> Result<(), String> {
            let mut cursor = body.walk();
            let items = body.named_children(&mut cursor).collect::<Vec<_>>();
            let mut inner = Some(true);
            for item in &items {
                if text(*item, source)?.trim().starts_with("#![") {
                    inner = and_cfg(inner, attribute_active(text(*item, source)?, false))
                }
            }
            if inner != Some(true) {
                return Ok(());
            }
            for item in items {
                self.nodes += 1;
                if self.nodes > 200000 {
                    return Err("Rust module proof node budget exceeded".into());
                }
                match item.kind() {
                    "function_item" => {
                        let Some(name) = item.child_by_field_name("name") else {
                            continue;
                        };
                        let Some(candidate) = candidates.get(&name.start_byte()) else {
                            continue;
                        };
                        if item_active(item, source, true)? != Some(true) {
                            continue;
                        }
                        let mut qualified = prefix.to_vec();
                        qualified.push(candidate.name.trim_start_matches("r#").to_owned());
                        let value = RustModuleTest {
                            file: file.to_owned(),
                            line: candidate.line,
                            character: candidate.character,
                            name: candidate.name.clone(),
                        };
                        if self
                            .result
                            .tests
                            .insert(qualified.join("::"), value)
                            .is_some()
                        {
                            return Err(
                                "Rust module proof has duplicate active qualified tests".into()
                            );
                        }
                    }
                    "mod_item" => {
                        if item_active(item, source, false)? != Some(true) {
                            continue;
                        }
                        let name = item
                            .child_by_field_name("name")
                            .ok_or("Rust module name is unavailable")?;
                        let name = text(name, source)?.trim_start_matches("r#");
                        let mut child = prefix.to_vec();
                        child.push(name.to_owned());
                        if let Some(body) = item.child_by_field_name("body") {
                            self.items(
                                body,
                                source,
                                file,
                                &child,
                                &directory.join(name),
                                candidates,
                            )?
                        } else {
                            let a = directory.join(format!("{name}.rs"));
                            let b = directory.join(name).join("mod.rs");
                            let (first, second) = (self.source(&a)?, self.source(&b)?);
                            let (path, value) = match (first, second) {
                                (Some(a), None) => (0u8, a),
                                (None, Some(b)) => (1u8, b),
                                (Some(_), Some(_)) => {
                                    return Err("Rust module source is ambiguous".into());
                                }
                                (None, None) => {
                                    return Err(
                                        "An active Rust module source is unavailable".into()
                                    );
                                }
                            };
                            let path = if path == 0 { a } else { b };
                            let base = if path.file_name().is_some_and(|n| n == "mod.rs") {
                                path.parent().unwrap().to_owned()
                            } else {
                                path.parent().unwrap().join(
                                    path.file_stem().ok_or("Rust module filename is missing")?,
                                )
                            };
                            self.file(&path, value, &child, &base)?
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        }
    }
    let mut walk = Walk {
        read,
        cache: Default::default(),
        active: vec![],
        bytes: 0,
        nodes: 0,
        result: Default::default(),
    };
    let source = walk
        .source(root)?
        .ok_or("Rust compiler root source is unavailable")?;
    walk.file(
        root,
        source,
        &[],
        root.parent().ok_or("Rust compiler root has no directory")?,
    )?;
    Ok(walk.result)
}
#[cfg(test)]
mod graph_tests {
    use super::*;
    use std::path::{Path, PathBuf};
    fn graph(files: &[(&str, &str)]) -> Result<RustModuleTests, String> {
        let files = files
            .iter()
            .map(|(p, s)| (PathBuf::from(p), s.to_string()))
            .collect::<std::collections::BTreeMap<_, _>>();
        module_test_graph(Path::new("/pkg/src/lib.rs"), &mut |p| {
            Ok(files.get(p).cloned())
        })
    }
    #[test]
    fn ordinary_external_and_inline_modules_have_exact_qualified_owners() {
        let value = graph(&[
            (
                "/pkg/src/lib.rs",
                "#[cfg(test)]mod cases{#[test]fn chosen(){}} mod external;",
            ),
            (
                "/pkg/src/external.rs",
                "mod nested; #[test]fn external_case(){}",
            ),
            ("/pkg/src/external/nested.rs", "#[test]fn nested_case(){}"),
        ])
        .unwrap();
        assert_eq!(
            value.tests.keys().cloned().collect::<Vec<_>>(),
            vec![
                "cases::chosen",
                "external::external_case",
                "external::nested::nested_case"
            ]
        );
        assert_eq!(
            value.tests["external::nested::nested_case"].file,
            PathBuf::from("/pkg/src/external/nested.rs")
        );
    }
    #[test]
    fn inactive_or_unknown_declarations_never_borrow_a_siblings_name() {
        let value=graph(&[("/pkg/src/lib.rs","#[cfg(any())]mod inactive{#[test]fn chosen(){}} #[cfg(feature=\"later\")]mod unknown{#[test]fn chosen(){}} mod active{#[test]fn chosen(){}}")]).unwrap();
        assert_eq!(
            value.tests.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["active::chosen"]
        );
    }
    #[test]
    fn default_mod_directory_and_unknown_path_attribute_are_not_guessed() {
        let value = graph(&[
            (
                "/pkg/src/lib.rs",
                "mod external; #[path=\"different.rs\"]mod unproven;",
            ),
            ("/pkg/src/external/mod.rs", "mod nested;"),
            ("/pkg/src/external/nested.rs", "#[test]fn chosen(){}"),
        ])
        .unwrap();
        assert_eq!(
            value.tests.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["external::nested::chosen"]
        );
        assert!(
            graph(&[
                ("/pkg/src/lib.rs", "mod external;"),
                ("/pkg/src/external.rs", "#[test]fn chosen(){}"),
                ("/pkg/src/external/mod.rs", "#[test]fn chosen(){}")
            ])
            .is_err()
        );
    }
}
</ui:copy>

## Read emitted source evidence without starting a process

Only the chosen adapter’s pinned sibling LLDB reads the emitted image. Initialization and symbol scripts are disabled; its Python environment and working directory are isolated from workspace imports. The fixed data-only query returns bounded, non-inlined own-function source entries from the admitted image. No debuggee or test body starts during this reading. Optimized or unavailable source evidence refuses rather than borrowing an inlined caller.

<ui:copy id="rust-test-source-metadata">
import json
import os

query = json.loads(os.environ["HICKORY_RUST_META_QUERY"])
if set(query) != {"base", "first", "last", "image"} or not isinstance(query["base"], str):
    raise ValueError("Malformed source metadata query")
if type(query["first"]) is not int or type(query["last"]) is not int or not 1 <= query["first"] <= query["last"] <= 2**32 - 2:
    raise ValueError("Invalid source metadata range")
target = lldb.debugger.GetSelectedTarget()
spec = target.GetExecutable()
image = os.path.realpath(os.path.join(spec.GetDirectory() or "", spec.GetFilename() or ""))
if image != query["image"] or target.GetProcess().IsValid():
    raise ValueError("Unexpected image or process during read-only metadata inspection")
main = target.FindModule(spec)
if not main.IsValid():
    raise ValueError("The admitted image has no readable module")
contexts = target.FindFunctions(query["base"], lldb.eFunctionNameTypeBase)
if contexts.GetSize() > 1024:
    raise ValueError("Source metadata symbol budget exceeded")
rows = []
work = 0
for context in contexts:
    if context.GetModule() != main:
        continue
    function = context.GetFunction()
    if not function.IsValid():
        symbol = context.GetSymbol()
        if symbol.IsValid() and symbol.GetName():
            rows.append({"name": symbol.GetName(), "optimized": True, "entries": []})
        continue
    start = function.GetStartAddress()
    end = function.GetEndAddress().GetFileAddress()
    unit = context.GetCompileUnit()
    entry = start.GetLineEntry()
    entries = []
    index = unit.FindLineEntryIndex(entry, True) if unit.IsValid() and entry.IsValid() else 2**32 - 1
    if index != 2**32 - 1:
        while index < unit.GetNumLineEntries():
            work += 1
            if work > 100000:
                raise ValueError("Source metadata line budget exceeded")
            value = unit.GetLineEntryAtIndex(index)
            address = value.GetStartAddress().GetFileAddress()
            if address >= end:
                break
            position = value.GetStartAddress()
            block = position.GetBlock()
            own_source = position.GetFunction() == function and block.IsValid() and not block.IsInlined() and not block.GetContainingInlinedBlock().IsValid()
            if own_source and 0 < value.GetLine() < 2**32 - 1 and (not entries or query["first"] <= value.GetLine() <= query["last"]):
                spec = value.GetFileSpec()
                row = {"file": os.path.normpath(os.path.join(spec.GetDirectory() or "", spec.GetFilename() or "")), "line": value.GetLine(), "column": value.GetColumn(), "address": str(address)}
                if row not in entries:
                    entries.append(row)
            if len(entries) > 4096:
                raise ValueError("Source metadata entry budget exceeded")
            index += 1
    rows.append({"name": function.GetName(), "optimized": function.GetIsOptimized(), "entries": entries})
print("HICKORY_RUST_SOURCE_V1=" + json.dumps({"version": 1, "producer": "lldb-image-source-v1", "functions": rows}, ensure_ascii=True))
</ui:copy>

## Join exact image, namespace and physical source

The decoder rejects extra fields and unknown versions. A basename or unique leaf is insufficient: canonical source path, selected declaration span and qualified namespace must agree, and the exact listed name must already belong to the captured module graph. This prevents a cfg-disabled or foreign-source sibling from being chosen merely because it is the only compiled suffix match.

<ui:copy id="rust-test-source-reader">
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceEvidence {
    Selected,
    Other,
    Unknown,
}
fn rust_compiled_source_evidence(
    raw: &serde_json::Value,
    crate_name: &str,
    declaration: &hick_plugins::testing::rust_debug_source::RustTestDeclaration,
    selected: &Path,
    is_crate_root: bool,
) -> Result<std::collections::BTreeMap<String, SourceEvidence>> {
    anyhow::ensure!(
        raw.as_object().is_some_and(|v| v.len() == 3
            && v.keys()
                .all(|k| matches!(k.as_str(), "version" | "producer" | "functions"))),
        "Unexpected Rust compiled source proof fields"
    );
    anyhow::ensure!(
        raw.get("version").and_then(|v| v.as_u64()) == Some(1)
            && raw.get("producer").and_then(|v| v.as_str()) == Some("lldb-image-source-v1"),
        "Unknown Rust compiled source proof"
    );
    let functions = raw
        .get("functions")
        .and_then(|v| v.as_array())
        .context("Rust compiled source symbols are unavailable")?;
    anyhow::ensure!(
        functions.len() <= 1024,
        "Rust compiled source symbol budget exceeded"
    );
    let selected = selected.canonicalize()?;
    let mut readings = std::collections::BTreeMap::new();
    let mut work = 0usize;
    let mut local = declaration.namespace.clone();
    local.push(declaration.name.clone());
    let local = local.join("::");
    for row in functions {
        anyhow::ensure!(
            row.as_object().is_some_and(|v| v.len() == 3
                && v.contains_key("name")
                && v.contains_key("entries")
                && v.contains_key("optimized")),
            "Unexpected Rust compiled function fields"
        );
        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .context("Rust compiled function name is missing")?;
        let optimized = row
            .get("optimized")
            .and_then(|v| v.as_bool())
            .context("Rust compiled function optimization state is missing")?;
        anyhow::ensure!(
            name.len() <= 8192,
            "Rust compiled function name budget exceeded"
        );
        let mut parts = name.split("::").collect::<Vec<_>>();
        if let Some(last) = parts.last() {
            if last.len() == 17
                && last.starts_with('h')
                && last[1..].bytes().all(|b| b.is_ascii_hexdigit())
                && *last != declaration.name
            {
                parts.pop();
            }
        }
        if parts.len() < 2 || parts[0] != crate_name.replace('-', "_") {
            continue;
        }
        parts.remove(0);
        if parts.last().copied() != Some(declaration.name.as_str())
            || parts.iter().any(|part| {
                part.is_empty()
                    || !part.chars().all(|c| {
                        c == '_'
                            || c.is_alphanumeric()
                            || !c.is_ascii() && !c.is_control() && !c.is_whitespace()
                    })
            })
        {
            continue;
        }
        let qualified = parts.join("::");
        let namespace = if is_crate_root {
            qualified == local
        } else {
            qualified == local || qualified.ends_with(&("::".to_owned() + &local))
        };
        let entries = row
            .get("entries")
            .and_then(|v| v.as_array())
            .context("Rust compiled function has no source reading")?;
        anyhow::ensure!(
            entries.len() <= 4096,
            "Rust compiled source entry budget exceeded"
        );
        let (mut matched, mut known) = (false, false);
        for entry in entries {
            anyhow::ensure!(
                entry.as_object().is_some_and(|v| v.len() == 4
                    && v.keys()
                        .all(|k| matches!(k.as_str(), "file" | "line" | "column" | "address"))),
                "Unexpected Rust source entry fields"
            );
            work += 1;
            anyhow::ensure!(
                work <= 100000,
                "Rust compiled source reading budget exceeded"
            );
            let line = entry
                .get("line")
                .and_then(|v| v.as_u64())
                .context("Rust compiled source line is missing")?;
            let file = entry
                .get("file")
                .and_then(|v| v.as_str())
                .context("Rust compiled source path is missing")?;
            anyhow::ensure!(
                file.len() <= 32768,
                "Rust compiled source path budget exceeded"
            );
            let path = Path::new(file);
            if !path.is_absolute() {
                continue;
            }
            if let Ok(path) = path.canonicalize() {
                known = true;
                if path == selected
                    && line >= declaration.first_line as u64
                    && line <= declaration.last_line as u64
                {
                    matched = true
                }
            }
        }
        let evidence = if optimized {
            SourceEvidence::Unknown
        } else if matched && namespace {
            SourceEvidence::Selected
        } else if matched {
            SourceEvidence::Unknown
        } else if known {
            SourceEvidence::Other
        } else {
            SourceEvidence::Unknown
        };
        let slot = readings.entry(qualified).or_insert(evidence);
        if *slot != evidence {
            *slot = SourceEvidence::Unknown
        }
    }
    Ok(readings)
}
#[cfg(test)]
fn rust_compiled_source_identity(
    raw: &serde_json::Value,
    crate_name: &str,
    declaration: &hick_plugins::testing::rust_debug_source::RustTestDeclaration,
    selected: &Path,
    is_crate_root: bool,
) -> Result<String> {
    let mut selected =
        rust_compiled_source_evidence(raw, crate_name, declaration, selected, is_crate_root)?
            .into_iter()
            .filter_map(|(name, state)| (state == SourceEvidence::Selected).then_some(name))
            .collect::<Vec<_>>();
    anyhow::ensure!(
        selected.len() == 1,
        "Selected Rust declaration has no unique compiled source/qualified identity"
    );
    Ok(selected.remove(0))
}
</ui:copy>

## Hold source ownership across compiler and metadata work

Preparation holds selected source, package manifest and only the ordinary module files read by declared test targets. It checks file identity, metadata revision and bytes before and after asynchronous work and before adapter launch; equal-byte replacement is obsolete. A related dirty physical module refuses, while unrelated drafts remain independent. Saved constructed authors retain the existing compiler-author guards. Physical Cargo dependencies and build scripts are outside this bounded source proof.

<ui:copy id="rust-test-source-capture">
use super::*;
use std::io::{Read, Seek};
struct SourcePin {
    path: PathBuf,
    canonical: PathBuf,
    held: same_file::Handle,
    identity: RustExecutableIdentity,
    changed: (i64, i64),
    text: String,
}
fn change_time(meta: &std::fs::Metadata) -> (i64, i64) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        (meta.ctime(), meta.ctime_nsec())
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        (0, 0)
    }
}
impl SourcePin {
    fn capture(path: &Path) -> Result<Self> {
        let original = std::fs::symlink_metadata(path)?;
        anyhow::ensure!(
            original.is_file() && !original.file_type().is_symlink() && original.len() <= 1048576,
            "Rust source proof needs a bounded ordinary file"
        );
        let file = std::fs::File::open(path)?;
        let held = same_file::Handle::from_file(file)?;
        let canonical = path.canonicalize()?;
        anyhow::ensure!(
            same_file::Handle::from_path(path)? == held,
            "Rust source ownership changed while capturing proof"
        );
        let identity = rust_executable_identity(&canonical)?;
        anyhow::ensure!(
            identity.modified.is_some(),
            "Rust source revision metadata is unavailable"
        );
        let changed = change_time(&held.as_file().metadata()?);
        let mut file = held.as_file().try_clone()?;
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.take(1048577).read_to_end(&mut bytes)?;
        anyhow::ensure!(
            bytes.len() <= 1048576,
            "Rust source proof byte budget exceeded"
        );
        let text = String::from_utf8(bytes).context("Rust source is not UTF-8")?;
        let result = Self {
            path: path.to_owned(),
            canonical,
            held,
            identity,
            changed,
            text,
        };
        result.current()?;
        Ok(result)
    }
    fn current(&self) -> Result<()> {
        anyhow::ensure!(
            self.path.canonicalize()? == self.canonical
                && same_file::Handle::from_path(&self.path)? == self.held
                && rust_executable_identity(&self.canonical)? == self.identity
                && change_time(&self.held.as_file().metadata()?) == self.changed,
            "Rust source identity/revision changed during compiler proof"
        );
        let mut file = self.held.as_file().try_clone()?;
        file.seek(std::io::SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.take(1048577).read_to_end(&mut bytes)?;
        anyhow::ensure!(
            bytes == self.text.as_bytes(),
            "Rust source bytes changed during compiler proof"
        );
        anyhow::ensure!(
            same_file::Handle::from_path(&self.path)? == self.held,
            "Rust source address was replaced during compiler proof"
        );
        Ok(())
    }
}
pub(super) struct Selection {
    selected: SourcePin,
    manifest: SourcePin,
    declaration: hick_plugins::testing::rust_debug_source::RustTestDeclaration,
    line: u32,
    character: u32,
}
impl Selection {
    pub(super) fn capture(
        path: &Path,
        line: u32,
        character: u32,
        hash: &str,
        name: &str,
        runtime: &Path,
    ) -> Result<Self> {
        let manifest = SourcePin::capture(&runtime.join("Cargo.toml"))?;
        let selected = SourcePin::capture(path)?;
        anyhow::ensure!(
            crate::serve::source_tests::digest(selected.text.as_bytes()) == hash,
            "Selected Rust source no longer matches the admitted compiler output"
        );
        let declaration = hick_plugins::testing::rust_debug_source::selected_declaration(
            &selected.text,
            line,
            character,
            name,
        )
        .map_err(anyhow::Error::msg)?;
        Ok(Self {
            selected,
            manifest,
            declaration,
            line,
            character,
        })
    }
    pub(super) fn current(&self) -> Result<()> {
        self.selected.current()?;
        self.manifest.current()
    }
}
struct SourceReadBudget {
    started: std::time::Instant,
    reads: usize,
    bytes: usize,
}
impl SourceReadBudget {
    fn new() -> Self {
        Self {
            started: std::time::Instant::now(),
            reads: 0,
            bytes: 0,
        }
    }
    fn charge(&mut self, bytes: usize) -> Result<(), String> {
        self.reads = self.reads.saturating_add(1);
        self.bytes = self.bytes.saturating_add(bytes);
        if self.reads > 1024
            || self.bytes > 32 * 1024 * 1024
            || self.started.elapsed() > std::time::Duration::from_secs(10)
        {
            return Err("Aggregate Cargo module-reading budget exceeded".into());
        }
        Ok(())
    }
}
pub(super) struct Preparation {
    selection: Selection,
    files: std::collections::BTreeMap<PathBuf, SourcePin>,
    graphs: std::collections::BTreeMap<
        PathBuf,
        hick_plugins::testing::rust_debug_source::RustModuleTests,
    >,
}
impl Preparation {
    pub(super) fn capture(
        selection: Selection,
        metadata: &serde_json::Value,
        runtime: &Path,
        physical: Option<(&Path, &std::collections::BTreeMap<String, String>)>,
    ) -> Result<Self> {
        selection.current()?;
        let runtime = runtime.canonicalize()?;
        let manifest = runtime.join("Cargo.toml").canonicalize()?;
        let packages = metadata
            .get("packages")
            .and_then(|v| v.as_array())
            .context("Cargo metadata has no package reading")?;
        anyhow::ensure!(
            packages.len() <= 4096,
            "Cargo metadata package budget exceeded"
        );
        let packages = packages
            .iter()
            .filter(|p| {
                p.get("manifest_path")
                    .and_then(|v| v.as_str())
                    .and_then(|p| Path::new(p).canonicalize().ok())
                    .as_ref()
                    == Some(&manifest)
            })
            .collect::<Vec<_>>();
        anyhow::ensure!(
            packages.len() == 1,
            "Cargo metadata has no unique admitted package manifest"
        );
        let targets = packages[0]
            .get("targets")
            .and_then(|v| v.as_array())
            .context("Cargo metadata has no targets")?;
        anyhow::ensure!(
            targets.len() <= 128,
            "Cargo test target proof budget exceeded"
        );
        let mut files = std::collections::BTreeMap::<PathBuf, SourcePin>::new();
        let mut total = selection.selected.text.len();
        let mut graphs = std::collections::BTreeMap::new();
        let mut reading_budget = SourceReadBudget::new();
        for target in targets {
            if target.get("test").and_then(|v| v.as_bool()) != Some(true) {
                continue;
            }
            let path = target
                .get("src_path")
                .and_then(|v| v.as_str())
                .context("Cargo test root is missing")?;
            let root = Path::new(path).canonicalize()?;
            anyhow::ensure!(
                root.starts_with(&runtime),
                "Cargo test root escaped its admitted package"
            );
            if graphs.contains_key(&root) {
                continue;
            }
            let mut read = |path: &Path| -> Result<Option<String>, String> {
                reading_budget.charge(0)?;
                if path == selection.selected.canonical {
                    reading_budget.charge(selection.selected.text.len())?;
                    return Ok(Some(selection.selected.text.clone()));
                }
                if let Some(pin) = files.get(path) {
                    reading_budget.charge(pin.text.len())?;
                    return Ok(Some(pin.text.clone()));
                }
                match std::fs::symlink_metadata(path) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                    Err(e) => return Err(e.to_string()),
                    _ => {}
                }
                let pin = SourcePin::capture(path).map_err(|e| e.to_string())?;
                if pin.canonical != path || !pin.canonical.starts_with(&runtime) {
                    return Err("Rust module mapping is not a proven ordinary package path".into());
                }
                if let Some((workspace, open)) = physical {
                    let relative = pin
                        .canonical
                        .strip_prefix(workspace)
                        .map_err(|_| "Rust module escaped the saved workspace")?
                        .to_str()
                        .ok_or("Rust module path is not UTF-8")?;
                    if open
                        .get(relative)
                        .is_some_and(|source| source.as_bytes() != pin.text.as_bytes())
                    {
                        return Err(format!(
                            "Save the related Rust module {relative} before Debug test"
                        ));
                    }
                }
                total += pin.text.len();
                if total > 8 * 1024 * 1024 || files.len() >= 256 {
                    return Err("Cargo module source proof budget exceeded".into());
                }
                reading_budget.charge(pin.text.len())?;
                let text = pin.text.clone();
                files.insert(pin.canonical.clone(), pin);
                Ok(Some(text))
            };
            let text = read(&root)
                .map_err(anyhow::Error::msg)?
                .context("Cargo test root is unreadable")?;
            hick_plugins::testing::rust_debug_source::builtin_crate_harness(&text)
                .map_err(anyhow::Error::msg)?;
            let graph =
                hick_plugins::testing::rust_debug_source::module_test_graph(&root, &mut read)
                    .map_err(anyhow::Error::msg)?;
            reading_budget.charge(0).map_err(anyhow::Error::msg)?;
            graphs.insert(root, graph);
        }
        let result = Self {
            selection,
            files,
            graphs,
        };
        result.current()?;
        anyhow::ensure!(
            result
                .graphs
                .values()
                .any(|g| g.tests.values().any(|case| result.selected(case))),
            "Selected Rust declaration has no proven active module/target identity; unknown cfg/path/include mappings were not replaced by a sibling"
        );
        Ok(result)
    }
    fn selected(&self, case: &hick_plugins::testing::rust_debug_source::RustModuleTest) -> bool {
        case.file == self.selection.selected.canonical
            && case.line == self.selection.line
            && case.character == self.selection.character
            && case.name.trim_start_matches("r#") == self.selection.declaration.name
    }
    pub(super) fn current(&self) -> Result<()> {
        self.selection.current()?;
        for pin in self.files.values() {
            pin.current()?
        }
        Ok(())
    }
    pub(super) fn ignored(&self) -> bool {
        self.selection.declaration.ignored
    }
}
pub(super) struct MetadataTool {
    path: PathBuf,
    canonical: PathBuf,
    held: same_file::Handle,
    identity: RustExecutableIdentity,
}
impl MetadataTool {
    pub(super) fn from_adapter(name: &str, command: &[String]) -> Result<Self> {
        anyhow::ensure!(
            name == "lldb-dap",
            "The selected Rust adapter does not expose verified read-only source metadata; no suffix-name fallback was used"
        );
        let adapter = Path::new(command.first().context("Rust adapter command is missing")?);
        anyhow::ensure!(
            adapter.is_absolute(),
            "Rust source metadata requires a pinned adapter executable"
        );
        let path = adapter
            .parent()
            .context("Rust adapter has no tool directory")?
            .join(if cfg!(windows) { "lldb.exe" } else { "lldb" });
        let canonical = path
            .canonicalize()
            .context("The selected LLVM adapter has no compatible LLDB metadata reader")?;
        let held = same_file::Handle::from_path(&canonical)?;
        let identity = rust_executable_identity(&canonical)?;
        let result = Self {
            path,
            canonical,
            held,
            identity,
        };
        result.current()?;
        Ok(result)
    }
    fn current(&self) -> Result<()> {
        anyhow::ensure!(
            self.path.canonicalize()? == self.canonical
                && same_file::Handle::from_path(&self.path)? == self.held
                && rust_executable_identity(&self.canonical)? == self.identity,
            "LLDB metadata reader identity changed"
        );
        Ok(())
    }
}

impl Preparation {
    pub(super) fn admit_artifact(&self, artifact: &RustTestArtifact) -> Result<()> {
        self.current()?;
        anyhow::ensure!(
            self.graphs.contains_key(&artifact.source),
            "Compiled Rust target has no captured built-in-harness/module graph; refused before listing"
        );
        Ok(())
    }
    pub(super) fn listed_identity(
        &self,
        artifact: &RustTestArtifact,
        names: &[String],
    ) -> Result<Option<String>> {
        let graph = self
            .graphs
            .get(&artifact.source)
            .context("Compiled Rust target has no captured module graph")?;
        let mut selected = None;
        for name in names {
            let normalized = name
                .split("::")
                .map(|part| part.strip_prefix("r#").unwrap_or(part))
                .collect::<Vec<_>>()
                .join("::");
            let case=graph.tests.get(&normalized).context("A matching compiled test has an unknown cfg/module/source association; no sibling substitution was used")?;
            if self.selected(case) {
                anyhow::ensure!(
                    selected.is_none(),
                    "Selected Rust declaration has ambiguous compiled test identities"
                );
                selected = Some(name.clone())
            }
        }
        Ok(selected)
    }
    pub(super) async fn verify_compiled(
        &self,
        tool: &MetadataTool,
        artifact: &RustTestArtifact,
        name: &str,
        crate_name: &str,
    ) -> Result<()> {
        self.current()?;
        tool.current()?;
        rust_artifact_current(artifact)?;
        let held = same_file::Handle::from_path(&artifact.executable)?;
        let temporary =
            tempfile::tempdir().context("Cannot allocate an isolated metadata reader directory")?;
        let query = serde_json::json!({"base":self.selection.declaration.name,"first":self.selection.declaration.first_line,"last":self.selection.declaration.last_line,"image":artifact.executable.canonicalize()?});
        let script = include_str!("rust_test_source_metadata.py");
        let mut command = tokio::process::Command::new(&tool.path);
        command
            .current_dir(temporary.path())
            .args([
                "--no-lldbinit",
                "--batch",
                "-O",
                "settings set target.load-script-from-symbol-file false",
                "-f",
            ])
            .arg(&artifact.executable)
            .arg("-o")
            .arg(format!("script exec({})", serde_json::to_string(script)?))
            .env("HICKORY_RUST_META_QUERY", serde_json::to_string(&query)?)
            .env_remove("PYTHONPATH")
            .env_remove("PYTHONHOME")
            .env("PYTHONNOUSERSITE", "1")
            .env("PYTHONSAFEPATH", "1");
        let (output, _) =
            rust_test_command_output_with_limit(command, std::time::Duration::from_secs(20))
                .await?;
        self.current()?;
        tool.current()?;
        rust_artifact_current(artifact)?;
        anyhow::ensure!(
            same_file::Handle::from_path(&artifact.executable)? == held,
            "Compiled Rust image identity changed during metadata inspection"
        );
        let text = std::str::from_utf8(&output).context("LLDB source metadata was not UTF-8")?;
        let values = text
            .lines()
            .filter_map(|line| line.strip_prefix("HICKORY_RUST_SOURCE_V1="))
            .collect::<Vec<_>>();
        anyhow::ensure!(
            values.len() == 1,
            "LLDB did not return one bounded compiled source reading"
        );
        let raw: serde_json::Value = serde_json::from_str(values[0])?;
        let evidence = rust_compiled_source_evidence(
            &raw,
            crate_name,
            &self.selection.declaration,
            &self.selection.selected.canonical,
            artifact.source == self.selection.selected.canonical,
        )?;
        let normalized = name
            .split("::")
            .map(|part| part.strip_prefix("r#").unwrap_or(part))
            .collect::<Vec<_>>()
            .join("::");
        anyhow::ensure!(
            evidence.get(&normalized) == Some(&SourceEvidence::Selected),
            "The emitted Rust test does not prove the selected source declaration; Debug refused rather than substituting a sibling"
        );
        Ok(())
    }
}
</ui:copy>

## Exercise source and decoder refusals

These focused witnesses cover disabled/same-leaf declarations, exact namespace and source spans, ordinary external module ownership, foreign same-basename files, unexpected metadata and equal-byte file replacement. Real compiler/LLDB and mounted host checks live in the selected-test Debug acceptance reading; model success alone does not establish usable host behavior.

<ui:copy id="rust-test-source-reader-tests">
#[cfg(test)]
mod source_reader_tests {
    use super::*;
    use serde_json::json;
    fn reading(file: &Path, line: u32, name: &str) -> serde_json::Value {
        json!({"version":1,"producer":"lldb-image-source-v1","functions":[{"name":name,"optimized":false,"entries":[{"file":file,"line":line,"column":0,"address":"100"}]}]})
    }
    fn declaration() -> hick_plugins::testing::rust_debug_source::RustTestDeclaration {
        hick_plugins::testing::rust_debug_source::RustTestDeclaration {
            name: "chosen".into(),
            namespace: vec!["cases".into()],
            first_line: 4,
            last_line: 7,
            ignored: false,
        }
    }
    #[test]
    fn file_span_and_exact_namespace_refuse_cfg_disabled_siblings() {
        let root = tempfile::tempdir().unwrap();
        let selected = root.path().join("lib.rs");
        std::fs::write(&selected, "").unwrap();
        let value = reading(&selected, 5, "owned::cases::chosen::h0123456789abcdef");
        assert_eq!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &selected, true)
                .unwrap(),
            "cases::chosen"
        );
        assert!(
            rust_compiled_source_identity(
                &reading(&selected, 13, "owned::cases::chosen::h0123456789abcdef"),
                "owned",
                &declaration(),
                &selected,
                true
            )
            .is_err()
        );
        assert!(
            rust_compiled_source_identity(
                &reading(&selected, 5, "owned::other::chosen::h0123456789abcdef"),
                "owned",
                &declaration(),
                &selected,
                true
            )
            .is_err()
        );
    }
    #[test]
    fn a_same_basename_is_not_the_selected_physical_source() {
        let root = tempfile::tempdir().unwrap();
        let a = root.path().join("one/lib.rs");
        let b = root.path().join("two/lib.rs");
        std::fs::create_dir_all(a.parent().unwrap()).unwrap();
        std::fs::create_dir_all(b.parent().unwrap()).unwrap();
        std::fs::write(&a, b"same").unwrap();
        std::fs::write(&b, b"same").unwrap();
        assert!(
            rust_compiled_source_identity(
                &reading(&b, 5, "owned::cases::chosen::h0123456789abcdef"),
                "owned",
                &declaration(),
                &a,
                true
            )
            .is_err()
        );
    }
    #[test]
    fn external_module_mapping_requires_one_emitted_full_identity() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("module.rs");
        std::fs::write(&source, "").unwrap();
        let mut value = reading(
            &source,
            5,
            "owned::ordinary::cases::chosen::h0123456789abcdef",
        );
        assert_eq!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &source, false).unwrap(),
            "ordinary::cases::chosen"
        );
        let row =
            reading(&source, 5, "owned::other::cases::chosen::h0123456789abcdef")["functions"][0]
                .clone();
        value["functions"].as_array_mut().unwrap().push(row);
        assert!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &source, false).is_err()
        );
    }
    #[test]
    fn unknown_metadata_effects_and_relative_paths_never_become_proof() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("lib.rs");
        std::fs::write(&source, "").unwrap();
        let mut value = reading(&source, 5, "owned::cases::chosen::h0123456789abcdef");
        value["command"] = json!("run sibling");
        assert!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &source, true).is_err()
        );
        let value = reading(
            Path::new("lib.rs"),
            5,
            "owned::cases::chosen::h0123456789abcdef",
        );
        assert!(
            rust_compiled_source_identity(&value, "owned", &declaration(), &source, true).is_err()
        );
    }
}

#[cfg(test)]
mod captured_source_tests {
    use super::*;
    #[test]
    fn repeated_target_readings_share_byte_count_and_time_limits() {
        let mut bytes = SourceReadBudget::new();
        assert!(bytes.charge(32 * 1024 * 1024 + 1).is_err());
        let mut count = SourceReadBudget::new();
        for _ in 0..1024 {
            count.charge(0).unwrap()
        }
        assert!(count.charge(0).is_err());
        let mut time = SourceReadBudget::new();
        time.started = std::time::Instant::now() - std::time::Duration::from_secs(11);
        assert!(time.charge(0).is_err());
    }

    #[test]
    fn selected_graph_never_admits_a_cfg_disabled_sibling() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        let path = root.path().join("src/lib.rs");
        let text = "#[cfg(any())]mod inactive {\n #[test] fn chosen(){}\n}\nmod active {\n #[test] fn chosen(){}\n}\n";
        std::fs::write(&path, text).unwrap();
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname='proof'\nversion='0.1.0'\n",
        )
        .unwrap();
        let candidates = hick_plugins::testing::candidates::rust("rust", text)
            .unwrap()
            .candidates;
        let metadata = serde_json::json!({"packages":[{"manifest_path":root.path().join("Cargo.toml"),"targets":[{"test":true,"src_path":path}]}]});
        let hash = crate::serve::source_tests::digest(text.as_bytes());
        let inactive = &candidates[0];
        let selection = Selection::capture(
            &path,
            inactive.line,
            inactive.character,
            &hash,
            &inactive.name,
            root.path(),
        )
        .unwrap();
        assert!(Preparation::capture(selection, &metadata, root.path(), None).is_err());
        let active = &candidates[1];
        let selection = Selection::capture(
            &path,
            active.line,
            active.character,
            &hash,
            &active.name,
            root.path(),
        )
        .unwrap();
        let proof = Preparation::capture(selection, &metadata, root.path(), None).unwrap();
        let artifact = RustTestArtifact {
            executable: path.clone(),
            source: path.canonicalize().unwrap(),
            identity: rust_executable_identity(&path).unwrap(),
        };
        proof.admit_artifact(&artifact).unwrap();
        let foreign_artifact = RustTestArtifact {
            source: root.path().join("unknown-root.rs"),
            executable: artifact.executable.clone(),
            identity: artifact.identity.clone(),
        };
        assert!(
            proof
                .admit_artifact(&foreign_artifact)
                .unwrap_err()
                .to_string()
                .contains("refused before listing")
        );
        assert_eq!(
            proof
                .listed_identity(&artifact, &["active::chosen".into()])
                .unwrap(),
            Some("active::chosen".into())
        );
        assert!(
            proof
                .listed_identity(&artifact, &["inactive::chosen".into()])
                .is_err()
        );
        let replacement = path.with_extension("replacement");
        std::fs::write(&replacement, text).unwrap();
        std::fs::rename(replacement, &path).unwrap();
        assert!(proof.current().is_err());
    }
}
</ui:copy>

<ui:file path="crates/hick-plugins/src/testing/rust_debug_source.rs"><ui:paste select="#rust-test-source-declaration"/></ui:file>
<ui:file path="crates/hickory-cli/src/rust_test_source_metadata.py"><ui:paste select="#rust-test-source-metadata"/></ui:file>
<ui:file path="crates/hickory-cli/src/rust_test_source_proof.rs"><ui:paste select="#rust-test-source-reader"/><ui:paste select="#rust-test-source-capture"/><ui:paste select="#rust-test-source-reader-tests"/></ui:file>
</ui:doc>
