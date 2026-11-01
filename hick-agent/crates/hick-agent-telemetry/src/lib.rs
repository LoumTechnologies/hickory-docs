//! Phone-home telemetry extension for the hick agent.
//!
//! Wraps any [`LlmClient`] in a [`TelemetryClient`] that reads
//! [`TokenUsage`] after every call and enqueues it to a background
//! [`UsageReporter`] which batches and POSTs records to the billing server.
//!
//! No prompt content is ever recorded — only token counts, provider, model,
//! and machine identity.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use hick_agent_sdk::{ChatChunk, LlmClient, Message, TokenUsage};

// ---------------------------------------------------------------------------
// Wire types sent to the billing server
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct UsageEvent {
    machine_id: String,
    timestamp: DateTime<Utc>,
    provider: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
}

#[derive(Serialize)]
struct UsageBatch {
    events: Vec<UsageEvent>,
}

// ---------------------------------------------------------------------------
// UsageReporter — background flush loop
// ---------------------------------------------------------------------------

/// Accepts [`TokenUsage`] records from any thread and batches them to the
/// billing server.  Fire-and-forget: a failed flush is logged and discarded
/// rather than surfaced to the caller.
pub struct UsageReporter {
    tx: mpsc::UnboundedSender<(TokenUsage, DateTime<Utc>)>,
}

impl UsageReporter {
    /// Spawn the background flush task.
    ///
    /// `billing_url` — base URL of the billing server, e.g.
    ///   `https://billing.hickory.dev`.
    /// `auth_token` — JWT obtained from `hick auth`.  May be `None` if the
    ///   user has not authenticated; events are sent without a user claim.
    /// `machine_id` — stable per-machine identifier.
    pub fn spawn(
        billing_url: String,
        auth_token: Option<String>,
        machine_id: String,
    ) -> Arc<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(flush_loop(billing_url, auth_token, machine_id, rx));
        Arc::new(Self { tx })
    }

    pub fn report(&self, usage: TokenUsage) {
        let _ = self.tx.send((usage, Utc::now()));
    }
}

async fn flush_loop(
    billing_url: String,
    auth_token: Option<String>,
    machine_id: String,
    mut rx: mpsc::UnboundedReceiver<(TokenUsage, DateTime<Utc>)>,
) {
    let client = reqwest::Client::new();
    let mut buf: Vec<(TokenUsage, DateTime<Utc>)> = Vec::new();
    let endpoint = format!("{billing_url}/api/usage");

    loop {
        // Drain up to 20 events or wait 30 s for the first one.
        let deadline = tokio::time::sleep(std::time::Duration::from_secs(30));
        tokio::pin!(deadline);

        loop {
            tokio::select! {
                item = rx.recv() => {
                    match item {
                        Some(ev) => {
                            buf.push(ev);
                            if buf.len() >= 20 { break; }
                        }
                        None => return, // channel closed — exit
                    }
                }
                _ = &mut deadline => break,
            }
        }

        if buf.is_empty() {
            continue;
        }

        let events: Vec<UsageEvent> = buf
            .drain(..)
            .map(|(u, ts)| UsageEvent {
                machine_id: machine_id.clone(),
                timestamp: ts,
                provider: u.provider,
                model: u.model,
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
                cache_creation_tokens: u.cache_creation_tokens,
                cache_read_tokens: u.cache_read_tokens,
            })
            .collect();

        let batch = UsageBatch { events };
        let mut req = client.post(&endpoint).json(&batch);
        if let Some(token) = &auth_token {
            req = req.bearer_auth(token);
        }

        if let Err(e) = req.send().await {
            log::warn!("telemetry flush failed: {e}");
        }
    }
}

// ---------------------------------------------------------------------------
// Stream wrapper that captures usage after stream exhaustion
// ---------------------------------------------------------------------------

struct UsageCapturingStream<S> {
    inner: S,
    done: bool,
    // Called exactly once when the stream returns Poll::Ready(None).
    on_done: Option<Box<dyn FnOnce() + Send>>,
}

impl<S: Stream<Item = anyhow::Result<ChatChunk>> + Unpin + Send> Stream
    for UsageCapturingStream<S>
{
    type Item = anyhow::Result<ChatChunk>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_next(cx);
        if matches!(result, std::task::Poll::Ready(None)) && !this.done {
            this.done = true;
            if let Some(cb) = this.on_done.take() {
                cb();
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// TelemetryClient — LlmClient wrapper
// ---------------------------------------------------------------------------

/// Wraps any [`LlmClient`] and transparently reports [`TokenUsage`] to a
/// [`UsageReporter`] after every call.
pub struct TelemetryClient {
    // Arc<Box<dyn LlmClient>> lets us clone the Arc into stream-end callbacks
    // without requiring LlmClient: Clone.
    inner: Arc<Box<dyn LlmClient>>,
    reporter: Arc<UsageReporter>,
}

impl TelemetryClient {
    pub fn new(inner: Box<dyn LlmClient>, reporter: Arc<UsageReporter>) -> Self {
        Self {
            inner: Arc::new(inner),
            reporter,
        }
    }
}

#[async_trait]
impl LlmClient for TelemetryClient {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let result = self.inner.complete(messages).await?;
        if let Some(usage) = self.inner.last_usage() {
            self.reporter.report(usage);
        }
        Ok(result)
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        let stream = self.inner.complete_stream(messages).await?;

        let inner = Arc::clone(&self.inner);
        let reporter = Arc::clone(&self.reporter);

        let capturing = UsageCapturingStream {
            inner: stream,
            done: false,
            on_done: Some(Box::new(move || {
                if let Some(usage) = inner.last_usage() {
                    reporter.report(usage);
                }
            })),
        };

        Ok(Box::pin(capturing))
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }

    fn last_usage(&self) -> Option<TokenUsage> {
        self.inner.last_usage()
    }
}

// ---------------------------------------------------------------------------
// Config helpers
// ---------------------------------------------------------------------------

/// Config read from the `[telemetry]` section of `agent.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    /// Base URL of the billing server.
    #[serde(default = "default_billing_url")]
    pub billing_url: String,
    /// Path to a file containing the auth JWT obtained from `hick auth`.
    /// Relative paths are resolved from the user home directory.
    /// Default: `~/.hick/billing_token`.
    #[serde(default)]
    pub token_file: Option<std::path::PathBuf>,
}

fn default_billing_url() -> String {
    "https://billing.hickory.dev".into()
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            billing_url: default_billing_url(),
            token_file: None,
        }
    }
}

impl TelemetryConfig {
    /// Read the auth token from disk (token_file → ~/.hick/billing_token).
    pub fn read_auth_token(&self) -> Option<String> {
        let path = self.token_file.clone().unwrap_or_else(|| {
            dirs_next::home_dir()
                .unwrap_or_default()
                .join(".hick")
                .join("billing_token")
        });
        std::fs::read_to_string(&path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}
