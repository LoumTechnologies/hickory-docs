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

fn default_deny() -> Verb {
    Verb::Deny
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            hosts: BTreeMap::new(),
            default: Verb::Deny,
        }
    }
}

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

/// The broker's append-only log.
pub struct BrokerLog {
    path: PathBuf,
}

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
