mod routes;
mod ws;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use hick_grove::plugin::NamespaceRegistry;
use hick_grove::plugins::tasks::{TasksHandler, TasksSqliteStore};
use hick_grove::{GroveConfig, GroveEngine};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let tasks_store = Arc::new(TasksSqliteStore::new_in_memory()?);
    let tasks_handler = Arc::new(TasksHandler::new(tasks_store.clone()));

    let mut registry = NamespaceRegistry::new();
    registry.register(tasks_handler);

    let engine = Arc::new(GroveEngine::new(GroveConfig { registry }));

    let app = Router::new()
        .route("/documents", post(routes::create_document))
        .route("/documents/{id}/messages", post(routes::process_message))
        .route("/documents/{id}/updates", post(routes::apply_update))
        .route(
            "/documents/{id}/state-vector",
            get(routes::get_state_vector),
        )
        .route("/documents/{id}/diff", post(routes::get_diff))
        .route("/documents/{id}/ws", get(ws::ws_handler))
        .with_state(engine);

    let addr = "0.0.0.0:3000";
    log::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
