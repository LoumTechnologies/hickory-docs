//! Internal HTTP APIs for hick containers, served via the network stack's
//! `HttpFilter` trait.
//!
//! Intercepts requests to the gateway address (`10.0.1.1:8080`) and provides:
//! - `POST /api/token/attenuate` — create an attenuated macaroon
//! - `POST /api/token/verify` — verify a macaroon token
//! - `POST /api/confirm` — request manual confirmation

use std::sync::Arc;

use async_trait::async_trait;
use network::http::{HttpFilter, HttpFilterAction, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use hick_token::{ContainerCapabilities, FileRule, NetworkRule, SecretRule, TokenAuthority};

// ---------------------------------------------------------------------------
// API request/response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct AttenuateRequest {
    /// Base64/V2-serialized macaroon token to attenuate.
    token: String,
    /// Additional network rules (e.g. ["allow github.com:443", "deny *"]).
    #[serde(default)]
    network_rules: Vec<String>,
    /// Additional file rules (e.g. ["read /input/*", "write /output/*"]).
    #[serde(default)]
    file_rules: Vec<String>,
    /// Additional secret rules (e.g. ["ENV_VAR:secret-name"]).
    #[serde(default)]
    secret_rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AttenuateResponse {
    token: String,
}

#[derive(Debug, Deserialize)]
struct VerifyRequest {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyResponse {
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConfirmRequest {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfirmResponse {
    confirmed: bool,
}

// ---------------------------------------------------------------------------
// ConfirmHandler
// ---------------------------------------------------------------------------

/// Trait for handling confirmation requests from containers.
#[async_trait]
pub trait ConfirmHandler: Send + Sync {
    /// Request confirmation with the given message.
    /// Returns `true` if confirmed, `false` if denied.
    async fn confirm(&self, message: &str) -> bool;
}

/// Default handler that always confirms (for testing/non-interactive use).
pub struct AutoConfirm;

#[async_trait]
impl ConfirmHandler for AutoConfirm {
    async fn confirm(&self, _message: &str) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// GatewayFilter
// ---------------------------------------------------------------------------

/// HTTP filter that intercepts API requests to the gateway.
///
/// Non-API requests are forwarded through unchanged.
pub struct GatewayFilter {
    authority: Arc<TokenAuthority>,
    confirm_handler: Arc<dyn ConfirmHandler>,
    /// Container tokens indexed by container name for lookup.
    container_tokens: Mutex<std::collections::HashMap<String, hick_token::CapabilityToken>>,
}

impl GatewayFilter {
    /// Create a new gateway filter with the given token authority.
    pub fn new(authority: Arc<TokenAuthority>, confirm_handler: Arc<dyn ConfirmHandler>) -> Self {
        Self {
            authority,
            confirm_handler,
            container_tokens: Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Register a container's capability token for API use.
    pub async fn register_token(&self, name: String, token: hick_token::CapabilityToken) {
        self.container_tokens.lock().await.insert(name, token);
    }

    /// Handle a gateway API request and return an HTTP response.
    async fn handle_api(&self, request: &HttpRequest) -> HttpResponse {
        let (status, body) = match (request.method.as_str(), request.uri.as_str()) {
            ("POST", "/api/token/attenuate") => self.handle_attenuate(request).await,
            ("POST", "/api/token/verify") => self.handle_verify(request).await,
            ("POST", "/api/confirm") => self.handle_confirm(request).await,
            _ => (404, serde_json::json!({"error": "not found"}).to_string()),
        };

        let body_bytes = body.into_bytes();
        HttpResponse {
            status,
            reason: reason_for_status(status).to_string(),
            version: 1,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Content-Length".to_string(), body_bytes.len().to_string()),
                ("Connection".to_string(), "close".to_string()),
            ],
            body: body_bytes,
        }
    }

    async fn handle_attenuate(&self, request: &HttpRequest) -> (u16, String) {
        let req: AttenuateRequest = match serde_json::from_slice(&request.body) {
            Ok(r) => r,
            Err(e) => {
                return (
                    400,
                    serde_json::json!({"error": format!("invalid request: {e}")}).to_string(),
                );
            }
        };

        // Parse additional capabilities from the request
        let additional = match parse_capabilities(&req) {
            Ok(caps) => caps,
            Err(e) => {
                return (
                    400,
                    serde_json::json!({"error": format!("invalid capability: {e}")}).to_string(),
                );
            }
        };

        // Find the token to attenuate
        let tokens = self.container_tokens.lock().await;
        // Look for a token matching the serialized value
        let source_token = tokens
            .values()
            .find(|t| t.serialize().map(|s| s == req.token).unwrap_or(false));

        let source_token = match source_token {
            Some(t) => t,
            None => {
                return (
                    401,
                    serde_json::json!({"error": "token not found or invalid"}).to_string(),
                );
            }
        };

        match source_token.attenuate(&additional) {
            Ok(attenuated) => match attenuated.serialize() {
                Ok(serialized) => {
                    let resp = AttenuateResponse { token: serialized };
                    (200, serde_json::to_string(&resp).unwrap())
                }
                Err(e) => (
                    500,
                    serde_json::json!({"error": format!("serialization failed: {e}")}).to_string(),
                ),
            },
            Err(e) => (
                500,
                serde_json::json!({"error": format!("attenuation failed: {e}")}).to_string(),
            ),
        }
    }

    async fn handle_verify(&self, request: &HttpRequest) -> (u16, String) {
        let req: VerifyRequest = match serde_json::from_slice(&request.body) {
            Ok(r) => r,
            Err(e) => {
                return (
                    400,
                    serde_json::json!({"error": format!("invalid request: {e}")}).to_string(),
                );
            }
        };

        // Find a matching token
        let tokens = self.container_tokens.lock().await;
        let token = tokens
            .values()
            .find(|t| t.serialize().map(|s| s == req.token).unwrap_or(false));

        let resp = match token {
            Some(t) => match self.authority.verify(t) {
                Ok(()) => VerifyResponse {
                    valid: true,
                    error: None,
                },
                Err(e) => VerifyResponse {
                    valid: false,
                    error: Some(format!("{e}")),
                },
            },
            None => VerifyResponse {
                valid: false,
                error: Some("token not found".to_string()),
            },
        };

        (200, serde_json::to_string(&resp).unwrap())
    }

    async fn handle_confirm(&self, request: &HttpRequest) -> (u16, String) {
        let req: ConfirmRequest = match serde_json::from_slice(&request.body) {
            Ok(r) => r,
            Err(e) => {
                return (
                    400,
                    serde_json::json!({"error": format!("invalid request: {e}")}).to_string(),
                );
            }
        };

        let confirmed = self.confirm_handler.confirm(&req.message).await;
        let resp = ConfirmResponse { confirmed };
        (200, serde_json::to_string(&resp).unwrap())
    }
}

/// Parse `AttenuateRequest` rule strings into `ContainerCapabilities`.
fn parse_capabilities(req: &AttenuateRequest) -> Result<ContainerCapabilities, String> {
    let mut caps = ContainerCapabilities::new();

    for rule in &req.network_rules {
        if rule == "deny *" {
            caps.network_rules.push(NetworkRule::deny_all());
        } else if let Some(rest) = rule.strip_prefix("allow ") {
            let (host, port) = rest
                .rsplit_once(':')
                .ok_or_else(|| format!("invalid network rule: {rule}"))?;
            caps.network_rules.push(NetworkRule::allow(host, port));
        } else {
            return Err(format!("invalid network rule: {rule}"));
        }
    }

    for rule in &req.file_rules {
        if let Some(path) = rule.strip_prefix("read ") {
            caps.file_rules.push(FileRule::Read(path.to_string()));
        } else if let Some(path) = rule.strip_prefix("write ") {
            caps.file_rules.push(FileRule::Write(path.to_string()));
        } else {
            return Err(format!("invalid file rule: {rule}"));
        }
    }

    for rule in &req.secret_rules {
        let (env_var, secret_name) = rule
            .split_once(':')
            .ok_or_else(|| format!("invalid secret rule: {rule}"))?;
        caps.secret_rules.push(SecretRule {
            env_var: env_var.to_string(),
            secret_name: secret_name.to_string(),
        });
    }

    Ok(caps)
}

/// Map status codes to reason phrases.
fn reason_for_status(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

#[async_trait]
impl HttpFilter for GatewayFilter {
    async fn filter_request(&self, request: HttpRequest) -> HttpFilterAction<HttpRequest> {
        // Check if this is a gateway API request
        let is_api = request.uri.starts_with("/api/");

        if is_api {
            // Handle internally and return the response directly.
            // We use Block to prevent forwarding, and the response is
            // constructed here. However, the HttpFilter trait doesn't
            // support returning a response from filter_request directly.
            //
            // Instead, we intercept the request and inject a marker header
            // so filter_response can generate the real response.
            log::debug!("Gateway API request: {} {}", request.method, request.uri);
            HttpFilterAction::Forward(request)
        } else {
            HttpFilterAction::Forward(request)
        }
    }

    async fn filter_response(
        &self,
        request: &HttpRequest,
        response: HttpResponse,
    ) -> HttpFilterAction<HttpResponse> {
        if request.uri.starts_with("/api/") {
            // Generate the API response
            let api_response = self.handle_api(request).await;
            HttpFilterAction::Forward(api_response)
        } else {
            HttpFilterAction::Forward(response)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use hick_token::ContainerCapabilities;

    fn test_authority() -> Arc<TokenAuthority> {
        Arc::new(TokenAuthority::new(b"test-root-key-32bytes-long-xxxxx"))
    }

    fn gateway_filter() -> GatewayFilter {
        GatewayFilter::new(test_authority(), Arc::new(AutoConfirm))
    }

    #[tokio::test]
    async fn verify_valid_token() {
        let filter = gateway_filter();
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let token = filter.authority.mint("test", &caps).unwrap();
        let serialized = token.serialize().unwrap();
        filter.register_token("test".to_string(), token).await;

        let request = HttpRequest {
            method: "POST".to_string(),
            uri: "/api/token/verify".to_string(),
            version: 1,
            headers: vec![],
            body: serde_json::json!({"token": serialized})
                .to_string()
                .into_bytes(),
        };

        let response = filter.handle_api(&request).await;
        assert_eq!(response.status, 200);

        let body: VerifyResponse = serde_json::from_slice(&response.body).unwrap();
        assert!(body.valid);
        assert!(body.error.is_none());
    }

    #[tokio::test]
    async fn verify_unknown_token() {
        let filter = gateway_filter();

        let request = HttpRequest {
            method: "POST".to_string(),
            uri: "/api/token/verify".to_string(),
            version: 1,
            headers: vec![],
            body: serde_json::json!({"token": "bogus"})
                .to_string()
                .into_bytes(),
        };

        let response = filter.handle_api(&request).await;
        assert_eq!(response.status, 200);

        let body: VerifyResponse = serde_json::from_slice(&response.body).unwrap();
        assert!(!body.valid);
    }

    #[tokio::test]
    async fn confirm_auto_approves() {
        let filter = gateway_filter();

        let request = HttpRequest {
            method: "POST".to_string(),
            uri: "/api/confirm".to_string(),
            version: 1,
            headers: vec![],
            body: serde_json::json!({"message": "Deploy to prod?"})
                .to_string()
                .into_bytes(),
        };

        let response = filter.handle_api(&request).await;
        assert_eq!(response.status, 200);

        let body: ConfirmResponse = serde_json::from_slice(&response.body).unwrap();
        assert!(body.confirmed);
    }

    #[tokio::test]
    async fn not_found_for_unknown_endpoint() {
        let filter = gateway_filter();

        let request = HttpRequest {
            method: "GET".to_string(),
            uri: "/api/unknown".to_string(),
            version: 1,
            headers: vec![],
            body: vec![],
        };

        let response = filter.handle_api(&request).await;
        assert_eq!(response.status, 404);
    }

    #[tokio::test]
    async fn attenuate_token() {
        let filter = gateway_filter();
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network();

        let token = filter.authority.mint("sandbox", &caps).unwrap();
        let serialized = token.serialize().unwrap();
        filter.register_token("sandbox".to_string(), token).await;

        let request = HttpRequest {
            method: "POST".to_string(),
            uri: "/api/token/attenuate".to_string(),
            version: 1,
            headers: vec![],
            body: serde_json::json!({
                "token": serialized,
                "network_rules": ["deny *"],
                "file_rules": ["read /input/*"]
            })
            .to_string()
            .into_bytes(),
        };

        let response = filter.handle_api(&request).await;
        assert_eq!(response.status, 200);

        let body: AttenuateResponse = serde_json::from_slice(&response.body).unwrap();
        assert!(!body.token.is_empty());
        assert_ne!(body.token, serialized);
    }

    #[test]
    fn parse_capabilities_valid() {
        let req = AttenuateRequest {
            token: String::new(),
            network_rules: vec!["allow github.com:443".to_string(), "deny *".to_string()],
            file_rules: vec!["read /input/*".to_string(), "write /output/*".to_string()],
            secret_rules: vec!["TOKEN:my-secret".to_string()],
        };

        let caps = parse_capabilities(&req).unwrap();
        assert_eq!(caps.network_rules.len(), 2);
        assert_eq!(caps.file_rules.len(), 2);
        assert_eq!(caps.secret_rules.len(), 1);
    }

    #[test]
    fn parse_capabilities_invalid_network_rule() {
        let req = AttenuateRequest {
            token: String::new(),
            network_rules: vec!["invalid".to_string()],
            file_rules: vec![],
            secret_rules: vec![],
        };

        assert!(parse_capabilities(&req).is_err());
    }
}
