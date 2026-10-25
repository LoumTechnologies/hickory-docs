//! `hick mcp` — the document tool set as an MCP server over stdio.
//!
//! Claude Code, Codex, and Grok CLI all speak the Model Context Protocol, so
//! one stdio server registers three ways and gives each of them the same five
//! tools the built-in agent uses. `hick init` writes the registration.
//!
//! **Why this exists alongside `hick doc`.** The commands are the universal
//! surface — any harness can run a process, and so can CI. This is the *good*
//! surface, and the difference is the session: the server is one long-lived
//! process, so it keeps an [`EditSession`] open per document and re-weaves
//! after every edit. That restores the property the per-command path can only
//! approximate — fresh hashes and provenance handed back immediately, with a
//! stale edit impossible by construction rather than merely detected.
//!
//! The protocol is implemented directly (JSON-RPC 2.0 over line-delimited
//! stdio) rather than through an SDK: the surface used here is five tools and
//! three methods, and a dependency that moves faster than that surface would
//! be a liability in a binary users install.
//!
#[path = "changeset_mcp.rs"]
mod changeset_tools;
#[path = "mcp_preview.rs"]
mod preview;
use crate::ExecutorChoice;
use anyhow::Result;
use hickory_agent::{EditSession, ToolInvocation, execute_tool};
use hickory_executor::Executor;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;

/// The MCP revision this server implements. A client asking for a different
/// one is answered with this; the spec expects a server to name what it
/// actually speaks rather than echo whatever it was sent.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// JSON-RPC error codes used here (the standard subset).
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

use crate::mcp_catalogue::tool_catalogue;

/// A running server: open sessions, one executor, an optional session log.
pub(crate) struct Server {
    root: PathBuf,
    changeset: Option<crate::changeset::Contract>,
    /// One [`EditSession`] per document, kept open for the process's life.
    /// This is the whole point of the MCP surface over the command surface.
    sessions: HashMap<PathBuf, EditSession>,
    executor: Arc<dyn Executor>,
    params: Vec<(String, String)>,
    /// The document used when a call names none.
    default_doc: Option<PathBuf>,
    /// `HICKORY_SESSION`: append every call to this `hick:session` document.
    session_log: Option<PathBuf>,
    /// Live debug sessions, by id. Bounded: see `debug_sessions`.
    debuggers: crate::debug_sessions::Registry,
    context_reads: Vec<hickory_agent::ContextRead>,
}

impl Server {
    pub(crate) fn take_context_reads(&mut self) -> Vec<hickory_agent::ContextRead> {
        std::mem::take(&mut self.context_reads)
    }

    pub(crate) fn embedded(
        root: PathBuf,
        doc: PathBuf,
        executor: Arc<dyn Executor>,
        session: PathBuf,
    ) -> Self {
        Self {
            root,
            changeset: None,
            sessions: HashMap::new(),
            executor,
            params: Vec::new(),
            default_doc: Some(doc),
            session_log: Some(session),
            debuggers: crate::debug_sessions::Registry::new(),
            context_reads: Vec::new(),
        }
    }

    /// The debug tools, which share one session registry.
    ///
    /// Every one of them answers in DOCUMENT coordinates, because an agent
    /// reading `read_doc` and an agent setting a breakpoint must be talking
    /// about the same lines.
    async fn call_debug_tool(&mut self, name: &str, args: &Value) -> Result<String, String> {
        use hick_dap::Step;

        let session_id = || -> Result<String, String> {
            args.get("session")
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| "which session? pass `session` from debug_start".to_string())
        };
        let fail = |e: anyhow::Error| format!("{e:#}");

        match name {
            "debug_start" => {
                let doc = self.resolve_doc(args)?;
                let breakpoints: Vec<hick_dap::Breakpoint> = args
                    .get("breakpoints")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| {
                                Some(hick_dap::Breakpoint {
                                    line: item.get("line")?.as_u64()? as u32,
                                    condition: item
                                        .get("condition")
                                        .and_then(Value::as_str)
                                        .map(str::to_string),
                                    hit_condition: item
                                        .get("hit_condition")
                                        .and_then(Value::as_str)
                                        .map(str::to_string),
                                    log_message: None,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // An agent naming the program is an agent that knows which of
                // several files it means; omitting it keeps the old behaviour.
                let program = args.get("program").and_then(Value::as_str);
                // An agent has no terminal to watch a build in, so the
                // build's own output is attached to the failure instead.
                // Dropping it would leave "building app.csproj failed" with
                // the one fact the agent already had, and none of the ones
                // it needs.
                let mut built: Vec<String> = Vec::new();
                let started = self
                    .debuggers
                    .start(&doc, &breakpoints, program, &mut |line| {
                        if let hick_dap::BuildOutput::Out(text) | hick_dap::BuildOutput::Err(text) =
                            line
                        {
                            built.push(text);
                        }
                    })
                    .await;
                let (id, live, _) = started.map_err(|error| {
                    if built.is_empty() {
                        fail(error)
                    } else {
                        fail(anyhow::anyhow!("{error:#}\n\n{}", built.join("\n")))
                    }
                })?;

                // Run to the first stop before answering: an agent that gets
                // a session id and no position has to guess whether the
                // program is running, stopped, or already finished.
                let stopped = live
                    .session
                    .wait_for_stop(std::time::Duration::from_secs(60))
                    .await
                    .map_err(fail)?;
                if let Some(stopped) = &stopped {
                    *live.thread_id.lock().await = Some(stopped.thread_id);
                    *live.stopped.lock().await = Some(stopped.clone());
                }

                let caps = live.session.capabilities();
                let mut out = format!("session {id}\n");
                out.push_str(&format!(
                    "adapter can: conditions={} hit-counts={} drop-frame={} set-value={} step-back={}\n",
                    caps.conditional_breakpoints,
                    caps.hit_conditional_breakpoints,
                    caps.restart_frame,
                    caps.set_variable,
                    caps.step_back,
                ));
                // Read AFTER the run to the first stop, not from the
                // set-time answer: a compiled language confirms nothing until
                // its module loads, so the set-time list says "pending" for
                // every line in a document that works perfectly.
                let statuses = live.session.breakpoint_statuses();
                for status in &statuses {
                    out.push_str(&format!(
                        "  line {}: {}{}\n",
                        status.line,
                        match status.state {
                            hick_dap::BindState::Bound => "bound",
                            // Not "NOT BOUND": the adapter has not answered
                            // yet, and telling an agent a breakpoint failed
                            // when it is about to work is how it gives up on
                            // a working document.
                            hick_dap::BindState::Pending => "not confirmed yet",
                            hick_dap::BindState::Refused => "NOT BOUND",
                        },
                        status
                            .message
                            .as_deref()
                            .map(|m| format!(" — {m}"))
                            .unwrap_or_default()
                    ));
                }
                out.push_str(&match &stopped {
                    Some(stopped) => format!("stopped: {}\n", stopped.reason),
                    None => {
                        // Nothing left to debug: reap now rather than leaving
                        // the adapter process and scratch copy to the idle
                        // sweep — the same teardown the app's bridge does.
                        let exit = live.session.exit().and_then(|e| e.code);
                        self.debuggers.reap(&id).await;
                        format!(
                            "the program ran to completion without stopping{} — the session \
                             ended and its scratch copy was deleted; call debug_start again \
                             (with breakpoints that bind) to re-run\n",
                            exit.map(|c| format!(" (exit code {c})"))
                                .unwrap_or_default()
                        )
                    }
                });
                if let Some(stopped) = &stopped {
                    let stack = live.session.stack(stopped.thread_id).await.map_err(fail)?;
                    let frame = stack.first().map(|f| f.id);
                    let variables = match frame {
                        Some(frame) => live.session.variables(frame).await.map_err(fail)?,
                        None => vec![],
                    };
                    if let Some(reading) = live
                        .capture_reading(&id, &stopped.reason, &stack, &variables, frame)
                        .await
                        .map_err(fail)?
                    {
                        out.push_str(&reading.agent_projection().to_string());
                    }
                }
                Ok(out)
            }

            "debug_state" => {
                let live = self.debuggers.get(&session_id()?).await.map_err(fail)?;
                let _operation = live.operation.lock().await;
                let thread = live
                    .thread_id
                    .lock()
                    .await
                    .ok_or_else(|| "the program is not stopped".to_string())?;
                let stack = live.session.stack(thread).await.map_err(fail)?;
                let frame_id = args
                    .get("frame")
                    .and_then(Value::as_i64)
                    .or_else(|| stack.first().map(|f| f.id))
                    .ok_or_else(|| "no frames — the program is not stopped".to_string())?;

                let variables = live.session.variables(frame_id).await.map_err(fail)?;
                let reading = live
                    .capture_reading(&session_id()?, "state", &stack, &variables, Some(frame_id))
                    .await
                    .map_err(fail)?;
                match reading {
                    Some(reading) => serde_json::to_string(&reading.agent_projection())
                        .map_err(|e| fail(e.into())),
                    None => {
                        Err("captured source unavailable; source attribution unsupported".into())
                    }
                }
            }

            "debug_eval" => {
                let live = self.debuggers.get(&session_id()?).await.map_err(fail)?;
                let _operation = live.operation.lock().await;
                let expression = args
                    .get("expression")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "pass an `expression` to evaluate".to_string())?;
                let frame = match args.get("frame").and_then(Value::as_i64) {
                    Some(frame) => Some(frame),
                    None => {
                        let thread = live.thread_id.lock().await.unwrap_or(1);
                        live.session
                            .stack(thread)
                            .await
                            .ok()
                            .and_then(|s| s.first().map(|f| f.id))
                    }
                };
                let value = live
                    .session
                    // `repl`, not `watch`: an agent typing an expression is a
                    // person typing an expression, and several adapters
                    // refuse side effects in `watch` — which is right for a
                    // capture and wrong here.
                    .evaluate(expression, frame, "repl")
                    .await
                    .map_err(fail)?;
                if let Some(reading) = live
                    .record_expression(expression, &value.value, frame)
                    .await
                    .map_err(fail)?
                {
                    Ok(reading.agent_projection().to_string())
                } else {
                    Ok(format!(
                        "{} = {} (source/stop attribution unavailable)",
                        value.name, value.value
                    ))
                }
            }

            "debug_step" => {
                let live = self.debuggers.get(&session_id()?).await.map_err(fail)?;
                let _operation = live.operation.lock().await;
                let resolved = match (
                    args.get("location").and_then(Value::as_str),
                    args.get("scope"),
                ) {
                    (Some(location), Some(scope)) => Some(
                        live.resolve_location(
                            location,
                            &serde_json::from_value(scope.clone()).map_err(|e| fail(e.into()))?,
                        )
                        .await
                        .map_err(fail)?,
                    ),
                    (None, None) => None,
                    _ => return Err("location and scope must be supplied together".into()),
                };
                *live.reading.lock().await = None;
                *live.stop_generation.lock().await += 1;
                let how = args.get("how").and_then(Value::as_str).unwrap_or("over");
                if how == "jump" {
                    let line = resolved
                        .map(u64::from)
                        .or_else(|| args.get("line").and_then(Value::as_u64))
                        .ok_or_else(|| {
                            "jumping needs a `line` — the document line to move to, which must \
                             be in the function you are stopped in"
                                .to_string()
                        })? as u32;
                    let thread = live
                        .thread_id
                        .lock()
                        .await
                        .ok_or_else(|| "the program is not stopped".to_string())?;
                    live.session.jump_to(line, thread).await.map_err(fail)?;
                    let stopped = live
                        .session
                        .wait_for_stop(std::time::Duration::from_secs(20))
                        .await
                        .map_err(fail)?;
                    if let Some(stopped) = &stopped {
                        *live.thread_id.lock().await = Some(stopped.thread_id);
                        *live.stopped.lock().await = Some(stopped.clone());
                    }
                    let stack = live.session.stack(thread).await.map_err(fail)?;
                    return Ok(match stack.first().and_then(|f| f.line) {
                        Some(line) => format!(
                            "moved to line {line} in {} — the frame's variables are unchanged, \
                             and the lines between here and where you were will run again",
                            stack.first().map(|f| f.name.as_str()).unwrap_or("?")
                        ),
                        None => "moved, but outside the document".to_string(),
                    });
                }
                let step = match how {
                    "over" => Step::Over,
                    "in" => Step::In,
                    "out" => Step::Out,
                    "continue" => Step::Continue,
                    "drop_frame" => Step::DropFrame,
                    other => {
                        return Err(format!(
                            "unknown step `{other}` — use over, in, out, continue, drop_frame or jump"
                        ));
                    }
                };
                let thread = live.thread_id.lock().await.ok_or_else(|| {
                    "the program is not stopped, so there is nothing to step".to_string()
                })?;
                let frame = match args.get("frame").and_then(Value::as_i64) {
                    Some(frame) => Some(frame),
                    None => live
                        .session
                        .stack(thread)
                        .await
                        .ok()
                        .and_then(|s| s.first().map(|f| f.id)),
                };
                live.session.step(step, thread, frame).await.map_err(fail)?;

                match live
                    .session
                    .wait_for_stop(std::time::Duration::from_secs(30))
                    .await
                    .map_err(fail)?
                {
                    Some(stopped) => {
                        *live.thread_id.lock().await = Some(stopped.thread_id);
                        *live.stopped.lock().await = Some(stopped.clone());
                        let stack = live.session.stack(stopped.thread_id).await.map_err(fail)?;
                        let top = stack.first();
                        Ok(match top.and_then(|f| f.line) {
                            Some(line) => format!(
                                "stopped ({}) in {} at line {line}",
                                stopped.reason,
                                top.map(|f| f.name.as_str()).unwrap_or("?")
                            ),
                            None => format!("stopped ({}) outside the document", stopped.reason),
                        })
                    }
                    None => {
                        *live.thread_id.lock().await = None;
                        let exit = live.session.exit().and_then(|e| e.code);
                        self.debuggers.reap(&session_id()?).await;
                        Ok(format!(
                            "the program finished{} — the session ended and its scratch copy \
                             was deleted; call debug_start to run again",
                            exit.map(|c| format!(" (exit code {c})"))
                                .unwrap_or_default()
                        ))
                    }
                }
            }

            "debug_stop" => {
                // Reap, not stop: a program that finished has already been
                // reaped automatically, and a follow-up stop finding "already
                // gone" is the expected case, not an error.
                let id = session_id()?;
                Ok(if self.debuggers.reap(&id).await {
                    format!("session {id} ended and its scratch copy deleted")
                } else {
                    format!("session {id} had already ended")
                })
            }

            other => Err(format!("unknown debug tool `{other}`")),
        }
    }

    /// The document a call refers to.
    fn resolve_doc(&self, args: &Value) -> Result<PathBuf, String> {
        if let Some(d) = args.get("doc").and_then(Value::as_str) {
            return Ok(self.root.join(d));
        }
        self.default_doc.clone().ok_or_else(|| {
            "no document: pass `doc` (a path to a .md file), or start the server with one"
                .to_string()
        })
    }

    async fn session_for(&mut self, doc: &PathBuf) -> Result<&mut EditSession, String> {
        if !self.sessions.contains_key(doc) {
            if !doc.exists() {
                return Err(format!(
                    "no such document: {} — pass the path to a .md source file, \
                     not a generated output",
                    doc.display()
                ));
            }
            let session = EditSession::open(doc, &self.params)
                .await
                .map_err(|e| format!("could not open {}: {e:#}", doc.display()))?;
            self.sessions.insert(doc.clone(), session);
        }
        Ok(self.sessions.get_mut(doc).expect("just inserted"))
    }

    /// Translate MCP call arguments into the tool vocabulary.
    ///
    /// `upstream` becomes the tools' `doc` argument, because at this layer
    /// "doc" already means "which document is this call about". Two things
    /// called `doc` meaning different things is exactly the kind of seam an
    /// agent gets wrong, so the MCP surface renames the rarer one.
    pub(crate) fn invocation(name: &str, args: &Value) -> Result<ToolInvocation, String> {
        let mut tool_args: Vec<(String, String)> = Vec::new();
        if let Some(u) = args.get("upstream").and_then(Value::as_str) {
            tool_args.push(("doc".into(), u.to_string()));
        }
        for key in ["path", "run", "after", "snapshot", "whole"] {
            if let Some(v) = args.get(key).and_then(Value::as_str) {
                tool_args.push((key.to_string(), v.to_string()));
            }
        }
        if let Some(n) = args.get("occurrence").and_then(Value::as_u64) {
            tool_args.push(("occurrence".into(), n.to_string()));
        }
        for key in ["from", "to"] {
            if let Some(n) = args.get(key).and_then(Value::as_u64) {
                tool_args.push((key.to_string(), n.to_string()));
            }
        }
        if args.get("with_lineage").and_then(Value::as_bool) == Some(true) {
            tool_args.push(("with_lineage".into(), "true".into()));
        }
        let input = args
            .get("input")
            .and_then(Value::as_str)
            .map(str::to_string);
        ToolInvocation::synthetic(name, tool_args, input)
    }

    fn record_activity(&self, name: &str, result: Result<String, String>) -> Value {
        if let Some(path) = &self.session_log
            && let Err(error) = crate::doc_tools::record_activity_result(path, name, &result)
        {
            return text_result(
                &format!("Tool ran but recording its result failed: {error:#}"),
                true,
            );
        }
        match result {
            Ok(text) => text_result(&text, false),
            Err(text) => text_result(&text, true),
        }
    }

    async fn call_tool(&mut self, name: &str, args: &Value) -> Value {
        if matches!(
            name,
            "begin_changeset" | "changeset_status" | "submit_changeset"
        ) {
            return self.call_changeset(name, args);
        }
        if hickory_agent::audit::is_tool(name) {
            let path = self
                .session_log
                .clone()
                .unwrap_or_else(|| self.root.join("sessions/assumption-audit.md"));
            return match hickory_agent::audit::call_at_in(
                &path,
                name,
                args.clone(),
                Some(&self.root),
            ) {
                Ok(text) => text_result(&text, false),
                Err(error) => text_result(&format!("{error:#}"), true),
            };
        }
        if let Some(path) = &self.session_log
            && let Err(error) = crate::doc_tools::record_intent(path, name, args)
        {
            return text_result(&format!("Recording failed; no tool ran: {error:#}"), true);
        }
        // Debugging is not a document edit, so it does not go through the
        // edit-session machinery: it has its own lifetime, its own scratch
        // copy, and — deliberately — no way to write anything.
        if name == "performance_profile" {
            let result=async {let request=serde_json::from_value(args.clone()).map_err(|e|e.to_string())?;crate::performance::dispatch(&self.root,request).await.map(|v|v.to_string()).map_err(|e|format!("{e:#}"))}.await;
            return self.record_activity(name,result);
        }
        if name == "code_coverage" {
            let result = async {
                let request = serde_json::from_value(args.clone()).map_err(|e| e.to_string())?;
                crate::coverage::dispatch(&self.root, request)
                    .await
                    .map(|v| v.to_string())
                    .map_err(|e| format!("{e:#}"))
            }
            .await;
            return self.record_activity(name, result);
        }
        if name.starts_with("debug_") {
            let result = self.call_debug_tool(name, args).await;
            return self.record_activity(name, result);
        }
        // Search is about the project, not one document, so it skips the
        // edit-session machinery entirely (and can never write anything).
        if name == "search" {
            return self.record_activity(name, call_search_tool(&self.root, args).await);
        }
        // Listing and creating are about the project, not one document, so
        // they route the way `search` does. They exist because every other
        // tool takes a `doc` and, without these, an agent arriving in a
        // project had no way to learn what documents there were or to start
        // a new one — it had to leave the tool surface and write the file
        // itself, which is the one thing the surface asks it not to do.
        if name == "list_docs" {
            return self.record_activity(name, call_list_docs_tool(&self.root, args));
        }
        if name == "create_doc" {
            return self.record_activity(name, call_create_doc_tool(&self.root, args));
        }
        let doc = match self.resolve_doc(args) {
            Ok(d) => d,
            Err(e) => return text_result(&e, true),
        };
        let invocation = match Self::invocation(name, args) {
            Ok(i) => i,
            Err(e) => return text_result(&e, true),
        };
        let executor = self.executor.clone();
        let session_log = self.session_log.clone();
        let session = match self.session_for(&doc).await {
            Ok(s) => s,
            Err(e) => return text_result(&e, true),
        };
        let outcome = execute_tool(session, executor, &invocation).await;
        let revision = session.revision();
        let sources = session.source_snapshots();

        if let Some(path) = &session_log
            && let Err(e) = crate::doc_tools::record_tool_call(path, &invocation, &outcome)
                .and_then(|()| {
                    if name == "verify" {
                        crate::doc_tools::record_check(path, outcome.ok, &revision, &sources)
                    } else {
                        Ok(())
                    }
                })
        {
            // stderr, never stdout: stdout carries the protocol, and a stray
            // line there desynchronizes the client's parser.
            return text_result(
                &format!(
                    "Tool result recording failed after the tool ran; inspect actual files. Session {} is incomplete: {e:#}",
                    path.display()
                ),
                true,
            );
        }
        // A refused tool is `isError: true` so the harness shows it as a
        // failed call — but the text still comes through, because a lineage
        // refusal's message is the instruction for what to do next.
        self.context_reads = outcome.reads;
        text_result(&outcome.text, !outcome.ok)
    }

    pub(crate) async fn handle(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Result<Value, (i64, String)> {
        match method {
            "initialize" => Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "hick", "version": env!("CARGO_PKG_VERSION") },
                "instructions":
                    "Edit hick documents through these tools rather than by writing to the \
                     files directly. Read a surface first (read_doc / read_output with \
                     with_lineage), anchor every edit on the SHA-256 content hashes it returns, \
                     put CODE changes through edit_output and structure or prose through \
                     edit_doc, and call verify before reporting the work done. For a final \
                     literate changeset, call begin_changeset BEFORE edits in standalone MCP \
                     (the dock supplies its boundary), then changeset_status and submit_changeset. \
                     Repair diagnostics until accepted; output equality does not verify prose or runtime behavior."
            })),
            "ping" => Ok(json!({})),
            "tools/list" => {
                let mut result = tool_catalogue();
                result["tools"]
                    .as_array_mut()
                    .unwrap()
                    .extend(crate::changeset::catalogue(true));
                Ok(result)
            }
            "tools/call" => {
                let name = params
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or((INVALID_PARAMS, "tools/call needs a tool name".to_string()))?;
                let empty = json!({});
                let args = params.get("arguments").unwrap_or(&empty);
                Ok(self.call_tool(name, args).await)
            }
            other => Err((
                METHOD_NOT_FOUND,
                format!(
                    "unknown method '{other}'; this server implements initialize, ping, tools/list, tools/call"
                ),
            )),
        }
    }
}

/// An MCP tool result carrying one block of text.
/// The `search` tool: same engine as `hick search` and the app's search
/// panel, answered in text an agent can act on (path:start-end + snippet).
async fn call_search_tool(project: &std::path::Path, args: &Value) -> Result<String, String> {
    let query = args.get("query").and_then(Value::as_str).map(str::trim);
    let related = args
        .get("related")
        .and_then(Value::as_str)
        .map(str::to_string);
    let top_k = args
        .get("top_k")
        .and_then(Value::as_u64)
        .map_or(8, |k| k as usize)
        .clamp(1, 50);
    if query.is_none_or(str::is_empty) && related.is_none() {
        return Err("pass `query` (what to look for) or `related` (FILE:LINE)".to_string());
    }
    let query = query.map(str::to_string);
    let root = project.to_path_buf();

    let (semantic, hits) = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let engine = hick_search::SearchEngine::open(&root)?;
        let hits = match (&related, &query) {
            (Some(spec), _) => {
                let (file, line) = hick_search::parse_file_line(spec)?;
                engine.related(&file, line, top_k)?
            }
            (None, Some(q)) => engine.search(q, top_k),
            (None, None) => unreachable!("checked above"),
        };
        Ok((engine.semantic(), hits))
    })
    .await
    .map_err(|e| format!("search task failed: {e}"))?
    .map_err(|e| format!("{e:#}"))?;

    if hits.is_empty() {
        return Ok("no matches".to_string());
    }
    let mut out = String::new();
    if !semantic {
        out.push_str(
            "(lexical ranking only — `hick search --install-model` adds semantic ranking)\n\n",
        );
    }
    for hit in hits {
        out.push_str(&format!(
            "{}:{}-{}\n",
            hit.path, hit.start_line, hit.end_line
        ));
        for line in hit.snippet.lines().filter(|l| !l.trim().is_empty()).take(4) {
            out.push_str(&format!("    {line}\n"));
        }
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}

/// `list_docs`: every `.md` document in the project, with what it weaves.
///
/// Gitignore-aware, for the same reason `hick ingest` is: a project's build
/// output routinely contains `.md` fixtures, and listing them as if they
/// were the user's documents sends an agent to edit a file that regenerates
/// over it.
fn call_list_docs_tool(project: &std::path::Path, args: &Value) -> Result<String, String> {
    let root = project.to_path_buf();
    let under = args.get("under").and_then(Value::as_str).unwrap_or("");
    let start = if under.is_empty() {
        root.clone()
    } else {
        root.join(under)
    };
    if !start.is_dir() {
        return Err(format!(
            "no such directory: {under} — pass a directory relative to the project root, \
             or omit `under` for the whole project"
        ));
    }

    let mut docs: Vec<(String, String)> = Vec::new();
    for entry in ignore::WalkBuilder::new(&start).build().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        // The weave, and what the document generates, without executing it:
        // a listing must never run anybody's cells.
        let summary = match std::fs::read_to_string(path) {
            // A session is a `.md` file with a `<hick:session>` root, not a
            // pipeline document, and the pipeline parser rightly refuses it.
            // Reporting somebody's own conversation record as a broken
            // document is worse than not listing it: it reads as damage.
            Ok(source) if hick_lang::is_session_source(&source) => {
                match hick_lang::parse_session(&source) {
                    Ok(session) => format!(
                        "a recorded session, {} entr{} — read-only here; \
                         `hick ingest --from session` turns one into a document",
                        session.nodes.len(),
                        if session.nodes.len() == 1 { "y" } else { "ies" }
                    ),
                    Err(e) => format!("(a session, but it does not parse: {e})"),
                }
            }
            Ok(source) => match hick_lang::parse_from_path(&source, path) {
                Ok(doc) => describe_document(&doc),
                Err(e) => format!("(does not parse: {e})"),
            },
            Err(e) => format!("(unreadable: {e})"),
        };
        docs.push((rel, summary));
    }
    docs.sort();

    if docs.is_empty() {
        return Ok(format!(
            "no hick documents{}. Create one with create_doc.",
            if under.is_empty() {
                String::new()
            } else {
                format!(" under {under}")
            }
        ));
    }
    let mut out = String::new();
    for (path, summary) in docs {
        out.push_str(&format!("{path}\n    {summary}\n"));
    }
    Ok(out.trim_end().to_string())
}

/// One line about a document: what it weaves, what it writes, what it runs.
///
/// Parsed, never executed. The counts are what an agent needs to decide which
/// document to open; the detail is what `read_doc` is for.
fn describe_document(doc: &hick_lang::HickDocument) -> String {
    let mut files: Vec<String> = Vec::new();
    let mut execs = 0usize;
    let mut stack: Vec<&hick_lang::HickNode> = doc.nodes.iter().collect();
    while let Some(node) = stack.pop() {
        if let hick_lang::HickNode::Tag(tag) = node {
            match tag.name.as_str() {
                "file" => {
                    if let Some(path) = hick_literate::tag_attr(tag, "path") {
                        files.push(path);
                    }
                }
                "exec" => execs += 1,
                _ => {}
            }
            stack.extend(tag.children.iter());
        }
    }
    files.sort();
    files.dedup();

    let mut parts = Vec::new();
    match &doc.weave_path {
        Some(w) => parts.push(format!("weaves {w}")),
        None => parts.push("weaves nothing (weave=\"none\")".to_string()),
    }
    if !files.is_empty() {
        parts.push(format!("writes {}", files.join(", ")));
    }
    if execs > 0 {
        parts.push(format!(
            "{execs} exec cell{}",
            if execs == 1 { "" } else { "s" }
        ));
    }
    parts.join("; ")
}

/// `create_doc`: a new bare document, refusing to overwrite.
fn call_create_doc_tool(project: &std::path::Path, args: &Value) -> Result<String, String> {
    let root = project.to_path_buf();
    let raw = args
        .get("path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .ok_or("pass `path`: where to create the document, relative to the project root")?;
    let mut rel = PathBuf::from(raw);
    if rel.extension().is_none_or(|e| e != "md") {
        rel.set_extension("md");
    }
    // A path that climbs out of the project is refused rather than
    // normalised: the tools are confined to the project by construction, and
    // silently rewriting somebody's path is worse than saying no.
    if rel.is_absolute() || rel.components().any(|c| c.as_os_str() == "..") {
        return Err(format!(
            "`{raw}` leaves the project — pass a path relative to the project root, \
             with no leading `/` and no `..`"
        ));
    }
    let full = root.join(&rel);
    if full.exists() {
        return Err(format!(
            "{} already exists — create_doc never overwrites. Use edit_doc to change a \
             document that is already there, or pick another path.",
            rel.display()
        ));
    }
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("could not create {}: {e}", parent.display()))?;
    }
    // Bare, per docs/specs/freeform/bare-documents.md: no wrapper, and the
    // weave defaults to the document's own name.
    let body = args.get("input").and_then(Value::as_str).unwrap_or("");
    let body = if body.is_empty() || body.ends_with('\n') {
        body.to_string()
    } else {
        format!("{body}\n")
    };
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&full)
        .map_err(|e| {
            format!(
                "could not create {} without overwriting: {e}",
                rel.display()
            )
        })?;
    file.write_all(body.as_bytes())
        .map_err(|e| format!("could not write {}: {e}", rel.display()))?;
    Ok(format!(
        "created {} ({} bytes). It weaves {}. Read it with read_doc to get edit anchors, \
         and call verify to execute it.",
        rel.display(),
        body.len(),
        rel.with_extension("md").display()
    ))
}

fn text_result(text: &str, is_error: bool) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error,
    })
}

#[path = "mcp_stdio.rs"]
mod stdio;
pub(crate) use stdio::engine_server;
pub use stdio::serve;

#[cfg(test)]
#[path = "mcp/tests/tests.rs"]
mod tests;
