//! POST /api/auth/signup, POST /api/auth/login, GET /api/me.

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AppState;
use crate::auth::{AuthUser, hash_password, issue_token, verify_password};
use crate::email_tokens;
use crate::error::{ApiError, ApiResult};
use crate::mail::Message;

#[derive(Deserialize, ToSchema)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

/// The user shape every auth endpoint returns.
///
/// `email_verified` lets the client decide whether to prompt;
/// `verification_required` tells it whether prompting means anything at all,
/// since a deployment with no mailer cannot require proof it cannot request.
/// Without the second field the UI would nag every user of an unconfigured
/// instance about something they can never resolve.
#[derive(Debug, Serialize, ToSchema)]
pub struct UserOut {
    pub id: Uuid,
    pub email: String,
    pub plan: String,
    pub email_verified: bool,
    pub verification_required: bool,
}

fn user_out(state: &AppState, user: &crate::auth::User) -> UserOut {
    UserOut {
        id: user.id,
        email: user.email.clone(),
        plan: user.plan_key.clone(),
        email_verified: user.email_verified,
        verification_required: state.mailer.is_configured(),
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthOut {
    pub token: String,
    pub user: UserOut,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StatusOut {
    pub status: String,
}

/// Whether `email` may create an account.
///
/// An empty allowlist means open signup, so existing deployments are
/// unaffected. An entry beginning with `@` matches a whole domain; anything
/// else must match the address exactly. `email` is already trimmed and
/// lowercased by the caller, and the allowlist is lowercased at parse time,
/// so this comparison is case-insensitive on both sides.
pub fn signup_allowed(email: &str, allowlist: &[String]) -> bool {
    if allowlist.is_empty() {
        return true;
    }
    allowlist.iter().any(|entry| {
        if let Some(domain) = entry.strip_prefix('@') {
            email.rsplit_once('@').is_some_and(|(_, d)| d == domain)
        } else {
            email == entry
        }
    })
}

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    request_body = Credentials,
    responses((status = 200, description = "account created", body = AuthOut)),
    tag = "auth"
)]
pub async fn signup(
    State(state): State<AppState>,
    Json(body): Json<Credentials>,
) -> ApiResult<Json<AuthOut>> {
    let email = body.email.trim().to_lowercase();
    if !email.contains('@') {
        return Err(ApiError::bad_request("invalid email address"));
    }
    if body.password.len() < 8 {
        return Err(ApiError::bad_request(
            "password must be at least 8 characters",
        ));
    }
    if !signup_allowed(&email, &state.config.signup_allowlist) {
        // Deliberately does not say whether the address exists or why it was
        // refused — a closed deployment should not confirm which addresses
        // would work.
        return Err(ApiError::forbidden("signups are not open on this instance"));
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
    let user = crate::auth::User {
        id,
        email: email.clone(),
        plan_key: "open".to_string(),
        price_key: None,
        billing_status: "active".to_string(),
        email_verified: false,
    };
    Ok(Json(AuthOut {
        token,
        user: user_out(&state, &user),
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = Credentials,
    responses((status = 200, description = "signed in", body = AuthOut)),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<Credentials>,
) -> ApiResult<Json<AuthOut>> {
    let email = body.email.trim().to_lowercase();
    let row =
        sqlx::query_as::<_, (Uuid, String)>("SELECT id, password_hash FROM users WHERE email = $1")
            .bind(&email)
            .fetch_optional(&state.db)
            .await?;
    let Some((id, hash)) = row else {
        return Err(ApiError::unauthorized("invalid email or password"));
    };
    if !verify_password(&body.password, &hash) {
        return Err(ApiError::unauthorized("invalid email or password"));
    }
    let token = issue_token(&state.config.jwt_secret, id, &email)?;
    let user = crate::auth::load_user(&state, id).await?;
    Ok(Json(AuthOut {
        token,
        user: user_out(&state, &user),
    }))
}

#[utoipa::path(
    get,
    path = "/api/me",
    responses((status = 200, description = "current user", body = UserOut)),
    tag = "auth"
)]
pub async fn me(State(state): State<AppState>, AuthUser(user): AuthUser) -> Json<UserOut> {
    Json(user_out(&state, &user))
}

// ---------------------------------------------------------------------------
// Email verification and password reset
// ---------------------------------------------------------------------------

/// How many links of one purpose a user may be sent per hour.
///
/// Without a cap the send endpoint is a free mail cannon pointed at whatever
/// address is named, and a sending domain's reputation is not recoverable on
/// the timescale that matters.
const MAX_SENDS_PER_HOUR: i64 = 5;

/// Send (or re-send) a verification link to the authenticated user.
#[utoipa::path(
    post,
    path = "/api/auth/verify/send",
    responses((status = 200, description = "verification email queued", body = StatusOut)),
    tag = "auth"
)]
pub async fn send_verification(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<StatusOut>> {
    if !state.mailer.is_configured() {
        return Err(ApiError::service_unavailable(
            "email is not configured on this instance (SENDGRID_API_KEY unset)",
        ));
    }
    let already: Option<(bool,)> = sqlx::query_as("SELECT email_verified FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_optional(&state.db)
        .await?;
    if already.map(|(v,)| v).unwrap_or(false) {
        return Ok(Json(StatusOut {
            status: "already_verified".to_string(),
        }));
    }

    let since = chrono::Utc::now() - chrono::Duration::hours(1);
    if email_tokens::issued_since(&state.db, user.id, email_tokens::Purpose::Verify, since).await?
        >= MAX_SENDS_PER_HOUR
    {
        return Err(ApiError::too_many_requests(
            "too many verification emails requested; try again later",
        ));
    }

    let token = email_tokens::issue(&state.db, user.id, email_tokens::Purpose::Verify).await?;
    let link = format!(
        "{}/#/verify?token={token}",
        state.config.app_base_url.trim_end_matches('/')
    );
    let delivered = send_or_log(
        &state,
        Message {
            to: user.email.clone(),
            subject: "Confirm your email".to_string(),
            body: format!(
                "Confirm your address to finish setting up your Hickory Docs account:\n\n\
                 {link}\n\nThe link is good for 24 hours. If you did not create an \
                 account, ignore this message."
            ),
        },
    )
    .await;
    // Still 200: the token was issued and the link works if it ever arrives.
    // But the caller is authenticated, so there is nothing to leak by saying
    // the send failed — and "check your inbox" is bad advice when the mail
    // was rejected. (Deliberately NOT done for password reset, where varying
    // the response would turn it into an account-existence oracle.)
    Ok(Json(StatusOut {
        status: if delivered { "sent" } else { "send_failed" }.to_string(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct TokenBody {
    pub token: String,
}

/// Redeem a verification link.
#[utoipa::path(
    post,
    path = "/api/auth/verify/confirm",
    request_body = TokenBody,
    responses((status = 200, description = "email verified", body = StatusOut)),
    tag = "auth"
)]
pub async fn confirm_verification(
    State(state): State<AppState>,
    Json(body): Json<TokenBody>,
) -> ApiResult<Json<StatusOut>> {
    let Some(user_id) =
        email_tokens::redeem(&state.db, &body.token, email_tokens::Purpose::Verify).await?
    else {
        // One message for unknown, expired, already-used, and wrong-purpose:
        // distinguishing them tells an attacker which tokens exist.
        return Err(ApiError::bad_request(
            "this link is invalid or has expired; request a new one",
        ));
    };
    sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(StatusOut {
        status: "verified".to_string(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct EmailBody {
    pub email: String,
}

/// Begin a password reset.
///
/// Always answers 200, whether or not the address has an account. Anything
/// else turns this endpoint into an account-existence oracle, which is the
/// classic way a reset flow leaks the user list.
#[utoipa::path(
    post,
    path = "/api/auth/reset/request",
    request_body = EmailBody,
    responses((status = 200, description = "reset email queued if the address exists", body = StatusOut)),
    tag = "auth"
)]
pub async fn request_reset(
    State(state): State<AppState>,
    Json(body): Json<EmailBody>,
) -> ApiResult<Json<StatusOut>> {
    let email = body.email.trim().to_lowercase();
    let found: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.db)
        .await?;

    if let Some((user_id,)) = found
        && state.mailer.is_configured()
    {
        let since = chrono::Utc::now() - chrono::Duration::hours(1);
        let recent =
            email_tokens::issued_since(&state.db, user_id, email_tokens::Purpose::Reset, since)
                .await?;
        if recent < MAX_SENDS_PER_HOUR {
            let token =
                email_tokens::issue(&state.db, user_id, email_tokens::Purpose::Reset).await?;
            let link = format!(
                "{}/#/reset?token={token}",
                state.config.app_base_url.trim_end_matches('/')
            );
            let _ = send_or_log(
                &state,
                Message {
                    to: email.clone(),
                    subject: "Reset your password".to_string(),
                    body: format!(
                        "Use this link to choose a new password:\n\n{link}\n\n\
                         The link is good for one hour and can be used once. If you did \
                         not ask for this, ignore this message — your password has not \
                         changed."
                    ),
                },
            )
            .await;
        }
    }
    Ok(Json(StatusOut {
        status: "sent".to_string(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct ResetBody {
    pub token: String,
    pub password: String,
}

/// Complete a password reset.
#[utoipa::path(
    post,
    path = "/api/auth/reset/confirm",
    request_body = ResetBody,
    responses((status = 200, description = "password reset", body = StatusOut)),
    tag = "auth"
)]
pub async fn confirm_reset(
    State(state): State<AppState>,
    Json(body): Json<ResetBody>,
) -> ApiResult<Json<StatusOut>> {
    if body.password.len() < 8 {
        return Err(ApiError::bad_request(
            "password must be at least 8 characters",
        ));
    }
    let Some(user_id) =
        email_tokens::redeem(&state.db, &body.token, email_tokens::Purpose::Reset).await?
    else {
        return Err(ApiError::bad_request(
            "this link is invalid or has expired; request a new one",
        ));
    };
    let hash = hash_password(&body.password)?;
    // Completing a reset proves control of the mailbox, so it also settles
    // the verification question — leaving the account unverified afterwards
    // would be asking for the same proof twice.
    sqlx::query("UPDATE users SET password_hash = $1, email_verified = true WHERE id = $2")
        .bind(&hash)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(StatusOut {
        status: "reset".to_string(),
    }))
}

/// Send, logging failures rather than propagating them.
///
/// A provider outage must not fail the request: the user's account exists and
/// the send endpoint is retryable, so a 500 here would lose more than it
/// protects.
async fn send_or_log(state: &AppState, message: Message) -> bool {
    let to = message.to.clone();
    match state.mailer.send(message).await {
        Ok(()) => true,
        Err(e) => {
            log::error!("failed to send mail to {to}: {e:#}");
            false
        }
    }
}
