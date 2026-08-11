//! The tunnel protocol, shared by `hickory serve --public` and the relay.
//!
//! A session on a laptop dials the relay once, outbound. Every guest's traffic
//! then travels over that single connection, multiplexed by stream id — so the
//! host needs no port forwarding, no inbound firewall rule, and no public
//! address of their own.
//!
//! Both ends depend on this crate rather than on each other's source, for the
//! usual reason: a protocol implemented twice is a protocol that will differ.
//!
//! ## Why frames rather than raw byte tunnelling
//!
//! A guest opens *several* things at once — page loads, API calls, and a
//! long-lived WebSocket carrying the document room. Raw TCP forwarding would
//! need one connection per guest connection, which a laptop behind NAT cannot
//! accept. Framing lets one socket carry all of them, and lets the relay tell
//! the laptop *which* guest connection each byte belongs to.
//!
//! ## Why WebSockets get their own frames
//!
//! An HTTP request is a head, a body, and a reply. A WebSocket is a long
//! conversation in both directions with no reply-shaped end. Both are streams
//! here: `Open` starts one, `Data` carries either direction, `Close` ends it.
//! The difference is only that a WebSocket's `Response` is the 101 that
//! upgrades it, after which `Data` flows both ways until someone closes.
//!
//! See `docs/specs/freeform/relay.md`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Protocol version. Bumped when a frame changes shape; the relay refuses a
/// client that speaks a different one, because a subtly misunderstood frame is
/// worse than a clean refusal.
pub const PROTOCOL_VERSION: u32 = 1;

/// Identifies one guest connection within a tunnel.
pub type StreamId = u64;

/// A guest's request, as much of it as the laptop needs to serve it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestHead {
    pub method: String,
    /// Path and query, e.g. `/api/docs/abc?x=1`.
    pub uri: String,
    pub headers: Vec<(String, String)>,
    /// True when this is a WebSocket upgrade, so the far end knows the reply
    /// will be a 101 and the stream will stay open afterwards.
    pub websocket: bool,
}

impl RequestHead {
    /// First value of a header, case-insensitively.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// The head of a reply.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseHead {
    pub status: u16,
    pub headers: Vec<(String, String)>,
}

/// Why a stream ended. Carried so the far end can tell "the guest went away"
/// from "we could not keep up", which look identical at the socket.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CloseReason {
    /// The end of a well-formed exchange.
    Done,
    /// The peer disconnected.
    PeerGone,
    /// The stream's queue overflowed — see the backpressure note in the spec.
    Overflow,
    /// The far end refused or failed to serve it.
    Failed,
}

/// Everything either end can say.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Frame {
    /// agent → relay: open a tunnel. `token` is a GitHub access token; the
    /// relay exchanges it for an identity and never stores it.
    Hello {
        version: u32,
        token: String,
        /// What the session would like to be called. The relay may ignore it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        preferred_slug: Option<String>,
    },
    /// relay → agent: the tunnel is open at this address.
    Ready {
        url: String,
        slug: String,
        /// Whom the relay believes this tunnel belongs to (a GitHub login),
        /// echoed so the host can see it in their own banner.
        account: String,
        /// When the relay will close this tunnel regardless.
        expires_in_secs: u64,
    },
    /// relay → agent: refused, with a reason meant for a human.
    Refused {
        message: String,
    },
    /// relay → agent: a guest connection begins.
    Open {
        stream: StreamId,
        head: Box<RequestHead>,
    },
    /// agent → relay: the head of the reply to a stream.
    Response {
        stream: StreamId,
        head: Box<ResponseHead>,
    },
    /// Either direction: body bytes, or one WebSocket message.
    Data {
        stream: StreamId,
        #[serde(with = "base64_bytes")]
        bytes: Vec<u8>,
        /// True when these bytes are a WebSocket *text* frame rather than
        /// binary. The Yjs channel is binary and the LSP bridge is text, so
        /// the distinction has to survive the tunnel.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        text: bool,
    },
    /// Either direction: this stream is over.
    Close {
        stream: StreamId,
        reason: CloseReason,
    },
    Ping,
    Pong,
}

impl Frame {
    /// Encode for the wire.
    pub fn encode(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Decode from the wire.
    pub fn decode(bytes: &[u8]) -> Result<Frame, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// The stream this frame belongs to, if any.
    pub fn stream(&self) -> Option<StreamId> {
        match self {
            Frame::Open { stream, .. }
            | Frame::Response { stream, .. }
            | Frame::Data { stream, .. }
            | Frame::Close { stream, .. } => Some(*stream),
            _ => None,
        }
    }
}

/// Base64 for the byte payloads.
///
/// The frames are JSON so both ends stay readable in a log and a mismatch is
/// obvious rather than a misaligned offset. Bodies are base64 inside that,
/// which costs a third more bytes on the wire and buys a protocol a person can
/// debug — the right trade for a control channel carrying documents, not video.
mod base64_bytes {
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD as B64;
    use serde::{Deserialize as _, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&B64.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(d)?;
        B64.decode(text.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Slugs
// ---------------------------------------------------------------------------

/// Characters a slug may contain: lowercase, digits, and a hyphen inside.
///
/// A slug becomes a DNS label, so it must be a valid one — and must not be
/// able to become a *different* address than the relay intended, which is what
/// a dot or an uppercase letter could do.
pub fn slug_is_valid(slug: &str) -> bool {
    let len = slug.len();
    if !(3..=40).contains(&len) {
        return false;
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return false;
    }
    slug.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

// ---------------------------------------------------------------------------
// Header hygiene
// ---------------------------------------------------------------------------

/// Headers that must not be forwarded verbatim.
///
/// Hop-by-hop headers describe *this* connection, not the message, so passing
/// them across a proxy boundary produces exactly the confusing failures they
/// were defined to prevent (RFC 9110 §7.6.1). `host` is dropped separately: the
/// guest asked for the relay's hostname and the laptop should answer as itself.
const HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// Whether a header should cross the tunnel.
pub fn is_forwardable(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !HOP_BY_HOP.contains(&lower.as_str()) && lower != "host"
}

/// Filter a header list for forwarding.
pub fn forwardable(headers: &[(String, String)]) -> Vec<(String, String)> {
    headers
        .iter()
        .filter(|(k, _)| is_forwardable(k))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Quotas
// ---------------------------------------------------------------------------

/// The limits the relay applies per account. Small on purpose: a quota system
/// with exceptions is a billing system, and pricing is unsettled.
#[derive(Debug, Clone, Copy)]
pub struct Quota {
    pub max_tunnels_per_account: usize,
    pub tunnel_ttl_secs: u64,
    pub max_streams_per_tunnel: usize,
    /// Frames queued for one stream before it is closed as `Overflow`.
    pub max_queued_frames: usize,
}

impl Default for Quota {
    fn default() -> Self {
        Self {
            max_tunnels_per_account: 3,
            tunnel_ttl_secs: 8 * 60 * 60,
            max_streams_per_tunnel: 256,
            max_queued_frames: 1024,
        }
    }
}

/// Counts tunnels per account so the quota can be enforced without a database.
#[derive(Debug, Default)]
pub struct TunnelCensus {
    per_account: HashMap<String, usize>,
}

impl TunnelCensus {
    /// Record a new tunnel, or refuse it. The message is what the host sees.
    pub fn admit(&mut self, account: &str, quota: &Quota) -> Result<(), String> {
        let count = self.per_account.entry(account.to_string()).or_insert(0);
        if *count >= quota.max_tunnels_per_account {
            return Err(format!(
                "{account} already has {count} tunnels open (the limit is {}). \
                 Close one — every `hickory serve --public` holds one until you stop it.",
                quota.max_tunnels_per_account
            ));
        }
        *count += 1;
        Ok(())
    }

    pub fn release(&mut self, account: &str) {
        if let Some(count) = self.per_account.get_mut(account) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.per_account.remove(account);
            }
        }
    }

    pub fn open_for(&self, account: &str) -> usize {
        self.per_account.get(account).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip_including_binary_payloads() {
        // The Yjs channel is binary and full of bytes that are not UTF-8; a
        // protocol that mangled them would corrupt documents rather than fail.
        let payload: Vec<u8> = (0u8..=255).collect();
        let frame = Frame::Data {
            stream: 7,
            bytes: payload.clone(),
            text: false,
        };
        let decoded = Frame::decode(&frame.encode().unwrap()).unwrap();
        assert_eq!(decoded, frame);
        match decoded {
            Frame::Data { bytes, .. } => assert_eq!(bytes, payload),
            other => panic!("wrong frame: {other:?}"),
        }
    }

    #[test]
    fn a_text_websocket_message_stays_text_across_the_tunnel() {
        // The LSP bridge is text and the document channel is binary; losing
        // the distinction would break one of them.
        let frame = Frame::Data {
            stream: 1,
            bytes: b"{\"jsonrpc\":\"2.0\"}".to_vec(),
            text: true,
        };
        let decoded = Frame::decode(&frame.encode().unwrap()).unwrap();
        assert_eq!(decoded, frame);
    }

    #[test]
    fn heads_and_control_frames_round_trip() {
        let head = RequestHead {
            method: "GET".into(),
            uri: "/api/ws?doc=doc:abc&token=xyz".into(),
            headers: vec![("upgrade".into(), "websocket".into())],
            websocket: true,
        };
        let frame = Frame::Open {
            stream: 3,
            head: Box::new(head.clone()),
        };
        assert_eq!(Frame::decode(&frame.encode().unwrap()).unwrap(), frame);
        assert_eq!(head.header("UPGRADE"), Some("websocket"));

        for frame in [
            Frame::Ping,
            Frame::Pong,
            Frame::Refused {
                message: "nope".into(),
            },
            Frame::Close {
                stream: 9,
                reason: CloseReason::Overflow,
            },
        ] {
            assert_eq!(Frame::decode(&frame.encode().unwrap()).unwrap(), frame);
        }
    }

    #[test]
    fn hop_by_hop_headers_do_not_cross_the_boundary() {
        let headers = vec![
            ("Host".into(), "x.relay.hickorydocs.com".into()),
            ("Connection".into(), "upgrade".into()),
            ("Authorization".into(), "Bearer t".into()),
            ("Content-Type".into(), "application/json".into()),
        ];
        let forwarded = forwardable(&headers);
        let kept: Vec<&str> = forwarded.iter().map(|(k, _)| k.as_str()).collect();
        assert!(kept.contains(&"Authorization"), "{kept:?}");
        assert!(kept.contains(&"Content-Type"), "{kept:?}");
        assert!(!kept.iter().any(|k| k.eq_ignore_ascii_case("host")));
        assert!(!kept.iter().any(|k| k.eq_ignore_ascii_case("connection")));
    }

    #[test]
    fn slugs_are_dns_labels_and_nothing_cleverer() {
        assert!(slug_is_valid("quiet-cedar-1042"));
        assert!(!slug_is_valid("ab")); // too short
        assert!(!slug_is_valid("-leading"));
        assert!(!slug_is_valid("trailing-"));
        assert!(!slug_is_valid("Upper"));
        // The dangerous one: a dot would make this a different address.
        assert!(!slug_is_valid("evil.example.com"));
        assert!(!slug_is_valid("has space"));
    }

    #[test]
    fn the_quota_counts_per_account_and_says_what_to_do() {
        let quota = Quota::default();
        let mut census = TunnelCensus::default();
        for _ in 0..quota.max_tunnels_per_account {
            census.admit("nate", &quota).unwrap();
        }
        let err = census.admit("nate", &quota).unwrap_err();
        assert!(err.contains("limit is 3"), "{err}");
        assert!(err.contains("Close one"), "{err}");

        // Someone else is unaffected — the limit is per account, not global.
        census.admit("someone-else", &quota).unwrap();

        census.release("nate");
        census
            .admit("nate", &quota)
            .expect("a freed slot is reusable");
        assert_eq!(census.open_for("someone-else"), 1);
    }

    #[test]
    fn releasing_an_unknown_account_is_harmless() {
        // Close paths run on every failure route; one that could panic would
        // take the relay down for everyone.
        let mut census = TunnelCensus::default();
        census.release("nobody");
        assert_eq!(census.open_for("nobody"), 0);
    }
}
