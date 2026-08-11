//! `/_relay/auth/*` — how a CLI gets a token to open a tunnel with.
//!
//! Two ways in, and the relay says which of them it has:
//!
//! - **email and password**, always available;
//! - **GitHub**, only when the relay was started with an OAuth app configured.
//!
//! `GET /_relay/auth/methods` is what makes the second honest. The CLI asks
//! before it offers anything, so a relay with no GitHub app never shows a
//! GitHub option — rather than showing one that fails at the last step, which
//! is the worst possible place to learn a feature is not configured.
//!
//! Whichever way in was used, the answer is the same: a token this relay
//! signed. The tunnel handshake then verifies its own signature and asks
//! nobody anything, which is why a page load through the relay costs no
//! network call to GitHub.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::RelayState;
use crate::accounts::{self, Account};

/// How long a sign-in lasts before the CLI has to do it again.
const TOKEN_DAYS: i64 = 30;

fn fail(status: StatusCode, message: impl Into<String>) -> Response {
    (status, Json(json!({ "error": message.into() }))).into_response()
}

#[derive(Debug, Serialize)]
pub struct GithubMethod {
    /// Public by construction: device flow has no client secret, and this id
    /// appears in every request the CLI makes to GitHub.
    pub client_id: String,
}

#[derive(Debug, Serialize)]
pub struct Methods {
    pub password: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<GithubMethod>,
}

/// `GET /_relay/auth/methods` — what this relay accepts.
pub async fn methods(State(state): State<RelayState>) -> Json<Methods> {
    Json(Methods {
        password: true,
        github: state
            .github_client_id
            .as_ref()
            .map(|client_id| GithubMethod {
                client_id: client_id.clone(),
            }),
    })
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

fn issued(state: &RelayState, account: Account) -> Response {
    match hickory_identity::issue_token(
        &state.token_secret,
        &account.id,
        &account.label,
        TOKEN_DAYS,
    ) {
        Ok(token) => Json(json!({
            "token": token,
            "account": account.label,
            "expires_in_days": TOKEN_DAYS,
        }))
        .into_response(),
        Err(e) => {
            log::error!("minting a token failed: {e:#}");
            fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "could not issue a token — try again shortly",
            )
        }
    }
}

/// `POST /_relay/auth/signup`
pub async fn signup(State(state): State<RelayState>, Json(body): Json<Credentials>) -> Response {
    let Some(pool) = state.accounts.as_ref() else {
        return fail(StatusCode::SERVICE_UNAVAILABLE, ACCOUNTS_UNAVAILABLE);
    };
    match accounts::sign_up(pool, &body.email, &body.password).await {
        Ok(account) => issued(&state, account),
        Err(e) => fail(StatusCode::BAD_REQUEST, format!("{e}")),
    }
}

/// `POST /_relay/auth/login`
pub async fn login(State(state): State<RelayState>, Json(body): Json<Credentials>) -> Response {
    let Some(pool) = state.accounts.as_ref() else {
        return fail(StatusCode::SERVICE_UNAVAILABLE, ACCOUNTS_UNAVAILABLE);
    };
    match accounts::sign_in(pool, &body.email, &body.password).await {
        Ok(account) => issued(&state, account),
        // 401, not 400: the credentials were well-formed and wrong.
        Err(e) => fail(StatusCode::UNAUTHORIZED, format!("{e}")),
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubExchange {
    /// A GitHub access token the CLI obtained through device flow.
    pub access_token: String,
}

/// `POST /_relay/auth/github` — trade a GitHub token for one of ours.
///
/// The exchange happens once, at sign-in. After it, the relay holds an account
/// of its own and never asks GitHub anything again — so a page load through a
/// tunnel costs no third-party round trip, and GitHub being down does not stop
/// an already-signed-in session.
pub async fn github(State(state): State<RelayState>, Json(body): Json<GithubExchange>) -> Response {
    let Some(pool) = state.accounts.as_ref() else {
        return fail(StatusCode::SERVICE_UNAVAILABLE, ACCOUNTS_UNAVAILABLE);
    };
    if state.github_client_id.is_none() {
        return fail(
            StatusCode::NOT_FOUND,
            "this relay does not offer GitHub sign-in — use an email and password",
        );
    }

    let login = match state.identifier.identify(&body.access_token).await {
        Ok(login) => login,
        Err(e) => return fail(StatusCode::UNAUTHORIZED, format!("{e:#}")),
    };
    match accounts::upsert_github(pool, &login).await {
        Ok(account) => issued(&state, account),
        Err(e) => {
            log::error!("linking a GitHub account failed: {e:#}");
            fail(
                StatusCode::INTERNAL_SERVER_ERROR,
                "could not complete the sign-in — try again shortly",
            )
        }
    }
}

const ACCOUNTS_UNAVAILABLE: &str = "this relay has no account store configured (RELAY_DATABASE_URL), so it cannot sign anyone \
     in. An operator can point it at a file — see docs/specs/freeform/relay.md.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_methods_answer_omits_github_entirely_when_unconfigured() {
        // Serialised shape matters: the CLI decides whether to offer GitHub by
        // the presence of the field, so an unconfigured relay must not send a
        // null that could be misread as "configured, empty".
        let with = Methods {
            password: true,
            github: Some(GithubMethod {
                client_id: "Iv1.example".into(),
            }),
        };
        let json = serde_json::to_value(&with).unwrap();
        assert_eq!(json["github"]["client_id"], "Iv1.example");

        let without = Methods {
            password: true,
            github: None,
        };
        let json = serde_json::to_value(&without).unwrap();
        assert!(json.get("github").is_none(), "{json}");
        assert_eq!(json["password"], true);
    }
}
