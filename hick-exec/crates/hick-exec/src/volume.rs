//! Volume system domain types.
//!
//! Defines volume declarations, access rules, and volume kinds used by
//! the DAG builder and pipeline executor to manage inter-container data flow.

/// The kind of volume: input, output, both, or ephemeral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeKind {
    /// Snapshot of an existing host directory (read-only source).
    Input { path: String },
    /// Contents become pipeline file outputs at the given path.
    Output { path: String },
    /// Reads existing state AND writes updates.
    InputOutput { input: String, output: String },
    /// Ephemeral inter-container scratch (no host mapping).
    Ephemeral,
}

/// Access level for a container on a volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VolumeAccess {
    /// Read-only access matching a glob pattern.
    Read(String),
    /// Write access matching a glob pattern (implies read).
    Write(String),
}

impl VolumeAccess {
    /// Check if this access rule permits reading the given path.
    pub fn permits_read(&self, path: &str) -> bool {
        match self {
            VolumeAccess::Read(pattern) | VolumeAccess::Write(pattern) => {
                glob_matches(path, pattern)
            }
        }
    }

    /// Check if this access rule permits writing the given path.
    pub fn permits_write(&self, path: &str) -> bool {
        match self {
            VolumeAccess::Write(pattern) => glob_matches(path, pattern),
            VolumeAccess::Read(_) => false,
        }
    }
}

/// An access rule binding a container to a volume with specific permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeAccessRule {
    /// The container this rule applies to.
    pub container: String,
    /// The access level granted.
    pub access: VolumeAccess,
}

/// A volume declaration parsed from `<hick:volume>`.
#[derive(Debug, Clone)]
pub struct VolumeDeclaration {
    /// Volume name (unique within a document).
    pub name: String,
    /// What kind of volume this is.
    pub kind: VolumeKind,
    /// Explicit access rules from `<hick:allow>` children.
    pub access_rules: Vec<VolumeAccessRule>,
}

impl VolumeDeclaration {
    /// Check if a container has read access to a path on this volume.
    ///
    /// If no access rules are defined, access is unrestricted (backward compat).
    pub fn check_read(&self, container: &str, path: &str) -> bool {
        if self.access_rules.is_empty() {
            return true;
        }
        self.access_rules
            .iter()
            .filter(|r| r.container == container)
            .any(|r| r.access.permits_read(path))
    }

    /// Check if a container has write access to a path on this volume.
    ///
    /// If no access rules are defined, access is unrestricted (backward compat).
    pub fn check_write(&self, container: &str, path: &str) -> bool {
        if self.access_rules.is_empty() {
            return true;
        }
        self.access_rules
            .iter()
            .filter(|r| r.container == container)
            .any(|r| r.access.permits_write(path))
    }

    /// Get the set of containers that have any write access on this volume.
    pub fn writers(&self) -> Vec<&str> {
        self.access_rules
            .iter()
            .filter(|r| matches!(r.access, VolumeAccess::Write(_)))
            .map(|r| r.container.as_str())
            .collect()
    }

    /// Get the set of containers that have read-only access on this volume.
    pub fn readers(&self) -> Vec<&str> {
        self.access_rules
            .iter()
            .filter(|r| matches!(r.access, VolumeAccess::Read(_)))
            .map(|r| r.container.as_str())
            .collect()
    }
}

/// Simple glob-style path matching.
///
/// Supports:
/// - `**` matches everything
/// - Trailing `*` matches any suffix
/// - `Dir/**` matches anything under Dir/
/// - Exact match otherwise
fn glob_matches(path: &str, pattern: &str) -> bool {
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_double_star() {
        assert!(glob_matches("foo/bar/baz.txt", "**"));
        assert!(glob_matches("anything", "**"));
    }

    #[test]
    fn glob_matches_prefix_double_star() {
        assert!(glob_matches(
            "Controllers/HomeController.cs",
            "Controllers/**"
        ));
        assert!(!glob_matches("Models/User.cs", "Controllers/**"));
    }

    #[test]
    fn glob_matches_trailing_star() {
        assert!(glob_matches("src/main.rs", "src/*"));
        assert!(!glob_matches("tests/main.rs", "src/*"));
    }

    #[test]
    fn glob_matches_exact() {
        assert!(glob_matches("README.md", "README.md"));
        assert!(!glob_matches("README.txt", "README.md"));
    }

    #[test]
    fn volume_access_read_permits() {
        let access = VolumeAccess::Read("src/**".to_string());
        assert!(access.permits_read("src/main.rs"));
        assert!(!access.permits_write("src/main.rs"));
    }

    #[test]
    fn volume_access_write_implies_read() {
        let access = VolumeAccess::Write("output/**".to_string());
        assert!(access.permits_read("output/file.txt"));
        assert!(access.permits_write("output/file.txt"));
    }

    #[test]
    fn volume_declaration_no_rules_allows_all() {
        let vol = VolumeDeclaration {
            name: "shared".to_string(),
            kind: VolumeKind::Ephemeral,
            access_rules: vec![],
        };
        assert!(vol.check_read("any-container", "any/path"));
        assert!(vol.check_write("any-container", "any/path"));
    }

    #[test]
    fn volume_declaration_with_rules() {
        let vol = VolumeDeclaration {
            name: "project".to_string(),
            kind: VolumeKind::Output {
                path: "src/MyApi/".to_string(),
            },
            access_rules: vec![
                VolumeAccessRule {
                    container: "scaffolder".to_string(),
                    access: VolumeAccess::Write("**".to_string()),
                },
                VolumeAccessRule {
                    container: "linter".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
                VolumeAccessRule {
                    container: "patcher".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
                VolumeAccessRule {
                    container: "patcher".to_string(),
                    access: VolumeAccess::Write("Controllers/**".to_string()),
                },
            ],
        };

        // scaffolder can write anything
        assert!(vol.check_write("scaffolder", "Models/User.cs"));
        assert!(vol.check_read("scaffolder", "Models/User.cs"));

        // linter can only read
        assert!(vol.check_read("linter", "Models/User.cs"));
        assert!(!vol.check_write("linter", "Models/User.cs"));

        // patcher can read anything, write only Controllers/
        assert!(vol.check_read("patcher", "Models/User.cs"));
        assert!(!vol.check_write("patcher", "Models/User.cs"));
        assert!(vol.check_write("patcher", "Controllers/HomeController.cs"));

        // unknown container has no access
        assert!(!vol.check_read("unknown", "anything"));
    }

    #[test]
    fn writers_and_readers() {
        let vol = VolumeDeclaration {
            name: "shared".to_string(),
            kind: VolumeKind::Ephemeral,
            access_rules: vec![
                VolumeAccessRule {
                    container: "writer1".to_string(),
                    access: VolumeAccess::Write("**".to_string()),
                },
                VolumeAccessRule {
                    container: "reader1".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
                VolumeAccessRule {
                    container: "both".to_string(),
                    access: VolumeAccess::Write("output/**".to_string()),
                },
                VolumeAccessRule {
                    container: "both".to_string(),
                    access: VolumeAccess::Read("**".to_string()),
                },
            ],
        };

        let writers = vol.writers();
        assert!(writers.contains(&"writer1"));
        assert!(writers.contains(&"both"));
        assert!(!writers.contains(&"reader1"));

        let readers = vol.readers();
        assert!(readers.contains(&"reader1"));
        assert!(readers.contains(&"both"));
        assert!(!readers.contains(&"writer1"));
    }
}
