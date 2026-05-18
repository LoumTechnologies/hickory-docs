//! Structured policy edit suggestions presented to the user on capability prompts.

use serde::{Deserialize, Serialize};

/// The category of resource being requested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionRuleType {
    Network,
    McpTool,
    ApiSink,
}

/// Generalization level of a proposed policy rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionScope {
    /// Exact match (e.g. `"api.example.com"` or `"fetch_webpage"`).
    Exact,
    /// Match any URL path under the given prefix.
    WildcardPath,
    /// Match any subdomain of the domain (e.g. `"*.example.com"`).
    WildcardDomain,
    /// Match anything of this rule type.
    WildcardAll,
}

/// A single proposed policy edit to present to the user alongside an approval prompt.
#[derive(Debug, Clone)]
pub struct PolicySuggestion {
    /// Category of rule being proposed.
    pub rule_type: SuggestionRuleType,
    /// The pattern to allow (host, tool name, sink type, etc.).
    pub pattern: String,
    /// Generalization level of this suggestion.
    pub scope: SuggestionScope,
}

impl PolicySuggestion {
    /// Plain-English description of this suggestion (no LLM, pure template fill).
    pub fn describe(&self) -> String {
        match (&self.rule_type, &self.scope) {
            (SuggestionRuleType::Network, SuggestionScope::Exact) => {
                format!("Allow connections to {}", self.pattern)
            }
            (SuggestionRuleType::Network, SuggestionScope::WildcardDomain) => {
                // pattern is e.g. "*.example.com" — strip leading "*." for display
                let domain = self.pattern.strip_prefix("*.").unwrap_or(&self.pattern);
                format!("Allow connections to any subdomain of {domain}")
            }
            (SuggestionRuleType::Network, SuggestionScope::WildcardPath) => {
                format!("Allow connections to {} and any path under it", self.pattern)
            }
            (SuggestionRuleType::Network, SuggestionScope::WildcardAll) => {
                "Allow connections to any host".to_string()
            }
            (SuggestionRuleType::McpTool, SuggestionScope::Exact) => {
                format!("Always allow the {} tool", self.pattern)
            }
            (SuggestionRuleType::McpTool, SuggestionScope::WildcardAll) => {
                format!("Always allow all tools on {}", self.pattern)
            }
            (SuggestionRuleType::McpTool, _) => {
                format!("Always allow {} tool calls", self.pattern)
            }
            (SuggestionRuleType::ApiSink, SuggestionScope::Exact) => {
                format!("Always allow calls to the {} sink", self.pattern)
            }
            (SuggestionRuleType::ApiSink, _) => {
                format!("Always allow {} API sink calls", self.pattern)
            }
        }
    }

    /// TOML `[[rules]]` entry to append to the relevant config file.
    pub fn to_toml_snippet(&self) -> String {
        match self.rule_type {
            SuggestionRuleType::Network => {
                format!(
                    "[[rules]]\npattern = \"{}\"\naction = \"allow\"\n",
                    self.pattern
                )
            }
            SuggestionRuleType::McpTool => {
                format!(
                    "[[rules]]\ntool = \"{}\"\naction = \"allow\"\n",
                    self.pattern
                )
            }
            SuggestionRuleType::ApiSink => {
                format!(
                    "[[rules]]\nsink = \"{}\"\naction = \"allow\"\n",
                    self.pattern
                )
            }
        }
    }

    /// Config file name this suggestion belongs in.
    pub fn config_file_key(&self) -> &'static str {
        match self.rule_type {
            SuggestionRuleType::Network => "network-rules.toml",
            SuggestionRuleType::McpTool => "tool-rules.toml",
            SuggestionRuleType::ApiSink => "api-rules.toml",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn network_exact(host: &str) -> PolicySuggestion {
        PolicySuggestion {
            rule_type: SuggestionRuleType::Network,
            pattern: host.to_string(),
            scope: SuggestionScope::Exact,
        }
    }

    fn network_wildcard_domain(pattern: &str) -> PolicySuggestion {
        PolicySuggestion {
            rule_type: SuggestionRuleType::Network,
            pattern: pattern.to_string(),
            scope: SuggestionScope::WildcardDomain,
        }
    }

    fn network_wildcard_all() -> PolicySuggestion {
        PolicySuggestion {
            rule_type: SuggestionRuleType::Network,
            pattern: "*".to_string(),
            scope: SuggestionScope::WildcardAll,
        }
    }

    fn mcp_tool_exact(tool: &str) -> PolicySuggestion {
        PolicySuggestion {
            rule_type: SuggestionRuleType::McpTool,
            pattern: tool.to_string(),
            scope: SuggestionScope::Exact,
        }
    }

    fn mcp_tool_all_on_server(server: &str) -> PolicySuggestion {
        PolicySuggestion {
            rule_type: SuggestionRuleType::McpTool,
            pattern: server.to_string(),
            scope: SuggestionScope::WildcardAll,
        }
    }

    fn api_sink_exact(sink: &str) -> PolicySuggestion {
        PolicySuggestion {
            rule_type: SuggestionRuleType::ApiSink,
            pattern: sink.to_string(),
            scope: SuggestionScope::Exact,
        }
    }

    #[test]
    fn describe_network_exact() {
        assert_eq!(
            network_exact("api.example.com").describe(),
            "Allow connections to api.example.com"
        );
    }

    #[test]
    fn describe_network_wildcard_domain() {
        assert_eq!(
            network_wildcard_domain("*.example.com").describe(),
            "Allow connections to any subdomain of example.com"
        );
    }

    #[test]
    fn describe_network_wildcard_all() {
        assert_eq!(
            network_wildcard_all().describe(),
            "Allow connections to any host"
        );
    }

    #[test]
    fn describe_mcp_tool_exact() {
        assert_eq!(
            mcp_tool_exact("fetch_webpage").describe(),
            "Always allow the fetch_webpage tool"
        );
    }

    #[test]
    fn describe_mcp_tool_wildcard_all() {
        assert_eq!(
            mcp_tool_all_on_server("my_server").describe(),
            "Always allow all tools on my_server"
        );
    }

    #[test]
    fn describe_api_sink_exact() {
        assert_eq!(
            api_sink_exact("email").describe(),
            "Always allow calls to the email sink"
        );
    }

    #[test]
    fn to_toml_snippet_network() {
        let snippet = network_exact("api.example.com").to_toml_snippet();
        assert!(snippet.contains("[[rules]]"));
        assert!(snippet.contains("pattern = \"api.example.com\""));
        assert!(snippet.contains("action = \"allow\""));
        // Must be valid TOML
        let table: toml::Value = toml::from_str(&snippet).unwrap();
        let rules = table.get("rules").unwrap().as_array().unwrap();
        assert_eq!(rules[0]["action"].as_str().unwrap(), "allow");
    }

    #[test]
    fn to_toml_snippet_mcp_tool() {
        let snippet = mcp_tool_exact("fetch_webpage").to_toml_snippet();
        assert!(snippet.contains("[[rules]]"));
        assert!(snippet.contains("tool = \"fetch_webpage\""));
        assert!(snippet.contains("action = \"allow\""));
    }

    #[test]
    fn to_toml_snippet_api_sink() {
        let snippet = api_sink_exact("email").to_toml_snippet();
        assert!(snippet.contains("[[rules]]"));
        assert!(snippet.contains("sink = \"email\""));
    }

    #[test]
    fn config_file_keys() {
        assert_eq!(network_exact("x").config_file_key(), "network-rules.toml");
        assert_eq!(mcp_tool_exact("x").config_file_key(), "tool-rules.toml");
        assert_eq!(api_sink_exact("x").config_file_key(), "api-rules.toml");
    }
}
