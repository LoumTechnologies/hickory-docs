//! Anthropic Message Batches API client (E5: verification fan-out).
//!
//! Batches are 50% cheaper than interactive calls and latency-insensitive —
//! exactly the shape of `hickory check` across many documents. Results
//! arrive in ANY order, so everything is keyed by `custom_id`; poll until
//! `processing_status == "ended"`, then fetch the results JSONL.
//!
//! Fan-out + caching note: concurrent identical-prefix requests cannot read
//! a cache entry that is still being written. When mixing interactive
//! warm-up with a fan-out, send ONE request first, await it, then send the
//! rest. Inside a batch the scheduler decides; do not rely on cache reads
//! between batch entries.
//!
//! Wiring into `hickory check` itself lives in `hickory-cli` (owned by
//! another workstream); this client is the complete transport it needs:
//! build one [`BatchEntry`] per document's verification prompt, `submit`,
//! `wait_until_ended`, `results`, and key pass/fail by document path used
//! as `custom_id`.

use serde::Deserialize;

use crate::llm::Message;
use crate::llm_anthropic::{DEFAULT_ANTHROPIC_MODEL, Effort, build_request};
use crate::usage::Usage;

const BATCHES_URL: &str = "https://api.anthropic.com/v1/messages/batches";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// One request in a batch, keyed by `custom_id` (results return in any
/// order — the id is how you find yours).
pub struct BatchEntry {
    /// Caller-chosen id (e.g. the document path being verified).
    pub custom_id: String,
    /// The conversation for this entry (System messages become the system
    /// prefix, same deterministic construction as the interactive path).
    pub messages: Vec<Message>,
}

/// Terminal result of one batch entry.
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// The caller's id for this entry.
    pub custom_id: String,
    /// Whether the entry succeeded.
    pub ok: bool,
    /// Response text on success; error description otherwise.
    pub text: String,
    /// Four-way usage for this entry (zero on error).
    pub usage: Usage,
}

/// Minimal Message Batches client.
pub struct AnthropicBatchClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    effort: Option<Effort>,
}

impl AnthropicBatchClient {
    /// Create with `ANTHROPIC_API_KEY` from env and the default model.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            model: DEFAULT_ANTHROPIC_MODEL.to_string(),
            max_tokens: 16384,
            effort: None,
        }
    }

    /// Override the model id.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Override the API key.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }

    /// Set `output_config.effort` for every entry (verification fan-out
    /// usually runs fine at [`Effort::Low`]).
    pub fn with_effort(mut self, effort: Effort) -> Self {
        self.effort = Some(effort);
        self
    }

    fn headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
    }

    /// Submit a batch; returns the batch id to poll.
    pub async fn submit(&self, entries: &[BatchEntry]) -> anyhow::Result<String> {
        let requests: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                let params = build_request(
                    &self.model,
                    self.max_tokens,
                    self.effort,
                    false,
                    &e.messages,
                    false,
                );
                serde_json::json!({
                    "custom_id": e.custom_id,
                    "params": params,
                })
            })
            .collect();
        let resp = self
            .headers(self.client.post(BATCHES_URL))
            .json(&serde_json::json!({ "requests": requests }))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("batch submit error {}: {}", status, resp.text().await?);
        }
        let parsed: BatchStatus = resp.json().await?;
        Ok(parsed.id)
    }

    /// Fetch the batch's current status.
    async fn status(&self, batch_id: &str) -> anyhow::Result<BatchStatus> {
        let resp = self
            .headers(self.client.get(format!("{BATCHES_URL}/{batch_id}")))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("batch status error {}: {}", status, resp.text().await?);
        }
        Ok(resp.json().await?)
    }

    /// Poll until `processing_status == "ended"` (interval `poll`, giving
    /// up after `timeout`).
    pub async fn wait_until_ended(
        &self,
        batch_id: &str,
        poll: std::time::Duration,
        timeout: std::time::Duration,
    ) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        loop {
            let status = self.status(batch_id).await?;
            if status.processing_status == "ended" {
                return Ok(());
            }
            if start.elapsed() > timeout {
                anyhow::bail!(
                    "batch {batch_id} still '{}' after {:?}",
                    status.processing_status,
                    timeout
                );
            }
            tokio::time::sleep(poll).await;
        }
    }

    /// Fetch the finished batch's results (any order; keyed by
    /// `custom_id`). Call only after [`Self::wait_until_ended`].
    pub async fn results(&self, batch_id: &str) -> anyhow::Result<Vec<BatchResult>> {
        let status = self.status(batch_id).await?;
        let url = status
            .results_url
            .ok_or_else(|| anyhow::anyhow!("batch {batch_id} has no results_url yet"))?;
        let resp = self.headers(self.client.get(url)).send().await?;
        let http_status = resp.status();
        if !http_status.is_success() {
            anyhow::bail!(
                "batch results error {}: {}",
                http_status,
                resp.text().await?
            );
        }
        let body = resp.text().await?;
        let mut results = Vec::new();
        for line in body.lines().filter(|l| !l.trim().is_empty()) {
            results.push(parse_result_line(line)?);
        }
        Ok(results)
    }
}

impl Default for AnthropicBatchClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct BatchStatus {
    id: String,
    processing_status: String,
    #[serde(default)]
    results_url: Option<String>,
}

/// Parse one results-JSONL line into a [`BatchResult`].
fn parse_result_line(line: &str) -> anyhow::Result<BatchResult> {
    let v: serde_json::Value = serde_json::from_str(line)?;
    let custom_id = v["custom_id"].as_str().unwrap_or_default().to_string();
    let result = &v["result"];
    let kind = result["type"].as_str().unwrap_or("unknown");
    if kind == "succeeded" {
        let message = &result["message"];
        let text = message["content"]
            .as_array()
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|b| b["text"].as_str())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();
        let usage = message
            .get("usage")
            .map(|u| Usage {
                input_tokens: u["input_tokens"].as_u64().unwrap_or(0),
                cache_creation_input_tokens: u["cache_creation_input_tokens"].as_u64().unwrap_or(0),
                cache_read_input_tokens: u["cache_read_input_tokens"].as_u64().unwrap_or(0),
                output_tokens: u["output_tokens"].as_u64().unwrap_or(0),
            })
            .unwrap_or_default();
        Ok(BatchResult {
            custom_id,
            ok: true,
            text,
            usage,
        })
    } else {
        Ok(BatchResult {
            custom_id,
            ok: false,
            text: format!("{kind}: {}", result["error"]),
            usage: Usage::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_succeeded_result_line() {
        let line = r#"{"custom_id":"docs/a.hick","result":{"type":"succeeded","message":{"content":[{"type":"text","text":"PASS"}],"usage":{"input_tokens":10,"output_tokens":2,"cache_read_input_tokens":8}}}}"#;
        let r = parse_result_line(line).unwrap();
        assert!(r.ok);
        assert_eq!(r.custom_id, "docs/a.hick");
        assert_eq!(r.text, "PASS");
        assert_eq!(r.usage.cache_read_input_tokens, 8);
    }

    #[test]
    fn parses_errored_result_line() {
        let line =
            r#"{"custom_id":"x","result":{"type":"errored","error":{"type":"invalid_request"}}}"#;
        let r = parse_result_line(line).unwrap();
        assert!(!r.ok);
        assert!(r.text.contains("errored"));
    }
}
