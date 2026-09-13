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
//! and re-weaves after every edit so a stale anchor is impossible by
//! construction. Here, freshness comes from the anchors themselves: an anchor
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
    Ok(())
}

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
             Pass the path to a .hick file — the document is the source, not a generated output.",
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
    let outcome = execute_tool(&mut session, executor.clone(), &invocation).await;
    executor.shutdown().await.ok();

    if let Some(path) = &req.session {
        // A failure to record is reported, never fatal: the edit already
        // happened, and killing the command afterwards would tell the caller
        // its work failed when the document on disk says otherwise.
        if let Err(e) = record_tool_call(path, &invocation, &outcome) {
            eprintln!("warning: could not record into {}: {e:#}", path.display());
        }
    }
    Ok(outcome)
}

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
             `hhhh|` prefix on each line."
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

/// The document a bare `hick doc` command should act on when none is
/// named and exactly one `.hick` file is in scope.
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

        std::fs::write(dir.path().join("only.hick"), "x").unwrap();
        std::fs::write(dir.path().join("notes.md"), "x").unwrap();
        assert_eq!(
            sole_document(dir.path()),
            Some(dir.path().join("only.hick"))
        );

        // Two documents: no guess. An edit aimed at the wrong file is not
        // something lineage can walk back.
        std::fs::write(dir.path().join("second.hick"), "x").unwrap();
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
