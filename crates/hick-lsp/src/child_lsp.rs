//! Child LSP process management.
//!
//! Manages spawning and communicating with child LSP servers (e.g.,
//! rust-analyzer, pyright) via JSON-RPC over stdio.

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

/// A running child LSP server process.
pub(crate) struct ChildLsp {
    /// The language ID this server handles (e.g., "rust", "python").
    #[allow(dead_code)]
    language_id: String,
    /// The spawned process.
    process: Child,
    /// Writer to the child's stdin.
    stdin: tokio::io::BufWriter<tokio::process::ChildStdin>,
    /// Reader from the child's stdout.
    stdout: BufReader<tokio::process::ChildStdout>,
    /// Next request ID for outgoing requests.
    next_id: i64,
    /// Whether the server has been initialized.
    initialized: bool,
}

impl ChildLsp {
    /// Spawn a new child LSP server for the given language.
    ///
    /// The language ID is mapped to a command using [`lsp_command`]. The child
    /// process is started with stdin/stdout piped for JSON-RPC communication
    /// and stderr inherited so operator can see child server logs.
    pub(crate) async fn spawn(language_id: &str) -> Result<Self, ChildLspError> {
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

        let child_stdin = child.stdin.take().expect(
            "stdin was configured as piped but take() returned None; this is a tokio bug",
        );
        let child_stdout = child.stdout.take().expect(
            "stdout was configured as piped but take() returned None; this is a tokio bug",
        );

        Ok(Self {
            language_id: language_id.to_string(),
            process: child,
            stdin: tokio::io::BufWriter::new(child_stdin),
            stdout: BufReader::new(child_stdout),
            next_id: 1,
            initialized: false,
        })
    }

    /// Send an LSP request and wait for the matching response.
    ///
    /// The method and params are wrapped in a JSON-RPC request envelope with an
    /// auto-incrementing ID. The response is read back, and JSON-RPC errors are
    /// surfaced as [`ChildLspError::JsonRpc`].
    pub(crate) async fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Value, ChildLspError> {
        let id = self.next_id;
        self.next_id += 1;

        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        self.send_message(&message).await?;

        // Read responses until we find one matching our request ID.
        // Notifications (messages without an id) are skipped.
        loop {
            let response = self.read_message().await?;

            // Skip notifications (no id field).
            let resp_id = match response.get("id") {
                Some(v) if !v.is_null() => v,
                _ => continue,
            };

            // Check if this is our response.
            if resp_id.as_i64() == Some(id) {
                if let Some(error) = response.get("error") {
                    let code = error
                        .get("code")
                        .and_then(|c| c.as_i64())
                        .unwrap_or(-1);
                    let message = error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    return Err(ChildLspError::JsonRpc { code, message });
                }

                return Ok(response
                    .get("result")
                    .cloned()
                    .unwrap_or(Value::Null));
            }
            // If the id doesn't match, skip (could be a response to a
            // cancelled or timed-out request).
        }
    }

    /// Send an LSP notification (no response expected).
    pub(crate) async fn notify(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<(), ChildLspError> {
        let message = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        self.send_message(&message).await
    }

    /// Initialize the child LSP server with the given workspace root.
    ///
    /// Sends the `initialize` request followed by the `initialized`
    /// notification per the LSP specification. Subsequent calls are a no-op.
    pub(crate) async fn initialize(
        &mut self,
        root_uri: &str,
    ) -> Result<Value, ChildLspError> {
        if self.initialized {
            return Ok(Value::Null);
        }

        let params = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "snippetSupport": false
                        }
                    },
                    "hover": {
                        "contentFormat": ["plaintext", "markdown"]
                    },
                    "definition": {}
                }
            }
        });

        let result = self.request("initialize", params).await?;
        self.notify("initialized", serde_json::json!({})).await?;
        self.initialized = true;

        Ok(result)
    }

    /// Shut down the child LSP server gracefully.
    ///
    /// Sends the `shutdown` request, then an `exit` notification, then kills
    /// the process if it hasn't already exited.
    pub(crate) async fn shutdown(&mut self) -> Result<(), ChildLspError> {
        // Best-effort: ignore errors during shutdown sequence since the child
        // may have already exited.
        let _ = self.request("shutdown", Value::Null).await;
        let _ = self.notify("exit", Value::Null).await;
        let _ = self.process.kill().await;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Send a JSON-RPC message with `Content-Length` framing.
    async fn send_message(&mut self, message: &Value) -> Result<(), ChildLspError> {
        let body = serde_json::to_string(message)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());

        self.stdin.write_all(header.as_bytes()).await?;
        self.stdin.write_all(body.as_bytes()).await?;
        self.stdin.flush().await?;

        Ok(())
    }

    /// Read one JSON-RPC message from the child's stdout.
    ///
    /// Parses the `Content-Length` header, then reads exactly that many bytes
    /// of JSON body.
    async fn read_message(&mut self) -> Result<Value, ChildLspError> {
        let content_length = self.read_content_length().await?;

        let mut body = vec![0u8; content_length];
        tokio::io::AsyncReadExt::read_exact(&mut self.stdout, &mut body).await?;

        let value: Value = serde_json::from_slice(&body)?;
        Ok(value)
    }

    /// Read headers until we find `Content-Length`, consuming the blank line
    /// that terminates the header block.
    async fn read_content_length(&mut self) -> Result<usize, ChildLspError> {
        let mut content_length: Option<usize> = None;
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = self.stdout.read_line(&mut line).await?;
            if bytes_read == 0 {
                return Err(ChildLspError::ProcessExited);
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                // End of headers.
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
            // Other headers (e.g. Content-Type) are ignored.
        }

        content_length.ok_or_else(|| {
            ChildLspError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "child LSP sent a message without a Content-Length header",
            ))
        })
    }
}

/// Errors that can occur when communicating with a child LSP server.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ChildLspError {
    /// No known LSP server for the requested language.
    #[error(
        "no LSP server known for language '{language_id}'. \
         Supported languages: rust, python, typescript, javascript, go, c, cpp, lua, zig, nix"
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
fn lsp_command(language_id: &str) -> Result<Vec<String>, ChildLspError> {
    match language_id {
        "rust" => Ok(vec!["rust-analyzer".into()]),
        "python" => Ok(vec![
            "pyright-langserver".into(),
            "--stdio".into(),
        ]),
        "typescript" | "javascript" | "typescriptreact" | "javascriptreact" => Ok(vec![
            "typescript-language-server".into(),
            "--stdio".into(),
        ]),
        "go" => Ok(vec!["gopls".into()]),
        "c" | "cpp" => Ok(vec!["clangd".into()]),
        "lua" => Ok(vec!["lua-language-server".into()]),
        "zig" => Ok(vec!["zls".into()]),
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
            "go",
            "c",
            "cpp",
            "lua",
            "zig",
            "nix",
        ] {
            let result = lsp_command(lang);
            assert!(result.is_ok(), "expected Ok for language '{lang}', got {result:?}");
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
