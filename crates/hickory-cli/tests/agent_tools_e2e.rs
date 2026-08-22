//! End-to-end proof of the document edit tool set (no network):
//!
//! a canned LLM drives `<hick:next>tool</hick:next>` turns — read_output
//! with lineage, an edit_output that lands through lineage, an edit on a
//! duplicated-paste range that gets the ROUTED refusal, the edit_doc that
//! follows the pointer, and verify — then the session file parses as a
//! `hick:session` document and replays through `hick-literate`.

use std::sync::Arc;

use hickory_agent::hashline::line_hash;
use hickory_agent::{AgentConfig, AgentEvent, ScriptedLlmClient, run_agent};
use hickory_executor::{Executor, LocalExecutor};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="greet">fn greet() {
    println!("hello");
}
</hick:copy>
<hick:copy id="dup">const SHARED: u8 = 1;
</hick:copy>
<hick:file path="gen.rs">// header
<hick:paste select="#greet" />// mid
<hick:paste select="#dup" />// tail
<hick:paste select="#dup" /></hick:file>
<hick:container name="c" image="alpine" />
<hick:exec container="c">
echo hi
<hick:expect match="exact">hi
</hick:expect>
</hick:exec>
</hick:doc>
"##;

fn tool_turn(thought: &str, name: &str, args: &[(&str, &str)], input: Option<&str>) -> String {
    let mut xml = format!("<hick:next>tool</hick:next>\n{thought}\n<hick:tool name=\"{name}\">\n");
    for (k, v) in args {
        xml.push_str(&format!("<hick:arg name=\"{k}\">{v}</hick:arg>\n"));
    }
    if let Some(input) = input {
        xml.push_str(&format!("<hick:input>\n{input}\n</hick:input>\n"));
    }
    xml.push_str("</hick:tool>");
    xml
}

#[tokio::test(flavor = "multi_thread")]
async fn agent_tool_session_edits_verifies_and_replays() {
    let project = tempfile::tempdir().unwrap();
    let project_dir = project.path();
    let doc_path = project_dir.join("doc.hick");
    std::fs::write(&doc_path, DOC).unwrap();

    let hello = line_hash("    println!(\"hello\");");
    let dup = line_hash("const SHARED: u8 = 1;");
    // The whole dup copy element in the document (open line through close
    // line): payloads must keep hick tags balanced, so the edit_doc turn
    // replaces the full element.
    let doc_dup_run = format!(
        "{}..{}",
        line_hash(r#"<hick:copy id="dup">const SHARED: u8 = 1;"#),
        line_hash("</hick:copy>")
    );

    let turns = vec![
        // (a) read the output with lineage.
        tool_turn(
            "Reading both surfaces first.",
            "read_output",
            &[("path", "gen.rs"), ("with_lineage", "true")],
            None,
        ),
        // (b) edit a fragment-owned range through the output.
        tool_turn(
            "Code change goes through the output.",
            "edit_output",
            &[("path", "gen.rs"), ("run", &hello)],
            Some("    println!(\"hi there\");"),
        ),
        // (c) attempt an edit on a duplicated-paste range → routed refusal.
        tool_turn(
            "Trying the shared constant in place.",
            "edit_output",
            &[("path", "gen.rs"), ("run", &dup), ("occurrence", "2")],
            Some("const SHARED: u8 = 2;"),
        ),
        // (d) follow the pointer: fix the source block with edit_doc.
        tool_turn(
            "Following the routing pointer to the document.",
            "edit_doc",
            &[("run", &doc_dup_run)],
            Some("<hick:copy id=\"dup\">const SHARED: u8 = 2;\n</hick:copy>"),
        ),
        // A script turn so the session replay has something to execute.
        "<hick:next>code</hick:next>\nLeaving a replayable breadcrumb.\n```sh\necho replay-me\n```"
            .to_string(),
        // (e) verify before finishing.
        tool_turn("Verifying.", "verify", &[], None),
        "<hick:next>done</hick:next>\nUpdated the greeting and the shared constant; verify passed."
            .to_string(),
    ];

    let llm = ScriptedLlmClient::new(turns);
    let executor: Arc<dyn Executor> = Arc::new(LocalExecutor::new().unwrap());
    let mut config = AgentConfig::new("update the generated code", project_dir);
    config.doc_path = Some(doc_path.clone());

    let mut events = Vec::new();
    let outcome = run_agent(&llm, executor.clone(), &config, &mut |e: AgentEvent| {
        events.push(e);
    })
    .await
    .expect("agent run failed");
    assert_eq!(outcome.turns, 7);

    // (b) landed: doc updated through lineage, byte-exactly.
    let doc_source = std::fs::read_to_string(&doc_path).unwrap();
    assert!(doc_source.contains("println!(\"hi there\");"));
    // (c) was refused with routing, and (d) then fixed the source block.
    let refusal = events
        .iter()
        .find_map(|e| match e {
            AgentEvent::ToolFinished {
                name,
                ok: false,
                text,
            } if name == "edit_output" => Some(text.clone()),
            _ => None,
        })
        .expect("the duplicated-paste edit must be refused");
    assert!(refusal.contains("routing, not failure"), "{refusal}");
    assert!(refusal.contains("edit_doc"), "{refusal}");
    assert!(refusal.contains("doc.hick"), "{refusal}");
    assert!(doc_source.contains("const SHARED: u8 = 2;"));

    // (e) verify passed and wrote the outputs.
    let verify_text = events
        .iter()
        .find_map(|e| match e {
            AgentEvent::ToolFinished {
                name,
                ok: true,
                text,
            } if name == "verify" => Some(text.clone()),
            _ => None,
        })
        .expect("verify must succeed");
    assert!(verify_text.starts_with("PASS"), "{verify_text}");
    let generated = std::fs::read_to_string(project_dir.join("gen.rs")).unwrap();
    assert!(generated.contains("println!(\"hi there\");"));
    assert_eq!(generated.matches("const SHARED: u8 = 2;").count(), 2);

    // The session file parses as a SessionDocument: tool elements ride along
    // as unknown (gracefully skipped) tags, the script action survives.
    let session_source = std::fs::read_to_string(&outcome.session_path).unwrap();
    assert!(hick_lang::is_session_source(&session_source));
    let session = hick_lang::parse_session(&session_source).expect("session must parse");
    assert!(matches!(
        session.nodes[0],
        hick_lang::SessionNode::User { .. }
    ));
    assert!(
        session.nodes.iter().any(|n| matches!(
            n,
            hick_lang::SessionNode::Assistant { actions, .. } if !actions.is_empty()
        )),
        "the script action must survive the session parse"
    );
    assert!(session_source.contains("<hick:tool name=\"edit_output\">"));
    assert!(
        session_source.contains("name=\"verify\" ok=\"true\">"),
        "{session_source}"
    );

    // And it REPLAYS: `hick run session.hick` semantics via hick-literate.
    hick_literate::run_pipeline_cmd(hick_literate::PipelineRunOpts {
        files: vec![outcome.session_path.clone()],
        config_path: None,
        key_file: None,
        secrets_dir: None,
        params: vec![],
        features: None,
        output_dir: None,
        dry_run: false,
        cache: false,
        freeze: false,
        clear_cache: false,
        verbose: false,
    })
    .await
    .expect("session replay must succeed");
}
