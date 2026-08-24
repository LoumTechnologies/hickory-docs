//! Binding an endpoint, admitting a peer, and carrying the existing API.
//!
//! A peer opens one QUIC bi-stream per request, writes an ordinary HTTP
//! request head, and this checks it against that machine's grants before
//! proxying it to the session's own loopback server. Both halves of the
//! product's surface ride that: the REST API is requests, and a room is the
//! websocket upgrade, which is a request too.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use base64::Engine as _;
use hickory_fleet::{Fleet, Identity, Machine};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

/// The protocol this channel speaks. Versioned, because two machines of one
/// engineer are updated on different days and a mismatch should be a refusal
/// naming both versions rather than a wire-format surprise.
pub const ALPN: &[u8] = b"hickory/fleet/0";

/// How this machine tries to be reachable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Reach {
    /// Direct paths, plus number0's relays to coordinate hole-punching and to
    /// carry traffic when no direct path exists.
    ///
    /// **Chosen deliberately on 2026-08-24**, over the direct-only posture:
    /// it is what makes a café work with no setup. It is not a server we run,
    /// so nothing about `local-only.md` changes — but two third-party
    /// surfaces come with it, and both are named rather than inherited:
    ///
    /// 1. **Traffic may transit number0's relays** when no direct path can be
    ///    found. It is encrypted end to end and they cannot read it; they can
    ///    see that two keys are talking, and how much.
    /// 2. **This machine's addresses are published to number0's DNS**
    ///    (`iroh.link`, via pkarr), which is what lets a peer find it by
    ///    public key from anywhere. Discovery is the half that makes "no
    ///    setup" true, and it is a *publication*, not merely a fallback.
    ///
    /// Neither is a server we run; both are somebody's. Use [`Reach::Own`] to
    /// move them onto hardware you control, or [`Reach::Direct`] to have
    /// neither.
    #[default]
    Default,
    /// The engineer's own relay, self-hosted (`iroh-relay` is separately
    /// published). Same seam `HICKORY_PUBLIC_URL` and PortZero already are.
    Own(String),
    /// No relays at all: a direct path or nothing. The LAN and the
    /// engineer's-own-overlay cases still work; two machines behind different
    /// NATs do not, and are reported unreachable rather than silently routed.
    Direct,
}

impl Reach {
    /// Read from the environment, defaulting to [`Reach::Default`].
    ///
    /// A knob that could reasonably differ between users is a variable, per
    /// `config-and-environments` — and this one differs by how much somebody
    /// minds a third party in the path.
    pub fn from_env() -> Result<Self> {
        match std::env::var("HICKORY_FLEET_RELAY").as_deref() {
            Err(_) | Ok("") | Ok("default") => Ok(Reach::Default),
            Ok("none") | Ok("direct") => Ok(Reach::Direct),
            Ok(url) if url.starts_with("http://") || url.starts_with("https://") => {
                Ok(Reach::Own(url.to_string()))
            }
            Ok(other) => bail!(
                "HICKORY_FLEET_RELAY is {other:?}, which is not a setting this \
                 understands.\n  \
                 Valid values: `default` (number0's relays — traffic may transit \
                 them when no direct path exists, and this machine says so when \
                 it does), `direct` (no relays at all: a direct path or nothing), \
                 or the URL of a relay you run yourself.\n  \
                 Unset it to get `default`."
            ),
        }
    }

    /// One line for a person, said wherever reachability is shown.
    pub fn summary(&self) -> String {
        match self {
            Reach::Default => "Reachable directly where possible, and through \
                               number0's relays otherwise. Neither is a server \
                               we run, and both are somebody's: this machine's \
                               addresses are published to number0's DNS so a \
                               peer can find it by key, and traffic transits \
                               their relays when no direct path exists — \
                               encrypted end to end, but transiting. A relayed \
                               connection is shown as relayed."
                .to_string(),
            Reach::Own(url) => format!(
                "Reachable directly where possible, and through your own relay \
                 at {url} otherwise. Nothing transits anybody else."
            ),
            Reach::Direct => "Reachable only where a direct path exists — your \
                              LAN, found by mDNS, or an overlay you run. No \
                              relays and nothing published to anybody, so two \
                              machines behind different NATs cannot reach each \
                              other and are reported unreachable rather than \
                              routed through anyone."
                .to_string(),
        }
    }
}

/// How a live connection is actually carried.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Reachability {
    /// A direct path: nothing in the middle.
    Direct,
    /// Through a relay. Encrypted end to end, and still worth saying.
    Relayed,
    /// Not connected.
    None,
}

/// A peer admitted to this machine.
pub struct Admitted {
    pub machine: Machine,
    pub how: Reachability,
}

/// This machine, listening for its own other machines.
pub struct PeerServer {
    endpoint: iroh::Endpoint,
    fleet: Fleet,
    /// The loopback address of this session's own server — what a permitted
    /// request is proxied to.
    local: SocketAddr,
    reach: Reach,
}

impl PeerServer {
    /// Bind an endpoint under this machine's own identity.
    ///
    /// The secret key is the machine's, not a fresh one: the transport dials
    /// by public key, so a second keypair would mean the fleet list
    /// authenticates an identity this machine never connects with.
    pub async fn bind(
        identity: &Identity,
        fleet: Fleet,
        local: SocketAddr,
        reach: Reach,
    ) -> Result<Self> {
        let secret = iroh::SecretKey::from_bytes(&identity.secret_bytes());
        let endpoint = builder_for(&reach)?
            .secret_key(secret)
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .context("could not bind the peer endpoint")?;
        Ok(Self {
            endpoint,
            fleet,
            local,
            reach,
        })
    }

    /// This machine's dialable address.
    pub fn addr(&self) -> iroh::EndpointAddr {
        self.endpoint.addr()
    }

    pub fn endpoint(&self) -> &iroh::Endpoint {
        &self.endpoint
    }

    pub fn reach(&self) -> &Reach {
        &self.reach
    }

    /// Accept connections until the endpoint closes.
    pub async fn serve(self: Arc<Self>) -> Result<()> {
        while let Some(incoming) = self.endpoint.accept().await {
            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.admit(incoming).await {
                    log::debug!("peer: connection ended: {e:#}");
                }
            });
        }
        Ok(())
    }

    /// Take one connection, decide whether its key is one of ours, and serve
    /// it under that machine's grants.
    async fn admit(&self, incoming: iroh::endpoint::Incoming) -> Result<()> {
        let connection = incoming.await.context("the handshake failed")?;
        // Already authenticated by the transport: this IS the peer's public
        // key, not a name it claimed.
        let remote = connection.remote_id();
        let key = base64::engine::general_purpose::STANDARD.encode(remote.as_bytes());

        let Some(machine) = self
            .fleet
            .machines()
            .into_iter()
            .find(|m| m.public_key == key)
        else {
            // A key nobody paired is not a peer. Closed with a reason the
            // other end can print, because the usual cause is that pairing
            // was done in one direction only — a fleet is mutual.
            connection.close(
                1u32.into(),
                b"this machine does not hold your key. A fleet is a MUTUAL list: \
                  run `hick fleet invite` there and `hick fleet accept` here.",
            );
            log::warn!("peer: refused an unpaired key {key}");
            return Ok(());
        };

        log::info!("peer: admitted \"{}\"", machine.name);
        loop {
            match connection.accept_bi().await {
                Ok((send, recv)) => {
                    let machine = machine.clone();
                    let local = self.local;
                    tokio::spawn(async move {
                        if let Err(e) = serve_request(machine, local, send, recv).await {
                            log::debug!("peer: request ended: {e:#}");
                        }
                    });
                }
                Err(_) => return Ok(()),
            }
        }
    }
}

/// The endpoint builder for a reachability posture.
///
/// `presets::N0` is n0's DNS address lookup **and** n0's relays; `Minimal` is
/// neither, and only sets the crypto provider that binding requires. Reaching
/// for `Minimal` and adding exactly what was asked for is what keeps a
/// third-party surface from arriving as a default nobody chose.
pub(crate) fn builder_for(reach: &Reach) -> Result<iroh::endpoint::Builder> {
    // mDNS on every posture. It is local, costs nothing, involves no third
    // party, and it is what makes `Reach::Direct` a usable answer rather than
    // "you must already know the address" — including for a spoken pairing
    // phrase, which carries no address at all.
    let mdns = iroh_mdns_address_lookup::MdnsAddressLookup::builder();
    Ok(match reach {
        Reach::Default => iroh::Endpoint::builder(iroh::endpoint::presets::N0).address_lookup(mdns),
        Reach::Direct => {
            iroh::Endpoint::builder(iroh::endpoint::presets::Minimal).address_lookup(mdns)
        }
        Reach::Own(url) => {
            let url: iroh::RelayUrl = url
                .parse()
                .with_context(|| format!("{url} is not a relay URL"))?;
            iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
                .address_lookup(mdns)
                .relay_mode(iroh::endpoint::RelayMode::Custom(
                    iroh::RelayMap::from_iter([iroh::RelayConfig::new(url, None)]),
                ))
        }
    })
}

/// One request from a peer: read its head, check it, proxy or refuse.
async fn serve_request(
    machine: Machine,
    local: SocketAddr,
    mut send: iroh::endpoint::SendStream,
    mut recv: iroh::endpoint::RecvStream,
) -> Result<()> {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        if head.len() > 16 * 1024 {
            bail!("request head is too long");
        }
        match recv.read(&mut byte).await? {
            Some(0) | None => return Ok(()),
            Some(_) => head.push(byte[0]),
        }
    }
    let text = String::from_utf8_lossy(&head).to_string();
    let (method, path) = request_line(&text).context("no request line")?;

    if let Err(denial) = crate::grants::permitted(&method, &path, &machine.grants, &machine.name) {
        // A refusal the other end can print, never a hang: the same reasoning
        // the broker uses for a denied host, and for the same reason — a
        // client that receives a timeout invents an explanation.
        let body = denial.message;
        let response = format!(
            "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain; charset=utf-8\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        send.write_all(response.as_bytes()).await?;
        send.finish()?;
        log::warn!("peer: refused {method} {path} from \"{}\"", machine.name);
        return Ok(());
    }

    // Permitted: hand it to this session's own server, unchanged. The peer
    // channel is a window onto the session, so what answers is the same code
    // that answers the window on this machine.
    //
    // **ONE request per stream, and this is the security boundary, not a
    // simplification.** The grant check above ran on ONE request head. If the
    // rest of the stream were pumped blindly into a keep-alive upstream
    // connection, a peer could append a second request behind the first and
    // have it served without ever being checked — `GET /api/health` followed
    // by `POST /api/docs/x/run`, on one stream, with only the first inspected.
    // So the body is forwarded by its declared length and nothing more.
    //
    // The exception is an upgrade that the upstream ACCEPTS (101): a
    // websocket is one long bidirectional exchange whose opening request was
    // checked, and pumping it is correct. Note the direction of the test —
    // the upstream's answer decides, never the peer's request, or a peer
    // could ask for an upgrade it does not get and be handed a raw pipe.
    let body_len = content_length(&text);
    if body_len.is_none() {
        let body = "this machine refused a request whose body length it could \
                    not determine (a chunked or unterminated body). One request \
                    per stream is what makes the grant check meaningful, and \
                    that needs a known length.";
        let response = format!(
            "HTTP/1.1 411 Length Required\r\nContent-Type: text/plain; charset=utf-8\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        send.write_all(response.as_bytes()).await?;
        send.finish()?;
        return Ok(());
    }
    let body_len = body_len.unwrap_or(0);

    let mut upstream = tokio::net::TcpStream::connect(local)
        .await
        .with_context(|| format!("could not reach this session's server at {local}"))?;
    upstream.write_all(&head).await?;

    // Exactly the declared body, then stop reading from the peer.
    let mut remaining = body_len;
    let mut buf = vec![0u8; 16 * 1024];
    while remaining > 0 {
        let want = remaining.min(buf.len());
        match recv.read(&mut buf[..want]).await? {
            Some(0) | None => break,
            Some(n) => {
                upstream.write_all(&buf[..n]).await?;
                remaining -= n;
            }
        }
    }

    // The response head, so we can tell an accepted upgrade from anything
    // else before deciding whether this stream keeps living.
    let mut resp = Vec::new();
    let mut byte = [0u8; 1];
    while !resp.ends_with(b"\r\n\r\n") {
        if resp.len() > 64 * 1024 {
            break;
        }
        match upstream.read(&mut byte).await {
            Ok(0) | Err(_) => break,
            Ok(_) => resp.push(byte[0]),
        }
    }
    let upgraded = String::from_utf8_lossy(&resp).starts_with("HTTP/1.1 101");
    send.write_all(&resp).await?;

    let (mut up_read, mut up_write) = upstream.into_split();
    if upgraded {
        // A live room: both directions, until either end hangs up.
        let pump_up = async move {
            let mut buf = vec![0u8; 16 * 1024];
            while let Ok(Some(n)) = recv.read(&mut buf).await {
                if n == 0 || up_write.write_all(&buf[..n]).await.is_err() {
                    break;
                }
            }
            let _ = up_write.shutdown().await;
        };
        let pump_down = async move {
            let mut buf = vec![0u8; 16 * 1024];
            loop {
                match up_read.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if send.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                }
            }
            let _ = send.finish();
        };
        tokio::join!(pump_up, pump_down);
        return Ok(());
    }

    // Ordinary response: upstream to peer only, then the stream is done. The
    // peer never gets a second unchecked request onto this socket.
    let mut buf = vec![0u8; 16 * 1024];
    loop {
        match up_read.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if send.write_all(&buf[..n]).await.is_err() {
                    break;
                }
            }
        }
    }
    let _ = send.finish();
    Ok(())
}

/// The declared body length of a request head.
///
/// `Some(0)` when there is no body, `Some(n)` for a `Content-Length`, and
/// **`None` when it cannot be known** — a chunked or unterminated body, which
/// is refused rather than guessed at, because guessing wrong is what would let
/// a second request ride in behind the first.
pub fn content_length(head: &str) -> Option<usize> {
    let mut chunked = false;
    let mut len: Option<usize> = None;
    for line in head.lines().skip(1) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim();
        if name == "transfer-encoding" && value.to_ascii_lowercase().contains("chunked") {
            chunked = true;
        }
        if name == "content-length" {
            match value.parse::<usize>() {
                Ok(n) => len = Some(n),
                Err(_) => return None,
            }
        }
    }
    if chunked {
        return None;
    }
    Some(len.unwrap_or(0))
}

/// `METHOD /path HTTP/1.1` from a request head.
pub fn request_line(head: &str) -> Option<(String, String)> {
    let line = head.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    if !path.starts_with('/') {
        return None;
    }
    Some((method, path))
}

/// A connection this machine opened to one of its own.
pub struct Attached {
    endpoint: iroh::Endpoint,
    connection: iroh::endpoint::Connection,
}

impl Attached {
    /// Dial a machine by its public key.
    pub async fn connect(
        identity: &Identity,
        addr: iroh::EndpointAddr,
        reach: Reach,
    ) -> Result<Self> {
        let secret = iroh::SecretKey::from_bytes(&identity.secret_bytes());
        let endpoint = builder_for(&reach)?
            .secret_key(secret)
            .bind()
            .await
            .context("could not bind the local endpoint")?;
        let connection = endpoint
            .connect(addr, ALPN)
            .await
            .context("could not reach that machine")?;
        Ok(Self {
            endpoint,
            connection,
        })
    }

    /// Whether this connection is direct or relayed — reported, because a
    /// third party in the path is a thing to be told rather than to discover.
    ///
    /// Direct wins where an IP path is in active use: a connection that found
    /// its way past the NAT is not relayed merely because a relay address is
    /// also known for that peer.
    pub async fn how(&self) -> Reachability {
        let remote = self.connection.remote_id();
        let Some(info) = self.endpoint.remote_info(remote).await else {
            return Reachability::None;
        };
        let mut relayed = false;
        for addr in info.addrs() {
            if !matches!(addr.usage(), iroh::endpoint::TransportAddrUsage::Active) {
                continue;
            }
            match addr.addr() {
                iroh::TransportAddr::Ip(_) => return Reachability::Direct,
                iroh::TransportAddr::Relay(_) => relayed = true,
                _ => {}
            }
        }
        if relayed {
            Reachability::Relayed
        } else {
            Reachability::None
        }
    }

    /// Make one request over the channel.
    pub async fn request(&self, method: &str, path: &str) -> Result<String> {
        let (mut send, mut recv) = self.connection.open_bi().await?;
        send.write_all(format!("{method} {path} HTTP/1.1\r\nHost: peer\r\n\r\n").as_bytes())
            .await?;
        send.finish()?;
        let body = recv.read_to_end(8 * 1024 * 1024).await?;
        Ok(String::from_utf8_lossy(&body).to_string())
    }

    /// The live connection, for callers that need to frame their own streams.
    pub fn connection(&self) -> &iroh::endpoint::Connection {
        &self.connection
    }

    /// Close the connection AND the endpoint.
    ///
    /// Dropping an endpoint without closing it makes iroh log an error, which
    /// a person reads as a failure after a command that worked. Awaiting the
    /// close is what turns "aborting ungracefully" into nothing at all.
    pub async fn close(&self) {
        self.connection.close(0u32.into(), b"done");
        self.endpoint.close().await;
    }

    /// Serve the remote session on a LOCAL port, so anything that speaks HTTP
    /// to it is talking to that machine.
    ///
    /// This is what makes the channel usable rather than merely proven: point
    /// a browser — or the desktop app, which is already a client of a server
    /// it need not be co-located with — at the address this binds, and the
    /// window is the other machine's session.
    ///
    /// **One local connection is one stream, deliberately.** The far end
    /// checks exactly one request head per stream against your grants, so a
    /// browser reusing a connection gets it closed and re-dials, which is
    /// ordinary HTTP. Mapping many requests onto one stream would be the
    /// bypass the far end is careful to prevent.
    ///
    /// **Loopback only.** Binding this anywhere else would hand the remote
    /// session to whatever else can reach the port, with none of the key
    /// checking that got you here.
    pub async fn forward(&self, listen: SocketAddr) -> Result<()> {
        if !listen.ip().is_loopback() {
            bail!(
                "{listen} is not a loopback address. This port IS the other \
                 machine's session, with your grants already applied — binding \
                 it where anything else can reach it would hand that session to \
                 whoever finds the port, with none of the key checking that got \
                 you here.\n  \
                 Next step: bind 127.0.0.1 and reach it from this machine."
            );
        }
        let listener = tokio::net::TcpListener::bind(listen)
            .await
            .with_context(|| format!("could not bind {listen}"))?;
        loop {
            let Ok((local, _)) = listener.accept().await else {
                continue;
            };
            let connection = self.connection.clone();
            tokio::spawn(async move {
                if let Err(e) = carry(connection, local).await {
                    log::debug!("peer: forwarded connection ended: {e:#}");
                }
            });
        }
    }
}

/// One local TCP connection, carried over one QUIC stream.
async fn carry(connection: iroh::endpoint::Connection, local: tokio::net::TcpStream) -> Result<()> {
    let (mut send, mut recv) = connection.open_bi().await?;
    let (mut local_read, mut local_write) = local.into_split();

    let up = async move {
        let mut buf = vec![0u8; 16 * 1024];
        loop {
            match local_read.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if send.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
            }
        }
        let _ = send.finish();
    };
    let down = async move {
        let mut buf = vec![0u8; 16 * 1024];
        while let Ok(Some(n)) = recv.read(&mut buf).await {
            if n == 0 || local_write.write_all(&buf[..n]).await.is_err() {
                break;
            }
        }
        let _ = local_write.shutdown().await;
    };
    tokio::join!(up, down);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_line_is_parsed_and_nonsense_is_not() {
        assert_eq!(
            request_line("GET /api/docs/a HTTP/1.1\r\nHost: x\r\n\r\n"),
            Some(("GET".into(), "/api/docs/a".into()))
        );
        assert!(request_line("garbage\r\n\r\n").is_none());
        // An absolute-form target would let a peer aim this at another host.
        assert!(request_line("GET http://elsewhere/ HTTP/1.1\r\n\r\n").is_none());
    }

    #[test]
    fn the_relay_setting_is_typed_and_a_bad_value_says_what_is_valid() {
        // `config-and-environments`: fail with a message naming the variable,
        // what was wrong, and what a valid value looks like.
        unsafe { std::env::set_var("HICKORY_FLEET_RELAY", "sometimes") };
        let err = Reach::from_env().unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("HICKORY_FLEET_RELAY"), "{msg}");
        assert!(msg.contains("`direct`"), "{msg}");
        unsafe { std::env::remove_var("HICKORY_FLEET_RELAY") };
        assert_eq!(Reach::from_env().unwrap(), Reach::Default);
    }

    #[test]
    fn a_body_length_is_known_or_the_request_is_refused() {
        // One request per stream is what makes the grant check meaningful,
        // and that needs a length. A body we cannot measure is refused rather
        // than guessed at — guessing short is exactly how a second, unchecked
        // request rides in behind the first.
        assert_eq!(
            content_length("GET /x HTTP/1.1\r\nHost: h\r\n\r\n"),
            Some(0)
        );
        assert_eq!(
            content_length("POST /x HTTP/1.1\r\nContent-Length: 12\r\n\r\n"),
            Some(12)
        );
        // Case-insensitive, because a header name is.
        assert_eq!(
            content_length("POST /x HTTP/1.1\r\ncontent-length: 5\r\n\r\n"),
            Some(5)
        );
        assert_eq!(
            content_length("POST /x HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n"),
            None
        );
        assert_eq!(
            content_length("POST /x HTTP/1.1\r\nContent-Length: banana\r\n\r\n"),
            None
        );
        // The request LINE is never mistaken for a header.
        assert_eq!(
            content_length("GET /content-length:9 HTTP/1.1\r\n\r\n"),
            Some(0)
        );
    }

    #[test]
    fn every_reach_says_who_is_in_the_path() {
        // The default transits somebody else's infrastructure AND publishes
        // this machine's addresses to it. Both are said, rather than left to
        // be discovered — which is the whole reason this string exists.
        let default = Reach::Default.summary();
        assert!(default.contains("number0"), "{default}");
        assert!(default.contains("Neither is a server we run"), "{default}");
        assert!(default.contains("published"), "{default}");
        assert!(default.contains("transiting"), "{default}");
        // Direct still finds a LAN peer, and says how — mDNS is local, so
        // "no third party" and "no discovery" are not the same claim.
        let direct = Reach::Direct.summary();
        assert!(direct.contains("mDNS"), "{direct}");
        assert!(direct.contains("nothing published to anybody"), "{direct}");
        assert!(
            Reach::Own("https://r".into())
                .summary()
                .contains("your own relay")
        );
        assert!(Reach::Direct.summary().contains("No relays"));
    }
}
