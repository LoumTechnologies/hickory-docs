//! Bearer auth for the hosted server.
//!
//! The primitives — argon2 hashing, HS256 tokens — live in `hickory-identity`,
//! shared with the relay. What stays here is what is specific to this server:
//! its `users` table, its extractors, and mapping a token failure onto an
//! `ApiError`.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::error::ApiError;

pub use hickory_identity::{hash_password, verify_password};

/// This server's claims. Distinct from `hickory_identity::Claims` because the
/// subject is a `Uuid` here and the field is named `email`; changing either
/// would invalidate every token already issued to a signed-in user.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub exp: i64,
}

pub fn issue_token(secret: &str, user_id: Uuid, email: &str) -> anyhow::Result<String> {
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp(),
    };
    Ok(jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn verify_token(secret: &str, token: &str) -> Result<Claims, ApiError> {
    jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| ApiError::unauthorized("invalid or expired token"))
}

/// A row from `users`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub plan_key: String,
    pub price_key: Option<String>,
    pub billing_status: String,
    pub email_verified: bool,
}

pub async fn load_user(state: &AppState, id: Uuid) -> Result<User, ApiError> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, plan_key, price_key, billing_status, email_verified \
         FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::unauthorized("user no longer exists"))
}

/// Extractor: requires a valid `Authorization: Bearer <JWT>`.
pub struct AuthUser(pub User);

/// Extractor: optional auth (public-doc reads).
pub struct MaybeUser(pub Option<User>);

fn bearer_token(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token =
            bearer_token(parts).ok_or_else(|| ApiError::unauthorized("missing bearer token"))?;
        let claims = verify_token(&state.config.jwt_secret, &token)?;
        Ok(AuthUser(load_user(state, claims.sub).await?))
    }
}

impl FromRequestParts<AppState> for MaybeUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match bearer_token(parts) {
            None => Ok(MaybeUser(None)),
            Some(token) => {
                let claims = verify_token(&state.config.jwt_secret, &token)?;
                Ok(MaybeUser(Some(load_user(state, claims.sub).await?)))
            }
        }
    }
}
