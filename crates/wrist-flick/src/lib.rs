pub mod app_state;
pub mod config;
pub mod llm;
pub mod routes;
pub mod sse;

use std::sync::Arc;

use tokio::sync::Mutex;

use app_state::AppState;
use drift_signal_core::{AppDriftSignal, DriftSignalConfig};
use llm::LlmClient;

/// Build the full AppState backed by drift-signal-core.
pub async fn build_app_state(
    llm: Arc<dyn LlmClient>,
    db_path: Option<&str>,
) -> anyhow::Result<AppState> {
    let config = DriftSignalConfig {
        user_id: "wrist-flick-user".into(),
        device_id: format!("device-{}", uuid::Uuid::now_v7()),
        db_path: db_path.unwrap_or("wrist-flick.db").to_string(),
    };

    let core = AppDriftSignal::open(config).await?;

    Ok(AppState {
        core: Arc::new(Mutex::new(core)),
        llm,
    })
}

/// Build the Axum router with the given state.
pub fn build_router(state: AppState) -> axum::Router {
    use tower_http::cors::{Any, CorsLayer};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    routes::router().with_state(state).layer(cors)
}
