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

use hick_dap::{Breakpoint, Step};
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
    Finished {
        session: String,
    },
    Ended {
        session: String,
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
    match handle_inner(registry, root, request).await {
        Ok(responses) => responses,
        Err((session, error)) => vec![Response::Failed {
            session,
            message: format!("{error:#}"),
            about: Some(about.to_string()),
            lines,
        }],
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
) -> Result<Vec<Response>, Failure> {
    match request {
        Request::Start {
            doc,
            breakpoints,
            program,
        } => {
            let path = resolve(root, &doc);
            let (session, live, statuses) = registry
                .start(&path, &breakpoints, program.as_deref())
                .await
                .map_err(|e| (None, e))?;
            let mut out = vec![Response::Started {
                session: session.clone(),
                capabilities: live.session.capabilities().clone(),
                breakpoints: statuses,
            }];
            // Run to the first stop before answering, so the UI never shows a
            // started session with no position in it.
            out.extend(settle(&session, &live).await);
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
            Ok(settle(&session, &live).await)
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
            Ok(settle(&session, &live).await)
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
            registry
                .stop(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(vec![Response::Ended { session }])
        }
    }
}

/// Wait for the program to stop again, then describe where it is.
async fn settle(session: &str, live: &Arc<crate::debug_sessions::Live>) -> Vec<Response> {
    match live.session.wait_for_stop(Duration::from_secs(60)).await {
        Ok(Some(stopped)) => {
            *live.thread_id.lock().await = Some(stopped.thread_id);
            position_with(session, live, &stopped.reason).await
        }
        Ok(None) => {
            *live.thread_id.lock().await = None;
            vec![Response::Finished {
                session: session.to_string(),
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
            "finished", "ended", "failed"
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
