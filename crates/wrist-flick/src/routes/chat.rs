use std::convert::Infallible;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::StreamExt;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::routes::conversations::MessageResponse;
use crate::service::chat::ChatService;

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

pub async fn send_message(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let chat_service = ChatService::new(
        state.engine.clone(),
        state.chat_store.clone(),
        state.memory_store.clone(),
        state.llm.clone(),
        state.doc_id.clone(),
    );

    let event_stream = chat_service
        .stream_response(&conversation_id, &req.content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sse_stream = event_stream.map(|chat_event| {
        let event_name = chat_event.event_name().to_string();
        let data = serde_json::to_string(&chat_event).unwrap_or_default();
        Ok(Event::default().event(event_name).data(data))
    });

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}

pub async fn list_messages(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let messages = state
        .chat_store
        .list_messages(&conversation_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        messages
            .into_iter()
            .map(|m| MessageResponse {
                id: m.id,
                role: m.role,
                content: m.content,
                created_at: m.created_at,
            })
            .collect(),
    ))
}
