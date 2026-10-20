/// Protects `docs/guarantees/authoring/a-document-may-begin-with-markdown.md`
#[test]
fn a_byte_order_mark_does_not_turn_a_wrapped_document_into_a_bare_one() {
    // Notepad, PowerShell's `Set-Content -Encoding UTF8`, and several
    // Windows editors write a BOM. `trim_start` does not remove it, so the
    // wrapper went unrecognised and every cell in the document was
    // silently skipped while the run reported success.
    let body = concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n",
        "# Title\n\n",
        "<hick:exec container=\"c\">\necho hi\n</hick:exec>\n",
        "</hick:doc>\n",
    );
    let plain = parse(body).expect("parses without a BOM");
    let with_bom = parse(&format!("\u{feff}{body}")).expect("parses with a BOM");

    assert_eq!(
        with_bom.weave_path, plain.weave_path,
        "the BOM hid the doc tag, so its attributes were lost"
    );
    assert_eq!(
        with_bom.nodes.len(),
        plain.nodes.len(),
        "a BOM changed how many nodes the document has"
    );
    assert!(
        plain.weave_path.is_some(),
        "the fixture must be a WRAPPED document, or this proves nothing"
    );
}

/// A BOM in the middle of a file is ordinary text and stays that way — the
/// no-escaping invariant is about content, and only the leading encoding
/// signature is not content.
#[test]
fn a_byte_order_mark_that_is_not_leading_is_left_alone() {
    let doc = parse("hello \u{feff} world").expect("a bare document");
    assert!(doc.source.contains('\u{feff}'), "{:?}", doc.source);
}
use super::*;
use std::path::Path;

/// The text a node list carries, tags flattened to their content — the
/// closest thing to "what does this weave" without running the weaver.
fn text_of(nodes: &[HickNode]) -> String {
    let mut out = String::new();
    collect_text(nodes, &mut out);
    out
}

#[test]
fn parse_minimal_document() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
</hick:doc>"#;
    let doc = parse(src).unwrap();
    assert_eq!(doc.prefix, "hick");
    assert!(doc.weave_path.is_none());
    assert!(
        doc.nodes.is_empty()
            || doc
                .nodes
                .iter()
                .all(|n| matches!(n, HickNode::Text(t, _) if t.trim().is_empty()))
    );
}

#[test]
fn python_fences_use_xml_attributes_and_explicit_exec_opt_in() {
    let doc = parse("```python exec=\"true\" id=\"summary\"\nprint(1)\n```\n").unwrap();
    let cell = doc.find_tags("exec").pop().unwrap();
    assert_eq!(cell.get_attribute("container"), Some("default"));
    assert_eq!(cell.get_attribute("id"), Some("summary"));
    assert_eq!(cell.text_content(), "print(1)\n");
    for info in [
        "python",
        "python exec=\"false\"",
        "python exec=\"false\" container=\"py\"",
        "python exec",
    ] {
        assert!(
            parse(&format!("```{info}\nprint(1)\n```\n"))
                .unwrap()
                .find_tags("exec")
                .is_empty()
        );
    }
}

#[test]
fn fenced_file_and_exec_are_normalised_to_elements() {
    let doc = parse(
        "```python path=\"test.py\"\nprint(\"Why not?\")\n```\n\n\
             ```shell container=\"reporter\" show=\"output\"\n\
             echo hello\n```\n",
    )
    .unwrap();
    let file = doc.find_tags("file").pop().unwrap();
    assert_eq!(file.get_attribute("path"), Some("test.py"));
    assert_eq!(file.get_attribute("language"), Some("python"));
    assert_eq!(file.text_content(), "print(\"Why not?\")\n");
    let exec = doc.find_tags("exec").pop().unwrap();
    assert_eq!(exec.get_attribute("container"), Some("reporter"));
    assert_eq!(exec.get_attribute("language"), Some("shell"));
    assert_eq!(exec.text_content(), "echo hello\n");
}

#[test]
fn fenced_code_is_opaque_unless_it_has_a_hick_attribute() {
    let doc = parse("```xml\n<hick:exec container=\"not-a-cell\" />\n```\n").unwrap();
    assert!(doc.find_tags("exec").is_empty());
    assert_eq!(doc.nodes.len(), 1);
    assert!(matches!(&doc.nodes[0], HickNode::Text(text, _) if text.contains("not-a-cell")));
}

#[test]
fn fenced_output_without_a_language_is_an_output_element() {
    let doc = parse("```output-for=\"#test1\" sha256=\"abc\" exit=\"0\"\nhello\n```\n").unwrap();
    let output = doc.find_tags("output").pop().unwrap();
    assert_eq!(output.get_attribute("output-for"), Some("#test1"));
    assert_eq!(output.get_attribute("sha256"), Some("abc"));
    assert_eq!(output.text_content(), "hello\n");
}

#[test]
fn parse_weave_attribute() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="TEMPLATE.md">
<hick:file path="out.txt">content</hick:file>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    assert_eq!(doc.weave_path, Some("TEMPLATE.md".to_string()));
}

#[test]
fn parse_weave_attribute_with_path() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="docs/README.md">
</hick:doc>"#;
    let doc = parse(src).unwrap();
    assert_eq!(doc.weave_path, Some("docs/README.md".to_string()));
}

#[test]
fn parse_text_content_with_special_chars() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
This has & and < and > without escaping.
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let text: String = doc
        .nodes
        .iter()
        .filter_map(|n| match n {
            HickNode::Text(t, _) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert!(text.contains("&"));
    assert!(text.contains("<"));
    assert!(text.contains(">"));
}

#[test]
fn parse_self_closing_tag() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:allow network="github.com:443" />
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let tags: Vec<_> = doc.find_tags("allow");
    assert_eq!(tags.len(), 1);
    assert!(tags[0].self_closing);
    assert_eq!(tags[0].get_attribute("network"), Some("github.com:443"));
}

#[test]
fn parse_nested_tags() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:deny network="*" />
</hick:container>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let containers = doc.find_tags("container");
    assert_eq!(containers.len(), 1);
    let c = containers[0];
    assert_eq!(c.get_attribute("name"), Some("reporter"));
    assert_eq!(c.get_attribute("image"), Some("python:3.12"));

    let child_tags: Vec<_> = c.child_tags().collect();
    assert_eq!(child_tags.len(), 2);
    assert_eq!(child_tags[0].name, "allow");
    assert_eq!(child_tags[1].name, "deny");
}

#[test]
fn parse_exec_with_raw_content() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
cat /etc/alpine-release
</hick:exec>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let execs = doc.find_tags("exec");
    assert_eq!(execs.len(), 1);
    assert_eq!(execs[0].get_attribute("container"), Some("demo"));
    assert_eq!(execs[0].get_attribute("image"), Some("alpine"));

    let content = execs[0].text_content();
    assert!(content.contains("cat /etc/alpine-release"));
}

#[test]
fn parse_file_with_nested_exec() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="Getting Started.md">
# Getting Started

    <hick:exec container="demo" image="alpine">
    cat /etc/alpine-release
    </hick:exec>

Done!
</hick:file>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].get_attribute("path"), Some("Getting Started.md"));

    let execs: Vec<_> = files[0].child_tags().collect();
    assert_eq!(execs.len(), 1);
    assert_eq!(execs[0].name, "exec");
}

#[test]
fn parse_copy_paste_cut() {
    let src = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="analyzer">
<hick:copy id="version-info">
python -c "import sys; print(sys.version)"
</hick:copy>
</hick:exec>

<hick:file path="report.md">
Python: <hick:paste select="#version-info" />
</hick:file>

<hick:cut id="internal-only">
This is removed from output.
</hick:cut>
</hick:doc>"##;
    let doc = parse(src).unwrap();

    let copies = doc.tags().flat_map(|t| {
        let mut found = Vec::new();
        find_tags_recursive(t, "copy", &mut found);
        found
    });
    assert_eq!(copies.count(), 1);

    let pastes = doc.tags().flat_map(|t| {
        let mut found = Vec::new();
        find_tags_recursive(t, "paste", &mut found);
        found
    });
    let pastes: Vec<_> = pastes.collect();
    assert_eq!(pastes.len(), 1);
    assert_eq!(pastes[0].get_attribute("select"), Some(r#"#version-info"#));
    assert!(pastes[0].self_closing);

    let cuts = doc.find_tags("cut");
    assert_eq!(cuts.len(), 1);
    assert_eq!(cuts[0].get_attribute("id"), Some("internal-only"));
}

// -----------------------------------------------------------------------
// Session node tests
// -----------------------------------------------------------------------

fn minimal_session(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="2026-01-01T00:00:00Z">
{body}
</hick:session>"#
    )
}

#[test]
fn parse_session_user_node() {
    let src = minimal_session("<hick:user>add a login page</hick:user>");
    let doc = parse_session(&src).unwrap();
    assert_eq!(doc.start_time.as_deref(), Some("2026-01-01T00:00:00Z"));
    assert_eq!(doc.nodes.len(), 1);
    assert_eq!(
        doc.nodes[0],
        SessionNode::User {
            text: "add a login page".into()
        }
    );
}

#[test]
fn parse_session_assistant_without_action() {
    let src = minimal_session("<hick:assistant>Here is the final answer.</hick:assistant>");
    let doc = parse_session(&src).unwrap();
    assert_eq!(
        doc.nodes[0],
        SessionNode::Assistant {
            text: "Here is the final answer.".into(),
            actions: vec![],
        }
    );
}

#[test]
fn parse_session_assistant_with_action() {
    let src = minimal_session(
        r#"<hick:assistant>
I will create the file now.
<hick:action lang="python">
import os
print(os.getcwd())
</hick:action>
</hick:assistant>"#,
    );
    let doc = parse_session(&src).unwrap();
    match &doc.nodes[0] {
        SessionNode::Assistant { actions, .. } => {
            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].lang, "python");
            assert!(actions[0].code.contains("os.getcwd()"));
        }
        other => panic!("expected Assistant, got {other:?}"),
    }
}

#[test]
fn parse_session_observation_node() {
    let src = minimal_session(
        r#"<hick:observation source="action-0" exit="0">Files created: src/login.py</hick:observation>"#,
    );
    let doc = parse_session(&src).unwrap();
    assert_eq!(
        doc.nodes[0],
        SessionNode::Observation {
            source: Some("action-0".into()),
            exit_code: Some(0),
            text: "Files created: src/login.py".into(),
        }
    );
}

#[test]
fn parse_session_command_node() {
    let src = minimal_session("<hick:command>ls -la src/</hick:command>");
    let doc = parse_session(&src).unwrap();
    assert_eq!(
        doc.nodes[0],
        SessionNode::Command {
            text: "ls -la src/".into()
        }
    );
}

#[test]
fn parse_session_tool_nodes_ride_along_with_raw_payloads() {
    // The agent tool vocabulary: hick:tool/hick:arg inside an assistant
    // turn, hick:input payloads and hick:tool-result observations that
    // may quote UNBALANCED hick fragments — captured verbatim, the
    // session still parses, and unknown tags are gracefully skipped.
    let src = minimal_session(
        r#"<hick:user>fix the constant</hick:user>
<hick:assistant>
Following the pointer.
<hick:tool name="edit_doc">
<hick:arg name="run">a1b2..c3d4</hick:arg>
<hick:input>
<hick:copy id="dup">unbalanced open, no close
</hick:input>
</hick:tool>
</hick:assistant>
<hick:tool-result name="edit_doc" ok="true">
edited region:
ffff|</hick:copy>
ffff|<hick:file path="gen.rs">// slice
</hick:tool-result>
<hick:assistant>Done.</hick:assistant>"#,
    );
    let doc = parse_session(&src).expect("tool session must parse");
    // user + two assistants; tool and tool-result are inert/skipped.
    assert_eq!(doc.nodes.len(), 3);
    assert!(matches!(doc.nodes[0], SessionNode::User { .. }));
    assert!(
        matches!(&doc.nodes[1], SessionNode::Assistant { text, actions }
                if text.contains("Following the pointer.") && actions.is_empty())
    );
    assert!(matches!(&doc.nodes[2], SessionNode::Assistant { text, .. } if text == "Done."));
}

#[test]
fn parse_session_full_turn_sequence() {
    let src = minimal_session(
        r#"<hick:user>add a login page</hick:user>
<hick:assistant>
I'll create the login page now.
<hick:action lang="python">
print("creating login page")
</hick:action>
</hick:assistant>
<hick:observation source="action-0" exit="0">creating login page</hick:observation>"#,
    );
    let doc = parse_session(&src).unwrap();
    assert_eq!(doc.nodes.len(), 3);
    assert!(matches!(doc.nodes[0], SessionNode::User { .. }));
    assert!(matches!(doc.nodes[1], SessionNode::Assistant { .. }));
    assert!(matches!(doc.nodes[2], SessionNode::Observation { .. }));
}

#[test]
fn parse_session_command_and_observation() {
    let src = minimal_session(
        r#"<hick:command>ls -la</hick:command>
<hick:observation source="command-0" exit="0">total 8
-rw-r--r-- 1 user user 42 main.py
</hick:observation>"#,
    );
    let doc = parse_session(&src).unwrap();
    assert_eq!(doc.nodes.len(), 2);
    assert!(matches!(doc.nodes[0], SessionNode::Command { .. }));
    assert!(matches!(doc.nodes[1], SessionNode::Observation { .. }));
}

#[test]
fn is_session_source_detection() {
    let session_src = minimal_session("<hick:user>hello</hick:user>");
    assert!(is_session_source(&session_src));

    let pipeline_src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="out.txt">hello</hick:file>
</hick:doc>"#;
    assert!(!is_session_source(pipeline_src));
}

#[test]
fn parse_session_mixed_with_unknown_tags() {
    // Unknown tags (not user/assistant/observation/command) are silently skipped
    let src = minimal_session(
        r#"<hick:user>hello</hick:user>
<hick:metadata version="1"/>
<hick:command>echo hi</hick:command>"#,
    );
    let doc = parse_session(&src).unwrap();
    assert_eq!(doc.nodes.len(), 2); // metadata skipped
    assert!(matches!(doc.nodes[0], SessionNode::User { .. }));
    assert!(matches!(doc.nodes[1], SessionNode::Command { .. }));
}

#[test]
fn pipeline_doc_with_session_like_nodes_parses_ok() {
    // A pipeline file can contain hick:user etc. without error (they're just unknown tags)
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="out.txt">hello</hick:file>
<hick:user>this is fine</hick:user>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    let users = doc.find_tags("user");
    assert_eq!(users.len(), 1);
}

fn find_tags_recursive<'a>(tag: &'a HickTag, name: &str, out: &mut Vec<&'a HickTag>) {
    if tag.name == name {
        out.push(tag);
    }
    for child in tag.child_tags() {
        find_tags_recursive(child, name, out);
    }
}

#[test]
fn parse_full_example_untrusted_code() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
  <hick:allow file-write="/output/*" />
</hick:container>

<hick:container name="mailer" image="alpine">
  <hick:allow network="smtp.gmail.com:587" />
  <hick:deny network="*" />
  <hick:allow file-read="/output/*" />
</hick:container>

<hick:volume name="shared-output" />

<hick:exec container="reporter" mount="shared-output:/output">
pip install untrusted-package
python -c "
import untrusted_package
with open('/output/report.html', 'w') as f:
    f.write(untrusted_package.generate())
"
</hick:exec>

<hick:exec container="mailer" mount="shared-output:/output">
apk add msmtp
cat /output/report.html | msmtp bob@example.com
</hick:exec>

</hick:doc>"#;
    let doc = parse(src).unwrap();

    // Containers
    let containers = doc.find_tags("container");
    assert_eq!(containers.len(), 2);
    assert_eq!(containers[0].get_attribute("name"), Some("reporter"));
    assert_eq!(containers[1].get_attribute("name"), Some("mailer"));

    // Reporter capabilities
    let reporter_rules: Vec<_> = containers[0].child_tags().collect();
    assert_eq!(reporter_rules.len(), 4);

    // Volumes
    let volumes = doc.find_tags("volume");
    assert_eq!(volumes.len(), 1);
    assert_eq!(volumes[0].get_attribute("name"), Some("shared-output"));

    // Execs
    let execs = doc.find_tags("exec");
    assert_eq!(execs.len(), 2);
    assert_eq!(execs[0].get_attribute("container"), Some("reporter"));
    assert_eq!(execs[1].get_attribute("container"), Some("mailer"));
    assert_eq!(
        execs[0].get_attribute("mount"),
        Some("shared-output:/output")
    );
}

#[test]
fn parse_attenuate_and_fork() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:attenuate container="sandbox">
  <hick:deny network="*" />
</hick:attenuate>

<hick:fork from="base" to="analyzer-a">
  <hick:deny network="*" />
</hick:fork>

<hick:confirm message="Proceed?" />
</hick:doc>"#;
    let doc = parse(src).unwrap();

    let attenuates = doc.find_tags("attenuate");
    assert_eq!(attenuates.len(), 1);
    assert_eq!(attenuates[0].get_attribute("container"), Some("sandbox"));

    let forks = doc.find_tags("fork");
    assert_eq!(forks.len(), 1);
    assert_eq!(forks[0].get_attribute("from"), Some("base"));
    assert_eq!(forks[0].get_attribute("to"), Some("analyzer-a"));

    let confirms = doc.find_tags("confirm");
    assert_eq!(confirms.len(), 1);
    assert_eq!(confirms[0].get_attribute("message"), Some("Proceed?"));
}

#[test]
fn parse_secret_tag() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="deploy" image="alpine">
  <hick:secret name="PULUMI_TOKEN" from="pulumi-token" />
</hick:container>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let containers = doc.find_tags("container");
    let secrets: Vec<_> = containers[0]
        .child_tags()
        .filter(|t| t.name == "secret")
        .collect();
    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].get_attribute("name"), Some("PULUMI_TOKEN"));
    assert_eq!(secrets[0].get_attribute("from"), Some("pulumi-token"));
}

#[test]
fn error_on_unclosed_tag() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo">
echo hello
</hick:doc>"#;
    let err = parse(src).unwrap_err();
    assert!(matches!(err, ParseError::UnexpectedClose { .. }));
}

// ---------------------------------------------------------------------
// Bare documents — docs/specs/freeform/bare-documents.md
// ---------------------------------------------------------------------

#[test]
fn a_rootless_file_is_a_bare_document() {
    // Was `error_on_missing_root`. A file with no `<hick:doc>` root is no
    // longer an error: it is the body of an implicit one.
    let src = "# Standup, Tuesday\n\nNothing blocking.\n";
    let doc = parse(src).unwrap();
    assert_eq!(doc.prefix, "hick");
    assert!(doc.frontmatter.is_none());
    assert_eq!(text_of(&doc.nodes), src);
}

#[test]
fn a_bare_document_recognises_tags() {
    let src = "# Notes\n\n<hick:copy id=\"n\">42</hick:copy>\n\nAfter.\n";
    let doc = parse(src).unwrap();
    let tags = doc.find_tags("copy");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].get_attribute("id"), Some("n"));
    // Prose on both sides of the tag survives byte-for-byte.
    assert_eq!(text_of(&doc.nodes), "# Notes\n\n42\n\nAfter.\n");
}

#[test]
fn an_empty_file_is_an_empty_bare_document() {
    let doc = parse("").unwrap();
    assert!(doc.nodes.is_empty());
    assert!(doc.weave_path.is_none());
}

#[test]
fn a_stray_closing_tag_in_a_bare_document_is_an_error() {
    // There is no root that could have opened it.
    let err = parse("# Notes\n\n</hick:copy>\n").unwrap_err();
    assert!(matches!(err, ParseError::UnexpectedClose { .. }));
}

#[test]
fn an_unclosed_tag_in_a_bare_document_still_errors() {
    let err = parse("# Notes\n\n<hick:copy id=\"n\">42\n").unwrap_err();
    assert!(matches!(err, ParseError::UnclosedTag { .. }));
}

/// Protects `docs/guarantees/authoring/a-syntax-error-names-the-invariant-it-hit.md`
///
/// Prose that quotes hick's own tag syntax — the exact situation a
/// tutorial documenting hick would hit — parses as a real tag, because
/// the no-escaping invariant applies everywhere. The three call sites
/// below are what a plausible piece of such prose actually reaches.
#[test]
fn a_malformed_tag_in_prose_names_the_no_escaping_invariant() {
    let cases = [
        "See `<hick:x =bad>` for details.",
        "Write `<hick:x y=unquoted>` like so.",
        "It looks like `<hick:x y=\"never closed",
    ];
    for src in cases {
        let err = parse(src).unwrap_err();
        let text = err.to_string();
        assert!(
            text.contains("hick has no escaping"),
            "error for {src:?} did not explain the invariant: {text}"
        );
        assert!(
            text.contains("bare-documents.md"),
            "error for {src:?} did not point at the fix: {text}"
        );
    }
}

#[test]
fn the_root_is_detected_from_the_prologue_not_the_file() {
    // Rule 3. Searching the whole file would find this `<hick:doc` and
    // parse it as the root, silently dropping every byte before it.
    let src = "Prose first.\n\n<hick:doc>inner</hick:doc>\n";
    let doc = parse(src).unwrap();
    assert!(
        matches!(&doc.nodes[0], HickNode::Text(t, _) if t.starts_with("Prose first.")),
        "leading prose must survive, got {:?}",
        doc.nodes[0]
    );
}

#[test]
fn a_wrapped_document_is_still_wrapped() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- a comment before the root -->
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
hello
</hick:doc>"#;
    let doc = parse(src).unwrap();
    assert_eq!(doc.weave_path, Some("out.md".to_string()));
    assert!(doc.frontmatter.is_none(), "frontmatter is bare-only");
}

// ---------------------------------------------------------------------
// Frontmatter
// ---------------------------------------------------------------------

#[test]
fn frontmatter_sets_reserved_keys() {
    let src = "---\nweave: notes/standup.md\nvolatile: true\n---\n\n# Standup\n";
    let doc = parse(src).unwrap();
    assert_eq!(doc.weave_path, Some("notes/standup.md".to_string()));
    assert!(doc.volatile);
}

#[test]
fn frontmatter_bytes_stay_in_the_node_stream() {
    // This is what makes the block weave through verbatim and keeps its
    // spans addressable, with nothing to reconstruct on the way back.
    let src = "---\nweave: out.md\n---\n\n# Standup\n";
    let doc = parse(src).unwrap();
    assert_eq!(text_of(&doc.nodes), src);
    assert_eq!(doc.frontmatter.unwrap().raw, "---\nweave: out.md\n---\n");
}

#[test]
fn frontmatter_metadata_is_preserved_and_never_interpreted() {
    let src = "---\ndate: 2026-08-18\nattendees: [nate, sam]\ntags:\n  - standup\n  - weekly\n---\n\n# Standup\n";
    let doc = parse(src).unwrap();
    let fm = doc.frontmatter.unwrap();
    assert_eq!(fm.get("date"), Some("2026-08-18"));
    assert_eq!(fm.get("attendees"), Some("[nate, sam]"));
    // A block value is recorded with an empty scalar, not parsed.
    assert_eq!(fm.get("tags"), Some(""));
    // Nothing here is reserved, so nothing here configures the document.
    assert!(doc.weave_path.is_none());
    assert!(!doc.volatile);
}

#[test]
fn frontmatter_lists_answer_the_same_in_both_spellings() {
    let block = parse("---\ntags:\n  - standup\n  - weekly\n---\n# T\n").unwrap();
    let inline = parse("---\ntags: [standup, weekly]\n---\n# T\n").unwrap();
    let expected = vec!["standup".to_string(), "weekly".to_string()];
    assert_eq!(block.frontmatter.unwrap().list("tags"), expected);
    assert_eq!(inline.frontmatter.unwrap().list("tags"), expected);
}

#[test]
fn a_scalar_or_absent_key_has_no_list_items() {
    let doc = parse("---\ndate: 2026-08-18\n---\n# T\n").unwrap();
    let fm = doc.frontmatter.unwrap();
    assert!(fm.list("date").is_empty());
    assert!(fm.list("nothing-here").is_empty());
}

#[test]
fn frontmatter_quotes_are_stripped() {
    let src = "---\nweave: \"a b.md\"\n---\n# T\n";
    let doc = parse(src).unwrap();
    assert_eq!(doc.weave_path, Some("a b.md".to_string()));
}

#[test]
fn frontmatter_can_rebind_the_prefix() {
    let src = "---\nprefix: h\n---\n\n# Notes\n\n<h:copy id=\"n\">42</h:copy>\n";
    let doc = parse(src).unwrap();
    assert_eq!(doc.prefix, "h");
    assert_eq!(doc.find_tags("copy").len(), 1);
}

#[test]
fn a_horizontal_rule_is_not_frontmatter() {
    // The accepted ambiguity, resolved toward content.
    let src = "---\n\nA note that opens with a rule.\n\n---\n";
    let doc = parse(src).unwrap();
    assert!(doc.frontmatter.is_none());
    assert_eq!(text_of(&doc.nodes), src);
}

#[test]
fn an_unterminated_fence_is_content() {
    let src = "---\nweave: out.md\n\n# Notes with no closing fence\n";
    let doc = parse(src).unwrap();
    assert!(doc.frontmatter.is_none());
    assert!(doc.weave_path.is_none());
    assert_eq!(text_of(&doc.nodes), src);
}

#[test]
fn a_non_mapping_fence_is_content() {
    let src = "---\njust some prose, not a mapping\n---\n";
    let doc = parse(src).unwrap();
    assert!(doc.frontmatter.is_none());
    assert_eq!(text_of(&doc.nodes), src);
}

#[test]
fn a_colon_without_a_space_is_not_a_mapping_entry() {
    // `https://example.com` must not read as key `https`.
    let src = "---\nhttps://example.com\n---\n";
    let doc = parse(src).unwrap();
    assert!(doc.frontmatter.is_none());
}

// ---------------------------------------------------------------------
// The weave default
// ---------------------------------------------------------------------

#[test]
fn a_document_weaves_a_markdown_file_of_its_own_name() {
    // The file name alone: outputs resolve relative to the document's own
    // directory, so this lands beside `notes/standup.md`. An absolute
    // path here would be refused as an escaping output path.
    let doc = parse_from_path("# Standup\n", Path::new("notes/standup.md")).unwrap();
    assert_eq!(doc.weave_path, Some("standup.md".to_string()));

    let absolute = parse_from_path("# Standup\n", Path::new("/tmp/x/standup.md")).unwrap();
    assert_eq!(absolute.weave_path, Some("standup.md".to_string()));
}

#[test]
fn an_explicit_weave_beats_the_default() {
    let src = "---\nweave: elsewhere.md\n---\n# T\n";
    let doc = parse_from_path(src, Path::new("notes/standup.md")).unwrap();
    assert_eq!(doc.weave_path, Some("elsewhere.md".to_string()));
}

#[test]
fn weave_none_opts_out_of_the_default() {
    let src = "---\nweave: none\n---\n# T\n";
    let doc = parse_from_path(src, Path::new("notes/standup.md")).unwrap();
    assert!(doc.weave_path.is_none());

    let wrapped = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
x
</hick:doc>"#;
    let doc = parse_from_path(wrapped, Path::new("g.md")).unwrap();
    assert!(doc.weave_path.is_none());
}

#[test]
fn parse_leaves_the_weave_path_unresolved() {
    // Analyses and language-server buffers must not invent an output file.
    let doc = parse("# Standup\n").unwrap();
    assert!(doc.weave_path.is_none());
}

#[test]
fn line_numbers_tracked() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<hick:exec container="demo">
echo hello
</hick:exec>

</hick:doc>"#;
    let doc = parse(src).unwrap();
    let execs = doc.find_tags("exec");
    assert_eq!(execs[0].source_line, 4);
}

#[test]
fn single_quoted_attributes() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:allow network='github.com:443' />
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let allows = doc.find_tags("allow");
    assert_eq!(allows[0].get_attribute("network"), Some("github.com:443"));
}

#[test]
fn multiple_execs_same_container() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
apk add curl
</hick:exec>
<hick:exec container="demo">
curl --version
</hick:exec>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let execs = doc.find_tags("exec");
    assert_eq!(execs.len(), 2);
    assert_eq!(execs[0].get_attribute("image"), Some("alpine"));
    assert!(execs[1].get_attribute("image").is_none());
}

// -----------------------------------------------------------------------
// Custom prefix tests
// -----------------------------------------------------------------------

#[test]
fn detect_prefix_from_xmlns() {
    assert_eq!(
        detect_prefix(r#"<h:doc xmlns:h="http://www.hickorydocs.com/1.0">"#),
        "h"
    );
    assert_eq!(
        detect_prefix(r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">"#),
        "hick"
    );
    assert_eq!(
        detect_prefix(r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">"#),
        "hick"
    );
    // Single-quoted
    assert_eq!(
        detect_prefix(r#"<x:doc xmlns:x='http://www.hickorydocs.com/1.0'>"#),
        "x"
    );
    // No xmlns -> fallback
    assert_eq!(detect_prefix(r#"<hick:doc>"#), "hick");
}

#[test]
fn parse_with_short_prefix() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:file path="out.txt">hello world</h:file>
</h:doc>"#;
    let doc = parse(src).unwrap();
    assert_eq!(doc.prefix, "h");
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].get_attribute("path"), Some("out.txt"));
    assert_eq!(files[0].text_content(), "hello world");
}

#[test]
fn custom_prefix_ignores_hick_in_text() {
    // Using prefix "h", so <hick:...> in text is NOT parsed as a tag
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:file path="guide.md">
Here is an example hick file:

<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
  <hick:file path="out.txt">hello</hick:file>
</hick:doc>

The above produces a file called out.txt.
</h:file>
</h:doc>"#;
    let doc = parse(src).unwrap();
    assert_eq!(doc.prefix, "h");

    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);

    // The <hick:...> example text should be raw text content, not parsed tags
    let content = files[0].text_content();
    assert!(
        content.contains("<hick:doc"),
        "expected literal <hick:doc in text, got: {content}"
    );
    assert!(
        content.contains("<hick:file"),
        "expected literal <hick:file in text, got: {content}"
    );
    assert!(
        content.contains("</hick:file>"),
        "expected literal </hick:file> in text, got: {content}"
    );
}

#[test]
fn custom_prefix_nested_tags() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:container name="demo" image="alpine">
  <h:allow network="github.com:443" />
  <h:deny network="*" />
  <h:secret name="TOKEN" from="my-token" />
</h:container>
<h:exec container="demo">
echo hello
</h:exec>
</h:doc>"#;
    let doc = parse(src).unwrap();
    let containers = doc.find_tags("container");
    assert_eq!(containers.len(), 1);
    assert_eq!(containers[0].get_attribute("name"), Some("demo"));

    let child_tags: Vec<_> = containers[0].child_tags().collect();
    assert_eq!(child_tags.len(), 3);
    assert_eq!(child_tags[0].name, "allow");
    assert_eq!(child_tags[1].name, "deny");
    assert_eq!(child_tags[2].name, "secret");

    let execs = doc.find_tags("exec");
    assert_eq!(execs.len(), 1);
}

#[test]
fn custom_prefix_copy_paste() {
    let src = r##"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:copy id="ver">1.0</h:copy>
<h:file path="out.txt"><h:paste select="#ver" /></h:file>
</h:doc>"##;
    let doc = parse(src).unwrap();
    let copies = doc.find_tags("copy");
    assert_eq!(copies.len(), 1);
    assert_eq!(copies[0].text_content(), "1.0");

    let files = doc.find_tags("file");
    let pastes: Vec<_> = files[0]
        .child_tags()
        .filter(|t| t.name == "paste")
        .collect();
    assert_eq!(pastes.len(), 1);
    assert_eq!(pastes[0].get_attribute("select"), Some("#ver"));
}

#[test]
fn custom_prefix_error_on_unclosed() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:exec container="demo">
echo hello
</h:doc>"#;
    let err = parse(src).unwrap_err();
    assert!(matches!(err, ParseError::UnexpectedClose { .. }));
}

// Source column tracking tests
#[test]
fn source_column_at_line_start() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="test.txt">content</hick:file>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].source_column, 0);
}

#[test]
fn source_column_indented() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
    <hick:file path="test.txt">content</hick:file>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].source_column, 4);
}

#[test]
fn source_column_nested() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
    <hick:when test="true">
        <hick:file path="test.txt">content</hick:file>
    </hick:when>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    // find_tags is non-recursive, so we need to search inside the when tag
    let whens = doc.find_tags("when");
    assert_eq!(whens.len(), 1);
    let files: Vec<_> = whens[0].child_tags().filter(|t| t.name == "file").collect();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].source_column, 8);
}

// Dedent tests
#[test]
fn dedent_no_indent() {
    let text = "line1\nline2\nline3";
    assert_eq!(dedent(text, 4), "line1\nline2\nline3");
}

#[test]
fn dedent_uniform_indent() {
    let text = "    line1\n    line2\n    line3";
    assert_eq!(dedent(text, 4), "line1\nline2\nline3");
}

#[test]
fn dedent_partial_strip() {
    let text = "    line1\n        line2\n    line3";
    assert_eq!(dedent(text, 4), "line1\n    line2\nline3");
}

#[test]
fn dedent_leading_newline() {
    let text = "\n    def hello():\n        print('hi')";
    assert_eq!(dedent(text, 4), "def hello():\n    print('hi')");
}

#[test]
fn dedent_trailing_newline() {
    let text = "    line1\n    line2\n";
    assert_eq!(dedent(text, 4), "line1\nline2\n");
}

#[test]
fn dedent_zero_indent() {
    let text = "    line1\n    line2";
    assert_eq!(dedent(text, 0), "    line1\n    line2");
}

#[test]
fn dedent_more_than_available() {
    let text = "  line1\n  line2";
    assert_eq!(dedent(text, 8), "line1\nline2");
}

// Source span tests
#[test]
fn text_nodes_have_spans() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
hello world
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let text_nodes: Vec<_> = doc
        .nodes
        .iter()
        .filter_map(|n| match n {
            HickNode::Text(t, span) => Some((t.as_str(), *span)),
            _ => None,
        })
        .collect();
    assert!(!text_nodes.is_empty(), "should have at least one text node");
    let (text, span) = &text_nodes[0];
    assert!(text.contains("hello world"));
    let span = span.expect("text node should have a span");
    assert!(span.start < span.end, "span should be non-empty");
    // Verify the span references the correct bytes in the source
    assert_eq!(&src[span.start..span.end], *text);
}

#[test]
fn tag_has_source_span() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="test.txt">content</hick:file>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    let span = files[0].source_span.expect("tag should have a source span");
    // The span should cover from '<' to '>' of the opening tag
    let tag_text = &src[span.start..span.end];
    assert!(
        tag_text.starts_with("<hick:file"),
        "span should start with opening tag, got: {tag_text}"
    );
    assert!(
        tag_text.ends_with(">"),
        "span should end with >, got: {tag_text}"
    );
}

#[test]
fn self_closing_tag_span() {
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:allow network="github.com:443" />
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let allows = doc.find_tags("allow");
    let span = allows[0]
        .source_span
        .expect("self-closing tag should have span");
    let tag_text = &src[span.start..span.end];
    assert!(tag_text.starts_with("<hick:allow"));
    assert!(tag_text.ends_with("/>"));
}

#[test]
fn comment_is_skipped() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<!-- this is a comment -->
<hick:file path="a.rs">hello</hick:file>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].get_attribute("path"), Some("a.rs"));
}

#[test]
fn comment_out_hick_file_block() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="keep.rs">kept</hick:file>
<!--
<hick:file path="removed.rs">this should not appear</hick:file>
-->
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].get_attribute("path"), Some("keep.rs"));
}

#[test]
fn multiple_comments() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<!-- first -->
<hick:file path="a.rs">a</hick:file>
<!-- second -->
<!-- third -->
<hick:file path="b.rs">b</hick:file>
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 2);
}

#[test]
fn unclosed_comment_error() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<!-- unclosed comment
<hick:file path="a.rs">a</hick:file>
</hick:doc>"#;
    let err = parse(src).unwrap_err();
    assert!(
        matches!(err, ParseError::UnclosedComment { line: 2 }),
        "expected UnclosedComment at line 2, got: {err:?}"
    );
}

#[test]
fn comment_preserves_surrounding_text() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
before<!-- comment -->after
</hick:doc>"#;
    let doc = parse(src).unwrap();
    let texts: Vec<&str> = doc
        .nodes
        .iter()
        .filter_map(|n| match n {
            HickNode::Text(t, _) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    let combined = texts.join("");
    assert!(
        combined.contains("before"),
        "missing 'before': {combined:?}"
    );
    assert!(combined.contains("after"), "missing 'after': {combined:?}");
    assert!(
        !combined.contains("comment"),
        "comment leaked: {combined:?}"
    );
}
