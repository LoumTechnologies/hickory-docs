//! The relay process.
//!
//! Configuration is read once, into a typed struct, and the process refuses to
//! start on anything it cannot make sense of — the same rule the server
//! follows (`.instructions/config-and-environments.md`). There is deliberately
//! very little of it: a relay that needed a database URL would not be a relay.

use std::sync::Arc;

use anyhow::{Context as _, Result};
use hickory_relay::{Quota, TunnelCensus};
use hickory_relay_server::github::GitHubIdentifier;
use hickory_relay_server::tunnel::TunnelRegistry;
use hickory_relay_server::{RelayState, router};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // The apex every session hangs off. Required: a relay that guessed its own
    // public name would hand out URLs that do not resolve.
    let apex = std::env::var("RELAY_APEX").context(
        "RELAY_APEX is required (e.g. relay.hickorydocs.com) — it is the domain \
                  sessions are addressed under, and cannot be guessed",
    )?;
    let scheme = std::env::var("RELAY_SCHEME").unwrap_or_else(|_| "https".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .map(|p| p.parse())
        .transpose()
        .context("invalid PORT")?
        .unwrap_or(8080);
    let github_api =
        std::env::var("GITHUB_API_BASE").unwrap_or_else(|_| "https://api.github.com".to_string());

    // Signs every token this relay issues. Required: a random default would
    // silently invalidate everyone's sign-in on each restart, which looks like
    // a bug in the CLI rather than a missing variable here.
    let token_secret = std::env::var("RELAY_TOKEN_SECRET").context(
        "RELAY_TOKEN_SECRET is required — it signs the tokens this relay issues, and a \
         value that changed between restarts would sign everyone out",
    )?;
    if token_secret.len() < 32 {
        anyhow::bail!("RELAY_TOKEN_SECRET must be at least 32 bytes");
    }

    // Where accounts live. Absent, the relay still forwards for anyone holding
    // a token it signed earlier, but nobody new can sign in — a degraded state
    // that is announced rather than hidden.
    let accounts = match std::env::var("RELAY_DATABASE_URL") {
        Ok(url) if !url.trim().is_empty() => Some(
            hickory_relay_server::accounts::open(&url)
                .await
                .context("opening the relay's account store")?,
        ),
        _ => {
            log::warn!(
                "RELAY_DATABASE_URL unset; sign-in is unavailable (set it to e.g. \
                 sqlite:///data/relay.db)"
            );
            None
        }
    };

    // GitHub sign-in exists only if an OAuth app was configured. The CLI asks
    // and hides the option when this is absent.
    let github_client_id = std::env::var("GH_OAUTH_CLIENT_ID")
        .ok()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty());
    match &github_client_id {
        Some(id) => log::info!("GitHub sign-in enabled (client id {id})"),
        None => log::info!("GitHub sign-in disabled (GH_OAUTH_CLIENT_ID unset)"),
    }

    let state = RelayState {
        tunnels: Arc::new(TunnelRegistry::default()),
        census: Arc::new(Mutex::new(TunnelCensus::default())),
        quota: Quota::default(),
        identifier: GitHubIdentifier::new(github_api),
        accounts,
        token_secret,
        github_client_id,
        apex: apex.clone(),
        scheme,
    };

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    log::info!("relay listening on {} for *.{apex}", listener.local_addr()?);
    axum::serve(listener, router(state)).await?;
    Ok(())
}
