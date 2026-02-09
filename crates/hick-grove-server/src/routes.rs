use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};

use hick_grove::GroveEngine;
use hick_grove::command::GroveCommand;
use hick_grove::namespace::{DocId, NamespaceUri};

pub type AppState = Arc<GroveEngine>;

#[derive(Deserialize)]
pub struct CreateDocRequest {
    pub namespaces: std::collections::HashMap<String, String>,
}

#[derive(Serialize)]
pub struct CreateDocResponse {
    pub id: String,
}

pub async fn create_document(
    State(engine): State<AppState>,
    Json(req): Json<CreateDocRequest>,
) -> Result<Json<CreateDocResponse>, StatusCode> {
    let namespaces: Vec<(String, NamespaceUri)> = req
        .namespaces
        .into_iter()
        .map(|(prefix, uri)| (prefix, NamespaceUri::new(uri)))
        .collect();

    let doc_id = engine.create_document(namespaces).await;

    Ok(Json(CreateDocResponse {
        id: doc_id.as_str().to_string(),
    }))
}

#[derive(Deserialize)]
pub struct MessageRequest {
    pub xml: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub events_dispatched: usize,
}

pub async fn process_message(
    State(engine): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MessageRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    let doc_id = DocId::from_string(id);
    let command = GroveCommand::StructuredMessage {
        doc_id,
        xml: req.xml,
    };

    let result = engine
        .process_command(command)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(MessageResponse {
        events_dispatched: result.events_dispatched,
    }))
}

#[derive(Serialize)]
pub struct StateVectorResponse {
    /// Base64-encoded state vector.
    pub state_vector: String,
}

pub async fn get_state_vector(
    State(engine): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StateVectorResponse>, StatusCode> {
    let doc_id = DocId::from_string(id);
    let sv = engine
        .state_vector(&doc_id)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&sv);

    Ok(Json(StateVectorResponse {
        state_vector: encoded,
    }))
}

#[derive(Deserialize)]
pub struct DiffRequest {
    /// Base64-encoded remote state vector.
    pub remote_state_vector: String,
}

#[derive(Serialize)]
pub struct DiffResponse {
    /// Base64-encoded diff.
    pub diff: String,
}

pub async fn get_diff(
    State(engine): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<DiffRequest>,
) -> Result<Json<DiffResponse>, StatusCode> {
    use base64::Engine as _;
    let doc_id = DocId::from_string(id);
    let remote_sv = base64::engine::general_purpose::STANDARD
        .decode(&req.remote_state_vector)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let diff = engine
        .encode_diff(&doc_id, &remote_sv)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let encoded = base64::engine::general_purpose::STANDARD.encode(&diff);

    Ok(Json(DiffResponse { diff: encoded }))
}

#[derive(Deserialize)]
pub struct BinaryUpdateRequest {
    /// Base64-encoded Yrs binary update.
    pub update: String,
}

pub async fn apply_update(
    State(engine): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<BinaryUpdateRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    use base64::Engine as _;
    let doc_id = DocId::from_string(id);
    let update = base64::engine::general_purpose::STANDARD
        .decode(&req.update)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let command = GroveCommand::BinaryUpdate { doc_id, update };

    let result = engine
        .process_command(command)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(MessageResponse {
        events_dispatched: result.events_dispatched,
    }))
}
