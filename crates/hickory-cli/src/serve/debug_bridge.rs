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

use anyhow::Context;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use hick_dap::BuildOutput;
use hickory_collab::CHANNEL_DEBUG;
use serde_json::{Value, json};
use tokio::sync::mpsc;

use crate::debug_sessions::Registry;
#[path = "debug_protocol.rs"]
mod protocol;
#[path = "debug_sources.rs"]
mod sources;
#[path = "debug_adapter_sources.rs"]
mod adapter_sources;
pub use protocol::{InstallOffer, Request, Response};
/// Handle one request and produce the responses to send back.
///
/// Returns several because one action often has two answers: a step both
/// moves and lands somewhere, and the app wants the landing.
#[cfg(test)]
pub async fn handle(
    registry: &Arc<Registry>,
    root: &std::path::Path,
    request: Request,
) -> Vec<Response> {
    handle_with(registry, root, request, |_| {}).await
}

/// Queue a complete response group before releasing the session operation lock.
/// Sockets spawn independent tasks; serial execution alone cannot order replies
/// if an old task is descheduled between returning and enqueueing its answer.
pub async fn dispatch(
    registry: &Arc<Registry>,
    root: &std::path::Path,
    request: Request,
    tx: &mpsc::UnboundedSender<Vec<u8>>,
) {
    let _ = handle_with(registry, root, request, |responses| reply(tx, responses)).await;
}

enum LaunchProgress<'a> {
    Preparing(Response),
    Started(&'a [Response]),
}

// The Rust harness can spend up to 120 seconds in its bounded Cargo and
// metadata phase. Adapter TCP connection and the DAP initialized event each
// allow up to 30 seconds. This permits a bounded client wait. It does not put
// a new cancellation deadline on DAP operations already sent to the adapter.
const TEST_PREPARATION_WAIT_MS: u32 = 180_000;

async fn handle_with(
    registry: &Arc<Registry>,
    root: &std::path::Path,
    request: Request,
    mut deliver: impl FnMut(&[Response]) + Send,
) -> Vec<Response> {
    let output_id = request.session_id().map(str::to_owned);
    let about = about_of(&request);
    let request_id = match &request {
        Request::StartTest(value) => Some(value.request_id.clone()),
        Request::Source(value) => Some(value.request_id.clone()),
        Request::Start { request_id, .. }
        | Request::Threads { request_id, .. }
        | Request::SelectThread { request_id, .. }
        | Request::State { request_id, .. }
        | Request::Eval { request_id, .. }
        | Request::Children { request_id, .. }
        | Request::Breakpoints { request_id, .. }
        | Request::SourceBreakpoints { request_id, .. }
        | Request::Step { request_id, .. }
        | Request::Jump { request_id, .. }
        | Request::RunTo { request_id, .. }
        | Request::SetVariable { request_id, .. } => request_id.clone(),
        _ => None,
    };
    let lines = lines_of(&request);
    let doc = match &request {
        Request::Start { doc, .. } => Some(doc.clone()),
        Request::StartTest(value) => Some(value.doc.clone()),
        _ => None,
    };
    // Collected out here rather than inside, because the case that needs it
    // is the failing one: a build that fails takes `handle_inner` down the
    // error arm, and anything gathered in there would go with it.
    let mut built: Vec<Value> = Vec::new();
    // Serialize state/evaluation/movement so a receipt cannot cross a resume.
    let operation_live = match request.session_id() {
        Some(id) => registry.get(id).await.ok(),
        None => None,
    };
    let _operation = match &operation_live {
        Some(_) if matches!(request, Request::Stop { .. } | Request::Source(_)) => None,
        Some(live)
            if matches!(
                request,
                Request::Breakpoints { .. }
                    | Request::SourceBreakpoints { .. }
                    | Request::SetExceptionBreakpoints { .. }
            ) =>
        {
            Some(live.breakpoint_operation.lock().await)
        }
        Some(live) => Some(live.operation.lock().await),
        None => None,
    };
    let mut started_live = None;
    let mut started_operation = None;
    let mut delivered_prefix = 0;
    let mut preparation = None;
    let mut progress = |update: LaunchProgress<'_>| match update {
        LaunchProgress::Preparing(response) => {
            deliver(std::slice::from_ref(&response));
            preparation = Some(response);
            delivered_prefix += 1;
        }
        LaunchProgress::Started(responses) => {
            deliver(responses);
            delivered_prefix += responses.len();
        }
    };
    let result = handle_inner(
        registry,
        root,
        request,
        &mut built,
        &mut started_live,
        &mut started_operation,
        &mut progress,
    )
    .await;
    drop(progress);
    let mut out: Vec<_> = preparation.into_iter().collect();
    // A success already moved its entire Build/Started batch into the live
    // prefix. Remaining build messages precede only the terminal failure.
    if !built.is_empty() {
        out.push(Response::Build {
            request_id: request_id.clone(),
            doc: doc.clone().unwrap_or_default(),
            events: built,
        });
    }
    match result {
        Ok(mut responses) => {
            for response in &mut responses {
                if let Response::Failed {
                    request_id: id,
                    about: operation,
                    ..
                } = response
                {
                    if id.is_none() {
                        *id = request_id.clone();
                    }
                    if operation.is_none() {
                        *operation = Some(about.to_owned());
                    }
                }
            }
            out.extend(responses);
        }
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
                scope_retired: false,
                request_id,
                doc: if session.is_none() { doc } else { None },
                session,
                message: format!("{error:#}"),
                about: Some(about.to_string()),
                lines,
                offer_install,
            })
        }
    }
    // The operation retains its Live even when Finished reaps the registry.
    // Send output before Finished, so clients still recognize its session.
    let started = out.iter().find_map(|response| match response {
        Response::Started { session, .. } => Some(session.clone()),
        _ => None,
    });
    // A program can finish during Start itself; retain that Live across reaping.
    let output_live = operation_live.as_ref().cloned().or(started_live);
    if let Some(live) = output_live {
        let events = live
            .session
            .take_output()
            .into_iter()
            .map(|mut event| {
                if let Some(path) = event.get("source").and_then(Value::as_str) {
                    if let Ok(relative) = std::path::Path::new(path).strip_prefix(root) {
                        event["source"] = json!(relative.to_string_lossy());
                    }
                }
                event
            })
            .collect::<Vec<_>>();
        if !events.is_empty() {
            let session = started.or(output_id).or_else(|| {
                out.iter().find_map(|response| match response {
                    Response::Stopped { session, .. }
                    | Response::Finished { session, .. }
                    | Response::Ended { session } => Some(session.clone()),
                    _ => None,
                })
            });
            if let Some(session) = session {
                let at = out
                    .iter()
                    .position(|r| matches!(r, Response::Finished { .. } | Response::Ended { .. }))
                    .unwrap_or(out.len());
                out.insert(at, Response::RunOutput { session, events });
            }
        }
    }
    deliver(&out[delivered_prefix..]);
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
        Request::Start { .. } | Request::StartTest(_) => "start",
        Request::Breakpoints { .. } => "breakpoints",
        Request::SourceBreakpoints { .. } => "source_breakpoints",
        Request::State { .. } => "state",
        Request::Threads { .. } => "threads",
        Request::Source(_) => "source",
        Request::SelectThread { .. } => "select_thread",
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
        Request::StartTest(value) => value.breakpoints.iter().map(|bp| bp.line).collect(),
        Request::Start { breakpoints, .. } | Request::Breakpoints { breakpoints, .. } => {
            breakpoints.iter().map(|b| b.line).collect()
        }
        Request::Jump { line, .. } | Request::RunTo { line, .. } => vec![*line],
        _ => Vec::new(),
    }
}
type Failure = (Option<String>, anyhow::Error);
fn intent_ticket(value: Option<&str>) -> anyhow::Result<&str> {
    let value = value.context(
        "Debugger request identity is missing. Refresh this debugger with a compatible client.",
    )?;
    anyhow::ensure!(
        !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control),
        "Debugger request identity is invalid. Refresh its controls and retry."
    );
    Ok(value)
}
async fn observed_scope(
    live: &crate::debug_sessions::Live,
    stop: Option<u64>,
    thread: Option<i64>,
    request_id: Option<&str>,
) -> anyhow::Result<(u64, i64)> {
    intent_ticket(request_id)?;
    let actual_stop = *live.stop_generation.lock().await;
    let actual_thread = (*live.thread_id.lock().await)
        .context("The debugger has no stopped thread. Pause it before this operation.")?;
    anyhow::ensure!(
        live.stopped.lock().await.is_some()
            && stop == Some(actual_stop)
            && thread == Some(actual_thread),
        "Debugger observation changed. Read the current stopped frame before trying again."
    );
    Ok((actual_stop, actual_thread))
}
async fn observed_frame(
    live: &crate::debug_sessions::Live,
    thread: i64,
    frame: Option<i64>,
) -> anyhow::Result<i64> {
    let frame = frame.context("Select a current stopped frame before this operation.")?;
    anyhow::ensure!(
        live.session
            .stack(thread)
            .await?
            .iter()
            .any(|entry| entry.id == frame),
        "Frame does not belong to the current stopped thread. Read its stack again."
    );
    Ok(frame)
}
async fn retire_values(live: &crate::debug_sessions::Live) {
    live.adapter_sources.lock().await.clear();
    *live.reading.lock().await = None;
    live.value_references.lock().await.clear();
    *live.stop_generation.lock().await += 1;
}
async fn handle_inner(
    registry: &Arc<Registry>,
    root: &std::path::Path,
    request: Request,
    built: &mut Vec<Value>,
    started_live: &mut Option<Arc<crate::debug_sessions::Live>>,
    started_operation: &mut Option<tokio::sync::OwnedMutexGuard<()>>,
    progress: &mut (impl FnMut(LaunchProgress<'_>) + Send),
) -> Result<Vec<Response>, Failure> {
    let (request, test_launch) = match request {
        Request::StartTest(value) => (
            Request::Start {
                doc: value.doc,
                request_id: Some(value.request_id),
                breakpoints: value.breakpoints,
                program: None,
                project: None,
                profile: None,
                sources: value.sources,
                source_hash: Some(value.source_hash),
                source_hashes: value.source_hashes,
                source_test: None,
            },
            Some(value.source_test),
        ),
        Request::Start {
            source_test: Some(_),
            ..
        } => {
            return Err((
                None,
                anyhow::anyhow!(
                    "Source tests require the start_test operation; ordinary Debug fallback is disabled"
                ),
            ));
        }
        other => (other, None),
    };
    match request {
        Request::StartTest(_) => unreachable!("Test launch was normalized above"),
        Request::Source(request) => adapter_sources::read(registry, request).await,
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
            let thread = (*live.thread_id.lock().await)
                .context("Pause the debugger before a located action")
                .map_err(|e| (Some(session.clone()), e))?;
            match how {
                protocol::LocatedAction::RunTo => live.session.run_to(line, thread).await,
                protocol::LocatedAction::Jump => live.session.jump_to(line, thread).await,
            }
            .map_err(|e| (Some(session.clone()), e))?;
            retire_values(&live).await;
            if matches!(how, protocol::LocatedAction::RunTo) {
                *live.stopped.lock().await = None;
            }
            Ok(match how {
                protocol::LocatedAction::Jump => settle_in_place(root, &session, &live).await,
                protocol::LocatedAction::RunTo => settle(registry, root, &session, &live).await,
            })
        }
        Request::SourceBreakpoints {
            session,
            source,
            breakpoints,
            source_hash,
            request_id,
        } => Ok(vec![
            sources::set(
                registry,
                root,
                session,
                source,
                breakpoints,
                source_hash,
                request_id,
            )
            .await,
        ]),
        Request::Start {
            doc,
            request_id,
            breakpoints,
            program,
            project,
            profile,
            sources,
            source_hash,
            source_hashes,
            source_test: _,
        } => {
            let launch_request = intent_ticket(request_id.as_deref())
                .map_err(|e| (None, e))?;
            let primary_hash = sources::valid_hash(source_hash.as_deref())
                .map_err(|e| (None, e))?
                .to_owned();
            let mut sources =
                sources::initial(root, sources, source_hashes).map_err(|e| (None, e))?;
            let path = resolve(root, &doc).map_err(|e| (None, e))?;
            if !doc.starts_with("hick-project:///") {
                if super::source_tests::digest(&std::fs::read(&path).map_err(|e| (None, e.into()))?)
                    != primary_hash
                {
                    return Err((
                        None,
                        anyhow::anyhow!(
                            "Displayed launch source changed. Save or reload it before Debug."
                        ),
                    ));
                }
            }
            sources.push(hick_dap::SourceBreakpoints {
                source: String::new(),
                document: path.clone(),
                source_hash: Some(primary_hash),
                breakpoints: breakpoints.clone(),
            });
            let endpoint = hick_lsp::ProjectEndpoint::parse(project.as_deref().unwrap_or("new"))
                .map_err(|e| (None, anyhow::Error::msg(e)))?;
            let since = std::time::Instant::now();
            let mut on_build = |line| built.push(build_event(line, since));
            // A document is woven into a scratch copy and debugged there; a
            // file that is not one is debugged as itself, in its own project,
            // with the folder the app opened as the root that bounds the
            // search for that project.
            let mut test_binding = None;
            let started = if let Some(request) = test_launch {
                let workspace = root.to_path_buf();
                let admitted = tokio::task::spawn_blocking(move || {
                    super::test_debug::admit(&workspace, &request)
                })
                .await
                .map_err(|error| (None, anyhow::anyhow!("Test admission failed: {error}")))?
                .map_err(|error| (None, anyhow::anyhow!("{}", error.detail())))?;
                if !admitted.plan.available {
                    return Err((
                        None,
                        anyhow::anyhow!(
                            "{}",
                            admitted
                                .plan
                                .reason
                                .as_deref()
                                .unwrap_or("Test Debug is unavailable")
                        ),
                    ));
                }
                if admitted.plan.source_uri != doc
                    || admitted.plan.source_hash != source_hash.as_deref().unwrap_or("")
                {
                    return Err((
                        None,
                        anyhow::anyhow!(
                            "Test debug input identity changed. Refresh source tests before launching."
                        ),
                    ));
                }
                progress(LaunchProgress::Preparing(Response::TestPreparing {
                    request_id: launch_request.to_owned(),
                    doc: doc.clone(),
                    wait_ms: TEST_PREPARATION_WAIT_MS,
                    source_test: admitted.binding.clone(),
                }));
                test_binding = Some(admitted.binding);
                registry
                    .start_test_with_sources(admitted.launch, &sources, &mut on_build)
                    .await
            } else if let Some(profile) = profile {
                let context = registry
                    .launch_context(
                        root,
                        &doc,
                        project.as_deref().unwrap_or("new"),
                        program.as_deref(),
                    )
                    .map_err(|e| (None, e))?;
                registry
                    .start_profile(context, &profile, &[], &sources, &mut on_build)
                    .await
            } else if doc.starts_with("hick-project:///") {
                let uri = tower_lsp::lsp_types::Url::parse(&doc).map_err(|e| (None, e.into()))?;
                registry
                    .start_captured_with_sources(&uri, &[], &sources, &mut on_build)
                    .await
            } else if crate::debug_sessions::is_document(&path) {
                registry
                    .start_at_with_sources(
                        &path,
                        &[],
                        program.as_deref(),
                        &endpoint,
                        &sources,
                        &mut on_build,
                    )
                    .await
            } else {
                registry
                    .start_plain_with_sources(&path, root, &[], &sources, &mut on_build)
                    .await
            };
            let (session, live, statuses) = started.map_err(|e| (None, e))?;
            {
                let mut intents = live.breakpoint_intents.lock().unwrap();
                for set in &sources {
                    intents.insert(
                        set.source.clone(),
                        crate::debug_sessions::BreakpointIntent {
                            request_id: request_id.clone().unwrap(),
                            source_hash: set.source_hash.clone().unwrap(),
                        },
                    );
                }
            }
            *started_live = Some(live.clone());
            *started_operation = Some(live.operation.clone().lock_owned().await);
            let mut out = Vec::new();
            if !built.is_empty() {
                out.push(Response::Build {
                    request_id: request_id.clone(),
                    doc: doc.clone(),
                    events: std::mem::take(built),
                });
            }
            out.push(Response::Started {
                request_id,
                session: session.clone(),
                doc,
                capabilities: live.session.capabilities().clone(),
                source_hash: live.session.primary_source_hash(),
                source_test: test_binding,
                breakpoints: statuses,
            });
            out.extend(sources::statuses(&session, &live));
            // Expose the real session immediately, so Stop and live breakpoint
            // updates work while a program runs towards its first stop.
            progress(LaunchProgress::Started(&out));
            out.extend(settle(registry, root, &session, &live).await);
            Ok(out)
        }

        Request::Breakpoints {
            session,
            breakpoints,
            source_hash,
            request_id,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let ticket = intent_ticket(request_id.as_deref())
                .map_err(|e| (Some(session.clone()), e))?
                .to_owned();
            let lines = breakpoints
                .iter()
                .map(|value| value.line)
                .collect::<Vec<_>>();
            let hash = sources::validate_primary(&live, source_hash.as_deref(), &lines)
                .map_err(|e| (Some(session.clone()), e))?;
            let update = sources::IntentUpdate::begin(&live.breakpoint_intents);
            live.session
                .set_breakpoints(&breakpoints)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            update.accept(
                String::new(),
                crate::debug_sessions::BreakpointIntent {
                    request_id: ticket,
                    source_hash: hash,
                },
            );
            Ok(vec![sources::primary_status(&session, &live)])
        }

        Request::Threads {
            session,
            request_id,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|error| (Some(session.clone()), error))?;
            let selected = (*live.thread_id.lock().await).ok_or_else(|| {
                (
                    Some(session.clone()),
                    anyhow::anyhow!("Pause the debugger before reading threads"),
                )
            })?;
            let stop = *live.stop_generation.lock().await;
            let threads = live
                .session
                .threads()
                .await
                .map_err(|error| (Some(session.clone()), error))?;
            Ok(vec![Response::Threads {
                session,
                request_id,
                stop,
                selected,
                threads,
            }])
        }
        Request::SelectThread {
            session,
            thread,
            stop,
            request_id,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|error| (Some(session.clone()), error))?;
            if live.thread_id.lock().await.is_none() || *live.stop_generation.lock().await != stop {
                return Err((
                    Some(session),
                    anyhow::anyhow!("The stopped thread observation changed. Read threads again."),
                ));
            }
            if !live
                .stopped
                .lock()
                .await
                .as_ref()
                .is_some_and(|stopped| stopped.all_threads_stopped || stopped.thread_id == thread)
            {
                return Err((
                    Some(session),
                    anyhow::anyhow!(
                        "The adapter has not reported this thread stopped. Pause it before selecting its stack."
                    ),
                ));
            }
            let threads = live
                .session
                .threads()
                .await
                .map_err(|error| (Some(session.clone()), error))?;
            if !threads.iter().any(|entry| entry.id == thread) {
                return Err((
                    Some(session),
                    anyhow::anyhow!(
                        "The selected thread is no longer reported by the adapter. Read threads again."
                    ),
                ));
            }
            // A reported thread may still be running. Require a real stack read
            // before changing selection; adapter refusal leaves prior frames intact.
            live.session
                .stack(thread)
                .await
                .map_err(|error| (Some(session.clone()), error))?;
            *live.thread_id.lock().await = Some(thread);
            retire_values(&live).await;
            let stop = *live.stop_generation.lock().await;
            let mut replies = position_with(root, &session, &live, "thread selection").await;
            replies.push(Response::Threads {
                session,
                request_id,
                stop,
                selected: thread,
                threads,
            });
            Ok(replies)
        }
        Request::State {
            session,
            frame,
            stop: expected_stop,
            thread: expected_thread,
            request_id,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            if let Some(frame) = frame {
                let (stop, thread) =
                    observed_scope(&live, expected_stop, expected_thread, request_id.as_deref())
                        .await
                        .map_err(|e| (Some(session.clone()), e))?;
                let frames = live
                    .session
                    .stack(thread)
                    .await
                    .map_err(|error| (Some(session.clone()), error))?;
                if !frames.iter().any(|entry| entry.id == frame) {
                    return Err((
                        Some(session),
                        anyhow::anyhow!(
                            "The requested locals frame is outside the selected thread. Read its stack again."
                        ),
                    ));
                }
                let variables = live
                    .session
                    .variables(frame)
                    .await
                    .map_err(|error| (Some(session.clone()), error))?;
                live.remember_values(stop, thread, frame, &variables)
                    .await
                    .map_err(|e| (Some(session.clone()), e))?;
                Ok(vec![Response::FrameVariables {
                    session,
                    stop,
                    thread,
                    frame,
                    request_id,
                    variables,
                }])
            } else {
                // If a previous observation failed after resume, only a new
                // actual stop can restore authority. A bounded refresh may
                // consume that queued stop; it never rebrands the old frame.
                if live.stopped.lock().await.is_none() {
                    if let Ok(Some(stopped)) =
                        live.session.wait_for_stop(Duration::from_millis(400)).await
                    {
                        *live.thread_id.lock().await = Some(stopped.thread_id);
                        *live.stopped.lock().await = Some(stopped);
                    }
                }
                Ok(position(root, &session, &live).await)
            }
        }

        Request::Eval {
            session,
            expression,
            frame,
            context,
            stop: expected_stop,
            thread: expected_thread,
            request_id,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let (stop, thread) =
                observed_scope(&live, expected_stop, expected_thread, request_id.as_deref())
                    .await
                    .map_err(|e| (Some(session.clone()), e))?;
            let frame = observed_frame(&live, thread, frame)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let value = live
                .session
                .evaluate(&expression, Some(frame), &context)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            live.remember_values(stop, thread, frame, std::slice::from_ref(&value))
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let mut out = Vec::new();
            if let Some(reading) = live
                .record_expression(&expression, &value.value, Some(frame))
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
                stop,
                thread: Some(thread),
                frame: Some(frame),
                request_id,
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
            stop,
            thread,
            request_id,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let (_, thread) = observed_scope(&live, stop, thread, request_id.as_deref())
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let frame = match frame {
                Some(frame) => Some(
                    observed_frame(&live, thread, Some(frame))
                        .await
                        .map_err(|e| (Some(session.clone()), e))?,
                ),
                None if matches!(how, hick_dap::Step::DropFrame) => {
                    return Err((
                        Some(session),
                        anyhow::anyhow!(
                            "Drop frame requires a current frame. Read its stack before trying again."
                        ),
                    ));
                }
                None => None,
            };
            live.session
                .step(how, thread, frame)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            retire_values(&live).await;
            *live.stopped.lock().await = None;
            Ok(settle(registry, root, &session, &live).await)
        }
        Request::Jump {
            session,
            line,
            stop,
            thread,
            request_id,
            source_hash,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let (_, thread) = observed_scope(&live, stop, thread, request_id.as_deref())
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            sources::validate_primary(&live, source_hash.as_deref(), &[line])
                .map_err(|e| (Some(session.clone()), e))?;
            live.session
                .jump_to(line, thread)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            retire_values(&live).await;
            Ok(settle_in_place(root, &session, &live).await)
        }
        Request::RunTo {
            session,
            line,
            stop,
            thread,
            request_id,
            source_hash,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let (_, thread) = observed_scope(&live, stop, thread, request_id.as_deref())
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            sources::validate_primary(&live, source_hash.as_deref(), &[line])
                .map_err(|e| (Some(session.clone()), e))?;
            live.session
                .run_to(line, thread)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            retire_values(&live).await;
            *live.stopped.lock().await = None;
            Ok(settle(registry, root, &session, &live).await)
        }

        Request::Children {
            session,
            reference,
            stop,
            thread,
            frame,
            request_id,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let (stop, thread) = observed_scope(&live, stop, thread, request_id.as_deref())
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let frame = observed_frame(&live, thread, frame)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            live.require_reference(stop, thread, frame, reference)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let variables = live
                .session
                .children(reference)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            live.remember_values(stop, thread, frame, &variables)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            Ok(vec![Response::Children {
                session,
                stop,
                thread: Some(thread),
                frame: Some(frame),
                request_id,
                reference,
                variables,
            }])
        }
        Request::SetVariable {
            session,
            container,
            name,
            value,
            stop,
            thread,
            frame,
            request_id,
        } => {
            let live = registry
                .get(&session)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let (stop, thread) = observed_scope(&live, stop, thread, request_id.as_deref())
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            let frame = observed_frame(&live, thread, frame)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            live.require_reference(stop, thread, frame, container)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
            live.session
                .set_variable(container, &name, &value)
                .await
                .map_err(|e| (Some(session.clone()), e))?;
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
    match live.session.wait_until_stop().await {
        Ok(Some(stopped)) => {
            *live.thread_id.lock().await = Some(stopped.thread_id);
            *live.stopped.lock().await = Some(stopped.clone());
            let mut out = vec![sources::primary_status(session, live)];
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
            scope_retired: true,
            request_id: None,
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
    if live.stopped.lock().await.is_none() {
        return vec![Response::Failed {scope_retired:true,request_id:None,session:Some(session.to_owned()),doc:None,message:"A current pause is not confirmed. Keep the retained stack as history and use Stop or pause the running program before reading another frame.".into(),about:None,lines:vec![],offer_install:None}];
    }
    let thread = match *live.thread_id.lock().await {
        Some(thread) => thread,
        None => {
            return vec![Response::Finished {
                session: session.to_string(),
                exit_code: live.session.exit().and_then(|exit| exit.code),
            }];
        }
    };
    let (frames, stack_error) = match live.session.stack(thread).await {
        Ok(frames) => (
            frames
                .into_iter()
                .map(|frame| relative_source(root, frame))
                .collect::<Vec<_>>(),
            None,
        ),
        Err(error) => (Vec::new(), Some(error.to_string())),
    };
    let source_stop = *live.stop_generation.lock().await;
    *live.adapter_sources.lock().await = adapter_sources::reported(&frames, source_stop, thread);
    let (mut variables, mut variables_error) = match frames.first() {
        Some(frame) => match live.session.variables(frame.id).await {
            Ok(variables) => (variables, None),
            Err(error) => (Vec::new(), Some(error.to_string())),
        },
        None => (Vec::new(), None),
    };
    if variables_error.is_none()
        && let Some(frame) = frames.first()
    {
        let stop = *live.stop_generation.lock().await;
        if let Err(error) = live
            .remember_values(stop, thread, frame.id, &variables)
            .await
        {
            variables.clear();
            variables_error = Some(error.to_string());
        }
    }
    let mut out = Vec::new();
    if stack_error.is_none()
        && variables_error.is_none()
        && let Ok(Some(reading)) = live
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
        stop: *live.stop_generation.lock().await,
        thread,
        stack_error,
        variables_error,
        observation_scope,
        session: session.to_string(),
        reason: reason.to_string(),
        line: frames.first().and_then(|frame| frame.line),
        frames,
        variables,
    });
    out
}
#[cfg(test)]
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
fn resolve(root: &std::path::Path, doc: &str) -> anyhow::Result<PathBuf> {
    crate::debug_sessions::source_document(root, doc)
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
pub fn reply(tx: &mpsc::UnboundedSender<Vec<u8>>, responses: &[Response]) {
    for response in responses {
        if tx.send(frame_of(response)).is_err() {
            return;
        }
    }
}
/// The shape the client sees, for its own tests.
pub fn describe() -> Value {
    json!({
        "channel": CHANNEL_DEBUG,
        "requests": [
            "start", "start_test", "breakpoints", "state", "eval", "step", "jump", "run_to",
            "children", "exception_breakpoints", "set_variable", "stop", "located", "threads", "select_thread"
        ],
        "responses": [
            "started", "stopped", "breakpoints", "value", "children",
            "finished", "ended", "failed", "build", "test_preparing", "reading", "threads", "frame_variables"
        ],
    })
}
#[cfg(test)]
#[path = "debug_bridge/tests/tests.rs"]
mod tests;
