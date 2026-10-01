//! Channel `0x02`: the notebook's language-server bridge.
//!
//! The editor in `apps/web` already speaks LSP — it just had nothing to speak
//! to. This is the other half: one `hick-lsp` session per **workspace**,
//! shared by every WebSocket connection (see [`LspHub`]), driven
//! **in-process** rather than by spawning the `hick-lsp` binary, so the desktop app needs nothing on the user's `PATH` to give a
//! document real diagnostics. The child language servers it delegates to
//! (rust-analyzer, pyright, whatever `.hick-lsp.json` names) are the user's,
//! and a missing one degrades exactly as it does in an editor —
//! `docs/guarantees/editor-intelligence/lsp-channel-degrades-never-errors.md`.
//!
//! Two translations happen here, and they are the whole job:
//!
//! * **Framing.** The wire is one JSON-RPC message per WebSocket frame with no
//!   headers (`docs/specs/freeform/api.md`); `tower-lsp` speaks the
//!   `Content-Length` framing of a stdio server. The duplex pipe below is
//!   where headers are added and stripped.
//! * **URIs.** The client names documents `hick:///<doc-path>`, because that
//!   is what it has — it never sees the user's filesystem. `hick-lsp` works in
//!   real paths. Every `uri` crossing this boundary is rewritten, in both
//!   directions, so neither side has to know about the other's scheme.
//!
//! `initialize` is handled here rather than by the client, per the spec: the
//! browser has no business knowing the project root, and the handshake would
//! otherwise have to make a round trip through the network before the first
//! keystroke could be answered.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt as _, AsyncReadExt as _, AsyncWriteExt as _, BufReader};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use hickory_collab::CHANNEL_LSP;

/// The scheme the notebook uses for documents.
const DOC_SCHEME: &str = "hick:///";
/// The scheme the notebook uses for a generated file it can open.
const OUTPUT_SCHEME: &str = "hick-output:///";

/// One `hick-lsp` session for the whole workspace, shared by every socket.
///
/// Per workspace, not per connection, for the reason every IDE has one
/// rust-analyzer rather than one per tab: the child language servers behind
/// `hick-lsp` index the project, and a project indexed once per open file is
/// a machine brought to its knees by opening three files. So the session is
/// started on the first language question anybody asks and kept for the life
/// of the server; each socket **subscribes** and gets its own view of it.
///
/// Three things make sharing honest rather than merely cheaper:
///
/// * **Request ids are per connection.** Every client counts from 1, so two
///   windows' ids collide. Each request is given a fresh server-side id on
///   the way in and its reply is returned under the original id to the one
///   connection that asked. Notifications — diagnostics, the capabilities
///   announcement — go to everybody, and each client already filters by URI.
/// * **An open file is reference-counted.** Two panes on one path are one
///   `didOpen` to the server and one `didClose`, when the last of them goes;
///   a second opener's text arrives as a change instead. Without this the
///   second pane's `didOpen` is a protocol violation and the first pane's
///   `didClose` takes the file away from the other.
/// * **A connection that drops closes what it opened**, and only what it was
///   the last to hold. A window closed mid-session must not leave a language
///   server believing a file is still open.
pub struct LspHub {
    root: PathBuf,
    state: Arc<Mutex<HubState>>,
}

struct HubState {
    session: Option<Session>,
    subscribers: HashMap<u64, mpsc::UnboundedSender<Vec<u8>>>,
    /// Server-side request id → (connection, the client's own id).
    pending: HashMap<i64, (u64, Value)>,
    next_id: i64,
    /// Open document URI (client scheme) → the connections holding it open.
    open: HashMap<String, HashSet<u64>>,
    /// The server's capabilities, replayed to every later subscriber: the
    /// editor cannot decode a semantic token without the legend in here.
    capabilities: Option<Value>,
}

/// The live session: the in-process server and the tasks that feed it.
struct Session {
    to_server: mpsc::UnboundedSender<Value>,
    tasks: Vec<JoinHandle<()>>,
}

impl Drop for Session {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

impl LspHub {
    /// A dependency action changed host environments; refresh existing children.
    pub fn refresh_environments(&self) {
        if let Some(session) = &self.state.lock().expect("hub state").session {
            let _ = session.to_server.send(serde_json::json!({"jsonrpc":"2.0", "method":"workspace/didChangeConfiguration", "params":{"settings":{}}}));
        }
    }

    /// A hub for the workspace at `root`. Nothing is started until the first
    /// subscriber sends something: a session that never asks a language
    /// question must never spawn a language server.
    pub fn new(root: &Path) -> Self {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        Self {
            root,
            state: Arc::new(Mutex::new(HubState {
                session: None,
                subscribers: HashMap::new(),
                pending: HashMap::new(),
                next_id: 1,
                open: HashMap::new(),
                capabilities: None,
            })),
        }
    }

    /// Attach a connection. Its replies and every notification arrive as
    /// framed `0x02` messages on `to_client`.
    pub fn subscribe(&self, conn: u64, to_client: mpsc::UnboundedSender<Vec<u8>>) -> Result<()> {
        let mut state = self.state.lock().expect("hub state");
        if state.session.is_none() {
            state.session = Some(Session::start(&self.root, Arc::clone(&self.state))?);
        }
        if let Some(capabilities) = &state.capabilities {
            let _ = to_client.send(frame(&capabilities_announcement(capabilities)));
        }
        state.subscribers.insert(conn, to_client);
        Ok(())
    }

    /// Detach a connection, closing whatever it was the last to hold open.
    pub fn unsubscribe(&self, conn: u64) {
        let mut state = self.state.lock().expect("hub state");
        state.subscribers.remove(&conn);
        state.pending.retain(|_, (owner, _)| *owner != conn);
        let mut orphaned = Vec::new();
        state.open.retain(|uri, holders| {
            holders.remove(&conn);
            if holders.is_empty() {
                orphaned.push(uri.clone());
                false
            } else {
                true
            }
        });
        if let Some(session) = &state.session {
            for uri in orphaned {
                let _ = session.to_server.send(serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/didClose",
                    "params": { "textDocument": { "uri": uri } },
                }));
            }
        }
    }

    /// Forward one client message from `conn` to the language server.
    ///
    /// Queued behind the handshake rather than raced against it, so a client
    /// that opens a document the instant the socket is up is answered rather
    /// than ignored.
    pub fn send(&self, conn: u64, mut message: Value) -> Result<()> {
        let mut state = self.state.lock().expect("hub state");
        if !state.subscribers.contains_key(&conn) {
            anyhow::bail!("connection {conn} is not subscribed to the language server");
        }
        let method = message
            .get("method")
            .and_then(Value::as_str)
            .map(str::to_string);
        let uri = message
            .pointer("/params/textDocument/uri")
            .and_then(Value::as_str)
            .map(str::to_string);
        match (method.as_deref(), uri) {
            (Some("textDocument/didOpen"), Some(uri)) => {
                let holders = state.open.entry(uri).or_default();
                let first = holders.is_empty();
                holders.insert(conn);
                if !first {
                    // Somebody else already opened it: this pane's text is a
                    // change to the open file, not a second opening.
                    message = did_open_as_change(message);
                }
            }
            (Some("textDocument/didClose"), Some(uri)) => {
                let still_open = match state.open.get_mut(&uri) {
                    Some(holders) => {
                        holders.remove(&conn);
                        !holders.is_empty()
                    }
                    None => false,
                };
                if still_open {
                    return Ok(());
                }
                state.open.remove(&uri);
            }
            _ => {}
        }
        if let Some(client_id) = message.get("id").cloned()
            && method.is_some()
        {
            let server_id = state.next_id;
            state.next_id += 1;
            state.pending.insert(server_id, (conn, client_id));
            message["id"] = Value::from(server_id);
        }
        let Some(session) = &state.session else {
            anyhow::bail!("the language server session has not started");
        };
        session
            .to_server
            .send(message)
            .context("the language server session has ended")
    }
}

/// A `didOpen` re-expressed as the full-text `didChange` the server's FULL
/// sync accepts, for a file it already has open.
fn did_open_as_change(mut message: Value) -> Value {
    let uri = message
        .pointer("/params/textDocument/uri")
        .cloned()
        .unwrap_or(Value::Null);
    let version = message
        .pointer("/params/textDocument/version")
        .cloned()
        .unwrap_or(Value::from(1));
    let text = message
        .pointer("/params/textDocument/text")
        .cloned()
        .unwrap_or(Value::from(""));
    message["method"] = Value::from("textDocument/didChange");
    message["params"] = serde_json::json!({
        "textDocument": { "uri": uri, "version": version },
        "contentChanges": [{ "text": text }],
    });
    message
}

fn capabilities_announcement(capabilities: &Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "hick/serverCapabilities",
        "params": { "capabilities": capabilities },
    })
}

/// One JSON-RPC message as a `0x02` frame.
fn frame(message: &Value) -> Vec<u8> {
    let mut frame = Vec::with_capacity(64);
    frame.push(CHANNEL_LSP);
    if let Ok(json) = serde_json::to_vec(message) {
        frame.extend_from_slice(&json);
    }
    frame
}

impl HubState {
    /// Deliver one message from the server: a reply to the connection that
    /// asked, anything else to everybody.
    fn dispatch(&mut self, mut message: Value) {
        let is_reply = message.get("id").is_some() && message.get("method").is_none();
        if is_reply {
            let Some(server_id) = message.get("id").and_then(Value::as_i64) else {
                return;
            };
            let Some((conn, client_id)) = self.pending.remove(&server_id) else {
                return;
            };
            message["id"] = client_id;
            if let Some(tx) = self.subscribers.get(&conn)
                && tx.send(frame(&message)).is_err()
            {
                self.subscribers.remove(&conn);
            }
            return;
        }
        let bytes = frame(&message);
        self.subscribers
            .retain(|_, tx| tx.send(bytes.clone()).is_ok());
    }
}

impl Session {
    /// Start the in-process server rooted at `root`, delivering what it says
    /// through `hub`.
    fn start(root: &Path, hub: Arc<Mutex<HubState>>) -> Result<Self> {
        let root = root.to_path_buf();
        let root_uri = path_to_file_uri(&root);

        // 64 KiB each way: a completion response over a large file is the
        // biggest thing that crosses here, and a pipe too small to hold one
        // deadlocks the writer against a reader that is waiting for the rest.
        let (bridge_end, server_end) = tokio::io::duplex(64 * 1024);
        let (server_read, server_write) = tokio::io::split(server_end);
        let (mut bridge_read, mut bridge_write) = tokio::io::split(bridge_end);

        let (service, socket) = tower_lsp::LspService::new(hick_lsp::HickBackend::new);
        let server = tokio::spawn(async move {
            tower_lsp::Server::new(server_read, server_write, socket)
                .serve(service)
                .await;
        });

        // The handshake gate. `tower-lsp` DISCARDS every notification that
        // arrives before it has sent the initialize response — silently, since
        // a notification has no reply to carry an error. Sending `initialized`
        // and the client's first `didOpen` straight after `initialize` gets
        // them both dropped, and the symptom is a language channel that
        // connects, accepts everything, and answers nothing. So the writer
        // holds the client's traffic until the reader has seen the response.
        let (initialized_tx, initialized_rx) = tokio::sync::oneshot::channel::<()>();

        let (to_server, mut outbox) = mpsc::unbounded_channel::<Value>();
        let root_for_writer = root.clone();
        let writer = tokio::spawn(async move {
            if write_framed(&mut bridge_write, &initialize_request(&root_uri))
                .await
                .is_err()
            {
                return;
            }
            if initialized_rx.await.is_err() {
                return;
            }
            if write_framed(&mut bridge_write, &initialized_notification())
                .await
                .is_err()
            {
                return;
            }
            while let Some(mut message) = outbox.recv().await {
                rewrite_uris(&mut message, &|uri| {
                    client_uri_to_server(uri, &root_for_writer)
                });
                if write_framed(&mut bridge_write, &message).await.is_err() {
                    break;
                }
            }
        });

        let root_for_reader = root.clone();
        let reader = tokio::spawn(async move {
            let mut stream = BufReader::new(&mut bridge_read);
            let mut initialized_tx = Some(initialized_tx);
            while let Ok(Some(mut message)) = read_framed(&mut stream).await {
                // The initialize response is this bridge's own business; the
                // client never sent the request, so it must not get the reply.
                //
                // It does get the *capabilities*, as a notification. The
                // editor cannot decode semantic tokens without the server's
                // legend — the token types are integers indexing into it —
                // and it should not draw a rename affordance for a server
                // that cannot rename. Sending the capabilities keeps the
                // browser out of the handshake without keeping it ignorant.
                if message.get("id").and_then(Value::as_i64) == Some(INITIALIZE_ID) {
                    if let Some(tx) = initialized_tx.take() {
                        let _ = tx.send(());
                    }
                    let capabilities = message
                        .get("result")
                        .and_then(|result| result.get("capabilities"))
                        .cloned()
                        .unwrap_or(Value::Null);
                    let mut state = hub.lock().expect("hub state");
                    state.dispatch(capabilities_announcement(&capabilities));
                    state.capabilities = Some(capabilities);
                    continue;
                }
                rewrite_uris(&mut message, &|uri| {
                    server_uri_to_client(uri, &root_for_reader)
                });
                hub.lock().expect("hub state").dispatch(message);
            }
        });

        Ok(Self {
            to_server,
            tasks: vec![server, writer, reader],
        })
    }
}

/// The handshake `initialize`, which this bridge performs on the client's
/// behalf: the browser has no business knowing the project root, and a
/// round trip through the network before the first keystroke could be
/// answered would be latency for nothing.
fn initialize_request(root_uri: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": INITIALIZE_ID,
        "method": "initialize",
        "params": {
            "processId": std::process::id(),
            "rootUri": root_uri,
            // Everything the notebook can render is declared, because a
            // server that is not asked does not answer: several children
            // gate a feature on the client claiming it, so under-declaring
            // here silently removes the feature from the editor.
            "capabilities": {
                "textDocument": {
                    "publishDiagnostics": { "relatedInformation": false },
                    "hover": { "contentFormat": ["markdown", "plaintext"] },
                    "completion": {
                        "completionItem": {
                            "snippetSupport": false,
                            "documentationFormat": ["markdown", "plaintext"],
                            "resolveSupport": { "properties": ["documentation", "detail"] },
                        }
                    },
                    "definition": {},
                    "declaration": {},
                    "typeDefinition": {},
                    "implementation": {},
                    "references": {},
                    "documentHighlight": {},
                    "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
                    "signatureHelp": {
                        "signatureInformation": {
                            "documentationFormat": ["markdown", "plaintext"],
                            "parameterInformation": { "labelOffsetSupport": true },
                        }
                    },
                    "semanticTokens": {
                        "requests": { "full": true },
                        // The server's own legend, not a copy of it: a second
                        // list here would drift, and the drift shows up as
                        // code coloured as the wrong kind of thing.
                        "tokenTypes": hick_lsp::semantic::legend_types(),
                        "tokenModifiers": hick_lsp::semantic::legend_modifiers(),
                        "formats": ["relative"],
                    },
                    "inlayHint": { "dynamicRegistration": false },
                    "foldingRange": { "lineFoldingOnly": false },
                    "selectionRange": {},
                    "codeAction": {
                        "codeActionLiteralSupport": {
                            "codeActionKind": {
                                "valueSet": ["quickfix", "refactor", "source"],
                            }
                        }
                    },
                    "codeLens": {},
                    "formatting": {},
                    "rename": { "prepareSupport": true },
                },
                "workspace": { "symbol": {} },
            },
        },
    })
}

fn initialized_notification() -> Value {
    serde_json::json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })
}

/// Request id for the handshake. Negative so it cannot collide with the
/// client's ids, which start at 1 and count up.
const INITIALIZE_ID: i64 = -1;

// ---------------------------------------------------------------------------
// Framing
// ---------------------------------------------------------------------------

async fn write_framed<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    message: &Value,
) -> Result<()> {
    let body = serde_json::to_vec(message)?;
    writer
        .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
        .await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}

async fn read_framed<R: tokio::io::AsyncBufRead + Unpin>(reader: &mut R) -> Result<Option<Value>> {
    let mut length: Option<usize> = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            length = value.trim().parse().ok();
        }
    }
    let Some(length) = length else {
        return Ok(None);
    };
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).await?;
    Ok(serde_json::from_slice(&body).ok())
}

// ---------------------------------------------------------------------------
// URI translation
// ---------------------------------------------------------------------------

/// Rewrite every `uri`-shaped string in `message` with `f`.
///
/// Keyed on the field name rather than on anything about the value, because
/// LSP puts URIs in a dozen shapes (`textDocument.uri`, `location.uri`,
/// `locationLink.targetUri`, `changes` keyed BY uri) and a rewriter that
/// pattern-matched on strings would eventually rewrite a URI inside a hover
/// snippet — i.e. inside the user's own code.
fn rewrite_uris(message: &mut Value, f: &dyn Fn(&str) -> Option<String>) {
    match message {
        Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if (key == "uri" || key == "targetUri" || key == "rootUri")
                    && let Some(text) = value.as_str()
                    && let Some(rewritten) = f(text)
                {
                    *value = Value::String(rewritten);
                    continue;
                }
                rewrite_uris(value, f);
            }
        }
        Value::Array(items) => {
            for item in items {
                rewrite_uris(item, f);
            }
        }
        _ => {}
    }
}

/// `hick:///docs/a.md` → `file:///…/docs/a.md`.
///
/// A path that escapes the project root is refused rather than translated: the
/// browser is not a trusted source of filesystem paths, and `hick:///../..`
/// would otherwise be a way to ask the language server about any file on the
/// machine.
fn client_uri_to_server(uri: &str, root: &Path) -> Option<String> {
    let rel = uri
        .strip_prefix(DOC_SCHEME)
        .or_else(|| uri.strip_prefix(OUTPUT_SCHEME))?;
    let rel = percent_decode(rel);
    if Path::new(&rel).is_absolute() || rel.split('/').any(|part| part == "..") {
        return None;
    }
    Some(path_to_file_uri(&root.join(rel)))
}

/// `file:///…/docs/a.md` → `hick:///docs/a.md`, and a virtual output file
/// → `hick-output:///<path>` so the client can open the Output view there.
fn server_uri_to_client(uri: &str, root: &Path) -> Option<String> {
    let path = file_uri_to_path(uri)?;
    if let Ok(rel) = path.strip_prefix(root) {
        return Some(format!(
            "{DOC_SCHEME}{}",
            rel.to_string_lossy().replace('\\', "/")
        ));
    }
    // hick-lsp stages the files a document *would* write under a temp
    // directory, one per document. Those are not files the user has, so they
    // are named by the output path the document gives them. Where that
    // directory is, and how to read a path under it, belongs to `hick-lsp`
    // rather than to a string match here: it moved once already, from a
    // hardcoded `/tmp` that did not exist on Windows.
    let rel = hick_lsp::staged_output_path(&path)?;
    Some(format!("{OUTPUT_SCHEME}{rel}"))
}

fn path_to_file_uri(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        // Windows: `C:/x` → `file:///C:/x`.
        format!("file:///{text}")
    }
}

fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let rest = rest
        .strip_prefix('/')
        .map(|r| r.to_string())
        .unwrap_or_else(|| rest.to_string());
    let decoded = percent_decode(&rest);
    // A Windows path came back as `C:/x`; a unix one lost its leading slash.
    if decoded.chars().nth(1) == Some(':') {
        Some(PathBuf::from(decoded))
    } else {
        Some(PathBuf::from(format!("/{decoded}")))
    }
}

/// Decode the escapes a URI can carry. Only `%XX` matters here — these are
/// paths, not query strings, so `+` is a literal plus.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        PathBuf::from("/home/u/project")
    }

    #[test]
    fn a_document_uri_round_trips() {
        let server = client_uri_to_server("hick:///docs/a.hick", &root()).unwrap();
        assert_eq!(server, "file:///home/u/project/docs/a.hick");
        assert_eq!(
            server_uri_to_client(&server, &root()).unwrap(),
            "hick:///docs/a.hick"
        );
    }

    #[test]
    fn a_path_escaping_the_project_is_refused() {
        // The browser is not a trusted source of paths.
        assert!(client_uri_to_server("hick:///../../etc/passwd", &root()).is_none());
        assert!(client_uri_to_server("hick:///a/../../b", &root()).is_none());
        assert!(client_uri_to_server("hick:////etc/passwd", &root()).is_none());
    }

    #[test]
    fn percent_escapes_survive_the_trip() {
        let server = client_uri_to_server("hick:///docs/a%20b.hick", &root()).unwrap();
        assert_eq!(server, "file:///home/u/project/docs/a b.hick");
    }

    #[test]
    fn a_virtual_output_file_becomes_an_output_uri() {
        // The staging directory is made fresh per run under the user's own
        // temp directory, so the shape — not a fixed path — is what this
        // asserts. `hick_lsp::STAGING_PREFIX` is the half both sides share.
        let uri = format!(
            "file:///tmp/{}ab12/9f2a/src/stats.py",
            hick_lsp::STAGING_PREFIX
        );
        assert_eq!(
            server_uri_to_client(&uri, &root()).unwrap(),
            "hick-output:///src/stats.py"
        );
    }

    #[test]
    fn a_uri_outside_both_worlds_is_left_alone() {
        assert!(server_uri_to_client("file:///usr/lib/python3/typing.py", &root()).is_none());
    }

    #[test]
    fn rewriting_touches_uri_fields_and_nothing_else() {
        let mut message = serde_json::json!({
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": "file:///home/u/project/a.hick",
                "diagnostics": [{
                    "message": "see file:///home/u/project/a.hick for context",
                    "relatedInformation": [{ "location": { "uri": "file:///home/u/project/b.hick" } }],
                }],
            },
        });
        rewrite_uris(&mut message, &|uri| server_uri_to_client(uri, &root()));
        assert_eq!(message["params"]["uri"], "hick:///a.hick");
        assert_eq!(
            message["params"]["diagnostics"][0]["relatedInformation"][0]["location"]["uri"],
            "hick:///b.hick"
        );
        // Prose that merely contains a URI is the user's text, not a field.
        assert!(
            message["params"]["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("file:///home/u/project/a.hick")
        );
    }

    #[tokio::test]
    async fn the_bridge_answers_a_real_document_with_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let doc = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                   <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
                   # Title\n\
                   </hick:doc>\n";
        std::fs::write(dir.path().join("a.hick"), doc).unwrap();

        let (to_client, mut from_server) = mpsc::unbounded_channel();
        let hub = LspHub::new(dir.path());
        hub.subscribe(1, to_client).unwrap();
        hub.send(
            1,
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": { "textDocument": {
                    "uri": "hick:///a.hick", "languageId": "hick", "version": 1, "text": doc,
                }},
            }),
        )
        .unwrap();

        // A well-formed document still publishes — an empty diagnostic list is
        // the answer that clears the editor's gutter. The server's own
        // `window/logMessage` chatter arrives first and is not what is being
        // waited for.
        //
        // The capabilities announcement is collected on the way past: the
        // editor cannot colour anything without the legend it carries.
        let mut capabilities = Value::Null;
        let message = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                let frame = from_server.recv().await.expect("channel open");
                assert_eq!(frame[0], CHANNEL_LSP, "everything here is channel 0x02");
                let message: Value = serde_json::from_slice(&frame[1..]).unwrap();
                if message["method"] == "hick/serverCapabilities" {
                    capabilities = message["params"]["capabilities"].clone();
                }
                if message["method"] == "textDocument/publishDiagnostics" {
                    return message;
                }
            }
        })
        .await
        .expect("the bridge should publish diagnostics for an opened document");

        // The browser never sends `initialize`, so this notification is the
        // only way it learns what the server can do — and the legend's ORDER
        // is the meaning of every token type integer that follows.
        let legend = &capabilities["semanticTokensProvider"]["legend"]["tokenTypes"];
        assert!(
            legend.as_array().is_some_and(|types| !types.is_empty()),
            "no semantic token legend reached the client: {capabilities}"
        );
        assert!(
            capabilities["renameProvider"] != Value::Null,
            "the editor was not told the server can rename: {capabilities}"
        );
        // And the reply to the handshake itself must NOT reach the client:
        // it answers a request the client never made.
        assert!(capabilities["__id"].is_null());
        // Named in the client's scheme, never in the user's filesystem.
        assert_eq!(message["params"]["uri"], "hick:///a.hick");
    }

    /// Protects docs/guarantees/editor-intelligence/one-language-server-per-workspace.md
    #[tokio::test]
    async fn two_windows_share_one_session_and_each_gets_its_own_replies() {
        let dir = tempfile::tempdir().unwrap();
        let doc = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                   <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
                   # Title\n\
                   </hick:doc>\n";
        std::fs::write(dir.path().join("a.hick"), doc).unwrap();
        let hub = LspHub::new(dir.path());

        let (to_one, mut one) = mpsc::unbounded_channel();
        let (to_two, mut two) = mpsc::unbounded_channel();
        hub.subscribe(1, to_one).unwrap();
        let open = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "hick:///a.hick", "languageId": "hick", "version": 1, "text": doc,
            }},
        });
        hub.send(1, open.clone()).unwrap();
        // Both windows ask with id 7, as every client counting from 1 will
        // eventually collide. Each must get ITS answer back under ITS id.
        let symbols = |id: i64| {
            serde_json::json!({
                "jsonrpc": "2.0", "id": id, "method": "textDocument/documentSymbol",
                "params": { "textDocument": { "uri": "hick:///a.hick" } },
            })
        };
        async fn wait_reply(rx: &mut mpsc::UnboundedReceiver<Vec<u8>>) -> Value {
            tokio::time::timeout(std::time::Duration::from_secs(10), async {
                loop {
                    let frame = rx.recv().await.expect("channel open");
                    let message: Value = serde_json::from_slice(&frame[1..]).unwrap();
                    if message.get("id").is_some() && message.get("method").is_none() {
                        return message;
                    }
                }
            })
            .await
            .expect("a reply")
        }
        hub.send(1, symbols(7)).unwrap();
        let reply = wait_reply(&mut one).await;
        assert_eq!(
            reply["id"], 7,
            "the reply carries the client's own id: {reply}"
        );

        // The second window arrives after the handshake: it still learns the
        // capabilities, because the editor cannot colour without the legend.
        hub.subscribe(2, to_two).unwrap();
        let first = two.recv().await.unwrap();
        let announced: Value = serde_json::from_slice(&first[1..]).unwrap();
        assert_eq!(announced["method"], "hick/serverCapabilities");
        // Its didOpen of the same file is a change, not a second opening —
        // and its own request with the same id is answered to it alone.
        hub.send(2, open).unwrap();
        hub.send(2, symbols(7)).unwrap();
        let reply = wait_reply(&mut two).await;
        assert_eq!(reply["id"], 7);
        assert!(
            one.try_recv().is_err()
                || serde_json::from_slice::<Value>(&one.try_recv().unwrap()[1..])
                    .unwrap()
                    .get("method")
                    .is_some(),
            "window one must not receive window two's reply"
        );

        // Window one closing does not take the file away from window two:
        // its didClose is held until the last holder goes.
        hub.unsubscribe(1);
        {
            let state = hub.state.lock().unwrap();
            assert!(state.open.contains_key("hick:///a.hick"));
            assert_eq!(state.open["hick:///a.hick"].len(), 1);
        }
        hub.unsubscribe(2);
        assert!(hub.state.lock().unwrap().open.is_empty());
    }
}
