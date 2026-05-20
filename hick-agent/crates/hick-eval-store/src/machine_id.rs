use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Stable per-machine identity.
///
/// The UUID is persisted in `~/.hick/machine-id` on first run. The NDJSON filename
/// combines hostname and UUID so files are both human-readable and globally unique.
pub struct MachineId {
    uuid: String,
    hostname: String,
}

impl MachineId {
    /// Load (or generate on first run) from `~/.hick/machine-id`.
    pub fn load() -> Result<Self> {
        let path = default_path()?;
        Self::load_from(&path)
    }

    /// Load from an explicit path — used in tests to avoid touching `~/.hick/`.
    pub fn load_from(path: &Path) -> Result<Self> {
        let uuid = if path.exists() {
            fs::read_to_string(path)?.trim().to_string()
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, &id)?;
            id
        };
        Ok(Self { uuid, hostname: read_hostname() })
    }

    /// The stable UUID string stored in `~/.hick/machine-id`.
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// NDJSON filename: `<hostname>_<uuid>.ndjson`.
    pub fn filename(&self) -> String {
        format!("{}_{}.ndjson", sanitize(&self.hostname), self.uuid)
    }
}

fn default_path() -> Result<PathBuf> {
    crate::path::home_dir()
        .map(|h| h.join(".hick").join("machine-id"))
        .ok_or_else(|| anyhow::anyhow!("could not determine home directory"))
}

fn read_hostname() -> String {
    if let Ok(s) = fs::read_to_string("/etc/hostname") {
        let t = s.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn machine_id_is_stable_across_loads() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("machine-id");

        let id1 = MachineId::load_from(&path).unwrap();
        let id2 = MachineId::load_from(&path).unwrap();

        assert_eq!(id1.uuid(), id2.uuid());
        assert!(!id1.uuid().is_empty());
    }

    #[test]
    fn machine_id_uuid_is_valid_uuid_format() {
        let dir = TempDir::new().unwrap();
        let id = MachineId::load_from(&dir.path().join("machine-id")).unwrap();
        // UUID v4 is 36 chars: 8-4-4-4-12 with dashes
        assert_eq!(id.uuid().len(), 36);
        assert_eq!(id.uuid().chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn filename_ends_with_ndjson_and_contains_uuid() {
        let dir = TempDir::new().unwrap();
        let id = MachineId::load_from(&dir.path().join("machine-id")).unwrap();
        let name = id.filename();
        assert!(name.ends_with(".ndjson"));
        assert!(name.contains(id.uuid()));
    }

    #[test]
    fn first_run_creates_machine_id_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("subdir").join("machine-id");
        assert!(!path.exists());
        MachineId::load_from(&path).unwrap();
        assert!(path.exists());
    }
}
