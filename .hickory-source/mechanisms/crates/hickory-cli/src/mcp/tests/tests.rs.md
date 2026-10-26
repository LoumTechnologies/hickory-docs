<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/mcp/tests/tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7ac04de70076"></a>
## `crates/hickory-cli/src/mcp/tests/tests.rs`

<a id="f-7ac04de70076-every_advertised_tool_has_a_schema_and_a_description"></a>
### `every_advertised_tool_has_a_schema_and_a_description`

<storycode:copy id="f-7ac04de70076-every_advertised_tool_has_a_schema_and_a_description">
use super::*;

#[test]
fn every_advertised_tool_has_a_schema_and_a_description() {
    let catalogue = tool_catalogue();
    let tools = catalogue["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    // The six the built-in agent has, plus the two an external agent
    // needs and the built-in one does not: it is started ON a document,
    // while an MCP client arrives at a folder and has to find out what
    // is in it. A surface that offers fewer makes bringing your own
    // agent the lesser path.
    assert_eq!(
        names,
        vec![
            "read_doc",
            "read_output",
            "read_file",
            "edit_output",
            "edit_doc",
            "verify",
            // Finding and starting documents: about the folder, not one
            // document. Without these an agent had to leave the tool
            // surface to learn what existed or to begin anything.
            "list_docs",
            "create_doc",
            // Project-wide search: about the folder, not one document,
            // and it can never write anything.
            "search",
            // Debugging: a session an agent drives, which writes
            // nothing.
            "debug_start",
            "debug_state",
            "debug_eval",
            "debug_step",
            "debug_stop",
            "record_assumption",
            "record_decision",
            "record_guarantee_review",
            "audit_assumptions"
        ]
    );
    for tool in tools {
        assert!(
            tool["description"].as_str().is_some_and(|d| d.len() > 40),
            "{} needs a description that teaches the doctrine",
            tool["name"]
        );
        assert_eq!(tool["inputSchema"]["type"], "object");
    }
}

</storycode:copy>
<a id="f-7ac04de70076-mcp_arguments_translate_into_the_tool_vocabulary"></a>
### `mcp_arguments_translate_into_the_tool_vocabulary`

<storycode:copy id="f-7ac04de70076-mcp_arguments_translate_into_the_tool_vocabulary">
#[test]
fn mcp_arguments_translate_into_the_tool_vocabulary() {
    let inv = Server::invocation(
        "edit_output",
        &json!({
            "doc": "a.md",
            "path": "gen.rs",
            "run": "a1b2..c3d4",
            "occurrence": 2,
            "input": "fn main() {}"
        }),
    )
    .unwrap();
    assert_eq!(inv.name, "edit_output");
    assert_eq!(inv.arg("path"), Some("gen.rs"));
    assert_eq!(inv.arg("run"), Some("a1b2..c3d4"));
    assert_eq!(inv.arg("occurrence"), Some("2"));
    assert_eq!(inv.input.as_deref(), Some("fn main() {}"));
    // `doc` selects WHICH document at this layer; it must not leak into
    // the tool arguments, where it means "an upstream document".
    assert_eq!(inv.arg("doc"), None);
}

</storycode:copy>
<a id="f-7ac04de70076-upstream_becomes_the_tools_doc_argument"></a>
### `upstream_becomes_the_tools_doc_argument`

<storycode:copy id="f-7ac04de70076-upstream_becomes_the_tools_doc_argument">
#[test]
fn upstream_becomes_the_tools_doc_argument() {
    let inv = Server::invocation(
        "edit_doc",
        &json!({ "doc": "primary.md", "upstream": "requirements.md", "run": "aaaa" }),
    )
    .unwrap();
    assert_eq!(inv.arg("doc"), Some("requirements.md"));
}

</storycode:copy>
<a id="f-7ac04de70076-with_lineage_is_only_sent_when_asked_for"></a>
### `with_lineage_is_only_sent_when_asked_for`

<storycode:copy id="f-7ac04de70076-with_lineage_is_only_sent_when_asked_for">
#[test]
fn with_lineage_is_only_sent_when_asked_for() {
    let on = Server::invocation(
        "read_output",
        &json!({ "path": "g.rs", "with_lineage": true }),
    )
    .unwrap();
    assert_eq!(on.arg("with_lineage"), Some("true"));
    let off = Server::invocation(
        "read_output",
        &json!({ "path": "g.rs", "with_lineage": false }),
    )
    .unwrap();
    assert_eq!(off.arg("with_lineage"), None);
}

</storycode:copy>
<a id="f-7ac04de70076-a_refusal_is_an_error_result_that_still_carries_its_text"></a>
### `a_refusal_is_an_error_result_that_still_carries_its_text`

<storycode:copy id="f-7ac04de70076-a_refusal_is_an_error_result_that_still_carries_its_text">
#[test]
fn a_refusal_is_an_error_result_that_still_carries_its_text() {
    let result = text_result("anchor run=\"4f20\" matches no lines", true);
    assert_eq!(result["isError"], true);
    assert_eq!(
        result["content"][0]["text"],
        "anchor run=\"4f20\" matches no lines"
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/mcp/tests/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/mcp/tests/tests.rs"><storycode:paste select="#f-7ac04de70076-every_advertised_tool_has_a_schema_and_a_description"/><storycode:paste select="#f-7ac04de70076-mcp_arguments_translate_into_the_tool_vocabulary"/><storycode:paste select="#f-7ac04de70076-upstream_becomes_the_tools_doc_argument"/><storycode:paste select="#f-7ac04de70076-with_lineage_is_only_sent_when_asked_for"/><storycode:paste select="#f-7ac04de70076-a_refusal_is_an_error_result_that_still_carries_its_text"/></storycode:file>

</storycode:doc>
