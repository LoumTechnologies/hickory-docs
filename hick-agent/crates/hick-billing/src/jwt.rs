//! JWT utilities for user auth tokens and short-lived OAuth CSRF nonces.

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const USER_TOKEN_TTL_DAYS: i64 = 90;
const OAUTH_STATE_TTL_SECS: i64 = 300;

#[derive(Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,         // user UUID
    pub github_login: String,
    pub exp: i64,
}

#[derive(Serialize, Deserialize)]
struct OAuthStateClaims {
    exp: i64,
    nonce: u64,
}

pub fn sign_user_token(secret: &str, user_id: Uuid, github_login: &str) -> anyhow::Result<String> {
    let claims = UserClaims {
        sub: user_id.to_string(),
        github_login: github_login.to_string(),
        exp: (Utc::now() + Duration::days(USER_TOKEN_TTL_DAYS)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(Into::into)
}

pub fn verify_user_token(secret: &str, token: &str) -> anyhow::Result<UserClaims> {
    let data = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

pub fn sign_oauth_state(secret: &str) -> String {
    let claims = OAuthStateClaims {
        exp: (Utc::now() + Duration::seconds(OAUTH_STATE_TTL_SECS)).timestamp(),
        nonce: rand_u64(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_default()
}

pub fn verify_oauth_state(secret: &str, token: &str) -> anyhow::Result<()> {
    decode::<OAuthStateClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(())
}

fn rand_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 ^ (d.as_secs().wrapping_mul(6364136223846793005)))
        .unwrap_or(42)
}
