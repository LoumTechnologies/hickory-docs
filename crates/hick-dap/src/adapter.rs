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
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::{Mutex, broadcast, oneshot};

use crate::protocol::{Event, Incoming, Response, ReverseRequest, content_length, frame};

/// How many events are buffered for a subscriber that is not reading yet.
///
/// A debugger's event traffic is bursty — a `continue` through a loop can
/// produce hundreds of `output` events before anyone asks — and a subscriber
/// that lags simply misses the oldest, which is the right failure for
/// terminal output and is why `stopped` is also tracked separately.
const EVENT_BUFFER: usize = 512;

/// How long to keep trying to reach an adapter that listens on a socket.
///
/// Generous because the first connection can be behind a compile: `dlv dap`
/// answers at once, but a cold `node` start on a slow machine is seconds.
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// How an adapter expects to be talked to.
///
/// Three of the four ecosystems this product most wants to debug do NOT
/// speak DAP over stdio, which is the single reason Go, JavaScript,
/// TypeScript and Ruby were reported as debuggable and could not work:
///
/// * `dlv dap` — "Starts a headless TCP server communicating via Debug
///   Adaptor Protocol", in delve's own help text.
/// * `js-debug` — prints "Debug server listening at ::1:8123" and never
///   reads stdin.
/// * `rdbg --open` — a UNIX domain socket, or TCP with `--port`.
///
/// Only debugpy, netcoredbg and codelldb speak stdio, and those are exactly
/// the three that had a live test and worked. Everything without one was
/// broken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Transport {
    /// The adapter reads requests on stdin and writes on stdout.
    #[default]
    Stdio,
    /// The adapter listens on a TCP port and is connected to. The command's
    /// arguments carry `{port}` where the port belongs.
    Tcp,
}

/// The `{port}` placeholder an adapter's arguments use.
pub const PORT_PLACEHOLDER: &str = "{port}";

/// A port nothing is listening on, released immediately so the adapter can
/// take it.
///
/// The gap between releasing and the adapter binding is a race, and it is the
/// same race every editor takes: DAP servers take a port number, not a
/// listening socket. Nothing here can close it, so it is named rather than
/// hidden.
fn free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .context("finding a free port for the debug adapter")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

pub struct Adapter {
    writer: Arc<Mutex<Box<dyn AsyncWrite + Send + Unpin>>>,
    next_seq: AtomicI64,
    pending: Arc<Mutex<HashMap<i64, oneshot::Sender<Response>>>>,
    events: broadcast::Sender<Event>,
    /// The adapter process, when hick started one. Always `Some` today;
    /// `Option` because an adapter reached over a socket need not be ours to
    /// kill, and that is the shape the Java plugin will need.
    process: Arc<Mutex<Option<Child>>>,
    /// The port this adapter was reached on, when it was reached over TCP.
    ///
    /// Kept so a **sibling** connection can be opened to the same server,
    /// which is what a multi-session adapter requires: js-debug does not run
    /// the program on the connection that launched it, it asks the client to
    /// open another one.
    port: Option<u16>,
}

impl Adapter {
    /// Start `command` over the transport it speaks.
    pub async fn start(command: &[String], transport: Transport) -> Result<Self> {
        match transport {
            Transport::Stdio => Adapter::spawn(command).await,
            Transport::Tcp => Adapter::connect_tcp(command).await,
        }
    }

    /// Spawn `command` and talk to it over its own stdin and stdout.
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
        Ok(Adapter::wire(Box::new(stdin), stdout, Some(child)))
    }

    /// Spawn a server that listens, then connect to it.
    ///
    /// `{port}` anywhere in the arguments is replaced with a free port. The
    /// process's stdout is left inherited along with its stderr: a listening
    /// adapter uses stdout for its own log lines ("Debug server listening
    /// at…"), and reading them as DAP frames would be wrong.
    pub async fn connect_tcp(command: &[String]) -> Result<Self> {
        let (program, args) = command
            .split_first()
            .context("a debug adapter command cannot be empty")?;
        let port = free_port()?;
        let args: Vec<String> = args
            .iter()
            .map(|arg| arg.replace(PORT_PLACEHOLDER, &port.to_string()))
            .collect();

        let child = tokio::process::Command::new(program)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("spawning the debug adapter `{program}`"))?;

        let deadline = std::time::Instant::now() + CONNECT_TIMEOUT;
        let stream = loop {
            match tokio::net::TcpStream::connect(("127.0.0.1", port)).await {
                Ok(stream) => break stream,
                Err(error) => {
                    if std::time::Instant::now() >= deadline {
                        return Err(anyhow::Error::new(error).context(format!(
                            "`{program}` never listened on port {port} within {}s. Its own \
                             output, above, is where it says why.",
                            CONNECT_TIMEOUT.as_secs()
                        )));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            }
        };
        // Nagle off: DAP is many small frames and a request that waits 40ms
        // for a coalescing buffer reads as a slow debugger.
        let _ = stream.set_nodelay(true);
        let (read, write) = stream.into_split();
        let mut adapter = Adapter::wire(Box::new(write), read, Some(child));
        adapter.port = Some(port);
        Ok(adapter)
    }

    /// A second connection to the same server, sharing its process.
    ///
    /// The process handle stays with the connection that spawned it: killing
    /// the server twice is not better than killing it once, and a sibling
    /// that outlived its parent would be talking to nothing.
    pub async fn sibling(&self) -> Result<Self> {
        let port = self
            .port
            .context("this adapter was not reached over a socket, so it has no sibling")?;
        let stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .with_context(|| format!("opening a second connection to the adapter on {port}"))?;
        let _ = stream.set_nodelay(true);
        let (read, write) = stream.into_split();
        let mut adapter = Adapter::wire(Box::new(write), read, None);
        adapter.port = Some(port);
        Ok(adapter)
    }

    /// The half both transports share: one writer, one reader, one loop.
    fn wire<R>(writer: Box<dyn AsyncWrite + Send + Unpin>, reader: R, child: Option<Child>) -> Self
    where
        R: AsyncRead + Send + Unpin + 'static,
    {
        let stdout = reader;
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

        Self {
            port: None,
            writer: Arc::new(Mutex::new(writer)),
            next_seq: AtomicI64::new(1),
            pending,
            events,
            process: Arc::new(Mutex::new(child)),
        }
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

    /// Answer a request the ADAPTER made of us.
    ///
    /// DAP is bidirectional, and an adapter that asks something and is never
    /// answered may simply stop. Until this existed every reverse request was
    /// turned into an event and dropped, which is fine for the advisory ones
    /// (`runInTerminal` is declined by capability) and not fine for
    /// `startDebugging`, which is a question js-debug waits on.
    pub async fn respond(&self, request_seq: i64, command: &str, body: Value) -> Result<()> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        self.send(&json!({
            "seq": seq,
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": command,
            "body": body,
        }))
        .await
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
        let mut stdin = self.writer.lock().await;
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
        let Some(process) = process.as_mut() else {
            return;
        };
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
