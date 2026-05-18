//! Network request filtering for agents.
//!
//! Provides configurable rules that control which hosts the agent can connect
//! to. Rules can allow, block, or prompt the user for each host pattern.
//! Configuration is loaded from `.hick/network-rules.toml` in the project
//! directory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::Stream;
use hick_policy::{CapabilityRequest, DeterministicSuggestionGenerator, PolicySuggestion, SuggestionGenerator};
use hick_token::ContainerCapabilities;
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};

use crate::llm::{ChatChunk, LlmClient, Message};

/// Verdict for a filtered network request: Allow or Block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkVerdict {
    Allow,
    Block,
}

/// Response sent back through the approval channel.
///
/// `verdict` is the immediate decision; `always_allow` carries an optional
/// policy rule to persist so the same request is automatically allowed in
/// future sessions.
#[derive(Debug)]
pub struct ApprovalResponse {
    pub verdict: NetworkVerdict,
    /// If `Some`, the filter appends this rule to the appropriate config file
    /// and reloads its rules before caching the verdict.
    pub always_allow: Option<PolicySuggestion>,
}

/// Action to take when a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Block,
    Prompt,
}

/// A network access rule matching a host pattern.
#[derive(Debug, Clone, Deserialize)]
pub struct NetworkRule {
    pub pattern: String,
    pub action: RuleAction,
}

/// Configuration for the network filter, loaded from `.hick/network-rules.toml`.
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

/// Request sent to the UI to prompt the user for network access approval.
pub struct NetworkApprovalRequest {
    /// The host being accessed.
    pub host: String,
    /// Pre-generated policy suggestions (deterministic, ready for immediate display).
    pub suggestions: Vec<PolicySuggestion>,
    /// Sender for the user's response (verdict + optional always-allow rule).
    pub response_tx: oneshot::Sender<ApprovalResponse>,
}

/// Filters outbound network requests by host against configurable rules.
///
/// When a rule action is `Prompt`, sends a [`NetworkApprovalRequest`] to the
/// caller and waits for the user's decision. Verdicts are cached per host for
/// the duration of the session.
///
/// An "Always Allow" response writes the chosen policy rule to the config file
/// and reloads the in-memory rules so subsequent requests are served from the
/// rule table without prompting.
pub struct NetworkFilter {
    config: Mutex<NetworkFilterConfig>,
    /// Path to the TOML config file, used for always-allow writes and reloads.
    config_path: Option<PathBuf>,
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
            config: Mutex::new(config),
            config_path: None,
            cache: Mutex::new(HashMap::new()),
            approval_tx,
        }
    }

    /// Load filter configuration from a TOML file.
    ///
    /// Remembers the file path so that always-allow writes can append rules
    /// and trigger a live reload.
    pub fn load(
        path: &Path,
        approval_tx: mpsc::UnboundedSender<NetworkApprovalRequest>,
    ) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: NetworkFilterConfig = toml::from_str(&content)?;
        Ok(Self {
            config: Mutex::new(config),
            config_path: Some(path.to_path_buf()),
            cache: Mutex::new(HashMap::new()),
            approval_tx,
        })
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

        for rule in &capabilities.network_rules {
            if let hick_token::NetworkRule::Allow { host, .. } = rule {
                rules.push(NetworkRule {
                    pattern: host.clone(),
                    action: RuleAction::Allow,
                });
            }
        }

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
        self.config.lock().unwrap().rules.len()
    }

    /// Human-readable name for the default action.
    pub fn default_action_name(&self) -> &'static str {
        match self.config.lock().unwrap().default {
            RuleAction::Allow => "allow",
            RuleAction::Block => "block",
            RuleAction::Prompt => "prompt",
        }
    }

    /// Check if a request to the given host is allowed.
    ///
    /// For `Prompt` rules, sends a request to the approval channel and blocks
    /// until the user responds. An "Always Allow" response writes the chosen
    /// rule to the config file and reloads rules before caching.
    pub async fn check(&self, host: &str) -> NetworkVerdict {
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
        let config = self.config.lock().unwrap();
        for rule in &config.rules {
            if host_matches(&rule.pattern, host) {
                return rule.action;
            }
        }
        config.default
    }

    fn cache_verdict(&self, host: &str, verdict: NetworkVerdict) {
        self.cache.lock().unwrap().insert(host.to_string(), verdict);
    }

    async fn prompt_user(&self, host: &str) -> NetworkVerdict {
        // Eagerly generate deterministic suggestions so the UI can display them
        // immediately without waiting for an LLM call.
        let req = CapabilityRequest::Network {
            host: host.to_string(),
        };
        let suggestions = DeterministicSuggestionGenerator.suggestions(&req).await;

        let (response_tx, response_rx) = oneshot::channel();
        let request = NetworkApprovalRequest {
            host: host.to_string(),
            suggestions,
            response_tx,
        };

        if self.approval_tx.send(request).is_err() {
            log::warn!("Network approval channel closed; blocking request to {host}");
            return NetworkVerdict::Block;
        }

        match response_rx.await {
            Ok(ApprovalResponse {
                verdict,
                always_allow: Some(suggestion),
            }) => {
                self.apply_always_allow(&suggestion);
                self.cache_verdict(host, verdict);
                verdict
            }
            Ok(ApprovalResponse {
                verdict,
                always_allow: None,
            }) => {
                self.cache_verdict(host, verdict);
                verdict
            }
            Err(_) => {
                log::warn!("Network approval response dropped; blocking request to {host}");
                NetworkVerdict::Block
            }
        }
    }

    /// Persist an always-allow rule to the config file and reload rules.
    fn apply_always_allow(&self, suggestion: &PolicySuggestion) {
        let Some(ref config_path) = self.config_path else {
            log::warn!(
                "NetworkFilter: no config path set; cannot persist always-allow rule for '{}'",
                suggestion.pattern
            );
            return;
        };

        // Only network-rules.toml is managed by this filter.
        if suggestion.config_file_key() != "network-rules.toml" {
            log::warn!(
                "NetworkFilter: ignoring always-allow for non-network rule type '{}'",
                suggestion.config_file_key()
            );
            return;
        }

        let snippet = suggestion.to_toml_snippet();
        if let Err(e) = append_to_config_file(config_path, &snippet) {
            log::warn!(
                "NetworkFilter: failed to persist always-allow rule to {}: {e}",
                config_path.display()
            );
            // Fallback: rule is allowed for this session via the cache but not persisted.
            return;
        }

        self.reload_rules_from_path(config_path);
    }

    /// Re-read and apply the TOML config from the stored path.
    fn reload_rules_from_path(&self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<NetworkFilterConfig>(&content) {
                Ok(config) => {
                    *self.config.lock().unwrap() = config;
                    log::debug!(
                        "NetworkFilter: reloaded rules from {}",
                        path.display()
                    );
                }
                Err(e) => {
                    log::warn!(
                        "NetworkFilter: failed to parse reloaded config from {}: {e}",
                        path.display()
                    );
                }
            },
            Err(e) => {
                log::warn!(
                    "NetworkFilter: failed to read config from {}: {e}",
                    path.display()
                );
            }
        }
    }
}

/// Append a TOML snippet to a config file, creating parent directories as needed.
pub(crate) fn append_to_config_file(path: &Path, snippet: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    // Ensure there is a blank line before the appended snippet.
    writeln!(file)?;
    file.write_all(snippet.as_bytes())?;
    Ok(())
}

/// Check if a host pattern matches a given host.
///
/// Supports `*`, `*.example.com` (wildcard subdomain), and exact matches.
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

/// An [`LlmClient`] wrapper that checks a [`NetworkFilter`] before each API call.
///
/// If the filter blocks the request, the call returns an error. If the filter
/// prompts, the call blocks until the user responds.
pub struct FilteredLlmClient {
    inner: Box<dyn LlmClient>,
    filter: Arc<NetworkFilter>,
    host: String,
}

impl FilteredLlmClient {
    /// Wrap an existing client with a network filter for the given host.
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

    fn allow_once() -> ApprovalResponse {
        ApprovalResponse {
            verdict: NetworkVerdict::Allow,
            always_allow: None,
        }
    }

    fn block_once() -> ApprovalResponse {
        ApprovalResponse {
            verdict: NetworkVerdict::Block,
            always_allow: None,
        }
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

        let request = rx.recv().await.unwrap();
        assert_eq!(request.host, "example.com");
        assert!(!request.suggestions.is_empty());
        request.response_tx.send(allow_once()).unwrap();

        let verdict = check_handle.await.unwrap();
        assert_eq!(verdict, NetworkVerdict::Allow);

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
        request.response_tx.send(block_once()).unwrap();

        let verdict = check_handle.await.unwrap();
        assert_eq!(verdict, NetworkVerdict::Block);
    }

    #[tokio::test]
    async fn filter_prompt_fails_closed_when_channel_closed() {
        let (tx, rx) = mpsc::unbounded_channel();
        let config = NetworkFilterConfig {
            default: RuleAction::Prompt,
            rules: vec![],
        };
        let filter = NetworkFilter::new(config, tx);

        drop(rx);

        assert_eq!(filter.check("example.com").await, NetworkVerdict::Block);
    }

    #[tokio::test]
    async fn always_allow_writes_to_config_and_reloads() {
        use hick_policy::{PolicySuggestion, SuggestionRuleType, SuggestionScope};
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("network-rules.toml");

        // Write a minimal config that defaults to Prompt
        std::fs::write(&config_path, "default = \"prompt\"\n").unwrap();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let filter = Arc::new(NetworkFilter::load(&config_path, tx).unwrap());

        let filter_clone = filter.clone();
        let handle = tokio::spawn(async move { filter_clone.check("api.example.com").await });

        let request = rx.recv().await.unwrap();
        assert_eq!(request.host, "api.example.com");

        // User picks "Always Allow" with an exact-host suggestion
        let suggestion = PolicySuggestion {
            rule_type: SuggestionRuleType::Network,
            pattern: "api.example.com".to_string(),
            scope: SuggestionScope::Exact,
        };
        request
            .response_tx
            .send(ApprovalResponse {
                verdict: NetworkVerdict::Allow,
                always_allow: Some(suggestion),
            })
            .unwrap();

        let verdict = handle.await.unwrap();
        assert_eq!(verdict, NetworkVerdict::Allow);

        // Config file should now contain the new rule
        let written = std::fs::read_to_string(&config_path).unwrap();
        assert!(written.contains("pattern = \"api.example.com\""));
        assert!(written.contains("action = \"allow\""));

        // A second request should bypass the prompt because the rule is live
        assert_eq!(
            filter.check("api.example.com").await,
            NetworkVerdict::Allow
        );
        assert!(rx.try_recv().is_err(), "no new prompt should fire");
    }

    #[tokio::test]
    async fn always_allow_write_failure_allows_session_only() {
        use hick_policy::{PolicySuggestion, SuggestionRuleType, SuggestionScope};

        // Use a path inside a non-existent directory that cannot be created
        // (simulate write failure by pointing to a path under a file, not a dir).
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Create a filter with a config_path that points to a location where
        // the parent is a file (not a dir), making writes fail.
        let config = NetworkFilterConfig {
            default: RuleAction::Prompt,
            rules: vec![],
        };
        let mut filter = NetworkFilter::new(config, tx);
        // Point config_path to an impossible location
        filter.config_path = Some(std::path::PathBuf::from("/dev/null/network-rules.toml"));
        let filter = Arc::new(filter);

        let filter_clone = filter.clone();
        let handle = tokio::spawn(async move { filter_clone.check("example.com").await });

        let request = rx.recv().await.unwrap();
        let suggestion = PolicySuggestion {
            rule_type: SuggestionRuleType::Network,
            pattern: "example.com".to_string(),
            scope: SuggestionScope::Exact,
        };
        request
            .response_tx
            .send(ApprovalResponse {
                verdict: NetworkVerdict::Allow,
                always_allow: Some(suggestion),
            })
            .unwrap();

        // Despite the write failure, the verdict is still Allow for this session
        let verdict = handle.await.unwrap();
        assert_eq!(verdict, NetworkVerdict::Allow);

        // Second request uses the cache (session-only allow)
        assert_eq!(filter.check("example.com").await, NetworkVerdict::Allow);
        assert!(rx.try_recv().is_err());
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
        assert_eq!(config.default, RuleAction::Prompt);
    }
}
