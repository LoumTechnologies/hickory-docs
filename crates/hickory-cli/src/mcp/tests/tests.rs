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

#[test]
fn upstream_becomes_the_tools_doc_argument() {
    let inv = Server::invocation(
        "edit_doc",
        &json!({ "doc": "primary.md", "upstream": "requirements.md", "run": "aaaa" }),
    )
    .unwrap();
    assert_eq!(inv.arg("doc"), Some("requirements.md"));
}

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

#[tokio::test]
async fn read_file_works_without_a_weavable_document_and_keeps_its_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap().join("workspace");
    std::fs::create_dir(&root).unwrap();
    let broken = root.join("project.md");
    std::fs::write(&broken, "<hick:doc><hick:file").unwrap();
    std::fs::write(root.join("AGENTS.md"), "first\nsecond\n").unwrap();
    std::fs::write(dir.path().join("outside.txt"), "outside").unwrap();
    let executor = ExecutorChoice::Local.build_for(Some(&root)).await.unwrap();
    let mut server = Server::embedded(root.clone(), broken, executor, root.join("session.md"));
    let args = json!({ "path": "AGENTS.md", "from": 2, "to": 2 });
    for selected in [true, false] {
        if !selected {
            server.default_doc = None;
        }
        let result = server.call_tool("read_file", &args).await;
        assert_ne!(result["isError"], true, "{result}");
        assert!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("second")
        );
        let reads = server.take_context_reads();
        assert_eq!(reads.len(), 1);
        assert_eq!(
            reads[0].source_path.as_deref(),
            Some(root.join("AGENTS.md").as_path())
        );
        assert_eq!(reads[0].byte_span, (6, 13));
        assert_eq!(reads[0].content, "second\n");
        assert!(server.sessions.is_empty());
    }
    let result = server
        .call_tool("read_file", &json!({ "path": "../outside.txt" }))
        .await;
    assert_eq!(result["isError"], true);
    assert!(server.take_context_reads().is_empty());
}

#[test]
fn a_refusal_is_an_error_result_that_still_carries_its_text() {
    let result = text_result("anchor run=\"4f20\" matches no lines", true);
    assert_eq!(result["isError"], true);
    assert_eq!(
        result["content"][0]["text"],
        "anchor run=\"4f20\" matches no lines"
    );
}
