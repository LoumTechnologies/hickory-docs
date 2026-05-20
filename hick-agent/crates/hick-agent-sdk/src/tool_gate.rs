//! MCP tool call gating with classification-aware allow/block/prompt verdicts.
//!
//! [`ToolGate`] is the security enforcement layer for MCP tool calls. Every
//! call to an MCP tool — regardless of exposure mode — must pass through
//! `ToolGate::check` before reaching `McpClient::call`.
//!
//! The design mirrors [`crate::NetworkFilter`]: the same allow/block/prompt
//! verdict pattern, an approval channel, and a per-session verdict cache.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use hick_classify::Classification;
use hick_policy::{CapabilityRequest, DeterministicSuggestionGenerator, PolicySuggestion, SuggestionGenerator};
use tokio::sync::{mpsc, oneshot};

use crate::agentignore::AgentIgnoreFilter;
use crate::network_filter::{ApprovalResponse, NetworkVerdict};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Trust level for an MCP server, used by [`ToolGate`] to determine gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolTrust {
    /// All non-blocked calls are allowed without prompting.
    Trusted,
    /// Any rejection classification present in scope blocks the call.
    Untrusted,
    /// The user is always prompted regardless of scope classifications.
    Prompt,
}

/// Verdict returned by [`ToolGate::check`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolVerdict {
    /// The tool call is permitted.
    Allow,
    /// The tool call is denied.
    Block,
}

/// Request sent to the UI when [`ToolGate`] needs the user's decision.
pub struct ToolApprovalRequest {
    /// Name of the MCP server owning the tool.
    pub server_name: String,
    /// Name of the tool being invoked.
    pub tool_name: String,
    /// Pre-generated policy suggestions (deterministic, ready for immediate display).
    pub suggestions: Vec<PolicySuggestion>,
    /// Sender for the user's response (verdict + optional always-allow rule).
    pub response_tx: oneshot::Sender<ApprovalResponse>,
}

/// Configuration for [`ToolGate`], typically produced by
/// [`crate::AgentSecurityConfig::to_tool_gate_config`].
#[derive(Debug, Clone, Default)]
pub struct ToolGateConfig {
    /// Classifications whose presence in scope blocks all MCP tool calls,
    /// regardless of server trust level.
    pub rejection_classifications: Vec<Classification>,
}

/// Governs every MCP tool call with classification-aware gating.
///
/// # Verdict rules (in priority order)
///
/// 1. If any scope classification matches a rejection criterion, **Block**
///    (regardless of server trust level).
/// 2. If the server is `Untrusted` and any scope classification matches a
///    rejection criterion, **Block** (already covered by rule 1, but explicit
///    for clarity).
/// 3. If the server is `Prompt` and no rejection classification is present,
///    send a [`ToolApprovalRequest`] and **wait** for the user's decision.
/// 4. If the server is `Trusted` and no rejection classification is present,
///    **Allow**.
///
/// Verdicts are cached per `(server_name, tool_name)` pair for the session.
/// An "Always Allow" response writes the chosen rule to `tool-rules.toml` in
/// `config_dir` (if set) so the rule persists across sessions.
pub struct ToolGate {
    config: ToolGateConfig,
    server_trust: HashMap<String, ToolTrust>,
    default_trust: ToolTrust,
    cache: Mutex<HashMap<(String, String), ToolVerdict>>,
    approval_tx: mpsc::UnboundedSender<ToolApprovalRequest>,
    /// Directory where `tool-rules.toml` should be written on always-allow.
    pub config_dir: Option<PathBuf>,
    ignore_filter: Option<AgentIgnoreFilter>,
}

fn network_verdict_to_tool(v: NetworkVerdict) -> ToolVerdict {
    match v {
        NetworkVerdict::Allow => ToolVerdict::Allow,
        NetworkVerdict::Block => ToolVerdict::Block,
    }
}

impl ToolGate {
    /// Create a new gate with the given configuration.
    ///
    /// * `config` — rejection classifications from `AgentSecurityConfig`.
    /// * `server_trust` — trust level per server name (from `McpServersConfig`).
    /// * `default_trust` — trust level for servers not in `server_trust`.
    /// * `approval_tx` — channel for prompting the user.
    pub fn new(
        config: ToolGateConfig,
        server_trust: HashMap<String, ToolTrust>,
        default_trust: ToolTrust,
        approval_tx: mpsc::UnboundedSender<ToolApprovalRequest>,
    ) -> Self {
        Self {
            config,
            server_trust,
            default_trust,
            cache: Mutex::new(HashMap::new()),
            approval_tx,
            config_dir: None,
            ignore_filter: None,
        }
    }

    /// Set the directory where `tool-rules.toml` should be written on always-allow.
    pub fn with_config_dir(mut self, dir: PathBuf) -> Self {
        self.config_dir = Some(dir);
        self
    }

    /// Attach an [`AgentIgnoreFilter`] so that [`ToolGate::check_path`] can
    /// block file-read requests whose paths match `.agentignore` patterns.
    pub fn with_ignore_filter(mut self, filter: AgentIgnoreFilter) -> Self {
        self.ignore_filter = Some(filter);
        self
    }

    /// Block a file-read request if the path matches `.agentignore` patterns.
    ///
    /// Returns [`ToolVerdict::Block`] when the path is ignored, otherwise
    /// [`ToolVerdict::Allow`]. Call this before any file-read MCP tool call
    /// to enforce the project's `.agentignore` restrictions.
    pub fn check_path(&self, path: &Path) -> ToolVerdict {
        match &self.ignore_filter {
            Some(filter) if filter.is_ignored(path) => ToolVerdict::Block,
            _ => ToolVerdict::Allow,
        }
    }

    /// Check whether the tool call is permitted.
    ///
    /// Returns [`ToolVerdict::Block`] immediately if any scope classification
    /// matches a configured rejection criterion. If trust level is `Prompt`
    /// and no rejection is present, waits for the user's decision via the
    /// approval channel. Verdicts are cached per `(server, tool)` pair.
    pub async fn check(
        &self,
        server: &str,
        tool: &str,
        scope_classifications: &[Classification],
    ) -> ToolVerdict {
        // Rule 1 & 2: rejection classification always wins.
        if self.any_rejected(scope_classifications) {
            log::debug!(
                "ToolGate: blocking call to {server}/{tool} — rejection classification in scope"
            );
            return ToolVerdict::Block;
        }

        // Check the per-session cache.
        let cache_key = (server.to_string(), tool.to_string());
        if let Some(&verdict) = self.cache.lock().unwrap().get(&cache_key) {
            return verdict;
        }

        let trust = self
            .server_trust
            .get(server)
            .copied()
            .unwrap_or(self.default_trust);

        match trust {
            ToolTrust::Trusted => {
                self.cache_verdict(cache_key, ToolVerdict::Allow);
                ToolVerdict::Allow
            }
            ToolTrust::Untrusted => {
                // No rejection classification present (already checked above),
                // but we still cache the allow for consistency with Trusted.
                self.cache_verdict(cache_key, ToolVerdict::Allow);
                ToolVerdict::Allow
            }
            ToolTrust::Prompt => self.prompt_user(server, tool, cache_key).await,
        }
    }

    fn any_rejected(&self, scope_classifications: &[Classification]) -> bool {
        scope_classifications.iter().any(|sc| {
            self.config.rejection_classifications.iter().any(|rc| {
                // A rejection criterion matches if any of its labels appears
                // in the scope classification's label set.
                rc.labels().iter().any(|label| sc.labels().contains(label))
            })
        })
    }

    fn cache_verdict(&self, key: (String, String), verdict: ToolVerdict) {
        self.cache.lock().unwrap().insert(key, verdict);
    }

    async fn prompt_user(
        &self,
        server: &str,
        tool: &str,
        cache_key: (String, String),
    ) -> ToolVerdict {
        // Eagerly generate deterministic suggestions for the UI.
        let req = CapabilityRequest::McpTool {
            server_name: server.to_string(),
            tool_name: tool.to_string(),
            args_summary: None,
        };
        let suggestions = DeterministicSuggestionGenerator.suggestions(&req).await;

        let (response_tx, response_rx) = oneshot::channel();
        let request = ToolApprovalRequest {
            server_name: server.to_string(),
            tool_name: tool.to_string(),
            suggestions,
            response_tx,
        };

        if self.approval_tx.send(request).is_err() {
            log::warn!(
                "ToolGate: approval channel closed; blocking call to {server}/{tool}"
            );
            return ToolVerdict::Block;
        }

        match response_rx.await {
            Ok(ApprovalResponse {
                verdict,
                always_allow: Some(suggestion),
            }) => {
                self.apply_always_allow(&suggestion);
                let tool_verdict = network_verdict_to_tool(verdict);
                self.cache_verdict(cache_key, tool_verdict);
                tool_verdict
            }
            Ok(ApprovalResponse {
                verdict,
                always_allow: None,
            }) => {
                let tool_verdict = network_verdict_to_tool(verdict);
                self.cache_verdict(cache_key, tool_verdict);
                tool_verdict
            }
            Err(_) => {
                log::warn!(
                    "ToolGate: approval response dropped; blocking call to {server}/{tool}"
                );
                ToolVerdict::Block
            }
        }
    }

    fn apply_always_allow(&self, suggestion: &PolicySuggestion) {
        let Some(ref config_dir) = self.config_dir else {
            log::warn!(
                "ToolGate: no config_dir set; cannot persist always-allow rule for '{}'",
                suggestion.pattern
            );
            return;
        };
        let config_path = config_dir.join(suggestion.config_file_key());
        let snippet = suggestion.to_toml_snippet();
        if let Err(e) = crate::network_filter::append_to_config_file(&config_path, &snippet) {
            log::warn!(
                "ToolGate: failed to persist always-allow rule to {}: {e}",
                config_path.display()
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn secret_classification() -> Classification {
        Classification::new(vec![json!({"sensitivity": "secret"})])
    }

    fn low_classification() -> Classification {
        Classification::new(vec![json!({"sensitivity": "low"})])
    }

    fn gate_with_secret_rejection() -> (ToolGate, mpsc::UnboundedReceiver<ToolApprovalRequest>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig {
            rejection_classifications: vec![Classification::new(vec![
                json!({"sensitivity": "secret"}),
            ])],
        };
        let gate = ToolGate::new(config, HashMap::new(), ToolTrust::Prompt, tx);
        (gate, rx)
    }

    #[tokio::test]
    async fn block_on_secret_classification() {
        let (gate, _rx) = gate_with_secret_rejection();
        let verdict = gate
            .check("myserver", "my_tool", &[secret_classification()])
            .await;
        assert_eq!(verdict, ToolVerdict::Block);
    }

    #[tokio::test]
    async fn block_regardless_of_trust_when_secret_present() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig {
            rejection_classifications: vec![Classification::new(vec![
                json!({"sensitivity": "secret"}),
            ])],
        };
        let mut trust = HashMap::new();
        trust.insert("trusted_server".into(), ToolTrust::Trusted);
        let gate = ToolGate::new(config, trust, ToolTrust::Prompt, tx);

        // Even a Trusted server is blocked when secret data is in scope.
        let verdict = gate
            .check("trusted_server", "any_tool", &[secret_classification()])
            .await;
        assert_eq!(verdict, ToolVerdict::Block);
    }

    #[tokio::test]
    async fn block_untrusted_server_with_classification() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig {
            rejection_classifications: vec![Classification::new(vec![
                json!({"sensitivity": "secret"}),
            ])],
        };
        let mut trust = HashMap::new();
        trust.insert("evil_server".into(), ToolTrust::Untrusted);
        let gate = ToolGate::new(config, trust, ToolTrust::Prompt, tx);

        let verdict = gate
            .check("evil_server", "some_tool", &[secret_classification()])
            .await;
        assert_eq!(verdict, ToolVerdict::Block);
    }

    #[tokio::test]
    async fn allow_trusted_server_without_rejection_classification() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig {
            rejection_classifications: vec![Classification::new(vec![
                json!({"sensitivity": "secret"}),
            ])],
        };
        let mut trust = HashMap::new();
        trust.insert("safe_server".into(), ToolTrust::Trusted);
        let gate = ToolGate::new(config, trust, ToolTrust::Prompt, tx);

        let verdict = gate
            .check("safe_server", "read_file", &[low_classification()])
            .await;
        assert_eq!(verdict, ToolVerdict::Allow);
    }

    #[tokio::test]
    async fn prompt_for_unknown_server() {
        let (gate, mut rx) = gate_with_secret_rejection();

        let gate = std::sync::Arc::new(gate);
        let gate_clone = gate.clone();

        // Run the check in a background task so we can respond on rx.
        let check_handle = tokio::spawn(async move {
            gate_clone
                .check("unknown_server", "some_tool", &[low_classification()])
                .await
        });

        // The gate should send a prompt request.
        let request = rx.recv().await.unwrap();
        assert_eq!(request.server_name, "unknown_server");
        assert_eq!(request.tool_name, "some_tool");
        request
            .response_tx
            .send(ApprovalResponse {
                verdict: NetworkVerdict::Allow,
                always_allow: None,
            })
            .unwrap();

        let verdict = check_handle.await.unwrap();
        assert_eq!(verdict, ToolVerdict::Allow);
    }

    #[tokio::test]
    async fn cache_reuse_skips_prompt() {
        let (gate, mut rx) = gate_with_secret_rejection();
        let gate = std::sync::Arc::new(gate);

        // First call: prompts the user.
        let gate_clone = gate.clone();
        let handle1 = tokio::spawn(async move {
            gate_clone
                .check("prompt_server", "tool_a", &[low_classification()])
                .await
        });
        let req = rx.recv().await.unwrap();
        req.response_tx
            .send(ApprovalResponse {
                verdict: NetworkVerdict::Allow,
                always_allow: None,
            })
            .unwrap();
        assert_eq!(handle1.await.unwrap(), ToolVerdict::Allow);

        // Second call for the same (server, tool): should use the cache, no prompt.
        let verdict = gate
            .check("prompt_server", "tool_a", &[low_classification()])
            .await;
        assert_eq!(verdict, ToolVerdict::Allow);
        // No new request should be pending.
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn channel_closed_blocks_safely() {
        let (tx, rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig::default();
        let gate = ToolGate::new(config, HashMap::new(), ToolTrust::Prompt, tx);

        // Close the receiver to simulate a dead approval channel.
        drop(rx);

        let verdict = gate.check("server", "tool", &[low_classification()]).await;
        assert_eq!(verdict, ToolVerdict::Block);
    }

    #[tokio::test]
    async fn block_when_scope_has_multiple_classifications_and_one_is_secret() {
        let (gate, _rx) = gate_with_secret_rejection();
        let scope = vec![low_classification(), secret_classification()];
        let verdict = gate.check("server", "tool", &scope).await;
        assert_eq!(verdict, ToolVerdict::Block);
    }

    #[tokio::test]
    async fn allow_empty_scope() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let config = ToolGateConfig {
            rejection_classifications: vec![Classification::new(vec![
                json!({"sensitivity": "secret"}),
            ])],
        };
        let mut trust = HashMap::new();
        trust.insert("myserver".into(), ToolTrust::Trusted);
        let gate = ToolGate::new(config, trust, ToolTrust::Prompt, tx);

        let verdict = gate.check("myserver", "tool", &[]).await;
        assert_eq!(verdict, ToolVerdict::Allow);
    }

    // -----------------------------------------------------------------------
    // check_path tests (task 38)
    // -----------------------------------------------------------------------

    fn gate_with_ignore_filter(filter: crate::agentignore::AgentIgnoreFilter) -> ToolGate {
        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = ToolGate::new(ToolGateConfig::default(), HashMap::new(), ToolTrust::Trusted, tx);
        gate.with_ignore_filter(filter)
    }

    #[test]
    fn check_path_blocks_ignored_path() {
        let filter = crate::agentignore::AgentIgnoreFilter::from_str_content(".env\n");
        let gate = gate_with_ignore_filter(filter);
        assert_eq!(gate.check_path(Path::new(".env")), ToolVerdict::Block);
    }

    #[test]
    fn check_path_allows_non_ignored_path() {
        let filter = crate::agentignore::AgentIgnoreFilter::from_str_content(".env\n");
        let gate = gate_with_ignore_filter(filter);
        assert_eq!(gate.check_path(Path::new("src/main.rs")), ToolVerdict::Allow);
    }

    #[test]
    fn check_path_allows_all_when_no_filter() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let gate = ToolGate::new(ToolGateConfig::default(), HashMap::new(), ToolTrust::Trusted, tx);
        assert_eq!(gate.check_path(Path::new(".env")), ToolVerdict::Allow);
    }
}
