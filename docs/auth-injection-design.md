# Transparent Auth Injection for WASM Containers

## Problem Statement

WASM containers in hick-agent need to call authenticated APIs (OpenAI, Anthropic,
GitHub, etc.). Today, API keys must either be baked into container environment
variables or passed through code — both of which expose secrets to the container's
address space. A compromised or misbehaving container can exfiltrate these secrets.

We want:

1. **Secrets never enter the WASM address space.** The container code never sees
   the real API key.
2. **Standard API clients work unmodified.** An AI coding agent writes
   `client = openai.OpenAI()` with no API key argument, and it works.
3. **Pluggable secrets backend.** rage (age encryption) locally, cloud providers
   (Vault, AWS Secrets Manager, etc.) in production — decided at deployment time.
4. **Single-process architecture preserved.** No sidecar, no external proxy.
   Everything runs in the hick-agent process.
5. **Capability-gated.** A container can only use secrets its macaroon token
   authorizes.

## Prerequisite: TLS Interception

Almost all API endpoints use HTTPS. The current network stack only inspects HTTP
traffic on port 80 — port 443 is opaque passthrough of encrypted bytes. To inject
auth headers into HTTPS requests, we must first implement selective TLS
man-in-the-middle.

This was planned in [network-stack-design.md](network-stack-design.md) (Decision
#4) but never built. It is the critical prerequisite for this feature.

## Architecture Overview

```
Container code: `client = openai.OpenAI()`
   │
   │  Env: OPENAI_API_KEY=hick-managed  (placeholder, not a real key)
   │  Client sends: Authorization: Bearer hick-managed
   │
   ↓
smoltcp in-process TCP stack
   ↓
TLS ClientHello intercepted — extract SNI
   ↓
SNI matches auth rule?
   ├── NO  → plain TLS passthrough (zero overhead)
   └── YES ↓
           Terminate TLS with container (Hick CA cert)
           ↓
           Plaintext HTTP visible
           ↓
           AuthInjector:
              1. Match host against auth rules
              2. Verify container's macaroon token authorizes the secret
              3. Resolve secret from SecretsProvider (with TTL cache)
              4. Replace auth header with real value
              5. Strip any remaining "hick-managed" placeholders
              6. Audit log: secret name, destination, container ID
           ↓
           Establish real TLS to destination
           ↓
           Forward modified request
```

## Design Decisions

### 1. Selective TLS MITM (not universal)

Only connections whose SNI matches an auth rule get intercepted. All other TLS
connections pass through unmodified. This minimizes performance overhead and
attack surface.

**Rationale:** Universal MITM would require injecting the CA cert into every
container image's trust store for all connections. Selective MITM means we only
need trust for specific, controlled API endpoints.

### 2. Placeholder environment variables

API clients typically require an API key at initialization. The OpenAI Python
client raises `OpenAIError` if `OPENAI_API_KEY` is unset. To avoid this, we
inject placeholder env vars into the container at startup:

```
OPENAI_API_KEY=hick-managed
ANTHROPIC_API_KEY=hick-managed
```

The client initializes, sends `Authorization: Bearer hick-managed`, and the
AuthInjector replaces it with the real key at the network boundary.

**Why "hick-managed":** Distinctive enough to grep for, clearly not a real key,
short enough to not trigger length-based validation in clients.

### 3. Placeholder stripping as defense-in-depth

After auth injection, a final pass strips any remaining `hick-managed` values
from auth headers — even if no rule matched. This prevents the placeholder from
leaking to remote APIs if configuration is incomplete. The request will fail with
401, which is the correct behavior for an unauthorized container.

Auth headers checked: `Authorization`, `x-api-key`, `api-key`.

### 4. Secret caching with configurable TTL

Decrypted secrets are cached in memory with a configurable TTL. `ttl = 0` means
no caching (every request decrypts from the provider). Cached values use the
`secrecy` crate's `SecretString` type for zeroize-on-drop.

**Default TTL:** 300 seconds (5 minutes) for cloud providers. For local rage,
TTL=60s is reasonable since decryption is fast (~1ms) but not free.

### 5. Audit logging by default

Every auth-related event is logged via `tracing`. Secret values are **never**
logged — only names.

Events logged:
- Secret injected (info): container_id, secret_name, destination host, header name
- Placeholder stripped (warn): container_id, destination host, header name
- Capability denied (warn): container_id, secret_name, destination host
- Secret not found (error): container_id, secret_name
- Cache hit/miss (debug): secret_name, ttl_remaining

### 6. rage as the default local provider

rage (Rust port of age) is already used in `hick-secrets`. It keeps everything
in-process: no daemon, no socket, no external dependency. CPU-bound decryption
runs on `tokio::task::spawn_blocking`.

### 7. Provider chain, not provider selection

Multiple providers are tried in order, not selected by configuration. This allows
layering: check env vars first (dev convenience), then rage (local secrets), then
cloud provider (production). First match wins.

## Detailed Component Design

### Component 1: TLS Interception (`network` crate)

#### New dependencies

```toml
# crates/network/Cargo.toml
rustls = "0.23"
tokio-rustls = "0.26"
rcgen = "0.13"
```

#### Hick CA

A self-signed CA certificate and private key, generated once per hick-agent
session (or loaded from rage-encrypted storage for persistence across sessions).

```rust
// crates/network/src/tls_ca.rs

pub struct HickCa {
    /// CA certificate (DER-encoded)
    pub cert_der: Vec<u8>,
    /// CA certificate (PEM-encoded, for injection into container trust stores)
    pub cert_pem: String,
    /// CA signing key
    key: rcgen::KeyPair,
    /// rcgen certificate params for signing
    ca_params: rcgen::CertificateParams,
}

impl HickCa {
    /// Generate a new ephemeral CA.
    pub fn generate() -> Result<Self>;

    /// Generate a TLS certificate for a specific hostname, signed by this CA.
    pub fn generate_cert(&self, hostname: &str) -> Result<CertifiedKey>;
}
```

The CA is ephemeral by default (generated fresh per session). For long-running
deployments, it can be persisted via the SecretsProvider. Generated host
certificates are cached in a `DashMap<String, Arc<CertifiedKey>>` keyed by
hostname.

#### SNI extraction

Before terminating TLS, we peek at the first bytes of the TCP stream to extract
the SNI from the ClientHello. This determines whether to MITM or passthrough.

```rust
// crates/network/src/tls_sni.rs

/// Extract the SNI hostname from a TLS ClientHello message.
///
/// Reads the first few hundred bytes without consuming them (peek).
/// Returns None if the data is not a TLS ClientHello or has no SNI.
pub fn extract_sni(client_hello: &[u8]) -> Option<String>;
```

The TLS ClientHello SNI extension is at a well-known offset in the handshake.
The `rustls` crate provides `rustls::server::Acceptor` which can extract SNI
without completing the handshake.

#### Network loop changes

The routing logic in `network_loop.rs` (line ~229) changes from two branches
(port 80 / everything else) to three:

```rust
if real_dest.port() == 443 && should_intercept_tls(&real_dest, &auth_rules) {
    // NEW: TLS MITM for auth-rule-matched connections
    tokio::spawn(async move {
        tls_relay::tls_mitm_relay(
            handle, real_dest, tx, write_rx,
            hick_ca.clone(), http_filter.clone(),
        ).await;
    });
} else if real_dest.port() == 80 {
    if let Some(ref hf) = http_filter {
        // EXISTING: HTTP filter for port 80
        tokio::spawn(async move {
            http::http_relay_real_socket(handle, real_dest, tx, write_rx, hf).await;
        });
    } else {
        tokio::spawn(async move {
            relay_real_socket(handle, real_dest, tx, write_rx).await;
        });
    }
} else {
    // EXISTING: passthrough for all other ports
    tokio::spawn(async move {
        relay_real_socket(handle, real_dest, tx, write_rx).await;
    });
}
```

Note: `should_intercept_tls` checks whether any auth rule matches the
destination. This requires the DNS cache (hostname → IP mapping) to resolve the
IP back to a hostname. The `CapabilityFilter` already maintains such a cache
(`dns_cache`). We extend this to be shared with the TLS routing decision.

#### TLS relay

```rust
// crates/network/src/tls_relay.rs

/// TLS MITM relay: terminates TLS from the container, inspects/modifies
/// the plaintext HTTP, then re-encrypts to the real server.
pub(crate) async fn tls_mitm_relay(
    handle: SocketHandle,
    dest: SocketAddr,
    event_tx: mpsc::Sender<RealSocketMsg>,
    data_rx: mpsc::Receiver<Vec<u8>>,
    ca: Arc<HickCa>,
    http_filter: Arc<dyn HttpFilter>,
) {
    // 1. Bridge mpsc channel to an AsyncRead/AsyncWrite stream
    // 2. Read ClientHello, extract SNI
    // 3. Generate cert for SNI hostname, signed by CA
    // 4. Complete TLS handshake with container using generated cert
    // 5. Connect real TLS to dest using tokio-rustls
    // 6. Run http_relay between the two plaintext streams, applying http_filter
    // 7. On close, clean up both sides
}
```

The inner HTTP relay reuses the existing `http_relay_real_socket` logic (parse
request, filter, forward, parse response, filter, return). The only difference
is that both sides are wrapped in TLS: container-side TLS is terminated by us,
server-side TLS is established by us.

#### CA injection into containers

At container boot (after kernel boot detection, before any user commands), inject
the Hick CA cert into the trust store:

```rust
// In container startup sequence:
container.write_stdin(format!(
    "cat >> /etc/ssl/certs/ca-certificates.crt << 'HICKCA'\n{}\nHICKCA\n",
    hick_ca.cert_pem
)).await;
```

For Alpine (BusyBox), this appends to the system CA bundle. Most TLS libraries
(OpenSSL, LibreSSL, BoringSSL) read this file. Python's `certifi` uses its own
bundle, but the `SSL_CERT_FILE` env var can override it:

```
SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
```

This env var is injected alongside the placeholder API keys.

### Component 2: Auth Injection (`network` crate, new module)

The AuthInjector implements `HttpFilter` and wraps an inner filter (if any).

#### Auth rules configuration

```toml
# .hick/auth-rules.toml

[cache]
ttl_seconds = 300   # 0 = no caching

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

[[rules]]
host = "api.github.com"
secret = "GITHUB_TOKEN"
header = "Authorization"
format = "token {secret}"
```

Rules are also expressible in hick XML for per-container configuration:

```xml
<hick:container image="python:3.12">
  <allow network="api.openai.com:443" secret="OPENAI_API_KEY" />
  <allow network="api.anthropic.com:443" secret="ANTHROPIC_API_KEY" />
</hick:container>
```

#### Rule matching

Rules match on the `Host` header of the HTTP request (after TLS termination).
Exact match only — no wildcards for auth rules (unlike network filter rules).
This is intentional: auth injection should be explicit about which endpoints
receive which credentials.

#### Data structures

```rust
// crates/network/src/auth_injector.rs

/// A single auth injection rule.
#[derive(Debug, Clone)]
pub struct AuthRule {
    /// Hostname to match (exact match against Host header).
    pub host: String,
    /// Name of the secret in the SecretsProvider.
    pub secret_name: String,
    /// HTTP header to set.
    pub header: String,
    /// Format string. `{secret}` is replaced with the secret value.
    pub format: String,
}

/// Configuration for auth injection.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub rules: Vec<AuthRule>,
    pub cache_ttl: Duration,
}
```

#### AuthInjector

```rust
pub struct AuthInjector {
    config: AuthConfig,
    cache: SecretCache,
    provider: Arc<dyn SecretsProvider>,
    /// Inner filter to delegate to after auth injection.
    inner: Option<Arc<dyn HttpFilter>>,
}

#[async_trait]
impl HttpFilter for AuthInjector {
    async fn filter_request(
        &self,
        mut request: HttpRequest,
    ) -> HttpFilterAction<HttpRequest> {
        let host = extract_host(&request);

        // Phase 1: Auth injection
        for rule in &self.config.rules {
            if rule.host != host { continue; }

            let secret = match self.cache.get_or_resolve(
                &rule.secret_name,
                self.provider.as_ref(),
            ).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(
                        secret_name = %rule.secret_name,
                        destination = %host,
                        error = %e,
                        "auth: failed to resolve secret"
                    );
                    continue;
                }
            };

            let header_value = rule.format.replace("{secret}", secret.expose_secret());
            set_header(&mut request, &rule.header, &header_value);

            tracing::info!(
                secret_name = %rule.secret_name,
                destination = %host,
                header = %rule.header,
                "auth: injected secret"
            );
        }

        // Phase 2: Strip placeholders
        strip_placeholders(&mut request, &host);

        // Phase 3: Delegate to inner filter
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
```

#### Placeholder stripping

```rust
const PLACEHOLDER: &str = "hick-managed";

/// Auth headers to check for placeholder values.
const AUTH_HEADERS: &[&str] = &["authorization", "x-api-key", "api-key"];

fn strip_placeholders(request: &mut HttpRequest, host: &str) {
    request.headers.retain(|(name, value)| {
        let is_auth = AUTH_HEADERS.contains(&name.to_lowercase().as_str());
        let has_placeholder = value.contains(PLACEHOLDER);
        if is_auth && has_placeholder {
            tracing::warn!(
                header = %name,
                destination = %host,
                "auth: stripped leaked placeholder"
            );
            return false;
        }
        true
    });
}
```

### Component 3: Secret Cache (`hick-secrets` crate)

```rust
// crates/hick-secrets/src/cache.rs

use std::time::{Duration, Instant};
use dashmap::DashMap;
use secrecy::{ExposeSecret, SecretString};

pub struct SecretCache {
    entries: DashMap<String, CachedSecret>,
    ttl: Duration,
}

struct CachedSecret {
    value: SecretString,
    cached_at: Instant,
}

impl SecretCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: DashMap::new(),
            ttl,
        }
    }

    /// Retrieve a secret, using cache if available and not expired.
    /// Falls back to the provider on cache miss or expiry.
    pub async fn get_or_resolve(
        &self,
        name: &str,
        provider: &dyn SecretsProvider,
    ) -> Result<SecretString, SecretsError> {
        // TTL=0 means no caching
        if !self.ttl.is_zero() {
            if let Some(entry) = self.entries.get(name) {
                if entry.cached_at.elapsed() < self.ttl {
                    tracing::debug!(
                        secret_name = %name,
                        ttl_remaining_secs = %(self.ttl - entry.cached_at.elapsed()).as_secs(),
                        "auth: cache hit"
                    );
                    return Ok(entry.value.clone());
                }
                // Expired — drop and re-fetch
            }
        }

        tracing::debug!(secret_name = %name, "auth: cache miss, resolving");
        let value = provider.get_secret(name).await?;
        let secret = SecretString::from(value);

        if !self.ttl.is_zero() {
            self.entries.insert(name.to_string(), CachedSecret {
                value: secret.clone(),
                cached_at: Instant::now(),
            });
        }

        Ok(secret)
    }

    /// Evict a specific secret from the cache (e.g., on rotation).
    pub fn evict(&self, name: &str) {
        self.entries.remove(name);
    }

    /// Evict all expired entries.
    pub fn evict_expired(&self) {
        self.entries.retain(|_, v| v.cached_at.elapsed() < self.ttl);
    }
}
```

### Component 4: Secrets Provider Chain (`hick-secrets` crate)

```rust
// crates/hick-secrets/src/chain.rs

/// A secrets provider that tries multiple providers in order.
/// First successful result wins.
pub struct SecretsProviderChain {
    providers: Vec<Box<dyn SecretsProvider>>,
}

impl SecretsProviderChain {
    pub fn new() -> Self {
        Self { providers: vec![] }
    }

    pub fn add(mut self, provider: impl SecretsProvider + 'static) -> Self {
        self.providers.push(Box::new(provider));
        self
    }
}

#[async_trait]
impl SecretsProvider for SecretsProviderChain {
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError> {
        for provider in &self.providers {
            match provider.get_secret(name).await {
                Ok(value) => return Ok(value),
                Err(SecretsError::NotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(SecretsError::NotFound(name.to_string()))
    }
}
```

#### EnvSecretsProvider

```rust
// crates/hick-secrets/src/env.rs

/// Secrets provider that reads from environment variables.
/// Useful for development: `export OPENAI_API_KEY=sk-...`
pub struct EnvSecretsProvider;

#[async_trait]
impl SecretsProvider for EnvSecretsProvider {
    async fn get_secret(&self, name: &str) -> Result<String, SecretsError> {
        std::env::var(name)
            .map_err(|_| SecretsError::NotFound(name.to_string()))
    }
}
```

#### Default chain

```rust
// For local usage:
let provider = SecretsProviderChain::new()
    .add(EnvSecretsProvider)
    .add(AgeSecretsProvider::new(key_path, secrets_dir));

// For cloud usage (future):
let provider = SecretsProviderChain::new()
    .add(VaultSecretsProvider::new(vault_addr, vault_token))
    .add(AgeSecretsProvider::new(key_path, secrets_dir));
```

### Component 5: Placeholder Env Var Injection

At container startup, after boot detection but before running user commands,
inject placeholder env vars for every auth rule the container is authorized to
use.

```rust
// In container startup, after boot detection:
fn inject_auth_placeholders(
    container: &mut Container,
    auth_rules: &[AuthRule],
    token: &MacaroonToken,
) {
    for rule in auth_rules {
        if token.allows_secret(&rule.secret_name) {
            // Derive env var name from the secret name (already uppercase convention)
            let env_var = &rule.secret_name;  // e.g., "OPENAI_API_KEY"
            container.set_env(env_var, "hick-managed");
        }
    }
    // Also set SSL_CERT_FILE for CA trust
    container.set_env("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
}
```

### Component 6: Capability Integration (`hick-token` crate)

The existing `SecretRule` in `hick-token` already encodes secret access:

```rust
pub struct SecretRule {
    pub env_var: String,       // e.g., "OPENAI_API_KEY"
    pub secret_name: String,   // e.g., "openai-api-key" (name in provider)
}
```

Caveat format: `secret = OPENAI_API_KEY:openai-api-key`

The AuthInjector checks `container_capabilities.secret_rules` to verify the
container is authorized before injecting. If the container's token doesn't
include a matching `SecretRule`, the auth injection is denied and logged.

Extension needed: add a convenience method to `ContainerCapabilities`:

```rust
impl ContainerCapabilities {
    /// Check if this container is authorized to use a given secret.
    pub fn allows_secret(&self, secret_name: &str) -> bool {
        self.secret_rules.iter().any(|r| r.secret_name == secret_name)
    }
}
```

## Wiring It All Together

In `hick-agent/src/main.rs` (or wherever the agent initializes containers):

```rust
// 1. Load auth rules
let auth_config = AuthConfig::load(".hick/auth-rules.toml")?;

// 2. Build secrets provider chain
let provider: Arc<dyn SecretsProvider> = Arc::new(
    SecretsProviderChain::new()
        .add(EnvSecretsProvider)
        .add(AgeSecretsProvider::new(key_path, secrets_dir))
);

// 3. Build secret cache
let cache = SecretCache::new(Duration::from_secs(auth_config.cache_ttl_seconds));

// 4. Build AuthInjector (implements HttpFilter)
let auth_injector: Arc<dyn HttpFilter> = Arc::new(AuthInjector::new(
    auth_config.clone(),
    cache,
    provider,
    None,  // no inner filter, or wrap existing filter
));

// 5. Generate ephemeral CA
let hick_ca = Arc::new(HickCa::generate()?);

// 6. Pass to network stack
run_network_with_config(
    stream,
    shutdown,
    connection_filter,
    net_config,
    Some(auth_injector),     // existing http_filter parameter
    Some(hick_ca),           // NEW parameter
    auth_config.hosts(),     // NEW: set of hosts to TLS-intercept
);

// 7. At container boot: inject CA cert + placeholder env vars
inject_ca_cert(&mut container, &hick_ca).await;
inject_auth_placeholders(&mut container, &auth_config.rules, &container_token);
```

## Implementation Plan

### Phase 1: TLS Interception Foundation

**Crate:** `network`

1. Add `rustls`, `tokio-rustls`, `rcgen` dependencies
2. Implement `HickCa` — ephemeral CA generation, host cert generation, cert cache
3. Implement `extract_sni` — parse TLS ClientHello for SNI extension
4. Implement `tls_mitm_relay` — TLS termination on both sides, inner HTTP relay
5. Modify `network_loop.rs` routing to support TLS interception branch
6. Add `hick_ca` and `tls_hosts` parameters to `run_network_with_config`
7. Implement CA cert injection into container at boot time

**Tests:**
- Unit: SNI extraction from sample ClientHello bytes
- Unit: CA generation, host cert generation, cert chain validation
- Integration: TLS MITM relay with mock HTTP server over TLS
- Integration: container makes HTTPS request through intercepted path

### Phase 2: Auth Injection

**Crate:** `network` (new module `auth_injector`)

1. Define `AuthRule`, `AuthConfig` structs
2. Implement TOML config loading
3. Implement `AuthInjector` as `HttpFilter`
4. Implement placeholder stripping
5. Wire `AuthInjector` into the `HttpFilter` chain

**Tests:**
- Unit: rule matching
- Unit: header injection with format strings
- Unit: placeholder stripping
- Integration: end-to-end auth injection through TLS MITM

### Phase 3: Secret Cache

**Crate:** `hick-secrets`

1. Add `dashmap` and `secrecy` dependencies
2. Implement `SecretCache` with TTL
3. Unit tests for cache hit/miss/expiry/eviction

### Phase 4: Provider Chain

**Crate:** `hick-secrets`

1. Implement `SecretsProviderChain`
2. Implement `EnvSecretsProvider`
3. Unit tests for chain resolution order, NotFound fallthrough, error propagation

### Phase 5: Container Integration

**Crate:** `hick-agent`

1. Load `auth-rules.toml` at startup
2. Build provider chain based on available configuration
3. Generate HickCa
4. Wire AuthInjector into network stack
5. Inject CA cert and placeholder env vars at container boot
6. End-to-end test: container Python script using OpenAI client succeeds
   without explicit API key

### Phase 6: Capability Gating (hardening)

**Crate:** `hick-token`, `network`

1. Add `allows_secret` to `ContainerCapabilities`
2. Pass container capabilities to AuthInjector per-connection
3. Enforce capability check before secret resolution
4. Audit log denied attempts
5. Test: container without secret capability gets 401, not injected key

## Future Work (Out of Scope)

- **AWS SigV4 request signing** — requires re-signing the full request (body,
  headers, query params) with HMAC-SHA256. Complex but architecturally possible
  as another `HttpFilter` implementation.
- **OAuth2 token refresh** — the AuthInjector could detect 401 responses,
  refresh the token via the OAuth2 flow, and retry. Requires state per auth rule.
- **Cloud secrets providers** — Vault, AWS Secrets Manager, GCP Secret Manager.
  Each is a new `SecretsProvider` implementation.
- **Persistent CA** — store the Hick CA keypair in rage-encrypted storage so
  containers don't need CA re-injection across sessions.
- **HTTP/2 support** — the current HTTP parser handles HTTP/1.x only. HTTP/2
  uses binary framing and would need a separate parser. Most API clients
  negotiate HTTP/1.1 when offered, so this is not urgent.
- **Per-container auth rules** — auth rules scoped to individual containers via
  hick XML `<allow secret=...>` syntax. Currently, rules are global to the agent
  session.

## Security Considerations

1. **CA key protection.** The ephemeral CA private key lives only in process
   memory. It is never written to disk unless explicitly persisted (via rage).
   If the process is compromised, the CA key is exposed — but so is everything
   else. The ephemeral approach limits the blast radius to a single session.

2. **Placeholder leakage.** Even with stripping, a container could construct a
   raw TCP connection to an IP (bypassing DNS, thus bypassing SNI-based
   interception) and send the placeholder. The placeholder is not a secret and
   carries no authorization, so this is harmless — the remote API rejects it.

3. **Secret caching.** Cached secrets live in host process memory (not WASM
   memory). They are zeroized on drop via the `secrecy` crate. The cache TTL
   bounds how long a secret persists after last use.

4. **Capability attenuation.** Macaroon tokens are append-only (can add caveats,
   never remove them). A container cannot escalate its secret access. An inner
   container (forked from an outer one) can only have equal or fewer capabilities.

5. **Audit trail.** All auth injection events are logged with container ID,
   secret name, and destination. This provides forensic capability without
   exposing secret values.
