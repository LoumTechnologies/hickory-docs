use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::service::task::TaskService;

#[derive(Serialize)]
pub struct TaskResponse {
    pub id: String,
    pub description: String,
    pub status: String,
    pub due_date: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub description: String,
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub description: Option<String>,
    pub status: Option<String>,
    pub due_date: Option<String>,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    let svc = TaskService::new(
        state.engine.clone(),
        state.task_store.clone(),
        state.doc_id.clone(),
    );
    let tasks = svc.list().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(
        tasks
            .into_iter()
            .map(|t| TaskResponse {
                id: t.id,
                description: t.description,
                status: t.status,
                due_date: t.due_date,
            })
            .collect(),
    ))
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), StatusCode> {
    let svc = TaskService::new(
        state.engine.clone(),
        state.task_store.clone(),
        state.doc_id.clone(),
    );
    let id = svc
        .create(&req.description)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((
        StatusCode::CREATED,
        Json(TaskResponse {
            id,
            description: req.description,
            status: "open".into(),
            due_date: None,
        }),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let svc = TaskService::new(
        state.engine.clone(),
        state.task_store.clone(),
        state.doc_id.clone(),
    );
    svc.update(
        &id,
        req.description.as_deref(),
        req.status.as_deref(),
        req.due_date.as_deref(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let task = svc
        .get(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(TaskResponse {
        id: task.id,
        description: task.description,
        status: task.status,
        due_date: task.due_date,
    }))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let svc = TaskService::new(
        state.engine.clone(),
        state.task_store.clone(),
        state.doc_id.clone(),
    );
    svc.delete(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
