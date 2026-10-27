use anyhow::{Context, Result, bail};
use base64::Engine as _;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use hickory_fleet::{Fleet, Identity};
use hickory_peer::{Attached, Reach, Reachability};
use http_body_util::{BodyExt, Full, Limited};
use hyper_util::rt::TokioIo;
use serde::Serialize;
use serde_json::Value;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context as TaskContext, Poll},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

pub struct BiStream {
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
}
impl AsyncRead for BiStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        AsyncRead::poll_read(Pin::new(&mut self.recv), cx, buf)
    }
}
impl AsyncWrite for BiStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        AsyncWrite::poll_write(Pin::new(&mut self.send), cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.send), cx)
    }
    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<std::io::Result<()>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.send), cx)
    }
}

pub struct Peer {
    attached: Attached,
    pub machine: String,
}
impl Peer {
    pub async fn connect(
        identity: &Identity,
        fleet: &Fleet,
        machine: &str,
        addr: iroh::EndpointAddr,
        reach: Reach,
    ) -> Result<Self> {
        let enrolled = fleet
            .get(machine)
            .context("Pair this desktop before connecting")?;
        let key = base64::engine::general_purpose::STANDARD.decode(&enrolled.public_key)?;
        if key.as_slice() != addr.id.as_bytes() {
            bail!("The address belongs to a different desktop key; nothing was connected");
        }
        let attached = tokio::time::timeout(
            Duration::from_secs(20),
            Attached::connect(identity, addr, reach),
        )
        .await
        .context("Desktop connection timed out; check that its peer session is running")??;
        Ok(Self {
            attached,
            machine: machine.into(),
        })
    }
    async fn stream(&self) -> Result<BiStream> {
        let (send, recv) = self.attached.connection().open_bi().await?;
        Ok(BiStream { send, recv })
    }
    pub async fn how(&self) -> Reachability {
        self.attached.how().await
    }
    pub async fn close(&self) {
        self.attached.close().await;
    }
    pub async fn request(&self, method: &str, path: &str, body: Value) -> Result<Value> {
        if !allowed(method, path) {
            bail!("This desktop operation is not supported by the mobile client");
        }
        let work = async {
            let stream = self.stream().await?;
            let (mut sender, connection) =
                hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
            let task = tokio::spawn(async move {
                let _ = connection.await;
            });
            let _guard = AbortOnDrop(task.abort_handle());
            let bytes = if method == "GET" {
                Vec::new()
            } else {
                serde_json::to_vec(&body)?
            };
            let request = hyper::Request::builder()
                .method(method)
                .uri(path)
                .header("Host", "peer")
                .header("Connection", "close")
                .header("Content-Type", "application/json")
                .header("Content-Length", bytes.len())
                .body(Full::new(Bytes::from(bytes)))?;
            let response = sender.send_request(request).await?;
            let status = response.status();
            let bytes = Limited::new(response.into_body(), 8 * 1024 * 1024)
                .collect()
                .await
                .map_err(|e| anyhow::anyhow!("Desktop response could not be read: {e}"))?
                .to_bytes();
            if !status.is_success() {
                bail!(
                    "Desktop refused ({status}): {}",
                    String::from_utf8_lossy(&bytes)
                );
            }
            Ok::<_, anyhow::Error>(serde_json::from_slice(&bytes)?)
        };
        tokio::time::timeout(Duration::from_secs(30), work).await.context("Desktop request timed out; a submitted run may have started. Refresh its state before retrying")?
    }
    pub async fn room(
        self: &Arc<Self>,
        doc: &str,
        emit: impl Fn(RoomEvent) + Send + 'static,
    ) -> Result<Room> {
        if !identifier(doc) {
            bail!("Invalid document identifier");
        }
        let (socket, _) = tokio::time::timeout(Duration::from_secs(20), async {
            tokio_tungstenite::client_async_with_config(
                format!("ws://peer/api/ws?doc=doc%3A{doc}"),
                self.stream().await?,
                Some(tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
                    max_message_size: Some(8 * 1024 * 1024),
                    max_frame_size: Some(8 * 1024 * 1024),
                    ..Default::default()
                }),
            )
            .await
            .map_err(anyhow::Error::from)
        })
        .await
        .context("Desktop room timed out; local draft remains available")??;
        let (mut sink, mut incoming) = socket.split();
        let (outgoing, mut pending) = mpsc::channel::<Vec<u8>>(128);
        let task = tokio::spawn(async move {
            loop {
                let result = tokio::select! {
                    frame = pending.recv() => match frame {
                        Some(frame) => sink.send(Message::Binary(frame)).await.map(|_| None),
                        None => break,
                    },
                    frame = incoming.next() => match frame {
                        Some(Ok(Message::Binary(bytes))) => Ok(Some(bytes)),
                        Some(Ok(Message::Ping(bytes))) => sink.send(Message::Pong(bytes)).await.map(|_| None),
                        Some(Ok(Message::Close(_))) | None => break,
                        Some(Ok(_)) => Ok(None),
                        Some(Err(error)) => Err(error),
                    }
                };
                match result {
                    Ok(Some(bytes)) => emit(RoomEvent::Frame { bytes }),
                    Ok(None) => (),
                    Err(error) => {
                        emit(RoomEvent::Closed {
                            reason: error.to_string(),
                        });
                        return;
                    }
                }
            }
            emit(RoomEvent::Closed {
                reason: "Desktop room closed; edits remain on this phone".into(),
            });
        });
        Ok(Room {
            outgoing,
            _task: AbortOnDrop(task.abort_handle()),
        })
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RoomEvent {
    Frame { bytes: Vec<u8> },
    Closed { reason: String },
}
pub struct Room {
    outgoing: mpsc::Sender<Vec<u8>>,
    _task: AbortOnDrop,
}
impl Room {
    pub fn send(&self, bytes: Vec<u8>) -> Result<()> {
        if bytes.len() > 8 * 1024 * 1024 {
            bail!("Room update exceeds 8 MB");
        }
        self.outgoing
            .try_send(bytes)
            .context("Desktop room cannot accept this update; reconnect to sync the saved draft")
    }
}
struct AbortOnDrop(tokio::task::AbortHandle);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b))
}
fn allowed(method: &str, path: &str) -> bool {
    let parts: Vec<_> = path.split('/').collect();
    match (method, parts.as_slice()) {
        ("GET", ["", "api", "projects"]) => true,
        ("GET", ["", "api", "projects", id, "docs"]) => identifier(id),
        ("GET", ["", "api", "docs", id]) => identifier(id),
        ("GET", ["", "api", "docs", id, "render"]) => identifier(id),
        ("GET", ["", "api", "runs", id]) => identifier(id),
        ("POST", ["", "api", "docs", id, "run"]) => identifier(id),
        _ => false,
    }
}
