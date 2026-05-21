use crate::backend::StorageBackend;
use crate::machine_id::MachineId;
use crate::path::{git_toplevel, home_dir};
use crate::record::EvalRecord;
use anyhow::Result;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git-backed eval store.
///
/// On construction: `git pull --ff-only` (non-fatal when no remote or not in a repo).
/// After a session: call [`GitBackend::sync_session`] to `git add`, `commit`, and `push`.
/// All git operations are invoked via `std::process::Command`; no git library dependency.
pub struct GitBackend {
    store_dir: PathBuf,
    machine_file: PathBuf,
    machine_id: MachineId,
    /// Repository root, or `None` when not inside a git repo.
    git_root: Option<PathBuf>,
}

impl GitBackend {
    /// Create the backend and run `git pull` (non-fatal).
    pub fn new() -> Result<Self> {
        let machine_id = MachineId::load()?;
        let git_root = git_toplevel();
        let store_dir = match &git_root {
            Some(root) => root.join(".hick").join("eval"),
            None => home_dir()
                .map(|h| h.join(".hick").join("eval"))
                .unwrap_or_else(|| PathBuf::from(".hick/eval")),
        };
        fs::create_dir_all(&store_dir)?;
        let machine_file = store_dir.join(machine_id.filename());
        let backend = Self { store_dir, machine_file, machine_id, git_root };
        backend.git_pull();
        Ok(backend)
    }

    /// Stage, commit, and push this machine's file after a session ends.
    ///
    /// Non-fatal: all git errors are silently ignored so callers are never interrupted.
    pub fn sync_session(&self) {
        if let Some(ref root) = self.git_root {
            self.git_add_commit_push(root);
        }
    }

    fn git_pull(&self) {
        let Some(ref root) = self.git_root else { return };
        if !has_remote(root) {
            return;
        }
        let _ = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(root)
            .output();
    }

    fn git_add_commit_push(&self, root: &Path) {
        if !self.machine_file.exists() {
            return;
        }
        let rel = self.machine_file
            .strip_prefix(root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| self.machine_file.clone());

        let staged = Command::new("git")
            .args(["add", &rel.to_string_lossy()])
            .current_dir(root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !staged {
            return;
        }

        let committed = Command::new("git")
            .args([
                "commit",
                "-m",
                &format!("eval: update {}", self.machine_id.uuid()),
            ])
            .current_dir(root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if committed && has_remote(root) {
            let _ = Command::new("git")
                .args(["push"])
                .current_dir(root)
                .status();
        }
    }
}

impl StorageBackend for GitBackend {
    fn append(&self, record: &EvalRecord) -> Result<()> {
        let line = serde_json::to_string(record)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.machine_file)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    /// Read all records across every `.ndjson` file in the store directory.
    ///
    /// Unknown record types are returned as [`EvalRecord::Unknown`]; malformed lines
    /// are silently skipped so a corrupted file on one machine cannot block the reader.
    fn read_all(&self) -> Result<Vec<EvalRecord>> {
        let mut records = Vec::new();
        let entries = match fs::read_dir(&self.store_dir) {
            Ok(e) => e,
            Err(_) => return Ok(records),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("ndjson") {
                continue;
            }
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(r) = serde_json::from_str::<EvalRecord>(line) {
                    records.push(r);
                }
            }
        }
        Ok(records)
    }
}

fn has_remote(root: &Path) -> bool {
    Command::new("git")
        .args(["remote"])
        .current_dir(root)
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{EvalRecord, EvalResultRecord, PruneRecord};
    use chrono::Utc;
    use tempfile::TempDir;

    /// Build a GitBackend backed by a temp directory (no git operations).
    fn backend_in(dir: &TempDir) -> GitBackend {
        let machine_id_path = dir.path().join("machine-id");
        let machine_id = MachineId::load_from(&machine_id_path).unwrap();
        let store_dir = dir.path().join("eval");
        fs::create_dir_all(&store_dir).unwrap();
        let machine_file = store_dir.join(machine_id.filename());
        GitBackend { store_dir, machine_file, machine_id, git_root: None }
    }

    fn eval_record(machine_id: &str) -> EvalRecord {
        EvalRecord::EvalResult(EvalResultRecord {
            machine_id: machine_id.to_string(),
            timestamp: Utc::now(),
            config_name: "cfg".to_string(),
            query: Some("q".to_string()),
            response: Some("r".to_string()),
            scores: None,
            token_cost: None,
            original_turn_id: None,
            wall_secs: None,
            rework_count: None,
        })
    }

    #[test]
    fn append_and_read_back() {
        let dir = TempDir::new().unwrap();
        let b = backend_in(&dir);
        b.append(&eval_record("m1")).unwrap();
        let all = b.read_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].record_type(), "eval_result");
    }

    #[test]
    fn prune_appended_not_deleted() {
        let dir = TempDir::new().unwrap();
        let b = backend_in(&dir);
        b.append(&eval_record("m1")).unwrap();
        b.append(&EvalRecord::Prune(PruneRecord {
            machine_id: "m1".to_string(),
            timestamp: Utc::now(),
            config_name: Some("cfg".to_string()),
            reason: Some("old".to_string()),
            pruned_before: None,
            scores: None,
            confidence: None,
            trigger: Some("manual".to_string()),
        }))
        .unwrap();
        let all = b.read_all().unwrap();
        assert_eq!(all.len(), 2, "prune is appended, original line unchanged");
        assert_eq!(all[0].record_type(), "eval_result");
        assert_eq!(all[1].record_type(), "prune");
    }

    #[test]
    fn two_machine_files_aggregated() {
        let dir = TempDir::new().unwrap();
        let store_dir = dir.path().join("eval");
        fs::create_dir_all(&store_dir).unwrap();

        // Write two separate machine files manually.
        for (hostname, uuid_suffix) in [("host1", "01"), ("host2", "02")] {
            let filename = format!(
                "{hostname}_aaaaaaaa-0000-0000-0000-0000000000{uuid_suffix}.ndjson"
            );
            let r = eval_record(hostname);
            let line = serde_json::to_string(&r).unwrap();
            fs::write(store_dir.join(&filename), format!("{line}\n")).unwrap();
        }

        let machine_id = MachineId::load_from(&dir.path().join("mid")).unwrap();
        let machine_file = store_dir.join(machine_id.filename());
        let b = GitBackend { store_dir, machine_file, machine_id, git_root: None };

        let all = b.read_all().unwrap();
        assert_eq!(all.len(), 2, "aggregates records from both machine files");
    }

    #[test]
    fn unknown_record_types_are_not_dropped() {
        let dir = TempDir::new().unwrap();
        let store_dir = dir.path().join("eval");
        fs::create_dir_all(&store_dir).unwrap();
        let path = store_dir.join("host_aaaaaaaa-0000-0000-0000-000000000099.ndjson");
        fs::write(
            &path,
            r#"{"record_type":"future_type","data":1}
{"record_type":"eval_result","machine_id":"m","timestamp":"2024-01-01T00:00:00Z","config_name":"c"}
"#,
        )
        .unwrap();

        let machine_id = MachineId::load_from(&dir.path().join("mid")).unwrap();
        let machine_file = store_dir.join(machine_id.filename());
        let b = GitBackend { store_dir, machine_file, machine_id, git_root: None };

        let all = b.read_all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].record_type(), "future_type");
        assert_eq!(all[1].record_type(), "eval_result");
    }

    #[test]
    fn malformed_lines_are_skipped_gracefully() {
        let dir = TempDir::new().unwrap();
        let store_dir = dir.path().join("eval");
        fs::create_dir_all(&store_dir).unwrap();
        let path = store_dir.join("host_aaaaaaaa-0000-0000-0000-000000000001.ndjson");
        fs::write(
            &path,
            "not json at all\n{\"record_type\":\"eval_result\",\"machine_id\":\"m\",\"timestamp\":\"2024-01-01T00:00:00Z\",\"config_name\":\"c\"}\n",
        )
        .unwrap();

        let machine_id = MachineId::load_from(&dir.path().join("mid")).unwrap();
        let machine_file = store_dir.join(machine_id.filename());
        let b = GitBackend { store_dir, machine_file, machine_id, git_root: None };

        let all = b.read_all().unwrap();
        assert_eq!(all.len(), 1, "malformed line skipped, valid line retained");
    }

    #[test]
    fn empty_store_dir_returns_empty_vec() {
        let dir = TempDir::new().unwrap();
        let b = backend_in(&dir);
        let all = b.read_all().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn resolve_store_dir_falls_back_gracefully() {
        // Smoke-test that resolve_store_dir returns something non-empty.
        let dir = crate::path::resolve_store_dir();
        assert!(!dir.as_os_str().is_empty());
    }
}
