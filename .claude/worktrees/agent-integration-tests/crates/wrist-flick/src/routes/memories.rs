use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::service::memory::MemoryService;

#[derive(Serialize)]
pub struct MemoryResponse {
    pub id: String,
    pub key: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct CreateMemoryRequest {
    pub key: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub id: String,
    pub key: String,
    pub content: String,
    pub rank: f64,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    let svc = MemoryService::new(
        state.engine.clone(),
        state.memory_store.clone(),
        state.doc_id.clone(),
    );
    let memories = svc.list().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        memories
            .into_iter()
            .map(|m| MemoryResponse {
                id: m.id,
                key: m.key,
                content: m.content,
            })
            .collect(),
    ))
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateMemoryRequest>,
) -> Result<(StatusCode, Json<MemoryResponse>), StatusCode> {
    let svc = MemoryService::new(
        state.engine.clone(),
        state.memory_store.clone(),
        state.doc_id.clone(),
    );
    let id = svc
        .create(&req.key, &req.content, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((
        StatusCode::CREATED,
        Json(MemoryResponse {
            id,
            key: req.key,
            content: req.content,
        }),
    ))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let svc = MemoryService::new(
        state.engine.clone(),
        state.memory_store.clone(),
        state.doc_id.clone(),
    );
    svc.delete(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let svc = MemoryService::new(
        state.engine.clone(),
        state.memory_store.clone(),
        state.doc_id.clone(),
    );
    let results = svc
        .search(&query.q)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                key: r.key,
                content: r.content,
                rank: r.rank,
            })
            .collect(),
    ))
}
