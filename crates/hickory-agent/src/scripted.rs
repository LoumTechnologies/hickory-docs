//! A canned [`LlmClient`] that replays scripted responses — no network.
//!
//! Used by tests (and offline development) to drive the ReAct loop
//! deterministically: each `complete` call pops the next scripted response.
//! Responses can carry fake [`Usage`] so cost-accounting plumbing is
//! testable offline (e.g. asserting `cache_read_input_tokens` propagates
//! into events on turn 2).

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::llm::{ChatChunk, ChatStream, LlmClient, Message};
use crate::usage::Usage;

/// An [`LlmClient`] that returns pre-scripted responses in order.
pub struct ScriptedLlmClient {
    responses: Mutex<VecDeque<(String, Usage)>>,
    model: String,
}

impl ScriptedLlmClient {
    /// Create with the responses to replay, in order (zero usage).
    pub fn new(responses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            responses: Mutex::new(
                responses
                    .into_iter()
                    .map(|r| (r.into(), Usage::default()))
                    .collect(),
            ),
            model: "scripted".into(),
        }
    }

    /// Create with responses that each report a fake [`Usage`].
    pub fn with_usages(responses: impl IntoIterator<Item = (impl Into<String>, Usage)>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().map(|(r, u)| (r.into(), u)).collect()),
            model: "scripted".into(),
        }
    }

    /// Report a different model name (e.g. to exercise a real price-table
    /// entry in cost tests).
    pub fn with_model_name(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    fn pop(&self) -> anyhow::Result<(String, Usage)> {
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("ScriptedLlmClient ran out of responses"))
    }
}

#[async_trait]
impl LlmClient for ScriptedLlmClient {
    async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
        Ok(self.pop()?.0)
    }

    async fn complete_with_usage(
        &self,
        _messages: Vec<Message>,
    ) -> anyhow::Result<(String, Usage)> {
        self.pop()
    }

    async fn complete_stream(&self, _messages: Vec<Message>) -> anyhow::Result<ChatStream> {
        let (text, usage) = self.pop()?;
        let stream = futures::stream::once(async move {
            Ok(ChatChunk {
                delta: text,
                finish_reason: Some("stop".into()),
                usage: Some(usage),
            })
        });
        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        "scripted"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}
