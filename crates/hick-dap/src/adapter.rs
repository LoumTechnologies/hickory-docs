//! A running debug adapter: one process, one request/response channel, and a
//! stream of events nobody asked for.
//!
//! The shape mirrors `hick-lsp`'s child handle, with one structural
//! difference that matters. A language server answers questions; a debugger
//! *narrates*. `stopped`, `terminated`, `output` and `exited` arrive on their
//! own schedule, and the useful thing a caller does — "continue, then tell me
//! where we stop next" — is a request followed by an event, not a request
//! followed by a response. So events are broadcast to a subscriber rather
//! than returned, and every waiter reads the same stream.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin};
use tokio::sync::{Mutex, broadcast, oneshot};

use crate::protocol::{Event, Incoming, Response, ReverseRequest, content_length, frame};

/// How many events are buffered for a subscriber that is not reading yet.
///
/// A debugger's event traffic is bursty — a `continue` through a loop can
/// produce hundreds of `output` events before anyone asks — and a subscriber
/// that lags simply misses the oldest, which is the right failure for
/// terminal output and is why `stopped` is also tracked separately.
const EVENT_BUFFER: usize = 512;

pub struct Adapter {
    stdin: Arc<Mutex<ChildStdin>>,
    next_seq: AtomicI64,
    pending: Arc<Mutex<HashMap<i64, oneshot::Sender<Response>>>>,
    events: broadcast::Sender<Event>,
    process: Arc<Mutex<Child>>,
}

impl Adapter {
    /// Spawn `command` and start reading it.
    pub async fn spawn(command: &[String]) -> Result<Self> {
        let (program, args) = command
            .split_first()
            .context("a debug adapter command cannot be empty")?;

        let mut child = tokio::process::Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // Inherited: an adapter's stderr is where it explains why it will
            // not start, and swallowing that is how "the debugger did
            // nothing" becomes unanswerable.
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("spawning the debug adapter `{program}`"))?;

        let stdin = child.stdin.take().context("adapter stdin")?;
        let stdout = child.stdout.take().context("adapter stdout")?;
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        let pending: Arc<Mutex<HashMap<i64, oneshot::Sender<Response>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let reader_pending = pending.clone();
        let reader_events = events.clone();
        tokio::spawn(async move {
            let mut stream = BufReader::new(stdout);
            loop {
                let Some(message) = read_message(&mut stream).await else {
                    break;
                };
                match serde_json::from_value::<Incoming>(message.clone()) {
                    Ok(Incoming::Response(response)) => {
                        if let Some(tx) = reader_pending.lock().await.remove(&response.request_seq)
                        {
                            let _ = tx.send(response);
                        }
                    }
                    Ok(Incoming::Event(event)) => {
                        // Ignored send error: nobody subscribed yet is normal
                        // and not a reason to stop reading.
                        let _ = reader_events.send(event);
                    }
                    Ok(Incoming::Request(request)) => {
                        tracing::debug!(command = %request.command, "adapter asked the client for something");
                        let _ = reader_events.send(reverse_as_event(&request));
                    }
                    Err(error) => {
                        tracing::debug!(%error, "unparseable message from the adapter");
                    }
                }
            }
            // The adapter is gone. Anyone waiting on a response would
            // otherwise wait forever, so the map is dropped and their
            // receivers error.
            reader_pending.lock().await.clear();
        });

        Ok(Self {
            stdin: Arc::new(Mutex::new(stdin)),
            next_seq: AtomicI64::new(1),
            pending,
            events,
            process: Arc::new(Mutex::new(child)),
        })
    }

    /// Subscribe to the event stream. Do this BEFORE the request that causes
    /// the event, or the event races the subscription.
    pub fn events(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    /// Send a request and wait for its response.
    pub async fn request(&self, command: &str, arguments: Value) -> Result<Value> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(seq, tx);

        let message = json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        });
        self.send(&message).await?;

        let response = rx
            .await
            .with_context(|| format!("the adapter ended before answering `{command}`"))?;
        if !response.success {
            let detail = response.message.unwrap_or_else(|| "no reason given".into());
            bail!("the debug adapter refused `{command}`: {detail}");
        }
        Ok(response.body)
    }

    /// Send a request and do not wait. For `disconnect`, where waiting for a
    /// reply from a process that is exiting is a way to hang.
    pub async fn notify(&self, command: &str, arguments: Value) -> Result<()> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        self.send(&json!({
            "seq": seq,
            "type": "request",
            "command": command,
            "arguments": arguments,
        }))
        .await
    }

    async fn send(&self, message: &Value) -> Result<()> {
        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(&frame(message))
            .await
            .context("writing to the debug adapter")?;
        stdin
            .flush()
            .await
            .context("flushing to the debug adapter")?;
        Ok(())
    }

    /// End the adapter, politely then not.
    pub async fn shutdown(&self) {
        let _ = self
            .notify("disconnect", json!({ "terminateDebuggee": true }))
            .await;
        // A short grace period, then the hammer: a debug adapter that will
        // not exit holds the debuggee, and the debuggee holds a workdir.
        let mut process = self.process.lock().await;
        for _ in 0..20 {
            match process.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => tokio::time::sleep(std::time::Duration::from_millis(25)).await,
                Err(_) => break,
            }
        }
        let _ = process.kill().await;
    }
}

/// A reverse request, re-published as an event so subscribers see it.
///
/// The client does not implement `runInTerminal` — the executor already owns
/// how a cell is launched — so this exists to make the refusal visible rather
/// than silent.
fn reverse_as_event(request: &ReverseRequest) -> Event {
    Event {
        seq: request.seq,
        event: format!("hick/reverseRequest/{}", request.command),
        body: request.arguments.clone(),
    }
}

/// Read one framed message, or `None` when the stream ends.
async fn read_message<R>(stream: &mut BufReader<R>) -> Option<Value>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut header = String::new();
    loop {
        let mut line = String::new();
        if stream.read_line(&mut line).await.ok()? == 0 {
            return None;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        header.push_str(&line);
    }
    let length = content_length(&header)?;
    let mut body = vec![0u8; length];
    stream.read_exact(&mut body).await.ok()?;
    serde_json::from_slice(&body).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_command_with_no_program_is_refused_before_spawning() {
        assert!(Adapter::spawn(&[]).await.is_err());
    }

    #[tokio::test]
    async fn an_adapter_that_is_not_installed_names_itself() {
        // The message a person sees when `hick dap install` has not been run.
        let error = match Adapter::spawn(&["definitely-not-an-adapter".into()]).await {
            Ok(_) => panic!("a nonexistent adapter appeared to start"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("definitely-not-an-adapter"), "{error}");
    }

    #[tokio::test]
    async fn a_response_from_a_dead_adapter_errors_rather_than_hanging() {
        // `true` exits immediately without speaking DAP. Waiting forever for
        // a reply is the failure mode this prevents: a debugger that appears
        // to be thinking when it is actually gone.
        let adapter = Adapter::spawn(&["true".into()]).await.expect("spawns");
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            adapter.request("initialize", json!({})),
        )
        .await;
        assert!(result.is_ok(), "the request hung instead of failing");
        assert!(result.unwrap().is_err());
    }
}
