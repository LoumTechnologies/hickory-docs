//! The bring-your-own-agent surface, driven exactly the way an outside coding
//! agent drives it: by running the shipped binary.
//!
//! Protects docs/guarantees/agent/byo-agent-tool-surface.md. The claim under
//! test is not "the commands exist" but "an agent that has never seen our
//! ReAct loop gets the same guarantees from them" — hashline anchors, an edit
//! through an output mapped back byte-exactly, a stale anchor refused rather
//! than misapplied, and a replayable `hick:session` written across separate
//! processes.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

fn write_doc(dir: &Path) -> PathBuf {
    let path = dir.join("demo.hick");
    std::fs::write(&path, DOC).unwrap();
    path
}

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

/// The 4-hex hash prefixing the first line that contains `needle`.
fn hash_of(hashlines: &str, needle: &str) -> String {
    hashlines
        .lines()
        .find(|l| l.contains(needle) && l.len() > 5 && l.as_bytes()[4] == b'|')
        .unwrap_or_else(|| panic!("no hashline containing {needle:?} in:\n{hashlines}"))
        .split('|')
        .next()
        .unwrap()
        .to_string()
}

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

#[test]
fn work_done_across_separate_processes_lands_in_one_replayable_session() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());
    let doc = doc.to_str().unwrap();
    let session = dir.path().join("sessions/byo.hick");

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

#[test]
fn a_command_that_names_no_document_explains_itself() {
    let dir = tempfile::tempdir().unwrap();
    // Two documents: guessing between them could put an edit in the wrong
    // file, so the command must refuse and say how to disambiguate.
    std::fs::write(dir.path().join("a.hick"), DOC).unwrap();
    std::fs::write(dir.path().join("b.hick"), DOC).unwrap();
    let out = hick()
        .arg("doc")
        .arg("read")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("exactly one .hick"), "{err}");
    assert!(err.contains("hick doc"), "{err}");
}

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

#[test]
fn the_mcp_server_speaks_the_protocol_and_keeps_one_session_open() {
    use serde_json::json;

    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path());

    let replies = mcp_exchange(
        &doc,
        &[
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}),
            // A notification carries no id and must draw no response at all.
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
            json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
                "name":"edit_output",
                "arguments":{"path":"greet.rs","run":"4f20","input":"fn greet() { println!(\"via mcp\"); }"}
            }}),
            // The SAME anchor again. In-process the session has already
            // re-woven, so this is stale — no re-read, no second open.
            json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{
                "name":"edit_output",
                "arguments":{"path":"greet.rs","run":"4f20","input":"must not apply"}
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
    // The document tools, then the debugger — an agent that can only read a
    // failing document is guessing, and these are what let it stop at the
    // failure and ask.
    assert_eq!(
        names,
        vec![
            "read_doc",
            "read_output",
            "read_file",
            "edit_output",
            "edit_doc",
            "verify",
            "search",
            "debug_start",
            "debug_state",
            "debug_eval",
            "debug_step",
            "debug_stop",
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
