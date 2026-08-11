//! One tunnel: an agent's control socket, and every guest stream riding it.
//!
//! The shape of the problem: a guest opens an ordinary HTTP connection to the
//! relay, but the machine that can answer it is behind NAT and can only be
//! reached over a socket *it* opened earlier. So every guest connection becomes
//! a stream on that socket, and the relay's job is bookkeeping — which bytes
//! belong to which guest, and what to do when either side vanishes.
//!
//! Two guest shapes, handled differently because they end differently:
//!
//! - **HTTP** — head, body, reply, done. The reply's head arrives on a
//!   `oneshot`, its body on a channel that becomes the response stream.
//! - **WebSocket** — upgraded, then bytes both ways for as long as someone is
//!   editing. This is the document room, so it is the one that has to be right.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Result, bail};
use axum::body::Body;
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{FromRequestParts as _, Request};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use futures::{SinkExt as _, StreamExt as _};
use hickory_relay::{
    CloseReason, Frame, PROTOCOL_VERSION, Quota, RequestHead, ResponseHead, StreamId, forwardable,
};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

use crate::RelayState;

/// The largest guest request body the relay will buffer.
///
/// Bodies are forwarded whole rather than streamed: a hick document is prose
/// and cells, an API call is JSON, and streaming request bodies would double
/// the protocol's complexity to serve a case this product does not have. A
/// request past this limit is refused with a status that says so, instead of
/// being silently truncated.
const MAX_REQUEST_BODY: usize = 8 * 1024 * 1024;

/// Response body chunks buffered per stream before the writer blocks.
const RESPONSE_QUEUE: usize = 64;

// ---------------------------------------------------------------------------
// Streams
// ---------------------------------------------------------------------------

/// The relay's half of one guest connection.
enum StreamHandle {
    /// An HTTP exchange waiting for its reply.
    AwaitingResponse {
        head: oneshot::Sender<ResponseHead>,
        body: mpsc::Sender<Vec<u8>>,
    },
    /// An HTTP exchange whose head has arrived; only body bytes remain.
    Responding { body: mpsc::Sender<Vec<u8>> },
    /// An upgraded WebSocket: `(bytes, is_text)` on their way to the guest.
    Upgraded {
        to_guest: mpsc::UnboundedSender<(Vec<u8>, bool)>,
    },
}

/// One live tunnel.
pub struct Tunnel {
    pub slug: String,
    pub account: String,
    to_agent: mpsc::UnboundedSender<Frame>,
    streams: Mutex<HashMap<StreamId, StreamHandle>>,
    next_stream: AtomicU64,
    quota: Quota,
}

impl Tunnel {
    fn send(&self, frame: Frame) -> Result<()> {
        self.to_agent
            .send(frame)
            .map_err(|_| anyhow::anyhow!("the session's connection has closed"))
    }

    /// Forward one guest request down the tunnel and return its reply.
    pub async fn forward(self: &Arc<Self>, req: Request<Body>) -> Response {
        match self.try_forward(req).await {
            Ok(resp) => resp,
            Err(e) => {
                log::debug!("forwarding failed on {}: {e:#}", self.slug);
                // BAD_GATEWAY is the honest status: the relay is fine, the
                // machine behind it did not answer.
                (
                    StatusCode::BAD_GATEWAY,
                    format!(
                        "The session at {} did not answer. It may have just ended — \
                         sessions run only while `hickory serve` is running.",
                        self.slug
                    ),
                )
                    .into_response()
            }
        }
    }

    async fn try_forward(self: &Arc<Self>, req: Request<Body>) -> Result<Response> {
        {
            let streams = self.streams.lock().await;
            if streams.len() >= self.quota.max_streams_per_tunnel {
                bail!(
                    "tunnel {} is at its stream ceiling ({})",
                    self.slug,
                    self.quota.max_streams_per_tunnel
                );
            }
        }

        let (mut parts, body) = req.into_parts();
        let is_ws = parts
            .headers
            .get(axum::http::header::UPGRADE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.eq_ignore_ascii_case("websocket"));

        let head = RequestHead {
            method: parts.method.to_string(),
            uri: parts
                .uri
                .path_and_query()
                .map(|pq| pq.to_string())
                .unwrap_or_else(|| parts.uri.path().to_string()),
            headers: forwardable(
                &parts
                    .headers
                    .iter()
                    .filter_map(|(k, v)| {
                        v.to_str()
                            .ok()
                            .map(|v| (k.as_str().to_string(), v.to_string()))
                    })
                    .collect::<Vec<_>>(),
            ),
            websocket: is_ws,
        };

        let stream = self.next_stream.fetch_add(1, Ordering::Relaxed);

        if is_ws {
            // The guest's socket is upgraded here, and its bytes are pumped
            // into the tunnel once the agent confirms with a 101.
            let upgrade = WebSocketUpgrade::from_request_parts(&mut parts, &())
                .await
                .map_err(|e| anyhow::anyhow!("not a valid WebSocket upgrade: {e:?}"))?;

            let (to_guest, from_tunnel) = mpsc::unbounded_channel::<(Vec<u8>, bool)>();
            self.streams
                .lock()
                .await
                .insert(stream, StreamHandle::Upgraded { to_guest });
            self.send(Frame::Open {
                stream,
                head: Box::new(head),
            })?;

            let tunnel = self.clone();
            return Ok(upgrade.on_upgrade(move |socket| async move {
                pump_guest_socket(tunnel, stream, socket, from_tunnel).await;
            }));
        }

        // Ordinary HTTP: read the body, send it, wait for the reply.
        let bytes = axum::body::to_bytes(body, MAX_REQUEST_BODY)
            .await
            .map_err(|_| anyhow::anyhow!("request body exceeds {MAX_REQUEST_BODY} bytes"))?;

        let (head_tx, head_rx) = oneshot::channel();
        let (body_tx, body_rx) = mpsc::channel::<Vec<u8>>(RESPONSE_QUEUE);
        self.streams.lock().await.insert(
            stream,
            StreamHandle::AwaitingResponse {
                head: head_tx,
                body: body_tx,
            },
        );

        self.send(Frame::Open {
            stream,
            head: Box::new(head),
        })?;
        if !bytes.is_empty() {
            self.send(Frame::Data {
                stream,
                bytes: bytes.to_vec(),
                text: false,
            })?;
        }
        // An empty `Data` would be ambiguous, so the end of the request body is
        // signalled by `Close` — the agent replies, and its own `Close` ends
        // the stream in the other direction.
        self.send(Frame::Close {
            stream,
            reason: CloseReason::Done,
        })?;

        let head = head_rx
            .await
            .map_err(|_| anyhow::anyhow!("the session closed before replying"))?;

        let mut response = Response::builder().status(head.status);
        for (name, value) in &head.headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::try_from(name.as_str()),
                HeaderValue::try_from(value.as_str()),
            ) {
                response = response.header(name, value);
            }
        }
        response
            .body(Body::from_stream(
                ReceiverStream::new(body_rx).map(Ok::<_, std::io::Error>),
            ))
            .map_err(|e| anyhow::anyhow!("building the reply: {e}"))
    }

    /// Route one frame arriving from the agent.
    async fn dispatch(&self, frame: Frame) {
        match frame {
            Frame::Response { stream, head } => {
                let mut streams = self.streams.lock().await;
                match streams.remove(&stream) {
                    Some(StreamHandle::AwaitingResponse { head: tx, body }) => {
                        let _ = tx.send(*head);
                        streams.insert(stream, StreamHandle::Responding { body });
                    }
                    Some(other) => {
                        // A second head, or a head for an upgraded socket:
                        // keep the stream rather than dropping the guest.
                        streams.insert(stream, other);
                    }
                    None => {}
                }
            }
            Frame::Data {
                stream,
                bytes,
                text,
            } => {
                let streams = self.streams.lock().await;
                match streams.get(&stream) {
                    Some(StreamHandle::Upgraded { to_guest }) => {
                        let _ = to_guest.send((bytes, text));
                    }
                    // `try_send` rather than `send`: holding the streams lock
                    // across an await would stall every other guest on this
                    // tunnel behind one slow reader.
                    Some(
                        StreamHandle::Responding { body }
                        | StreamHandle::AwaitingResponse { body, .. },
                    ) => {
                        if let Err(e) = body.try_send(bytes) {
                            log::debug!("dropping body bytes for stream {stream}: {e}");
                        }
                    }
                    None => {}
                }
            }
            Frame::Close { stream, .. } => {
                self.streams.lock().await.remove(&stream);
            }
            Frame::Ping => {
                let _ = self.send(Frame::Pong);
            }
            _ => {}
        }
    }

    /// Tear down every stream — the agent is gone.
    async fn shutdown(&self) {
        self.streams.lock().await.clear();
    }
}

/// Pump one guest WebSocket in both directions.
async fn pump_guest_socket(
    tunnel: Arc<Tunnel>,
    stream: StreamId,
    socket: WebSocket,
    mut from_tunnel: mpsc::UnboundedReceiver<(Vec<u8>, bool)>,
) {
    let (mut sink, mut guest) = socket.split();

    let writer = tokio::spawn(async move {
        while let Some((bytes, text)) = from_tunnel.recv().await {
            let message = if text {
                match String::from_utf8(bytes) {
                    Ok(s) => WsMessage::Text(s.into()),
                    Err(_) => continue,
                }
            } else {
                WsMessage::Binary(bytes.into())
            };
            if sink.send(message).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(message)) = guest.next().await {
        let (bytes, text) = match message {
            WsMessage::Binary(b) => (b.to_vec(), false),
            WsMessage::Text(t) => (t.as_bytes().to_vec(), true),
            WsMessage::Close(_) => break,
            // Ping/Pong are handled by axum's socket itself.
            _ => continue,
        };
        if tunnel
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
    tunnel.streams.lock().await.remove(&stream);
    let _ = tunnel.send(Frame::Close {
        stream,
        reason: CloseReason::PeerGone,
    });
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Every live tunnel, by slug. Nothing here is persisted: a relay restart drops
/// every tunnel and every client reconnects, which is the whole recovery plan.
#[derive(Default)]
pub struct TunnelRegistry {
    by_slug: Mutex<HashMap<String, Arc<Tunnel>>>,
}

impl TunnelRegistry {
    pub async fn get(&self, slug: &str) -> Option<Arc<Tunnel>> {
        self.by_slug.lock().await.get(slug).cloned()
    }

    pub async fn count(&self) -> usize {
        self.by_slug.lock().await.len()
    }

    async fn insert(&self, tunnel: Arc<Tunnel>) {
        self.by_slug
            .lock()
            .await
            .insert(tunnel.slug.clone(), tunnel);
    }

    async fn remove(&self, slug: &str) {
        self.by_slug.lock().await.remove(slug);
    }

    /// Drop a tunnel as if its agent had vanished — what a closing laptop lid
    /// does. Exposed so a test can prove a guest is told plainly, rather than
    /// left waiting.
    pub async fn remove_for_test(&self, slug: &str) {
        self.remove(slug).await;
    }
}

// ---------------------------------------------------------------------------
// The agent side
// ---------------------------------------------------------------------------

/// Serve one agent's control socket for the life of its tunnel.
pub async fn serve_agent(state: RelayState, socket: WebSocket) -> Result<()> {
    let (mut sink, mut incoming) = socket.split();

    // The opening frame must be `Hello`, and it must arrive promptly: a socket
    // that connects and says nothing is either broken or probing.
    let hello = tokio::time::timeout(std::time::Duration::from_secs(10), incoming.next())
        .await
        .map_err(|_| anyhow::anyhow!("no Hello within 10s"))?
        .ok_or_else(|| anyhow::anyhow!("connection closed before Hello"))?;
    let hello = match hello? {
        WsMessage::Binary(b) => Frame::decode(&b)?,
        WsMessage::Text(t) => Frame::decode(t.as_bytes())?,
        _ => bail!("expected a Hello frame"),
    };

    let (version, token, preferred) = match hello {
        Frame::Hello {
            version,
            token,
            preferred_slug,
        } => (version, token, preferred_slug),
        other => bail!("expected Hello, got {other:?}"),
    };

    /// Tell the agent why it cannot have a tunnel, then let the socket close.
    /// Every refusal reaches a human staring at a terminal, so the message is
    /// the whole point of the frame.
    async fn refuse(sink: &mut futures::stream::SplitSink<WebSocket, WsMessage>, message: String) {
        if let Ok(bytes) = (Frame::Refused { message }).encode() {
            let _ = sink.send(WsMessage::Binary(bytes.into())).await;
        }
    }

    if version != PROTOCOL_VERSION {
        // A mismatched protocol is refused rather than guessed at: a frame
        // misunderstood in the middle of a document edit is worse than a
        // client that is told to upgrade.
        refuse(
            &mut sink,
            format!(
                "this relay speaks tunnel protocol {PROTOCOL_VERSION}, your hickory speaks \
                 {version} — update with `hickory update` (or reinstall)"
            ),
        )
        .await;
        return Ok(());
    }

    let account = match state.identifier.identify(&token).await {
        Ok(account) => account,
        Err(e) => {
            refuse(&mut sink, format!("{e:#}")).await;
            return Ok(());
        }
    };

    if let Err(message) = state.census.lock().await.admit(&account, &state.quota) {
        refuse(&mut sink, message).await;
        return Ok(());
    }

    let slug = choose_slug(&state, preferred.as_deref(), &account).await;
    let (to_agent, mut outbound) = mpsc::unbounded_channel::<Frame>();

    let tunnel = Arc::new(Tunnel {
        slug: slug.clone(),
        account: account.clone(),
        to_agent,
        streams: Mutex::new(HashMap::new()),
        next_stream: AtomicU64::new(1),
        quota: state.quota,
    });

    // The writer owns the sink from here; everything else queues frames.
    let writer = tokio::spawn(async move {
        while let Some(frame) = outbound.recv().await {
            let Ok(bytes) = frame.encode() else { continue };
            if sink.send(WsMessage::Binary(bytes.into())).await.is_err() {
                break;
            }
        }
    });

    tunnel.send(Frame::Ready {
        url: state.url_for(&slug),
        slug: slug.clone(),
        account: account.clone(),
        expires_in_secs: state.quota.tunnel_ttl_secs,
    })?;
    state.tunnels.insert(tunnel.clone()).await;
    log::info!("tunnel {slug} open for {account}");

    // A tunnel is a working session, not a deployment: it ends on its own.
    let ttl = tokio::time::sleep(std::time::Duration::from_secs(state.quota.tunnel_ttl_secs));
    tokio::pin!(ttl);

    loop {
        tokio::select! {
            _ = &mut ttl => {
                log::info!("tunnel {slug} reached its time limit");
                break;
            }
            message = incoming.next() => {
                let Some(message) = message else { break };
                let bytes = match message {
                    Ok(WsMessage::Binary(b)) => b.to_vec(),
                    Ok(WsMessage::Text(t)) => t.as_bytes().to_vec(),
                    Ok(WsMessage::Close(_)) | Err(_) => break,
                    Ok(_) => continue,
                };
                match Frame::decode(&bytes) {
                    Ok(frame) => tunnel.dispatch(frame).await,
                    Err(e) => log::debug!("undecodable frame on {slug}: {e}"),
                }
            }
        }
    }

    state.tunnels.remove(&slug).await;
    state.census.lock().await.release(&account);
    tunnel.shutdown().await;
    writer.abort();
    log::info!("tunnel {slug} closed");
    Ok(())
}

/// Pick the slug a session will be reachable at.
///
/// A requested slug is honoured only when it is well-formed and free —
/// first come, first served, with no reservation system. Otherwise a readable
/// random one is minted, because a person has to be able to say it aloud and
/// type it from a screenshot.
async fn choose_slug(state: &RelayState, preferred: Option<&str>, account: &str) -> String {
    if let Some(slug) = preferred
        && hickory_relay::slug_is_valid(slug)
        && state.tunnels.get(slug).await.is_none()
    {
        return slug.to_string();
    }
    for _ in 0..32 {
        let candidate = random_slug(account);
        if state.tunnels.get(&candidate).await.is_none() {
            return candidate;
        }
    }
    // 32 collisions against a 3-word space means something is very wrong;
    // falling back to a longer name is better than looping forever.
    format!("{}-{}", random_slug(account), random_slug(account))
}

const ADJECTIVES: &[&str] = &[
    "quiet", "amber", "brisk", "clever", "dusty", "eager", "fresh", "gentle", "hidden", "ivory",
    "jolly", "keen", "lively", "mellow", "noble", "olive", "plain", "quick", "rustic", "silver",
];

const NOUNS: &[&str] = &[
    "cedar", "brook", "ember", "fern", "grove", "harbor", "island", "juniper", "kettle", "lantern",
    "meadow", "nutmeg", "orchard", "pebble", "quarry", "ridge", "spruce", "thicket", "vale",
    "willow",
];

/// `quiet-cedar-1042`. Readable on purpose: this address gets typed by hand.
fn random_slug(seed_material: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    for byte in seed_material.as_bytes().iter().chain(&now.to_le_bytes()) {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    let adjective = ADJECTIVES[(hash % ADJECTIVES.len() as u64) as usize];
    let noun = NOUNS[((hash >> 16) % NOUNS.len() as u64) as usize];
    let number = (hash >> 32) % 10_000;
    format!("{adjective}-{noun}-{number}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_minted_slug_is_a_valid_address_and_readable() {
        for _ in 0..50 {
            let slug = random_slug("nate");
            assert!(hickory_relay::slug_is_valid(&slug), "{slug}");
            assert_eq!(slug.split('-').count(), 3, "{slug}");
        }
    }
}
