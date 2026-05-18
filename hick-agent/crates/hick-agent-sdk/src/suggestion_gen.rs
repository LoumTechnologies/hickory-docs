//! LLM-backed suggestion generator for the Always Allow flow.

use async_trait::async_trait;
use hick_policy::{
    CapabilityRequest, DeterministicSuggestionGenerator, PolicySuggestion, SuggestionGenerator,
    SuggestionRuleType, SuggestionScope,
};
use serde::Deserialize;

use crate::llm::{LlmClient, Message, Role};

// ---------------------------------------------------------------------------
// LlmSuggestionGenerator
// ---------------------------------------------------------------------------

/// LLM-backed [`SuggestionGenerator`] that produces suggestions from a
/// capability request using a chat completion call.
///
/// # Privacy and correctness guarantee
///
/// The LLM call carries **only** the serialized [`CapabilityRequest`] — no
/// conversation history, no agent task context. This ensures suggestions are
/// based solely on the capability being requested.
///
/// Falls back to [`DeterministicSuggestionGenerator`] output if the LLM call
/// fails or returns unparseable JSON.
pub struct LlmSuggestionGenerator {
    client: Box<dyn LlmClient>,
}

#[derive(Deserialize)]
struct RawSuggestion {
    pattern: String,
    scope: String,
}

impl LlmSuggestionGenerator {
    /// Create a generator backed by the given LLM client.
    pub fn new(client: Box<dyn LlmClient>) -> Self {
        Self { client }
    }

    fn rule_type_for(req: &CapabilityRequest) -> SuggestionRuleType {
        match req {
            CapabilityRequest::Network { .. } => SuggestionRuleType::Network,
            CapabilityRequest::McpTool { .. } => SuggestionRuleType::McpTool,
            CapabilityRequest::ApiSink { .. } => SuggestionRuleType::ApiSink,
        }
    }

    fn parse_scope(s: &str) -> Option<SuggestionScope> {
        match s {
            "exact" => Some(SuggestionScope::Exact),
            "wildcard_domain" => Some(SuggestionScope::WildcardDomain),
            "wildcard_path" => Some(SuggestionScope::WildcardPath),
            "wildcard_all" => Some(SuggestionScope::WildcardAll),
            _ => None,
        }
    }
}

#[async_trait]
impl SuggestionGenerator for LlmSuggestionGenerator {
    async fn suggestions(&self, req: &CapabilityRequest) -> Vec<PolicySuggestion> {
        let fallback = DeterministicSuggestionGenerator;

        let Ok(req_json) = serde_json::to_string(req) else {
            return fallback.suggestions(req).await;
        };

        let prompt = format!(
            "Generate 2-4 policy rule suggestions for this capability request at different \
             generalization levels (most specific to most permissive).\n\
             Request: {req_json}\n\
             Return a JSON array of objects with fields:\n\
             - \"pattern\": the rule pattern string\n\
             - \"scope\": one of exact, wildcard_domain, wildcard_path, wildcard_all\n\
             Return ONLY valid JSON, no other text."
        );

        // IMPORTANT: send only the capability request — no conversation history,
        // no agent task context.
        let messages = vec![Message {
            role: Role::User,
            content: prompt,
        }];

        let Ok(response) = self.client.complete(messages).await else {
            return fallback.suggestions(req).await;
        };

        let trimmed = response.trim();
        // Accept response that starts with '[' directly or after whitespace/markdown.
        let json_start = trimmed.find('[').unwrap_or(usize::MAX);
        if json_start == usize::MAX {
            return fallback.suggestions(req).await;
        }

        let Ok(items) = serde_json::from_str::<Vec<RawSuggestion>>(&trimmed[json_start..]) else {
            return fallback.suggestions(req).await;
        };

        if items.is_empty() {
            return fallback.suggestions(req).await;
        }

        let rule_type = Self::rule_type_for(req);
        let parsed: Vec<PolicySuggestion> = items
            .into_iter()
            .filter_map(|item| {
                Some(PolicySuggestion {
                    rule_type: rule_type.clone(),
                    pattern: item.pattern,
                    scope: Self::parse_scope(&item.scope)?,
                })
            })
            .collect();

        if parsed.is_empty() {
            fallback.suggestions(req).await
        } else {
            parsed
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatChunk, LlmClient, Message};
    use async_trait::async_trait;
    use futures::Stream;
    use std::pin::Pin;

    struct MockLlmClient {
        response: String,
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
            Ok(self.response.clone())
        }

        async fn complete_stream(
            &self,
            _messages: Vec<Message>,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
            unimplemented!("not needed for suggestion tests")
        }

        fn provider_name(&self) -> &str {
            "mock"
        }

        fn model_name(&self) -> &str {
            "mock-model"
        }
    }

    struct FailingLlmClient;

    #[async_trait]
    impl LlmClient for FailingLlmClient {
        async fn complete(&self, _messages: Vec<Message>) -> anyhow::Result<String> {
            anyhow::bail!("simulated LLM failure")
        }

        async fn complete_stream(
            &self,
            _messages: Vec<Message>,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<ChatChunk>> + Send>>> {
            unimplemented!()
        }

        fn provider_name(&self) -> &str {
            "mock"
        }

        fn model_name(&self) -> &str {
            "mock-fail"
        }
    }

    #[tokio::test]
    async fn valid_llm_response_produces_suggestions() {
        let json = r#"[
            {"pattern": "api.example.com", "scope": "exact"},
            {"pattern": "*.example.com", "scope": "wildcard_domain"},
            {"pattern": "*", "scope": "wildcard_all"}
        ]"#;
        let sg = LlmSuggestionGenerator::new(Box::new(MockLlmClient {
            response: json.to_string(),
        }));
        let req = CapabilityRequest::Network {
            host: "api.example.com".to_string(),
        };
        let suggestions = sg.suggestions(&req).await;
        assert_eq!(suggestions.len(), 3);
        assert_eq!(suggestions[0].pattern, "api.example.com");
        assert_eq!(suggestions[0].scope, SuggestionScope::Exact);
        assert_eq!(suggestions[1].scope, SuggestionScope::WildcardDomain);
        assert_eq!(suggestions[2].scope, SuggestionScope::WildcardAll);
    }

    #[tokio::test]
    async fn invalid_json_falls_back_to_deterministic() {
        let sg = LlmSuggestionGenerator::new(Box::new(MockLlmClient {
            response: "not valid json at all".to_string(),
        }));
        let req = CapabilityRequest::Network {
            host: "api.example.com".to_string(),
        };
        let suggestions = sg.suggestions(&req).await;
        // DeterministicSuggestionGenerator produces 3 for api.example.com
        assert_eq!(suggestions.len(), 3);
        assert_eq!(suggestions[0].pattern, "api.example.com");
    }

    #[tokio::test]
    async fn llm_failure_falls_back_to_deterministic() {
        let sg = LlmSuggestionGenerator::new(Box::new(FailingLlmClient));
        let req = CapabilityRequest::Network {
            host: "api.example.com".to_string(),
        };
        let suggestions = sg.suggestions(&req).await;
        assert_eq!(suggestions.len(), 3);
    }

    #[tokio::test]
    async fn empty_array_falls_back_to_deterministic() {
        let sg = LlmSuggestionGenerator::new(Box::new(MockLlmClient {
            response: "[]".to_string(),
        }));
        let req = CapabilityRequest::McpTool {
            server_name: "my_server".to_string(),
            tool_name: "fetch_webpage".to_string(),
            args_summary: None,
        };
        let suggestions = sg.suggestions(&req).await;
        // DeterministicSuggestionGenerator produces 2 for McpTool
        assert_eq!(suggestions.len(), 2);
    }
}
