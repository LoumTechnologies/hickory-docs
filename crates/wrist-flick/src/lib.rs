pub mod app_state;
pub mod config;
pub mod handlers;
pub mod llm;
pub mod routes;
pub mod service;
pub mod sse;

use std::sync::Arc;

use hick_grove::namespace::NamespaceUri;
use hick_grove::plugin::NamespaceRegistry;
use hick_grove::{GroveConfig, GroveEngine};

use app_state::AppState;
use handlers::chat_handler::{ChatHandler, ChatSqliteStore};
use handlers::memory_handler::{MemoryHandler, MemorySqliteStore};
use handlers::task_handler::{WfTaskHandler, WfTaskSqliteStore};
use llm::StreamingLlmClient;

/// Build the full AppState with all handlers registered.
pub async fn build_app_state(llm: Arc<dyn StreamingLlmClient>) -> anyhow::Result<AppState> {
    let chat_store = Arc::new(ChatSqliteStore::new_in_memory()?);
    let memory_store = Arc::new(MemorySqliteStore::new_in_memory()?);
    let task_store = Arc::new(WfTaskSqliteStore::new_in_memory()?);

    let chat_handler = Arc::new(ChatHandler::new(chat_store.clone()));
    let memory_handler = Arc::new(MemoryHandler::new(memory_store.clone()));
    let task_handler = Arc::new(WfTaskHandler::new(task_store.clone()));

    let mut registry = NamespaceRegistry::new();
    registry.register(chat_handler);
    registry.register(memory_handler);
    registry.register(task_handler);

    let engine = Arc::new(GroveEngine::new(GroveConfig { registry }));

    let doc_id = engine
        .create_document(vec![
            (
                "chat".into(),
                NamespaceUri::new("https://wrist-flick.dev/ns/chat#"),
            ),
            (
                "mem".into(),
                NamespaceUri::new("https://wrist-flick.dev/ns/memory#"),
            ),
            (
                "task".into(),
                NamespaceUri::new("https://wrist-flick.dev/ns/task#"),
            ),
        ])
        .await;

    Ok(AppState {
        engine,
        chat_store,
        memory_store,
        task_store,
        llm,
        doc_id,
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
