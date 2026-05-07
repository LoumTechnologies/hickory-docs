//! LLM-based merge strategy using an HTTP API.

use async_trait::async_trait;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use hick_store::ObjectStore;

use crate::error::MergeError;
use crate::merge_strategy::MergeStrategy;

/// Request body sent to the LLM merge API.
#[derive(Debug, Serialize)]
struct LlmMergeRequest {
    path: String,
    base: String,
    generated: String,
    edited: String,
}

/// Response body from the LLM merge API.
#[derive(Debug, Deserialize)]
struct LlmMergeResponse {
    merged: String,
}

/// Merge strategy that delegates to an LLM API for intelligent conflict resolution.
///
/// Caches results in the ObjectStore to avoid re-calling the API for identical
/// conflicts.
pub struct LlmMergeStrategy {
    api_url: String,
    client: reqwest::Client,
    cache: Option<Box<dyn ObjectStore>>,
}

impl LlmMergeStrategy {
    /// Create a new LLM merge strategy.
    ///
    /// `api_url` is the endpoint that accepts POST requests with `{base, generated, edited}`.
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            client: reqwest::Client::new(),
            cache: None,
        }
    }

    /// Enable caching of merge results in an ObjectStore.
    pub fn with_cache(mut self, store: Box<dyn ObjectStore>) -> Self {
        self.cache = Some(store);
        self
    }

    fn cache_key(base: &[u8], generated: &[u8], edited: &[u8]) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(base);
        hasher.update(b"|");
        hasher.update(generated);
        hasher.update(b"|");
        hasher.update(edited);
        let hash = hex::encode(hasher.finalize());
        format!("merge-cache/{hash}.merged")
    }
}

#[async_trait]
impl MergeStrategy for LlmMergeStrategy {
    async fn merge(
        &self,
        path: &str,
        base: &[u8],
        generated: &[u8],
        edited: &[u8],
    ) -> Result<Vec<u8>, MergeError> {
        // Check cache
        let cache_key = Self::cache_key(base, generated, edited);
        if let Some(cache) = &self.cache
            && let Ok(cached) = cache.get(&cache_key).await
        {
            debug!("Cache hit for merge of {path}");
            return Ok(cached);
        }

        info!("Calling LLM merge API for {path}");

        let request = LlmMergeRequest {
            path: path.to_string(),
            base: String::from_utf8_lossy(base).into_owned(),
            generated: String::from_utf8_lossy(generated).into_owned(),
            edited: String::from_utf8_lossy(edited).into_owned(),
        };

        let response = self
            .client
            .post(&self.api_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| MergeError::LlmApi(format!("request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(MergeError::LlmApi(format!("API returned {status}: {body}")));
        }

        let merge_response: LlmMergeResponse = response
            .json()
            .await
            .map_err(|e| MergeError::LlmApi(format!("invalid response: {e}")))?;

        let result = merge_response.merged.into_bytes();

        // Store in cache
        if let Some(cache) = &self.cache {
            let _ = cache.put(&cache_key, &result).await;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_store::InMemoryObjectStore;
    use std::sync::Arc;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn llm_merge_calls_api() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/merge"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "merged": "merged content"
            })))
            .mount(&mock_server)
            .await;

        let strategy = LlmMergeStrategy::new(&format!("{}/merge", mock_server.uri()));
        let result = strategy
            .merge("file.txt", b"base", b"generated", b"edited")
            .await
            .unwrap();
        assert_eq!(result, b"merged content");
    }

    #[tokio::test]
    async fn llm_merge_caches_result() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/merge"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "merged": "cached result"
            })))
            .expect(1) // Should only be called once
            .mount(&mock_server)
            .await;

        let cache = Box::new(InMemoryObjectStore::new());
        let strategy =
            LlmMergeStrategy::new(&format!("{}/merge", mock_server.uri())).with_cache(cache);

        // First call hits API
        let r1 = strategy
            .merge("file.txt", b"base", b"gen", b"edit")
            .await
            .unwrap();
        assert_eq!(r1, b"cached result");

        // Second call uses cache
        let r2 = strategy
            .merge("file.txt", b"base", b"gen", b"edit")
            .await
            .unwrap();
        assert_eq!(r2, b"cached result");
    }

    #[tokio::test]
    async fn llm_merge_api_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/merge"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&mock_server)
            .await;

        let strategy = LlmMergeStrategy::new(&format!("{}/merge", mock_server.uri()));
        let result = strategy.merge("file.txt", b"base", b"gen", b"edit").await;
        assert!(matches!(result, Err(MergeError::LlmApi(_))));
    }
}
