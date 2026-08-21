//! A running backend: spawn it, ask it, stop it.
//!
//! Short, because the protocol is. There are no events, no reverse requests,
//! and no long-lived conversation — a request goes out, an answer comes back,
//! and the only interesting engineering is making sure a backend that hangs,
//! crashes, or writes nonsense does not take the app with it.
//!
//! Three rules do that:
//!
//! * **Every request has a deadline.** A formula that never returns is a
//!   plausible thing to write (`=while True: pass`, near enough), and a table
//!   that stops responding because one cell looped is a table nobody trusts.
//! * **A crash is an answer.** If the process dies, every formula in the
//!   batch gets an error saying so, rather than the caller waiting forever.
//! * **stderr is captured, not inherited.** A backend printing a warning must
//!   not scribble on the terminal the app is running in.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::protocol::{
    EvalRequest, EvalResponse, Formula, FormulaError, FormulaResult, Request, Response, frame,
};

/// How long one batch may take before it is abandoned.
///
/// Generous, because a formula that reads a file or fits a model is a
/// legitimate thing to write in a real language; finite, because an infinite
/// loop is a legitimate thing to write by accident.
pub const EVAL_TIMEOUT: Duration = Duration::from_secs(30);

/// A backend process, mid-conversation.
pub struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
    language: String,
}

impl Session {
    /// Start the backend for `language`, writing it into the project's cache
    /// first if it is not there. See `backend::ensure`.
    pub async fn start(root: &Path, language: &str) -> Result<Self> {
        let (interpreter, script) = crate::backend::ensure(root, language)?;
        let mut child = Command::new(&interpreter)
            .arg(&script)
            // The project root, so a formula that reads a relative path means
            // the same thing a cell in the same document would.
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Captured rather than inherited: a backend printing a warning
            // must not scribble on the terminal the app is running in.
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("could not start {interpreter} {}", script.display()))?;

        let stdin = child.stdin.take().context("the backend has no stdin")?;
        let stdout = BufReader::new(child.stdout.take().context("the backend has no stdout")?);
        let mut session = Self {
            child,
            stdin,
            stdout,
            next_id: 1,
            language: language.to_string(),
        };
        session
            .request("initialize", serde_json::json!({}))
            .await
            .with_context(|| format!("the {language} formula backend did not start"))?;
        Ok(session)
    }

    /// Evaluate a batch, in the order given.
    ///
    /// Order matters and is the host's: `graph.rs` worked it out, and a
    /// backend that reordered would break a table mixing two languages.
    pub async fn evaluate(&mut self, formulas: Vec<Formula>) -> Result<EvalResponse> {
        if formulas.is_empty() {
            return Ok(EvalResponse {
                results: Vec::new(),
            });
        }
        let ids: Vec<String> = formulas.iter().map(|f| f.id.clone()).collect();
        let params = serde_json::to_value(EvalRequest { formulas })?;
        match tokio::time::timeout(EVAL_TIMEOUT, self.request("evaluate", params)).await {
            Ok(Ok(value)) => Ok(serde_json::from_value(value)?),
            // A crash or a hang is an ANSWER: every formula in the batch gets
            // an error naming what happened, rather than the caller waiting
            // for a process that is never going to reply.
            Ok(Err(error)) => Ok(Self::all_failed(&ids, &format!("{error:#}"))),
            Err(_) => {
                let _ = self.child.start_kill();
                Ok(Self::all_failed(
                    &ids,
                    &format!(
                        "the {} formula backend did not answer within {}s and was stopped — \
                         a formula is probably looping",
                        self.language,
                        EVAL_TIMEOUT.as_secs()
                    ),
                ))
            }
        }
    }

    fn all_failed(ids: &[String], message: &str) -> EvalResponse {
        EvalResponse {
            results: ids
                .iter()
                .map(|id| FormulaResult {
                    id: id.clone(),
                    value: None,
                    error: Some(FormulaError {
                        message: message.to_string(),
                    }),
                })
                .collect(),
        }
    }

    /// Ask the backend to stop, then make sure it did.
    pub async fn shutdown(mut self) {
        let _ = self.request("shutdown", serde_json::json!({})).await;
        let _ = self.child.start_kill();
        let _ = self.child.wait().await;
    }

    async fn request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let id = self.next_id;
        self.next_id += 1;
        let body = serde_json::to_string(&Request {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        })?;
        self.stdin.write_all(frame(&body).as_bytes()).await?;
        self.stdin.flush().await?;

        loop {
            let response = self.read_response().await?;
            // Ignore anything that is not the answer to this request. There
            // should be nothing else, but a backend that logs a stray object
            // should be untidy rather than fatal.
            if response.id != Some(id) {
                continue;
            }
            if let Some(error) = response.error {
                bail!("{} (code {})", error.message, error.code);
            }
            return Ok(response.result.unwrap_or(serde_json::Value::Null));
        }
    }

    async fn read_response(&mut self) -> Result<Response> {
        let mut length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line).await?;
            if read == 0 {
                bail!("the {} formula backend closed its output", self.language);
            }
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() {
                break;
            }
            if let Some(rest) = trimmed.to_ascii_lowercase().strip_prefix("content-length:") {
                length = rest.trim().parse().ok();
            }
        }
        let length = length.context("a backend message arrived with no Content-Length")?;
        let mut body = vec![0u8; length];
        self.stdout.read_exact(&mut body).await?;
        Ok(serde_json::from_slice(&body)?)
    }
}
