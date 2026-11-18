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
