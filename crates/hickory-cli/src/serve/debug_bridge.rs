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

use hick_dap::BuildOutput;
use hickory_collab::CHANNEL_DEBUG;
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::debug_sessions::Registry;
#[path = "debug_sources.rs"]
mod sources;

#[path = "debug_protocol.rs"]
mod protocol;
pub use protocol::{InstallOffer, Request, Response};

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
    let doc = match &request {
        Request::Start { doc, .. } => Some(doc.clone()),
        _ => None,
    };
    // Collected out here rather than inside, because the case that needs it
    // is the failing one: a build that fails takes `handle_inner` down the
    // error arm, and anything gathered in there would go with it.
    let mut built: Vec<Value> = Vec::new();
    // Serialize state/evaluation/movement so a receipt cannot cross a resume.
    let operation_live = match if matches!(request, Request::Stop { .. }) {
        None
    } else {
        request.session_id()
    } {
        Some(id) => registry.get(id).await.ok(),
        None => None,
    };
    let _operation = match &operation_live {
        Some(live) => Some(live.operation.lock().await),
        None => None,
    };
    let result = handle_inner(registry, root, request, &mut built).await;
    let mut out = Vec::new();
    if !built.is_empty() {
        out.push(Response::Build {
            doc: doc.clone().unwrap_or_default(),
            events: built,
        });
    }
    match result {
        Ok(responses) => out.extend(responses),
        Err((session, error)) => {
            // Typed, not string-matched: `MissingAdapter` exists so this
            // decision is made on a fact rather than on the wording of a
            // sentence somebody may reword.
            let offer_install = error
                .downcast_ref::<hick_dap::MissingAdapter>()
                .filter(|missing| missing.installable)
                .map(|missing| InstallOffer {
                    kind: "dap".to_string(),
                    language: missing.language.clone(),
                });
            out.push(Response::Failed {
                doc: if session.is_none() { doc } else { None },
                session,
                message: format!("{error:#}"),
                about: Some(about.to_string()),
                lines,
                offer_install,
            })
        }
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
        Request::SourceBreakpoints { .. } => "source_breakpoints",
        Request::State { .. } => "state",
        Request::Located { .. } => "located",
        Request::Eval { .. } => "eval",
        Request::Step { .. } => "step",
        Request::Jump { .. } => "jump",
        Request::RunTo { .. } => "run_to",
        Request::Children { .. } => "children",
        Request::SetExceptionBreakpoints { .. } => "exception_breakpoints",
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
        Request::Located {
            session,
            location,
            scope,
            how,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let line = live
                .resolve_location(&location, &scope)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            *live.reading.lock().await = None;
            *live.stop_generation.lock().await += 1;
            let thread = live.thread_id.lock().await.unwrap_or(1);
            match how {
                protocol::LocatedAction::RunTo => live.session.run_to(line, thread).await,
                protocol::LocatedAction::Jump => live.session.jump_to(line, thread).await,
            }
            .map_err(|e| (Some(session.clone()), e))?;
            Ok(match how {
                protocol::LocatedAction::Jump => settle_in_place(root, &session, &live).await,
                protocol::LocatedAction::RunTo => settle(registry, root, &session, &live).await,
            })
        }
        Request::SourceBreakpoints {
            session,
            source,
            breakpoints,
        } => Ok(vec![
            sources::set(registry, root, session, source, breakpoints).await,
        ]),
        Request::Start {
            doc,
            breakpoints,
            program,
            project,
            profile,
            sources,
        } => {
            let sources = sources::initial(root, sources).map_err(|e| (None, e))?;
            let path = resolve(root, &doc);
            let endpoint = hick_lsp::ProjectEndpoint::parse(project.as_deref().unwrap_or("new"))
                .map_err(|e| (None, anyhow::Error::msg(e)))?;
            let since = std::time::Instant::now();
            let mut on_build = |line| built.push(build_event(line, since));
            // A document is woven into a scratch copy and debugged there; a
            // file that is not one is debugged as itself, in its own project,
            // with the folder the app opened as the root that bounds the
            // search for that project.
            let started = if let Some(profile) = profile {
                let context = registry
                    .launch_context(
                        root,
                        &doc,
                        project.as_deref().unwrap_or("new"),
                        program.as_deref(),
                    )
                    .map_err(|e| (None, e))?;
                registry
                    .start_profile(context, &profile, &breakpoints, &sources, &mut on_build)
                    .await
            } else if doc.starts_with("hick-project:///") {
                let uri = tower_lsp::lsp_types::Url::parse(&doc).map_err(|e| (None, e.into()))?;
                registry
                    .start_captured_with_sources(&uri, &breakpoints, &sources, &mut on_build)
                    .await
            } else if crate::debug_sessions::is_document(&path) {
                registry
                    .start_at_with_sources(
                        &path,
                        &breakpoints,
                        program.as_deref(),
                        &endpoint,
                        &sources,
                        &mut on_build,
                    )
                    .await
            } else {
                registry
                    .start_plain_with_sources(&path, root, &breakpoints, &sources, &mut on_build)
                    .await
            };
            let (session, live, statuses) = started.map_err(|e| (None, e))?;
            let mut out = vec![Response::Started {
                session: session.clone(),
                doc,
                capabilities: live.session.capabilities().clone(),
                breakpoints: statuses,
            }];
            out.extend(sources::statuses(&session, &live));
            // Run to the first stop before answering, so the UI never shows a
            // started session with no position in it.
            out.extend(settle(registry, root, &session, &live).await);
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
            Ok(position(root, &session, &live).await)
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
            let mut out = Vec::new();
            if let Some(reading) = live
                .record_expression(&expression, &value.value, frame)
                .await
                .map_err(|e| (Some(session.clone()), e))?
            {
                out.push(Response::Reading {
                    session: session.clone(),
                    reading,
                });
            }
            out.push(Response::Value {
                session,
                expression,
                value: value.value,
                type_name: value.type_name,
                reference: value.variables_reference,
            });
            Ok(out)
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
            *live.reading.lock().await = None;
            *live.stop_generation.lock().await += 1;
            live.session
                .step(how, thread, frame)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(settle(registry, root, &session, &live).await)
        }

        Request::Jump { session, line } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let thread = live.thread_id.lock().await.unwrap_or(1);
            *live.reading.lock().await = None;
            *live.stop_generation.lock().await += 1;
            live.session
                .jump_to(line, thread)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(settle_in_place(root, &session, &live).await)
        }

        Request::RunTo { session, line } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let thread = live.thread_id.lock().await.unwrap_or(1);
            *live.reading.lock().await = None;
            *live.stop_generation.lock().await += 1;
            live.session
                .run_to(line, thread)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(settle(registry, root, &session, &live).await)
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
            Ok(position(root, &session, &live).await)
        }

        Request::SetExceptionBreakpoints { session, filters } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            live.session
                .set_exception_breakpoints(&filters)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(Vec::new())
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
    root: &std::path::Path,
    session: &str,
    live: &Arc<crate::debug_sessions::Live>,
) -> Vec<Response> {
    match live.session.wait_for_stop(Duration::from_secs(60)).await {
        Ok(Some(stopped)) => {
            *live.thread_id.lock().await = Some(stopped.thread_id);
            *live.stopped.lock().await = Some(stopped.clone());
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
            out.extend(sources::statuses(session, live));
            out.extend(position_with(root, session, live, &stopped.reason).await);
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
            doc: None,
            message: format!("{error:#}"),
            about: None,
            lines: Vec::new(),
            offer_install: None,
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
async fn settle_in_place(
    root: &std::path::Path,
    session: &str,
    live: &Arc<crate::debug_sessions::Live>,
) -> Vec<Response> {
    if let Ok(Some(stopped)) = live.session.wait_for_stop(Duration::from_millis(400)).await {
        *live.thread_id.lock().await = Some(stopped.thread_id);
        *live.stopped.lock().await = Some(stopped.clone());
        return position_with(root, session, live, &stopped.reason).await;
    }
    position_with(root, session, live, "goto").await
}

async fn position(
    root: &std::path::Path,
    session: &str,
    live: &Arc<crate::debug_sessions::Live>,
) -> Vec<Response> {
    position_with(root, session, live, "state").await
}

/// A frame's source, in the client's own scheme when it is a file in the
/// folder the app opened.
///
/// The adapter names a frame's file the way it was compiled or launched —
/// an absolute path. A frame outside the file being debugged but inside the
/// project (`src/lib.rs` under a plain `src/main.rs`) is one the app can
/// open as a tab, and the tab is addressed by the root-relative path every
/// other part of the app uses. A path outside the root — a document's
/// scratch copy, the standard library — is left as the adapter said it.
pub fn relative_source(root: &std::path::Path, frame: hick_dap::Frame) -> hick_dap::Frame {
    let Some(source) = frame.source.as_deref() else {
        return frame;
    };
    let path = std::path::Path::new(source);
    let relative: Option<PathBuf> = match path.strip_prefix(root) {
        Ok(rel) => Some(rel.to_path_buf()),
        Err(_) => match (root.canonicalize(), path.canonicalize()) {
            (Ok(root), Ok(path)) => path.strip_prefix(&root).ok().map(|p| p.to_path_buf()),
            _ => None,
        },
    };
    match relative {
        Some(relative) => hick_dap::Frame {
            source: Some(relative.to_string_lossy().replace('\\', "/")),
            ..frame
        },
        None => frame,
    }
}

/// The stack, the top frame's variables, and the line the gutter marks.
async fn position_with(
    root: &std::path::Path,
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
    let frames: Vec<hick_dap::Frame> = live
        .session
        .stack(thread)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|frame| relative_source(root, frame))
        .collect();
    let variables = match frames.first() {
        Some(frame) => live.session.variables(frame.id).await.unwrap_or_default(),
        None => Vec::new(),
    };
    let mut out = Vec::new();
    if let Ok(Some(reading)) = live
        .capture_reading(
            session,
            reason,
            &frames,
            &variables,
            frames.first().map(|f| f.id),
        )
        .await
    {
        out.push(Response::Reading {
            session: session.into(),
            reading,
        });
    }
    let observation_scope = out.iter().find_map(|r| match r {
        Response::Reading { reading, .. } => Some(reading.scope.clone()),
        _ => None,
    });
    out.push(Response::Stopped {
        observation_scope,
        session: session.to_string(),
        reason: reason.to_string(),
        line: frames.first().and_then(|frame| frame.line),
        frames,
        variables,
    });
    out
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
            "children", "exception_breakpoints", "set_variable", "stop", "located"
        ],
        "responses": [
            "started", "stopped", "breakpoints", "value", "children",
            "finished", "ended", "failed", "build", "reading"
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_dap::Step;

    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn agent_action_fixture_uses_opaque_handles_without_display_line_arithmetic() {
        let reading: hick_observe::reading::Reading = serde_json::from_str(include_str!(
            "../../../../apps/web/src/debug/fixtures/reading.json"
        ))
        .unwrap();
        let action = json!({"op":"located", "session":"debug-fixture", "location":reading.locations[0].id, "scope":reading.scope, "how":"jump"});
        assert!(action.get("line").is_none());
        let request: Request = serde_json::from_value(action).unwrap();
        match request {
            Request::Located {
                location, scope, ..
            } => {
                assert_eq!(
                    reading
                        .resolve(&location, &scope, &reading.source.bytes, Some(7))
                        .unwrap(),
                    1
                );
                let mut stale = scope;
                stale.stop = "old-stop".into();
                assert!(
                    reading
                        .resolve(&location, &stale, &reading.source.bytes, Some(7))
                        .is_err()
                );
            }
            _ => panic!("expected located action"),
        }
    }

    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    async fn real_adapter_relocation_and_stale_actions_share_the_reading() {
        use hick_observe::reading::DebugAnnotation as A;
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
        if cache.join("adapters/python/bin/python3").exists() {
            std::os::unix::fs::symlink(cache, dir.path().join(".hick-cache")).unwrap();
        }
        if hick_dap::discover("python", dir.path()).is_none() {
            eprintln!("SKIPPED: install Python adapter with hick dap install python");
            return;
        }
        let source = "# Reading\n<hick:file path=\"main.py\">\nvalue = 1\n\n\ndef count(n):\n    return count(n - 1) if n else n\nprint(count(2))\n</hick:file>\n";
        let path = dir.path().join("demo.md");
        std::fs::write(&path, source).unwrap();
        let registry = Arc::new(Registry::new());
        let out = handle(
            &registry,
            dir.path(),
            Request::Start {
                doc: "hick:///demo.md".into(),
                breakpoints: vec![hick_dap::Breakpoint {
                    line: 3,
                    condition: None,
                    hit_condition: None,
                    log_message: None,
                }],
                program: Some("main.py".into()),
                project: None,
                profile: None,
                sources: Default::default(),
            },
        )
        .await;
        let (session, reading) = out
            .iter()
            .find_map(|r| match r {
                Response::Reading { session, reading } => Some((session.clone(), reading.clone())),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing reading: {out:?}"));
        let requested = reading
            .annotations
            .iter()
            .find(|a| matches!(a.payload, A::BreakpointRequested { .. }))
            .unwrap();
        let bound = reading
            .annotations
            .iter()
            .find(|a| matches!(a.payload, A::BreakpointBound { .. }))
            .unwrap();
        let paused = reading
            .annotations
            .iter()
            .find(|a| matches!(a.payload, A::Paused { .. }))
            .unwrap();
        assert_ne!(requested.location, bound.location);
        assert_eq!(bound.location, paused.location);
        assert_eq!(reading.source.bytes, source.as_bytes());
        let location = paused.location.clone().unwrap();
        let mut wrong_frame = reading.scope.clone();
        wrong_frame.frame = Some(-999);
        assert!(
            registry
                .get(&session)
                .await
                .unwrap()
                .resolve_location(&location, &wrong_frame)
                .await
                .is_err()
        );
        std::fs::write(&path, source.replace("value = 1", "value = 10")).unwrap();
        let changed = handle(
            &registry,
            dir.path(),
            Request::Located {
                session: session.clone(),
                location: location.clone(),
                scope: reading.scope.clone(),
                how: protocol::LocatedAction::Jump,
            },
        )
        .await;
        assert!(changed.iter().any(
            |r| matches!(r, Response::Failed { message, .. } if message.contains("source changed"))
        ));
        std::fs::write(&path, source).unwrap();
        let _ = handle(
            &registry,
            dir.path(),
            Request::Step {
                session: session.clone(),
                how: hick_dap::Step::Over,
                frame: None,
            },
        )
        .await;
        let stale = handle(
            &registry,
            dir.path(),
            Request::Located {
                session: session.clone(),
                location,
                scope: reading.scope,
                how: protocol::LocatedAction::RunTo,
            },
        )
        .await;
        assert!(stale.iter().any(|r| matches!(r, Response::Failed { .. })));
        registry.reap(&session).await;
    }

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
            dir.path().join("a.md"),
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
                doc: "hick:///a.md".into(),
                breakpoints: vec![],
                program: None,
                project: None,
                profile: None,
                sources: Default::default(),
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
        assert_eq!(resolve(root, "hick:///docs/a.md"), root.join("docs/a.md"));
        assert_eq!(resolve(root, "/docs/a.md"), root.join("docs/a.md"));
        assert_eq!(resolve(root, "a.md"), root.join("a.md"));
    }

    #[test]
    fn requests_parse_from_their_frame() {
        let mut frame = vec![CHANNEL_DEBUG];
        frame.extend_from_slice(
            br#"{"op":"start","doc":"hick:///a.md","breakpoints":[{"line":7}]}"#,
        );
        let request = request_of(&frame).expect("parses");
        match request {
            Request::Start {
                doc,
                breakpoints,
                program,
                project,
                ..
            } => {
                assert!(project.is_none());
                assert_eq!(doc, "hick:///a.md");
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
            doc: None,
            session: Some("dbg-1".into()),
            message: "that line is prose, not code".into(),
            about: Some("breakpoints".into()),
            lines: vec![4],
            offer_install: None,
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
            doc: None,
            session: None,
            message: "no such session".into(),
            about: None,
            lines: Vec::new(),
            offer_install: None,
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
