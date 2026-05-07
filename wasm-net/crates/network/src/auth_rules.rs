//! TOML-based auth injection rules.
//!
//! Maps destination hostnames to secrets and HTTP headers for automatic
//! credential injection by [`AuthInjector`](crate::auth_injector::AuthInjector).

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

/// A single auth injection rule.
#[derive(Debug, Clone)]
pub struct AuthRule {
    /// Exact hostname to match (e.g. "api.openai.com").
    pub host: String,
    /// Secret name in the secrets provider (e.g. "OPENAI_API_KEY").
    pub secret_name: String,
    /// HTTP header to set (e.g. "Authorization").
    pub header: String,
    /// Format string with `{secret}` placeholder (e.g. "Bearer {secret}").
    pub format: String,
}

/// Top-level auth injection configuration.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub rules: Vec<AuthRule>,
    pub cache_ttl: Duration,
}

impl AuthConfig {
    /// Load configuration from a TOML file.
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::from_toml(&content)
    }

    /// Parse configuration from a TOML string.
    pub fn from_toml(content: &str) -> anyhow::Result<Self> {
        let raw: RawAuthConfig = toml::from_str(content)?;
        let cache_ttl = raw
            .cache
            .map(|c| Duration::from_secs(c.ttl_seconds))
            .unwrap_or(Duration::from_secs(300));
        let rules = raw
            .rules
            .unwrap_or_default()
            .into_iter()
            .map(|r| AuthRule {
                host: r.host,
                secret_name: r.secret,
                header: r.header,
                format: r.format,
            })
            .collect();
        Ok(Self { rules, cache_ttl })
    }

    /// Return the set of hostnames that have auth rules.
    pub fn intercepted_hosts(&self) -> HashSet<String> {
        self.rules.iter().map(|r| r.host.clone()).collect()
    }

    /// Empty configuration with no rules.
    pub fn empty() -> Self {
        Self {
            rules: Vec::new(),
            cache_ttl: Duration::from_secs(300),
        }
    }
}

// -- TOML deserialization types --

#[derive(Deserialize)]
struct RawAuthConfig {
    cache: Option<RawCacheConfig>,
    rules: Option<Vec<RawAuthRule>>,
}

#[derive(Deserialize)]
struct RawCacheConfig {
    ttl_seconds: u64,
}

#[derive(Deserialize)]
struct RawAuthRule {
    host: String,
    secret: String,
    header: String,
    format: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_config() {
        let toml = r#"
[[rules]]
host = "api.openai.com"
secret = "OPENAI_API_KEY"
header = "Authorization"
format = "Bearer {secret}"

[[rules]]
host = "api.anthropic.com"
secret = "ANTHROPIC_API_KEY"
header = "x-api-key"
format = "{secret}"
"#;
        let config = AuthConfig::from_toml(toml).unwrap();
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.cache_ttl, Duration::from_secs(300));
        assert_eq!(config.rules[0].host, "api.openai.com");
        assert_eq!(config.rules[1].secret_name, "ANTHROPIC_API_KEY");
    }

    #[test]
    fn parse_with_custom_ttl() {
        let toml = r#"
[cache]
ttl_seconds = 60

[[rules]]
host = "api.example.com"
secret = "API_KEY"
header = "Authorization"
format = "Bearer {secret}"
"#;
        let config = AuthConfig::from_toml(toml).unwrap();
        assert_eq!(config.cache_ttl, Duration::from_secs(60));
    }

    #[test]
    fn intercepted_hosts_set() {
        let toml = r#"
[[rules]]
host = "api.openai.com"
secret = "KEY1"
header = "Authorization"
format = "Bearer {secret}"

[[rules]]
host = "api.anthropic.com"
secret = "KEY2"
header = "x-api-key"
format = "{secret}"
"#;
        let config = AuthConfig::from_toml(toml).unwrap();
        let hosts = config.intercepted_hosts();
        assert!(hosts.contains("api.openai.com"));
        assert!(hosts.contains("api.anthropic.com"));
        assert_eq!(hosts.len(), 2);
    }

    #[test]
    fn empty_config() {
        let config = AuthConfig::empty();
        assert!(config.rules.is_empty());
        assert!(config.intercepted_hosts().is_empty());
    }
}
