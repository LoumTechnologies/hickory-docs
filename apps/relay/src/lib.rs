//! The relay: the one cloud service local-first keeps.
//!
//! It forwards bytes between a guest's browser and a `hickory serve` session on
//! someone's laptop, and does nothing else — no execution, no storage, no
//! state that outlives a tunnel. See `docs/specs/freeform/relay.md`.
//!
//! ```text
//! guest ──HTTPS──► <slug>.relay.hickorydocs.com ──existing outbound WS──► laptop
//! ```
//!
//! Two entry points, distinguished by hostname:
//!
//! - `POST/GET /_relay/tunnel` on the apex — an agent opening a tunnel.
//! - anything on `<slug>.<apex>` — a guest, forwarded down that tunnel.
//!
//! The identity model is deliberately borrowed: a tunnel is opened with a
//! GitHub token, the relay asks GitHub who that is, and keeps the login for the
//! life of the connection. There is no user table here, and there must not be
//! one — the account exists so abuse is attributable and a quota is
//! enforceable, not so we can have users.

pub mod accounts;
pub mod auth_routes;
pub mod github;
pub mod tunnel;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State, WebSocketUpgrade};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use hickory_relay::{Quota, TunnelCensus};
use tokio::sync::Mutex;

use github::Identifier;
use tunnel::TunnelRegistry;

/// Everything the relay holds. Note what is absent: no database, no cache of
/// documents, nothing persisted.
#[derive(Clone)]
pub struct RelayState {
    pub tunnels: Arc<TunnelRegistry>,
    pub census: Arc<Mutex<TunnelCensus>>,
    pub quota: Quota,
    /// Resolves a GitHub token to a login, at sign-in only. Swapped for a
    /// stub in tests, which is the only reason it is a trait object.
    pub identifier: Arc<dyn Identifier>,
    /// The account store. `None` means this relay cannot sign anyone in, which
    /// is a configuration state rather than a crash.
    pub accounts: Option<sqlx::SqlitePool>,
    /// Signs the tokens this relay issues and accepts.
    pub token_secret: String,
    /// Set when a GitHub OAuth app is configured. Its absence is what makes
    /// the CLI hide the GitHub option rather than offer one that cannot work.
    pub github_client_id: Option<String>,
    /// The apex the relay answers on, e.g. `relay.hickorydocs.com`. A guest
    /// address is `<slug>.<apex>`.
    pub apex: String,
    /// `https` in production; `http` when running locally without TLS.
    pub scheme: String,
}

impl RelayState {
    /// The public address of a tunnel.
    pub fn url_for(&self, slug: &str) -> String {
        format!("{}://{slug}.{}", self.scheme, self.apex)
    }

    /// The slug a request is addressed to, if it is for a tunnel rather than
    /// for the relay itself.
    ///
    /// Matching on the *suffix* is what keeps this safe: a `Host` header of
    /// `evil.com` or `slug.relay.hickorydocs.com.evil.com` does not match, so a
    /// crafted header cannot make the relay serve a tunnel it did not mean to.
    pub fn slug_of<'h>(&self, host: &'h str) -> Option<&'h str> {
        // Compare hostnames, not authorities: the configured apex may carry a
        // port (it does whenever the relay runs without TLS, which is every
        // local run) and the Host header may or may not repeat it. Stripping
        // the port from exactly one side is how this silently matched nothing.
        let host = host.split(':').next()?;
        let apex = self.apex.split(':').next()?;
        let rest = host.strip_suffix(apex)?;
        let slug = rest.strip_suffix('.')?;
        (!slug.is_empty() && !slug.contains('.') && hickory_relay::slug_is_valid(slug))
            .then_some(slug)
    }
}

/// Build the relay's router.
pub fn router(state: RelayState) -> Router {
    Router::new()
        .route("/_relay/tunnel", get(open_tunnel))
        .route("/_relay/health", get(health))
        .route("/_relay/auth/methods", get(auth_routes::methods))
        .route(
            "/_relay/auth/signup",
            axum::routing::post(auth_routes::signup),
        )
        .route(
            "/_relay/auth/login",
            axum::routing::post(auth_routes::login),
        )
        .route(
            "/_relay/auth/github",
            axum::routing::post(auth_routes::github),
        )
        .fallback(forward)
        .with_state(state)
}

async fn health(State(state): State<RelayState>) -> Response {
    // Deliberately says how many tunnels are open and nothing about who owns
    // them: a health endpoint is public, and "which accounts are online" is
    // not the operator's to publish.
    axum::Json(serde_json::json!({
        "ok": true,
        "tunnels": state.tunnels.count().await,
    }))
    .into_response()
}

/// `GET /_relay/tunnel` — an agent opening a tunnel (a WebSocket upgrade).
async fn open_tunnel(State(state): State<RelayState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(move |socket| async move {
        if let Err(e) = tunnel::serve_agent(state, socket).await {
            log::debug!("tunnel ended: {e:#}");
        }
    })
}

/// Everything else: a guest, addressed to `<slug>.<apex>`.
async fn forward(State(state): State<RelayState>, req: Request<Body>) -> Response {
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let Some(slug) = state.slug_of(&host).map(str::to_string) else {
        // The apex itself is not a product surface: someone who lands here
        // followed a link to a session that is gone, or typed the host by
        // hand.
        return (
            StatusCode::NOT_FOUND,
            "This is the Hickory Docs relay. Sessions live at \
             <name>.<relay host> and are opened by `hickory serve --public` \
             on someone's machine.",
        )
            .into_response();
    };

    let Some(tunnel) = state.tunnels.get(&slug).await else {
        // The most common real failure: the host closed their laptop. Say so
        // in the words a guest would use, not "502".
        return (
            StatusCode::NOT_FOUND,
            format!(
                "No session is running at {slug}. The person who shared this link \
                 has ended it — sessions live only as long as `hickory serve` is running \
                 on their machine."
            ),
        )
            .into_response();
    };

    tunnel.forward(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> RelayState {
        RelayState {
            tunnels: Arc::new(TunnelRegistry::default()),
            census: Arc::new(Mutex::new(TunnelCensus::default())),
            quota: Quota::default(),
            identifier: Arc::new(github::StubIdentifier::new("nate")),
            accounts: None,
            token_secret: "a-test-secret-that-is-long-enough".into(),
            github_client_id: None,
            apex: "relay.hickorydocs.com".into(),
            scheme: "https".into(),
        }
    }

    #[test]
    fn a_guest_host_resolves_to_its_slug() {
        let s = state();
        // Ports appear on both sides in local runs, on neither in production,
        // and on one side or the other in between. All four must resolve.
        let local = RelayState {
            apex: "localhost:9000".into(),
            scheme: "http".into(),
            ..state()
        };
        assert_eq!(
            local.slug_of("quiet-cedar.localhost:9000"),
            Some("quiet-cedar")
        );
        assert_eq!(local.slug_of("quiet-cedar.localhost"), Some("quiet-cedar"));

        assert_eq!(
            s.slug_of("quiet-cedar.relay.hickorydocs.com"),
            Some("quiet-cedar")
        );
        // Ports are normal in local testing and must not defeat the match.
        assert_eq!(
            s.slug_of("quiet-cedar.relay.hickorydocs.com:8080"),
            Some("quiet-cedar")
        );
    }

    #[test]
    fn a_crafted_host_header_cannot_borrow_a_tunnel() {
        let s = state();
        // The apex itself is not a tunnel.
        assert_eq!(s.slug_of("relay.hickorydocs.com"), None);
        // A suffix that merely *contains* the apex is a different domain.
        assert_eq!(s.slug_of("x.relay.hickorydocs.com.evil.example"), None);
        assert_eq!(s.slug_of("evil.example"), None);
        // Nested labels are not slugs: `a.b.relay…` must not resolve to `a.b`.
        assert_eq!(s.slug_of("a.b.relay.hickorydocs.com"), None);
        // Anything a DNS label cannot hold is refused before it is looked up.
        assert_eq!(s.slug_of("UPPER.relay.hickorydocs.com"), None);
        assert_eq!(s.slug_of(".relay.hickorydocs.com"), None);
    }

    #[test]
    fn the_public_url_is_built_from_the_configured_scheme() {
        let mut s = state();
        assert_eq!(s.url_for("abc"), "https://abc.relay.hickorydocs.com");
        s.scheme = "http".into();
        s.apex = "localhost:9000".into();
        assert_eq!(s.url_for("abc"), "http://abc.localhost:9000");
    }
}
