<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/byo_agent_surface.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-43da4a5da7c9"></a>
## `crates/hickory-cli/tests/byo_agent_surface.rs`

Related contracts: [Use the same document tools from an external coding agent](../../../../guarantees/agent/byo-agent-tool-surface/contract.md), [Finish an agent turn against its captured project boundary](../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md).

[Read the test `hick` beside its requirement](../../../../guarantees/agent/byo-agent-tool-surface/contract.md#f-43da4a5da7c9-hick).
<a id="f-43da4a5da7c9-DOC"></a>
### `DOC`

<storycode:copy id="f-43da4a5da7c9-DOC">
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

</storycode:copy>
<a id="f-43da4a5da7c9-write_doc"></a>
### `write_doc`

<storycode:copy id="f-43da4a5da7c9-write_doc">
fn write_doc(dir: &Path) -> PathBuf {
    let path = dir.join("demo.md");
    std::fs::write(&path, DOC).unwrap();
    path
}

</storycode:copy>
<a id="f-43da4a5da7c9-doc_cmd"></a>
### `doc_cmd`

Run a `hick doc` command with optional stdin. Returns (code, stdout).

<storycode:copy id="f-43da4a5da7c9-doc_cmd">
/// Run a `hick doc` command with optional stdin. Returns (code, stdout).
fn doc_cmd(args: &[&str], stdin: Option<&str>, session: Option<&Path>) -> (i32, String) {
    let mut cmd = hick();
    cmd.arg("doc")
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(path) = session {
        cmd.env("HICKORY_SESSION", path);
    }
    let mut child = cmd.spawn().unwrap();
    if let Some(text) = stdin {
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
    }
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
    )
}

</storycode:copy>
<a id="f-43da4a5da7c9-hash_of"></a>
### `hash_of`

The SHA-256 hash prefixing the first line that contains `needle`.

<storycode:copy id="f-43da4a5da7c9-hash_of">
/// The SHA-256 hash prefixing the first line that contains `needle`.
fn hash_of(hashlines: &str, needle: &str) -> String {
    hashlines
        .lines()
        .find(|l| l.contains(needle) && l.len() > 65 && l.as_bytes()[64] == b'|')
        .unwrap_or_else(|| panic!("no hashline containing {needle:?} in:\n{hashlines}"))
        .split('|')
        .next()
        .unwrap()
        .to_string()
}

</storycode:copy>
<a id="f-43da4a5da7c9-an_outside_agent_can_read_edit_and_verify_through_the_command_surface"></a>
### `an_outside_agent_can_read_edit_and_verify_through_the_command_surface`

<storycode:copy id="f-43da4a5da7c9-an_outside_agent_can_read_edit_and_verify_through_the_command_surface">
#[test]
fn an_outside_agent_can_read_edit_and_verify_through_the_command_surface() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    let doc = doc.to_str().unwrap();

    // 1. Read the output with lineage — which is how an agent learns what it
    //    is allowed to edit through the generated file.
    let (code, out) = doc_cmd(
        &["read-output", doc, "--path", "greet.rs", "--lineage"],
        None,
        None,
    );
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("editable via edit_output"), "{out}");
    let anchor = hash_of(&out, "fn greet()");

    // 2. Edit CODE through the output. The document — not just the generated
    //    file — must carry the change, byte-exactly.
    let (code, out) = doc_cmd(
        &["edit-output", doc, "--path", "greet.rs", "--run", &anchor],
        Some("fn greet() { println!(\"hello, world\"); }"),
        None,
    );
    assert_eq!(code, 0, "{out}");
    let source = std::fs::read_to_string(doc).unwrap();
    assert!(
        source.contains(r#"fn greet() { println!("hello, world"); }"#),
        "the edit did not reach the document source:\n{source}"
    );

    // 3. The same anchor a second time must NOT apply. This is the property
    //    that makes the per-command surface safe: an anchor is a hash of the
    //    content it names, so an edit built against text that has since
    //    changed cannot land in the wrong place.
    let (code, out) = doc_cmd(
        &["edit-output", doc, "--path", "greet.rs", "--run", &anchor],
        Some("this must never be applied"),
        None,
    );
    assert_eq!(code, 1, "a stale anchor must fail: {out}");
    assert!(
        out.contains("re-read"),
        "the refusal must say what to do: {out}"
    );
    let source = std::fs::read_to_string(doc).unwrap();
    assert!(
        !source.contains("this must never be applied"),
        "a refused edit still changed the document:\n{source}"
    );

    // 4. Verify executes for real and writes the outputs.
    let (code, out) = doc_cmd(&["verify", doc, "--json"], None, None);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("\"ok\":true"), "{out}");
    let generated = std::fs::read_to_string(dir.path().join("greet.rs")).unwrap();
    assert!(generated.contains("hello, world"), "{generated}");
}

</storycode:copy>
<a id="f-43da4a5da7c9-work_done_across_separate_processes_lands_in_one_replayable_session"></a>
### `work_done_across_separate_processes_lands_in_one_replayable_session`

<storycode:copy id="f-43da4a5da7c9-work_done_across_separate_processes_lands_in_one_replayable_session">
#[test]
fn work_done_across_separate_processes_lands_in_one_replayable_session() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    let doc = doc.to_str().unwrap();
    let session = dir.path().join("sessions/byo.md");

    let (_, out) = doc_cmd(&["read", doc], None, Some(&session));
    let anchor = hash_of(&out, "hick:copy");
    let (code, out) = doc_cmd(
        &["edit", doc, "--run", &anchor],
        Some("<hick:copy id=\"greet\">fn greet() { println!(\"recorded\"); }"),
        Some(&session),
    );
    assert_eq!(code, 0, "{out}");

    let recorded = std::fs::read_to_string(&session).unwrap();
    // Every process appended rather than truncating: both calls are there.
    assert!(
        recorded.contains(r#"<hick:tool name="read_doc">"#),
        "{recorded}"
    );
    assert!(
        recorded.contains(r#"<hick:tool name="edit_doc">"#),
        "{recorded}"
    );
    assert!(
        recorded.matches("<hick:tool-result").count() == 2,
        "expected both results:\n{recorded}"
    );
    // …and the file is a closed, parseable session after EVERY command, not
    // only once the agent decides to stop.
    assert!(
        recorded.trim_end().ends_with("</hick:session>"),
        "{recorded}"
    );
    hick_lang::parse_session(&recorded).expect("the recorded session must parse");
}

</storycode:copy>
<a id="f-43da4a5da7c9-a_command_that_names_no_document_explains_itself"></a>
### `a_command_that_names_no_document_explains_itself`

<storycode:copy id="f-43da4a5da7c9-a_command_that_names_no_document_explains_itself">
#[test]
fn a_command_that_names_no_document_explains_itself() {
    let dir = tempfile::tempdir().unwrap();
    // Two documents: guessing between them could put an edit in the wrong
    // file, so the command must refuse and say how to disambiguate.
    std::fs::write(dir.path().join("a.md"), DOC).unwrap();
    std::fs::write(dir.path().join("b.md"), DOC).unwrap();
    let out = hick()
        .arg("doc")
        .arg("read")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("exactly one .md"), "{err}");
    assert!(err.contains("hick doc"), "{err}");
}

</storycode:copy>
<a id="f-43da4a5da7c9-mcp_exchange"></a>
### `mcp_exchange`

MCP: the same tools, over the protocol every major coding agent speaks.
Feed newline-delimited JSON-RPC to `hick mcp` and collect the replies.

<storycode:copy id="f-43da4a5da7c9-mcp_exchange">
// ---------------------------------------------------------------------------
// MCP: the same tools, over the protocol every major coding agent speaks.
// ---------------------------------------------------------------------------

/// Feed newline-delimited JSON-RPC to `hick mcp` and collect the replies.
fn mcp_exchange(doc: &Path, requests: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut child = hick()
        .arg("mcp")
        .arg("--doc")
        .arg(doc)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        for req in requests {
            writeln!(stdin, "{req}").unwrap();
        }
    }
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("every line on stdout must be JSON-RPC"))
        .collect()
}

</storycode:copy>
<a id="f-43da4a5da7c9-the_mcp_server_speaks_the_protocol_and_keeps_one_session_open"></a>
### `the_mcp_server_speaks_the_protocol_and_keeps_one_session_open`

<storycode:copy id="f-43da4a5da7c9-the_mcp_server_speaks_the_protocol_and_keeps_one_session_open">
#[test]
fn the_mcp_server_speaks_the_protocol_and_keeps_one_session_open() {
    // Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
    use serde_json::json;

    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());

    let anchor = hickory_agent::hashline::line_hash("fn greet() { println!(\"hello\"); }");
    let replies = mcp_exchange(
        &doc,
        &[
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}),
            // A notification carries no id and must draw no response at all.
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
            json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
                "name":"edit_output",
                "arguments":{"path":"greet.rs","run":anchor,"input":"fn greet() { println!(\"via mcp\"); }"}
            }}),
            // The SAME anchor again. In-process the session has already
            // re-woven, so this is stale — no re-read, no second open.
            json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{
                "name":"edit_output",
                "arguments":{"path":"greet.rs","run":anchor,"input":"must not apply"}
            }}),
            json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"verify","arguments":{}}}),
        ],
    );

    // Five requests, five replies: the notification was not answered.
    assert_eq!(replies.len(), 5, "{replies:#?}");
    assert_eq!(replies[0]["result"]["serverInfo"]["name"], "hick");
    let names: Vec<&str> = replies[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    // The document tools, then the two project-level ones, then the
    // debugger. `list_docs` and `create_doc` are here and not in the
    // built-in agent's set because the two arrive differently: the built-in
    // agent is started ON a document, while an MCP client is pointed at a
    // folder and has to find out what is in it. An agent that can only read
    // a failing document is guessing, and the debugger is what lets it stop
    // at the failure and ask.
    assert_eq!(
        names,
        vec![
            "read_doc",
            "read_output",
            "read_file",
            "edit_output",
            "edit_doc",
            "verify",
            "list_docs",
            "create_doc",
            "search",
            "debug_start",
            "debug_state",
            "debug_eval",
            "debug_step",
            "debug_stop",
            "record_assumption",
            "record_decision",
            "record_guarantee_review",
            "audit_assumptions",
            "begin_changeset",
            "changeset_status",
            "submit_changeset",
        ]
    );

    assert_eq!(replies[2]["result"]["isError"], false, "{:#?}", replies[2]);
    assert_eq!(
        replies[3]["result"]["isError"], true,
        "the reused session must know the anchor is stale: {:#?}",
        replies[3]
    );
    assert_eq!(replies[4]["result"]["isError"], false, "{:#?}", replies[4]);

    let source = std::fs::read_to_string(&doc).unwrap();
    assert!(source.contains("via mcp"), "{source}");
    assert!(!source.contains("must not apply"), "{source}");
}

</storycode:copy>
<a id="f-43da4a5da7c9-an_unknown_mcp_method_is_a_jsonrpc_error_not_a_dead_connection"></a>
### `an_unknown_mcp_method_is_a_jsonrpc_error_not_a_dead_connection`

<storycode:copy id="f-43da4a5da7c9-an_unknown_mcp_method_is_a_jsonrpc_error_not_a_dead_connection">
#[test]
fn an_unknown_mcp_method_is_a_jsonrpc_error_not_a_dead_connection() {
    use serde_json::json;

    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    let replies = mcp_exchange(
        &doc,
        &[
            json!({"jsonrpc":"2.0","id":1,"method":"resources/list"}),
            // The server must still be serving after the error.
            json!({"jsonrpc":"2.0","id":2,"method":"ping"}),
        ],
    );
    assert_eq!(replies.len(), 2, "{replies:#?}");
    assert_eq!(replies[0]["error"]["code"], -32601);
    assert!(
        replies[0]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("tools/call"),
        "the error must name what this server does implement: {:#?}",
        replies[0]
    );
    assert!(replies[1]["result"].is_object());
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/byo_agent_surface.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/byo_agent_surface.rs"><storycode:paste select="#f-43da4a5da7c9-hick"/><storycode:paste select="#f-43da4a5da7c9-DOC"/><storycode:paste select="#f-43da4a5da7c9-write_doc"/><storycode:paste select="#f-43da4a5da7c9-doc_cmd"/><storycode:paste select="#f-43da4a5da7c9-hash_of"/><storycode:paste select="#f-43da4a5da7c9-an_outside_agent_can_read_edit_and_verify_through_the_command_surface"/><storycode:paste select="#f-43da4a5da7c9-work_done_across_separate_processes_lands_in_one_replayable_session"/><storycode:paste select="#f-43da4a5da7c9-a_command_that_names_no_document_explains_itself"/><storycode:paste select="#f-43da4a5da7c9-mcp_exchange"/><storycode:paste select="#f-43da4a5da7c9-the_mcp_server_speaks_the_protocol_and_keeps_one_session_open"/><storycode:paste select="#f-43da4a5da7c9-an_unknown_mcp_method_is_a_jsonrpc_error_not_a_dead_connection"/></storycode:file>

</storycode:doc>
