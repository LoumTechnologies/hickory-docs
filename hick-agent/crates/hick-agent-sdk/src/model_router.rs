//! Role-based model routing.

use std::collections::HashMap;
use std::sync::Arc;

use crate::llm::LlmClient;

/// The role for which a model client is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelRole {
    Default,
    Smol,
    Slow,
    Plan,
    Commit,
}

/// Routes model roles to their registered [`LlmClient`]s.
///
/// Falls back to the `Default` client for any role that has no registered client.
pub struct ModelRouter {
    clients: HashMap<ModelRole, Arc<dyn LlmClient>>,
}

impl ModelRouter {
    /// Create a router with only the `Default` role configured.
    pub fn new(default: Arc<dyn LlmClient>) -> Self {
        let mut clients = HashMap::new();
        clients.insert(ModelRole::Default, default);
        Self { clients }
    }

    /// Register a client for a specific role (builder pattern).
    pub fn with_role(mut self, role: ModelRole, client: Arc<dyn LlmClient>) -> Self {
        self.clients.insert(role, client);
        self
    }

    /// Return the client for `role`, falling back to the `Default` client.
    pub fn get(&self, role: ModelRole) -> &Arc<dyn LlmClient> {
        self.clients
            .get(&role)
            .unwrap_or_else(|| {
                self.clients
                    .get(&ModelRole::Default)
                    .expect("ModelRouter always has a Default client")
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatChunk, Message};
    use async_trait::async_trait;
    use futures::Stream;
    use std::pin::Pin;

    struct StubClient(&'static str);

    #[async_trait]
    impl LlmClient for StubClient {
        async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
            Ok(self.0.to_string())
        }
        async fn complete_stream(
            &self,
            _messages: Vec<Message>,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
            let s = futures::stream::once(async {
                Ok(ChatChunk { delta: String::new(), finish_reason: Some("stop".into()) })
            });
            Ok(Box::pin(s))
        }
        fn provider_name(&self) -> &str { "stub" }
        fn model_name(&self) -> &str { self.0 }
    }

    #[test]
    fn get_default_returns_default_client() {
        let router = ModelRouter::new(Arc::new(StubClient("default")));
        assert_eq!(router.get(ModelRole::Default).model_name(), "default");
    }

    #[test]
    fn get_smol_returns_smol_client_when_registered() {
        let router = ModelRouter::new(Arc::new(StubClient("default")))
            .with_role(ModelRole::Smol, Arc::new(StubClient("smol")));
        assert_eq!(router.get(ModelRole::Smol).model_name(), "smol");
    }

    #[test]
    fn get_smol_falls_back_to_default_when_not_registered() {
        let router = ModelRouter::new(Arc::new(StubClient("default")));
        assert_eq!(router.get(ModelRole::Smol).model_name(), "default");
    }
}
