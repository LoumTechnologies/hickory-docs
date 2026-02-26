//! Network request filtering for the agent.
//!
//! Provides configurable rules that control which hosts the agent can connect
//! to. Rules can allow, block, or prompt the user for each host pattern.
//! Configuration is loaded from `.hick/network-rules.toml` in the project
//! directory.
//!
//! Inspired by the connection filter in the container-testbed crate.

use std::collections::HashMap;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::Stream;
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};

use hick_token::ContainerCapabilities;

use crate::llm::{ChatChunk, LlmClient, Message};

/// Verdict for a filtered network request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkVerdict {
    Allow,
    Block,
}

/// Action to take when a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    /// Allow the request without prompting.
    Allow,
    /// Block the request silently.
    Block,
    /// Prompt the user for a decision.
    Prompt,
}

/// A network access rule matching a host pattern.
#[derive(Debug, Clone, Deserialize)]
pub struct NetworkRule {
    /// Host pattern. Exact match, wildcard prefix (`*.example.com`), or `*`
    /// to match all hosts.
    pub pattern: String,
    /// Action to take when the pattern matches.
    pub action: RuleAction,
}

/// Configuration for the network filter.
///
/// Loaded from `.hick/network-rules.toml`:
///
/// ```toml
/// default = "prompt"
///
/// [[rules]]
/// pattern = "api.anthropic.com"
/// action = "allow"
///
/// [[rules]]
/// pattern = "*.evil.com"
/// action = "block"
/// ```
#[derive(Debug, Deserialize)]
pub struct NetworkFilterConfig {
    /// Default action when no rule matches.
    #[serde(default = "default_action")]
    pub default: RuleAction,
    /// Ordered list of rules. First match wins.
    #[serde(default)]
    pub rules: Vec<NetworkRule>,
}

fn default_action() -> RuleAction {
    RuleAction::Prompt
}

impl Default for NetworkFilterConfig {
    fn default() -> Self {
        Self {
            default: RuleAction::Allow,
            rules: vec![],
        }
    }
}

/// Request sent to the TUI to prompt the user for network approval.
pub struct NetworkApprovalRequest {
    /// The host being accessed.
    pub host: String,
    /// Sender for the user's verdict.
    pub response_tx: oneshot::Sender<NetworkVerdict>,
}

/// Filters outbound network requests by host against configurable rules.
///
/// When a rule action is `Prompt`, sends a [`NetworkApprovalRequest`] to the
/// TUI and waits for the user's decision. Verdicts are cached per host for
/// the duration of the session.
pub struct NetworkFilter {
    config: NetworkFilterConfig,
    cache: Mutex<HashMap<String, NetworkVerdict>>,
    approval_tx: mpsc::UnboundedSender<NetworkApprovalRequest>,
}

impl NetworkFilter {
    /// Create a filter with the given configuration and approval channel.
    pub fn new(
        config: NetworkFilterConfig,
        approval_tx: mpsc::UnboundedSender<NetworkApprovalRequest>,
    ) -> Self {
        Self {
            config,
            cache: Mutex::new(HashMap::new()),
            approval_tx,
        }
    }

    /// Load filter configuration from a TOML file.
    pub fn load(
        path: &Path,
        approval_tx: mpsc::UnboundedSender<NetworkApprovalRequest>,
    ) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: NetworkFilterConfig = toml::from_str(&content)?;
        Ok(Self::new(config, approval_tx))
    }

    /// Create a filter that allows everything (used when no config file exists).
    pub fn allow_all(approval_tx: mpsc::UnboundedSender<NetworkApprovalRequest>) -> Self {
        Self::new(NetworkFilterConfig::default(), approval_tx)
    }

    /// Derive a network filter from [`ContainerCapabilities`].
    ///
    /// Translates capability network rules into filter rules and optionally
    /// adds an extra allowed host (e.g. the LLM provider's API endpoint).
    pub fn from_capabilities(
        capabilities: &ContainerCapabilities,
        extra_allow: Option<&str>,
        approval_tx: mpsc::UnboundedSender<NetworkApprovalRequest>,
    ) -> Self {
        let has_deny_all = capabilities
            .network_rules
            .iter()
            .any(|r| matches!(r, hick_token::NetworkRule::DenyAll));

        let default = if has_deny_all {
            RuleAction::Block
        } else {
            RuleAction::Allow
        };

        let mut rules = Vec::new();

        // Add explicit allows from capabilities.
        for rule in &capabilities.network_rules {
            if let hick_token::NetworkRule::Allow { host, .. } = rule {
                rules.push(NetworkRule {
                    pattern: host.clone(),
                    action: RuleAction::Allow,
                });
            }
        }

        // Add the extra allowed host (e.g. LLM API endpoint).
        if let Some(host) = extra_allow {
            rules.push(NetworkRule {
                pattern: host.to_string(),
                action: RuleAction::Allow,
            });
        }

        let config = NetworkFilterConfig { default, rules };
        Self::new(config, approval_tx)
    }

    /// Number of configured rules.
    pub fn rule_count(&self) -> usize {
        self.config.rules.len()
    }

    /// Human-readable name for the default action.
    pub fn default_action_name(&self) -> &'static str {
        match self.config.default {
            RuleAction::Allow => "allow",
            RuleAction::Block => "block",
            RuleAction::Prompt => "prompt",
        }
    }

    /// Check if a request to the given host is allowed.
    ///
    /// For `Prompt` rules, sends a request to the TUI and blocks until the
    /// user responds.
    pub async fn check(&self, host: &str) -> NetworkVerdict {
        // Check cache first
        if let Some(&verdict) = self.cache.lock().unwrap().get(host) {
            return verdict;
        }

        let action = self.find_matching_action(host);

        match action {
            RuleAction::Allow => {
                self.cache_verdict(host, NetworkVerdict::Allow);
                NetworkVerdict::Allow
            }
            RuleAction::Block => {
                self.cache_verdict(host, NetworkVerdict::Block);
                NetworkVerdict::Block
            }
            RuleAction::Prompt => self.prompt_user(host).await,
        }
    }

    fn find_matching_action(&self, host: &str) -> RuleAction {
        for rule in &self.config.rules {
            if host_matches(&rule.pattern, host) {
                return rule.action;
            }
        }
        self.config.default
    }

    fn cache_verdict(&self, host: &str, verdict: NetworkVerdict) {
        self.cache.lock().unwrap().insert(host.to_string(), verdict);
    }

    async fn prompt_user(&self, host: &str) -> NetworkVerdict {
        let (response_tx, response_rx) = oneshot::channel();
        let request = NetworkApprovalRequest {
            host: host.to_string(),
            response_tx,
        };

        if self.approval_tx.send(request).is_err() {
            // Channel closed (TUI gone) — fail open
            return NetworkVerdict::Allow;
        }

        match response_rx.await {
            Ok(verdict) => {
                self.cache_verdict(host, verdict);
                verdict
            }
            // Sender dropped (TUI gone) — fail open
            Err(_) => NetworkVerdict::Allow,
        }
    }
}

/// Check if a host pattern matches a given host.
///
/// Supports:
/// - `*` — matches any host
/// - `*.example.com` — matches `foo.example.com`, `bar.baz.example.com`,
///   and `example.com` itself
/// - `api.example.com` — exact match
fn host_matches(pattern: &str, host: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        host == suffix || host.ends_with(&format!(".{suffix}"))
    } else {
        host == pattern
    }
}

/// Extract the host component from a URL string.
pub fn extract_host_from_url(url: &str) -> Option<String> {
    let after_scheme = url.split("://").nth(1)?;
    let host_port = after_scheme.split('/').next()?;
    let host = host_port.split(':').next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

// ---------------------------------------------------------------------------
// FilteredLlmClient
// ---------------------------------------------------------------------------

/// An [`LlmClient`] wrapper that checks a [`NetworkFilter`] before each
/// API call.
///
/// If the filter blocks the request, the call returns an error. If the filter
/// prompts, the call blocks until the user responds in the TUI.
pub struct FilteredLlmClient {
    inner: Box<dyn LlmClient>,
    filter: Arc<NetworkFilter>,
    host: String,
}

impl FilteredLlmClient {
    pub fn new(inner: Box<dyn LlmClient>, filter: Arc<NetworkFilter>, host: String) -> Self {
        Self {
            inner,
            filter,
            host,
        }
    }

    async fn check_access(&self) -> anyhow::Result<()> {
        match self.filter.check(&self.host).await {
            NetworkVerdict::Allow => Ok(()),
            NetworkVerdict::Block => {
                anyhow::bail!("Network request to {} blocked by filter", self.host)
            }
        }
    }
}

#[async_trait]
impl LlmClient for FilteredLlmClient {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        self.check_access().await?;
        self.inner.complete(messages).await
    }

    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        self.check_access().await?;
        self.inner.complete_stream(messages).await
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_matches_exact() {
        assert!(host_matches("api.anthropic.com", "api.anthropic.com"));
        assert!(!host_matches("api.anthropic.com", "api.openai.com"));
    }

    #[test]
    fn host_matches_wildcard_all() {
        assert!(host_matches("*", "anything.example.com"));
        assert!(host_matches("*", "localhost"));
    }

    #[test]
    fn host_matches_wildcard_subdomain() {
        assert!(host_matches("*.example.com", "api.example.com"));
        assert!(host_matches("*.example.com", "foo.bar.example.com"));
        assert!(host_matches("*.example.com", "example.com"));
        assert!(!host_matches("*.example.com", "notexample.com"));
        assert!(!host_matches("*.example.com", "example.org"));
    }

    #[test]
    fn extract_host_basic() {
        assert_eq!(
            extract_host_from_url("https://api.anthropic.com/v1/messages"),
            Some("api.anthropic.com".to_string())
        );
        assert_eq!(
            extract_host_from_url("https://api.openai.com:443/v1"),
            Some("api.openai.com".to_string())
        );
        assert_eq!(
            extract_host_from_url("http://localhost:8080"),
            Some("localhost".to_string())
        );
    }

    #[test]
    fn extract_host_invalid() {
        assert_eq!(extract_host_from_url("not-a-url"), None);
        assert_eq!(extract_host_from_url("://"), None);
    }

    #[tokio::test]
    async fn filter_allow_all_permits_everything() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let filter = NetworkFilter::allow_all(tx);
        assert_eq!(filter.check("anything.com").await, NetworkVerdict::Allow);
    }

    #[tokio::test]
    async fn filter_block_rule() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = NetworkFilterConfig {
            default: RuleAction::Allow,
            rules: vec![NetworkRule {
                pattern: "evil.com".to_string(),
                action: RuleAction::Block,
            }],
        };
        let filter = NetworkFilter::new(config, tx);
        assert_eq!(filter.check("evil.com").await, NetworkVerdict::Block);
        assert_eq!(filter.check("good.com").await, NetworkVerdict::Allow);
    }

    #[tokio::test]
    async fn filter_wildcard_block() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = NetworkFilterConfig {
            default: RuleAction::Allow,
            rules: vec![NetworkRule {
                pattern: "*.evil.com".to_string(),
                action: RuleAction::Block,
            }],
        };
        let filter = NetworkFilter::new(config, tx);
        assert_eq!(filter.check("api.evil.com").await, NetworkVerdict::Block);
        assert_eq!(filter.check("evil.com").await, NetworkVerdict::Block);
        assert_eq!(filter.check("good.com").await, NetworkVerdict::Allow);
    }

    #[tokio::test]
    async fn filter_first_match_wins() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = NetworkFilterConfig {
            default: RuleAction::Block,
            rules: vec![
                NetworkRule {
                    pattern: "api.anthropic.com".to_string(),
                    action: RuleAction::Allow,
                },
                NetworkRule {
                    pattern: "*.anthropic.com".to_string(),
                    action: RuleAction::Block,
                },
            ],
        };
        let filter = NetworkFilter::new(config, tx);
        assert_eq!(
            filter.check("api.anthropic.com").await,
            NetworkVerdict::Allow
        );
        assert_eq!(
            filter.check("other.anthropic.com").await,
            NetworkVerdict::Block
        );
    }

    #[tokio::test]
    async fn filter_caches_verdicts() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = NetworkFilterConfig {
            default: RuleAction::Allow,
            rules: vec![],
        };
        let filter = NetworkFilter::new(config, tx);

        assert_eq!(filter.check("example.com").await, NetworkVerdict::Allow);
        assert!(filter.cache.lock().unwrap().contains_key("example.com"));
    }

    #[tokio::test]
    async fn filter_prompt_with_allow_response() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let config = NetworkFilterConfig {
            default: RuleAction::Prompt,
            rules: vec![],
        };
        let filter = Arc::new(NetworkFilter::new(config, tx));

        let filter_clone = filter.clone();
        let check_handle = tokio::spawn(async move { filter_clone.check("example.com").await });

        // Simulate TUI: receive the approval request and respond
        let request = rx.recv().await.unwrap();
        assert_eq!(request.host, "example.com");
        request.response_tx.send(NetworkVerdict::Allow).unwrap();

        let verdict = check_handle.await.unwrap();
        assert_eq!(verdict, NetworkVerdict::Allow);

        // Should be cached — no more prompts
        assert_eq!(filter.check("example.com").await, NetworkVerdict::Allow);
    }

    #[tokio::test]
    async fn filter_prompt_with_block_response() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let config = NetworkFilterConfig {
            default: RuleAction::Prompt,
            rules: vec![],
        };
        let filter = Arc::new(NetworkFilter::new(config, tx));

        let filter_clone = filter.clone();
        let check_handle = tokio::spawn(async move { filter_clone.check("evil.com").await });

        let request = rx.recv().await.unwrap();
        request.response_tx.send(NetworkVerdict::Block).unwrap();

        let verdict = check_handle.await.unwrap();
        assert_eq!(verdict, NetworkVerdict::Block);
    }

    #[tokio::test]
    async fn filter_prompt_fails_open_when_channel_closed() {
        let (tx, rx) = mpsc::unbounded_channel();
        let config = NetworkFilterConfig {
            default: RuleAction::Prompt,
            rules: vec![],
        };
        let filter = NetworkFilter::new(config, tx);

        // Drop the receiver to simulate TUI gone
        drop(rx);

        // Should fail open (allow)
        assert_eq!(filter.check("example.com").await, NetworkVerdict::Allow);
    }

    #[test]
    fn config_deserialization() {
        let toml_str = r#"
default = "prompt"

[[rules]]
pattern = "api.anthropic.com"
action = "allow"

[[rules]]
pattern = "*.evil.com"
action = "block"
"#;
        let config: NetworkFilterConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default, RuleAction::Prompt);
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].pattern, "api.anthropic.com");
        assert_eq!(config.rules[0].action, RuleAction::Allow);
        assert_eq!(config.rules[1].pattern, "*.evil.com");
        assert_eq!(config.rules[1].action, RuleAction::Block);
    }

    #[test]
    fn config_default_action_when_omitted() {
        let toml_str = r#"
[[rules]]
pattern = "example.com"
action = "allow"
"#;
        let config: NetworkFilterConfig = toml::from_str(toml_str).unwrap();
        // Default action should be "prompt" when not specified in config
        assert_eq!(config.default, RuleAction::Prompt);
    }
}
