//! Hickory Docs server (axum): REST JSON API, realtime WS (Yjs sync + run
//! events), auth, execution, billing. The API contract is
//! `docs/specs/freeform/api.md` — change it there first.

pub mod analytics;
pub mod auth;
pub mod byok;
pub mod config;
pub mod doc_store;
pub mod email_tokens;
pub mod error;
pub mod executor;
pub mod gitstore;
pub mod keyvault;
pub mod lsp;
pub mod mail;
pub mod openapi;
pub mod output_rooms;
pub mod plans;
pub mod render_cache;
pub mod routes;
pub mod runs;
pub mod ws;

use std::sync::Arc;

use anyhow::{Context as _, Result};
use axum::Router;
use axum::routing::{get, post};
use sqlx::postgres::PgPoolOptions;

pub use config::{AgentLlmConfig, Config, parse_allowlist};
pub use routes::auth::signup_allowed;

/// WS channel prefixes (api.md), defined once in the collaboration crate so
/// the hosted server and `hickory serve` cannot disagree about the framing.
pub use hickory_collab::{CHANNEL_LSP, CHANNEL_RUN, CHANNEL_YJS};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: sqlx::PgPool,
    pub git: gitstore::GitStore,
    pub catalog: Arc<plans::Catalog>,
    pub analytics: analytics::Analytics,
    pub rooms: Arc<hickory_collab::RoomRegistry>,
    pub output_rooms: Arc<output_rooms::OutputRoomRegistry>,
    pub editors: Arc<ws::EditorTracker>,
    pub http: reqwest::Client,
    /// Bounded in-process cache of woven block models (see `render_cache`).
    pub renders: Arc<render_cache::RenderCache>,
    /// Transactional email. A `NullMailer` when unconfigured, which reports
    /// `is_configured() == false` so verification is not *required* on a
    /// deployment that cannot send it.
    pub mailer: Arc<dyn mail::Mailer>,
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
    // Rooms persist through this: the docs row for the fast path, the project
    // git repo for the durable one.
    let doc_store = doc_store::PostgresDocStore::new(db.clone(), git.clone());
    let catalog = Arc::new(plans::Catalog::load().context("parsing embedded plans.json")?);
    let analytics = analytics::Analytics::new(config.posthog.clone());
    let mailer: Arc<dyn mail::Mailer> = match &config.sendgrid {
        Some(sg) => Arc::new(mail::SendGridMailer::new(
            sg.api_key.clone(),
            sg.from_email.clone(),
            sg.from_name.clone(),
        )),
        None => Arc::new(mail::NullMailer),
    };

    Ok(AppState {
        mailer,
        config: Arc::new(config),
        db,
        git,
        catalog,
        analytics,
        rooms: Arc::new(hickory_collab::RoomRegistry::new(doc_store)),
        output_rooms: Arc::new(output_rooms::OutputRoomRegistry::default()),
        editors: Arc::new(ws::EditorTracker::default()),
        http: reqwest::Client::new(),
        renders: Arc::new(render_cache::RenderCache::default()),
    })
}

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/auth/signup", post(routes::auth::signup))
        .route("/auth/login", post(routes::auth::login))
        .route("/me", get(routes::auth::me))
        .route(
            "/me/llm-keys",
            get(routes::llm_keys::list_llm_keys).put(routes::llm_keys::select_llm_key),
        )
        .route(
            "/me/llm-keys/{provider}",
            axum::routing::put(routes::llm_keys::save_llm_key)
                .delete(routes::llm_keys::delete_llm_key),
        )
        .route("/auth/verify/send", post(routes::auth::send_verification))
        .route(
            "/auth/verify/confirm",
            post(routes::auth::confirm_verification),
        )
        .route("/auth/reset/request", post(routes::auth::request_reset))
        .route("/auth/reset/confirm", post(routes::auth::confirm_reset))
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
        .route("/docs/{id}/outputs/nav", post(routes::outputs::outputs_nav))
        .route("/docs/{id}/run", post(routes::runs::run_doc))
        .route("/docs/{id}/check", post(routes::runs::check_doc))
        .route("/docs/{id}/agent", post(routes::agent::start_agent))
        .route("/docs/{id}/agent/turns", get(routes::agent::list_turns))
        .route("/runs/{id}", get(routes::runs::get_run))
        .route("/billing/plans", get(routes::billing::get_plans))
        .route("/billing/checkout", post(routes::billing::checkout))
        .route("/billing/webhook", post(routes::billing::webhook))
        .route("/analytics/capture", post(routes::analytics::capture))
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
