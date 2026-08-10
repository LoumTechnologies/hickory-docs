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

/// Volume access level for capability tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeAccess {
    /// Read-only access matching a glob pattern.
    Read(String),
    /// Write access matching a glob pattern (implies read).
    Write(String),
}

/// Volume access rule for a container capability token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeRule {
    /// Volume name.
    pub volume: String,
    /// Access level.
    pub access: VolumeAccess,
}

impl VolumeRule {
    pub fn read(volume: &str, pattern: &str) -> Self {
        VolumeRule {
            volume: volume.to_string(),
            access: VolumeAccess::Read(pattern.to_string()),
        }
    }

    pub fn write(volume: &str, pattern: &str) -> Self {
        VolumeRule {
            volume: volume.to_string(),
            access: VolumeAccess::Write(pattern.to_string()),
        }
    }

    /// Convert to a macaroon caveat string.
    pub fn to_caveat(&self) -> String {
        match &self.access {
            VolumeAccess::Read(pattern) => {
                format!("volume-read = {}:{}", self.volume, pattern)
            }
            VolumeAccess::Write(pattern) => {
                format!("volume-write = {}:{}", self.volume, pattern)
            }
        }
    }

    /// Parse from a macaroon caveat string.
    pub fn from_caveat(caveat: &str) -> Option<Self> {
        if let Some(rest) = caveat.strip_prefix("volume-read = ") {
            let (volume, pattern) = rest.split_once(':')?;
            return Some(VolumeRule {
                volume: volume.to_string(),
                access: VolumeAccess::Read(pattern.to_string()),
            });
        }
        if let Some(rest) = caveat.strip_prefix("volume-write = ") {
            let (volume, pattern) = rest.split_once(':')?;
            return Some(VolumeRule {
                volume: volume.to_string(),
                access: VolumeAccess::Write(pattern.to_string()),
            });
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
// Extended rule types
// ---------------------------------------------------------------------------

/// Data access rule: controls which data classification patterns are accessible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataAccessRule {
    pub pattern: String,
}

impl DataAccessRule {
    pub fn to_caveat(&self) -> String {
        format!("data-access = {}", self.pattern)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let pattern = caveat.strip_prefix("data-access = ")?;
        Some(Self {
            pattern: pattern.to_string(),
        })
    }
}

/// API sink rule: controls which sink types are accessible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiSinkRule {
    pub sink_type: String,
}

impl ApiSinkRule {
    pub fn to_caveat(&self) -> String {
        format!("api-sink = {}", self.sink_type)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let sink_type = caveat.strip_prefix("api-sink = ")?;
        Some(Self {
            sink_type: sink_type.to_string(),
        })
    }
}

/// Purpose rule: constrains data usage to a specific context and purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurposeRule {
    pub context: String,
    pub purpose: String,
}

impl PurposeRule {
    pub fn to_caveat(&self) -> String {
        format!("purpose = {}:{}", self.context, self.purpose)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let rest = caveat.strip_prefix("purpose = ")?;
        let (context, purpose) = rest.split_once(':')?;
        Some(Self {
            context: context.to_string(),
            purpose: purpose.to_string(),
        })
    }
}

/// Expiration rule: token is invalid after this timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpiresRule {
    pub expires_at: String,
}

impl ExpiresRule {
    pub fn to_caveat(&self) -> String {
        format!("expires-at = {}", self.expires_at)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let ts = caveat.strip_prefix("expires-at = ")?;
        Some(Self {
            expires_at: ts.to_string(),
        })
    }
}

/// Max calls rule: limits the number of calls to a specific sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaxCallsRule {
    pub sink_type: String,
    pub max_calls: u64,
}

impl MaxCallsRule {
    pub fn to_caveat(&self) -> String {
        format!("max-calls = {}:{}", self.sink_type, self.max_calls)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let rest = caveat.strip_prefix("max-calls = ")?;
        let (sink_type, n) = rest.rsplit_once(':')?;
        let max_calls = n.parse().ok()?;
        Some(Self {
            sink_type: sink_type.to_string(),
            max_calls,
        })
    }
}

/// Recipient rule: constrains the recipient for a sink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipientRule {
    pub sink_type: String,
    pub recipient: String,
}

impl RecipientRule {
    pub fn to_caveat(&self) -> String {
        format!("recipient = {}:{}", self.sink_type, self.recipient)
    }

    pub fn from_caveat(caveat: &str) -> Option<Self> {
        let rest = caveat.strip_prefix("recipient = ")?;
        let (sink_type, recipient) = rest.split_once(':')?;
        Some(Self {
            sink_type: sink_type.to_string(),
            recipient: recipient.to_string(),
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
    pub volume_rules: Vec<VolumeRule>,
    pub data_access_rules: Vec<DataAccessRule>,
    pub api_sink_rules: Vec<ApiSinkRule>,
    pub purpose_rules: Vec<PurposeRule>,
    pub expires_rules: Vec<ExpiresRule>,
    pub max_calls_rules: Vec<MaxCallsRule>,
    pub recipient_rules: Vec<RecipientRule>,
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

    pub fn allow_volume_read(mut self, volume: &str, pattern: &str) -> Self {
        self.volume_rules.push(VolumeRule::read(volume, pattern));
        self
    }

    pub fn allow_volume_write(mut self, volume: &str, pattern: &str) -> Self {
        self.volume_rules.push(VolumeRule::write(volume, pattern));
        self
    }

    pub fn allow_data_access(mut self, pattern: &str) -> Self {
        self.data_access_rules.push(DataAccessRule {
            pattern: pattern.to_string(),
        });
        self
    }

    pub fn allow_api_sink(mut self, sink_type: &str) -> Self {
        self.api_sink_rules.push(ApiSinkRule {
            sink_type: sink_type.to_string(),
        });
        self
    }

    pub fn with_purpose(mut self, context: &str, purpose: &str) -> Self {
        self.purpose_rules.push(PurposeRule {
            context: context.to_string(),
            purpose: purpose.to_string(),
        });
        self
    }

    pub fn with_expires(mut self, expires_at: &str) -> Self {
        self.expires_rules.push(ExpiresRule {
            expires_at: expires_at.to_string(),
        });
        self
    }

    pub fn with_max_calls(mut self, sink_type: &str, max_calls: u64) -> Self {
        self.max_calls_rules.push(MaxCallsRule {
            sink_type: sink_type.to_string(),
            max_calls,
        });
        self
    }

    pub fn with_recipient(mut self, sink_type: &str, recipient: &str) -> Self {
        self.recipient_rules.push(RecipientRule {
            sink_type: sink_type.to_string(),
            recipient: recipient.to_string(),
        });
        self
    }

    /// Whether these capabilities ask for **any** outbound network access:
    /// true exactly when at least one `Allow` rule is present.
    ///
    /// This is the question a sandbox can actually answer. A container is
    /// confined by a switch — the docker executor's `--network` — so the
    /// choice is connectivity or none, and only an explicit
    /// `<hick:allow network="host:port">` turns it on. `DenyAll` alone, or
    /// no rules at all, means no network.
    ///
    /// **This is deliberately stricter than [`Self::check_network`], which
    /// answers a different question.** `check_network` is the policy
    /// predicate: with no `DenyAll` present it treats the container as
    /// unconstrained, because a document that says nothing about the network
    /// has stated no policy. At the enforcement boundary silence must mean
    /// *no*, or every legacy document would be handed connectivity the
    /// moment capabilities started driving the sandbox.
    ///
    /// Note the coarseness in the other direction: the host and port in an
    /// `Allow` rule are **not** enforced by a switch, so a container granted
    /// `github.com:443` can reach whatever the chosen network reaches.
    /// Narrowing that needs an egress proxy.
    pub fn allows_network(&self) -> bool {
        self.network_rules
            .iter()
            .any(|r| matches!(r, NetworkRule::Allow { .. }))
    }

    /// Intersect two capability sets, producing the most restrictive combination.
    ///
    /// For attenuation: the result only contains capabilities present in both sets.
    /// - Network: keep only Allow rules present in both; preserve DenyAll from either
    /// - File/Volume: keep only rules present in both (intersection of allowed paths)
    /// - Secret: keep only secrets present in both
    pub fn intersect(&self, restrictor: &ContainerCapabilities) -> ContainerCapabilities {
        let network_rules = {
            let mut rules = Vec::new();
            // Preserve DenyAll from either side
            let has_deny_self = self
                .network_rules
                .iter()
                .any(|r| matches!(r, NetworkRule::DenyAll));
            let has_deny_other = restrictor
                .network_rules
                .iter()
                .any(|r| matches!(r, NetworkRule::DenyAll));
            if has_deny_self || has_deny_other {
                rules.push(NetworkRule::DenyAll);
            }
            // Keep only Allow rules present in both
            for rule in &self.network_rules {
                if let NetworkRule::Allow { .. } = rule
                    && restrictor.network_rules.contains(rule)
                    && !rules.contains(rule)
                {
                    rules.push(rule.clone());
                }
            }
            rules
        };

        let file_rules = self
            .file_rules
            .iter()
            .filter(|r| restrictor.file_rules.contains(r))
            .cloned()
            .collect();

        let secret_rules = self
            .secret_rules
            .iter()
            .filter(|r| restrictor.secret_rules.contains(r))
            .cloned()
            .collect();

        let volume_rules = self
            .volume_rules
            .iter()
            .filter(|r| restrictor.volume_rules.contains(r))
            .cloned()
            .collect();

        let data_access_rules = self
            .data_access_rules
            .iter()
            .filter(|r| restrictor.data_access_rules.contains(r))
            .cloned()
            .collect();

        let api_sink_rules = self
            .api_sink_rules
            .iter()
            .filter(|r| restrictor.api_sink_rules.contains(r))
            .cloned()
            .collect();

        let purpose_rules = self
            .purpose_rules
            .iter()
            .filter(|r| restrictor.purpose_rules.contains(r))
            .cloned()
            .collect();

        // Expires: keep the most restrictive (earliest) from both
        let mut expires_rules = self.expires_rules.clone();
        expires_rules.extend(restrictor.expires_rules.clone());

        // MaxCalls: keep all from both (each is additive restriction)
        let mut max_calls_rules = self.max_calls_rules.clone();
        for r in &restrictor.max_calls_rules {
            if !max_calls_rules.contains(r) {
                max_calls_rules.push(r.clone());
            }
        }

        // Recipient: keep all from both
        let mut recipient_rules = self.recipient_rules.clone();
        for r in &restrictor.recipient_rules {
            if !recipient_rules.contains(r) {
                recipient_rules.push(r.clone());
            }
        }

        ContainerCapabilities {
            network_rules,
            file_rules,
            secret_rules,
            volume_rules,
            data_access_rules,
            api_sink_rules,
            purpose_rules,
            expires_rules,
            max_calls_rules,
            recipient_rules,
        }
    }

    /// Merge additional caveat-derived rules onto these capabilities (union of rules).
    ///
    /// This is used internally by `CapabilityToken::attenuate()` to track the
    /// union of all caveats in the in-memory capabilities struct. Each caveat is
    /// an *additional restriction* — the macaroon verifier checks all caveats pass.
    /// The in-memory capabilities struct is a convenience cache; the macaroon is
    /// the source of truth for enforcement.
    ///
    /// **Do not use this for capability widening** — it must only be called
    /// alongside macaroon attenuation, which adds corresponding caveats.
    pub fn merge_caveats(&self, additional: &ContainerCapabilities) -> ContainerCapabilities {
        let mut caps = self.clone();

        for r in &additional.network_rules {
            if !caps.network_rules.contains(r) {
                caps.network_rules.push(r.clone());
            }
        }
        for r in &additional.file_rules {
            if !caps.file_rules.contains(r) {
                caps.file_rules.push(r.clone());
            }
        }
        for r in &additional.secret_rules {
            if !caps.secret_rules.contains(r) {
                caps.secret_rules.push(r.clone());
            }
        }
        for r in &additional.volume_rules {
            if !caps.volume_rules.contains(r) {
                caps.volume_rules.push(r.clone());
            }
        }
        for r in &additional.data_access_rules {
            if !caps.data_access_rules.contains(r) {
                caps.data_access_rules.push(r.clone());
            }
        }
        for r in &additional.api_sink_rules {
            if !caps.api_sink_rules.contains(r) {
                caps.api_sink_rules.push(r.clone());
            }
        }
        for r in &additional.purpose_rules {
            if !caps.purpose_rules.contains(r) {
                caps.purpose_rules.push(r.clone());
            }
        }
        for r in &additional.expires_rules {
            if !caps.expires_rules.contains(r) {
                caps.expires_rules.push(r.clone());
            }
        }
        for r in &additional.max_calls_rules {
            if !caps.max_calls_rules.contains(r) {
                caps.max_calls_rules.push(r.clone());
            }
        }
        for r in &additional.recipient_rules {
            if !caps.recipient_rules.contains(r) {
                caps.recipient_rules.push(r.clone());
            }
        }

        caps
    }

    /// Build capabilities from a list of caveat strings.
    pub fn from_caveats(caveats: &[String]) -> Self {
        let mut caps = Self::new();
        for caveat in caveats {
            if let Some(r) = NetworkRule::from_caveat(caveat) {
                caps.network_rules.push(r);
            } else if let Some(r) = FileRule::from_caveat(caveat) {
                caps.file_rules.push(r);
            } else if let Some(r) = SecretRule::from_caveat(caveat) {
                caps.secret_rules.push(r);
            } else if let Some(r) = VolumeRule::from_caveat(caveat) {
                caps.volume_rules.push(r);
            } else if let Some(r) = DataAccessRule::from_caveat(caveat) {
                caps.data_access_rules.push(r);
            } else if let Some(r) = ApiSinkRule::from_caveat(caveat) {
                caps.api_sink_rules.push(r);
            } else if let Some(r) = PurposeRule::from_caveat(caveat) {
                caps.purpose_rules.push(r);
            } else if let Some(r) = ExpiresRule::from_caveat(caveat) {
                caps.expires_rules.push(r);
            } else if let Some(r) = MaxCallsRule::from_caveat(caveat) {
                caps.max_calls_rules.push(r);
            } else if let Some(r) = RecipientRule::from_caveat(caveat) {
                caps.recipient_rules.push(r);
            }
        }
        caps
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
        for rule in &self.volume_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.data_access_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.api_sink_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.purpose_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.expires_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.max_calls_rules {
            caveats.push(rule.to_caveat());
        }
        for rule in &self.recipient_rules {
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
    /// Denies access when no file rules are defined (closed-by-default).
    pub fn check_file_read(&self, path: &str) -> bool {
        if self.file_rules.is_empty() {
            return false;
        }
        self.file_rules.iter().any(|r| match r {
            FileRule::Read(pattern) => path_matches(path, pattern),
            FileRule::Write(pattern) => path_matches(path, pattern), // write implies read
            #[allow(unreachable_patterns)]
            _ => false,
        })
    }

    /// Check if file write is allowed at the given path.
    /// Denies access when no file rules are defined (closed-by-default).
    pub fn check_file_write(&self, path: &str) -> bool {
        if self.file_rules.is_empty() {
            return false;
        }
        self.file_rules.iter().any(|r| match r {
            FileRule::Write(pattern) => path_matches(path, pattern),
            _ => false,
        })
    }

    /// Check if volume read access is allowed.
    /// Denies access when no volume rules are defined for the given volume (closed-by-default).
    pub fn check_volume_read(&self, volume: &str, path: &str) -> bool {
        let relevant: Vec<_> = self
            .volume_rules
            .iter()
            .filter(|r| r.volume == volume)
            .collect();
        if relevant.is_empty() {
            return false;
        }
        relevant.iter().any(|r| match &r.access {
            VolumeAccess::Read(pattern) | VolumeAccess::Write(pattern) => {
                path_matches(path, pattern)
            }
        })
    }

    /// Check if volume write access is allowed.
    /// Denies access when no volume rules are defined for the given volume (closed-by-default).
    pub fn check_volume_write(&self, volume: &str, path: &str) -> bool {
        let relevant: Vec<_> = self
            .volume_rules
            .iter()
            .filter(|r| r.volume == volume)
            .collect();
        if relevant.is_empty() {
            return false;
        }
        relevant.iter().any(|r| match &r.access {
            VolumeAccess::Write(pattern) => path_matches(path, pattern),
            _ => false,
        })
    }

    /// Check if data access to the given classification pattern is allowed.
    /// Denies access when no data access rules are defined (closed-by-default).
    pub fn check_data_access(&self, pattern: &str) -> bool {
        if self.data_access_rules.is_empty() {
            return false;
        }
        self.data_access_rules.iter().any(|r| r.pattern == pattern)
    }

    /// Check if the given API sink type is allowed.
    /// If no API sink rules are defined, no sinks are allowed.
    pub fn check_api_sink(&self, sink_type: &str) -> bool {
        if self.api_sink_rules.is_empty() {
            return false;
        }
        self.api_sink_rules.iter().any(|r| r.sink_type == sink_type)
    }

    /// Check if the token has expired given the current time (ISO 8601).
    pub fn check_expires(&self, now: &str) -> bool {
        if self.expires_rules.is_empty() {
            return true;
        }
        let now_dt = match chrono::DateTime::parse_from_rfc3339(now) {
            Ok(dt) => dt,
            Err(_) => return false,
        };
        self.expires_rules.iter().all(|r| {
            match chrono::DateTime::parse_from_rfc3339(&r.expires_at) {
                Ok(exp) => now_dt < exp,
                Err(_) => false,
            }
        })
    }

    /// Check if the max calls limit for a sink has been reached.
    pub fn check_max_calls(&self, sink_type: &str, current_count: u64) -> bool {
        for rule in &self.max_calls_rules {
            if rule.sink_type == sink_type && current_count >= rule.max_calls {
                return false;
            }
        }
        true
    }

    /// Check if the recipient matches any recipient constraints for the sink.
    pub fn check_recipient(&self, sink_type: &str, recipient: &str) -> bool {
        let relevant: Vec<_> = self
            .recipient_rules
            .iter()
            .filter(|r| r.sink_type == sink_type)
            .collect();
        if relevant.is_empty() {
            return true;
        }
        relevant.iter().any(|r| r.recipient == recipient)
    }
}

/// Simple glob-style path matching.
///
/// Supports:
/// - `**` matches everything
/// - `prefix/**` matches anything under `prefix/`
/// - Trailing `*` matches any suffix after prefix
/// - Exact match otherwise
fn path_matches(path: &str, pattern: &str) -> bool {
    if pattern == "**" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("**") {
        return path.starts_with(prefix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return path.starts_with(prefix);
    }
    path == pattern
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
        || VolumeRule::from_caveat(s).is_some()
        || DataAccessRule::from_caveat(s).is_some()
        || ApiSinkRule::from_caveat(s).is_some()
        || PurposeRule::from_caveat(s).is_some()
        || ExpiresRule::from_caveat(s).is_some()
        || MaxCallsRule::from_caveat(s).is_some()
        || RecipientRule::from_caveat(s).is_some()
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

        // Track the union of caveats in the in-memory capabilities struct.
        // Actual enforcement is via the macaroon verifier checking all caveats.
        let merged = self.capabilities.merge_caveats(additional);

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

    /// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
    #[test]
    fn allows_network_is_deny_by_default() {
        // Silence is not consent: a document that never mentions the network
        // gets none, which is the opposite of `check_network`'s reading and
        // the reason the two are separate methods.
        assert!(!ContainerCapabilities::new().allows_network());
        assert!(
            !ContainerCapabilities::new()
                .deny_all_network()
                .allows_network()
        );
        assert!(
            !ContainerCapabilities::new()
                .allow_file_read("/input/*")
                .allows_network()
        );

        // An explicit grant, with or without a surrounding default-deny.
        assert!(
            ContainerCapabilities::new()
                .allow_network("github.com", "443")
                .allows_network()
        );
        assert!(
            ContainerCapabilities::new()
                .deny_all_network()
                .allow_network("pypi.org", "443")
                .allows_network()
        );
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

    #[test]
    fn volume_rule_caveat_roundtrip() {
        let read_rule = VolumeRule::read("project", "**");
        let caveat = read_rule.to_caveat();
        assert_eq!(caveat, "volume-read = project:**");
        let parsed = VolumeRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, read_rule);

        let write_rule = VolumeRule::write("output", "Controllers/**");
        let caveat = write_rule.to_caveat();
        assert_eq!(caveat, "volume-write = output:Controllers/**");
        let parsed = VolumeRule::from_caveat(&caveat).unwrap();
        assert_eq!(parsed, write_rule);
    }

    #[test]
    fn volume_check_no_rules_denies_all() {
        let caps = ContainerCapabilities::new();
        assert!(!caps.check_volume_read("any-vol", "any/path"));
        assert!(!caps.check_volume_write("any-vol", "any/path"));
    }

    #[test]
    fn volume_check_with_rules() {
        let caps = ContainerCapabilities::new()
            .allow_volume_read("project", "**")
            .allow_volume_write("project", "Controllers/*");

        assert!(caps.check_volume_read("project", "Models/User.cs"));
        assert!(caps.check_volume_write("project", "Controllers/Home.cs"));
        assert!(!caps.check_volume_write("project", "Models/User.cs"));

        // Different volume has no rules → denied (closed-by-default)
        assert!(!caps.check_volume_read("other", "any/path"));
    }

    #[test]
    fn volume_token_mint_and_verify() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_volume_read("shared", "**")
            .allow_volume_write("shared", "output/**");

        let token = authority.mint("worker", &caps).unwrap();
        authority.verify(&token).unwrap();

        let caveats = caps.to_caveats();
        assert!(caveats.contains(&"volume-read = shared:**".to_string()));
        assert!(caveats.contains(&"volume-write = shared:output/**".to_string()));
    }

    #[test]
    fn intersect_network_rules() {
        let a = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network();
        let b = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let result = a.intersect(&b);
        // Only github.com:443 is in both; DenyAll preserved
        assert!(result.check_network("github.com", 443));
        assert!(!result.check_network("pypi.org", 443));
    }

    #[test]
    fn intersect_file_rules() {
        let a = ContainerCapabilities::new()
            .allow_file_read("/input/*")
            .allow_file_write("/output/*");
        let b = ContainerCapabilities::new().allow_file_read("/input/*");

        let result = a.intersect(&b);
        assert!(result.check_file_read("/input/data.txt"));
        // /output/* write only in a, not b → removed
        assert!(!result.check_file_write("/output/report.html"));
    }

    #[test]
    fn intersect_secrets() {
        let a = ContainerCapabilities::new()
            .with_secret("TOKEN", "my-token")
            .with_secret("KEY", "my-key");
        let b = ContainerCapabilities::new().with_secret("TOKEN", "my-token");

        let result = a.intersect(&b);
        assert_eq!(result.secret_rules.len(), 1);
        assert_eq!(result.secret_rules[0].env_var, "TOKEN");
    }

    #[test]
    fn attenuate_merges_restrictions() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .allow_file_read("/input/*");

        let token = authority.mint("sandbox", &caps).unwrap();

        // Attenuate: add deny-all (original allows still in effect)
        let restrictor = ContainerCapabilities::new().deny_all_network();

        let attenuated = token.attenuate(&restrictor).unwrap();
        authority.verify(&attenuated).unwrap();

        // Original allows preserved, but non-whitelisted blocked by deny-all
        assert!(attenuated.capabilities().check_network("github.com", 443));
        assert!(attenuated.capabilities().check_network("pypi.org", 443));
        assert!(!attenuated.capabilities().check_network("evil.com", 80));
    }

    #[test]
    fn intersect_produces_common_subset() {
        let source = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .allow_network("pypi.org", "443")
            .deny_all_network()
            .allow_file_write("/output/*");

        let restrictor = ContainerCapabilities::new()
            .allow_network("github.com", "443")
            .deny_all_network();

        let result = source.intersect(&restrictor);

        assert!(result.check_network("github.com", 443));
        assert!(!result.check_network("pypi.org", 443));
        // file_write not in restrictor → removed
        assert!(result.file_rules.is_empty());
    }

    #[test]
    fn extended_caveat_roundtrip() {
        let da = DataAccessRule {
            pattern: "secret:ssn".into(),
        };
        let caveat = da.to_caveat();
        assert_eq!(caveat, "data-access = secret:ssn");
        assert_eq!(DataAccessRule::from_caveat(&caveat).unwrap(), da);

        let sink = ApiSinkRule {
            sink_type: "email".into(),
        };
        let caveat = sink.to_caveat();
        assert_eq!(caveat, "api-sink = email");
        assert_eq!(ApiSinkRule::from_caveat(&caveat).unwrap(), sink);

        let purpose = PurposeRule {
            context: "email".into(),
            purpose: "recipient".into(),
        };
        let caveat = purpose.to_caveat();
        assert_eq!(caveat, "purpose = email:recipient");
        assert_eq!(PurposeRule::from_caveat(&caveat).unwrap(), purpose);

        let expires = ExpiresRule {
            expires_at: "2026-02-17T15:00:00Z".into(),
        };
        let caveat = expires.to_caveat();
        assert_eq!(caveat, "expires-at = 2026-02-17T15:00:00Z");
        assert_eq!(ExpiresRule::from_caveat(&caveat).unwrap(), expires);

        let max = MaxCallsRule {
            sink_type: "email".into(),
            max_calls: 1,
        };
        let caveat = max.to_caveat();
        assert_eq!(caveat, "max-calls = email:1");
        assert_eq!(MaxCallsRule::from_caveat(&caveat).unwrap(), max);

        let recip = RecipientRule {
            sink_type: "email".into(),
            recipient: "alice@co.com".into(),
        };
        let caveat = recip.to_caveat();
        assert_eq!(caveat, "recipient = email:alice@co.com");
        assert_eq!(RecipientRule::from_caveat(&caveat).unwrap(), recip);
    }

    #[test]
    fn extended_caps_check_methods() {
        let caps = ContainerCapabilities::new()
            .allow_data_access("public:*")
            .allow_api_sink("email")
            .with_expires("2026-12-31T23:59:59Z")
            .with_max_calls("email", 2)
            .with_recipient("email", "alice@co.com");

        assert!(caps.check_data_access("public:*"));
        assert!(!caps.check_data_access("secret:ssn"));

        assert!(caps.check_api_sink("email"));
        assert!(!caps.check_api_sink("slack"));

        // Before expiry
        assert!(caps.check_expires("2026-06-01T00:00:00Z"));
        // After expiry
        assert!(!caps.check_expires("2027-01-01T00:00:00Z"));

        // Max calls: 0 < 2 ok, 1 < 2 ok, 2 >= 2 fail
        assert!(caps.check_max_calls("email", 0));
        assert!(caps.check_max_calls("email", 1));
        assert!(!caps.check_max_calls("email", 2));

        assert!(caps.check_recipient("email", "alice@co.com"));
        assert!(!caps.check_recipient("email", "bob@co.com"));
        // No recipient rules for slack → allowed
        assert!(caps.check_recipient("slack", "anyone"));
    }

    #[test]
    fn extended_token_mint_and_verify() {
        let authority = test_authority();
        let caps = ContainerCapabilities::new()
            .allow_api_sink("email")
            .allow_data_access("public:*")
            .with_expires("2026-12-31T23:59:59Z")
            .with_max_calls("email", 5)
            .with_recipient("email", "alice@co.com");

        let token = authority.mint("worker", &caps).unwrap();
        authority.verify(&token).unwrap();

        let caveats = caps.to_caveats();
        assert!(caveats.contains(&"api-sink = email".to_string()));
        assert!(caveats.contains(&"data-access = public:*".to_string()));
        assert!(caveats.contains(&"expires-at = 2026-12-31T23:59:59Z".to_string()));
        assert!(caveats.contains(&"max-calls = email:5".to_string()));
        assert!(caveats.contains(&"recipient = email:alice@co.com".to_string()));
    }
}
