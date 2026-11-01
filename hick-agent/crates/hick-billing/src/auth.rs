//! GitHub OAuth 2.0 login handlers.
//!
//! Flow: GET /auth/github → redirect to GitHub → GET /auth/github/callback
//! → exchange code → upsert user → set JWT cookie → render token page.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{Html, Redirect, Response};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::AppError;
use crate::jwt;

const GITHUB_AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";

// ---------------------------------------------------------------------------
// Initiate OAuth
// ---------------------------------------------------------------------------

pub async fn login(State(state): State<Arc<AppState>>) -> Response {
    // Encode a short-lived signed nonce as the OAuth state parameter to
    // prevent CSRF.  No server-side storage needed.
    let oauth_state = jwt::sign_oauth_state(&state.jwt_secret);

    let url = format!(
        "{GITHUB_AUTHORIZE_URL}?client_id={}&scope=read:user,user:email&state={}",
        state.github_client_id, oauth_state
    );

    Redirect::temporary(&url).into_response()
}

// ---------------------------------------------------------------------------
// OAuth callback
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GithubTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
    email: Option<String>,
}

pub async fn callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CallbackParams>,
) -> Result<Response, AppError> {
    // Verify the CSRF nonce.
    jwt::verify_oauth_state(&state.jwt_secret, &params.state)
        .map_err(|_| AppError::bad_request("invalid OAuth state parameter"))?;

    // Exchange code for access token.
    let http = reqwest::Client::new();
    let token_resp: GithubTokenResponse = http
        .post(GITHUB_TOKEN_URL)
        .header("accept", "application/json")
        .json(&serde_json::json!({
            "client_id":     state.github_client_id,
            "client_secret": state.github_client_secret,
            "code":          params.code,
        }))
        .send()
        .await
        .map_err(AppError::internal)?
        .json()
        .await
        .map_err(AppError::internal)?;

    // Fetch user profile.
    let gh_user: GithubUser = http
        .get(GITHUB_USER_URL)
        .bearer_auth(&token_resp.access_token)
        .header("user-agent", "hick-billing/1.0")
        .send()
        .await
        .map_err(AppError::internal)?
        .json()
        .await
        .map_err(AppError::internal)?;

    // Upsert user in the database.
    let user = crate::db::upsert_user(
        &state.db,
        gh_user.id,
        &gh_user.login,
        gh_user.email.as_deref(),
    )
    .await
    .map_err(AppError::internal)?;

    // Create a Stripe customer if we have a key and the user doesn't have one.
    if user.stripe_customer_id.is_none() {
        if let Some(stripe_key) = &state.stripe_secret_key {
            match crate::billing::stripe_create_customer(
                stripe_key,
                &user.github_login,
                user.github_email.as_deref(),
            )
            .await
            {
                Ok(cid) => {
                    if let Err(e) =
                        crate::db::set_stripe_customer(&state.db, user.id, &cid).await
                    {
                        log::warn!("failed to save Stripe customer id: {e}");
                    }
                }
                Err(e) => log::warn!("failed to create Stripe customer: {e}"),
            }
        }
    }

    // Issue a long-lived JWT.
    let token = jwt::sign_user_token(&state.jwt_secret, user.id, &user.github_login)
        .map_err(AppError::internal)?;

    // Render a page that shows the token for the user to save locally.
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8"><title>hick billing — authenticated</title></head>
<body>
<h1>Authenticated as @{login}</h1>
<p>Save this token to <code>~/.hick/billing_token</code>:</p>
<pre id="tok">{token}</pre>
<button onclick="navigator.clipboard.writeText(document.getElementById('tok').textContent)">
  Copy to clipboard
</button>
<hr/>
<p>Or run:</p>
<pre>mkdir -p ~/.hick &amp;&amp; echo '{token}' &gt; ~/.hick/billing_token</pre>
</body>
</html>"#,
        login = user.github_login,
        token = token,
    );

    Ok(Html(html).into_response())
}
