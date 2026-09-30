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
                "name": "read_file",
                "description":
                    "Read any file of the project, read-only, hashline-rendered — a data \
                     export, a config, a source file. Relative to the document's directory; \
                     nothing outside the project. A directory lists its entries. The read is \
                     recorded in the session as context: which file, at which hash and commit, \
                     which lines were in front of the agent when it later wrote.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "doc": doc_arg,
                        "path": { "type": "string", "description": "File path, relative to the document's directory." },
                        "from": { "type": "integer", "description": "First line (1-based)." },
                        "to": { "type": "integer", "description": "Last line (1-based, inclusive)." }
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
                "name": "list_docs",
                "description":
                    "List the project's hick documents: every `.hick` file, what it weaves, \
                     and the files it generates. Start here — every other tool takes a `doc`, \
                     and this is the only way to learn what documents exist. Gitignored \
                     directories are skipped, so build output never appears.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "under": {
                            "type": "string",
                            "description": "Only list documents under this directory, relative \
                                            to the project root. Omit for the whole project."
                        }
                    }
                }
            },
            {
                "name": "create_doc",
                "description":
                    "Create a new hick document. The body is BARE markdown — no `<hick:doc>` \
                     wrapper, which is optional and which ordinary documents omit; the weave \
                     defaults to the document's own name. Refuses to overwrite an existing \
                     file, so this can never destroy work: to change a document that already \
                     exists, use edit_doc. Creates parent directories.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Where to create it, relative to the project root. \
                                            `.hick` is appended if absent."
                        },
                        "input": {
                            "type": "string",
                            "description": "The document body: markdown, and any hick elements \
                                            it needs. Omit for an empty document."
                        }
                    },
                    "required": ["path"]
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
pub(crate) struct Server {
    root: PathBuf,
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
    pub(crate) fn embedded(
        root: PathBuf,
        doc: PathBuf,
        executor: Arc<dyn Executor>,
        session: PathBuf,
    ) -> Self {
        Self {
            root,
            sessions: HashMap::new(),
            executor,
            params: Vec::new(),
            default_doc: Some(doc),
            session_log: Some(session),
            debuggers: crate::debug_sessions::Registry::new(),
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
            return Ok(self.root.join(d));
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
            return match call_search_tool(&self.root, args).await {
                Ok(text) => text_result(&text, false),
                Err(text) => text_result(&text, true),
            };
        }
        // Listing and creating are about the project, not one document, so
        // they route the way `search` does. They exist because every other
        // tool takes a `doc` and, without these, an agent arriving in a
        // project had no way to learn what documents there were or to start
        // a new one — it had to leave the tool surface and write the file
        // itself, which is the one thing the surface asks it not to do.
        if name == "list_docs" {
            return match call_list_docs_tool(&self.root, args) {
                Ok(text) => text_result(&text, false),
                Err(text) => text_result(&text, true),
            };
        }
        if name == "create_doc" {
            return match call_create_doc_tool(&self.root, args) {
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

/// `list_docs`: every `.hick` document in the project, with what it weaves.
///
/// Gitignore-aware, for the same reason `hick ingest` is: a project's build
/// output routinely contains `.hick` fixtures, and listing them as if they
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
            // A session is a `.hick` file with a `<hick:session>` root, not a
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
        rel.set_extension("hick");
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
    std::fs::write(&full, &body).map_err(|e| format!("could not write {}: {e}", rel.display()))?;
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
pub use stdio::serve;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_advertised_tool_has_a_schema_and_a_description() {
        let catalogue = tool_catalogue();
        let tools = catalogue["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        // The six the built-in agent has, plus the two an external agent
        // needs and the built-in one does not: it is started ON a document,
        // while an MCP client arrives at a folder and has to find out what
        // is in it. A surface that offers fewer makes bringing your own
        // agent the lesser path.
        assert_eq!(
            names,
            vec![
                "read_doc",
                "read_output",
                "read_file",
                "edit_output",
                "edit_doc",
                "verify",
                // Finding and starting documents: about the folder, not one
                // document. Without these an agent had to leave the tool
                // surface to learn what existed or to begin anything.
                "list_docs",
                "create_doc",
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
