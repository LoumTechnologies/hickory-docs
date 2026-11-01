//! hick-billing: token usage accounting and Stripe metered billing.
//!
//! Environment variables (all required unless marked optional):
//!   GITHUB_CLIENT_ID       — GitHub OAuth app client ID
//!   GITHUB_CLIENT_SECRET   — GitHub OAuth app client secret
//!   DATABASE_URL           — PostgreSQL connection string
//!   JWT_SECRET             — secret for signing auth JWTs (≥ 32 bytes)
//!   STRIPE_SECRET_KEY      — Stripe secret key (optional; disables Stripe if absent)
//!   BILLING_BIND           — listen address (default 0.0.0.0:3001)

mod api;
mod auth;
mod billing;
mod db;
mod error;
mod jwt;

use std::io::Write as _;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use clap::Parser;
use sqlx::PgPool;

#[derive(Parser)]
#[command(about = "hick-billing: token usage telemetry and Stripe metered billing")]
struct Cli {
    #[arg(long, env = "BILLING_BIND", default_value = "0.0.0.0:3001")]
    bind: SocketAddr,

    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(long, env = "GITHUB_CLIENT_ID")]
    github_client_id: String,

    #[arg(long, env = "GITHUB_CLIENT_SECRET")]
    github_client_secret: String,

    #[arg(long, env = "JWT_SECRET")]
    jwt_secret: String,

    #[arg(long, env = "STRIPE_SECRET_KEY")]
    stripe_secret_key: Option<String>,
}

/// Shared application state injected into all handlers.
pub struct AppState {
    pub db: PgPool,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub jwt_secret: String,
    pub stripe_secret_key: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "<log timestamp=\"{}\" level=\"{}\" target=\"{}\">{}</log>",
                chrono::Utc::now().to_rfc3339(),
                record.level(),
                record.target(),
                record.args(),
            )
        })
        .init();

    let cli = Cli::parse();

    let pool = PgPool::connect(&cli.database_url).await?;
    db::run_migrations(&pool).await?;

    let state = Arc::new(AppState {
        db: pool,
        github_client_id: cli.github_client_id,
        github_client_secret: cli.github_client_secret,
        jwt_secret: cli.jwt_secret,
        stripe_secret_key: cli.stripe_secret_key,
    });

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/github", get(auth::login))
        .route("/auth/github/callback", get(auth::callback))
        .route("/api/usage", post(api::ingest_usage))
        .route("/api/balance", get(api::get_balance))
        .with_state(state);

    log::info!("hick-billing listening on {}", cli.bind);
    let listener = tokio::net::TcpListener::bind(cli.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
