//! Agent security configuration.
//!
//! Loads capability restrictions from `.hick/agent-security.toml` and converts
//! them to [`ContainerCapabilities`] used by the sandbox executor.

use std::path::Path;

use hick_classify::Classification;
use hick_sink::{SinkDef, SinkSlot};
use hick_token::ContainerCapabilities;
use serde::Deserialize;

/// A host:port entry for network access rules.
#[derive(Debug, Clone, Deserialize)]
pub struct HostEntry {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: String,
}

fn default_port() -> String {
    "*".to_string()
}

/// File access configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileAccessConfig {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

/// A slot within a sink definition (TOML-friendly).
#[derive(Debug, Clone, Deserialize)]
pub struct SinkSlotConfig {
    pub name: String,
    #[serde(default)]
    pub accepted: Vec<serde_json::Value>,
    #[serde(default)]
    pub rejected: Vec<serde_json::Value>,
}

/// A sink definition entry (TOML-friendly).
///
/// ```toml
/// [[sinks]]
/// sink_type = "script-execution"
///
/// [[sinks.slots]]
/// name = "code"
/// rejected = [{"sensitivity": "secret"}]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct SinkDefConfig {
    pub sink_type: String,
    #[serde(default)]
    pub slots: Vec<SinkSlotConfig>,
}

/// Top-level agent security configuration loaded from `.hick/agent-security.toml`.
///
/// ```toml
/// deny_all_network = true
/// policy_script = ".hick/agent-policy.py"
///
/// [[allowed_hosts]]
/// host = "pypi.org"
/// port = "443"
///
/// [file_access]
/// read = ["/workspace/**"]
/// write = ["/workspace/**"]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AgentSecurityConfig {
    /// When true, blocks all network access except explicitly allowed hosts.
    #[serde(default)]
    pub deny_all_network: bool,
    /// Hosts that the sandbox is allowed to connect to.
    #[serde(default)]
    pub allowed_hosts: Vec<HostEntry>,
    /// File access patterns (inside the container).
    #[serde(default)]
    pub file_access: FileAccessConfig,
    /// Optional path to a Python policy script for evaluating data flow rules.
    #[serde(default)]
    pub policy_script: Option<String>,
    /// Sink definitions for API call gating.
    #[serde(default)]
    pub sinks: Vec<SinkDefConfig>,
}

impl Default for AgentSecurityConfig {
    fn default() -> Self {
        Self {
            deny_all_network: true,
            allowed_hosts: vec![
                HostEntry {
                    host: "pypi.org".to_string(),
                    port: "443".to_string(),
                },
                HostEntry {
                    host: "files.pythonhosted.org".to_string(),
                    port: "443".to_string(),
                },
            ],
            file_access: FileAccessConfig {
                read: vec!["/workspace/**".to_string()],
                write: vec!["/workspace/**".to_string()],
            },
            policy_script: None,
            sinks: vec![SinkDefConfig {
                sink_type: "script-execution".to_string(),
                slots: vec![SinkSlotConfig {
                    name: "code".to_string(),
                    accepted: vec![],
                    rejected: vec![serde_json::json!({"sensitivity": "secret"})],
                }],
            }],
        }
    }
}

impl AgentSecurityConfig {
    /// Load from a TOML file, or return defaults if the file does not exist.
    pub fn load_or_default(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Convert sink configs to [`SinkDef`] instances for the sink guard.
    pub fn to_sink_defs(&self) -> Vec<SinkDef> {
        self.sinks
            .iter()
            .map(|cfg| SinkDef {
                sink_type: cfg.sink_type.clone(),
                slots: cfg
                    .slots
                    .iter()
                    .map(|s| SinkSlot {
                        name: s.name.clone(),
                        accepted_classifications: s
                            .accepted
                            .iter()
                            .map(|v| Classification::new(vec![v.clone()]))
                            .collect(),
                        rejected_classifications: s
                            .rejected
                            .iter()
                            .map(|v| Classification::new(vec![v.clone()]))
                            .collect(),
                    })
                    .collect(),
                required_capabilities: vec![format!("api-sink = {}", cfg.sink_type)],
            })
            .collect()
    }

    /// Convert to [`ContainerCapabilities`] for the sandbox executor.
    pub fn to_capabilities(&self) -> ContainerCapabilities {
        let mut caps = ContainerCapabilities::new();

        for host in &self.allowed_hosts {
            caps = caps.allow_network(&host.host, &host.port);
        }
        if self.deny_all_network {
            caps = caps.deny_all_network();
        }

        for pattern in &self.file_access.read {
            caps = caps.allow_file_read(pattern);
        }
        for pattern in &self.file_access.write {
            caps = caps.allow_file_write(pattern);
        }

        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_produces_capabilities() {
        let config = AgentSecurityConfig::default();
        let caps = config.to_capabilities();

        assert!(caps.check_network("pypi.org", 443));
        assert!(!caps.check_network("evil.com", 80));
        assert!(caps.check_file_read("/workspace/src/main.rs"));
        assert!(caps.check_file_write("/workspace/output.txt"));
    }

    #[test]
    fn toml_deserialization() {
        let toml_str = r#"
deny_all_network = true

[[allowed_hosts]]
host = "example.com"
port = "443"

[[allowed_hosts]]
host = "api.example.com"
port = "8080"

[file_access]
read = ["/workspace/**"]
write = ["/workspace/output/**"]
"#;
        let config: AgentSecurityConfig = toml::from_str(toml_str).unwrap();
        assert!(config.deny_all_network);
        assert_eq!(config.allowed_hosts.len(), 2);
        assert_eq!(config.file_access.write.len(), 1);

        let caps = config.to_capabilities();
        assert!(caps.check_network("example.com", 443));
        assert!(caps.check_network("api.example.com", 8080));
        assert!(!caps.check_network("other.com", 80));
        assert!(caps.check_file_read("/workspace/src/main.rs"));
        assert!(caps.check_file_write("/workspace/output/report.txt"));
        assert!(!caps.check_file_write("/workspace/src/main.rs"));
    }

    #[test]
    fn default_config_has_sink_defs() {
        let config = AgentSecurityConfig::default();
        let defs = config.to_sink_defs();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].sink_type, "script-execution");
    }
}
