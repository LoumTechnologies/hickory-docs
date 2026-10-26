<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# One Road Out, With A Toll Booth On It — Never An Airgap

For: Engineers using local sessions or pairing their own machines.

Given a machine bought to run an agent on, when it is sealed and pointed at a
broker, then it holds no credential worth stealing and reaches the network only
through one process the engineer runs — and every host that process forwards to
is one the engineer allowed by name.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/collaboration/one-road-out-with-a-toll-booth.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-ff3e7f8e3a1e"></a>
## `crates/hickory-broker/src/policy.rs`

Related contracts: [One Road Out, With A Toll Booth On It — Never An Airgap](contract.md).

<a id="f-ff3e7f8e3a1e-Verb"></a>
### `Verb`

Per-host policy, and the log of what the broker did.
See `docs/specs/freeform/the-broker-and-the-sealed-machine.md`.

<storycode:copy id="f-ff3e7f8e3a1e-Verb">
//! Per-host policy, and the log of what the broker did.
//!
//! See `docs/specs/freeform/the-broker-and-the-sealed-machine.md`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use serde::{Deserialize, Serialize};

/// What the broker does with a request to a host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verb {
    /// Forward it. The broker sees the hostname only.
    Allow,
    /// Refuse it. Hostname only.
    Deny,
    /// Hold it and ask a human, over the fleet channel. Hostname only.
    Ask,
    /// Forward it, replacing the stub credential with the real one — which
    /// requires terminating TLS, so the broker sees **the whole request and
    /// response**. Never reached by an upgrade from `allow`: a host is
    /// configured for it deliberately, because "we can read your model
    /// traffic" must be stated rather than discovered.
    Substitute,
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-impl-Verb"></a>
### `impl-Verb`

<storycode:copy id="f-ff3e7f8e3a1e-impl-Verb">
impl Verb {
    pub fn as_str(self) -> &'static str {
        match self {
            Verb::Allow => "allow",
            Verb::Deny => "deny",
            Verb::Ask => "ask",
            Verb::Substitute => "substitute",
        }
    }

    /// Whether this verb requires the broker to be inside the connection.
    pub fn reads_the_body(self) -> bool {
        matches!(self, Verb::Substitute)
    }
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-Policy"></a>
### `Policy`

The engineer's policy: what each host may do, and what an unlisted one gets.

<storycode:copy id="f-ff3e7f8e3a1e-Policy">
/// The engineer's policy: what each host may do, and what an unlisted one gets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Host → verb. Exact hostnames, and `*.suffix` for a subtree.
    #[serde(default)]
    pub hosts: BTreeMap<String, Verb>,
    /// What a host nobody listed gets.
    ///
    /// **Deny.** A default of anything else would make the toll booth a sign
    /// rather than a booth — and the whole claim is that there is exactly one
    /// road out and something is standing on it.
    #[serde(default = "default_deny")]
    pub default: Verb,
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-default_deny"></a>
### `default_deny`

<storycode:copy id="f-ff3e7f8e3a1e-default_deny">
fn default_deny() -> Verb {
    Verb::Deny
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-impl-Default-for-Policy"></a>
### `impl-Default for Policy`

<storycode:copy id="f-ff3e7f8e3a1e-impl-Default-for-Policy">
impl Default for Policy {
    fn default() -> Self {
        Self {
            hosts: BTreeMap::new(),
            default: Verb::Deny,
        }
    }
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-impl-Policy"></a>
### `impl-Policy`

<storycode:copy id="f-ff3e7f8e3a1e-impl-Policy">
impl Policy {
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(raw) => serde_json::from_str(&raw)
                .with_context(|| format!("unreadable broker policy in {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("could not write {}", path.display()))
    }

    /// The verb for `host`.
    ///
    /// Exact match first, then the longest matching `*.suffix`: a rule about
    /// `*.example.com` must not quietly outrank one about
    /// `api.internal.example.com`.
    pub fn verb_for(&self, host: &str) -> Verb {
        let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
        if let Some(v) = self.hosts.get(&host) {
            return *v;
        }
        let mut best: Option<(usize, Verb)> = None;
        for (pattern, verb) in &self.hosts {
            let Some(suffix) = pattern.strip_prefix("*.") else {
                continue;
            };
            let suffix = suffix.to_ascii_lowercase();
            if host == suffix || host.ends_with(&format!(".{suffix}")) {
                let len = suffix.len();
                if best.is_none_or(|(best_len, _)| len > best_len) {
                    best = Some((len, *verb));
                }
            }
        }
        best.map(|(_, v)| v).unwrap_or(self.default)
    }

    /// Set a host's verb.
    ///
    /// **A host is never silently upgraded to `substitute`**, because that is
    /// the verb that puts the broker inside the connection. Going there is an
    /// explicit act, and this refuses to do it as a side effect of something
    /// else.
    pub fn set(&mut self, host: &str, verb: Verb, deliberate: bool) -> Result<()> {
        if verb == Verb::Substitute && !deliberate {
            bail!(
                "`substitute` puts the broker INSIDE the connection: it \
                 terminates TLS, so it can read every byte of that host's \
                 traffic in both directions.\n  \
                 That is acceptable — both machines are yours and the traffic \
                 is your own — but it is never something to arrive at by \
                 accident, so it has to be asked for explicitly.\n  \
                 Next step: pass --substitute to say you mean it."
            );
        }
        self.hosts.insert(host.trim().to_ascii_lowercase(), verb);
        Ok(())
    }
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-LogEntry"></a>
### `LogEntry`

One line of what the broker did.
Append-only, and the engineer's to read. A hostname, never a body: the
three verbs that do not terminate TLS cannot see one, and the log does not
pretend otherwise.

<storycode:copy id="f-ff3e7f8e3a1e-LogEntry">
/// One line of what the broker did.
///
/// Append-only, and the engineer's to read. A hostname, never a body: the
/// three verbs that do not terminate TLS cannot see one, and the log does not
/// pretend otherwise.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogEntry {
    pub at: String,
    pub host: String,
    pub port: u16,
    pub verb: Verb,
    /// Bytes carried, when the connection was forwarded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    /// Why, when it was refused — the same sentence the agent was given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-BrokerLog"></a>
### `BrokerLog`

<storycode:copy id="f-ff3e7f8e3a1e-BrokerLog">
/// The broker's append-only log.
pub struct BrokerLog {
    path: PathBuf,
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-impl-BrokerLog"></a>
### `impl-BrokerLog`

<storycode:copy id="f-ff3e7f8e3a1e-impl-BrokerLog">
impl BrokerLog {
    pub fn at(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, entry: &LogEntry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("opening {}", self.path.display()))?;
        writeln!(file, "{}", serde_json::to_string(entry)?)
            .with_context(|| format!("writing {}", self.path.display()))
    }

    pub fn read(&self) -> Vec<LogEntry> {
        std::fs::read_to_string(&self.path)
            .ok()
            .map(|raw| {
                raw.lines()
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(|l| serde_json::from_str(l).ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-denial"></a>
### `denial`

The body of the broker's refusal, which the agent surfaces verbatim.
**A denial must be legible to the agent, not a hang.** An agent that
receives a timeout invents a reason; an agent that receives a sentence
repeats it.

<storycode:copy id="f-ff3e7f8e3a1e-denial">
/// The body of the broker's refusal, which the agent surfaces verbatim.
///
/// **A denial must be legible to the agent, not a hang.** An agent that
/// receives a timeout invents a reason; an agent that receives a sentence
/// repeats it.
pub fn denial(host: &str) -> String {
    format!(
        "the broker denied a request to `{host}`; ask the engineer to allow \
         that host. This machine has one road out and a toll booth on it — it \
         holds no credential that is worth anything anywhere else, and it \
         cannot reach the internet except through the broker."
    )
}

</storycode:copy>
<a id="f-ff3e7f8e3a1e-tests"></a>
### `tests`

<storycode:copy id="f-ff3e7f8e3a1e-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unlisted_host_is_denied() {
        // A default of anything else makes the toll booth a sign rather than
        // a booth.
        let policy = Policy::default();
        assert_eq!(policy.verb_for("evil.example"), Verb::Deny);
    }

    #[test]
    fn an_exact_host_beats_a_wildcard() {
        let mut policy = Policy::default();
        policy.set("*.example.com", Verb::Allow, false).unwrap();
        policy
            .set("secrets.example.com", Verb::Deny, false)
            .unwrap();
        assert_eq!(policy.verb_for("api.example.com"), Verb::Allow);
        assert_eq!(policy.verb_for("secrets.example.com"), Verb::Deny);
    }

    #[test]
    fn the_longest_wildcard_wins_rather_than_the_first() {
        // A rule about `*.example.com` must not quietly outrank one about
        // `*.internal.example.com`.
        let mut policy = Policy::default();
        policy.set("*.example.com", Verb::Allow, false).unwrap();
        policy
            .set("*.internal.example.com", Verb::Deny, false)
            .unwrap();
        assert_eq!(policy.verb_for("db.internal.example.com"), Verb::Deny);
        assert_eq!(policy.verb_for("api.example.com"), Verb::Allow);
    }

    #[test]
    fn a_host_is_never_silently_upgraded_to_substitute() {
        // It is the verb that puts the broker inside the connection.
        let mut policy = Policy::default();
        let err = policy
            .set("api.anthropic.com", Verb::Substitute, false)
            .unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("INSIDE the connection"), "{msg}");
        assert!(msg.contains("--substitute"), "{msg}");
        assert_eq!(policy.verb_for("api.anthropic.com"), Verb::Deny);
        policy
            .set("api.anthropic.com", Verb::Substitute, true)
            .unwrap();
        assert_eq!(policy.verb_for("api.anthropic.com"), Verb::Substitute);
    }

    #[test]
    fn only_substitute_reads_the_body() {
        assert!(!Verb::Allow.reads_the_body());
        assert!(!Verb::Deny.reads_the_body());
        assert!(!Verb::Ask.reads_the_body());
        assert!(Verb::Substitute.reads_the_body());
    }

    #[test]
    fn a_denial_is_a_sentence_the_agent_can_repeat() {
        let text = denial("raw.githubusercontent.com");
        assert!(text.contains("raw.githubusercontent.com"));
        assert!(text.contains("ask the engineer to allow that host"));
        // Never "airgapped": a machine that reaches a model is on a network.
        assert!(!text.contains("airgap"));
        assert!(text.contains("one road out"));
    }

    #[test]
    fn a_policy_round_trips_and_an_absent_file_is_deny_everything() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("policy.json");
        assert_eq!(Policy::load(&path).unwrap().verb_for("x"), Verb::Deny);
        let mut policy = Policy::default();
        policy.set("api.anthropic.com", Verb::Allow, false).unwrap();
        policy.save(&path).unwrap();
        assert_eq!(
            Policy::load(&path).unwrap().verb_for("api.anthropic.com"),
            Verb::Allow
        );
    }

    #[test]
    fn the_log_is_append_only_and_names_the_verb() {
        let dir = tempfile::tempdir().unwrap();
        let log = BrokerLog::at(&dir.path().join("broker.jsonl"));
        log.append(&LogEntry {
            at: "2026-08-23T00:00:00Z".into(),
            host: "api.anthropic.com".into(),
            port: 443,
            verb: Verb::Allow,
            bytes: Some(1024),
            reason: None,
        })
        .unwrap();
        log.append(&LogEntry {
            at: "2026-08-23T00:00:01Z".into(),
            host: "evil.example".into(),
            port: 443,
            verb: Verb::Deny,
            bytes: None,
            reason: Some(denial("evil.example")),
        })
        .unwrap();
        let entries = log.read();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].verb, Verb::Deny);
        assert!(entries[1].reason.is_some());
    }
}
</storycode:copy>

### Assemble `crates/hickory-broker/src/policy.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-broker/src/policy.rs"><storycode:paste select="#f-ff3e7f8e3a1e-Verb"/><storycode:paste select="#f-ff3e7f8e3a1e-impl-Verb"/><storycode:paste select="#f-ff3e7f8e3a1e-Policy"/><storycode:paste select="#f-ff3e7f8e3a1e-default_deny"/><storycode:paste select="#f-ff3e7f8e3a1e-impl-Default-for-Policy"/><storycode:paste select="#f-ff3e7f8e3a1e-impl-Policy"/><storycode:paste select="#f-ff3e7f8e3a1e-LogEntry"/><storycode:paste select="#f-ff3e7f8e3a1e-BrokerLog"/><storycode:paste select="#f-ff3e7f8e3a1e-impl-BrokerLog"/><storycode:paste select="#f-ff3e7f8e3a1e-denial"/><storycode:paste select="#f-ff3e7f8e3a1e-tests"/></storycode:file>

<a id="output-24f287415c45"></a>
## `crates/hickory-broker/src/proxy.rs`

Related contracts: [One Road Out, With A Toll Booth On It — Never An Airgap](contract.md).

<a id="f-24f287415c45-Broker"></a>
### `Broker`

The broker: a CONNECT proxy with a host policy and a log.
One process, on **a different machine** than the sealed one — the
engineer's laptop, a NAS, a Raspberry Pi. It is a first-party HTTP proxy
and nothing else: it does not execute documents, does not store them, and
never accepts a connection from outside the local network.
**`allow` and `deny` only, here.** No TLS termination, no keys, no CA —
which is most of the value, and which is why it stands alone. A `CONNECT`
tunnel is forwarded byte-for-byte, so the broker sees the hostname and the
byte count and nothing else; `substitute` is the verb that would put it
inside the connection, and that is a later step with a real cost that must
be stated rather than discovered.
Say **"one road out, with a toll booth"** — never "airgapped", which a
machine that talks to a model is not.

<storycode:copy id="f-24f287415c45-Broker">
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

</storycode:copy>
<a id="f-24f287415c45-Decision"></a>
### `Decision`

What one request resolved to. Returned so a caller can assert on it
without parsing the log.

<storycode:copy id="f-24f287415c45-Decision">
/// What one request resolved to. Returned so a caller can assert on it
/// without parsing the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub host: String,
    pub port: u16,
    pub verb: Verb,
}

</storycode:copy>
<a id="f-24f287415c45-parse_connect"></a>
### `parse_connect`

Parse a `CONNECT host:port HTTP/1.1` request line.
Only CONNECT: the broker is the one road out for TLS traffic to a model,
and a plain proxied GET would be a second road with different properties.
Refusing it is clearer than quietly supporting two things.

<storycode:copy id="f-24f287415c45-parse_connect">
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

</storycode:copy>
<a id="f-24f287415c45-is_local"></a>
### `is_local`

Whether an address is on this machine or its local network.
The broker **never accepts a connection from outside the local network**.
This is checked rather than assumed, because a broker exposed to the
internet is an open proxy, which is the one thing it must never become.

<storycode:copy id="f-24f287415c45-is_local">
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

</storycode:copy>
<a id="f-24f287415c45-impl-Broker"></a>
### `impl-Broker`

<storycode:copy id="f-24f287415c45-impl-Broker">
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

</storycode:copy>
<a id="f-24f287415c45-respond"></a>
### `respond`

<storycode:copy id="f-24f287415c45-respond">
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

</storycode:copy>
<a id="f-24f287415c45-tests"></a>
### `tests`

<storycode:copy id="f-24f287415c45-tests">
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
</storycode:copy>

### Assemble `crates/hickory-broker/src/proxy.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-broker/src/proxy.rs"><storycode:paste select="#f-24f287415c45-Broker"/><storycode:paste select="#f-24f287415c45-Decision"/><storycode:paste select="#f-24f287415c45-parse_connect"/><storycode:paste select="#f-24f287415c45-is_local"/><storycode:paste select="#f-24f287415c45-impl-Broker"/><storycode:paste select="#f-24f287415c45-respond"/><storycode:paste select="#f-24f287415c45-tests"/></storycode:file>

<a id="output-c24a2c2f2c9d"></a>
## `crates/hickory-broker/src/seal.rs`

Related contracts: [One Road Out, With A Toll Booth On It — Never An Airgap](contract.md).

<a id="f-c24a2c2f2c9d-PROVIDER_VARS"></a>
### `PROVIDER_VARS`

The seal: what a machine asserts about itself, and how it is checked.
Sealing is a property a machine asserts about itself, set by the person
standing at it, recorded in that machine's own config. It is deliberately
**not** a grant another machine can hand out or take away: a fleet peer
that could unseal a box would mean a compromised laptop unseals the box,
which inverts the whole point.
`hick sealed --check` is the whole of it as one command. It **verifies
rather than performs**: the default route being denied at the OS is
firewall configuration on the engineer's own machine, and we do not
reconfigure somebody's networking for them.

<storycode:copy id="f-c24a2c2f2c9d-PROVIDER_VARS">
//! The seal: what a machine asserts about itself, and how it is checked.
//!
//! Sealing is a property a machine asserts about itself, set by the person
//! standing at it, recorded in that machine's own config. It is deliberately
//! **not** a grant another machine can hand out or take away: a fleet peer
//! that could unseal a box would mean a compromised laptop unseals the box,
//! which inverts the whole point.
//!
//! `hick sealed --check` is the whole of it as one command. It **verifies
//! rather than performs**: the default route being denied at the OS is
//! firewall configuration on the engineer's own machine, and we do not
//! reconfigure somebody's networking for them.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The provider variables a sealed machine must not have. Their names are the
/// ecosystem's, not ours, so a user's existing environment works without
/// translation — which is exactly why they are the ones to look for.
pub const PROVIDER_VARS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "DEEPSEEK_API_KEY",
    "XAI_API_KEY",
    "OPENROUTER_API_KEY",
    "GAB_API_KEY",
    "GOOGLE_API_KEY",
    "GEMINI_API_KEY",
];

</storycode:copy>
<a id="f-c24a2c2f2c9d-BASE_URL_VARS"></a>
### `BASE_URL_VARS`

The base-URL variables that must point at the broker.

<storycode:copy id="f-c24a2c2f2c9d-BASE_URL_VARS">
/// The base-URL variables that must point at the broker.
pub const BASE_URL_VARS: &[&str] = &[
    "ANTHROPIC_BASE_URL",
    "OPENAI_BASE_URL",
    "DEEPSEEK_BASE_URL",
    "XAI_BASE_URL",
    "OPENROUTER_BASE_URL",
    "GAB_BASE_URL",
];

</storycode:copy>
<a id="f-c24a2c2f2c9d-SealFinding"></a>
### `SealFinding`

<storycode:copy id="f-c24a2c2f2c9d-SealFinding">
/// One thing the check found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealFinding {
    /// Short name of what was checked.
    pub check: String,
    pub ok: bool,
    /// What is true, and — when it is not ok — what to do about it.
    pub detail: String,
}

</storycode:copy>
<a id="f-c24a2c2f2c9d-SealCheck"></a>
### `SealCheck`

<storycode:copy id="f-c24a2c2f2c9d-SealCheck">
/// The whole verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealCheck {
    /// Whether this machine claims to be sealed at all.
    pub sealed: bool,
    /// The broker it says it reaches the model through.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broker: Option<String>,
    pub findings: Vec<SealFinding>,
}

</storycode:copy>
<a id="f-c24a2c2f2c9d-impl-SealCheck"></a>
### `impl-SealCheck`

<storycode:copy id="f-c24a2c2f2c9d-impl-SealCheck">
impl SealCheck {
    pub fn ok(&self) -> bool {
        self.findings.iter().all(|f| f.ok)
    }
}

</storycode:copy>
<a id="f-c24a2c2f2c9d-SealConfig"></a>
### `SealConfig`

What a machine's `sealed.toml`-equivalent holds. JSON rather than TOML
because everything else this product writes is JSON, and one format is one
thing to get wrong.

<storycode:copy id="f-c24a2c2f2c9d-SealConfig">
/// What a machine's `sealed.toml`-equivalent holds. JSON rather than TOML
/// because everything else this product writes is JSON, and one format is one
/// thing to get wrong.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SealConfig {
    #[serde(default)]
    pub sealed: bool,
    /// `http://broker.lan:7749` — where this machine reaches the model.
    #[serde(default)]
    pub broker: Option<String>,
    /// The stub credential. Worthless everywhere else: minted by the broker,
    /// bound to this machine's key, and rejected by the vendor because it was
    /// never a vendor credential. An agent that leaks it leaks nothing.
    #[serde(default)]
    pub stub: Option<String>,
}

</storycode:copy>
<a id="f-c24a2c2f2c9d-impl-SealConfig"></a>
### `impl-SealConfig`

<storycode:copy id="f-c24a2c2f2c9d-impl-SealConfig">
impl SealConfig {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

</storycode:copy>
<a id="f-c24a2c2f2c9d-check_seal"></a>
### `check_seal`

Check the seal, given the config and this machine's environment.
Pure in its inputs so the awkward cases are arguable in a test rather than
against a real machine. `key_file_present` is whether a provider key store
exists on disk; `route_denied` is whether the default route is denied at
the OS, which the caller determines and this only reports.

<storycode:copy id="f-c24a2c2f2c9d-check_seal">
/// Check the seal, given the config and this machine's environment.
///
/// Pure in its inputs so the awkward cases are arguable in a test rather than
/// against a real machine. `key_file_present` is whether a provider key store
/// exists on disk; `route_denied` is whether the default route is denied at
/// the OS, which the caller determines and this only reports.
pub fn check_seal(
    config: &SealConfig,
    env: &BTreeMap<String, String>,
    key_file_present: bool,
    route_denied: Option<bool>,
) -> SealCheck {
    let mut findings = Vec::new();

    if !config.sealed {
        findings.push(SealFinding {
            check: "sealed".into(),
            ok: false,
            detail: "this machine is not sealed. Sealing is something the person \
                     standing at a machine asserts about it — deliberately not a \
                     grant another machine can hand out, because a fleet peer \
                     that could unseal this box would mean a compromised laptop \
                     unseals it.\n  Next step: `hick sealed --set --broker \
                     http://broker.lan:7749`."
                .into(),
        });
        return SealCheck {
            sealed: false,
            broker: config.broker.clone(),
            findings,
        };
    }

    // A provider variable existing at all is a boot failure, not a fallback:
    // silently ignoring it would leave a real key sitting in a process that is
    // not supposed to hold one.
    let present: Vec<&str> = PROVIDER_VARS
        .iter()
        .copied()
        .filter(|v| env.get(*v).is_some_and(|value| !value.trim().is_empty()))
        .collect();
    findings.push(if present.is_empty() {
        SealFinding {
            check: "no provider key in the environment".into(),
            ok: true,
            detail: "no provider variable is set, so there is no real credential \
                     in this process."
                .into(),
        }
    } else {
        SealFinding {
            check: "no provider key in the environment".into(),
            ok: false,
            detail: format!(
                "{} is set on a sealed machine. Unset it; this machine reaches \
                 the model through the broker at {}, using the stub credential \
                 in this machine's seal config.\n  \
                 Leaving it set is not a harmless fallback — it is a real key \
                 sitting in a process that is not supposed to hold one.",
                present.join(", "),
                config.broker.as_deref().unwrap_or("(no broker configured)"),
            ),
        }
    });

    findings.push(SealFinding {
        check: "no key file on disk".into(),
        ok: !key_file_present,
        detail: if key_file_present {
            "a provider key store exists on this machine. On a sealed machine \
             `KeyStore::set` refuses and keys live on the broker.\n  \
             Next step: delete it, and check the broker holds the real key."
                .into()
        } else {
            "no provider key store is on this machine.".into()
        },
    });

    let broker = config.broker.clone();
    findings.push(match &broker {
        Some(url) if !url.trim().is_empty() => SealFinding {
            check: "a broker is configured".into(),
            ok: true,
            detail: format!("this machine reaches the model through {url}."),
        },
        _ => SealFinding {
            check: "a broker is configured".into(),
            ok: false,
            detail: "this machine is sealed but names no broker, so it cannot \
                     use a model at all. That is a coherent state — it is what \
                     the seal alone buys — and it is worth knowing you are in \
                     it.\n  Next step: `hick sealed --set --broker \
                     http://broker.lan:7749`."
                .into(),
        },
    });

    // Every provider base URL must resolve to the broker, and a client
    // configured otherwise refuses to start rather than reaching the vendor
    // directly.
    if let Some(url) = broker.as_deref().filter(|u| !u.trim().is_empty()) {
        let wrong: Vec<String> = BASE_URL_VARS
            .iter()
            .filter_map(|var| {
                let value = env.get(*var)?;
                (!value.starts_with(url)).then(|| format!("{var}={value}"))
            })
            .collect();
        findings.push(if wrong.is_empty() {
            SealFinding {
                check: "base URLs point at the broker".into(),
                ok: true,
                detail: "every provider base URL that is set points at the broker.".into(),
            }
        } else {
            SealFinding {
                check: "base URLs point at the broker".into(),
                ok: false,
                detail: format!(
                    "{} does not point at {url}, so that client would reach the \
                     vendor directly and go around the toll booth entirely.\n  \
                     Next step: set it to {url}, or unset it.",
                    wrong.join(", ")
                ),
            }
        });
    }

    findings.push(SealFinding {
        check: "the default route is denied".into(),
        ok: route_denied.unwrap_or(false),
        detail: match route_denied {
            Some(true) => "the default route is denied, so a process that ignores \
                           the proxy settings gets nothing rather than getting out."
                .into(),
            Some(false) => "the default route is NOT denied, so a process that \
                            ignores the proxy settings reaches the internet \
                            directly — the road out is not the only road.\n  \
                            This is firewall configuration on your own machine, \
                            and this command verifies it rather than performing \
                            it: we do not reconfigure somebody's networking for \
                            them."
                .into(),
            None => "could not determine whether the default route is denied. \
                     That is firewall configuration on your own machine, and \
                     this command verifies rather than performs — so an answer \
                     it cannot get is reported, never assumed."
                .into(),
        },
    });

    SealCheck {
        sealed: true,
        broker,
        findings,
    }
}

</storycode:copy>
<a id="f-c24a2c2f2c9d-tests"></a>
### `tests`

<storycode:copy id="f-c24a2c2f2c9d-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn sealed() -> SealConfig {
        SealConfig {
            sealed: true,
            broker: Some("http://broker.lan:7749".into()),
            stub: Some("stub-not-a-vendor-credential".into()),
        }
    }

    #[test]
    fn an_unsealed_machine_says_so_and_says_who_may_seal_it() {
        let check = check_seal(&SealConfig::default(), &env(&[]), false, Some(true));
        assert!(!check.sealed);
        assert!(!check.ok());
        let detail = &check.findings[0].detail;
        // Not a grant another machine can hand out: a fleet peer that could
        // unseal this box would mean a compromised laptop unseals it.
        assert!(detail.contains("standing at a machine"), "{detail}");
    }

    #[test]
    fn a_provider_variable_is_a_failure_and_not_a_fallback() {
        let check = check_seal(
            &sealed(),
            &env(&[("ANTHROPIC_API_KEY", "sk-real")]),
            false,
            Some(true),
        );
        assert!(!check.ok());
        let finding = check
            .findings
            .iter()
            .find(|f| f.check.contains("environment"))
            .unwrap();
        assert!(!finding.ok);
        assert!(finding.detail.contains("ANTHROPIC_API_KEY"), "{finding:?}");
        assert!(finding.detail.contains("Unset it"), "{finding:?}");
        // The message must not leak the value it found.
        assert!(!finding.detail.contains("sk-real"), "{finding:?}");
    }

    #[test]
    fn an_empty_provider_variable_is_not_a_key() {
        let check = check_seal(
            &sealed(),
            &env(&[("OPENAI_API_KEY", "")]),
            false,
            Some(true),
        );
        assert!(check.ok(), "{check:?}");
    }

    #[test]
    fn a_base_url_pointing_past_the_broker_is_caught() {
        // Otherwise that client goes around the toll booth entirely.
        let check = check_seal(
            &sealed(),
            &env(&[("ANTHROPIC_BASE_URL", "https://api.anthropic.com")]),
            false,
            Some(true),
        );
        assert!(!check.ok());
        let finding = check
            .findings
            .iter()
            .find(|f| f.check.contains("base URLs"))
            .unwrap();
        assert!(
            finding.detail.contains("around the toll booth"),
            "{finding:?}"
        );
    }

    #[test]
    fn a_sealed_machine_with_no_broker_is_a_coherent_state_that_is_named() {
        // On its own the seal is a machine that cannot use a model at all,
        // which is exactly what step 1 buys.
        let config = SealConfig {
            sealed: true,
            broker: None,
            stub: None,
        };
        let check = check_seal(&config, &env(&[]), false, Some(true));
        let finding = check
            .findings
            .iter()
            .find(|f| f.check.contains("broker"))
            .unwrap();
        assert!(!finding.ok);
        assert!(finding.detail.contains("coherent state"), "{finding:?}");
    }

    #[test]
    fn an_undetermined_route_is_reported_and_never_assumed() {
        let check = check_seal(&sealed(), &env(&[]), false, None);
        let finding = check
            .findings
            .iter()
            .find(|f| f.check.contains("default route"))
            .unwrap();
        assert!(!finding.ok);
        assert!(
            finding.detail.contains("verifies rather than performs"),
            "{finding:?}"
        );
    }

    #[test]
    fn a_fully_sealed_machine_passes() {
        let check = check_seal(
            &sealed(),
            &env(&[("ANTHROPIC_BASE_URL", "http://broker.lan:7749/v1")]),
            false,
            Some(true),
        );
        assert!(check.ok(), "{check:?}");
    }
}
</storycode:copy>

### Assemble `crates/hickory-broker/src/seal.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-broker/src/seal.rs"><storycode:paste select="#f-c24a2c2f2c9d-PROVIDER_VARS"/><storycode:paste select="#f-c24a2c2f2c9d-BASE_URL_VARS"/><storycode:paste select="#f-c24a2c2f2c9d-SealFinding"/><storycode:paste select="#f-c24a2c2f2c9d-SealCheck"/><storycode:paste select="#f-c24a2c2f2c9d-impl-SealCheck"/><storycode:paste select="#f-c24a2c2f2c9d-SealConfig"/><storycode:paste select="#f-c24a2c2f2c9d-impl-SealConfig"/><storycode:paste select="#f-c24a2c2f2c9d-check_seal"/><storycode:paste select="#f-c24a2c2f2c9d-tests"/></storycode:file>

<a id="output-8fed533d60aa"></a>
## `crates/hickory-broker/tests/proxy_e2e.rs`

Related contracts: [One Road Out, With A Toll Booth On It — Never An Airgap](contract.md).

<a id="f-8fed533d60aa-echo_server"></a>
### `echo_server`

The broker over a real socket: allow forwards, deny refuses with a
sentence, and nothing else is a road.
Protects `docs/guarantees/collaboration/one-road-out-with-a-toll-booth.md`.

<storycode:copy id="f-8fed533d60aa-echo_server">
//! The broker over a real socket: allow forwards, deny refuses with a
//! sentence, and nothing else is a road.
//!
//! Protects `docs/guarantees/collaboration/one-road-out-with-a-toll-booth.md`.

use std::sync::Arc;

use hickory_broker::policy::Verb;
use hickory_broker::{Broker, BrokerLog, Policy};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};

/// An upstream that echoes whatever it is sent — the thing a CONNECT tunnel
/// is supposed to reach.
async fn echo_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                while let Ok(n) = sock.read(&mut buf).await {
                    if n == 0 || sock.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
            });
        }
    });
    port
}

</storycode:copy>
<a id="f-8fed533d60aa-start"></a>
### `start`

<storycode:copy id="f-8fed533d60aa-start">
async fn start(policy: Policy, dir: &std::path::Path) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let broker = Arc::new(Broker {
        policy,
        log: Arc::new(BrokerLog::at(&dir.join("broker.jsonl"))),
        now: Arc::new(|| "2026-08-23T00:00:00Z".to_string()),
    });
    tokio::spawn(async move {
        let _ = broker.serve(listener).await;
    });
    port
}

</storycode:copy>
<a id="f-8fed533d60aa-connect_through"></a>
### `connect_through`

<storycode:copy id="f-8fed533d60aa-connect_through">
async fn connect_through(port: u16, target: &str) -> (String, TcpStream) {
    let mut sock = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    sock.write_all(format!("CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        let n = sock.read(&mut byte).await.unwrap();
        if n == 0 {
            break;
        }
        head.push(byte[0]);
    }
    (String::from_utf8_lossy(&head).to_string(), sock)
}

</storycode:copy>
<a id="f-8fed533d60aa-an_allowed_host_is_forwarded_byte_for_byte"></a>
### `an_allowed_host_is_forwarded_byte_for_byte`

<storycode:copy id="f-8fed533d60aa-an_allowed_host_is_forwarded_byte_for_byte">
#[tokio::test]
async fn an_allowed_host_is_forwarded_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let upstream = echo_server().await;
    let mut policy = Policy::default();
    policy.set("127.0.0.1", Verb::Allow, false).unwrap();
    let port = start(policy, dir.path()).await;

    let (head, mut tunnel) = connect_through(port, &format!("127.0.0.1:{upstream}")).await;
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");

    tunnel.write_all(b"hello").await.unwrap();
    let mut buf = [0u8; 5];
    tunnel.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"hello");

    // The broker is OUTSIDE the connection: it logs a hostname and a byte
    // count, never a body.
    drop(tunnel);
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let entries = BrokerLog::at(&dir.path().join("broker.jsonl")).read();
    let allowed = entries.iter().find(|e| e.verb == Verb::Allow).unwrap();
    assert_eq!(allowed.host, "127.0.0.1");
    assert!(allowed.bytes.unwrap_or(0) >= 5);
    let raw = std::fs::read_to_string(dir.path().join("broker.jsonl")).unwrap();
    assert!(!raw.contains("hello"), "the log holds a body: {raw}");
}

</storycode:copy>
<a id="f-8fed533d60aa-an_unlisted_host_is_denied_with_a_sentence_the_agent_can_repeat"></a>
### `an_unlisted_host_is_denied_with_a_sentence_the_agent_can_repeat`

<storycode:copy id="f-8fed533d60aa-an_unlisted_host_is_denied_with_a_sentence_the_agent_can_repeat">
#[tokio::test]
async fn an_unlisted_host_is_denied_with_a_sentence_the_agent_can_repeat() {
    // An agent that receives a timeout invents a reason; an agent that
    // receives a sentence repeats it.
    let dir = tempfile::tempdir().unwrap();
    let port = start(Policy::default(), dir.path()).await;
    let (head, mut sock) = connect_through(port, "raw.githubusercontent.com:443").await;
    assert!(head.starts_with("HTTP/1.1 403"), "{head}");

    let mut body = String::new();
    sock.read_to_string(&mut body).await.unwrap();
    assert!(body.contains("raw.githubusercontent.com"), "{body}");
    assert!(
        body.contains("ask the engineer to allow that host"),
        "{body}"
    );
    assert!(body.contains("one road out"), "{body}");
    // Never "airgapped": a machine that talks to a model is on a network.
    assert!(!body.to_lowercase().contains("airgap"), "{body}");
}

</storycode:copy>
<a id="f-8fed533d60aa-ask_and_substitute_are_denied_rather_than_left_hanging"></a>
### `ask_and_substitute_are_denied_rather_than_left_hanging`

<storycode:copy id="f-8fed533d60aa-ask_and_substitute_are_denied_rather_than_left_hanging">
#[tokio::test]
async fn ask_and_substitute_are_denied_rather_than_left_hanging() {
    // `ask` needs the fleet channel and `substitute` needs a CA; neither is
    // built. An agent that hangs for an hour produces a half-finished turn
    // nobody can explain, so both refuse and say why.
    let dir = tempfile::tempdir().unwrap();
    let mut policy = Policy::default();
    policy.set("ask.example", Verb::Ask, false).unwrap();
    policy.set("sub.example", Verb::Substitute, true).unwrap();
    let port = start(policy, dir.path()).await;

    for (host, expected) in [
        ("ask.example", "ASK a human"),
        ("sub.example", "SUBSTITUTE"),
    ] {
        let (head, mut sock) = connect_through(port, &format!("{host}:443")).await;
        assert!(head.starts_with("HTTP/1.1 403"), "{host}: {head}");
        let mut body = String::new();
        sock.read_to_string(&mut body).await.unwrap();
        assert!(body.contains(expected), "{host}: {body}");
    }
    let entries = BrokerLog::at(&dir.path().join("broker.jsonl")).read();
    let sub = entries.iter().find(|e| e.verb == Verb::Substitute).unwrap();
    assert!(
        sub.reason
            .as_deref()
            .unwrap()
            .contains("no credential was used")
    );
}

</storycode:copy>
<a id="f-8fed533d60aa-anything_other_than_connect_is_refused"></a>
### `anything_other_than_connect_is_refused`

<storycode:copy id="f-8fed533d60aa-anything_other_than_connect_is_refused">
#[tokio::test]
async fn anything_other_than_connect_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let port = start(Policy::default(), dir.path()).await;
    let mut sock = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    sock.write_all(b"GET http://example.com/ HTTP/1.1\r\nHost: x\r\n\r\n")
        .await
        .unwrap();
    let mut answer = String::new();
    sock.read_to_string(&mut answer).await.unwrap();
    assert!(answer.starts_with("HTTP/1.1 405"), "{answer}");
    assert!(answer.contains("CONNECT only"), "{answer}");
}

</storycode:copy>
<a id="f-8fed533d60aa-an_allowed_host_that_cannot_be_reached_says_it_is_the_network"></a>
### `an_allowed_host_that_cannot_be_reached_says_it_is_the_network`

<storycode:copy id="f-8fed533d60aa-an_allowed_host_that_cannot_be_reached_says_it_is_the_network">
#[tokio::test]
async fn an_allowed_host_that_cannot_be_reached_says_it_is_the_network() {
    let dir = tempfile::tempdir().unwrap();
    let mut policy = Policy::default();
    policy.set("127.0.0.1", Verb::Allow, false).unwrap();
    let port = start(policy, dir.path()).await;
    // Port 1 on loopback: allowed by policy, nothing listening.
    let (head, mut sock) = connect_through(port, "127.0.0.1:1").await;
    assert!(head.starts_with("HTTP/1.1 502"), "{head}");
    let mut body = String::new();
    sock.read_to_string(&mut body).await.unwrap();
    assert!(
        body.contains("That is the network, not the policy"),
        "{body}"
    );
}
</storycode:copy>

### Assemble `crates/hickory-broker/tests/proxy_e2e.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-broker/tests/proxy_e2e.rs"><storycode:paste select="#f-8fed533d60aa-echo_server"/><storycode:paste select="#f-8fed533d60aa-start"/><storycode:paste select="#f-8fed533d60aa-connect_through"/><storycode:paste select="#f-8fed533d60aa-an_allowed_host_is_forwarded_byte_for_byte"/><storycode:paste select="#f-8fed533d60aa-an_unlisted_host_is_denied_with_a_sentence_the_agent_can_repeat"/><storycode:paste select="#f-8fed533d60aa-ask_and_substitute_are_denied_rather_than_left_hanging"/><storycode:paste select="#f-8fed533d60aa-anything_other_than_connect_is_refused"/><storycode:paste select="#f-8fed533d60aa-an_allowed_host_that_cannot_be_reached_says_it_is_the_network"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-broker/src/policy.rs](contract.md#output-ff3e7f8e3a1e)
- [crates/hickory-broker/src/proxy.rs](contract.md#output-24f287415c45)
- [crates/hickory-broker/src/seal.rs](contract.md#output-c24a2c2f2c9d)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)

### Tests and fixtures

- [crates/hickory-broker/tests/proxy_e2e.rs](contract.md#output-8fed533d60aa)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
