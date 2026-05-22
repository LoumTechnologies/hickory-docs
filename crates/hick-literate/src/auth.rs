//! Auth injection stack for transparent credential management.
//!
//! Bundles all auth-injection state into a single [`AuthStack`] that can
//! be passed through the executor → container pipeline.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use log::info;

use hick_secrets::{AgeSecretsProvider, SecretsProvider};
use hick_secrets::cache::SecretCache;
use hick_secrets::chain::SecretsProviderChain;
use hick_secrets::env::EnvSecretsProvider;
use hick_net::auth_injector::{self, AuthInjector};
use hick_net::auth_rules::AuthConfig;
use hick_net::http::HttpFilter;
use hick_net::network_loop::TlsInterceptConfig;
use hick_net::tls_ca::HickCa;

/// Bundled auth injection state.
///
/// Contains everything needed to intercept HTTPS connections and inject
/// credentials: the HTTP filter, TLS intercept config, container env vars,
/// and the CA certificate for trust store injection.
pub struct AuthStack {
    /// HTTP filter that injects secrets into auth headers.
    pub http_filter: Arc<dyn HttpFilter>,
    /// TLS interception config for MITM of matched hosts.
    pub tls_intercept: TlsInterceptConfig,
    /// Environment variables to set in the container (`(name, "hick-managed")`).
    pub container_env: Vec<(String, String)>,
    /// CA certificate in PEM format for injection into container trust stores.
    pub ca_cert_pem: String,
}

impl AuthStack {
    /// Build an auth stack from config files.
    ///
    /// - `rules_path`: path to `auth-rules.toml`
    /// - `age_key_path`: path to age identity (private key) file
    /// - `age_secrets_dir`: directory containing `<name>.age` encrypted secrets
    ///
    /// Returns `None` if the rules file doesn't exist or has no rules.
    pub fn from_config(
        rules_path: &Path,
        age_key_path: &Path,
        age_secrets_dir: &Path,
    ) -> Result<Option<Self>> {
        if !rules_path.exists() {
            return Ok(None);
        }

        let config = AuthConfig::load(rules_path)
            .with_context(|| format!("failed to load auth rules from {}", rules_path.display()))?;

        if config.rules.is_empty() {
            return Ok(None);
        }

        info!(
            "Auth injection: {} rules loaded from {}",
            config.rules.len(),
            rules_path.display()
        );

        // Build provider chain: env vars first (for dev), then age-encrypted files.
        let mut chain = SecretsProviderChain::new().add(EnvSecretsProvider);
        if age_key_path.exists() && age_secrets_dir.exists() {
            chain = chain.add(AgeSecretsProvider::new(age_key_path, age_secrets_dir));
        }

        let provider: Arc<dyn SecretsProvider> = Arc::new(chain);
        let cache = SecretCache::new(config.cache_ttl);

        // Generate ephemeral CA.
        let ca = HickCa::generate().context("failed to generate ephemeral CA")?;
        let ca_cert_pem = ca.cert_pem.clone();
        let ca = Arc::new(ca);

        // Collect hosts that need TLS interception.
        let intercept_hosts = config.intercepted_hosts();

        // Build the auth injector (implements HttpFilter).
        let injector = AuthInjector::new(config.clone(), cache, provider, None);

        // Collect placeholder env vars.
        let container_env = auth_injector::placeholder_env_vars(&config);

        Ok(Some(Self {
            http_filter: Arc::new(injector),
            tls_intercept: TlsInterceptConfig {
                ca,
                intercept_hosts,
            },
            container_env,
            ca_cert_pem,
        }))
    }
}
