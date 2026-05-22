//! HTTP filter that injects secrets into auth headers.
//!
//! Matches outgoing HTTP requests by `Host` header against configured rules,
//! resolves the corresponding secret, and sets the appropriate auth header.
//! As defense-in-depth, strips any `hick-managed` placeholder values from
//! auth headers before forwarding.

use std::sync::Arc;

use async_trait::async_trait;
use log::{info, warn};
use secrecy::ExposeSecret;

use hick_secrets_crate::{SecretsProvider, cache::SecretCache};

use crate::auth_rules::AuthConfig;
use crate::http::{HttpFilter, HttpFilterAction, HttpRequest, HttpResponse};

/// Placeholder value injected into container environment variables.
pub const PLACEHOLDER: &str = "hick-managed";

/// Auth headers to check for placeholder values.
const AUTH_HEADERS: &[&str] = &["authorization", "x-api-key", "api-key"];

/// HTTP filter that injects secrets into auth headers.
pub struct AuthInjector {
    config: AuthConfig,
    cache: SecretCache,
    provider: Arc<dyn SecretsProvider>,
    /// Optional inner filter to delegate to after auth injection.
    inner: Option<Arc<dyn HttpFilter>>,
}

impl AuthInjector {
    pub fn new(
        config: AuthConfig,
        cache: SecretCache,
        provider: Arc<dyn SecretsProvider>,
        inner: Option<Arc<dyn HttpFilter>>,
    ) -> Self {
        Self {
            config,
            cache,
            provider,
            inner,
        }
    }
}

#[async_trait]
impl HttpFilter for AuthInjector {
    async fn filter_request(&self, mut request: HttpRequest) -> HttpFilterAction<HttpRequest> {
        let host = extract_host(&request);

        // Phase 1: Auth injection — match rules by host.
        for rule in &self.config.rules {
            if rule.host == host {
                match self
                    .cache
                    .get_or_resolve(&rule.secret_name, self.provider.as_ref())
                    .await
                {
                    Ok(secret) => {
                        let value = rule.format.replace("{secret}", secret.expose_secret());
                        set_header(&mut request, &rule.header, &value);
                        info!(
                            "auth: injected secret '{}' for host '{}'",
                            rule.secret_name, host
                        );
                    }
                    Err(e) => {
                        warn!(
                            "auth: failed to resolve secret '{}' for host '{}': {e}",
                            rule.secret_name, host
                        );
                    }
                }
            }
        }

        // Phase 2: Strip placeholder values as defense-in-depth.
        strip_placeholders(&mut request, &host);

        // Phase 3: Delegate to inner filter if present.
        if let Some(ref inner) = self.inner {
            inner.filter_request(request).await
        } else {
            HttpFilterAction::Forward(request)
        }
    }

    async fn filter_response(
        &self,
        request: &HttpRequest,
        response: HttpResponse,
    ) -> HttpFilterAction<HttpResponse> {
        if let Some(ref inner) = self.inner {
            inner.filter_response(request, response).await
        } else {
            HttpFilterAction::Forward(response)
        }
    }
}

/// Generate de-duplicated placeholder environment variable pairs.
///
/// Returns `(secret_name, "hick-managed")` pairs for each unique secret name
/// in the auth rules. These are injected into the container environment at
/// boot so that API client libraries don't fail due to missing env vars.
pub fn placeholder_env_vars(config: &AuthConfig) -> Vec<(String, String)> {
    let mut seen = std::collections::HashSet::new();
    let mut vars = Vec::new();
    for rule in &config.rules {
        if seen.insert(rule.secret_name.clone()) {
            vars.push((rule.secret_name.clone(), PLACEHOLDER.to_string()));
        }
    }
    vars
}

/// Extract the host from the `Host` header, stripping any port suffix.
fn extract_host(request: &HttpRequest) -> String {
    for (name, value) in &request.headers {
        if name.eq_ignore_ascii_case("host") {
            // Strip port if present (e.g. "api.example.com:443" -> "api.example.com")
            return value.split(':').next().unwrap_or(value).to_string();
        }
    }
    String::new()
}

/// Replace an existing header (case-insensitive) or append a new one.
fn set_header(request: &mut HttpRequest, name: &str, value: &str) {
    for (hdr_name, hdr_value) in &mut request.headers {
        if hdr_name.eq_ignore_ascii_case(name) {
            *hdr_value = value.to_string();
            return;
        }
    }
    request
        .headers
        .push((name.to_string(), value.to_string()));
}

/// Remove auth headers that contain the placeholder value.
fn strip_placeholders(request: &mut HttpRequest, host: &str) {
    request.headers.retain(|(name, value)| {
        let is_auth = AUTH_HEADERS
            .iter()
            .any(|h| name.eq_ignore_ascii_case(h));
        if is_auth && value.contains(PLACEHOLDER) {
            warn!(
                "auth: stripped placeholder from header '{}' for host '{}'",
                name, host
            );
            return false;
        }
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_rules::AuthConfig;
    use hick_secrets_crate::InMemorySecretsProvider;

    fn make_request(host: &str, auth_header: Option<(&str, &str)>) -> HttpRequest {
        let mut headers = vec![("Host".into(), host.into())];
        if let Some((name, value)) = auth_header {
            headers.push((name.into(), value.into()));
        }
        HttpRequest {
            method: "POST".into(),
            uri: "/v1/chat/completions".into(),
            version: 1,
            headers,
            body: Vec::new(),
        }
    }

    #[test]
    fn placeholder_env_vars_deduplicates() {
        let config = AuthConfig::from_toml(
            r#"
[[rules]]
host = "a.com"
secret = "KEY"
header = "Authorization"
format = "Bearer {secret}"

[[rules]]
host = "b.com"
secret = "KEY"
header = "x-api-key"
format = "{secret}"
"#,
        )
        .unwrap();
        let vars = placeholder_env_vars(&config);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars[0], ("KEY".into(), "hick-managed".into()));
    }

    #[tokio::test]
    async fn injects_auth_header() {
        let config = AuthConfig::from_toml(
            r#"
[[rules]]
host = "api.openai.com"
secret = "OPENAI_API_KEY"
header = "Authorization"
format = "Bearer {secret}"
"#,
        )
        .unwrap();
        let mut provider = InMemorySecretsProvider::new();
        provider.add_secret("OPENAI_API_KEY", "sk-test-123");

        let cache = SecretCache::new(std::time::Duration::from_secs(300));
        let injector = AuthInjector::new(config, cache, Arc::new(provider), None);

        let request = make_request("api.openai.com", None);
        let result = injector.filter_request(request).await;
        match result {
            HttpFilterAction::Forward(req) => {
                let auth = req
                    .headers
                    .iter()
                    .find(|(n, _)| n == "Authorization")
                    .unwrap();
                assert_eq!(auth.1, "Bearer sk-test-123");
            }
            HttpFilterAction::Block => panic!("should not block"),
        }
    }

    #[tokio::test]
    async fn strips_placeholder_values() {
        let config = AuthConfig::empty();
        let provider = InMemorySecretsProvider::new();
        let cache = SecretCache::new(std::time::Duration::from_secs(300));
        let injector = AuthInjector::new(config, cache, Arc::new(provider), None);

        let request = make_request(
            "api.openai.com",
            Some(("Authorization", "Bearer hick-managed")),
        );
        let result = injector.filter_request(request).await;
        match result {
            HttpFilterAction::Forward(req) => {
                assert!(
                    !req.headers
                        .iter()
                        .any(|(n, _)| n.eq_ignore_ascii_case("Authorization")),
                    "placeholder auth header should be stripped"
                );
            }
            HttpFilterAction::Block => panic!("should not block"),
        }
    }

    #[tokio::test]
    async fn no_match_passes_through() {
        let config = AuthConfig::from_toml(
            r#"
[[rules]]
host = "api.openai.com"
secret = "OPENAI_API_KEY"
header = "Authorization"
format = "Bearer {secret}"
"#,
        )
        .unwrap();
        let provider = InMemorySecretsProvider::new();
        let cache = SecretCache::new(std::time::Duration::from_secs(300));
        let injector = AuthInjector::new(config, cache, Arc::new(provider), None);

        let request = make_request("other.example.com", None);
        let result = injector.filter_request(request).await;
        assert!(matches!(result, HttpFilterAction::Forward(_)));
    }
}
