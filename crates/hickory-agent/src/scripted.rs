//! A canned [`LlmClient`] that replays scripted responses — no network.
//!
//! Used by tests (and offline development) to drive the ReAct loop
//! deterministically: each `complete` call pops the next scripted response.

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::llm::{LlmClient, Message};

/// An [`LlmClient`] that returns pre-scripted responses in order.
pub struct ScriptedLlmClient {
    responses: Mutex<VecDeque<String>>,
}

impl ScriptedLlmClient {
    /// Create with the responses to replay, in order.
    pub fn new(responses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().map(Into::into).collect()),
        }
    }
}

#[async_trait]
impl LlmClient for ScriptedLlmClient {
    async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("ScriptedLlmClient ran out of responses"))
    }

    fn provider_name(&self) -> &str {
        "scripted"
    }

    fn model_name(&self) -> &str {
        "scripted"
    }
}
