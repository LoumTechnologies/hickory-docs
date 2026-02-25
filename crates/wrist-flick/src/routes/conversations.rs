use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};

use hick_grove::command::GroveCommand;

use crate::app_state::AppState;

#[derive(Serialize)]
pub struct ConversationResponse {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateConversationRequest {
    #[serde(default)]
    pub title: String,
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateConversationRequest>,
) -> Result<(StatusCode, Json<ConversationResponse>), StatusCode> {
    let id = format!("conv-{}", uuid::Uuid::now_v7());
    let title = if req.title.is_empty() {
        "New conversation".to_string()
    } else {
        req.title
    };

    let xml = format!(
        r#"<msg xmlns:chat="https://wrist-flick.dev/ns/chat#">
          <chat:conversation id="{}" title="{}"/>
        </msg>"#,
        id,
        escape_xml_attr(&title)
    );

    state
        .engine
        .process_command(GroveCommand::StructuredMessage {
            doc_id: state.doc_id.clone(),
            xml,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let conv = state
        .chat_store
        .get_conversation(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(ConversationResponse {
            id: conv.id,
            title: conv.title,
            created_at: conv.created_at,
            updated_at: conv.updated_at,
        }),
    ))
}

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConversationResponse>>, StatusCode> {
    let convs = state
        .chat_store
        .list_conversations(state.doc_id.as_str())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        convs
            .into_iter()
            .map(|c| ConversationResponse {
                id: c.id,
                title: c.title,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect(),
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ConversationDetailResponse>, StatusCode> {
    let conv = state
        .chat_store
        .get_conversation(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let messages = state
        .chat_store
        .list_messages(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ConversationDetailResponse {
        id: conv.id,
        title: conv.title,
        created_at: conv.created_at,
        updated_at: conv.updated_at,
        messages: messages
            .into_iter()
            .map(|m| MessageResponse {
                id: m.id,
                role: m.role,
                content: m.content,
                created_at: m.created_at,
            })
            .collect(),
    }))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state
        .chat_store
        .delete_conversation(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ConversationDetailResponse {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<MessageResponse>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
