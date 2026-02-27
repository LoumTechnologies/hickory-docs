pub mod capabilities;
pub mod chat;
pub mod conversations;
pub mod health;
pub mod memories;
pub mod sync;
pub mod tasks;

use axum::Router;
use axum::routing::{delete, get, patch, post};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Conversations
        .route("/api/v1/conversations", post(conversations::create))
        .route("/api/v1/conversations", get(conversations::list))
        .route("/api/v1/conversations/{id}", get(conversations::get_one))
        .route(
            "/api/v1/conversations/{id}",
            delete(conversations::delete_one),
        )
        // Chat (messages)
        .route(
            "/api/v1/conversations/{id}/messages",
            post(chat::send_message),
        )
        .route(
            "/api/v1/conversations/{id}/messages",
            get(chat::list_messages),
        )
        // Memories
        .route("/api/v1/memories", get(memories::list))
        .route("/api/v1/memories", post(memories::create))
        .route("/api/v1/memories/{id}", delete(memories::delete_one))
        .route("/api/v1/memories/search", get(memories::search))
        // Tasks
        .route("/api/v1/tasks", get(tasks::list))
        .route("/api/v1/tasks", post(tasks::create))
        .route("/api/v1/tasks/{id}", patch(tasks::update))
        .route("/api/v1/tasks/{id}", delete(tasks::delete_one))
        // Capabilities
        .route("/api/v1/capabilities", get(capabilities::list_registered))
        .route(
            "/api/v1/capabilities/find/{cap_type}",
            get(capabilities::find_providers),
        )
        .route(
            "/api/v1/capabilities/invoke",
            post(capabilities::invoke),
        )
        .route(
            "/api/v1/capabilities/invocations/{id}",
            get(capabilities::get_invocation),
        )
        // Sync
        .route("/api/v1/sync/status", get(sync::status))
        .route("/api/v1/sync/ws", get(sync::ws_handler))
        .route("/api/v1/sync/state-vector", get(sync::state_vector))
        // Health
        .route("/health", get(health::health))
}
