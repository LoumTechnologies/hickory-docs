use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;

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
    let core = state.core.lock().await;
    let tasks = core
        .list_tasks()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
    let mut core = state.core.lock().await;
    let id = core
        .create_task(&req.description, None)
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
    let mut core = state.core.lock().await;

    // Build UpdateTask command attrs
    let mut attrs = BTreeMap::new();
    attrs.insert("id".into(), id.clone());
    if let Some(d) = &req.description {
        attrs.insert("description".into(), d.clone());
    }
    if let Some(s) = &req.status {
        attrs.insert("status".into(), s.clone());
    }
    if let Some(dd) = &req.due_date {
        attrs.insert("due_date".into(), dd.clone());
    }
    core.sync_app_mut()
        .execute_command("task", "UpdateTask", &attrs, None)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let task = core
        .get_task(&id)
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
    let mut core = state.core.lock().await;
    core.delete_task(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}
