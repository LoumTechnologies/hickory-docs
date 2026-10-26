<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Use the same document tools from an external coding agent

For: Authors using an agent to change documents while retaining review control.

An external agent reads and edits a document through its anchored tool interface, then inspects the resulting source and evidence.

## What must happen

The CLI document commands and MCP surface expose the shared anchored document and lineage tools. Edits recheck their supplied source identities and return their actual results. Optional session recording retains requests and observations. The built-in Python loop is a client of the shared host boundaries, not a second document-edit implementation. External adapters retain their own execution authority.

## Limits of the evidence

A shared tool schema does not authenticate external agent claims or confine that agent's independent filesystem tools.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/byo-agent-tool-surface.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Keep the common document-tool contract and apply the current Python ownership amendment. Separate the external agent interface from hosted snippet restrictions.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-011b12ab71c2"></a>
## `crates/hickory-cli/src/doc_tools.rs`

Related contracts: [Use the same document tools from an external coding agent](contract.md), [Distinguish an accepted change from a verified runtime result](../completion-does-not-certify-verification/contract.md), [Recording failure is visible](../recording-failure-is-visible/contract.md).

<a id="f-011b12ab71c2-OutputFormat"></a>
### `OutputFormat`

`hick doc …` — the document edit tool set, as commands.
These are the same five tools the built-in agent uses (`read_doc`,
`read_output`, `edit_output`, `edit_doc`, `verify`), reached through the
same [`hickory_agent::execute_tool`] with the same [`EditSession`]. Nothing
is reimplemented here: this module builds a [`ToolInvocation`] out of
command-line arguments and prints the resulting observation.
**Why this exists.** Hashline anchors and byte-exact lineage are the part
of hick that a general coding agent cannot reproduce by editing text and
hoping. Leaving them reachable only from inside our own ReAct loop meant
bringing your own agent — Claude Code, Codex, Grok CLI — bought you a
worse experience than using ours, which is the wrong way round for a
product whose durable artifact is a file in your git repo. A command
surface is the universal one: every harness can run a process, whether or
not it speaks MCP, and CI can run one too.
**What one invocation is.** A command opens a session, runs one tool, and
exits. That is weaker than the in-process loop, where the session persists
and re-weaves after edits, refusing a changed surface until it is read again. Here, freshness comes from the anchors themselves: an anchor
is a hash of the line's content, so an edit against a line that changed
since it was read does not resolve, and is refused rather than misapplied.
The cost is a weave per command, not correctness.

<storycode:copy id="f-011b12ab71c2-OutputFormat">
//! `hick doc …` — the document edit tool set, as commands.
//!
//! These are the same five tools the built-in agent uses (`read_doc`,
//! `read_output`, `edit_output`, `edit_doc`, `verify`), reached through the
//! same [`hickory_agent::execute_tool`] with the same [`EditSession`]. Nothing
//! is reimplemented here: this module builds a [`ToolInvocation`] out of
//! command-line arguments and prints the resulting observation.
//!
//! **Why this exists.** Hashline anchors and byte-exact lineage are the part
//! of hick that a general coding agent cannot reproduce by editing text and
//! hoping. Leaving them reachable only from inside our own ReAct loop meant
//! bringing your own agent — Claude Code, Codex, Grok CLI — bought you a
//! worse experience than using ours, which is the wrong way round for a
//! product whose durable artifact is a file in your git repo. A command
//! surface is the universal one: every harness can run a process, whether or
//! not it speaks MCP, and CI can run one too.
//!
//! **What one invocation is.** A command opens a session, runs one tool, and
//! exits. That is weaker than the in-process loop, where the session persists
//! and re-weaves after edits, refusing a changed surface until it is read again. Here, freshness comes from the anchors themselves: an anchor
//! is a hash of the line's content, so an edit against a line that changed
//! since it was read does not resolve, and is refused rather than misapplied.
//! The cost is a weave per command, not correctness.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use hickory_agent::{EditSession, ToolInvocation, ToolOutcome, execute_tool};

use crate::ExecutorChoice;

/// How the observation is printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// The observation text, as the model would see it.
    Text,
    /// `{"tool", "ok", "text"}` — for a caller that wants the ok flag
    /// separately from the prose, e.g. an MCP wrapper.
    Json,
}

</storycode:copy>
<a id="f-011b12ab71c2-DocToolRequest"></a>
### `DocToolRequest`

One `hick doc` invocation: which document, which tool, what arguments.

<storycode:copy id="f-011b12ab71c2-DocToolRequest">
/// One `hick doc` invocation: which document, which tool, what arguments.
pub struct DocToolRequest {
    pub doc: PathBuf,
    pub tool: String,
    pub args: Vec<(String, String)>,
    pub input: Option<String>,
    pub params: Vec<(String, String)>,
    pub format: OutputFormat,
    /// Append this invocation and its result to a `hick:session` document.
    pub session: Option<PathBuf>,
}

</storycode:copy>
<a id="f-011b12ab71c2-session_from"></a>
### `session_from`

The session file to record into: `--session`, else `HICKORY_SESSION`.
An environment variable is the right shape for this because the caller is
usually a coding agent that was told "work on this document" once, at the
start — not a human retyping a path on every command. Setting it in the
agent's environment makes recording the default for a whole piece of work
without threading a flag through every tool call the agent decides to make.

<storycode:copy id="f-011b12ab71c2-session_from">
/// The session file to record into: `--session`, else `HICKORY_SESSION`.
///
/// An environment variable is the right shape for this because the caller is
/// usually a coding agent that was told "work on this document" once, at the
/// start — not a human retyping a path on every command. Setting it in the
/// agent's environment makes recording the default for a whole piece of work
/// without threading a flag through every tool call the agent decides to make.
pub fn session_from(explicit: Option<PathBuf>) -> Option<PathBuf> {
    explicit.or_else(|| {
        std::env::var("HICKORY_SESSION")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(PathBuf::from)
    })
}

</storycode:copy>
<a id="f-011b12ab71c2-record_tool_call"></a>
### `record_tool_call`

Append one tool call and its result to a session document.
What this captures is the *tool-level* truth: which document, which
anchors, what text, what came back. What it cannot capture is the external
agent's reasoning, which lives in that harness's own transcript and is not
ours to read. That is a deliberate boundary rather than an oversight —
tool calls are what `hick ingest --from session` replays, and importing three vendors'
private log formats to recover prose would be a maintenance burden that
buys nothing replayable.

<storycode:copy id="f-011b12ab71c2-record_tool_call">
/// Append one tool call and its result to a session document.
///
/// What this captures is the *tool-level* truth: which document, which
/// anchors, what text, what came back. What it cannot capture is the external
/// agent's reasoning, which lives in that harness's own transcript and is not
/// ours to read. That is a deliberate boundary rather than an oversight —
/// tool calls are what `hick ingest --from session` replays, and importing three vendors'
/// private log formats to recover prose would be a maintenance burden that
/// buys nothing replayable.
pub fn record_tool_call(
    session_path: &Path,
    invocation: &ToolInvocation,
    outcome: &ToolOutcome,
) -> Result<()> {
    use hickory_agent::{HickSessionLog, SessionEvent, SessionLog as _, record_outcome};

    let log = HickSessionLog::append_or_create(session_path)?;
    log.record(SessionEvent::ToolCall {
        prose: "",
        xml: &invocation.raw_xml,
        reasoning: None,
    });
    record_outcome(&log, outcome);
    // Close the root after every command, so the file is a parseable session
    // between calls and not only once the agent happens to stop.
    log.record(SessionEvent::End);
    log.check()?;
    Ok(())
}

</storycode:copy>
<a id="f-011b12ab71c2-record_intent"></a>
### `record_intent`

Durable intent before effects; failure prevents running the requested tool.

<storycode:copy id="f-011b12ab71c2-record_intent">
/// Durable intent before effects; failure prevents running the requested tool.
pub fn record_intent(path: &Path, tool: &str, input: &serde_json::Value) -> Result<()> {
    use hickory_agent::{HickSessionLog, SessionEvent, SessionLog};
    let log = HickSessionLog::append_or_create(path)?;
    let data = serde_json::json!({"tool":tool,"input":input,"state":"requested","recorded_at":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis().to_string(),"actor":"harness","assumption_coverage":"Not established; inspect declared audit entries"});
    log.record(SessionEvent::Context {
        kind: "audit-action",
        data: &data,
    });
    log.record(SessionEvent::End);
    log.check()
}

</storycode:copy>
<a id="f-011b12ab71c2-record_check"></a>
### `record_check`

<storycode:copy id="f-011b12ab71c2-record_check">
pub fn record_check(
    path: &Path,
    ok: bool,
    revision: &str,
    sources: &[serde_json::Value],
) -> Result<()> {
    use hickory_agent::{HickSessionLog, SessionEvent, SessionLog};
    let log = HickSessionLog::append_or_create(path)?;
    log.record(SessionEvent::Context { kind: "verification", data: &serde_json::json!({"state":if ok {"passed"} else {"failed"},"revision":revision,"source_snapshots":sources,"actor":"harness","tool":"verify","scope":"Document execution and expectations for this source closure; external data, semantic truth and completeness are not certified"}) });
    log.record(SessionEvent::End);
    log.check()
}

</storycode:copy>
<a id="f-011b12ab71c2-record_activity_result"></a>
### `record_activity_result`

<storycode:copy id="f-011b12ab71c2-record_activity_result">
pub fn record_activity_result(
    path: &Path,
    tool: &str,
    result: &std::result::Result<String, String>,
) -> Result<()> {
    use hickory_agent::{HickSessionLog, SessionEvent, SessionLog};
    let log = HickSessionLog::append_or_create(path)?;
    log.record(SessionEvent::Context { kind: "audit-action", data: &serde_json::json!({"tool":tool,"state":"completed","ok":result.is_ok(),"result":result.as_ref().ok(),"error":result.as_ref().err(),"actor":"harness"}) });
    log.record(SessionEvent::End);
    log.check()
}

</storycode:copy>
<a id="f-011b12ab71c2-run_doc_tool"></a>
### `run_doc_tool`

Run one tool against `doc` and return its outcome.
Errors here are *setup* failures (an unreadable document, a payload that
cannot be represented). Everything the tool itself refuses — an anchor that
does not resolve, a lineage refusal, an unknown output path — comes back as
a [`ToolOutcome`] with `ok: false` and a message that says where to go
next, because that is routing information rather than a crash.

<storycode:copy id="f-011b12ab71c2-run_doc_tool">
/// Run one tool against `doc` and return its outcome.
///
/// Errors here are *setup* failures (an unreadable document, a payload that
/// cannot be represented). Everything the tool itself refuses — an anchor that
/// does not resolve, a lineage refusal, an unknown output path — comes back as
/// a [`ToolOutcome`] with `ok: false` and a message that says where to go
/// next, because that is routing information rather than a crash.
pub async fn run_doc_tool(req: &DocToolRequest) -> Result<ToolOutcome> {
    if !req.doc.exists() {
        anyhow::bail!(
            "no such document: {}\n\
             Pass the path to a .md file — the document is the source, not a generated output.",
            req.doc.display()
        );
    }
    let invocation =
        ToolInvocation::synthetic(req.tool.clone(), req.args.clone(), req.input.clone())
            .map_err(|e| anyhow::anyhow!(e))?;

    let mut session = EditSession::open(&req.doc, &req.params)
        .await
        .with_context(|| format!("opening an edit session on {}", req.doc.display()))?;

    // `verify` is the only tool that executes anything, but the executor is
    // built either way: choosing it from HICKORY_EXECUTOR in one place keeps
    // `hick doc verify` identical to `hick run` rather than quietly
    // local-only.
    let executor = ExecutorChoice::from_env()?
        .build_for(req.doc.parent())
        .await?;
    if let Some(path) = &req.session {
        record_intent(
            path,
            &req.tool,
            &serde_json::json!({"args":req.args,"input":req.input}),
        )?;
    }
    let mut outcome = execute_tool(&mut session, executor.clone(), &invocation).await;
    executor.shutdown().await.ok();

    if let Some(path) = &req.session {
        // A failure to record is reported, never fatal: the edit already
        // happened, and killing the command afterwards would tell the caller
        // its work failed when the document on disk says otherwise.
        let recorded = record_tool_call(path, &invocation, &outcome).and_then(|()| {
            if invocation.name == "verify" {
                record_check(
                    path,
                    outcome.ok,
                    &session.revision(),
                    &session.source_snapshots(),
                )
            } else {
                Ok(())
            }
        });
        if let Err(e) = recorded {
            outcome.ok = false;
            outcome.text.push_str(&format!("\nRecording failed after the tool ran; inspect actual files. Session {} is incomplete: {e:#}", path.display()));
        }
    }
    Ok(outcome)
}

</storycode:copy>
<a id="f-011b12ab71c2-print_outcome"></a>
### `print_outcome`

Print an outcome and return the process exit code.
The observation goes to **stdout** whether or not the tool succeeded: a
refusal's text is the useful part (it names the document location to edit
instead), and an agent that only reads stdout on success would throw away
the answer. The one-line status goes to stderr, where a human reads it and
a pipeline ignores it.

<storycode:copy id="f-011b12ab71c2-print_outcome">
/// Print an outcome and return the process exit code.
///
/// The observation goes to **stdout** whether or not the tool succeeded: a
/// refusal's text is the useful part (it names the document location to edit
/// instead), and an agent that only reads stdout on success would throw away
/// the answer. The one-line status goes to stderr, where a human reads it and
/// a pipeline ignores it.
pub fn print_outcome(outcome: &ToolOutcome, format: OutputFormat) -> Result<u8> {
    match format {
        OutputFormat::Text => {
            println!("{}", outcome.text);
            if !outcome.ok {
                eprintln!(
                    "{}: refused or failed (see the message above)",
                    outcome.name
                );
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "tool": outcome.name,
                    "ok": outcome.ok,
                    "text": outcome.text,
                }))?
            );
        }
    }
    Ok(if outcome.ok { 0 } else { 1 })
}

</storycode:copy>
<a id="f-011b12ab71c2-read_input"></a>
### `read_input`

Read an edit payload: the `--input` value, or stdin when it is `-` or
absent.
Absent means stdin rather than "empty" because a replacement text is
usually a block of code, and shell quoting mangles those. Deleting a run is
therefore an explicit `--input ""`, not an omission — an accidental
deletion is a much worse default than a command that waits for input.

<storycode:copy id="f-011b12ab71c2-read_input">
/// Read an edit payload: the `--input` value, or stdin when it is `-` or
/// absent.
///
/// Absent means stdin rather than "empty" because a replacement text is
/// usually a block of code, and shell quoting mangles those. Deleting a run is
/// therefore an explicit `--input ""`, not an omission — an accidental
/// deletion is a much worse default than a command that waits for input.
pub fn read_input(arg: Option<&str>) -> Result<Option<String>> {
    match arg {
        Some("-") | None => {
            use std::io::Read as _;
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .context("reading the replacement text from stdin")?;
            Ok(Some(buf))
        }
        Some(text) => Ok(Some(text.to_string())),
    }
}

</storycode:copy>
<a id="f-011b12ab71c2-edit_args"></a>
### `edit_args`

Assemble the tool arguments for an edit, rejecting the two mistakes that
produce a confusing tool-level error later: naming both anchors, or naming
neither.

<storycode:copy id="f-011b12ab71c2-edit_args">
/// Assemble the tool arguments for an edit, rejecting the two mistakes that
/// produce a confusing tool-level error later: naming both anchors, or naming
/// neither.
pub fn edit_args(
    path: Option<&str>,
    run: Option<&str>,
    after: Option<&str>,
    occurrence: Option<usize>,
) -> Result<Vec<(String, String)>> {
    if run.is_some() && after.is_some() {
        anyhow::bail!(
            "--run and --after are alternatives: --run REPLACES the anchored line(s), \
             --after INSERTS below a single line. Pass one."
        );
    }
    if run.is_none() && after.is_none() {
        anyhow::bail!(
            "an edit needs an anchor: --run <hash>[..<hash>] to replace lines, or \
             --after <hash> (or --after ^ for the top of the file) to insert.\n\
             Hashes come from `hick doc read` / `hick doc read-output` — the \
             `hash|` SHA-256 prefix on each line."
        );
    }
    let mut args = Vec::new();
    if let Some(p) = path {
        args.push(("path".to_string(), p.to_string()));
    }
    if let Some(r) = run {
        args.push(("run".to_string(), r.to_string()));
    }
    if let Some(a) = after {
        args.push(("after".to_string(), a.to_string()));
    }
    if let Some(n) = occurrence {
        args.push(("occurrence".to_string(), n.to_string()));
    }
    Ok(args)
}

</storycode:copy>
<a id="f-011b12ab71c2-sole_document"></a>
### `sole_document`

The document a bare `hick doc` command should act on when none is
named and exactly one `.md` file is in scope.
Deliberately narrow: it looks in one directory and gives up the moment
there is more than one candidate. Guessing between documents would put an
edit in the wrong file, which lineage cannot undo.

<storycode:copy id="f-011b12ab71c2-sole_document">
/// The document a bare `hick doc` command should act on when none is
/// named and exactly one `.md` file is in scope.
///
/// Deliberately narrow: it looks in one directory and gives up the moment
/// there is more than one candidate. Guessing between documents would put an
/// edit in the wrong file, which lineage cannot undo.
pub fn sole_document(dir: &Path) -> Option<PathBuf> {
    let mut found = None;
    for entry in std::fs::read_dir(dir).ok()? {
        let path = entry.ok()?.path();
        if path.extension().is_some_and(|e| e == "md") {
            if found.is_some() {
                return None;
            }
            found = Some(path);
        }
    }
    found
}

</storycode:copy>
<a id="f-011b12ab71c2-tests"></a>
### `tests`

<storycode:copy id="f-011b12ab71c2-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_edit_needs_exactly_one_anchor() {
        let both = edit_args(None, Some("aaaa"), Some("bbbb"), None).unwrap_err();
        assert!(both.to_string().contains("alternatives"), "{both}");

        let neither = edit_args(None, None, None, None).unwrap_err();
        // The message has to say where hashes come from; an anchor is not
        // something a caller can invent.
        assert!(neither.to_string().contains("hick doc read"), "{neither}");
    }

    #[test]
    fn edit_args_carry_through_in_tool_vocabulary() {
        let args = edit_args(Some("gen.rs"), Some("a1b2..c3d4"), None, Some(2)).unwrap();
        assert_eq!(
            args,
            vec![
                ("path".to_string(), "gen.rs".to_string()),
                ("run".to_string(), "a1b2..c3d4".to_string()),
                ("occurrence".to_string(), "2".to_string()),
            ]
        );
    }

    #[test]
    fn a_lone_document_is_found_and_an_ambiguous_directory_is_not() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(sole_document(dir.path()), None);

        std::fs::write(dir.path().join("only.md"), "x").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "x").unwrap();
        assert_eq!(sole_document(dir.path()), Some(dir.path().join("only.md")));

        // Two documents: no guess. An edit aimed at the wrong file is not
        // something lineage can walk back.
        std::fs::write(dir.path().join("second.md"), "x").unwrap();
        assert_eq!(sole_document(dir.path()), None);
    }

    /// Guarantee: docs/guarantees/agent/byo-agent-tool-surface.md — a
    /// synthesized invocation is representable in a session document.
    #[test]
    fn a_payload_that_cannot_be_recorded_is_refused_up_front() {
        let err = ToolInvocation::synthetic(
            "edit_doc",
            vec![("run".into(), "aaaa".into())],
            Some("text with </hick:input> inside".into()),
        )
        .unwrap_err();
        assert!(err.contains("</hick:input>"), "{err}");
    }

    #[test]
    fn a_synthesized_invocation_round_trips_through_the_session_parser() {
        let inv = ToolInvocation::synthetic(
            "edit_output",
            vec![
                ("path".into(), "gen.rs".into()),
                ("run".into(), "a1b2..c3d4".into()),
            ],
            Some("fn main() {}".into()),
        )
        .unwrap();
        let response = format!("<hick:next>tool</hick:next>\n{}", inv.raw_xml);
        let parsed = hickory_agent::parse_tool_invocation(&response).unwrap();
        assert_eq!(parsed.name, "edit_output");
        assert_eq!(parsed.arg("path"), Some("gen.rs"));
        assert_eq!(parsed.arg("run"), Some("a1b2..c3d4"));
        assert_eq!(parsed.input.as_deref(), Some("fn main() {}"));
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/doc_tools.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/doc_tools.rs"><storycode:paste select="#f-011b12ab71c2-OutputFormat"/><storycode:paste select="#f-011b12ab71c2-DocToolRequest"/><storycode:paste select="#f-011b12ab71c2-session_from"/><storycode:paste select="#f-011b12ab71c2-record_tool_call"/><storycode:paste select="#f-011b12ab71c2-record_intent"/><storycode:paste select="#f-011b12ab71c2-record_check"/><storycode:paste select="#f-011b12ab71c2-record_activity_result"/><storycode:paste select="#f-011b12ab71c2-run_doc_tool"/><storycode:paste select="#f-011b12ab71c2-print_outcome"/><storycode:paste select="#f-011b12ab71c2-read_input"/><storycode:paste select="#f-011b12ab71c2-edit_args"/><storycode:paste select="#f-011b12ab71c2-sole_document"/><storycode:paste select="#f-011b12ab71c2-tests"/></storycode:file>

## Test the behavior: `hick`

This is the maintained test fragment from `crates/hickory-cli/tests/byo_agent_surface.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-43da4a5da7c9-hick"></a>
### `hick`

The bring-your-own-agent surface, driven exactly the way an outside coding
agent drives it: by running the shipped binary.
Protects docs/guarantees/agent/byo-agent-tool-surface.md. The claim under
test is not "the commands exist" but "an agent that has never seen our
ReAct loop gets the same guarantees from them" — hashline anchors, an edit
through an output mapped back byte-exactly, a stale anchor refused rather
than misapplied, and a replayable `hick:session` written across separate
processes.

<storycode:copy id="f-43da4a5da7c9-hick">
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

</storycode:copy>

## Test the behavior: `init_registers_the_mcp_server_without_clobbering_other_entries`

This is the maintained test fragment from `crates/hickory-cli/tests/init_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-7418ab26c01b-init_registers_the_mcp_server_without_clobbering_other_entries"></a>
### `init_registers_the_mcp_server_without_clobbering_other_entries`

`hick init` registers the MCP server for harnesses that read the
project's `.mcp.json` — and must never damage what is already there.
Protects docs/guarantees/agent/byo-agent-tool-surface.md.

<storycode:copy id="f-7418ab26c01b-init_registers_the_mcp_server_without_clobbering_other_entries">
/// `hick init` registers the MCP server for harnesses that read the
/// project's `.mcp.json` — and must never damage what is already there.
///
/// Protects docs/guarantees/agent/byo-agent-tool-surface.md.
#[test]
fn init_registers_the_mcp_server_without_clobbering_other_entries() {
    let repo = init_repo();
    let mcp_path = repo.path().join(".mcp.json");

    // A repo that already registers another server. Re-running init must be
    // additive: an init that deleted someone's server would make the command
    // unsafe to re-run, which is the one property it promises.
    std::fs::write(
        &mcp_path,
        r#"{"mcpServers":{"other":{"command":"other-server","args":[]}}}"#,
    )
    .unwrap();

    run_init(repo.path());
    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mcp_path).unwrap()).unwrap();
    assert_eq!(parsed["mcpServers"]["hick"]["command"], "hick");
    assert_eq!(parsed["mcpServers"]["hick"]["args"][0], "mcp");
    assert_eq!(
        parsed["mcpServers"]["other"]["command"], "other-server",
        "init destroyed an unrelated MCP server"
    );

    // Idempotent: a second run rewrites nothing.
    let first = std::fs::read_to_string(&mcp_path).unwrap();
    run_init(repo.path());
    assert_eq!(first, std::fs::read_to_string(&mcp_path).unwrap());
}

</storycode:copy>

## Implementation review

### Implementation

- [crates/hickory-agent/src/session.rs](../../../mechanisms/crates/hickory-agent/src/session.rs.md#output-9c7ae0f28c14)
- [crates/hickory-cli/src/doc_tools.rs](contract.md#output-011b12ab71c2)
- [crates/hickory-cli/src/init.rs](../../../mechanisms/crates/hickory-cli/src/init.rs.md#output-a329ec22d0b8)
- [crates/hickory-cli/src/mcp.rs](../../../mechanisms/crates/hickory-cli/src/mcp.rs.md#output-5cbce5960257)

### Tests and fixtures

- [crates/hickory-cli/tests/byo_agent_surface.rs](../../../mechanisms/crates/hickory-cli/tests/byo_agent_surface.rs.md#output-43da4a5da7c9)
- [crates/hickory-cli/tests/init_tests.rs](../../../mechanisms/crates/hickory-cli/tests/init_tests.rs.md#output-7418ab26c01b)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
