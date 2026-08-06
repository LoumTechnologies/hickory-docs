//! GET/POST /api/projects, GET/POST /api/projects/:id/docs.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::gitstore::GitStore;
use crate::plans;

fn project_json(id: Uuid, name: &str, visibility: &str, created_at: DateTime<Utc>) -> Value {
    json!({
        "id": id,
        "name": name,
        "visibility": visibility,
        "created_at": created_at.to_rfc3339(),
    })
}

pub async fn list_projects(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Value>> {
    let rows = sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(
        "SELECT id, name, visibility, created_at FROM projects WHERE owner_id = $1 ORDER BY created_at",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(Value::Array(
        rows.iter()
            .map(|(id, name, vis, at)| project_json(*id, name, vis, *at))
            .collect(),
    )))
}

#[derive(Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub visibility: String,
}

pub async fn create_project(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateProject>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    if body.name.trim().is_empty() {
        return Err(ApiError::bad_request("project name required"));
    }
    if !matches!(body.visibility.as_str(), "public" | "private") {
        return Err(ApiError::bad_request("visibility must be public or private"));
    }

    // Entitlement: private-project count.
    if body.visibility == "private" {
        let ents = plans::resolve(
            &state.catalog,
            &user.plan_key,
            user.price_key.as_deref(),
            &user.billing_status,
        );
        let current: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM projects WHERE owner_id = $1 AND visibility = 'private'",
        )
        .bind(user.id)
        .fetch_one(&state.db)
        .await?;
        if !ents.private_projects.allows(current as u64) {
            return Err(ApiError::forbidden(format!(
                "private project limit reached on the {} plan — upgrade to add more",
                ents.plan_key
            )));
        }
    }

    let id = Uuid::new_v4();
    let row = sqlx::query_as::<_, (DateTime<Utc>,)>(
        "INSERT INTO projects (id, owner_id, name, visibility) VALUES ($1, $2, $3, $4) RETURNING created_at",
    )
    .bind(id)
    .bind(user.id)
    .bind(body.name.trim())
    .bind(&body.visibility)
    .fetch_one(&state.db)
    .await?;
    state.git.init_project(id).await?;
    Ok((
        StatusCode::CREATED,
        Json(project_json(id, body.name.trim(), &body.visibility, row.0)),
    ))
}

pub async fn list_docs(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(project_id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let owner: Option<(Uuid, String)> =
        sqlx::query_as("SELECT owner_id, visibility FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_optional(&state.db)
            .await?;
    let Some((owner_id, visibility)) = owner else {
        return Err(ApiError::not_found("project not found"));
    };
    if owner_id != user.id && visibility != "public" {
        return Err(ApiError::forbidden("no access to this project"));
    }
    let rows = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
        "SELECT id, path, updated_at FROM docs WHERE project_id = $1 ORDER BY path",
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(Value::Array(
        rows.iter()
            .map(|(id, path, at)| {
                json!({ "id": id, "path": path, "updated_at": at.to_rfc3339() })
            })
            .collect(),
    )))
}

#[derive(Deserialize)]
pub struct CreateDoc {
    pub path: String,
    pub source: String,
}

pub async fn create_doc(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateDoc>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT owner_id FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(&state.db)
        .await?;
    match owner {
        None => return Err(ApiError::not_found("project not found")),
        Some((owner_id,)) if owner_id != user.id => {
            return Err(ApiError::forbidden("only the project owner can create docs"));
        }
        _ => {}
    }
    GitStore::validate_path(&body.path).map_err(|e| ApiError::bad_request(e.to_string()))?;

    let id = Uuid::new_v4();
    let row = sqlx::query_as::<_, (DateTime<Utc>,)>(
        "INSERT INTO docs (id, project_id, path, source) VALUES ($1, $2, $3, $4)
         ON CONFLICT (project_id, path) DO NOTHING RETURNING updated_at",
    )
    .bind(id)
    .bind(project_id)
    .bind(&body.path)
    .bind(&body.source)
    .fetch_optional(&state.db)
    .await?;
    let Some((updated_at,)) = row else {
        return Err(ApiError::conflict("a doc with this path already exists"));
    };
    state
        .git
        .save_file(project_id, &body.path, &body.source, &format!("Create {}", body.path))
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "path": body.path,
            "source": body.source,
            "updated_at": updated_at.to_rfc3339(),
        })),
    ))
}
