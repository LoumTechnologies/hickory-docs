//! POST /api/auth/signup, POST /api/auth/login, GET /api/me.

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::auth::{AuthUser, hash_password, issue_token, verify_password};
use crate::error::{ApiError, ApiResult};

#[derive(Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

fn user_json(id: Uuid, email: &str, plan: &str) -> Value {
    json!({ "id": id, "email": email, "plan": plan })
}

pub async fn signup(
    State(state): State<AppState>,
    Json(body): Json<Credentials>,
) -> ApiResult<Json<Value>> {
    let email = body.email.trim().to_lowercase();
    if !email.contains('@') {
        return Err(ApiError::bad_request("invalid email address"));
    }
    if body.password.len() < 8 {
        return Err(ApiError::bad_request(
            "password must be at least 8 characters",
        ));
    }
    let id = Uuid::new_v4();
    let hash = hash_password(&body.password)?;
    let inserted = sqlx::query(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)
         ON CONFLICT (email) DO NOTHING",
    )
    .bind(id)
    .bind(&email)
    .bind(&hash)
    .execute(&state.db)
    .await?;
    if inserted.rows_affected() == 0 {
        return Err(ApiError::conflict(
            "an account with this email already exists",
        ));
    }
    let token = issue_token(&state.config.jwt_secret, id, &email)?;
    state.analytics.capture(
        &id.to_string(),
        "signup",
        json!({ "email_domain": email.split('@').nth(1) }),
    );
    Ok(Json(
        json!({ "token": token, "user": user_json(id, &email, "open") }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<Credentials>,
) -> ApiResult<Json<Value>> {
    let email = body.email.trim().to_lowercase();
    let row = sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT id, password_hash, plan_key FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.db)
    .await?;
    let Some((id, hash, plan)) = row else {
        return Err(ApiError::unauthorized("invalid email or password"));
    };
    if !verify_password(&body.password, &hash) {
        return Err(ApiError::unauthorized("invalid email or password"));
    }
    let token = issue_token(&state.config.jwt_secret, id, &email)?;
    Ok(Json(
        json!({ "token": token, "user": user_json(id, &email, &plan) }),
    ))
}

pub async fn me(AuthUser(user): AuthUser) -> Json<Value> {
    Json(user_json(user.id, &user.email, &user.plan_key))
}
