//! The app's debugger, on channel `0x03`.
//!
//! ## Why this is not raw DAP
//!
//! The spec sketched this as "DAP messages the way `0x02` carries LSP ones".
//! Building it showed that to be the wrong shape, for one reason: DAP speaks
//! in the coordinates of the file the debugger is running, and the app speaks
//! in the coordinates of the document a person is reading. Proxying raw DAP
//! would put that mapping in the browser — a second implementation of the one
//! thing the session API exists to get right, in the one place least able to
//! test it.
//!
//! So the channel carries the session API's own verbs, and the mapping stays
//! in Rust with the code that already knows how to do it. The client is a
//! debugger UI, not a debug adapter client.
//!
//! ## What it may not do
//!
//! Write. There is no edit operation on this channel — document edits go
//! through the CRDT room and the hashline API, neither of which this holds —
//! and a session's debuggee runs in a scratch copy that is deleted with it.
//! That is what makes "no amount of stepping can change the file" a fact
//! about the plumbing rather than a promise about behaviour.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::{Breakpoint, BuildOutput, Step};
use hickory_collab::CHANNEL_DEBUG;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::debug_sessions::Registry;

/// What the app asks for.
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    /// Start a session over a document, with breakpoints in document lines.
    Start {
        doc: String,
        #[serde(default)]
        breakpoints: Vec<Breakpoint>,
        /// Which generated file to run. Omitted means "the first one that can
        /// be debugged", which is only ever right by accident once a document
        /// generates two.
        #[serde(default)]
        program: Option<String>,
    },
    /// Replace the breakpoint set while a session is running.
    Breakpoints {
        session: String,
        breakpoints: Vec<Breakpoint>,
    },
    /// Where are we, and what is in scope.
    State {
        session: String,
    },
    /// Evaluate in a frame. `context` is DAP's: `hover`, `watch` or `repl`.
    Eval {
        session: String,
        expression: String,
        #[serde(default)]
        frame: Option<i64>,
        #[serde(default = "repl")]
        context: String,
    },
    /// Move. `how` is the session API's own verb.
    Step {
        session: String,
        how: Step,
        #[serde(default)]
        frame: Option<i64>,
    },
    /// Move the instruction pointer to a document line in this frame.
    Jump {
        session: String,
        line: u32,
    },
    /// Run to a line without leaving a breakpoint behind.
    RunTo {
        session: String,
        line: u32,
    },
    /// Expand one composite value.
    Children {
        session: String,
        reference: i64,
    },
    /// Change a value in the running program.
    SetVariable {
        session: String,
        container: i64,
        name: String,
        value: String,
    },
    Stop {
        session: String,
    },
}

fn repl() -> String {
    "repl".to_string()
}

/// What the app is told.
#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Response {
    Started {
        session: String,
        capabilities: hick_dap::Capabilities,
        breakpoints: Vec<hick_dap::BreakpointStatus>,
    },
    /// Where execution is now — the paused line the gutter marks, and the
    /// values the editor shows inline.
    Stopped {
        session: String,
        reason: String,
        frames: Vec<hick_dap::Frame>,
        variables: Vec<hick_dap::Variable>,
        /// The top frame's document line, hoisted so the editor need not
        /// hunt for it.
        line: Option<u32>,
    },
    Breakpoints {
        session: String,
        breakpoints: Vec<hick_dap::BreakpointStatus>,
    },
    Value {
        session: String,
        expression: String,
        value: String,
        #[serde(rename = "type")]
        type_name: Option<String>,
        reference: i64,
    },
    Children {
        session: String,
        reference: i64,
        variables: Vec<hick_dap::Variable>,
    },
    /// The program ended. Not an error: it is what `continue` usually does.
    ///
    /// The session is reaped WITH it — adapter process killed, scratch clone
    /// deleted, registry entry gone — because a debugger over a program that
    /// no longer exists is a process holding a workdir open. The id in here
    /// no longer answers; the client keeps its breakpoints and starts a new
    /// session to run again.
    Finished {
        session: String,
        /// The debuggee's exit code, when the adapter reported one.
        exit_code: Option<i64>,
    },
    Ended {
        session: String,
    },
    /// A build that had to happen before there was a program to launch.
    ///
    /// Carried as `TranscriptEvent`-shaped events on purpose: the client
    /// already has a terminal that renders exactly those
    /// (`terminal/WatchingTerminal.tsx`), and a build tool's ANSI colour and
    /// carriage-return rewriting are the whole reason it is a terminal and
    /// not a text card. This is NOT a transcript: nothing here is recorded
    /// under a cache key, woven, or compared. The terminal is the run
    /// happening; the transcript is the record.
    ///
    /// It arrives BEFORE `started` on success and before `failed` on
    /// failure, and the failing case is the one it exists for — a build that
    /// fails says why in MSBuild's own words, and "build failed" throws all
    /// of that away.
    Build {
        events: Vec<Value>,
    },
    /// Something did not work, in words a person can act on.
    Failed {
        session: Option<String>,
        message: String,
        /// Which request failed, so the app can put the message where it
        /// belongs. A failure with no home becomes a banner that outlives its
        /// cause and attaches itself to whatever the person does next.
        #[serde(skip_serializing_if = "Option::is_none")]
        about: Option<String>,
        /// The document lines the failed request was about, when it was about
        /// lines: a breakpoint that could not be set is a fact about that
        /// breakpoint, shown on it.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        lines: Vec<u32>,
    },
}

/// Handle one request and produce the responses to send back.
///
/// Returns several because one action often has two answers: a step both
/// moves and lands somewhere, and the app wants the landing.
pub async fn handle(
    registry: &Arc<Registry>,
    root: &std::path::Path,
    request: Request,
) -> Vec<Response> {
    let about = about_of(&request);
    let lines = lines_of(&request);
    // Collected out here rather than inside, because the case that needs it
    // is the failing one: a build that fails takes `handle_inner` down the
    // error arm, and anything gathered in there would go with it.
    let mut built: Vec<Value> = Vec::new();
    let result = handle_inner(registry, root, request, &mut built).await;
    let mut out = Vec::new();
    if !built.is_empty() {
        out.push(Response::Build { events: built });
    }
    match result {
        Ok(responses) => out.extend(responses),
        Err((session, error)) => out.push(Response::Failed {
            session,
            message: format!("{error:#}"),
            about: Some(about.to_string()),
            lines,
        }),
    }
    out
}

/// One build line, in the shape the client's terminal already reads.
///
/// `t` is milliseconds since the build started — the same meaning `t` has on
/// a cell's transcript, so the same component orders them the same way.
fn build_event(line: BuildOutput, since: std::time::Instant) -> Value {
    let t = since.elapsed().as_millis() as u64;
    match line {
        BuildOutput::Cmd(text) => json!({ "t": t, "kind": "cmd", "data": text }),
        // A note is this app talking, not the build tool. It is carried as
        // output rather than as a fifth kind because the terminal renders
        // what a person reads, and the indent is how the spec writes it.
        BuildOutput::Note(text) => json!({ "t": t, "kind": "out", "data": format!("  {text}\n") }),
        BuildOutput::Out(text) => json!({ "t": t, "kind": "out", "data": format!("{text}\n") }),
        BuildOutput::Err(text) => json!({ "t": t, "kind": "err", "data": format!("{text}\n") }),
        BuildOutput::Exit(code) => json!({ "t": t, "kind": "exit", "code": code }),
    }
}

/// The name of the request, for a failure to point at.
fn about_of(request: &Request) -> &'static str {
    match request {
        Request::Start { .. } => "start",
        Request::Breakpoints { .. } => "breakpoints",
        Request::State { .. } => "state",
        Request::Eval { .. } => "eval",
        Request::Step { .. } => "step",
        Request::Jump { .. } => "jump",
        Request::RunTo { .. } => "run_to",
        Request::Children { .. } => "children",
        Request::SetVariable { .. } => "set_variable",
        Request::Stop { .. } => "stop",
    }
}

/// The document lines a request is about, when it is about lines.
fn lines_of(request: &Request) -> Vec<u32> {
    match request {
        Request::Start { breakpoints, .. } | Request::Breakpoints { breakpoints, .. } => {
            breakpoints.iter().map(|b| b.line).collect()
        }
        Request::Jump { line, .. } | Request::RunTo { line, .. } => vec![*line],
        _ => Vec::new(),
    }
}

type Failure = (Option<String>, anyhow::Error);

async fn handle_inner(
    registry: &Arc<Registry>,
    root: &std::path::Path,
    request: Request,
    built: &mut Vec<Value>,
) -> Result<Vec<Response>, Failure> {
    match request {
        Request::Start {
            doc,
            breakpoints,
            program,
        } => {
            let path = resolve(root, &doc);
            let since = std::time::Instant::now();
            let (session, live, statuses) = registry
                .start(&path, &breakpoints, program.as_deref(), &mut |line| {
                    built.push(build_event(line, since))
                })
                .await
                .map_err(|e| (None, e))?;
            let mut out = vec![Response::Started {
                session: session.clone(),
                capabilities: live.session.capabilities().clone(),
                breakpoints: statuses,
            }];
            // Run to the first stop before answering, so the UI never shows a
            // started session with no position in it.
            out.extend(settle(registry, &session, &live).await);
            Ok(out)
        }

        Request::Breakpoints {
            session,
            breakpoints,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let statuses = live
                .session
                .set_breakpoints(&breakpoints)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(vec![Response::Breakpoints {
                session,
                breakpoints: statuses,
            }])
        }

        Request::State { session } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(position(&session, &live).await)
        }

        Request::Eval {
            session,
            expression,
            frame,
            context,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let frame = match frame {
                Some(frame) => Some(frame),
                None => top_frame(&live).await,
            };
            let value = live
                .session
                .evaluate(&expression, frame, &context)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(vec![Response::Value {
                session,
                expression,
                value: value.value,
                type_name: value.type_name,
                reference: value.variables_reference,
            }])
        }

        Request::Step {
            session,
            how,
            frame,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let thread = live.thread_id.lock().await.unwrap_or(1);
            let frame = match frame {
                Some(frame) => Some(frame),
                None => top_frame(&live).await,
            };
            live.session
                .step(how, thread, frame)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(settle(registry, &session, &live).await)
        }

        Request::Jump { session, line } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let thread = live.thread_id.lock().await.unwrap_or(1);
            live.session
                .jump_to(line, thread)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(settle_in_place(&session, &live).await)
        }

        Request::RunTo { session, line } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let thread = live.thread_id.lock().await.unwrap_or(1);
            live.session
                .run_to(line, thread)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(settle(registry, &session, &live).await)
        }

        Request::Children { session, reference } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let variables = live
                .session
                .children(reference)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(vec![Response::Children {
                session,
                reference,
                variables,
            }])
        }

        Request::SetVariable {
            session,
            container,
            name,
            value,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            live.session
                .set_variable(container, &name, &value)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            // Re-read the position: changing a value changes what the inline
            // display should show, and recomputing it here saves the client
            // from knowing that.
            Ok(position(&session, &live).await)
        }

        Request::Stop { session } => {
            // Reap rather than stop: the debuggee finishing and the person
            // pressing Stop legitimately race, and the loser must find
            // "already gone" — the state they asked for — rather than an
            // error about a session that ended a beat before they clicked.
            registry.reap(&session).await;
            Ok(vec![Response::Ended { session }])
        }
    }
}

/// Wait for the program to stop again, then describe where it is.
///
/// When the answer is "it ended", the session is reaped HERE, before the
/// client is told: a debuggee that ran to completion must not leave its
/// adapter process, scratch clone and registry entry waiting fifteen minutes
/// for the idle sweep. Reaping is idempotent with the explicit stop that may
/// arrive a beat later.
async fn settle(
    registry: &Arc<Registry>,
    session: &str,
    live: &Arc<crate::debug_sessions::Live>,
) -> Vec<Response> {
    match live.session.wait_for_stop(Duration::from_secs(60)).await {
        Ok(Some(stopped)) => {
            *live.thread_id.lock().await = Some(stopped.thread_id);
            let mut out = vec![Response::Breakpoints {
                session: session.to_string(),
                // Re-read rather than reuse the set-time answer: for a
                // compiled language every breakpoint is unconfirmed until the
                // module loads, and the adapter's `breakpoint` event has
                // arrived by the time the program is stopped. Without this
                // the gutter draws every C# breakpoint half-filled forever
                // while the program stops on it perfectly.
                breakpoints: live.session.breakpoint_statuses(),
            }];
            out.extend(position_with(session, live, &stopped.reason).await);
            out
        }
        Ok(None) => {
            *live.thread_id.lock().await = None;
            registry.reap(session).await;
            vec![Response::Finished {
                session: session.to_string(),
                exit_code: live.session.exit().and_then(|exit| exit.code),
            }]
        }
        Err(error) => vec![Response::Failed {
            session: Some(session.to_string()),
            message: format!("{error:#}"),
            about: None,
            lines: Vec::new(),
        }],
    }
}

/// Answer a move that leaves the program *already stopped*.
///
/// A jump moves the instruction pointer without resuming, and the protocol
/// says the adapter then reports a `stopped` with reason `goto`. debugpy does
/// not, so waiting for one is waiting for the timeout — and the app would sit
/// with a stale paused line for a minute over a move that already happened.
/// A short grace period picks up the event where an adapter does send it, and
/// otherwise the answer is simply where we are now.
async fn settle_in_place(session: &str, live: &Arc<crate::debug_sessions::Live>) -> Vec<Response> {
    if let Ok(Some(stopped)) = live.session.wait_for_stop(Duration::from_millis(400)).await {
        *live.thread_id.lock().await = Some(stopped.thread_id);
        return position_with(session, live, &stopped.reason).await;
    }
    position_with(session, live, "goto").await
}

async fn position(session: &str, live: &Arc<crate::debug_sessions::Live>) -> Vec<Response> {
    position_with(session, live, "state").await
}

/// The stack, the top frame's variables, and the line the gutter marks.
async fn position_with(
    session: &str,
    live: &Arc<crate::debug_sessions::Live>,
    reason: &str,
) -> Vec<Response> {
    let thread = match *live.thread_id.lock().await {
        Some(thread) => thread,
        None => {
            return vec![Response::Finished {
                session: session.to_string(),
                exit_code: live.session.exit().and_then(|exit| exit.code),
            }];
        }
    };
    let frames = live.session.stack(thread).await.unwrap_or_default();
    let variables = match frames.first() {
        Some(frame) => live.session.variables(frame.id).await.unwrap_or_default(),
        None => Vec::new(),
    };
    vec![Response::Stopped {
        session: session.to_string(),
        reason: reason.to_string(),
        line: frames.first().and_then(|frame| frame.line),
        frames,
        variables,
    }]
}

async fn top_frame(live: &Arc<crate::debug_sessions::Live>) -> Option<i64> {
    let thread = (*live.thread_id.lock().await)?;
    live.session
        .stack(thread)
        .await
        .ok()?
        .first()
        .map(|frame| frame.id)
}

/// The client names a document in its own scheme; this is where it lives.
fn resolve(root: &std::path::Path, doc: &str) -> PathBuf {
    let relative = doc
        .strip_prefix("hick:///")
        .unwrap_or_else(|| doc.trim_start_matches('/'));
    root.join(relative)
}

/// Send one response as a `0x03` frame.
pub fn frame_of(response: &Response) -> Vec<u8> {
    let mut frame = vec![CHANNEL_DEBUG];
    let body = serde_json::to_vec(response).unwrap_or_else(|_| b"{}".to_vec());
    frame.extend_from_slice(&body);
    frame
}

/// Parse a `0x03` frame from the client.
pub fn request_of(data: &[u8]) -> Result<Request, serde_json::Error> {
    serde_json::from_slice(&data[1..])
}

/// Send responses to the client, dropping them if it has gone.
pub fn reply(tx: &mpsc::UnboundedSender<Vec<u8>>, responses: Vec<Response>) {
    for response in responses {
        if tx.send(frame_of(&response)).is_err() {
            return;
        }
    }
}

/// The shape the client sees, for its own tests.
pub fn describe() -> Value {
    json!({
        "channel": CHANNEL_DEBUG,
        "requests": [
            "start", "breakpoints", "state", "eval", "step", "jump", "run_to",
            "children", "set_variable", "stop"
        ],
        "responses": [
            "started", "stopped", "breakpoints", "value", "children",
            "finished", "ended", "failed", "build"
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A C# document whose build cannot succeed, and what the client is told.
    ///
    /// The point is the ORDER and the SURVIVAL: the build's own output has to
    /// reach the client on the failing path, before the failure, or a person
    /// gets "building app.csproj failed" and none of the reasons.
    #[tokio::test]
    async fn a_failed_build_reaches_the_client_before_the_failure() {
        let dir = tempfile::tempdir().unwrap();
        // A document that generates C# and no project file — the refusal
        // `build` makes without running anything, so this test needs no
        // .NET SDK to be meaningful.
        std::fs::write(
            dir.path().join("a.hick"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             <hick:file path=\"Program.cs\">\n\
             class Program { static void Main() {} }\n\
             </hick:file>\n\
             </hick:doc>\n",
        )
        .unwrap();

        let registry = Arc::new(Registry::new());
        let responses = handle(
            &registry,
            dir.path(),
            Request::Start {
                doc: "hick:///a.hick".into(),
                breakpoints: vec![],
                program: None,
            },
        )
        .await;

        // Discovery runs before the build, so on a machine with no
        // netcoredbg the adapter is what is missing and the build never
        // gets a chance to speak. Either way the person is told something
        // they can act on, and that is what is asserted.
        let failure = responses
            .iter()
            .find_map(|r| match r {
                Response::Failed { message, .. } => Some(message.clone()),
                _ => None,
            })
            .expect("a C# document with no project file cannot start");
        assert!(
            failure.contains("no project file") || failure.contains("no debug adapter"),
            "the failure does not say what is missing: {failure}"
        );
        // And it says how to get out of it. Which sentence that is depends on
        // which of the two things was missing, and both are checked by
        // `dap_install`'s drift test — the point here is that a person is
        // never left with only the bad news.
        assert!(
            failure.contains("hick dap install csharp")
                || failure.contains("hick ingest")
                || failure.contains("hick:file"),
            "the failure says what is wrong but not what to do: {failure}"
        );
        // Whatever the build managed to say arrives BEFORE the failure.
        if let Some(build_at) = responses
            .iter()
            .position(|r| matches!(r, Response::Build { .. }))
        {
            let failed_at = responses
                .iter()
                .position(|r| matches!(r, Response::Failed { .. }))
                .unwrap();
            assert!(build_at < failed_at, "the build arrived after the failure");
        }
    }

    #[test]
    fn a_build_line_is_shaped_like_a_transcript_event() {
        // Not because it IS a transcript — nothing here is recorded or
        // compared — but because the client already has a terminal that
        // renders exactly this shape.
        let since = std::time::Instant::now();
        let cmd = build_event(BuildOutput::Cmd("dotnet build".into()), since);
        assert_eq!(cmd["kind"], "cmd");
        assert_eq!(cmd["data"], "dotnet build");
        let out = build_event(BuildOutput::Out("Restored app.csproj".into()), since);
        assert_eq!(out["kind"], "out");
        // Lines arrive without their newline and the terminal needs one.
        assert_eq!(out["data"], "Restored app.csproj\n");
        // A note is this app talking, not the build tool, and it is indented
        // the way the spec writes it.
        let note = build_event(BuildOutput::Note("packages restore into x".into()), since);
        assert_eq!(note["data"], "  packages restore into x\n");
        let exit = build_event(BuildOutput::Exit(1), since);
        assert_eq!(exit["kind"], "exit");
        assert_eq!(exit["code"], 1);
    }

    #[test]
    fn a_document_is_named_in_the_clients_scheme() {
        // The browser never learns where the file is on disk, exactly as on
        // the language channel.
        let root = std::path::Path::new("/home/u/project");
        assert_eq!(
            resolve(root, "hick:///docs/a.hick"),
            root.join("docs/a.hick")
        );
        assert_eq!(resolve(root, "/docs/a.hick"), root.join("docs/a.hick"));
        assert_eq!(resolve(root, "a.hick"), root.join("a.hick"));
    }

    #[test]
    fn requests_parse_from_their_frame() {
        let mut frame = vec![CHANNEL_DEBUG];
        frame.extend_from_slice(
            br#"{"op":"start","doc":"hick:///a.hick","breakpoints":[{"line":7}]}"#,
        );
        let request = request_of(&frame).expect("parses");
        match request {
            Request::Start {
                doc,
                breakpoints,
                program,
            } => {
                assert_eq!(doc, "hick:///a.hick");
                assert_eq!(breakpoints[0].line, 7);
                // Optional fields are absent, not zero.
                assert!(breakpoints[0].condition.is_none());
                // No program named: the session falls back to the first
                // debuggable file, which is what a one-file document wants.
                assert!(program.is_none());
            }
            other => panic!("wrong request: {other:?}"),
        }
    }

    #[test]
    fn a_step_names_the_session_apis_own_verb() {
        let mut frame = vec![CHANNEL_DEBUG];
        frame.extend_from_slice(br#"{"op":"step","session":"dbg-0","how":"drop_frame"}"#);
        match request_of(&frame).expect("parses") {
            Request::Step { how, .. } => assert_eq!(how, Step::DropFrame),
            other => panic!("wrong request: {other:?}"),
        }
    }

    #[test]
    fn a_response_frame_is_tagged_with_the_debug_channel() {
        let frame = frame_of(&Response::Ended {
            session: "dbg-0".into(),
        });
        assert_eq!(frame[0], CHANNEL_DEBUG);
        let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
        assert_eq!(value["event"], "ended");
        assert_eq!(value["session"], "dbg-0");
    }

    #[test]
    fn a_failure_carries_words_rather_than_a_code() {
        // The UI shows this text; a code would need a second table nobody
        // would keep in step.
        let frame = frame_of(&Response::Failed {
            session: Some("dbg-1".into()),
            message: "that line is prose, not code".into(),
            about: Some("breakpoints".into()),
            lines: vec![4],
        });
        let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
        assert_eq!(value["event"], "failed");
        assert!(value["message"].as_str().unwrap().contains("prose"));
        // And says what it was about, so the app can show it on the
        // breakpoint rather than in a banner that outlives its cause.
        assert_eq!(value["about"], "breakpoints");
        assert_eq!(value["lines"][0], 4);
    }

    #[test]
    fn a_failure_about_nothing_in_particular_carries_no_lines() {
        // `about`/`lines` are omitted rather than sent empty: a UI that keys
        // on their presence should not have to know two spellings of absent.
        let frame = frame_of(&Response::Failed {
            session: None,
            message: "no such session".into(),
            about: None,
            lines: Vec::new(),
        });
        let value: Value = serde_json::from_slice(&frame[1..]).unwrap();
        assert!(value.get("about").is_none(), "{value}");
        assert!(value.get("lines").is_none(), "{value}");
    }

    #[test]
    fn the_channel_carries_no_way_to_edit_anything() {
        // The isolation guarantee, checked against the protocol itself: if an
        // edit verb ever appears here, this fails.
        let described = describe();
        let requests = described["requests"].as_array().unwrap();
        for request in requests {
            let name = request.as_str().unwrap();
            assert!(
                !name.contains("edit") && !name.contains("write") && !name.contains("save"),
                "the debug channel grew a way to write: {name}"
            );
        }
    }
}
