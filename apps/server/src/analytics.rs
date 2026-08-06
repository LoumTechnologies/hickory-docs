//! Server-side PostHog capture. No-op when PostHog is not configured
//! (graceful degradation, logged once at boot).

use serde_json::{Value, json};

use crate::config::PosthogConfig;

#[derive(Clone)]
pub struct Analytics {
    inner: Option<(reqwest::Client, PosthogConfig)>,
}

impl Analytics {
    pub fn new(config: Option<PosthogConfig>) -> Analytics {
        Analytics {
            inner: config.map(|c| (reqwest::Client::new(), c)),
        }
    }

    /// Fire-and-forget event capture.
    pub fn capture(&self, distinct_id: &str, event: &str, properties: Value) {
        let Some((client, cfg)) = self.inner.clone() else {
            return;
        };
        let body = json!({
            "api_key": cfg.api_key,
            "event": event,
            "distinct_id": distinct_id,
            "properties": properties,
        });
        let event = event.to_string();
        tokio::spawn(async move {
            let url = format!("{}/capture/", cfg.host.trim_end_matches('/'));
            if let Err(e) = client.post(url).json(&body).send().await {
                log::warn!("posthog capture '{event}' failed: {e}");
            }
        });
    }

    /// Evaluate a feature flag via `/decide` (plan-set selection). Returns
    /// `None` on any failure or when PostHog is not configured.
    pub async fn feature_flag(&self, distinct_id: &str, flag: &str) -> Option<String> {
        let (client, cfg) = self.inner.as_ref()?;
        let url = format!("{}/decide/?v=3", cfg.host.trim_end_matches('/'));
        let body = json!({ "api_key": cfg.api_key, "distinct_id": distinct_id });
        let resp = client
            .post(url)
            .json(&body)
            .timeout(std::time::Duration::from_millis(1500))
            .send()
            .await
            .ok()?;
        let v: Value = resp.json().await.ok()?;
        v.get("featureFlags")?
            .get(flag)?
            .as_str()
            .map(|s| s.to_string())
    }
}
