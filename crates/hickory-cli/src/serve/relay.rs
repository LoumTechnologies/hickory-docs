//! Reaching a session that lives on a laptop.
//!
//! `--share` binds the session to the local network, which covers two people
//! in one room and nothing else. A collaborator somewhere else needs a public
//! address, and a laptop behind NAT does not have one — so something has to
//! forward. That something is a **relay**, and this module is the seam where
//! one is chosen.
//!
//! Two providers, deliberately in this order:
//!
//! 1. **A URL the host already arranged** (`HICKORY_PUBLIC_URL`). Cloudflare
//!    Tunnel, ngrok, a Tailscale funnel, an SSH reverse tunnel, a reverse proxy
//!    someone already runs — all of them end in "here is a public URL that
//!    reaches this port". Supporting that first means the feature works today,
//!    for everyone, with no dependency to adopt.
//! 2. **PortZero** (`PZ_TUNNEL`), which the architecture already leans on to
//!    reach a non-cloud machine. Its daemon reads the variable from the
//!    process environment at exec time, so the session must be *launched* with
//!    it set — this module tells the host exactly how rather than silently
//!    doing nothing.
//!
//! What this module will not do is pretend. A `.portzero.local` domain is
//! reachable by machines running PortZero on the same overlay, not by the
//! internet, and saying otherwise would send someone a link that cannot open.
//! The distinction is surfaced, not smoothed over.

use anyhow::Result;

/// How (and whether) this session is reachable from outside its own network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Relay {
    /// Not requested: the session is loopback- or LAN-only.
    None,
    /// Our own relay: a tunnel is opened once the listener exists, and the
    /// address comes back from it. Requires `hickory login`.
    Hickory { base: String, token: String },
    /// A public base URL the host arranged themselves.
    Explicit(String),
    /// A PortZero tunnel this process was launched under.
    PortZero {
        domain: String,
        url: String,
        /// False for a `.local` overlay domain, which reaches machines running
        /// PortZero rather than the internet.
        public: bool,
    },
}

impl Relay {
    /// The base URL to build a share link from, if there is one.
    pub fn base_url(&self) -> Option<&str> {
        match self {
            Relay::None => None,
            // Not known until the tunnel is open; `serve` fills the address in
            // from the relay's answer.
            Relay::Hickory { .. } => None,
            Relay::Explicit(url) => Some(url),
            Relay::PortZero { url, .. } => Some(url),
        }
    }

    /// Whether a link built from this reaches beyond the host's own network.
    pub fn is_public(&self) -> bool {
        match self {
            Relay::None => false,
            Relay::Hickory { .. } => true,
            Relay::Explicit(_) => true,
            Relay::PortZero { public, .. } => *public,
        }
    }
}

/// Everything [`discover`] needs, read from the environment by the caller so
/// this stays a pure decision that can be tested.
pub struct RelayEnv {
    /// `--public` was asked for.
    pub requested: bool,
    /// `--share`: the session is bound beyond loopback.
    pub shared: bool,
    /// `HICKORY_PUBLIC_URL`.
    pub explicit_url: Option<String>,
    /// `PZ_TUNNEL`, the domain PortZero's daemon will publish this process as.
    pub pz_tunnel: Option<String>,
    /// The relay's base URL (`HICKORY_RELAY_URL`, else the default).
    pub relay_base: String,
    /// A stored GitHub token, when the host has run `hickory login`.
    pub hickory_token: Option<String>,
    /// Resolve a PortZero domain to a URL (`portzero url <domain>`). Injected
    /// so a test does not need a daemon.
    pub resolve_pz: fn(&str) -> Option<String>,
}

/// Decide how this session is reachable.
pub fn discover(env: &RelayEnv) -> Result<Relay> {
    if !env.requested {
        return Ok(Relay::None);
    }
    // A public address for a loopback-only session would be a URL that
    // resolves to a socket nothing outside this machine can reach.
    if !env.shared {
        anyhow::bail!(
            "--public needs --share: a public address is only meaningful for a session \
             that is listening beyond loopback.\n\
             Try: hickory serve --share --public …"
        );
    }

    if let Some(url) = env
        .explicit_url
        .as_deref()
        .map(str::trim)
        .filter(|u| !u.is_empty())
    {
        return Ok(Relay::Explicit(url.trim_end_matches('/').to_string()));
    }

    // Our own relay, when the host is signed in. Second rather than first:
    // someone who set HICKORY_PUBLIC_URL has told us which tunnel they want,
    // and overriding that would be presumptuous.
    if let Some(token) = env
        .hickory_token
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        return Ok(Relay::Hickory {
            base: env.relay_base.trim_end_matches('/').to_string(),
            token: token.to_string(),
        });
    }

    if let Some(domain) = env
        .pz_tunnel
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
    {
        let Some(url) = (env.resolve_pz)(domain) else {
            anyhow::bail!(
                "PZ_TUNNEL is set to {domain:?} but PortZero could not resolve it yet.\n\
                 Check `portzero status` — the daemon has to be running, and a tunnel that \
                 reaches the internet needs `portzero login`."
            );
        };
        let public = !url.contains(".portzero.local");
        return Ok(Relay::PortZero {
            domain: domain.to_string(),
            url: url.trim_end_matches('/').to_string(),
            public,
        });
    }

    anyhow::bail!(
        "--public needs something to forward through, and none was found.\n\
         \n\
         The simplest route is our relay, which needs a GitHub account so a tunnel \
         pointed at the internet is attributable to somebody:\n  \
           hickory login\n\
         \n\
         Or point at a tunnel you already run:\n  \
           HICKORY_PUBLIC_URL=https://your-tunnel.example hickory serve --share --public …\n\
         (Cloudflare Tunnel, ngrok, a Tailscale funnel, or any reverse proxy pointed at \
         this port.)\n\
         \n\
         Or launch the session under PortZero, whose daemon reads the variable at start:\n  \
           PZ_TUNNEL=hickory hickory serve --share --public …\n\
         (`portzero login` first — a tunnel reaching the internet is a cloud tunnel.)"
    );
}

/// Where the relay lives unless told otherwise. Overridable so a test — or a
/// self-hoster — can point at their own.
pub const DEFAULT_RELAY_URL: &str = "https://relay.hickorydocs.com";

/// Ask the `portzero` CLI to resolve a domain. `None` when the binary is
/// missing, the daemon is down, or the domain is not published yet.
pub fn resolve_portzero(domain: &str) -> Option<String> {
    let out = std::process::Command::new("portzero")
        .arg("url")
        .arg(domain)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let url = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!url.is_empty()).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(requested: bool, shared: bool) -> RelayEnv {
        RelayEnv {
            requested,
            shared,
            explicit_url: None,
            pz_tunnel: None,
            relay_base: DEFAULT_RELAY_URL.to_string(),
            hickory_token: None,
            resolve_pz: |_| None,
        }
    }

    #[test]
    fn not_asked_for_is_not_an_error() {
        assert_eq!(discover(&env(false, false)).unwrap(), Relay::None);
        // …even alongside a URL that would have worked.
        let mut e = env(false, true);
        e.explicit_url = Some("https://x.example".into());
        assert_eq!(discover(&e).unwrap(), Relay::None);
    }

    #[test]
    fn a_public_address_for_a_loopback_session_is_refused() {
        let err = discover(&env(true, false)).unwrap_err().to_string();
        assert!(err.contains("--public needs --share"), "{err}");
    }

    #[test]
    fn an_arranged_tunnel_wins_and_is_normalised() {
        let mut e = env(true, true);
        e.explicit_url = Some("  https://team.example/  ".into());
        e.pz_tunnel = Some("hickory".into());
        assert_eq!(
            discover(&e).unwrap(),
            Relay::Explicit("https://team.example".into())
        );
    }

    #[test]
    fn a_portzero_cloud_tunnel_is_public_and_a_local_one_is_not() {
        let mut e = env(true, true);
        e.pz_tunnel = Some("hickory".into());
        e.resolve_pz = |_| Some("https://hickory.alice.tunnel.portzero.cloud".into());
        let relay = discover(&e).unwrap();
        assert!(relay.is_public());
        assert_eq!(
            relay.base_url(),
            Some("https://hickory.alice.tunnel.portzero.cloud")
        );

        // The overlay domain reaches machines running PortZero, not the
        // internet — a link built from it must not be described as public.
        e.resolve_pz = |_| Some("http://hickory.portzero.local".into());
        let relay = discover(&e).unwrap();
        assert!(!relay.is_public());
    }

    #[test]
    fn an_unresolvable_tunnel_says_what_to_check() {
        let mut e = env(true, true);
        e.pz_tunnel = Some("hickory".into());
        let err = discover(&e).unwrap_err().to_string();
        assert!(err.contains("portzero status"), "{err}");
        assert!(err.contains("portzero login"), "{err}");
    }

    #[test]
    fn being_signed_in_is_enough_on_its_own() {
        let mut e = env(true, true);
        e.hickory_token = Some("gho_token".into());
        assert_eq!(
            discover(&e).unwrap(),
            Relay::Hickory {
                base: DEFAULT_RELAY_URL.to_string(),
                token: "gho_token".into()
            }
        );

        // …but a tunnel the host explicitly named still wins: they said which
        // one they wanted.
        e.explicit_url = Some("https://mine.example".into());
        assert_eq!(
            discover(&e).unwrap(),
            Relay::Explicit("https://mine.example".into())
        );
    }

    #[test]
    fn with_no_provider_the_error_teaches_both_paths() {
        let err = discover(&env(true, true)).unwrap_err().to_string();
        assert!(err.contains("HICKORY_PUBLIC_URL"), "{err}");
        assert!(err.contains("PZ_TUNNEL"), "{err}");
        // Naming alternatives matters: this is the one error where the fix is
        // "adopt some tunnel", and the host should not have to guess which.
        assert!(err.contains("ngrok") || err.contains("Cloudflare"), "{err}");
        // The route we actually want people to take is named first.
        assert!(err.contains("hickory login"), "{err}");
    }
}
