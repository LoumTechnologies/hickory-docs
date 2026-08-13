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
}

impl Server {
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
                "verify"
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
