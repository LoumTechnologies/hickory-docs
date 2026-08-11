//! Who opened this tunnel.
//!
//! The relay has no accounts of its own. It takes the GitHub token the agent
//! presents, asks GitHub whose it is, and keeps the login for the life of the
//! connection — nothing is stored, and there is no user table to leak.
//!
//! The consequence, stated so nobody is surprised by it: a token revoked at
//! GitHub keeps working until the tunnel reconnects. For a session measured in
//! hours that is the right trade against calling GitHub on every request.

use std::sync::Arc;
use std::sync::Mutex;

use anyhow::{Context as _, Result, bail};
use async_trait::async_trait;

/// Resolves an access token to a GitHub login.
#[async_trait]
pub trait Identifier: Send + Sync {
    async fn identify(&self, token: &str) -> Result<String>;
}

/// The real one: `GET https://api.github.com/user`.
pub struct GitHubIdentifier {
    client: reqwest::Client,
    api_base: String,
}

impl GitHubIdentifier {
    pub fn new(api_base: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            client: reqwest::Client::new(),
            api_base: api_base.into().trim_end_matches('/').to_string(),
        })
    }
}

#[async_trait]
impl Identifier for GitHubIdentifier {
    async fn identify(&self, token: &str) -> Result<String> {
        let resp = self
            .client
            .get(format!("{}/user", self.api_base))
            .header("authorization", format!("Bearer {token}"))
            .header("accept", "application/vnd.github+json")
            // GitHub rejects requests with no user agent, and a specific one
            // makes our traffic identifiable if we ever need to be asked about
            // it.
            .header("user-agent", "hickory-relay")
            .send()
            .await
            .context("asking GitHub who this token belongs to")?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            bail!("GitHub rejected this token — run `hickory login` again");
        }
        if !status.is_success() {
            bail!("GitHub answered {status} when asked to identify this token");
        }

        let body: serde_json::Value = resp.json().await.context("reading GitHub's answer")?;
        let login = body
            .get("login")
            .and_then(|v| v.as_str())
            .context("GitHub's answer carried no login")?;
        Ok(login.to_string())
    }
}

/// A stub for tests: every token maps to one account, and the calls are
/// counted so a test can prove the relay does not ask GitHub per request.
pub struct StubIdentifier {
    account: String,
    calls: Mutex<usize>,
    reject: Mutex<bool>,
}

impl StubIdentifier {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            calls: Mutex::new(0),
            reject: Mutex::new(false),
        }
    }

    pub fn calls(&self) -> usize {
        *self.calls.lock().unwrap()
    }

    pub fn set_reject(&self, reject: bool) {
        *self.reject.lock().unwrap() = reject;
    }
}

#[async_trait]
impl Identifier for StubIdentifier {
    async fn identify(&self, _token: &str) -> Result<String> {
        *self.calls.lock().unwrap() += 1;
        if *self.reject.lock().unwrap() {
            bail!("GitHub rejected this token — run `hickory login` again");
        }
        Ok(self.account.clone())
    }
}
