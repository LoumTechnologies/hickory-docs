//! Source candidates are not a runner's discovered test identities.
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser};
pub type Detector = fn(&str, &str) -> Result<CandidateScan, String>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Syntax,
    Heuristic,
    Dynamic,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCandidate {
    pub line: u32,
    pub character: u32,
    pub byte: usize,
    pub name: String,
    /// A declaration/family name filter, never a confirmed runner case ID.
    pub name_filter: Option<String>,
    pub confidence: Confidence,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateScan {
    pub candidates: Vec<TestCandidate>,
    pub syntax_complete: bool,
    pub notices: Vec<String>,
}
struct Found {
    byte: usize,
    name: String,
    filter: Option<String>,
    confidence: Confidence,
}
type Extractor = for<'a> fn(Node<'a>, &str) -> Option<Found>;
fn finish(
    source: &str,
    mut found: Vec<Found>,
    syntax_complete: bool,
    notices: Vec<String>,
) -> Result<CandidateScan, String> {
    if found.len() > 10000 {
        return Err(
            "More than 10,000 source test candidates. Narrow the file or use runner discovery."
                .into(),
        );
    }
    let mut lines = vec![0];
    lines.extend(source.match_indices('\n').map(|(byte, _)| byte + 1));
    found.sort_by(|a, b| a.byte.cmp(&b.byte).then(a.name.cmp(&b.name)));
    found.dedup_by(|a, b| a.byte == b.byte && a.name == b.name);
    let candidates = found
        .into_iter()
        .filter_map(|item| {
            if item.name.is_empty() || !source.is_char_boundary(item.byte) {
                return None;
            }
            let row = lines
                .partition_point(|start| *start <= item.byte)
                .checked_sub(1)?;
            Some(TestCandidate {
                line: row as u32 + 1,
                character: source[lines[row]..item.byte].encode_utf16().count() as u32,
                byte: item.byte,
                name: item.name,
                name_filter: item.filter,
                confidence: item.confidence,
            })
        })
        .collect();
    Ok(CandidateScan {
        candidates,
        syntax_complete,
        notices,
    })
}
fn syntax(language: &str, source: &str, extract: Extractor) -> Result<CandidateScan, String> {
    if source.len() > 2 * 1024 * 1024 {
        return Err("Source test analysis supports up to 2 MiB. Use project runner discovery for this file.".into());
    }
    let Some(crate::Interface::Structure { grammar, .. }) =
        crate::registry().interface(language, crate::InterfaceKind::Structure)
    else {
        return Err(format!(
            "No source grammar for {language}. Use runner discovery."
        ));
    };
    let grammar = grammar(language)
        .ok_or_else(|| format!("No source grammar for {language}. Use runner discovery."))?;
    let mut parser = Parser::new();
    parser
        .set_language(&grammar)
        .map_err(|e| format!("Source grammar unavailable: {e}. Reinstall language support."))?;
    #[allow(deprecated)]
    parser.set_timeout_micros(1_000_000);
    let tree = parser
        .parse(source, None)
        .ok_or("Source test analysis timed out. Use runner discovery or narrow the file.")?;
    let complete = !tree.root_node().has_error();
    let mut stack = vec![tree.root_node()];
    let mut found = vec![];
    while let Some(node) = stack.pop() {
        if let Some(item) = extract(node, source) {
            found.push(item);
            if found.len() > 10000 {
                break;
            }
        }
        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }
    finish(
        source,
        found,
        complete,
        if complete {
            vec![]
        } else {
            vec!["Source has syntax errors; candidates may be incomplete. Correct the source or use runner discovery.".into()]
        },
    )
}
fn text<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    source.get(node.byte_range())
}
fn named(node: Node<'_>, source: &str) -> Option<Found> {
    let name = node.child_by_field_name("name")?;
    let value = text(name, source)?.to_string();
    Some(Found {
        byte: name.start_byte(),
        name: value.clone(),
        filter: Some(value),
        confidence: Confidence::Syntax,
    })
}
pub fn python(language: &str, source: &str) -> Result<CandidateScan, String> {
    syntax(language, source, |node, source| {
        if node.kind() != "function_definition" {
            return None;
        }
        let item = named(node, source)?;
        if !item.name.starts_with("test_") {
            return None;
        }
        let mut parent = node.parent();
        while let Some(at) = parent {
            if at.kind() == "function_definition" {
                return None;
            }
            parent = at.parent()
        }
        Some(item)
    })
}
pub fn rust(language: &str, source: &str) -> Result<CandidateScan, String> {
    syntax(language, source, |node, source| {
        if node.kind() != "function_item" {
            return None;
        }
        let mut sibling = node.prev_named_sibling();
        let mut test = false;
        while let Some(at) = sibling {
            match at.kind() {
                "line_comment" | "block_comment" => {}
                "attribute_item" => {
                    let value = text(at, source)?
                        .trim()
                        .trim_start_matches("#[")
                        .trim_end_matches(']')
                        .trim();
                    let path = value.split('(').next()?.trim();
                    if matches!(path, "test" | "tokio::test" | "async_std::test" | "rstest") {
                        test = true
                    }
                }
                _ => break,
            };
            sibling = at.prev_named_sibling();
        }
        test.then(|| named(node, source)).flatten()
    })
}
fn test_callee(mut node: Node<'_>, source: &str) -> Option<bool> {
    let mut parameterized = false;
    for _ in 0..128 {
        match node.kind() {
            "identifier" => {
                return matches!(text(node, source), Some("test" | "it")).then_some(parameterized);
            }
            "member_expression" => {
                let property = node
                    .child_by_field_name("property")
                    .and_then(|n| text(n, source))?;
                if !matches!(
                    property,
                    "only" | "skip" | "todo" | "each" | "runIf" | "skipIf"
                ) {
                    return None;
                }
                parameterized |= property == "each";
                node = node.child_by_field_name("object")?;
            }
            "call_expression" => {
                node = node
                    .child_by_field_name("function")
                    .filter(|n| n.kind() == "member_expression")?;
            }
            _ => return None,
        }
    }
    None
}
pub fn javascript(language: &str, source: &str) -> Result<CandidateScan, String> {
    syntax(language, source, |node, source| {
        if node.kind() != "call_expression" {
            return None;
        }
        let callee = node.child_by_field_name("function")?;
        let parameterized = test_callee(callee, source)?;
        let args = node.child_by_field_name("arguments")?;
        let first = args.named_child(0)?;
        // Modifier factory setup does not itself declare a test case.
        if callee.kind() == "member_expression"
            && matches!(
                callee
                    .child_by_field_name("property")
                    .and_then(|n| text(n, source)),
                Some("each" | "runIf" | "skipIf")
            )
        {
            return None;
        }
        let raw = text(first, source)?;
        let quote = raw.as_bytes().first().copied();
        let literal = matches!(first.kind(), "string" | "template_string")
            && raw.len() >= 2
            && quote == raw.as_bytes().last().copied();
        let display = if literal {
            raw[1..raw.len() - 1].to_string()
        } else {
            "test(…)".into()
        };
        let dynamic = !literal || raw.contains('\\') || raw.contains("${") || parameterized;
        Some(Found {
            byte: node.start_byte(),
            name: display.clone(),
            filter: (!dynamic).then_some(display),
            confidence: if dynamic {
                Confidence::Dynamic
            } else {
                Confidence::Syntax
            },
        })
    })
}
fn heuristic(source: &str, csharp: bool) -> Result<CandidateScan, String> {
    if source.len() > 2 * 1024 * 1024 {
        return Err("Source test analysis supports up to 2 MiB. Use runner discovery.".into());
    }
    let mut found = vec![];
    let mut offset = 0;
    let mut attribute: Option<usize> = None;
    let mut block_comment = false;
    let mut raw_string = false;
    for (row, line) in source.split_inclusive('\n').enumerate() {
        let trimmed = line.trim();
        let blocked = block_comment || raw_string || trimmed.starts_with("//");
        if trimmed.contains("/*") {
            block_comment = true
        }
        if trimmed.contains("*/") {
            block_comment = false
        }
        if !csharp && line.bytes().filter(|byte| *byte == b'`').count() % 2 == 1 {
            raw_string = !raw_string
        }
        if !blocked && !trimmed.starts_with("/*") {
            let mut declaration = trimmed;
            if csharp && trimmed.starts_with('[') {
                let after = trimmed.trim_start_matches('[').trim_start();
                if ["Fact", "Theory", "Test", "TestMethod", "TestCase"]
                    .iter()
                    .any(|name| {
                        after.strip_prefix(*name).is_some_and(|tail| {
                            tail.starts_with(['(', ']', ','])
                                || tail.starts_with(char::is_whitespace)
                        })
                    })
                {
                    attribute = Some(row)
                }
                if let Some(end) = trimmed.rfind(']') {
                    declaration = trimmed[end + 1..].trim()
                }
            }
            let name = if csharp {
                attribute
                    .filter(|start| row.saturating_sub(*start) <= 6)
                    .and_then(|_| declaration.split_once('('))
                    .and_then(|(head, _)| head.split_whitespace().last())
                    .filter(|name| name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            } else {
                declaration
                    .strip_prefix("func")
                    .filter(|tail| tail.starts_with(char::is_whitespace))
                    .and_then(|tail| tail.trim_start().split_once('('))
                    .filter(|(_, args)| {
                        args.split(')').next().is_some_and(|args| {
                            args.split_whitespace()
                                .collect::<String>()
                                .contains("*testing.T")
                        })
                    })
                    .map(|(name, _)| name.trim())
                    .filter(|name| {
                        name.starts_with("Test")
                            && name.len() > 4
                            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    })
            };
            if let Some(name) = name {
                let byte = offset + line.find(name).unwrap_or(0);
                found.push(Found {
                    byte,
                    name: name.into(),
                    filter: Some(name.into()),
                    confidence: Confidence::Heuristic,
                });
                attribute = None
            }
            if attribute.is_some_and(|start| row.saturating_sub(start) > 6) {
                attribute = None
            }
        }
        offset += line.len();
    }
    finish(source,found,false,vec!["Text-recognized candidates only. Use explicit runner discovery to confirm test identities.".into()])
}
pub fn csharp(_language: &str, source: &str) -> Result<CandidateScan, String> {
    heuristic(source, true)
}
pub fn go(_language: &str, source: &str) -> Result<CandidateScan, String> {
    heuristic(source, false)
}
/// Dispatch through the same language capability that plans an explicit run.
pub fn detect(language: &str, source: &str) -> Result<CandidateScan, String> {
    let language = crate::routing::language_id(language).unwrap_or(language);
    match crate::registry().interface(language, crate::InterfaceKind::TestRunner) {
        Some(crate::Interface::TestRunner {
            detect: Some(detector),
            ..
        }) => detector(language, source),
        _ => Err(format!(
            "No source test provider for {language}. Choose a supported language or use configured project tests."
        )),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn python_candidates_exclude_examples_and_keep_partial_evidence() {
        let source = "example = \"\"\"\ndef test_example(): pass\n\"\"\"\ndef test_real():\n    pass\ndef helper():\n    def test_nested(): pass\n";
        let scan = detect("py", source).unwrap();
        assert_eq!(
            scan.candidates
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>(),
            ["test_real"]
        );
        assert!(scan.syntax_complete);
        let item = &scan.candidates[0];
        assert!(
            source
                .lines()
                .nth(item.line as usize - 1)
                .unwrap()
                .contains("def test_real")
        );
        assert_eq!(item.name_filter.as_deref(), Some("test_real"));
        let broken = detect("python", "def test_kept():\n    pass\nif ???\n").unwrap();
        assert!(!broken.syntax_complete);
        assert!(!broken.notices.is_empty());
        assert!(
            broken
                .candidates
                .iter()
                .any(|item| item.name == "test_kept")
        );
    }
    #[test]
    fn rust_candidates_follow_test_attributes_not_strings() {
        let source = "const EXAMPLE: &str = \"#[test] fn fake() {}\";\n#[test]\n#[ignore]\n/// docs\nfn first() {}\nfn ordinary() {}\n#[tokio::test]\nasync fn second() {}\n";
        let scan = detect("rust", source).unwrap();
        assert_eq!(
            scan.candidates
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        assert!(
            scan.candidates
                .iter()
                .all(|c| c.confidence == Confidence::Syntax)
        );
    }
    #[test]
    fn script_candidates_preserve_dynamic_names_and_utf16_positions() {
        let source = "// it('comment',()=>{});\nconst example=\"test('string',()=>{})\";\nit.only('works',()=>{});\ntest . each([[1]])('number %s',()=>{});\ntest('escaped\\nname',()=>{});\ntest . skipIf(false)('conditional',()=>{});\n";
        let scan = detect("typescript", source).unwrap();
        assert_eq!(
            scan.candidates[..3]
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>(),
            ["works", "number %s", "escaped\\nname"]
        );
        assert_eq!(scan.candidates[0].name_filter.as_deref(), Some("works"));
        assert!(
            scan.candidates[1..3]
                .iter()
                .all(|c| c.name_filter.is_none() && c.confidence == Confidence::Dynamic)
        );
        assert_eq!(scan.candidates.len(), 4);
        assert_eq!(
            scan.candidates[3].name_filter.as_deref(),
            Some("conditional")
        );
        let unicode = detect("javascript", "\"😀\"; it(\"shown\",()=>{});").unwrap();
        assert_eq!(unicode.candidates[0].byte, 8);
        assert_eq!(unicode.candidates[0].character, 6);
    }
    #[test]
    fn fallback_candidates_remain_heuristic_and_admission_is_bounded() {
        let cs = detect("csharp", "[Fact] public void Works() {}\n").unwrap();
        assert_eq!(cs.candidates[0].name, "Works");
        assert_eq!(cs.candidates[0].confidence, Confidence::Heuristic);
        assert!(!cs.syntax_complete);
        assert!(!cs.notices.is_empty());
        let go=detect("go","var sample = `\nfunc TestFake(t *testing.T) {}\n`\nfunc\tTestReal(t *testing.T) {}\nfunc TestMain(m *testing.M) {}\n").unwrap();
        assert_eq!(
            go.candidates
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>(),
            ["TestReal"]
        );
        assert!(!go.syntax_complete);
        assert!(
            detect("java", "")
                .unwrap_err()
                .contains("supported language")
        );
        assert!(
            detect("python", &" ".repeat(2 * 1024 * 1024 + 1))
                .unwrap_err()
                .contains("2 MiB")
        );
    }
}
