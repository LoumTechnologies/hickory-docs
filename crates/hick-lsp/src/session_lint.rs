//! Problems in a `hick:session` document, as LSP diagnostics.
//!
//! A session is a record of a conversation, and a record can be incomplete or
//! inconsistent in ways a person reading it would want marked: a turn nobody
//! answered, a tool call whose result never came back, a `parent=` that names
//! a turn the file does not hold. Those are problems WITH THE RECORD and they
//! count — errors and warnings, in the status bar with everything else.
//!
//! What the agent did is not a problem with the record. A refused tool, a
//! command that exited 1, a compaction the harness performed — those are
//! facts the conversation contains, and the reader wants them findable
//! without the status bar saying the document is broken. They are published
//! as *information*: a marker in the gutter, a line in the problems list,
//! never a count. (The web client counts errors and warnings only — see
//! `apps/web/src/lib/problems.ts`.)
//!
//! Only a document whose root is `hick:session` is linted here; a note is
//! somebody else's business.

use hick_lang::{HickDocument, HickNode, HickTag};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::structural::byte_to_position;

const SOURCE: &str = "hick-session";

/// Diagnostics for `doc`, or none when it is not a session.
pub fn session_diagnostics(source: &str, doc: &HickDocument) -> Vec<Diagnostic> {
    let Some(root) = doc.tags().find(|t| t.name == "session") else {
        return Vec::new();
    };
    let turns: Vec<&HickTag> = root
        .children
        .iter()
        .filter_map(|n| match n {
            HickNode::Tag(t) => Some(t),
            HickNode::Text(..) => None,
        })
        .collect();

    let mut out = Vec::new();
    lint_turns(source, &turns, &mut out);
    lint_tool_calls(source, &turns, &mut out);
    lint_outcomes(source, &turns, &mut out);
    lint_broken_close_tags(source, &mut out);
    out
}

/// The range of a tag's opening marker — the line a reader's eye lands on.
fn range_of(source: &str, tag: &HickTag) -> Range {
    match &tag.source_span {
        Some(span) => {
            let (sl, sc) = byte_to_position(source, span.start);
            let (el, ec) = byte_to_position(source, span.end);
            Range {
                start: Position {
                    line: sl,
                    character: sc,
                },
                end: Position {
                    line: el,
                    character: ec,
                },
            }
        }
        None => {
            let line = tag.source_line.saturating_sub(1) as u32;
            Range {
                start: Position { line, character: 0 },
                end: Position {
                    line,
                    character: u32::MAX,
                },
            }
        }
    }
}

fn diagnostic(
    range: Range,
    severity: DiagnosticSeverity,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(severity),
        source: Some(SOURCE.to_string()),
        message: message.into(),
        ..Default::default()
    }
}

/// Turns: each `hick:user` needs a reply, a `parent=` must name a turn in
/// this file, and two turns must not share an id.
fn lint_turns(source: &str, turns: &[&HickTag], out: &mut Vec<Diagnostic>) {
    let ids: Vec<Option<&str>> = turns
        .iter()
        .filter(|t| t.name == "user")
        .map(|t| t.get_attribute("turn"))
        .collect();
    let mut seen: Vec<&str> = Vec::new();
    for (i, t) in turns.iter().enumerate() {
        if t.name != "user" {
            continue;
        }
        // Answered: an assistant element before the next user element.
        let answered = turns[i + 1..]
            .iter()
            .take_while(|n| n.name != "user")
            .any(|n| n.name == "assistant");
        if !answered {
            out.push(diagnostic(
                range_of(source, t),
                DiagnosticSeverity::WARNING,
                "This turn has no reply recorded — the conversation ended (or was interrupted) \
                 before the agent answered.",
            ));
        }
        if let Some(turn) = t.get_attribute("turn") {
            if seen.contains(&turn) {
                out.push(diagnostic(
                    range_of(source, t),
                    DiagnosticSeverity::ERROR,
                    format!("Two turns share the id `{turn}`; the tree cannot tell them apart."),
                ));
            }
            seen.push(turn);
        }
        if let Some(parent) = t.get_attribute("parent")
            && !ids.contains(&Some(parent))
        {
            out.push(diagnostic(
                range_of(source, t),
                DiagnosticSeverity::WARNING,
                format!(
                    "`parent=\"{parent}\"` names a turn this file does not hold — the branch \
                     point is missing, so the tree view will root this turn."
                ),
            ));
        }
    }
}

/// Tool calls: every `hick:tool` inside an assistant turn should have a
/// `hick:tool-result` before the next assistant or user element — matched by
/// `call=` when both sides carry it, by order otherwise.
fn lint_tool_calls(source: &str, turns: &[&HickTag], out: &mut Vec<Diagnostic>) {
    for (i, t) in turns.iter().enumerate() {
        if t.name != "assistant" {
            continue;
        }
        let calls: Vec<&HickTag> = t
            .children
            .iter()
            .filter_map(|n| match n {
                HickNode::Tag(c) if c.name == "tool" => Some(c),
                _ => None,
            })
            .collect();
        if calls.is_empty() {
            continue;
        }
        let results: Vec<&HickTag> = turns[i + 1..]
            .iter()
            .take_while(|n| n.name != "assistant" && n.name != "user")
            .filter(|n| n.name == "tool-result")
            .copied()
            .collect();
        for (k, call) in calls.iter().enumerate() {
            let matched = match call.get_attribute("call") {
                Some(id) => results
                    .iter()
                    .any(|r| r.get_attribute("call").is_none_or(|rid| rid == id)),
                None => results.len() > k,
            };
            if !matched {
                let name = call.get_attribute("name").unwrap_or("?");
                out.push(diagnostic(
                    range_of(source, call),
                    DiagnosticSeverity::WARNING,
                    format!(
                        "The `{name}` call has no result recorded — the session ended, or was cut, \
                         before the tool answered."
                    ),
                ));
            }
        }
    }
}

/// Outcomes the reader wants to find: a refused or failed tool, a command
/// that did not exit 0, a compaction. Information, never a count.
fn lint_outcomes(source: &str, turns: &[&HickTag], out: &mut Vec<Diagnostic>) {
    for t in turns {
        match t.name.as_str() {
            "tool-result" if t.get_attribute("ok") == Some("false") => {
                let name = t.get_attribute("name").unwrap_or("tool");
                let first = first_line(&t.text_content());
                out.push(diagnostic(
                    range_of(source, t),
                    DiagnosticSeverity::INFORMATION,
                    format!("`{name}` refused or failed: {first}"),
                ));
            }
            "observation" => {
                if let Some(exit) = t.get_attribute("exit")
                    && exit != "0"
                {
                    out.push(diagnostic(
                        range_of(source, t),
                        DiagnosticSeverity::INFORMATION,
                        format!("The command exited {exit}."),
                    ));
                }
            }
            "context" if t.get_attribute("kind") == Some("compact") => {
                out.push(diagnostic(
                    range_of(source, t),
                    DiagnosticSeverity::INFORMATION,
                    "The conversation was compacted here: what the model saw after this point \
                     is a summary of what came before, not the turns themselves.",
                ));
            }
            _ => {}
        }
    }
}

/// `</hick:input >` and friends: a verbatim body that quoted its own close
/// tag, which `hick ingest --from claude-code` broke with a space so the element would not end
/// early. The one place an imported byte differs from the transcript.
fn lint_broken_close_tags(source: &str, out: &mut Vec<Diagnostic>) {
    for element in ["input", "tool-result", "reasoning", "context", "transcript"] {
        let token = format!("</hick:{element} >");
        let mut from = 0;
        while let Some(at) = source[from..].find(&token) {
            let start = from + at;
            let (sl, sc) = byte_to_position(source, start);
            let (el, ec) = byte_to_position(source, start + token.len());
            out.push(diagnostic(
                Range {
                    start: Position {
                        line: sl,
                        character: sc,
                    },
                    end: Position {
                        line: el,
                        character: ec,
                    },
                },
                DiagnosticSeverity::INFORMATION,
                format!(
                    "This body quoted its own close tag; the import broke `</hick:{element}>` \
                     with a space so the element would not end here. The original had no space."
                ),
            ));
            from = start + token.len();
        }
    }
}

fn first_line(text: &str) -> String {
    let line = text.trim().lines().next().unwrap_or("").trim();
    if line.chars().count() > 80 {
        let cut: String = line.chars().take(77).collect();
        format!("{cut}…")
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lint(body: &str) -> Vec<Diagnostic> {
        let src = format!(
            "<hick:session xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:session>\n"
        );
        let doc = hick_lang::parse(&src).unwrap();
        session_diagnostics(&src, &doc)
    }

    fn messages(d: &[Diagnostic]) -> Vec<String> {
        d.iter().map(|d| d.message.clone()).collect()
    }

    #[test]
    fn a_note_is_not_linted() {
        let src = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n<hick:user>x</hick:user>\n</hick:doc>\n";
        let doc = hick_lang::parse(src).unwrap();
        assert!(session_diagnostics(src, &doc).is_empty());
    }

    #[test]
    fn an_unanswered_turn_is_a_warning_on_its_line() {
        // docs/guarantees/agent/a-session-is-the-conversation.md
        let d = lint(
            "<hick:user turn=\"a\">hello</hick:user>\n<hick:assistant>hi</hick:assistant>\n<hick:user turn=\"b\" parent=\"a\">and?</hick:user>",
        );
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, Some(DiagnosticSeverity::WARNING));
        assert!(d[0].message.contains("no reply"));
        assert_eq!(d[0].range.start.line, 3);
    }

    #[test]
    fn a_parent_that_names_no_turn_and_a_duplicate_id_are_problems() {
        let d = lint(
            "<hick:user turn=\"a\" parent=\"zzz\">x</hick:user>\n<hick:assistant>y</hick:assistant>\n<hick:user turn=\"a\">x</hick:user>\n<hick:assistant>y</hick:assistant>",
        );
        let m = messages(&d);
        assert!(m.iter().any(|s| s.contains("parent=\"zzz\"")));
        assert!(
            d.iter()
                .any(|d| d.severity == Some(DiagnosticSeverity::ERROR)
                    && d.message.contains("share the id"))
        );
    }

    #[test]
    fn a_tool_call_without_a_result_is_a_warning_by_id_or_by_order() {
        let d = lint(
            "<hick:user>x</hick:user>\n<hick:assistant>\n<hick:tool name=\"Read\" call=\"t1\"></hick:tool>\n<hick:tool name=\"Bash\" call=\"t2\"></hick:tool>\n</hick:assistant>\n<hick:tool-result name=\"Read\" call=\"t1\" ok=\"true\">ok</hick:tool-result>\n<hick:assistant>done</hick:assistant>",
        );
        let m = messages(&d);
        assert_eq!(m.len(), 1, "{m:?}");
        assert!(m[0].contains("`Bash` call has no result"));
        let d = lint(
            "<hick:user>x</hick:user>\n<hick:assistant>\n<hick:tool name=\"read_doc\"></hick:tool>\n</hick:assistant>\n<hick:assistant>done</hick:assistant>",
        );
        assert!(messages(&d)[0].contains("`read_doc` call has no result"));
    }

    #[test]
    fn outcomes_are_information_not_problems() {
        let d = lint(
            "<hick:user>x</hick:user>\n<hick:assistant>\n<hick:action lang=\"sh\">false</hick:action>\n</hick:assistant>\n<hick:observation source=\"action-0\" exit=\"1\"></hick:observation>\n<hick:tool-result name=\"write_doc\" ok=\"false\">\ndeclined — fixture\n</hick:tool-result>\n<hick:context kind=\"compact\">Conversation compacted</hick:context>\n<hick:assistant>done</hick:assistant>",
        );
        assert_eq!(d.len(), 3, "{:?}", messages(&d));
        assert!(
            d.iter()
                .all(|d| d.severity == Some(DiagnosticSeverity::INFORMATION))
        );
        let m = messages(&d);
        assert!(m.iter().any(|s| s == "The command exited 1."));
        assert!(
            m.iter()
                .any(|s| s == "`write_doc` refused or failed: declined — fixture")
        );
        assert!(m.iter().any(|s| s.contains("compacted")));
    }

    #[test]
    fn a_broken_close_tag_is_pointed_at_exactly() {
        let d = lint(
            "<hick:user>x</hick:user>\n<hick:assistant>y</hick:assistant>\n<hick:tool-result name=\"Read\" ok=\"true\">\nquoted: </hick:tool-result >\n</hick:tool-result>",
        );
        let broken = d.iter().find(|d| d.message.contains("broke")).unwrap();
        assert_eq!(broken.range.start.line, 4);
        assert_eq!(broken.range.start.character, 8);
        assert_eq!(broken.severity, Some(DiagnosticSeverity::INFORMATION));
    }
}
