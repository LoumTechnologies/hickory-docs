<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/debug_bridge.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b6fc115ecefd"></a>
## `crates/hickory-cli/src/serve/debug_bridge.rs`

Related contracts: [A Breakpoint That Is Not Bound Yet Is Not Refused](../../../../../guarantees/debugging/a-breakpoint-that-is-not-bound-yet-is-not-refused/contract.md), [Launch the artifact built from the selected compiled project](../../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [Install a supported missing debugger and resume the requested launch](../../../../../guarantees/debugging/a-missing-debugger-is-a-button/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [An Exception Breakpoint Uses The Adapter's Own Filters](../../../../../guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters/contract.md), [Debugging A Document Never Writes To The Project](../../../../../guarantees/debugging/debugging-never-writes-to-the-project/contract.md), [Observations read beside their captured source](../../../../../guarantees/debugging/source-anchored-observation-reading/contract.md).

<a id="f-b6fc115ecefd-sources"></a>
### `sources`

The app's debugger, on channel `0x03`.
## Why this is not raw DAP
The spec sketched this as "DAP messages the way `0x02` carries LSP ones".
Building it showed that to be the wrong shape, for one reason: DAP speaks
in the coordinates of the file the debugger is running, and the app speaks
in the coordinates of the document a person is reading. Proxying raw DAP
would put that mapping in the browser — a second implementation of the one
thing the session API exists to get right, in the one place least able to
test it.
So the channel carries the session API's own verbs, and the mapping stays
in Rust with the code that already knows how to do it. The client is a
debugger UI, not a debug adapter client.
## What it may not do
Write. There is no edit operation on this channel — document edits go
through the CRDT room and the hashline API, neither of which this holds —
and a session's debuggee runs in a scratch copy that is deleted with it.
That is what makes "no amount of stepping can change the file" a fact
about the plumbing rather than a promise about behaviour.

<storycode:copy id="f-b6fc115ecefd-sources">
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

</storycode:copy>
<a id="f-b6fc115ecefd-protocol"></a>
### `protocol`

<storycode:copy id="f-b6fc115ecefd-protocol">
#[path = "debug_protocol.rs"]
mod protocol;
</storycode:copy>
<a id="f-b6fc115ecefd-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-b6fc115ecefd-use_declaration">
pub use protocol::{InstallOffer, Request, Response};

</storycode:copy>
<a id="f-b6fc115ecefd-handle"></a>
### `handle`

Handle one request and produce the responses to send back.
Returns several because one action often has two answers: a step both
moves and lands somewhere, and the app wants the landing.

<storycode:copy id="f-b6fc115ecefd-handle">
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

</storycode:copy>
<a id="f-b6fc115ecefd-build_event"></a>
### `build_event`

One build line, in the shape the client's terminal already reads.
`t` is milliseconds since the build started — the same meaning `t` has on
a cell's transcript, so the same component orders them the same way.

<storycode:copy id="f-b6fc115ecefd-build_event">
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

</storycode:copy>
<a id="f-b6fc115ecefd-about_of"></a>
### `about_of`

The name of the request, for a failure to point at.

<storycode:copy id="f-b6fc115ecefd-about_of">
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

</storycode:copy>
<a id="f-b6fc115ecefd-lines_of"></a>
### `lines_of`

The document lines a request is about, when it is about lines.

<storycode:copy id="f-b6fc115ecefd-lines_of">
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

</storycode:copy>
<a id="f-b6fc115ecefd-Failure"></a>
### `Failure`

<storycode:copy id="f-b6fc115ecefd-Failure">
type Failure = (Option<String>, anyhow::Error);

</storycode:copy>
<a id="f-b6fc115ecefd-handle_inner"></a>
### `handle_inner`

<storycode:copy id="f-b6fc115ecefd-handle_inner">
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

</storycode:copy>
<a id="f-b6fc115ecefd-settle"></a>
### `settle`

Wait for the program to stop again, then describe where it is.
When the answer is "it ended", the session is reaped HERE, before the
client is told: a debuggee that ran to completion must not leave its
adapter process, scratch clone and registry entry waiting fifteen minutes
for the idle sweep. Reaping is idempotent with the explicit stop that may
arrive a beat later.

<storycode:copy id="f-b6fc115ecefd-settle">
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

</storycode:copy>
<a id="f-b6fc115ecefd-settle_in_place"></a>
### `settle_in_place`

Answer a move that leaves the program *already stopped*.
A jump moves the instruction pointer without resuming, and the protocol
says the adapter then reports a `stopped` with reason `goto`. debugpy does
not, so waiting for one is waiting for the timeout — and the app would sit
with a stale paused line for a minute over a move that already happened.
A short grace period picks up the event where an adapter does send it, and
otherwise the answer is simply where we are now.

<storycode:copy id="f-b6fc115ecefd-settle_in_place">
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

</storycode:copy>
<a id="f-b6fc115ecefd-position"></a>
### `position`

<storycode:copy id="f-b6fc115ecefd-position">
async fn position(
    root: &std::path::Path,
    session: &str,
    live: &Arc<crate::debug_sessions::Live>,
) -> Vec<Response> {
    position_with(root, session, live, "state").await
}

</storycode:copy>
<a id="f-b6fc115ecefd-relative_source"></a>
### `relative_source`

A frame's source, in the client's own scheme when it is a file in the
folder the app opened.
The adapter names a frame's file the way it was compiled or launched —
an absolute path. A frame outside the file being debugged but inside the
project (`src/lib.rs` under a plain `src/main.rs`) is one the app can
open as a tab, and the tab is addressed by the root-relative path every
other part of the app uses. A path outside the root — a document's
scratch copy, the standard library — is left as the adapter said it.

<storycode:copy id="f-b6fc115ecefd-relative_source">
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

</storycode:copy>
<a id="f-b6fc115ecefd-position_with"></a>
### `position_with`

The stack, the top frame's variables, and the line the gutter marks.

<storycode:copy id="f-b6fc115ecefd-position_with">
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

</storycode:copy>
<a id="f-b6fc115ecefd-top_frame"></a>
### `top_frame`

<storycode:copy id="f-b6fc115ecefd-top_frame">
async fn top_frame(live: &Arc<crate::debug_sessions::Live>) -> Option<i64> {
    let thread = (*live.thread_id.lock().await)?;
    live.session
        .stack(thread)
        .await
        .ok()?
        .first()
        .map(|frame| frame.id)
}

</storycode:copy>
<a id="f-b6fc115ecefd-resolve"></a>
### `resolve`

The client names a document in its own scheme; this is where it lives.

<storycode:copy id="f-b6fc115ecefd-resolve">
/// The client names a document in its own scheme; this is where it lives.
fn resolve(root: &std::path::Path, doc: &str) -> PathBuf {
    let relative = doc
        .strip_prefix("hick:///")
        .unwrap_or_else(|| doc.trim_start_matches('/'));
    root.join(relative)
}

</storycode:copy>
<a id="f-b6fc115ecefd-frame_of"></a>
### `frame_of`

<storycode:copy id="f-b6fc115ecefd-frame_of">
/// Send one response as a `0x03` frame.
pub fn frame_of(response: &Response) -> Vec<u8> {
    let mut frame = vec![CHANNEL_DEBUG];
    let body = serde_json::to_vec(response).unwrap_or_else(|_| b"{}".to_vec());
    frame.extend_from_slice(&body);
    frame
}

</storycode:copy>
<a id="f-b6fc115ecefd-request_of"></a>
### `request_of`

<storycode:copy id="f-b6fc115ecefd-request_of">
/// Parse a `0x03` frame from the client.
pub fn request_of(data: &[u8]) -> Result<Request, serde_json::Error> {
    serde_json::from_slice(&data[1..])
}

</storycode:copy>
<a id="f-b6fc115ecefd-reply"></a>
### `reply`

Send responses to the client, dropping them if it has gone.

<storycode:copy id="f-b6fc115ecefd-reply">
/// Send responses to the client, dropping them if it has gone.
pub fn reply(tx: &mpsc::UnboundedSender<Vec<u8>>, responses: Vec<Response>) {
    for response in responses {
        if tx.send(frame_of(&response)).is_err() {
            return;
        }
    }
}

</storycode:copy>
<a id="f-b6fc115ecefd-describe"></a>
### `describe`

The shape the client sees, for its own tests.

<storycode:copy id="f-b6fc115ecefd-describe">
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

</storycode:copy>
<a id="f-b6fc115ecefd-tests"></a>
### `tests`

<storycode:copy id="f-b6fc115ecefd-tests">
#[cfg(test)]
#[path = "debug_bridge/tests/tests.rs"]
mod tests;
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/debug_bridge.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/debug_bridge.rs"><storycode:paste select="#f-b6fc115ecefd-sources"/><storycode:paste select="#f-b6fc115ecefd-protocol"/><storycode:paste select="#f-b6fc115ecefd-use_declaration"/><storycode:paste select="#f-b6fc115ecefd-handle"/><storycode:paste select="#f-b6fc115ecefd-build_event"/><storycode:paste select="#f-b6fc115ecefd-about_of"/><storycode:paste select="#f-b6fc115ecefd-lines_of"/><storycode:paste select="#f-b6fc115ecefd-Failure"/><storycode:paste select="#f-b6fc115ecefd-handle_inner"/><storycode:paste select="#f-b6fc115ecefd-settle"/><storycode:paste select="#f-b6fc115ecefd-settle_in_place"/><storycode:paste select="#f-b6fc115ecefd-position"/><storycode:paste select="#f-b6fc115ecefd-relative_source"/><storycode:paste select="#f-b6fc115ecefd-position_with"/><storycode:paste select="#f-b6fc115ecefd-top_frame"/><storycode:paste select="#f-b6fc115ecefd-resolve"/><storycode:paste select="#f-b6fc115ecefd-frame_of"/><storycode:paste select="#f-b6fc115ecefd-request_of"/><storycode:paste select="#f-b6fc115ecefd-reply"/><storycode:paste select="#f-b6fc115ecefd-describe"/><storycode:paste select="#f-b6fc115ecefd-tests"/></storycode:file>

</storycode:doc>
