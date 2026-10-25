<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/claude_code/tests.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Claude Code Transcript Imports As A Session](../../../../../contract.md)

Captured SHA-256: `a8cbd3bf22061045e5cfc94a615a4f65cdb604e3a8f9b7d7bcc4f41e1823df57`.

<book:entry path="crates/hickory-cli/src/claude_code/tests.rs">
use super::*;

/// A small transcript with the shapes that matter: a prompt, a reply
/// that thinks, speaks and calls a tool (split across two records as
/// Claude Code writes them), the tool's answer, a second prompt on the
/// same branch, a meta prompt, a compaction boundary, a reminder
/// attachment, a title, and bookkeeping to skip.
const SMALL: &str = include_str!("../../tests/fixtures/claude-code-small.jsonl");

#[test]
fn a_transcript_becomes_a_session_that_reads_back() {
    // docs/guarantees/agent/a-claude-code-transcript-imports-as-a-session.md
    let c = convert(SMALL).unwrap();
    let doc = hick_lang::parse(&c.hick).unwrap();
    assert!(
        hick_blocks::attribute_errors(&c.hick, &doc).is_empty(),
        "{:?}",
        hick_blocks::attribute_errors(&c.hick, &doc)
    );
    let view = hickory_agent::session_view::session_view(&c.hick);
    assert_eq!(view.turns.len(), 2);
    assert_eq!(view.turns[0].prompt, "Count the lines in fruit.csv");
    assert_eq!(view.turns[0].model.as_deref(), Some("claude-opus-5"));
    assert_eq!(view.turns[0].provider.as_deref(), Some("anthropic"));
    // The second prompt names the first as its parent: the tree is kept.
    assert_eq!(
        view.turns[1].parent.as_deref(),
        Some(view.turns[0].id.as_str())
    );
    let kinds: Vec<&str> = view.turns[0]
        .steps
        .iter()
        .map(|s| match s {
            hickory_agent::session_view::Step::Reasoning { .. } => "reasoning",
            hickory_agent::session_view::Step::Prose { .. } => "prose",
            hickory_agent::session_view::Step::Tool { .. } => "tool",
            hickory_agent::session_view::Step::ToolResult { .. } => "tool-result",
            hickory_agent::session_view::Step::Context { .. } => "context",
            _ => "other",
        })
        .collect();
    // The meta caveat and the compaction both sit on the first turn, as
    // context the model was shown before the second prompt.
    assert_eq!(
        kinds,
        [
            "reasoning",
            "prose",
            "tool",
            "tool-result",
            "prose",
            "context",
            "context"
        ]
    );
    assert_eq!(
        view.turns[0].answer.as_deref(),
        Some("fruit.csv has 3 lines.")
    );
    // Usage rode along.
    assert_eq!(view.turns[0].usage.as_ref().map(|u| u.output), Some(50));
    assert_eq!(c.title.as_deref(), Some("Count fruit lines"));
    assert_eq!(c.file_name(), "20260820-090000-count-fruit-lines.md");
}

#[test]
fn tool_calls_and_results_keep_their_ids_and_names() {
    let c = convert(SMALL).unwrap();
    assert!(
        c.hick
            .contains(r#"<hick:tool name="Bash" call="toolu_01">"#)
    );
    assert!(
        c.hick
            .contains(r#"<hick:arg name="command">wc -l < fruit.csv</hick:arg>"#)
    );
    assert!(
        c.hick
            .contains(r#"<hick:tool-result id="2" name="Bash" call="toolu_01" ok="true""#)
    );
    assert!(c.hick.contains(
        "<hick:usage turn=\"0\" input=\"100\" cache-write=\"5\" cache-read=\"50\" output=\"40\"/>"
    ));
}

#[test]
fn what_is_dropped_is_counted() {
    let c = convert(SMALL).unwrap();
    assert_eq!(
        c.stats.dropped.get("total_tokens_reminder attachment"),
        Some(&1)
    );
    assert_eq!(c.stats.dropped.get("mode record"), Some(&1));
    assert_eq!(c.stats.dropped.get("thinking signature"), Some(&1));
    assert_eq!(c.stats.context, 2); // the caveat echo and the compaction
    assert!(c.hick.contains(r#"<hick:context kind="compact""#));
    assert!(c.hick.contains(r#"<hick:context kind="local-command""#));
}

#[test]
fn a_prompt_or_result_quoting_a_hick_tag_still_parses() {
    // The no-escaping invariant: a tool result that prints an unclosed
    // tag must not swallow the rest of the session, and a prompt that
    // quotes one is wrapped where the parser captures verbatim.
    let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"What does <hick:exec> do?"}}
{"type":"assistant","uuid":"a1","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"id":"m1","model":"claude-opus-5","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"a.md"}}],"usage":{"input_tokens":1,"output_tokens":1}}}
{"type":"user","uuid":"u2","parentUuid":"a1","timestamp":"2026-01-01T00:00:02.000Z","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","content":"<hick:doc>\n<hick:exec container=\"x\">\nls\n"}]}}
{"type":"assistant","uuid":"a2","parentUuid":"u2","timestamp":"2026-01-01T00:00:03.000Z","message":{"id":"m2","model":"claude-opus-5","content":[{"type":"text","text":"It runs a cell, written as <hick:exec>."}],"usage":{"input_tokens":1,"output_tokens":1}}}
{"type":"user","uuid":"u3","parentUuid":"a2","timestamp":"2026-01-01T00:00:04.000Z","message":{"role":"user","content":"Thanks"}}
"#;
    let c = convert(jsonl).unwrap();
    let view = hickory_agent::session_view::session_view(&c.hick);
    assert_eq!(view.turns.len(), 2);
    assert_eq!(view.turns[0].prompt, "What does <hick:exec> do?");
    assert_eq!(
        view.turns[0].answer.as_deref(),
        Some("It runs a cell, written as <hick:exec>.")
    );
    assert_eq!(view.turns[1].prompt, "Thanks");
    assert_eq!(c.stats.adapted.len(), 2);
}

#[test]
fn a_slash_command_echo_is_context_not_a_prompt() {
    let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"<command-name>/model</command-name>\n<command-message>model</command-message>"}}
{"type":"user","uuid":"u2","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"role":"user","content":"<local-command-stdout>Set model to Sonnet 5</local-command-stdout>"}}
{"type":"user","uuid":"u3","parentUuid":"u2","timestamp":"2026-01-01T00:00:02.000Z","message":{"role":"user","content":"Now help me."}}
"#;
    let c = convert(jsonl).unwrap();
    assert_eq!(c.stats.prompts, 1);
    assert_eq!(c.stats.context, 2);
    assert!(c.hick.contains(r#"<hick:context kind="local-command""#));
    let view = hickory_agent::session_view::session_view(&c.hick);
    assert_eq!(view.turns[0].prompt, "Now help me.");
    assert_eq!(view.turns[0].parent, None);
}

#[test]
fn a_comment_marker_in_prose_is_carried_verbatim_too() {
    // `<!--` is structure to the parser (a comment, skipped) — a prompt
    // that mentions one must not lose the words after it.
    let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"Why does the file start with <!-- woven by hickory?"}}
{"type":"assistant","uuid":"a1","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Write","input":{"content":"<!-- banner\nbody"}}],"usage":{}}}
"#;
    let c = convert(jsonl).unwrap();
    assert!(c.stats.problems.is_empty(), "{:?}", c.stats.problems);
    let view = hickory_agent::session_view::session_view(&c.hick);
    assert_eq!(
        view.turns[0].prompt,
        "Why does the file start with <!-- woven by hickory?"
    );
}

#[test]
fn a_body_quoting_its_own_close_tag_is_broken_not_truncated() {
    // A tool result that printed a session file: the literal
    // `</hick:tool-result>` inside it would end the element early.
    let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"show the session"}}
{"type":"assistant","uuid":"a1","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"s.md"}}],"usage":{}}}
{"type":"user","uuid":"u2","parentUuid":"a1","timestamp":"2026-01-01T00:00:02.000Z","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","content":"<hick:tool-result name=\"x\">\nhi\n</hick:tool-result>\nafter"}]}}
{"type":"assistant","uuid":"a2","parentUuid":"u2","timestamp":"2026-01-01T00:00:03.000Z","message":{"id":"m2","model":"m","content":[{"type":"text","text":"Done."}],"usage":{}}}
"#;
    let c = convert(jsonl).unwrap();
    assert!(c.stats.problems.is_empty(), "{:?}", c.stats.problems);
    assert!(c.hick.contains("</hick:tool-result >\nafter"));
    let view = hickory_agent::session_view::session_view(&c.hick);
    assert_eq!(view.turns[0].answer.as_deref(), Some("Done."));
    assert_eq!(c.stats.adapted.len(), 1);
}

#[test]
fn a_tool_input_quoting_a_hick_tag_is_written_as_json() {
    let jsonl = r#"
{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00.000Z","message":{"role":"user","content":"edit"}}
{"type":"assistant","uuid":"a1","parentUuid":"u1","timestamp":"2026-01-01T00:00:01.000Z","message":{"id":"m1","model":"m","content":[{"type":"tool_use","id":"t1","name":"Edit","input":{"file_path":"a.md","new_string":"<hick:file path=\"x\">"}}],"usage":{}}}
"#;
    let c = convert(jsonl).unwrap();
    assert!(c.hick.contains("<hick:input>\n{\n"));
    assert!(!c.hick.contains("<hick:arg"));
    let view = hickory_agent::session_view::session_view(&c.hick);
    assert!(matches!(
        view.turns[0].steps.first(),
        Some(hickory_agent::session_view::Step::Tool { input: Some(_), .. })
    ));
}

#[test]
fn attributes_pick_the_quote_the_value_lacks() {
    let mut s = Stats::default();
    let mut out = String::new();
    attr(&mut out, "title", r#"Fix the "thing""#, &mut s);
    assert_eq!(out, r#" title='Fix the "thing"'"#);
    let mut out = String::new();
    attr(&mut out, "title", r#"It's "both""#, &mut s);
    assert_eq!(out, "");
    assert_eq!(s.dropped.len(), 1);
}

#[test]
fn file_names_are_deterministic_and_dated() {
    assert_eq!(
        stamp_of("2026-08-13T16:29:17.371Z").as_deref(),
        Some("20260813-162917")
    );
    assert_eq!(
        slug("Review uncommitted updates and verify documentation"),
        "review-uncommitted-updates-and-verify-do"
    );
    assert_eq!(slug("///"), "session");
}

#[test]
fn an_empty_or_foreign_file_is_refused_with_a_reason() {
    assert!(convert("").is_err());
    assert!(
        convert("not json\n")
            .unwrap_err()
            .to_string()
            .contains("no records")
    );
}
</book:entry>

</book:doc>
