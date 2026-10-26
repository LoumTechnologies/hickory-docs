<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/acp/transport.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b81a21246202"></a>
## `crates/hickory-cli/src/serve/acp/transport.rs`

<a id="f-b81a21246202-Reply"></a>
### `Reply`

Bidirectional, line-delimited JSON-RPC. One reader; concurrent requests.

<storycode:copy id="f-b81a21246202-Reply">
//! Bidirectional, line-delimited JSON-RPC. One reader; concurrent requests.
use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, oneshot};

type Reply = Result<Value, String>;
</storycode:copy>
<a id="f-b81a21246202-Pending"></a>
### `Pending`

<storycode:copy id="f-b81a21246202-Pending">
type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Reply>>>>;

</storycode:copy>
<a id="f-b81a21246202-Rpc"></a>
### `Rpc`

<storycode:copy id="f-b81a21246202-Rpc">
pub struct Rpc {
    writer: tokio::sync::Mutex<tokio::process::ChildStdin>,
    pending: Pending,
    next: std::sync::atomic::AtomicU64,
    child: tokio::sync::Mutex<tokio::process::Child>,
    reader: tokio::task::JoinHandle<()>,
    stderr: tokio::task::JoinHandle<()>,
    diagnostic: Arc<Mutex<String>>,
    pub ordered: Arc<std::sync::atomic::AtomicBool>,
}

</storycode:copy>
<a id="f-b81a21246202-impl-Rpc"></a>
### `impl-Rpc`

<storycode:copy id="f-b81a21246202-impl-Rpc">
impl Rpc {
    pub fn spawn(
        mut command: tokio::process::Command,
    ) -> Result<(Arc<Self>, mpsc::UnboundedReceiver<Value>)> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let writer = child.stdin.take().expect("piped");
        let stdout = child.stdout.take().expect("piped");
        let stderr = child.stderr.take().expect("piped");
        let pending: Pending = Arc::default();
        let waiting = pending.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        let diagnostic = Arc::new(Mutex::new(String::new()));
        let ordered = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let ordering = ordered.clone();
        let errors = diagnostic.clone();
        let stderr = tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut buffer = errors.lock().unwrap();
                buffer.push_str(&line);
                buffer.push('\n');
                if buffer.len() > 8192 {
                    let start = buffer
                        .char_indices()
                        .find(|(i, _)| *i >= buffer.len() - 8192)
                        .map_or(0, |(i, _)| i);
                    buffer.drain(..start);
                }
            }
        });
        let reader = tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let message: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => {
                        let _ = tx.send(json!({"transportError": "The agent wrote invalid JSON to its protocol stream. Check its ACP command in Settings."}));
                        break;
                    }
                };
                if message.get("method").is_some() {
                    let _ = tx.send(message);
                } else if let Some(id) = message["id"].as_u64() {
                    if ordering.load(std::sync::atomic::Ordering::Relaxed) {
                        let _ = tx.send(message);
                        continue;
                    }
                    let reply = if let Some(error) = message.get("error") {
                        Err(error["message"]
                            .as_str()
                            .unwrap_or("agent request failed")
                            .to_string())
                    } else {
                        Ok(message["result"].clone())
                    };
                    if let Some(sender) = waiting.lock().unwrap().remove(&id) {
                        let _ = sender.send(reply);
                    }
                }
            }
            if !ordering.load(std::sync::atomic::Ordering::Relaxed) {
                for (_, sender) in waiting.lock().unwrap().drain() {
                    let _ = sender.send(Err(
                        "The agent process disconnected. Reconnect to continue.".into(),
                    ));
                }
            }
            let _ = tx.send(json!({"transportClosed": true}));
        });
        Ok((
            Arc::new(Self {
                writer: tokio::sync::Mutex::new(writer),
                pending,
                next: 1.into(),
                child: tokio::sync::Mutex::new(child),
                reader,
                stderr,
                diagnostic,
                ordered,
            }),
            rx,
        ))
    }

    pub async fn send(&self, message: Value) -> Result<()> {
        let mut writer = self.writer.lock().await;
        writer
            .write_all(format!("{}\n", message).as_bytes())
            .await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn request(
        &self,
        method: &str,
        params: Value,
        timeout: std::time::Duration,
    ) -> Result<Value> {
        let id = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id, tx);
        let result = async {
            self.send(json!({"jsonrpc":"2.0", "id":id, "method":method, "params":params}))
                .await?;
            match tokio::time::timeout(timeout, rx).await {
                Ok(Ok(Ok(value))) => Ok(value),
                Ok(Ok(Err(message))) => bail!("{message}"),
                Ok(Err(_)) => {
                    bail!("The agent disconnected while handling {method}. Reconnect to continue.")
                }
                Err(_) => bail!(
                    "The agent did not finish {method} in time. Stop and reconnect to try again."
                ),
            }
        }
        .await;
        self.pending.lock().unwrap().remove(&id);
        result.map_err(|e: anyhow::Error| {
            // Raw stderr can contain credentials; keep it local, never return it in HTTP.
            let _ = &self.diagnostic;
            anyhow!("{e:#}")
        })
    }

    pub async fn notify(&self, method: &str, params: Value) -> Result<()> {
        self.send(json!({"jsonrpc":"2.0", "method":method, "params":params}))
            .await
    }
    pub fn deliver(&self, message: &Value) {
        if let Some(id) = message["id"].as_u64()
            && let Some(sender) = self.pending.lock().unwrap().remove(&id)
        {
            let reply = if let Some(error) = message.get("error") {
                Err(error["message"]
                    .as_str()
                    .unwrap_or("agent request failed")
                    .to_string())
            } else {
                Ok(message["result"].clone())
            };
            let _ = sender.send(reply);
        }
    }
    pub fn disconnected(&self) {
        for (_, sender) in self.pending.lock().unwrap().drain() {
            let _ = sender.send(Err(
                "The agent process disconnected. Reconnect to continue.".into(),
            ));
        }
    }
    pub async fn kill(&self) {
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
}

</storycode:copy>
<a id="f-b81a21246202-impl-Drop-for-Rpc"></a>
### `impl-Drop for Rpc`

<storycode:copy id="f-b81a21246202-impl-Drop-for-Rpc">
impl Drop for Rpc {
    fn drop(&mut self) {
        self.reader.abort();
        self.stderr.abort();
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/acp/transport.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/acp/transport.rs"><storycode:paste select="#f-b81a21246202-Reply"/><storycode:paste select="#f-b81a21246202-Pending"/><storycode:paste select="#f-b81a21246202-Rpc"/><storycode:paste select="#f-b81a21246202-impl-Rpc"/><storycode:paste select="#f-b81a21246202-impl-Drop-for-Rpc"/></storycode:file>

</storycode:doc>
