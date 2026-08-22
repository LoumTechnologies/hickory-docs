//! `<hick:expect>` verification support.
//!
//! An `<hick:expect>` child of `<hick:exec>` declares the expected stdout of
//! that exec. It is verification metadata: the exec's command text excludes
//! the expect subtree (see `hick_exec::dag`), and expectations never feed
//! stdin.
//!
//! Matching modes (`match` attribute):
//! - `exact` (default): expected body must equal the captured stdout,
//!   byte for byte.
//! - `regex-lines`: each expectation line is a full-line regex (implicitly
//!   anchored) matched against the corresponding output line; the
//!   expectation must cover ALL output lines (same line count).
//!
//! During `hick run`, expectations are evaluated and recorded but never
//! fail the run. `hick test` turns any unmet expectation into a
//! non-zero exit.

use hick_lang::{HickNode, HickTag, SourceSpan};
use serde::Serialize;

use crate::{CellId, tag_attr};

/// How an expectation body is matched against output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatchMode {
    Exact,
    RegexLines,
}

impl std::fmt::Display for MatchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MatchMode::Exact => "exact",
            MatchMode::RegexLines => "regex-lines",
        })
    }
}

impl MatchMode {
    pub fn parse(s: Option<&str>) -> Result<Self, String> {
        match s {
            None | Some("exact") => Ok(MatchMode::Exact),
            Some("regex-lines") => Ok(MatchMode::RegexLines),
            Some(other) => Err(format!(
                "unknown expect match mode '{other}' (expected \"exact\" or \"regex-lines\")"
            )),
        }
    }
}

/// A parsed expectation attached to one cell.
#[derive(Debug, Clone)]
pub struct ExpectSpec {
    /// Which cell the expectation belongs to.
    ///
    /// A [`CellId`] rather than the `(container, exec_line)` pair this used to
    /// be, for the same reason `never_run` was generalized: an agent cell has
    /// no container, and a key that assumes one cannot name it.
    pub cell: CellId,
    /// Source line of the owning cell's tag (1-based).
    pub exec_line: usize,
    /// Source span of the owning exec tag, when available.
    pub exec_span: Option<SourceSpan>,
    /// Matching mode.
    pub mode: MatchMode,
    /// Raw expectation body (text content of the expect tag).
    pub body: String,
}

/// The outcome of evaluating one expectation against real output.
#[derive(Debug, Clone, Serialize)]
pub struct ExpectationOutcome {
    pub doc: String,
    /// Container of the cell that produced the output, when it has one. An
    /// agent cell has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    /// 1-based source line of the exec block.
    pub line: usize,
    /// Byte span of the exec block in the source, when known.
    pub span: Option<(usize, usize)>,
    pub mode: MatchMode,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
    /// Human-readable mismatch detail (empty when passed).
    pub detail: String,
}

/// Collect all expectations declared in a document, in document order.
///
/// Both `<hick:exec>` and `<hick:agent>` may carry one — an agent cell's
/// expectation asserts something about the answer it settled on, which is as
/// legitimate a claim as any about a command's stdout, and is why the key is
/// a [`CellId`] rather than a container name.
///
/// Returns an error for malformed expectations (unknown match mode, multiple
/// expect children on one cell).
pub fn collect_expectations(nodes: &[HickNode]) -> Result<Vec<ExpectSpec>, String> {
    let mut specs = Vec::new();
    collect_from_nodes(nodes, &mut specs)?;
    Ok(specs)
}

fn collect_from_nodes(nodes: &[HickNode], specs: &mut Vec<ExpectSpec>) -> Result<(), String> {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            let cell = match tag.name.as_str() {
                "exec" => Some(CellId::exec(
                    tag_attr(tag, "container").unwrap_or_default(),
                    tag.source_line,
                )),
                "agent" => Some(CellId::containerless(tag.source_line)),
                _ => None,
            };
            if let Some(cell) = cell {
                let expects: Vec<&HickTag> =
                    tag.child_tags().filter(|t| t.name == "expect").collect();
                if expects.len() > 1 {
                    return Err(format!(
                        "{} block at line {} has {} <hick:expect> children (max 1)",
                        tag.name,
                        tag.source_line,
                        expects.len()
                    ));
                }
                if let Some(expect_tag) = expects.first() {
                    let mode = MatchMode::parse(tag_attr(expect_tag, "match").as_deref())
                        .map_err(|e| format!("line {}: {e}", expect_tag.source_line))?;
                    specs.push(ExpectSpec {
                        exec_line: tag.source_line,
                        exec_span: tag.source_span,
                        cell,
                        mode,
                        body: expect_tag.text_content(),
                    });
                }
            }
            collect_from_nodes(&tag.children, specs)?;
        }
    }
    Ok(())
}

/// Evaluate an expectation against captured stdout.
pub fn evaluate(spec: &ExpectSpec, doc: &str, actual: &str) -> ExpectationOutcome {
    let (passed, detail) = match spec.mode {
        MatchMode::Exact => {
            if spec.body == actual {
                (true, String::new())
            } else {
                (false, diff_detail(&spec.body, actual))
            }
        }
        MatchMode::RegexLines => match_regex_lines(&spec.body, actual),
    };
    ExpectationOutcome {
        doc: doc.to_string(),
        container: spec.cell.container().map(str::to_string),
        line: spec.exec_line,
        span: spec.exec_span.map(|s| (s.start, s.end)),
        mode: spec.mode,
        expected: spec.body.clone(),
        actual: actual.to_string(),
        passed,
        detail,
    }
}

fn diff_detail(expected: &str, actual: &str) -> String {
    let exp_lines: Vec<&str> = expected.lines().collect();
    let act_lines: Vec<&str> = actual.lines().collect();
    // The one mistake everybody makes once: a line break after the opening
    // tag, so the expectation's first line is empty and nothing else can
    // match. The message has to name it, or the author reads "expected
    // nothing" and goes looking for a cell that printed too much.
    if let (Some(first), Some(a)) = (exp_lines.first(), act_lines.first())
        && first.is_empty()
        && !a.is_empty()
        && exp_lines.get(1) == Some(a)
    {
        return format!(
            "the expectation begins with an empty line, but the output begins with {a:?}: \
             expected text starts right after the opening tag — write `<hick:expect …>{a}` \
             with no line break after `>`, so the first expected line is the first output line"
        );
    }
    for (i, (e, a)) in exp_lines.iter().zip(act_lines.iter()).enumerate() {
        if e != a {
            return format!(
                "first mismatch at output line {}: expected {e:?}, got {a:?}",
                i + 1
            );
        }
    }
    if exp_lines.len() != act_lines.len() {
        return format!(
            "line count differs: expected {} line(s), got {}",
            exp_lines.len(),
            act_lines.len()
        );
    }
    // Same lines but different bytes (trailing newline / whitespace).
    "outputs differ only in trailing whitespace or final newline".to_string()
}

fn match_regex_lines(expected: &str, actual: &str) -> (bool, String) {
    let exp_lines: Vec<&str> = expected.lines().collect();
    let act_lines: Vec<&str> = actual.lines().collect();
    if exp_lines.len() != act_lines.len() {
        return (
            false,
            format!(
                "line count differs: {} expectation line(s) vs {} output line(s) \
                 (regex-lines must cover all output lines)",
                exp_lines.len(),
                act_lines.len()
            ),
        );
    }
    for (i, (pat, line)) in exp_lines.iter().zip(act_lines.iter()).enumerate() {
        let anchored = format!("^(?:{pat})$");
        match regex::Regex::new(&anchored) {
            Ok(re) => {
                if !re.is_match(line) {
                    return (
                        false,
                        format!(
                            "output line {} does not match: pattern {pat:?}, got {line:?}",
                            i + 1
                        ),
                    );
                }
            }
            Err(e) => {
                return (
                    false,
                    format!("invalid regex on expectation line {}: {e}", i + 1),
                );
            }
        }
    }
    (true, String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(mode: MatchMode, body: &str) -> ExpectSpec {
        ExpectSpec {
            cell: CellId::exec("c", 1),
            exec_line: 1,
            exec_span: None,
            mode,
            body: body.into(),
        }
    }

    #[test]
    fn exact_match_passes_and_fails() {
        assert!(evaluate(&spec(MatchMode::Exact, "a\nb\n"), "d", "a\nb\n").passed);
        let out = evaluate(&spec(MatchMode::Exact, "a\nb\n"), "d", "a\nc\n");
        assert!(!out.passed);
        assert!(out.detail.contains("line 2"), "{}", out.detail);
    }

    /// A line break after `<hick:expect>` makes the first expected line
    /// empty; the message must say so rather than report "expected nothing".
    #[test]
    fn a_leading_newline_in_the_expectation_is_named() {
        let out = evaluate(
            &spec(MatchMode::Exact, "\nbaseline 203\n"),
            "d",
            "baseline 203\n",
        );
        assert!(!out.passed);
        assert!(
            out.detail.contains("begins with an empty line"),
            "{}",
            out.detail
        );
        assert!(
            out.detail.contains("no line break after `>`"),
            "{}",
            out.detail
        );
    }

    #[test]
    fn exact_is_byte_exact_about_trailing_newline() {
        let out = evaluate(&spec(MatchMode::Exact, "a\n"), "d", "a");
        assert!(!out.passed);
    }

    #[test]
    fn regex_lines_full_line_anchoring() {
        assert!(evaluate(&spec(MatchMode::RegexLines, r"n = \d+"), "d", "n = 30\n").passed);
        // Partial matches must not pass: pattern is anchored to the full line.
        assert!(!evaluate(&spec(MatchMode::RegexLines, r"\d+"), "d", "n = 30\n").passed);
    }

    #[test]
    fn regex_lines_must_cover_all_output() {
        let out = evaluate(&spec(MatchMode::RegexLines, r"one"), "d", "one\ntwo\n");
        assert!(!out.passed);
        assert!(out.detail.contains("line count"), "{}", out.detail);
    }

    #[test]
    fn collect_finds_expect_and_mode() {
        let src = r#"<?xml version="1.0"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="c">
echo hi
<hick:expect match="regex-lines">h.
</hick:expect>
</hick:exec>
</hick:doc>"#;
        let doc = hick_lang::parse(src).unwrap();
        let specs = collect_expectations(&doc.nodes).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].mode, MatchMode::RegexLines);
        assert_eq!(specs[0].body, "h.\n");
        assert_eq!(specs[0].cell, CellId::exec("c", 3));
    }

    #[test]
    fn unknown_mode_is_an_error() {
        let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="c">true
<hick:expect match="fuzzy">x</hick:expect>
</hick:exec>
</hick:doc>"#;
        let doc = hick_lang::parse(src).unwrap();
        assert!(collect_expectations(&doc.nodes).is_err());
    }
}
