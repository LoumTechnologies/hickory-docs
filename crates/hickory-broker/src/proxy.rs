//! The broker: a CONNECT proxy with a host policy and a log.
//!
//! One process, on **a different machine** than the sealed one — the
//! engineer's laptop, a NAS, a Raspberry Pi. It is a first-party HTTP proxy
//! and nothing else: it does not execute documents, does not store them, and
//! never accepts a connection from outside the local network.
//!
//! **`allow` and `deny` only, here.** No TLS termination, no keys, no CA —
//! which is most of the value, and which is why it stands alone. A `CONNECT`
//! tunnel is forwarded byte-for-byte, so the broker sees the hostname and the
//! byte count and nothing else; `substitute` is the verb that would put it
//! inside the connection, and that is a later step with a real cost that must
//! be stated rather than discovered.
//!
//! Say **"one road out, with a toll booth"** — never "airgapped", which a
//! machine that talks to a model is not.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context as _, Result};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};

use crate::policy::{BrokerLog, LogEntry, Policy, Verb, denial};

/// How the broker was configured, and where it writes.
pub struct Broker {
    pub policy: Policy,
    pub log: Arc<BrokerLog>,
    /// Supplies the timestamp for a log line. There is no clock in this crate,
    /// so a test and a real run produce comparable bytes.
    pub now: Arc<dyn Fn() -> String + Send + Sync>,
}

/// What one request resolved to. Returned so a caller can assert on it
/// without parsing the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub host: String,
    pub port: u16,
    pub verb: Verb,
}

/// Parse a `CONNECT host:port HTTP/1.1` request line.
///
/// Only CONNECT: the broker is the one road out for TLS traffic to a model,
/// and a plain proxied GET would be a second road with different properties.
/// Refusing it is clearer than quietly supporting two things.
pub fn parse_connect(head: &str) -> Option<(String, u16)> {
    let line = head.lines().next()?;
    let mut parts = line.split_whitespace();
    if !parts.next()?.eq_ignore_ascii_case("CONNECT") {
        return None;
    }
    let target = parts.next()?;
    let (host, port) = target.rsplit_once(':')?;
    Some((
        host.trim_matches(['[', ']']).to_ascii_lowercase(),
        port.parse().ok()?,
    ))
}

/// Whether an address is on this machine or its local network.
///
/// The broker **never accepts a connection from outside the local network**.
/// This is checked rather than assumed, because a broker exposed to the
/// internet is an open proxy, which is the one thing it must never become.
pub fn is_local(addr: &SocketAddr) -> bool {
    match addr.ip() {
        std::net::IpAddr::V4(ip) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
        std::net::IpAddr::V6(ip) => {
            ip.is_loopback()
                // Unique-local (fc00::/7) and link-local (fe80::/10).
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

impl Broker {
    /// Serve until the listener closes. One connection at a time is enough
    /// for one agent and keeps the failure modes countable.
    pub async fn serve(self: Arc<Self>, listener: TcpListener) -> Result<()> {
        loop {
            let (stream, peer) = match listener.accept().await {
                Ok(pair) => pair,
                Err(e) => {
                    log::warn!("broker: accept failed: {e}");
                    continue;
                }
            };
            let broker = self.clone();
            tokio::spawn(async move {
                if let Err(e) = broker.handle(stream, peer).await {
                    log::debug!("broker: connection from {peer} ended: {e:#}");
                }
            });
        }
    }

    async fn handle(&self, mut client: TcpStream, peer: SocketAddr) -> Result<()> {
        if !is_local(&peer) {
            // Never a sentence explaining the policy: a stranger who reached
            // this port learns nothing from us.
            let _ = client
                .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
                .await;
            log::warn!("broker: refused a connection from {peer}, which is not local");
            return Ok(());
        }

        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            if head.len() > 8192 {
                anyhow::bail!("request head is too long to be a CONNECT");
            }
            let n = client.read(&mut byte).await?;
            if n == 0 {
                return Ok(());
            }
            head.push(byte[0]);
        }
        let head = String::from_utf8_lossy(&head).to_string();

        let Some((host, port)) = parse_connect(&head) else {
            let body = "the broker speaks CONNECT only: it is the one road out \
                        for this machine, and a second kind of proxied request \
                        would be a second road with different properties.";
            respond(&mut client, 405, body).await?;
            return Ok(());
        };

        let verb = self.policy.verb_for(&host);
        match verb {
            Verb::Allow => self.forward(client, &host, port).await,
            // `ask` needs somewhere to ask, and that is the fleet channel —
            // which is not built. Denying with the reason is the honest
            // behaviour: an agent that hangs for an hour produces a
            // half-finished turn nobody can explain.
            Verb::Ask => {
                let reason = format!(
                    "{} (`{host}` is configured to ASK a human, and asking needs \
                     the fleet channel, which this build does not have — so it \
                     is denied rather than left hanging)",
                    denial(&host)
                );
                self.refuse(client, &host, port, verb, reason).await
            }
            Verb::Substitute => {
                let reason = format!(
                    "{} (`{host}` is configured to SUBSTITUTE a credential, which \
                     needs the broker inside the connection — a CA and TLS \
                     termination — and this build has neither. Nothing was \
                     forwarded and no credential was used.)",
                    denial(&host)
                );
                self.refuse(client, &host, port, verb, reason).await
            }
            Verb::Deny => {
                let reason = denial(&host);
                self.refuse(client, &host, port, verb, reason).await
            }
        }
    }

    async fn refuse(
        &self,
        mut client: TcpStream,
        host: &str,
        port: u16,
        verb: Verb,
        reason: String,
    ) -> Result<()> {
        self.log.append(&LogEntry {
            at: (self.now)(),
            host: host.to_string(),
            port,
            verb,
            bytes: None,
            reason: Some(reason.clone()),
        })?;
        // A structured refusal the agent surfaces verbatim, never a timeout.
        respond(&mut client, 403, &reason).await
    }

    async fn forward(&self, mut client: TcpStream, host: &str, port: u16) -> Result<()> {
        let upstream = TcpStream::connect((host, port)).await;
        let mut upstream = match upstream {
            Ok(s) => s,
            Err(e) => {
                let reason = format!(
                    "the broker allows `{host}` but could not reach it ({e}). \
                     That is the network, not the policy."
                );
                self.log.append(&LogEntry {
                    at: (self.now)(),
                    host: host.to_string(),
                    port,
                    verb: Verb::Allow,
                    bytes: None,
                    reason: Some(reason.clone()),
                })?;
                return respond(&mut client, 502, &reason).await;
            }
        };
        client
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await
            .context("could not answer the CONNECT")?;

        // Byte-for-byte, in both directions. The broker sees the hostname and
        // the count; it is outside the TLS session and cannot see a body.
        let (bytes, _) = tokio::io::copy_bidirectional(&mut client, &mut upstream)
            .await
            .unwrap_or((0, 0));

        self.log.append(&LogEntry {
            at: (self.now)(),
            host: host.to_string(),
            port,
            verb: Verb::Allow,
            bytes: Some(bytes),
            reason: None,
        })?;
        Ok(())
    }
}

async fn respond(client: &mut TcpStream, status: u16, body: &str) -> Result<()> {
    let reason = match status {
        403 => "Forbidden",
        405 => "Method Not Allowed",
        502 => "Bad Gateway",
        _ => "Error",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/plain; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    client.write_all(head.as_bytes()).await?;
    client.write_all(body.as_bytes()).await?;
    client.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_connect_line_is_parsed_and_anything_else_is_not() {
        assert_eq!(
            parse_connect("CONNECT api.anthropic.com:443 HTTP/1.1\r\nHost: x\r\n\r\n"),
            Some(("api.anthropic.com".to_string(), 443))
        );
        // The broker speaks CONNECT only: a second kind of proxied request
        // would be a second road with different properties.
        assert!(parse_connect("GET http://example.com/ HTTP/1.1\r\n\r\n").is_none());
        assert!(parse_connect("CONNECT no-port HTTP/1.1\r\n\r\n").is_none());
    }

    #[test]
    fn a_host_is_matched_case_insensitively() {
        assert_eq!(
            parse_connect("CONNECT API.Anthropic.COM:443 HTTP/1.1\r\n\r\n"),
            Some(("api.anthropic.com".to_string(), 443))
        );
    }

    #[test]
    fn only_local_addresses_are_accepted() {
        // A broker exposed to the internet is an open proxy, which is the one
        // thing it must never become.
        assert!(is_local(&"127.0.0.1:1".parse().unwrap()));
        assert!(is_local(&"192.168.1.5:1".parse().unwrap()));
        assert!(is_local(&"10.0.0.9:1".parse().unwrap()));
        assert!(is_local(&"[::1]:1".parse().unwrap()));
        assert!(!is_local(&"8.8.8.8:1".parse().unwrap()));
        assert!(!is_local(&"[2001:4860:4860::8888]:1".parse().unwrap()));
    }
}
