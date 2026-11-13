//! Subscription selectors are resolved before API-key factories and remain sticky across restarts.
use super::{
    LocalState,
    agent::{DocSelection, TurnRecord},
    api::{ApiError, ApiResult},
};
use hickory_agent::{LlmClient, Usage, cost_usd};
use std::sync::Arc;
pub fn is_subscription(provider: &str) -> bool {
    matches!(
        provider,
        "codex-subscription" | "grok-subscription" | "claude-subscription"
    )
}
pub fn default_model(provider: &str) -> Option<&'static str> {
    match provider {
        "claude-subscription" => Some("sonnet"),
        super::codex_subscription::SELECTOR => Some(super::codex_subscription::DEFAULT_MODEL),
        super::grok_subscription::SELECTOR => Some(super::grok_subscription::DEFAULT_MODEL),
        _ => None,
    }
}
pub fn retained_choice(turns: &[TurnRecord]) -> Option<DocSelection> {
    let turn = turns
        .iter()
        .filter(|t| t.backend == "python")
        .max_by(|a, b| a.created_at.cmp(&b.created_at))?;
    is_subscription(&turn.provider).then(|| DocSelection {
        effort: None,
        provider: Some(turn.provider.clone()),
        model: Some(turn.model.clone()).filter(|m| !m.is_empty()),
    })
}
pub fn select_client(
    state: &LocalState,
    choice: &DocSelection,
    limit: Option<u32>,
) -> ApiResult<(Arc<dyn LlmClient>, String, String)> {
    if limit.is_some() {
        return Err(ApiError::unavailable(
            "Subscription inference is unavailable for private predictions; no fallback was attempted.",
        ));
    }
    let client: Arc<dyn LlmClient> = match choice.provider.as_deref() {
        Some(super::codex_subscription::SELECTOR) => {
            return super::codex_subscription::select_client(state, choice, limit);
        }
        Some("claude-subscription") => Arc::new(
            ClaudeClient::new(state, choice)
                .map_err(|e| ApiError::unavailable(format!("{e:#}")))?,
        ),
        Some(super::grok_subscription::SELECTOR) => Arc::new(
            super::grok_subscription::Client::new(state, choice.model.as_deref())
                .map_err(|e| ApiError::unavailable(format!("{e:#}")))?,
        ),
        _ => {
            return Err(ApiError::unavailable(
                "Unknown subscription backend; choose a provider explicitly",
            ));
        }
    };
    Ok((
        client.clone(),
        client.provider_name().into(),
        client.model_name().into(),
    ))
}
pub fn totals_of(turns: &[TurnRecord]) -> (Usage, Option<f64>) {
    let mut total = Usage::default();
    let mut usd = 0.0;
    let mut known = !turns.iter().any(|t| {
        t.provider.starts_with("acp:") || super::subscription::is_subscription(&t.provider)
    });
    for turn in turns {
        let Some(usage) = &turn.usage else { continue };
        total.add(usage);
        match cost_usd(&turn.model, usage) {
            Some(cost) => usd += cost,
            None if usage.is_zero() => {}
            None => known = false,
        }
    }
    (total, known.then_some(usd))
}

use anyhow::{Context, Result, ensure};
use async_trait::async_trait;
use hickory_agent::{ChatChunk, ChatStream, Message};
use serde_json::Value;
#[cfg(test)]
use serde_json::json;
use std::path::PathBuf;

/// Only the vendor CLI sees credentials. Hickory receives text for its Python loop.
struct ClaudeClient {
    binary: PathBuf,
    model: String,
    effort: Option<String>,
}
impl ClaudeClient {
    fn new(state: &LocalState, choice: &DocSelection) -> Result<Self> {
        ensure!(
            state.private_prediction.is_none() && !crate::engine::private_policy::active()?,
            "Claude subscription is unavailable for private predictions; no API fallback was attempted."
        );
        let model = choice.model.as_deref().unwrap_or("sonnet");
        ensure!(
            matches!(model, "sonnet" | "opus" | "haiku")
                || (model.len() <= 128
                    && model.starts_with("claude-")
                    && model
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))),
            "Choose a Claude model ID or sonnet, opus or haiku and retry."
        );
        Ok(Self { binary: super::acp::config::executable("claude", &super::acp::config::directory(state))
            .context("Install native Claude Code and run claude auth login with your subscription, then retry.")?,
            model: model.into(), effort: choice.effort.clone() })
    }
    fn command(&self, scratch: &std::path::Path) -> Result<tokio::process::Command> {
        let home = PathBuf::from(
            std::env::var_os("HOME")
                .context("Restart Hickory from your user account; HOME is missing.")?,
        )
        .canonicalize()?;
        let credentials = home
            .join(".claude")
            .canonicalize()
            .context("Run claude auth login on the workspace server, then retry.")?;
        ensure!(
            credentials.starts_with(&home)
                && credentials != home
                && home != std::path::Path::new("/"),
            "Claude credential directory is too broad."
        );
        ensure!(
            !std::path::Path::new("/Library/Application Support/ClaudeCode/managed-settings.json")
                .exists()
                && !std::path::Path::new("/etc/claude-code/managed-settings.json").exists(),
            "Managed Claude policy cannot yet be verified for inference-only use. Use the native Claude agent; no API fallback was attempted."
        );
        let denied = ["projects", "history.jsonl", "tasks", "shell-snapshots"]
            .map(|name| credentials.join(name));
        let mut cmd = super::subscription_process::command_with_helpers(
            &self.binary,
            scratch,
            &[credentials],
            &[home.join(".claude.json")],
            &denied,
            // Claude Code delegates macOS Keychain access to this fixed utility.
            // Native model tools remain empty; no shell/interpreter is executable.
            &[PathBuf::from("/usr/bin/security")],
        )?;
        // Do not inherit API keys, cloud-provider routing, custom config or auth tokens.
        cmd.env_clear();
        for key in [
            "PATH",
            "LANG",
            "LC_ALL",
            "HTTPS_PROXY",
            "HTTP_PROXY",
            "ALL_PROXY",
            "NO_PROXY",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
        ] {
            if let Some(value) = std::env::var_os(key) {
                cmd.env(key, value);
            }
        }
        cmd.env("HOME", home)
            .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1");
        Ok(cmd)
    }
}
fn claude_subscription_auth(value: &Value) -> bool {
    value["loggedIn"] == true
        && value["authMethod"] == "claude.ai"
        && value["subscriptionType"]
            .as_str()
            .is_some_and(|kind| !kind.is_empty())
}
fn claude_result(value: Value) -> Result<String> {
    ensure!(
        value["type"] == "result" && value["subtype"] == "success" && value["is_error"] == false,
        "Claude inference did not complete. Check subscription sign-in and usage limits, then retry; no API fallback was attempted."
    );
    Ok(value["result"]
        .as_str()
        .context("Claude returned no text; update Claude Code and retry.")?
        .into())
}
#[async_trait]
impl LlmClient for ClaudeClient {
    async fn complete(&self, messages: Vec<Message>) -> Result<String> {
        ensure!(
            messages.iter().all(|m| m.images.is_empty()),
            "Remove image attachments before using Claude subscription inference."
        );
        let scratch = tempfile::tempdir()?;
        let mut auth = self.command(scratch.path())?;
        auth.args(["auth", "status"]);
        let bytes = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            super::subscription_process::capture(auth),
        )
        .await
        .context("Claude sign-in check timed out. Run claude auth status and retry.")??;
        let status: Value = serde_json::from_slice(&bytes)?;
        ensure!(
            claude_subscription_auth(&status),
            "Run claude auth login with a Claude subscription on the workspace server. API and cloud-provider authentication are not accepted."
        );
        let mut cmd = self.command(scratch.path())?;
        cmd.args([
            "--print",
            "--output-format",
            "json",
            "--safe-mode",
            "--tools",
            "",
            "--strict-mcp-config",
            "--mcp-config",
            "{\"mcpServers\":{}}",
            "--setting-sources",
            "",
            "--settings",
            "{\"forceLoginMethod\":\"claudeai\",\"disableAllHooks\":true}",
            "--no-session-persistence",
            "--max-turns",
            "1",
            "--model",
            &self.model,
        ]);
        if let Some(effort) = &self.effort {
            cmd.args(["--effort", effort]);
        }
        let prompt = format!(
            "Continue this conversation with one assistant response. Use no native tools.\n{}",
            serde_json::to_string(&messages)?
        );
        ensure!(
            prompt.len() <= 800_000,
            "Conversation exceeds Claude input limit. Start a shorter conversation."
        );
        let bytes = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            super::subscription_process::capture_input(cmd, Some(prompt.into_bytes())),
        )
        .await
        .context("Claude timed out. Retry after checking your connection.")??;
        claude_result(serde_json::from_slice(&bytes)?)
    }
    async fn complete_stream(&self, messages: Vec<Message>) -> Result<ChatStream> {
        let delta = self.complete(messages).await?;
        Ok(Box::pin(futures::stream::once(async move {
            Ok(ChatChunk {
                delta,
                ..Default::default()
            })
        })))
    }
    fn provider_name(&self) -> &str {
        "claude-subscription"
    }
    fn model_name(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod claude_tests {
    use super::*;
    #[test]
    fn subscription_auth_and_results_fail_closed() {
        // .hickory-source/frontends/contract.md: subscription-only Python agent.
        assert!(claude_subscription_auth(
            &json!({"loggedIn":true,"authMethod":"claude.ai","subscriptionType":"max"})
        ));
        for method in [
            "api_key",
            "api_key_helper",
            "third_party",
            "oauth_token",
            "none",
        ] {
            assert!(!claude_subscription_auth(
                &json!({"loggedIn":true,"authMethod":method,"subscriptionType":"max"})
            ));
        }
        assert!(!claude_subscription_auth(
            &json!({"loggedIn":false,"authMethod":"claude.ai"})
        ));
        assert_eq!(
            claude_result(
                json!({"type":"result","subtype":"success","is_error":false,"result":"hello"})
            )
            .unwrap(),
            "hello"
        );
        assert!(
            claude_result(json!({"type":"result","subtype":"error_max_turns","is_error":true}))
                .is_err()
        );
        assert!(claude_result(json!({"type":"tool_use"})).is_err());
    }
}
