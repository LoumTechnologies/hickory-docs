//! Child LSP process management.
//!
//! Manages spawning and communicating with child LSP servers (e.g.,
//! rust-analyzer, pyright) via JSON-RPC over stdio.
//!
//! The I/O is split: a writer half sends requests/notifications to the child's
//! stdin, while a background tokio task reads from stdout and dispatches
//! responses (to waiting request futures) and notifications (to a channel).

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;

/// A notification received from a child LSP server.
#[derive(Debug, Clone)]
pub struct ChildNotification {
    /// The language ID of the child that sent this notification.
    pub language_id: String,
    /// The JSON-RPC method name (e.g., "textDocument/publishDiagnostics").
    pub method: String,
    /// The notification params.
    pub params: Value,
}

/// Handle for communicating with a child LSP server.
///
/// The handle is cheaply cloneable (all state is behind `Arc`). A background
/// task continuously reads the child's stdout, dispatching responses to waiting
/// callers and forwarding notifications to a shared channel.
#[derive(Clone)]
pub struct ChildLspHandle {
    language_id: String,
    stdin: Arc<Mutex<tokio::io::BufWriter<tokio::process::ChildStdin>>>,
    next_id: Arc<AtomicI64>,
    pending: Arc<Mutex<HashMap<i64, oneshot::Sender<Value>>>>,
    process: Arc<Mutex<Child>>,
    _reader_handle: Arc<JoinHandle<()>>,
}

impl ChildLspHandle {
    /// Spawn a new child LSP server and start the background reader.
    ///
    /// Returns the handle and an unbounded receiver for notifications from
    /// this child. The caller should drain the receiver to process
    /// notifications like `textDocument/publishDiagnostics`.
    pub async fn spawn(
        language_id: &str,
        notification_tx: mpsc::UnboundedSender<ChildNotification>,
    ) -> Result<Self, ChildLspError> {
        let cmd_parts = lsp_command(language_id)?;
        let program = &cmd_parts[0];
        let args = &cmd_parts[1..];

        let mut child = Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|source| ChildLspError::SpawnFailed {
                command: cmd_parts.join(" "),
                source,
            })?;

        let child_stdin = child
            .stdin
            .take()
            .expect("stdin was configured as piped but take() returned None; this is a tokio bug");
        let child_stdout = child
            .stdout
            .take()
            .expect("stdout was configured as piped but take() returned None; this is a tokio bug");

        let pending: Arc<Mutex<HashMap<i64, oneshot::Sender<Value>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let lang_id = language_id.to_string();
        let pending_clone = Arc::clone(&pending);
        let reader_handle = tokio::spawn(async move {
            let mut stdout = BufReader::new(child_stdout);
            while let Ok(msg) = read_message(&mut stdout).await {
                // Response (has "id" and no "method")?
                if let Some(id) = msg.get("id").and_then(|v| v.as_i64())
                    && msg.get("method").is_none()
                {
                    let mut pending = pending_clone.lock().await;
                    if let Some(tx) = pending.remove(&id) {
                        let _ = tx.send(msg);
                    }
                    continue;
                }
                // Notification (has "method", no "id" or "id" is null)?
                if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
                    let _ = notification_tx.send(ChildNotification {
                        language_id: lang_id.clone(),
                        method: method.to_string(),
                        params: msg.get("params").cloned().unwrap_or(Value::Null),
                    });
                }
            }
            // Drop all pending response senders so that callers waiting on
            // rx.await get Err(Canceled) instead of blocking forever.
            let mut pending = pending_clone.lock().await;
            pending.clear();
        });

        Ok(Self {
            language_id: language_id.to_string(),
            stdin: Arc::new(Mutex::new(tokio::io::BufWriter::new(child_stdin))),
            next_id: Arc::new(AtomicI64::new(1)),
            pending,
            process: Arc::new(Mutex::new(child)),
            _reader_handle: Arc::new(reader_handle),
        })
    }

    /// Send an LSP request and wait for the matching response.
    pub async fn request(&self, method: &str, params: Value) -> Result<Value, ChildLspError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, tx);
        }

        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        send_message(&self.stdin, &message).await?;

        let response = rx.await.map_err(|_| ChildLspError::ProcessExited)?;

        if let Some(error) = response.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error")
                .to_string();
            return Err(ChildLspError::JsonRpc { code, message });
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }

    /// Send an LSP notification (no response expected).
    pub async fn notify(&self, method: &str, params: Value) -> Result<(), ChildLspError> {
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        send_message(&self.stdin, &message).await
    }

    /// Initialize the child LSP server with the given workspace root.
    pub async fn initialize(&self, root_uri: &str) -> Result<Value, ChildLspError> {
        let params = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": {
                "workspace": { "symbol": {}, "workspaceEdit": { "documentChanges": true } },
                "textDocument": {
                    "publishDiagnostics": {
                        "relatedInformation": false
                    },
                    "completion": {
                        "completionItem": {
                            "snippetSupport": false
                        }
                    },
                    "hover": {
                        "contentFormat": ["plaintext", "markdown"]
                    },
                    "definition": {},
                    // Declared because a server that is not asked for a
                    // feature does not implement it: pyright and
                    // rust-analyzer both gate semantic tokens, inlay hints
                    // and code actions on the client saying it wants them.
                    "declaration": {},
                    "typeDefinition": {},
                    "implementation": {},
                    "references": {},
                    "documentHighlight": {},
                    "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
                    "signatureHelp": { "signatureInformation": { "documentationFormat": ["markdown", "plaintext"] } },
                    "codeAction": {
                        "codeActionLiteralSupport": {
                            "codeActionKind": { "valueSet": ["quickfix", "refactor", "source"] }
                        }
                    },
                    "rename": { "prepareSupport": true },
                    "formatting": {},
                    "rangeFormatting": {},
                    "foldingRange": { "lineFoldingOnly": true },
                    "selectionRange": {},
                    "inlayHint": {},
                    "codeLens": {},
                    "semanticTokens": {
                        "requests": { "full": true, "range": true },
                        "tokenTypes": [
                            "namespace", "type", "class", "enum", "interface", "struct",
                            "typeParameter", "parameter", "variable", "property", "enumMember",
                            "event", "function", "method", "macro", "keyword", "modifier",
                            "comment", "string", "number", "regexp", "operator", "decorator"
                        ],
                        "tokenModifiers": [
                            "declaration", "definition", "readonly", "static", "deprecated",
                            "abstract", "async", "modification", "documentation", "defaultLibrary"
                        ],
                        "formats": ["relative"]
                    }
                }
            }
        });

        let result = self.request("initialize", params).await?;
        self.notify("initialized", serde_json::json!({})).await?;

        Ok(result)
    }

    /// Shut down the child LSP server gracefully.
    pub async fn shutdown(&self) -> Result<(), ChildLspError> {
        let _ = self.request("shutdown", Value::Null).await;
        let _ = self.notify("exit", Value::Null).await;
        let _ = self.process.lock().await.kill().await;
        Ok(())
    }

    pub fn language_id(&self) -> &str {
        &self.language_id
    }
}

// ------------------------------------------------------------------
// Free-standing I/O helpers
// ------------------------------------------------------------------

async fn send_message(
    stdin: &Arc<Mutex<tokio::io::BufWriter<tokio::process::ChildStdin>>>,
    message: &Value,
) -> Result<(), ChildLspError> {
    let body = serde_json::to_string(message)?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());

    let mut stdin = stdin.lock().await;
    stdin.write_all(header.as_bytes()).await?;
    stdin.write_all(body.as_bytes()).await?;
    stdin.flush().await?;

    Ok(())
}

async fn read_message(
    stdout: &mut BufReader<tokio::process::ChildStdout>,
) -> Result<Value, ChildLspError> {
    let content_length = read_content_length(stdout).await?;

    let mut body = vec![0u8; content_length];
    tokio::io::AsyncReadExt::read_exact(stdout, &mut body).await?;

    let value: Value = serde_json::from_slice(&body)?;
    Ok(value)
}

async fn read_content_length(
    stdout: &mut BufReader<tokio::process::ChildStdout>,
) -> Result<usize, ChildLspError> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = stdout.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Err(ChildLspError::ProcessExited);
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }

        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = Some(value.trim().parse::<usize>().map_err(|_| {
                ChildLspError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "child LSP sent a Content-Length header with a \
                         non-numeric value: '{}'",
                        value.trim()
                    ),
                ))
            })?);
        }
    }

    content_length.ok_or_else(|| {
        ChildLspError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "child LSP sent a message without a Content-Length header",
        ))
    })
}

/// Errors that can occur when communicating with a child LSP server.
#[derive(Debug, thiserror::Error)]
pub enum ChildLspError {
    /// No known LSP server for the requested language.
    #[error(
        "no LSP server known for language '{language_id}'. \
         Supported languages: rust, python, typescript, javascript, json, go, c, cpp, lua, zig, nix"
    )]
    UnknownLanguage { language_id: String },

    /// The child LSP process failed to start.
    #[error(
        "failed to spawn LSP server '{command}': {source}. \
         Make sure the language server is installed and available on PATH"
    )]
    SpawnFailed {
        command: String,
        source: std::io::Error,
    },

    /// An I/O error while reading from or writing to the child process.
    #[error("I/O error communicating with child LSP: {0}")]
    Io(#[from] std::io::Error),

    /// The child LSP returned a JSON-RPC error response.
    #[error("JSON-RPC error from child LSP: {message} (code {code})")]
    JsonRpc { code: i64, message: String },

    /// Failed to serialize or deserialize JSON-RPC messages.
    #[error("failed to parse JSON-RPC message: {0}")]
    Json(#[from] serde_json::Error),

    /// The child process exited before sending a complete response.
    #[error(
        "child LSP server exited unexpectedly. \
         This may indicate a crash in the language server"
    )]
    ProcessExited,
}

/// Map a language ID to the command (program + arguments) for spawning that
/// language's LSP server.
///
/// A project's `.hick-lsp.json` wins over the built-in default, so a
/// repository that has already chosen a server for a language gets the same
/// one inside its `hick:file` blocks — see [`crate::server_config`].
pub fn lsp_command(language_id: &str) -> Result<Vec<String>, ChildLspError> {
    // 1. What the project declared, if anything. An explicit choice always
    //    wins — `hick init` writes it from the repository's own editor config.
    if let Some(command) = crate::server_config::command_for(language_id) {
        return Ok(command);
    }
    // 2. What is actually installed, found where installers put things. This
    //    is the path that makes configuration unnecessary: a machine with
    //    pyright gets pyright without anybody saying so, and a project that
    //    pins its own server in node_modules gets that one instead.
    if let Some(found) =
        crate::discovery::discover(language_id, crate::server_config::project_root())
    {
        tracing::info!(
            language_id,
            origin = found.origin,
            command = found.command.join(" "),
            "discovered a language server"
        );
        return Ok(found.command);
    }
    // 3. The bare name, so a server on PATH under a conventional name still
    //    works even if discovery's directory list missed it.
    match language_id {
        "rust" => Ok(vec!["rust-analyzer".into()]),
        "python" => Ok(vec!["pyright-langserver".into(), "--stdio".into()]),
        "typescript" | "javascript" | "typescriptreact" | "javascriptreact" => {
            Ok(vec!["typescript-language-server".into(), "--stdio".into()])
        }
        "go" => Ok(vec!["gopls".into()]),
        "c" | "cpp" => Ok(vec!["clangd".into()]),
        "lua" => Ok(vec!["lua-language-server".into()]),
        "zig" => Ok(vec!["zls".into()]),
        "json" => Ok(vec!["vscode-json-language-server".into(), "--stdio".into()]),
        "nix" => Ok(vec!["nil".into()]),
        _ => Err(ChildLspError::UnknownLanguage {
            language_id: language_id.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_languages_return_commands() {
        for lang in &[
            "rust",
            "python",
            "typescript",
            "javascript",
            "typescriptreact",
            "javascriptreact",
            "json",
            "go",
            "c",
            "cpp",
            "lua",
            "zig",
            "nix",
        ] {
            let result = lsp_command(lang);
            assert!(
                result.is_ok(),
                "expected Ok for language '{lang}', got {result:?}"
            );
            assert!(
                !result.unwrap().is_empty(),
                "expected non-empty command for language '{lang}'"
            );
        }
    }

    #[test]
    fn unknown_language_returns_error() {
        let result = lsp_command("brainfuck");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("brainfuck"),
            "error should mention the unknown language: {msg}"
        );
        assert!(
            msg.contains("Supported languages"),
            "error should list supported languages: {msg}"
        );
    }
}
