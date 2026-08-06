//! Transcript builder for `.hick` session replay files.
//!
//! Accumulates container declarations, capability rules, volumes, forks,
//! and executed commands, then serializes them to valid `.hick` XML that
//! can be replayed with `hick session.hick`.

// ---------------------------------------------------------------------------
// Capability rules
// ---------------------------------------------------------------------------

/// A single capability rule for a container.
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityRule {
    AllowNetwork { host: String, port: String },
    DenyNetwork,
    AllowFileRead { path: String },
    AllowFileWrite { path: String },
}

// ---------------------------------------------------------------------------
// Transcript actions
// ---------------------------------------------------------------------------

/// An action recorded during a REPL session.
#[derive(Debug, Clone)]
pub enum TranscriptAction {
    ContainerDeclared {
        name: String,
        image: String,
    },
    CapabilityAdded {
        container: String,
        rule: CapabilityRule,
    },
    VolumeCreated {
        name: String,
    },
    VolumeMounted {
        container: String,
        volume: String,
        path: String,
    },
    CommandExecuted {
        container: String,
        command: String,
    },
    Forked {
        from: String,
        to: String,
    },
}

// ---------------------------------------------------------------------------
// Container state (for serialization)
// ---------------------------------------------------------------------------

/// Accumulated state of a container for transcript serialization.
#[derive(Debug, Clone)]
pub struct TranscriptContainer {
    pub name: String,
    pub image: String,
    pub capabilities: Vec<CapabilityRule>,
    pub mounts: Vec<(String, String)>, // (volume, path)
}

// ---------------------------------------------------------------------------
// Transcript builder
// ---------------------------------------------------------------------------

/// Builds a `.hick` XML transcript from accumulated session actions.
pub struct TranscriptBuilder {
    containers: Vec<TranscriptContainer>,
    volumes: Vec<String>,
    forks: Vec<(String, String)>,
    actions: Vec<TranscriptAction>,
}

impl Default for TranscriptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptBuilder {
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
            volumes: Vec::new(),
            forks: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Declare a new container with the given image.
    pub fn declare_container(&mut self, name: &str, image: &str) {
        self.containers.push(TranscriptContainer {
            name: name.to_string(),
            image: image.to_string(),
            capabilities: Vec::new(),
            mounts: Vec::new(),
        });
        self.actions.push(TranscriptAction::ContainerDeclared {
            name: name.to_string(),
            image: image.to_string(),
        });
    }

    /// Add a capability rule to a container.
    pub fn add_capability(&mut self, container: &str, rule: CapabilityRule) {
        if let Some(c) = self.containers.iter_mut().find(|c| c.name == container) {
            c.capabilities.push(rule.clone());
        }
        self.actions.push(TranscriptAction::CapabilityAdded {
            container: container.to_string(),
            rule,
        });
    }

    /// Create a named volume.
    pub fn create_volume(&mut self, name: &str) {
        if !self.volumes.contains(&name.to_string()) {
            self.volumes.push(name.to_string());
        }
        self.actions.push(TranscriptAction::VolumeCreated {
            name: name.to_string(),
        });
    }

    /// Mount a volume in a container.
    pub fn mount_volume(&mut self, container: &str, volume: &str, path: &str) {
        if let Some(c) = self.containers.iter_mut().find(|c| c.name == container) {
            c.mounts.push((volume.to_string(), path.to_string()));
        }
        self.actions.push(TranscriptAction::VolumeMounted {
            container: container.to_string(),
            volume: volume.to_string(),
            path: path.to_string(),
        });
    }

    /// Record a command execution.
    pub fn record_command(&mut self, container: &str, command: &str) {
        self.actions.push(TranscriptAction::CommandExecuted {
            container: container.to_string(),
            command: command.to_string(),
        });
    }

    /// Record a fork.
    pub fn record_fork(&mut self, from: &str, to: &str) {
        self.forks.push((from.to_string(), to.to_string()));
        self.actions.push(TranscriptAction::Forked {
            from: from.to_string(),
            to: to.to_string(),
        });
    }

    /// Rename a container. Only valid before any commands have been executed.
    pub fn rename_container(&mut self, old_name: &str, new_name: &str) -> bool {
        if let Some(c) = self.containers.iter_mut().find(|c| c.name == old_name) {
            c.name = new_name.to_string();
            true
        } else {
            false
        }
    }

    /// Check if a container exists.
    pub fn has_container(&self, name: &str) -> bool {
        self.containers.iter().any(|c| c.name == name)
    }

    /// Get the list of container names.
    pub fn container_names(&self) -> Vec<String> {
        self.containers.iter().map(|c| c.name.clone()).collect()
    }

    /// Get the list of volume names.
    pub fn volume_names(&self) -> Vec<String> {
        self.volumes.clone()
    }

    /// Get the mounts for a container.
    pub fn container_mounts(&self, name: &str) -> Vec<(String, String)> {
        self.containers
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.mounts.clone())
            .unwrap_or_default()
    }

    /// Get the capabilities for a container.
    pub fn container_capabilities(&self, name: &str) -> Vec<CapabilityRule> {
        self.containers
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.capabilities.clone())
            .unwrap_or_default()
    }

    /// Serialize the transcript to `.hick` XML.
    ///
    /// Order:
    /// 1. XML declaration + `<hick:doc>` open
    /// 2. Container declarations (with nested deny/allow children)
    /// 3. Volume declarations
    /// 4. Fork declarations
    /// 5. Exec tags for each CommandExecuted action
    /// 6. `</hick:doc>` close
    pub fn to_hick_xml(&self) -> String {
        let mut out = String::new();

        // XML declaration + root element
        out.push_str(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
"#,
        );

        // Container declarations
        for container in &self.containers {
            if container.capabilities.is_empty() {
                out.push_str(&format!(
                    "<hick:container name=\"{}\" image=\"{}\" />\n",
                    container.name, container.image,
                ));
            } else {
                out.push_str(&format!(
                    "<hick:container name=\"{}\" image=\"{}\">\n",
                    container.name, container.image,
                ));
                for cap in &container.capabilities {
                    match cap {
                        CapabilityRule::DenyNetwork => {
                            out.push_str("<hick:deny network=\"*\" />\n");
                        }
                        CapabilityRule::AllowNetwork { host, port } => {
                            out.push_str(&format!("<hick:allow network=\"{host}:{port}\" />\n",));
                        }
                        CapabilityRule::AllowFileRead { path } => {
                            out.push_str(&format!("<hick:allow file-read=\"{path}\" />\n",));
                        }
                        CapabilityRule::AllowFileWrite { path } => {
                            out.push_str(&format!("<hick:allow file-write=\"{path}\" />\n",));
                        }
                    }
                }
                out.push_str("</hick:container>\n");
            }
        }

        // Volume declarations
        for vol in &self.volumes {
            out.push_str(&format!("<hick:volume name=\"{vol}\" />\n"));
        }

        // Fork declarations
        for (from, to) in &self.forks {
            out.push_str(&format!("<hick:fork from=\"{from}\" to=\"{to}\" />\n",));
        }

        // Exec tags for each CommandExecuted action
        for action in &self.actions {
            if let TranscriptAction::CommandExecuted { container, command } = action {
                let mounts = self.container_mounts(container);
                let mount_attr = if mounts.is_empty() {
                    String::new()
                } else {
                    let mount_str: String = mounts
                        .iter()
                        .map(|(v, p)| format!("{v}:{p}"))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(" mount=\"{mount_str}\"")
                };
                out.push_str(&format!(
                    "<hick:exec container=\"{container}\"{mount_attr}>\n{command}\n</hick:exec>\n",
                ));
            }
        }

        out.push_str("</hick:doc>\n");
        out
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_transcript_produces_valid_xml() {
        let builder = TranscriptBuilder::new();
        let xml = builder.to_hick_xml();
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<hick:doc xmlns:hick="));
        assert!(xml.contains("</hick:doc>"));
    }

    #[test]
    fn container_with_deny_network() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("alpine", "alpine");
        builder.add_capability("alpine", CapabilityRule::DenyNetwork);
        let xml = builder.to_hick_xml();
        assert!(xml.contains("<hick:container name=\"alpine\" image=\"alpine\">"));
        assert!(xml.contains("<hick:deny network=\"*\" />"));
        assert!(xml.contains("</hick:container>"));
    }

    #[test]
    fn container_with_allow_network() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("web", "alpine");
        builder.add_capability(
            "web",
            CapabilityRule::AllowNetwork {
                host: "github.com".into(),
                port: "443".into(),
            },
        );
        let xml = builder.to_hick_xml();
        assert!(xml.contains("<hick:allow network=\"github.com:443\" />"));
    }

    #[test]
    fn container_with_file_rules() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("writer", "alpine");
        builder.add_capability(
            "writer",
            CapabilityRule::AllowFileRead {
                path: "/data".into(),
            },
        );
        builder.add_capability(
            "writer",
            CapabilityRule::AllowFileWrite {
                path: "/output".into(),
            },
        );
        let xml = builder.to_hick_xml();
        assert!(xml.contains("<hick:allow file-read=\"/data\" />"));
        assert!(xml.contains("<hick:allow file-write=\"/output\" />"));
    }

    #[test]
    fn container_no_capabilities_self_closing() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("simple", "alpine");
        let xml = builder.to_hick_xml();
        assert!(xml.contains("<hick:container name=\"simple\" image=\"alpine\" />"));
    }

    #[test]
    fn volume_and_mount() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("worker", "alpine");
        builder.create_volume("data");
        builder.mount_volume("worker", "data", "/mnt/data");
        builder.record_command("worker", "ls /mnt/data");
        let xml = builder.to_hick_xml();
        assert!(xml.contains("<hick:volume name=\"data\" />"));
        assert!(xml.contains("mount=\"data:/mnt/data\""));
    }

    #[test]
    fn fork_declaration() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("base", "alpine");
        builder.record_fork("base", "forked");
        let xml = builder.to_hick_xml();
        assert!(xml.contains("<hick:fork from=\"base\" to=\"forked\" />"));
    }

    #[test]
    fn exec_tags_in_order() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("alpine", "alpine");
        builder.record_command("alpine", "echo hello");
        builder.record_command("alpine", "echo world");
        let xml = builder.to_hick_xml();
        let hello_pos = xml.find("echo hello").unwrap();
        let world_pos = xml.find("echo world").unwrap();
        assert!(hello_pos < world_pos);
    }

    #[test]
    fn rename_container() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("alpine", "alpine");
        assert!(builder.rename_container("alpine", "worker"));
        assert!(builder.has_container("worker"));
        assert!(!builder.has_container("alpine"));
    }

    #[test]
    fn full_session_roundtrip() {
        let mut builder = TranscriptBuilder::new();
        builder.declare_container("alpine", "alpine");
        builder.add_capability("alpine", CapabilityRule::DenyNetwork);
        builder.create_volume("shared");
        builder.mount_volume("alpine", "shared", "/data");
        builder.record_command("alpine", "echo hello > /data/greeting.txt");
        builder.record_command("alpine", "cat /data/greeting.txt");
        builder.record_fork("alpine", "checker");
        builder.record_command("checker", "wc -l /data/greeting.txt");

        let xml = builder.to_hick_xml();

        // Verify structural order: container before volume before fork before exec
        let container_pos = xml.find("<hick:container").unwrap();
        let volume_pos = xml.find("<hick:volume").unwrap();
        let fork_pos = xml.find("<hick:fork").unwrap();
        let first_exec_pos = xml.find("<hick:exec").unwrap();

        assert!(container_pos < volume_pos);
        assert!(volume_pos < fork_pos);
        assert!(fork_pos < first_exec_pos);
    }
}
