//! The coding agent loop.
//!
//! Orchestrates LLM decisions with Hick edit format and sandboxed command execution.

use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;

use futures::Stream;
use hick_agent_core::{
    CommandExecutor, HickEditFormat, HickEditService, QueryMatch, SourceEdit, WasmSandboxConfig,
    WasmSandboxExecutor,
};

use crate::llm::{ChatChunk, LlmClient, Message, Role};

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

/// AI coding agent that uses Hick XML edit format and WASM sandbox.
pub struct Agent {
    config: AgentConfig,
    llm: Box<dyn LlmClient>,
    edit_service: HickEditService,
    executor: Option<WasmSandboxExecutor>,
}

impl Agent {
    /// Create a new agent with the given configuration and LLM client.
    pub fn new(config: AgentConfig, llm: Box<dyn LlmClient>) -> Self {
        Self {
            config,
            llm,
            edit_service: HickEditService::new(),
            executor: None,
        }
    }

    /// Get the agent configuration.
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get or create the sandbox executor.
    fn executor(&mut self) -> &mut WasmSandboxExecutor {
        if self.executor.is_none() {
            self.executor = Some(WasmSandboxExecutor::new(WasmSandboxConfig {
                images_dir: self.config.images_dir.clone(),
                workdir: self.config.project_dir.clone(),
                default_image: self.config.default_image.clone(),
            }));
        }
        self.executor.as_mut().unwrap()
    }

    /// Run the Hick pipeline on project sources.
    pub async fn run_pipeline(
        &mut self,
        sources: &[(String, String)],
        params: &[(String, String)],
    ) -> anyhow::Result<HashMap<String, String>> {
        let output = self.edit_service.run_pipeline(sources, params).await?;
        Ok(output.files)
    }

    /// Execute a command in the sandbox.
    pub async fn exec(
        &mut self,
        command: &str,
        container: Option<&str>,
        image: Option<&str>,
        timeout: Option<u64>,
    ) -> anyhow::Result<String> {
        let container = container.unwrap_or("workspace");
        let image = image
            .map(String::from)
            .unwrap_or_else(|| self.config.default_image.clone());
        let exec = self.executor();
        exec.execute(container, command, &image, timeout).await
    }

    /// Query output structure.
    pub async fn query(
        &self,
        selector: &str,
        file: Option<&str>,
    ) -> anyhow::Result<Vec<QueryMatch>> {
        self.edit_service.query(selector, file).await
    }

    /// Get structure summary for an output file.
    pub async fn structure(&self, file: &str) -> anyhow::Result<String> {
        self.edit_service.structure(file).await
    }

    /// Get provenance for a line range in an output file.
    pub async fn provenance(
        &self,
        file: &str,
        start_line: usize,
        end_line: usize,
    ) -> anyhow::Result<String> {
        self.edit_service
            .provenance(file, start_line, end_line)
            .await
    }

    /// Call the LLM with messages (for ReAct loop).
    pub async fn llm_complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        self.llm.complete(messages).await
    }

    /// Stream a chat completion (for ReAct loop with real-time output).
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

    /// Compute source edits for a selector replacement.
    pub async fn compute_edits(
        &self,
        file: &str,
        selector: &str,
        new_content: &str,
    ) -> anyhow::Result<Vec<SourceEdit>> {
        self.edit_service
            .compute_edits(file, selector, new_content)
            .await
    }

    /// Apply edits to source files.
    pub fn apply_edits(
        sources: &HashMap<String, String>,
        edits: &[SourceEdit],
    ) -> HashMap<String, String> {
        <HickEditService as HickEditFormat>::apply_edits(sources, edits)
    }

    /// Run one agent turn: send context to LLM, parse response, execute tools.
    ///
    /// Returns the assistant's final response.
    pub async fn run_turn(
        &mut self,
        user_message: &str,
        system_prompt: Option<&str>,
        context: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut messages = Vec::new();

        if let Some(system) = system_prompt {
            messages.push(Message {
                role: Role::System,
                content: system.to_string(),
            });
        }

        if let Some(ctx) = context {
            messages.push(Message {
                role: Role::User,
                content: format!("Context:\n{ctx}\n\nUser request: {user_message}"),
            });
        } else {
            messages.push(Message {
                role: Role::User,
                content: user_message.to_string(),
            });
        }

        let response = self.llm.complete(messages).await?;
        Ok(response)
    }
}
