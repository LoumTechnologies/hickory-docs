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
//! Everything the model sees — tool names, argument names, the doctrine in the
//! descriptions — is the same vocabulary as the built-in agent's system
//! prompt. An external agent that learns hick here has learned the same
//! thing our agent knows.

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use hickory_agent::{EditSession, ToolInvocation, execute_tool};
use hickory_executor::Executor;
use serde_json::{Value, json};

use crate::ExecutorChoice;

/// The MCP revision this server implements. A client asking for a different
/// one is answered with this; the spec expects a server to name what it
/// actually speaks rather than echo whatever it was sent.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// JSON-RPC error codes used here (the standard subset).
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// The tool catalogue, in the vocabulary the built-in agent already uses.
///
/// Descriptions carry the doctrine, not just the mechanics: an agent that
/// reads only this list should still edit code through the output and prose
/// through the document, because that is where lineage can reproduce the edit
/// byte-for-byte.
fn tool_catalogue() -> Value {
    let doc_arg = json!({
        "type": "string",
        "description": "Path to the .hick document. Omit if the server was started with one."
    });
    json!({
        "tools": [
            {
                "name": "read_doc",
                "description":
                    "Read the hick document source, hashline-rendered: every line is prefixed \
                     `hhhh|`, a 4-hex hash of that line's content. Those hashes are the anchors \
                     every edit takes — line numbers are never used, which is what makes a stale \
                     edit impossible rather than merely unlikely. Read this before editing prose \
                     or structure.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "doc": doc_arg,
                        "upstream": {
                            "type": "string",
                            "description": "Read an upstream document (by name or path) instead. \
                                            The primary's upstream closure is listed in the result."
                        }
                    }
                }
            },
            {
                "name": "read_output",
                "description":
                    "Read a generated output file, hashline-rendered. With with_lineage, each \
                     range is annotated with where it came from in the document and whether it \
                     can be edited through the output. Read this before editing code.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "doc": doc_arg,
                        "path": {
                            "type": "string",
                            "description": "The output file, as the document names it."
                        },
                        "with_lineage": {
                            "type": "boolean",
                            "description": "Annotate ranges with provenance and editability."
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "edit_output",
                "description":
                    "Edit CODE by editing the generated output; the change is mapped back into \
                     the document byte-exactly through lineage. This is the preferred path for \
                     code, because the output is what you actually reason about. A refusal is \
                     routing, not failure: it names the document location to edit with edit_doc \
                     instead.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "doc": doc_arg,
                        "path": { "type": "string", "description": "The output file." },
                        "run": {
                            "type": "string",
                            "description": "Line run to REPLACE: `hash` for one line, \
                                            `first..last` for a range."
                        },
                        "after": {
                            "type": "string",
                            "description": "Insert BELOW this line hash instead of replacing; \
                                            `^` inserts at the top of the file."
                        },
                        "occurrence": {
                            "type": "integer",
                            "description": "1-based index when the anchor matches several places."
                        },
                        "input": {
                            "type": "string",
                            "description": "Replacement text. Omit to DELETE the anchored run."
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "edit_doc",
                "description":
                    "Edit the document source directly, using read_doc hashes. This is the path \
                     for structure and prose — headings, copy blocks, pipeline shape — and where \
                     lineage refusals send you.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "doc": doc_arg,
                        "upstream": {
                            "type": "string",
                            "description": "Edit an upstream document instead; the primary is \
                                            re-woven afterwards, and the edit is rolled back if \
                                            that re-weave fails."
                        },
                        "run": { "type": "string", "description": "Line run to replace." },
                        "after": { "type": "string", "description": "Insert below this line." },
                        "occurrence": { "type": "integer" },
                        "input": {
                            "type": "string",
                            "description": "Replacement text. Omit to DELETE the anchored run."
                        }
                    }
                }
            },
            {
                "name": "verify",
                "description":
                    "Execute the document for real — every exec cell, every expectation — and \
                     write its outputs. Do this before reporting an edit as done: an edit that \
                     was never verified is a guess about what the document produces.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "doc": doc_arg }
                }
            },
            {
                "name": "search",
                "description":
                    "Search the whole project: a natural-language or code query returns ranked \
                     chunks with exact file:line, like semble. Lexical (BM25) always; semantic \
                     as well when the project has an embedding model (`hick search \
                     --install-model`). Prefer this over grepping when you are looking for \
                     where something is done rather than an exact string. Pass `related` \
                     (FILE:LINE) instead of `query` to find code similar to a location.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "What to look for, in natural language or code."
                        },
                        "related": {
                            "type": "string",
                            "description": "FILE:LINE (relative to the project root) to find \
                                            similar code for, instead of a query."
                        },
                        "top_k": {
                            "type": "integer",
                            "description": "How many results (default 8, max 50)."
                        }
                    }
                }
            },
            {
                "name": "debug_start",
                "description":
                    "Start a debug session over the document's generated code and run to the \
                     first breakpoint. Lines are DOCUMENT lines — the same ones `read_doc` \
                     shows. Returns a session id, what the adapter can do, and which \
                     breakpoints it could actually bind. Reading a failing document is \
                     guessing; stopping inside it and asking what a variable holds is not. \
                     The session runs in a scratch copy, so nothing it does can change the \
                     document or its outputs.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "doc": doc_arg,
                        "program": {
                            "type": "string",
                            "description": "Which generated file to run, e.g. `orders.py`. \
                                            Omit only when the document generates one \
                                            debuggable file: otherwise the first one is used, \
                                            which is right by accident at best.",
                        },
                        "breakpoints": {
                            "type": "array",
                            "description": "Where to stop. A breakpoint on prose is refused with \
                                            a reason rather than silently ignored.",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "line": { "type": "integer", "description": "0-based document line" },
                                    "condition": { "type": "string", "description": "stop only when this is true, in the debuggee's language" },
                                    "hit_condition": { "type": "string", "description": "stop only on the Nth hit, e.g. \">5\"" }
                                },
                                "required": ["line"]
                            }
                        }
                    }
                }
            },
            {
                "name": "debug_state",
                "description":
                    "Where the program is stopped: the reason, the call stack in DOCUMENT \
                     coordinates, and the selected frame's variables. Frames outside the \
                     document are marked as such rather than given a line they do not have.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "session": { "type": "string" },
                        "frame": { "type": "integer", "description": "frame id; defaults to the top frame" }
                    },
                    "required": ["session"]
                }
            },
            {
                "name": "debug_eval",
                "description":
                    "Evaluate an expression in a frame, in the debuggee's own language. The \
                     same evaluator the app's expression box uses, so an expression that \
                     works here works there.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "session": { "type": "string" },
                        "expression": { "type": "string" },
                        "frame": { "type": "integer", "description": "frame id; defaults to the top frame" }
                    },
                    "required": ["session", "expression"]
                }
            },
            {
                "name": "debug_step",
                "description":
                    "Move: `over`, `in`, `out`, `continue`, or one of the two ways BACKWARDS — \
                     `drop_frame`, which re-enters the current function from its first line, and \
                     `jump`, which moves the instruction pointer to `line` in the current frame. \
                     Both re-run rather than rewind: side effects already performed stay \
                     performed, and the frame's variables are intact. Most adapters have one or \
                     the other — debugpy has `jump` — and `debug_start` says which.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "session": { "type": "string" },
                        "how": {
                            "type": "string",
                            "enum": ["over", "in", "out", "continue", "drop_frame", "jump"]
                        },
                        "line": {
                            "type": "integer",
                            "description": "for `jump`: the 0-based DOCUMENT line to move to, in the current frame"
                        },
                        "frame": { "type": "integer", "description": "for drop_frame; defaults to the top frame" }
                    },
                    "required": ["session", "how"]
                }
            },
            {
                "name": "debug_stop",
                "description":
                    "End the session and delete its scratch copy. Sessions also end on their \
                     own after being untouched — a debugger nobody is watching is a process \
                     holding a working directory open.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "session": { "type": "string" } },
                    "required": ["session"]
                }
            }
        ]
    })
}

/// A running server: open sessions, one executor, an optional session log.
struct Server {
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
}

impl Server {
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
                let (id, live, statuses) = self
                    .debuggers
                    .start(&doc, &breakpoints, program)
                    .await
                    .map_err(fail)?;

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
                for status in &statuses {
                    out.push_str(&format!(
                        "  line {}: {}{}\n",
                        status.line,
                        if status.verified {
                            "bound"
                        } else {
                            "NOT BOUND"
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
                Ok(out)
            }

            "debug_state" => {
                let live = self.debuggers.get(&session_id()?).await.map_err(fail)?;
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

                let mut out = String::from("stack (document lines):\n");
                for frame in &stack {
                    out.push_str(&match frame.line {
                        Some(line) => format!("  #{} {} at line {}\n", frame.id, frame.name, line),
                        None => format!(
                            "  #{} {} (outside the document: {})\n",
                            frame.id,
                            frame.name,
                            frame.source.as_deref().unwrap_or("unknown")
                        ),
                    });
                }
                out.push_str(&format!("\nvariables in frame #{frame_id}:\n"));
                for variable in live.session.variables(frame_id).await.map_err(fail)? {
                    out.push_str(&format!(
                        "  {} = {}{}\n",
                        variable.name,
                        variable.value,
                        variable
                            .type_name
                            .map(|t| format!("  ({t})"))
                            .unwrap_or_default()
                    ));
                }
                Ok(out)
            }

            "debug_eval" => {
                let live = self.debuggers.get(&session_id()?).await.map_err(fail)?;
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
                Ok(match value.type_name {
                    Some(type_name) => format!("{} = {}  ({type_name})", value.name, value.value),
                    None => format!("{} = {}", value.name, value.value),
                })
            }

            "debug_step" => {
                let live = self.debuggers.get(&session_id()?).await.map_err(fail)?;
                let how = args.get("how").and_then(Value::as_str).unwrap_or("over");
                if how == "jump" {
                    let line = args.get("line").and_then(Value::as_u64).ok_or_else(|| {
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
            return Ok(PathBuf::from(d));
        }
        self.default_doc.clone().ok_or_else(|| {
            "no document: pass `doc` (a path to a .hick file), or start the server with one"
                .to_string()
        })
    }

    async fn session_for(&mut self, doc: &PathBuf) -> Result<&mut EditSession, String> {
        if !self.sessions.contains_key(doc) {
            if !doc.exists() {
                return Err(format!(
                    "no such document: {} — pass the path to a .hick source file, \
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
    fn invocation(name: &str, args: &Value) -> Result<ToolInvocation, String> {
        let mut tool_args: Vec<(String, String)> = Vec::new();
        if let Some(u) = args.get("upstream").and_then(Value::as_str) {
            tool_args.push(("doc".into(), u.to_string()));
        }
        for key in ["path", "run", "after"] {
            if let Some(v) = args.get(key).and_then(Value::as_str) {
                tool_args.push((key.to_string(), v.to_string()));
            }
        }
        if let Some(n) = args.get("occurrence").and_then(Value::as_u64) {
            tool_args.push(("occurrence".into(), n.to_string()));
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

    async fn call_tool(&mut self, name: &str, args: &Value) -> Value {
        // Debugging is not a document edit, so it does not go through the
        // edit-session machinery: it has its own lifetime, its own scratch
        // copy, and — deliberately — no way to write anything.
        if name.starts_with("debug_") {
            return match self.call_debug_tool(name, args).await {
                Ok(text) => text_result(&text, false),
                Err(text) => text_result(&text, true),
            };
        }
        // Search is about the project, not one document, so it skips the
        // edit-session machinery entirely (and can never write anything).
        if name == "search" {
            return match call_search_tool(args).await {
                Ok(text) => text_result(&text, false),
                Err(text) => text_result(&text, true),
            };
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

        if let Some(path) = &session_log
            && let Err(e) = crate::doc_tools::record_tool_call(path, &invocation, &outcome)
        {
            // stderr, never stdout: stdout carries the protocol, and a stray
            // line there desynchronizes the client's parser.
            eprintln!("warning: could not record into {}: {e:#}", path.display());
        }
        // A refused tool is `isError: true` so the harness shows it as a
        // failed call — but the text still comes through, because a lineage
        // refusal's message is the instruction for what to do next.
        text_result(&outcome.text, !outcome.ok)
    }

    async fn handle(&mut self, method: &str, params: &Value) -> Result<Value, (i64, String)> {
        match method {
            "initialize" => Ok(json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "hick", "version": env!("CARGO_PKG_VERSION") },
                "instructions":
                    "Edit hick documents through these tools rather than by writing to the \
                     files directly. Read a surface first (read_doc / read_output with \
                     with_lineage), anchor every edit on the 4-hex content hashes it returns, \
                     put CODE changes through edit_output and structure or prose through \
                     edit_doc, and call verify before reporting the work done."
            })),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(tool_catalogue()),
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
async fn call_search_tool(args: &Value) -> Result<String, String> {
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
    let root = std::env::current_dir().map_err(|e| format!("no working directory: {e}"))?;

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

fn text_result(text: &str, is_error: bool) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error,
    })
}

/// Serve MCP on stdin/stdout until the client closes the stream.
///
/// One request at a time, on purpose: the sessions this server holds are
/// single-writer over a document, and interleaving two edits to one file would
/// reintroduce exactly the staleness the design exists to prevent.
pub async fn serve(default_doc: Option<PathBuf>, params: Vec<(String, String)>) -> Result<()> {
    let executor = ExecutorChoice::from_env()?.build().await?;
    let mut server = Server {
        sessions: HashMap::new(),
        executor: executor.clone(),
        params,
        default_doc,
        session_log: crate::doc_tools::session_from(None),
        debuggers: crate::debug_sessions::Registry::new(),
    };

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                // Parse errors have no id to answer against; report and keep
                // serving rather than dropping the connection.
                eprintln!("mcp: ignoring unparseable message: {e}");
                continue;
            }
        };
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let id = request.get("id").cloned();
        let empty = json!({});
        let params = request.get("params").unwrap_or(&empty).clone();

        // A notification (no id) gets no response — answering one is a
        // protocol violation that some clients treat as fatal.
        if id.is_none() {
            continue;
        }

        let response = match server.handle(&method, &params).await {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, message)) => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
            }
        };
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }

    executor.shutdown().await.ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_advertised_tool_has_a_schema_and_a_description() {
        let catalogue = tool_catalogue();
        let tools = catalogue["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        // The same five the built-in agent has. A surface that offers fewer
        // makes bringing your own agent the lesser path.
        assert_eq!(
            names,
            vec![
                "read_doc",
                "read_output",
                "edit_output",
                "edit_doc",
                "verify",
                // Project-wide search: about the folder, not one document,
                // and it can never write anything.
                "search",
                // Debugging: a session an agent drives, which writes
                // nothing.
                "debug_start",
                "debug_state",
                "debug_eval",
                "debug_step",
                "debug_stop"
            ]
        );
        for tool in tools {
            assert!(
                tool["description"].as_str().is_some_and(|d| d.len() > 40),
                "{} needs a description that teaches the doctrine",
                tool["name"]
            );
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    #[test]
    fn mcp_arguments_translate_into_the_tool_vocabulary() {
        let inv = Server::invocation(
            "edit_output",
            &json!({
                "doc": "a.hick",
                "path": "gen.rs",
                "run": "a1b2..c3d4",
                "occurrence": 2,
                "input": "fn main() {}"
            }),
        )
        .unwrap();
        assert_eq!(inv.name, "edit_output");
        assert_eq!(inv.arg("path"), Some("gen.rs"));
        assert_eq!(inv.arg("run"), Some("a1b2..c3d4"));
        assert_eq!(inv.arg("occurrence"), Some("2"));
        assert_eq!(inv.input.as_deref(), Some("fn main() {}"));
        // `doc` selects WHICH document at this layer; it must not leak into
        // the tool arguments, where it means "an upstream document".
        assert_eq!(inv.arg("doc"), None);
    }

    #[test]
    fn upstream_becomes_the_tools_doc_argument() {
        let inv = Server::invocation(
            "edit_doc",
            &json!({ "doc": "primary.hick", "upstream": "requirements.hick", "run": "aaaa" }),
        )
        .unwrap();
        assert_eq!(inv.arg("doc"), Some("requirements.hick"));
    }

    #[test]
    fn with_lineage_is_only_sent_when_asked_for() {
        let on = Server::invocation(
            "read_output",
            &json!({ "path": "g.rs", "with_lineage": true }),
        )
        .unwrap();
        assert_eq!(on.arg("with_lineage"), Some("true"));
        let off = Server::invocation(
            "read_output",
            &json!({ "path": "g.rs", "with_lineage": false }),
        )
        .unwrap();
        assert_eq!(off.arg("with_lineage"), None);
    }

    #[test]
    fn a_refusal_is_an_error_result_that_still_carries_its_text() {
        let result = text_result("anchor run=\"4f20\" matches no lines", true);
        assert_eq!(result["isError"], true);
        assert_eq!(
            result["content"][0]["text"],
            "anchor run=\"4f20\" matches no lines"
        );
    }
}
