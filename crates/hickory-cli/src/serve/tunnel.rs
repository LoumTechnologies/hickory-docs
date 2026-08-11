//! The laptop's half of the tunnel.
//!
//! Dials the relay once, outbound, then serves every guest request that comes
//! back down it — against the *same* router the local session already answers
//! on 127.0.0.1. That sameness is the design: a guest arriving through the
//! relay hits exactly the code path a guest on the LAN hits, including the
//! capability check, so there is no second, weaker door.
//!
//! See `docs/specs/freeform/relay.md`.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use futures::{SinkExt as _, StreamExt as _};
use hickory_relay::{CloseReason, Frame, PROTOCOL_VERSION, RequestHead, ResponseHead, StreamId};
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::tungstenite::Message as TtMessage;
use tower::ServiceExt as _;

/// What the relay granted.
#[derive(Debug, Clone)]
pub struct TunnelHandle {
    pub url: String,
    pub slug: String,
    pub account: String,
    pub expires_in_secs: u64,
}

/// A guest WebSocket the local router upgraded, as seen from this side.
struct GuestSocket {
    to_local: mpsc::UnboundedSender<(Vec<u8>, bool)>,
}

/// The in-flight streams this end is tracking.
///
/// An HTTP request's body arrives in `Data` frames *after* its `Open`, so the
/// channel has to be registered before the request is served — otherwise the
/// body is dropped and the local router sees an empty one. (It did, and every
/// PUT and POST through the tunnel came back 400.)
#[derive(Default)]
struct Streams {
    sockets: HashMap<StreamId, GuestSocket>,
    /// `Some(bytes)` is more body; `None` is the end of it.
    bodies: HashMap<StreamId, mpsc::UnboundedSender<Option<Vec<u8>>>>,
}

/// Open a tunnel and serve it until the relay or the process goes away.
///
/// Returns once the tunnel is established, having spawned the pump; the handle
/// carries the address to print. An error here is a refusal the host needs to
/// read (not signed in, quota reached, version mismatch), so it is returned
/// rather than logged.
pub async fn open(
    relay_url: &str,
    token: &str,
    preferred_slug: Option<String>,
    router: Router,
    // This session's own listener, e.g. `ws://127.0.0.1:4321`. Guest
    // WebSockets are bridged to it rather than faked, so the local handler —
    // capability check included — is the only door.
    local_ws_base: &str,
) -> Result<TunnelHandle> {
    let endpoint = tunnel_endpoint(relay_url);
    let (socket, _) = tokio_tungstenite::connect_async(&endpoint)
        .await
        .with_context(|| format!("connecting to the relay at {endpoint}"))?;
    let (mut sink, mut incoming) = socket.split();

    sink.send(TtMessage::Binary(
        Frame::Hello {
            version: PROTOCOL_VERSION,
            token: token.to_string(),
            preferred_slug,
        }
        .encode()?,
    ))
    .await
    .context("sending the tunnel handshake")?;

    // The relay answers Ready or Refused; anything else is a protocol error.
    let handle = loop {
        let Some(message) = incoming.next().await else {
            bail!("the relay closed the connection before answering");
        };
        let bytes = match message.context("reading the relay's answer")? {
            TtMessage::Binary(b) => b.to_vec(),
            TtMessage::Text(t) => t.as_bytes().to_vec(),
            TtMessage::Close(_) => bail!("the relay closed the connection before answering"),
            _ => continue,
        };
        match Frame::decode(&bytes)? {
            Frame::Ready {
                url,
                slug,
                account,
                expires_in_secs,
            } => {
                break TunnelHandle {
                    url,
                    slug,
                    account,
                    expires_in_secs,
                };
            }
            // The relay's refusals are written for a person; passing the text
            // through unchanged is the whole point of the frame.
            Frame::Refused { message } => bail!("{message}"),
            other => bail!("unexpected answer from the relay: {other:?}"),
        }
    };

    let (to_relay, mut outbound) = mpsc::unbounded_channel::<Frame>();
    tokio::spawn(async move {
        while let Some(frame) = outbound.recv().await {
            let Ok(bytes) = frame.encode() else { continue };
            if sink.send(TtMessage::Binary(bytes)).await.is_err() {
                break;
            }
        }
    });

    let streams: Arc<Mutex<Streams>> = Arc::new(Mutex::new(Streams::default()));
    let local_ws_base = local_ws_base.to_string();
    tokio::spawn(async move {
        while let Some(Ok(message)) = incoming.next().await {
            let bytes = match message {
                TtMessage::Binary(b) => b.to_vec(),
                TtMessage::Text(t) => t.as_bytes().to_vec(),
                TtMessage::Close(_) => break,
                _ => continue,
            };
            let Ok(frame) = Frame::decode(&bytes) else {
                continue;
            };
            handle_frame(frame, &router, &to_relay, &streams, &local_ws_base).await;
        }
        log::info!("the relay connection has ended; this session is local-only again");
    });

    Ok(handle)
}

/// `https://relay.example` → `wss://relay.example/_relay/tunnel`.
fn tunnel_endpoint(relay_url: &str) -> String {
    let base = relay_url.trim_end_matches('/');
    let ws = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        format!("wss://{base}")
    };
    format!("{ws}/_relay/tunnel")
}

async fn handle_frame(
    frame: Frame,
    router: &Router,
    to_relay: &mpsc::UnboundedSender<Frame>,
    streams: &Arc<Mutex<Streams>>,
    local_ws_base: &str,
) {
    match frame {
        Frame::Open { stream, head } => {
            // Register the receiving end BEFORE spawning anything. Whatever
            // follows an `Open` — a request body, or a guest's first
            // WebSocket frame — arrives on this same reader loop, and a task
            // that has not been scheduled yet has nowhere to put it. Both
            // races were real: the body one turned every PUT into a 400, and
            // the socket one made the document room sit silent because the
            // guest's "send me the document" was dropped.
            let (body_rx, socket_rx) = if head.websocket {
                let (tx, rx) = mpsc::unbounded_channel();
                streams
                    .lock()
                    .await
                    .sockets
                    .insert(stream, GuestSocket { to_local: tx });
                (None, Some(rx))
            } else {
                let (tx, rx) = mpsc::unbounded_channel();
                streams.lock().await.bodies.insert(stream, tx);
                (Some(rx), None)
            };

            let router = router.clone();
            let to_relay = to_relay.clone();
            let streams = streams.clone();
            let local_ws_base = local_ws_base.to_string();
            // One task per guest request: a slow document render must not
            // block the socket carrying everyone else's edits.
            tokio::spawn(async move {
                let result = serve_stream(
                    stream,
                    *head,
                    router,
                    to_relay.clone(),
                    streams.clone(),
                    StreamInbox {
                        body: body_rx,
                        socket: socket_rx,
                    },
                    &local_ws_base,
                )
                .await;
                if let Err(e) = result {
                    log::debug!("stream {stream} failed: {e:#}");
                    let _ = to_relay.send(Frame::Close {
                        stream,
                        reason: CloseReason::Failed,
                    });
                }
                streams.lock().await.bodies.remove(&stream);
            });
        }
        Frame::Data {
            stream,
            bytes,
            text,
        } => {
            let streams = streams.lock().await;
            if let Some(socket) = streams.sockets.get(&stream) {
                let _ = socket.to_local.send((bytes, text));
            } else if let Some(body) = streams.bodies.get(&stream) {
                let _ = body.send(Some(bytes));
            }
        }
        Frame::Close { stream, .. } => {
            let mut streams = streams.lock().await;
            streams.sockets.remove(&stream);
            // Ends the request body rather than dropping the stream: the reply
            // still has to be produced and sent.
            if let Some(body) = streams.bodies.get(&stream) {
                let _ = body.send(None);
            }
        }
        Frame::Ping => {
            let _ = to_relay.send(Frame::Pong);
        }
        _ => {}
    }
}

/// Serve one guest request from the local router and stream the reply back.
/// The receiving ends registered for one stream before it was spawned.
struct StreamInbox {
    /// Request body chunks; `None` ends the body.
    body: Option<mpsc::UnboundedReceiver<Option<Vec<u8>>>>,
    /// Guest WebSocket frames on their way to the local session.
    socket: Option<mpsc::UnboundedReceiver<(Vec<u8>, bool)>>,
}

async fn serve_stream(
    stream: StreamId,
    head: RequestHead,
    router: Router,
    to_relay: mpsc::UnboundedSender<Frame>,
    streams: Arc<Mutex<Streams>>,
    inbox: StreamInbox,
    local_ws_base: &str,
) -> Result<()> {
    let StreamInbox {
        body: body_rx,
        socket: socket_rx,
    } = inbox;
    if head.websocket {
        let from_relay =
            socket_rx.context("a websocket stream was opened without its receiving end")?;
        return serve_websocket(stream, head, to_relay, streams, from_relay, local_ws_base).await;
    }

    let mut builder = Request::builder()
        .method(head.method.as_str())
        .uri(&head.uri);
    for (name, value) in &head.headers {
        builder = builder.header(name, value);
    }

    // Collect the request body before serving. Whole rather than streamed, for
    // the same reason the relay buffers it: these are documents and JSON, not
    // uploads, and a streaming body would double the protocol to serve a case
    // this product does not have.
    let mut body = Vec::new();
    if let Some(mut rx) = body_rx {
        while let Some(chunk) = rx.recv().await {
            match chunk {
                Some(bytes) => body.extend_from_slice(&bytes),
                None => break,
            }
        }
    }

    let request = builder.body(Body::from(body))?;
    let response = router
        .oneshot(request)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let (parts, body) = response.into_parts();
    to_relay.send(Frame::Response {
        stream,
        head: Box::new(ResponseHead {
            status: parts.status.as_u16(),
            headers: parts
                .headers
                .iter()
                .filter_map(|(k, v)| {
                    v.to_str()
                        .ok()
                        .map(|v| (k.as_str().to_string(), v.to_string()))
                })
                .collect(),
        }),
    })?;

    let bytes = axum::body::to_bytes(body, usize::MAX).await?;
    if !bytes.is_empty() {
        to_relay.send(Frame::Data {
            stream,
            bytes: bytes.to_vec(),
            text: false,
        })?;
    }
    to_relay.send(Frame::Close {
        stream,
        reason: CloseReason::Done,
    })?;
    Ok(())
}

/// A guest WebSocket, bridged into the local router.
///
/// The local router expects a real upgrade, so this dials the session's own
/// listener over loopback rather than trying to fake one through `oneshot`.
/// That keeps the local WS handler — capability check included — exactly as it
/// is for a guest on the LAN.
async fn serve_websocket(
    stream: StreamId,
    head: RequestHead,
    to_relay: mpsc::UnboundedSender<Frame>,
    streams: Arc<Mutex<Streams>>,
    mut from_relay: mpsc::UnboundedReceiver<(Vec<u8>, bool)>,
    local_ws_base: &str,
) -> Result<()> {
    let local = format!("{local_ws_base}{}", head.uri);
    let socket = match tokio_tungstenite::connect_async(&local).await {
        Ok((socket, response)) => {
            to_relay.send(Frame::Response {
                stream,
                head: Box::new(ResponseHead {
                    status: response.status().as_u16(),
                    headers: Vec::new(),
                }),
            })?;
            socket
        }
        Err(e) => {
            streams.lock().await.sockets.remove(&stream);
            let _ = to_relay.send(Frame::Close {
                stream,
                reason: CloseReason::Failed,
            });
            return Err(anyhow::anyhow!(e))
                .with_context(|| format!("dialling the local session at {local}"));
        }
    };

    let (mut local_sink, mut local_rx) = socket.split();

    // relay → local
    let writer = tokio::spawn(async move {
        while let Some((bytes, text)) = from_relay.recv().await {
            let message = if text {
                match String::from_utf8(bytes) {
                    Ok(s) => TtMessage::Text(s),
                    Err(_) => continue,
                }
            } else {
                TtMessage::Binary(bytes)
            };
            if local_sink.send(message).await.is_err() {
                break;
            }
        }
    });

    // local → relay
    while let Some(Ok(message)) = local_rx.next().await {
        let (bytes, text) = match message {
            TtMessage::Binary(b) => (b.to_vec(), false),
            TtMessage::Text(t) => (t.as_bytes().to_vec(), true),
            TtMessage::Close(_) => break,
            _ => continue,
        };
        if to_relay
            .send(Frame::Data {
                stream,
                bytes,
                text,
            })
            .is_err()
        {
            break;
        }
    }

    writer.abort();
    streams.lock().await.sockets.remove(&stream);
    let _ = to_relay.send(Frame::Close {
        stream,
        reason: CloseReason::PeerGone,
    });
    Ok(())
}

/// The status a guest sees when the tunnel cannot serve them.
pub const UNAVAILABLE: StatusCode = StatusCode::BAD_GATEWAY;
