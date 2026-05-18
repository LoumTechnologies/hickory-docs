//! `SuggestionGenerator` trait and deterministic implementation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::suggestion::{PolicySuggestion, SuggestionRuleType, SuggestionScope};

// ---------------------------------------------------------------------------
// CapabilityRequest
// ---------------------------------------------------------------------------

/// Structured description of a capability that was requested by an agent.
///
/// Used as input to a [`SuggestionGenerator`] to produce [`PolicySuggestion`]
/// values at different generalization levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CapabilityRequest {
    /// An outbound network connection to a given host.
    Network { host: String },
    /// An MCP tool call on a named server.
    McpTool {
        server_name: String,
        tool_name: String,
        args_summary: Option<String>,
    },
    /// A call to an API sink slot.
    ApiSink {
        sink_type: String,
        slot_name: String,
    },
}

// ---------------------------------------------------------------------------
// SuggestionGenerator trait
// ---------------------------------------------------------------------------

/// Generates a ranked list of [`PolicySuggestion`] values from a
/// [`CapabilityRequest`].
///
/// Implementations must be `Send + Sync` so they can be shared across tokio
/// tasks. The list should contain 2–4 suggestions at different generalization
/// levels (exact → wildcard), ordered from most specific to most permissive.
#[async_trait]
pub trait SuggestionGenerator: Send + Sync {
    async fn suggestions(&self, req: &CapabilityRequest) -> Vec<PolicySuggestion>;
}

// ---------------------------------------------------------------------------
// DeterministicSuggestionGenerator
// ---------------------------------------------------------------------------

/// Produces suggestions mechanically from the request — no LLM involved.
pub struct DeterministicSuggestionGenerator;

#[async_trait]
impl SuggestionGenerator for DeterministicSuggestionGenerator {
    async fn suggestions(&self, req: &CapabilityRequest) -> Vec<PolicySuggestion> {
        match req {
            CapabilityRequest::Network { host } => {
                let mut suggestions = vec![PolicySuggestion {
                    rule_type: SuggestionRuleType::Network,
                    pattern: host.clone(),
                    scope: SuggestionScope::Exact,
                }];
                // Wildcard domain: strip one subdomain level if present
                if let Some(domain) = strip_one_subdomain(host) {
                    suggestions.push(PolicySuggestion {
                        rule_type: SuggestionRuleType::Network,
                        pattern: format!("*.{domain}"),
                        scope: SuggestionScope::WildcardDomain,
                    });
                }
                suggestions.push(PolicySuggestion {
                    rule_type: SuggestionRuleType::Network,
                    pattern: "*".to_string(),
                    scope: SuggestionScope::WildcardAll,
                });
                suggestions
            }
            CapabilityRequest::McpTool {
                server_name,
                tool_name,
                ..
            } => {
                vec![
                    PolicySuggestion {
                        rule_type: SuggestionRuleType::McpTool,
                        pattern: tool_name.clone(),
                        scope: SuggestionScope::Exact,
                    },
                    PolicySuggestion {
                        rule_type: SuggestionRuleType::McpTool,
                        pattern: server_name.clone(),
                        scope: SuggestionScope::WildcardAll,
                    },
                ]
            }
            CapabilityRequest::ApiSink { sink_type, .. } => {
                vec![PolicySuggestion {
                    rule_type: SuggestionRuleType::ApiSink,
                    pattern: sink_type.clone(),
                    scope: SuggestionScope::Exact,
                }]
            }
        }
    }
}

/// Strip the leftmost label from a hostname if it has more than two labels.
///
/// `"api.example.com"` → `Some("example.com")`
/// `"example.com"` → `None` (already a base domain)
fn strip_one_subdomain(host: &str) -> Option<String> {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() > 2 {
        Some(parts[1..].join("."))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn network_request_produces_three_suggestions() {
        let sg = DeterministicSuggestionGenerator;
        let req = CapabilityRequest::Network {
            host: "api.example.com".to_string(),
        };
        let suggestions = sg.suggestions(&req).await;
        assert_eq!(suggestions.len(), 3);
        assert_eq!(suggestions[0].scope, SuggestionScope::Exact);
        assert_eq!(suggestions[0].pattern, "api.example.com");
        assert_eq!(suggestions[1].scope, SuggestionScope::WildcardDomain);
        assert_eq!(suggestions[1].pattern, "*.example.com");
        assert_eq!(suggestions[2].scope, SuggestionScope::WildcardAll);
        assert_eq!(suggestions[2].pattern, "*");
    }

    #[tokio::test]
    async fn network_request_no_subdomain_produces_two_suggestions() {
        let sg = DeterministicSuggestionGenerator;
        let req = CapabilityRequest::Network {
            host: "example.com".to_string(),
        };
        let suggestions = sg.suggestions(&req).await;
        // No subdomain to strip, so only exact + wildcard_all
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].scope, SuggestionScope::Exact);
        assert_eq!(suggestions[1].scope, SuggestionScope::WildcardAll);
    }

    #[tokio::test]
    async fn mcp_tool_request_produces_two_suggestions() {
        let sg = DeterministicSuggestionGenerator;
        let req = CapabilityRequest::McpTool {
            server_name: "my_server".to_string(),
            tool_name: "fetch_webpage".to_string(),
            args_summary: None,
        };
        let suggestions = sg.suggestions(&req).await;
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].scope, SuggestionScope::Exact);
        assert_eq!(suggestions[0].pattern, "fetch_webpage");
        assert_eq!(suggestions[1].scope, SuggestionScope::WildcardAll);
        assert_eq!(suggestions[1].pattern, "my_server");
    }

    #[tokio::test]
    async fn api_sink_request_produces_one_suggestion() {
        let sg = DeterministicSuggestionGenerator;
        let req = CapabilityRequest::ApiSink {
            sink_type: "email".to_string(),
            slot_name: "to".to_string(),
        };
        let suggestions = sg.suggestions(&req).await;
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].scope, SuggestionScope::Exact);
        assert_eq!(suggestions[0].pattern, "email");
    }

    #[test]
    fn capability_request_serializes_and_roundtrips() {
        let req = CapabilityRequest::Network {
            host: "api.example.com".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: CapabilityRequest = serde_json::from_str(&json).unwrap();
        let CapabilityRequest::Network { host } = decoded else {
            panic!("wrong variant");
        };
        assert_eq!(host, "api.example.com");
    }
}
