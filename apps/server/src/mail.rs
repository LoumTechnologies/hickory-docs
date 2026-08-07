//! Transactional email: the SendGrid backend and the trait it sits behind.
//!
//! Two rules shape this module.
//!
//! **Sending must never fail a request.** A verification mail that does not
//! send is a retry (`POST /api/auth/verify/send`); a signup that 500s because
//! SendGrid was slow is a lost account. So `send` returns a `Result` the
//! callers log rather than propagate, and an unconfigured mailer is a no-op
//! that says so in the log — the same way Stripe and PostHog already degrade.
//!
//! **The trait exists so tests never send mail.** [`CapturingMailer`] records
//! what would have gone out, which is how the verification tests assert on a
//! link without a network or an API key.

use std::sync::Mutex;

use anyhow::{Context as _, Result};
use async_trait::async_trait;

/// One outbound message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub to: String,
    pub subject: String,
    /// Plain text only. Every message this product sends is a sentence and a
    /// link; HTML would add a spam-filter surface and a rendering problem for
    /// no gain.
    pub body: String,
}

#[async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, message: Message) -> Result<()>;
    /// Whether mail can actually be delivered. Callers use this to decide
    /// whether to *require* verification, so an unconfigured deployment does
    /// not lock everyone out of a product they could previously use.
    fn is_configured(&self) -> bool;
}

/// SendGrid v3 (`POST /v3/mail/send`).
pub struct SendGridMailer {
    client: reqwest::Client,
    api_key: String,
    from_email: String,
    from_name: String,
}

impl SendGridMailer {
    pub fn new(api_key: String, from_email: String, from_name: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            from_email,
            from_name,
        }
    }
}

#[async_trait]
impl Mailer for SendGridMailer {
    async fn send(&self, message: Message) -> Result<()> {
        let body = serde_json::json!({
            "personalizations": [{ "to": [{ "email": message.to }] }],
            "from": { "email": self.from_email, "name": self.from_name },
            "subject": message.subject,
            "content": [{ "type": "text/plain", "value": message.body }],
        });
        let resp = self
            .client
            .post("https://api.sendgrid.com/v3/mail/send")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("posting to SendGrid")?;

        let status = resp.status();
        if !status.is_success() {
            // SendGrid puts the actual reason in the body — a bare status is
            // almost never enough to tell an unverified sender identity from
            // a bad key, and those need different fixes.
            let detail = resp.text().await.unwrap_or_default();
            anyhow::bail!("SendGrid returned {status}: {}", detail.trim());
        }
        Ok(())
    }

    fn is_configured(&self) -> bool {
        true
    }
}

/// A mailer that drops everything and says so.
///
/// What a deployment with no `SENDGRID_API_KEY` gets. It reports
/// `is_configured() == false`, so verification is not *required* there —
/// otherwise forgetting one environment variable would silently make the
/// product unusable.
pub struct NullMailer;

#[async_trait]
impl Mailer for NullMailer {
    async fn send(&self, message: Message) -> Result<()> {
        log::info!(
            "mail not configured (SENDGRID_API_KEY unset); dropping {:?} to {}",
            message.subject,
            message.to
        );
        Ok(())
    }

    fn is_configured(&self) -> bool {
        false
    }
}

/// Records messages instead of sending them. Tests only.
#[derive(Default)]
pub struct CapturingMailer {
    sent: Mutex<Vec<Message>>,
}

impl CapturingMailer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sent(&self) -> Vec<Message> {
        self.sent.lock().unwrap().clone()
    }
}

#[async_trait]
impl Mailer for CapturingMailer {
    async fn send(&self, message: Message) -> Result<()> {
        self.sent.lock().unwrap().push(message);
        Ok(())
    }

    fn is_configured(&self) -> bool {
        true
    }
}
