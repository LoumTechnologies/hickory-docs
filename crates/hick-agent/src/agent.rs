//! The coding agent: LLM client + configuration.
//!
//! Simplified for the script-first model. The agent holds the LLM client and
//! project configuration. Pipeline execution is handled transparently by the
//! PipelineWatcher, and all tool operations are available to the LLM via
//! Python scripts using the `hickory` SDK.

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use hick_token::{CapabilityToken, TokenAuthority};

use crate::llm::{ChatChunk, LlmClient, Message};

/// Configuration for the coding agent.
#[derive(Clone)]
pub struct AgentConfig {
    /// Directory containing `.hick` source files.
    pub project_dir: PathBuf,
    /// Directory containing `.wasm` container images.
    pub images_dir: PathBuf,
    /// Default container image for command execution.
    pub default_image: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            project_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            images_dir: PathBuf::from("."),
            default_image: "alpine".to_string(),
        }
    }
}

/// AI coding agent — LLM interface + configuration.
///
/// In the script-first model, the agent is thin: it wraps an LLM client and
/// holds project configuration. The ReactLoop handles script extraction and
/// execution, the PipelineWatcher handles transparent pipeline reruns, and
/// the `hickory` Python SDK provides structural operations inside scripts.
pub struct Agent {
    config: AgentConfig,
    llm: Box<dyn LlmClient>,
    /// Token authority for minting and verifying capability tokens.
    token_authority: Option<Arc<TokenAuthority>>,
    /// Root capability token for this agent session.
    session_token: Option<Arc<CapabilityToken>>,
}

impl Agent {
    /// Create a new agent with the given configuration and LLM client.
    pub fn new(config: AgentConfig, llm: Box<dyn LlmClient>) -> Self {
        Self {
            config,
            llm,
            token_authority: None,
            session_token: None,
        }
    }

    /// Set the token authority and session token for capability enforcement.
    pub fn with_security(
        mut self,
        authority: Arc<TokenAuthority>,
        token: Arc<CapabilityToken>,
    ) -> Self {
        self.token_authority = Some(authority);
        self.session_token = Some(token);
        self
    }

    /// Get the agent configuration.
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get the token authority (if configured).
    pub fn token_authority(&self) -> Option<&Arc<TokenAuthority>> {
        self.token_authority.as_ref()
    }

    /// Get the session capability token (if configured).
    pub fn session_token(&self) -> Option<&Arc<CapabilityToken>> {
        self.session_token.as_ref()
    }

    /// Call the LLM with messages (full response).
    pub async fn llm_complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        self.llm.complete(messages).await
    }

    /// Stream a chat completion (for real-time output).
    pub async fn llm_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
        self.llm.complete_stream(messages).await
    }

    /// Provider name (for display).
    pub fn provider_name(&self) -> &str {
        self.llm.provider_name()
    }

    /// Model name (for display).
    pub fn model_name(&self) -> &str {
        self.llm.model_name()
    }
}
