//! Macaroon-based capability token system for hick containers.
//!
//! Each container receives a macaroon token that encodes its capabilities
//! (network access, file access, secrets). Tokens can be attenuated (add
//! restrictions) but never escalated.

use std::fmt;

use macaroon::{ByteString, Macaroon, MacaroonKey, Verifier};

// ---------------------------------------------------------------------------
// Capability types
// ---------------------------------------------------------------------------

/// Network access rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkRule {
    /// Allow traffic to a specific host:port (port can be `"*"` for wildcard).
    Allow { host: String, port: String },
    /// Deny all network traffic.
    DenyAll,
}

impl NetworkRule {
    pub fn allow(host: &str, port: &str) -> Self {
        NetworkRule::Allow {
            host: host.to_string(),
            port: port.to_string(),
        }
    }

    pub fn deny_all() -> Self {
        NetworkRule::DenyAll
    }

    /// Convert to a macaroon caveat string.
    pub fn to_caveat(&self) -> String {
        match self {
            NetworkRule::Allow { host, port } => format!("network = allow {host}:{port}"),
            NetworkRule::DenyAll => "network = deny *".to_string(),
        }
    }

    /// Parse from a macaroon caveat string.
    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let caveat = caveat.strip_prefix("network = ")?;
        if caveat == "deny *" {
            return Some(NetworkRule::DenyAll);
        }
        let rest = caveat.strip_prefix("allow ")?;
        let (host, port) = rest.rsplit_once(':')?;
        Some(NetworkRule::Allow {
            host: host.to_string(),
            port: port.to_string(),
        })
    }
}

/// File access rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileRule {
    Read(String),
    Write(String),
}

impl FileRule {
    pub fn to_caveat(&self) -> String {
        match self {
            FileRule::Read(path) => format!("file-read = {path}"),
            FileRule::Write(path) => format!("file-write = {path}"),
        }
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        if let Some(path) = caveat.strip_prefix("file-read = ") {
            return Some(FileRule::Read(path.to_string()));
        }
        if let Some(path) = caveat.strip_prefix("file-write = ") {
            return Some(FileRule::Write(path.to_string()));
        }
        None
    }
}

/// Secret injection rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretRule {
    pub env_var: String,
    pub secret_name: String,
}

impl SecretRule {
    pub fn to_caveat(&self) -> String {
        format!("secret = {}:{}", self.env_var, self.secret_name)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let rest = caveat.strip_prefix("secret = ")?;
        let (env_var, secret_name) = rest.split_once(':')?;
        Some(SecretRule {
            env_var: env_var.to_string(),
            secret_name: secret_name.to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// ContainerCapabilities
// ---------------------------------------------------------------------------

/// The complete set of capabilities for a container.
#[derive(Debug, Clone, Default)]
pub struct ContainerCapabilities {
    pub network_rules: Vec<NetworkRule>,
    pub file_rules: Vec<FileRule>,
    pub secret_rules: Vec<SecretRule>,
}

impl ContainerCapabilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_network(mut self, host: &str, port: &str) -> Self {
        self.network_rules.push(NetworkRule::allow(host, port));
        self
    }

    pub fn deny_all_network(mut self) -> Self {
        self.network_rules.push(NetworkRule::deny_all());
        self
    }

    pub fn allow_file_read(mut self, path: &str) -> Self {
        self.file_rules.push(FileRule::Read(path.to_string()));
        self
    }

    pub fn allow_file_write(mut self, path: &str) -> Self {
        self.file_rules.push(FileRule::Write(path.to_string()));
        self
    }

    pub fn with_secret(mut self, env_var: &str, secret_name: &str) -> Self {
        self.secret_rules.push(SecretRule {
            env_var: env_var.to_string(),
            secret_name: secret_name.to_string(),
        });
        self
    }

    /// Convert all capabilities to caveat strings.
    pub fn to_caveats(&self) -> Vec<String> {
        let mut caveats = Vec::new();
        for rule in &self.network_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.file_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.secret_rules {
            caveats.push(rule.to_caveat());
        }
        caveats
    }

    /// Check if network access to `host:port` is allowed.
    ///
    /// Logic: if any `DenyAll` rule exists, only explicit `Allow` rules permit
    /// traffic. If no `DenyAll`, all traffic is allowed by default.
    pub fn check_network(&self, host: &str, port: u16) -> bool {
        let has_deny_all = self
            .network_rules
            .iter()
            .any(|r| matches!(r, NetworkRule::DenyAll));
        if !has_deny_all {
            return true;
        }

        let port_str = port.to_string();
        self.network_rules.iter().any(|r| match r {
            NetworkRule::Allow { host: h, port: p } => {
                let host_match = h == host || h == "*";
                let port_match = p == "*" || p == &port_str;
                host_match && port_match
            }
            NetworkRule::DenyAll => false,
        })
    }

    /// Check if file read is allowed at the given path.
    pub fn check_file_read(&self, path: &str) -> bool {
        // If no file rules defined, allow all reads
        if self.file_rules.is_empty() {
            return true;
        }
        self.file_rules.iter().any(|r| match r {
            FileRule::Read(pattern) => path_matches(path, pattern),
            FileRule::Write(pattern) => path_matches(path, pattern), // write implies read
            #[allow(unreachable_patterns)]
            _ => false,
        })
    }

    /// Check if file write is allowed at the given path.
    pub fn check_file_write(&self, path: &str) -> bool {
        if self.file_rules.is_empty() {
            return true;
        }
        self.file_rules.iter().any(|r| match r {
            FileRule::Write(pattern) => path_matches(path, pattern),
            _ => false,
        })
    }
}

/// Simple glob-style path matching (supports trailing `*`).
fn path_matches(path: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        path.starts_with(prefix)
    } else {
        path == pattern
    }
}

// ---------------------------------------------------------------------------
// TokenAuthority
// ---------------------------------------------------------------------------

/// The authority that mints and verifies capability tokens.
pub struct TokenAuthority {
    root_key: MacaroonKey,
}

/// Errors from token operations.
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("macaroon error: {0}")]
    Macaroon(String),

    #[error("verification failed: {0}")]
    Verification(String),

    #[error("invalid caveat: {0}")]
    InvalidCaveat(String),
}

impl TokenAuthority {
    /// Create a new authority with the given root key bytes.
    pub fn new(key_bytes: &[u8]) -> Self {
        let root_key = MacaroonKey::generate(key_bytes);
        Self { root_key }
    }

    /// Mint a new capability token for a container.
    pub fn mint(
        &self,
        container_name: &str,
        capabilities: &ContainerCapabilities,
    ) -> Result<CapabilityToken, TokenError> {
        let identifier = format!("container-{container_name}");
        let location = "hick://gateway:8080".to_string();

        let mut macaroon = Macaroon::create(Some(location), &self.root_key, identifier.into())
            .map_err(|e| TokenError::Macaroon(format!("{e:?}")))?;

        for caveat in capabilities.to_caveats() {
            macaroon.add_first_party_caveat(caveat.into());
        }

        Ok(CapabilityToken {
            macaroon,
            capabilities: capabilities.clone(),
        })
    }

    /// Verify a token and extract its capabilities.
    pub fn verify(&self, token: &CapabilityToken) -> Result<(), TokenError> {
        let mut verifier = Verifier::default();

        // Satisfy all known caveat patterns
        verifier.satisfy_general(verify_hick_caveat);

        verifier
            .verify(&token.macaroon, &self.root_key, Default::default())
            .map_err(|e| TokenError::Verification(format!("{e:?}")))
    }
}

/// Verifier callback for hick capability caveats.
fn verify_hick_caveat(caveat: &ByteString) -> bool {
    let s = match std::str::from_utf8(caveat.as_ref()) {
        Ok(s) => s,
        Err(_) => return false,
    };
    NetworkRule::from_caveat(s).is_some()
        || FileRule::from_caveat(s).is_some()
        || SecretRule::from_caveat(s).is_some()
}

// ---------------------------------------------------------------------------
// CapabilityToken
// ---------------------------------------------------------------------------

/// A capability token (macaroon) with parsed capabilities.
pub struct CapabilityToken {
    macaroon: Macaroon,
    capabilities: ContainerCapabilities,
}

impl CapabilityToken {
    /// Get the parsed capabilities.
    pub fn capabilities(&self) -> &ContainerCapabilities {
        &self.capabilities
    }

    /// Attenuate this token by adding more restrictions.
    /// Returns a new token with the additional caveats. The original is unchanged.
    pub fn attenuate(
        &self,
        additional: &ContainerCapabilities,
    ) -> Result<CapabilityToken, TokenError> {
        let mut new_macaroon = self.macaroon.clone();

        for caveat in additional.to_caveats() {
            new_macaroon.add_first_party_caveat(caveat.into());
        }

        // Merge capabilities: attenuated = original + additional restrictions
        let mut merged = self.capabilities.clone();
        merged
            .network_rules
            .extend(additional.network_rules.clone());
        merged.file_rules.extend(additional.file_rules.clone());
        merged.secret_rules.extend(additional.secret_rules.clone());

        Ok(CapabilityToken {
            macaroon: new_macaroon,
            capabilities: merged,
        })
    }

    /// Serialize the token to a string (V2 format).
    pub fn serialize(&self) -> Result<String, TokenError> {
        self.macaroon
            .serialize(macaroon::Format::V2)
            .map_err(|e| TokenError::Macaroon(format!("{e:?}")))
    }
}

impl fmt::Debug for CapabilityToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapabilityToken")
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_authority() -> TokenAuthority {
        TokenAuthority::new(b"test-root-key-32bytes-long-xxxxx")
    }

    #[test]
    fn mint_and_verify() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let token = authority.mint("reporter", &caps).unwrap();
        authority.verify(&token).unwrap();
    }

    #[test]
    fn attenuate_adds_restrictions() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network();

        let token = authority.mint("sandbox", &caps).unwrap();

        // Attenuate: also deny all network
        let extra = ContainerCapabilities::new().deny_all_network();
        let attenuated = token.attenuate(&extra).unwrap();

        // Original still verifies
        authority.verify(&token).unwrap();
        // Attenuated also verifies
        authority.verify(&attenuated).unwrap();
    }

    #[test]
    fn network_check_deny_all_blocks() {
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        assert!(caps.check_network("github.com", 443));
        assert!(!caps.check_network("evil.com", 80));
    }

    #[test]
    fn network_check_wildcard_port() {
        let caps = ContainerCapabilities::new()
            .allow_network("example.com", "*")
            .deny_all_network();

        assert!(caps.check_network("example.com", 80));
        assert!(caps.check_network("example.com", 443));
        assert!(!caps.check_network("other.com", 80));
    }

    #[test]
    fn network_check_no_deny_allows_all() {
        let caps = ContainerCapabilities::new().allow_network("github.com", "443");

        assert!(caps.check_network("github.com", 443));
        assert!(caps.check_network("anything.com", 9999));
    }

    #[test]
    fn file_rules_check() {
        let caps = ContainerCapabilities::new()
            .allow_file_read("/input/*")
            .allow_file_write("/output/*");

        assert!(caps.check_file_read("/input/data.txt"));
        assert!(!caps.check_file_read("/secret/key"));
        assert!(caps.check_file_write("/output/report.html"));
        assert!(!caps.check_file_write("/input/data.txt"));
        // Write implies read
        assert!(caps.check_file_read("/output/report.html"));
    }

    #[test]
    fn serialize_token() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new().deny_all_network();
        let token = authority.mint("test", &caps).unwrap();
        let serialized = token.serialize().unwrap();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn caveat_roundtrip() {
        let rule = NetworkRule::allow("github.com", "443");
        let caveat = rule.to_caveat();
        let parsed = NetworkRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, rule);

        let deny = NetworkRule::deny_all();
        let caveat = deny.to_caveat();
        let parsed = NetworkRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, deny);

        let file = FileRule::Write("/output/*".to_string());
        let caveat = file.to_caveat();
        let parsed = FileRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, file);

        let secret = SecretRule {
            env_var: "TOKEN".to_string(),
            secret_name: "my-token".to_string(),
        };
        let caveat = secret.to_caveat();
        let parsed = SecretRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, secret);
    }

    #[test]
    fn capabilities_from_hick_tag() {
        // Simulate building capabilities from parsed hick tags
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network()
            .allow_file_write("/output/*")
            .with_secret("PULUMI_TOKEN", "pulumi-token");

        let caveats = caps.to_caveats();
        assert_eq!(caveats.len(), 5);
        assert!(caveats.contains(&"network = allow github.com:443".to_string()));
        assert!(caveats.contains(&"network = deny *".to_string()));
        assert!(caveats.contains(&"file-write = /output/*".to_string()));
    }
}
