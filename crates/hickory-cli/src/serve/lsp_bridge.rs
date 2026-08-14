//! Channel `0x02`: the notebook's language-server bridge.
//!
//! The editor in `apps/web` already speaks LSP — it just had nothing to speak
//! to. This is the other half: one `hick-lsp` session per WebSocket
//! connection, driven **in-process** rather than by spawning the `hick-lsp`
//! binary, so the desktop app needs nothing on the user's `PATH` to give a
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

use std::path::{Path, PathBuf};

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

/// A live `hick-lsp` session for one connection.
pub struct LspBridge {
    /// JSON-RPC messages headed for the language server.
    to_server: mpsc::UnboundedSender<Value>,
    tasks: Vec<JoinHandle<()>>,
}

impl Drop for LspBridge {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

impl LspBridge {
    /// Start a session rooted at `root`, sending framed replies to `to_client`.
    ///
    /// Started lazily, on the connection's first `0x02` frame: a session that
    /// never asks a language question must never spawn a language server.
    pub fn start(root: &Path, to_client: mpsc::UnboundedSender<Vec<u8>>) -> Result<Self> {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
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
                    let announcement = serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "hick/serverCapabilities",
                        "params": { "capabilities": capabilities },
                    });
                    let mut frame = Vec::with_capacity(64);
                    frame.push(CHANNEL_LSP);
                    if let Ok(json) = serde_json::to_vec(&announcement) {
                        frame.extend_from_slice(&json);
                        if to_client.send(frame).is_err() {
                            break;
                        }
                    }
                    continue;
                }
                rewrite_uris(&mut message, &|uri| {
                    server_uri_to_client(uri, &root_for_reader)
                });
                let mut frame = Vec::with_capacity(64);
                frame.push(CHANNEL_LSP);
                match serde_json::to_vec(&message) {
                    Ok(json) => frame.extend_from_slice(&json),
                    Err(_) => continue,
                }
                if to_client.send(frame).is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            to_server,
            tasks: vec![server, writer, reader],
        })
    }

    /// Forward one client message to the language server.
    ///
    /// Queued behind the handshake rather than raced against it, so a client
    /// that opens a document the instant the socket is up is answered rather
    /// than ignored.
    pub fn send(&self, message: Value) -> Result<()> {
        self.to_server
            .send(message)
            .context("the language server session has ended")
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

/// `hick:///docs/a.hick` → `file:///…/docs/a.hick`.
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

/// `file:///…/docs/a.hick` → `hick:///docs/a.hick`, and a virtual output file
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
    // are named by the output path the document gives them.
    let text = path.to_string_lossy().replace('\\', "/");
    let rest = text.split("/hick-lsp-vfiles/").nth(1)?;
    let rel = rest.split_once('/').map(|(_hash, rel)| rel)?;
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
        let uri = "file:///tmp/hick-lsp-vfiles/9f2a/src/stats.py";
        assert_eq!(
            server_uri_to_client(uri, &root()).unwrap(),
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
        let bridge = LspBridge::start(dir.path(), to_client).unwrap();
        bridge
            .send(serde_json::json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": { "textDocument": {
                    "uri": "hick:///a.hick", "languageId": "hick", "version": 1, "text": doc,
                }},
            }))
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
}
