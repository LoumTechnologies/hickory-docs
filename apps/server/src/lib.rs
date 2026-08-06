//! Hickory Docs server (axum): REST JSON API, realtime WS (Yjs sync + run
//! events), auth, execution, billing. The API contract is
//! `docs/specs/freeform/api.md` — change it there first.

pub mod analytics;
pub mod auth;
pub mod config;
pub mod error;
pub mod executor;
pub mod gitstore;
pub mod plans;
pub mod routes;
pub mod runs;
pub mod ws;

use std::sync::Arc;

use anyhow::{Context as _, Result};
use axum::Router;
use axum::routing::{get, post};
use sqlx::postgres::PgPoolOptions;

pub use config::Config;

/// WS channel prefixes (api.md).
pub const CHANNEL_YJS: u8 = 0x00;
pub const CHANNEL_RUN: u8 = 0x01;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: sqlx::PgPool,
    pub git: gitstore::GitStore,
    pub catalog: Arc<plans::Catalog>,
    pub analytics: analytics::Analytics,
    pub rooms: Arc<ws::RoomRegistry>,
    pub editors: Arc<ws::EditorTracker>,
    pub http: reqwest::Client,
}

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Connect to Postgres and run migrations.
pub async fn init_db(database_url: &str) -> Result<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("connecting to Postgres (DATABASE_URL)")?;
    MIGRATOR.run(&pool).await.context("running migrations")?;
    Ok(pool)
}

pub fn build_state(mut config: Config, db: sqlx::PgPool) -> Result<AppState> {
    executor::validate_executor(&mut config)?;
    let git = gitstore::GitStore::new(&config.git_data_dir)?;
    let catalog = Arc::new(plans::Catalog::load().context("parsing embedded plans.json")?);
    let analytics = analytics::Analytics::new(config.posthog.clone());
    Ok(AppState {
        config: Arc::new(config),
        db,
        git,
        catalog,
        analytics,
        rooms: Arc::new(ws::RoomRegistry::default()),
        editors: Arc::new(ws::EditorTracker::default()),
        http: reqwest::Client::new(),
    })
}

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/signup", post(routes::auth::signup))
        .route("/auth/login", post(routes::auth::login))
        .route("/me", get(routes::auth::me))
        .route(
            "/projects",
            get(routes::projects::list_projects).post(routes::projects::create_project),
        )
        .route(
            "/projects/{id}/docs",
            get(routes::projects::list_docs).post(routes::projects::create_doc),
        )
        .route(
            "/docs/{id}",
            get(routes::docs::get_doc).put(routes::docs::put_doc),
        )
        .route("/docs/{id}/render", get(routes::docs::render_doc))
        .route("/docs/{id}/outputs", get(routes::outputs::list_outputs))
        .route(
            "/docs/{id}/outputs/file",
            get(routes::outputs::get_output_file),
        )
        .route(
            "/docs/{id}/outputs/edit",
            post(routes::outputs::edit_outputs),
        )
        .route("/docs/{id}/run", post(routes::runs::run_doc))
        .route("/docs/{id}/check", post(routes::runs::check_doc))
        .route("/docs/{id}/agent", post(routes::agent::start_agent))
        .route("/runs/{id}", get(routes::runs::get_run))
        .route("/billing/plans", get(routes::billing::get_plans))
        .route("/billing/checkout", post(routes::billing::checkout))
        .route("/billing/webhook", post(routes::billing::webhook))
        .route("/ws", get(ws::ws_handler))
        .route("/health", get(routes::health::health))
        .route("/executor", get(routes::health::executor));

    let mut router = Router::new().nest("/api", api);

    // Static web app fallback (deploy image bundles apps/web/dist).
    if let Some(dist) = &state.config.web_dist_dir {
        let index = dist.join("index.html");
        let serve = tower_http::services::ServeDir::new(dist)
            .fallback(tower_http::services::ServeFile::new(index));
        router = router.fallback_service(serve);
        log::info!("serving web app from {}", dist.display());
    }

    router.with_state(state)
}
