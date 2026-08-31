//! Git-backed VersionStore using git plumbing commands.
//!
//! Operates on a separate git object database at `.hick/git/` (not the
//! user's `.git/`). All commands run via `tokio::process::Command` with
//! `GIT_DIR` pointing to the hick-specific repository.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use log::{debug, info};
use tokio::process::Command;

use crate::error::StoreError;
use crate::types::{BlobHash, FileProvenance, Snapshot, SnapshotId};
use crate::version_store::VersionStore;

/// VersionStore backed by a git object database.
pub struct GitVersionStore {
    git_dir: PathBuf,
}

impl GitVersionStore {
    /// Create a new git-backed store at the given directory.
    ///
    /// Initializes a bare git repository if one doesn't exist.
    pub async fn new(git_dir: impl AsRef<Path>) -> Result<Self, StoreError> {
        let git_dir = git_dir.as_ref().to_path_buf();

        if !git_dir.exists() {
            info!("Initializing git store at {}", git_dir.display());
            let output = Command::new("git")
                .args(["init", "--bare"])
                .arg(&git_dir)
                .output()
                .await
                .map_err(|e| StoreError::GitCommand(format!("failed to run git init: {e}")))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(StoreError::GitCommand(format!("git init failed: {stderr}")));
            }
        }

        Ok(Self { git_dir })
    }

    fn git_cmd(&self) -> Command {
        let mut cmd = Command::new("git");
        cmd.env("GIT_DIR", &self.git_dir);
        // Snapshot commits need an identity even on hosts with no git
        // config (CI runners); this store's history is machine-authored.
        cmd.env("GIT_AUTHOR_NAME", "hick-store");
        cmd.env("GIT_AUTHOR_EMAIL", "store@hickorydocs.invalid");
        cmd.env("GIT_COMMITTER_NAME", "hick-store");
        cmd.env("GIT_COMMITTER_EMAIL", "store@hickorydocs.invalid");
        cmd
    }

    async fn run_git(&self, args: &[&str]) -> Result<String, StoreError> {
        let output = self
            .git_cmd()
            .args(args)
            .output()
            .await
            .map_err(|e| StoreError::GitCommand(format!("failed to run git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(StoreError::GitCommand(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn run_git_with_stdin(
        &self,
        args: &[&str],
        stdin_data: &[u8],
    ) -> Result<String, StoreError> {
        use tokio::io::AsyncWriteExt;

        let mut child = self
            .git_cmd()
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| StoreError::GitCommand(format!("failed to spawn git: {e}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(stdin_data)
                .await
                .map_err(|e| StoreError::GitCommand(format!("failed to write stdin: {e}")))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| StoreError::GitCommand(format!("failed to wait for git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(StoreError::GitCommand(format!(
                "git {} failed: {}",
                args.join(" "),
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Recursively build a git tree from a flat file map.
    ///
    /// Groups files by their top-level directory component, creates sub-trees
    /// for directories, and assembles the root tree via `mktree`.
    async fn build_tree(&self, files: &BTreeMap<String, BlobHash>) -> Result<String, StoreError> {
        use std::collections::BTreeMap as Map;

        // Separate files at this level vs files in subdirectories
        let mut blobs_here: Map<String, &BlobHash> = Map::new();
        let mut subdirs: Map<String, BTreeMap<String, BlobHash>> = Map::new();

        for (path, hash) in files {
            if let Some((dir, rest)) = path.split_once('/') {
                subdirs
                    .entry(dir.to_string())
                    .or_default()
                    .insert(rest.to_string(), hash.clone());
            } else {
                blobs_here.insert(path.clone(), hash);
            }
        }

        let mut entries = String::new();

        // Add blob entries
        for (name, hash) in &blobs_here {
            entries.push_str(&format!("100644 blob {}\t{name}\n", hash.0));
        }

        // Recursively create sub-trees
        for (dir_name, sub_files) in &subdirs {
            let sub_tree_sha = Box::pin(self.build_tree(sub_files)).await?;
            entries.push_str(&format!("040000 tree {sub_tree_sha}\t{dir_name}\n"));
        }

        self.run_git_with_stdin(&["mktree"], entries.as_bytes())
            .await
    }

    /// Store provenance as a JSON blob and return its hash.
    async fn store_provenance(
        &self,
        provenance: &BTreeMap<String, FileProvenance>,
    ) -> Result<BlobHash, StoreError> {
        let json = serde_json::to_string_pretty(provenance)
            .map_err(|e| StoreError::Serialization(e.to_string()))?;
        self.put_blob(json.as_bytes()).await
    }

    /// Read provenance from a tree's `.hick/provenance.json` blob.
    async fn read_provenance(
        &self,
        commit_sha: &str,
    ) -> Result<BTreeMap<String, FileProvenance>, StoreError> {
        // Try to read .hick/provenance.json from the tree
        let result = self
            .run_git(&["show", &format!("{commit_sha}:.hick/provenance.json")])
            .await;

        match result {
            Ok(json) => {
                serde_json::from_str(&json).map_err(|e| StoreError::Serialization(e.to_string()))
            }
            Err(_) => Ok(BTreeMap::new()), // No provenance in this commit
        }
    }
}

#[async_trait]
impl VersionStore for GitVersionStore {
    async fn put_blob(&self, content: &[u8]) -> Result<BlobHash, StoreError> {
        let hash = self
            .run_git_with_stdin(&["hash-object", "-w", "--stdin"], content)
            .await?;
        debug!("Stored git blob: {hash}");
        Ok(BlobHash(hash))
    }

    async fn get_blob(&self, hash: &BlobHash) -> Result<Vec<u8>, StoreError> {
        let output = self
            .git_cmd()
            .args(["cat-file", "blob", &hash.0])
            .output()
            .await
            .map_err(|e| StoreError::GitCommand(format!("failed to run git cat-file: {e}")))?;

        if !output.status.success() {
            return Err(StoreError::BlobNotFound(hash.clone()));
        }

        Ok(output.stdout)
    }

    async fn put_snapshot(&self, snapshot: &Snapshot) -> Result<SnapshotId, StoreError> {
        // Build git tree hierarchy from snapshot files.
        // mktree requires flat entries (no slashes), so we build sub-trees
        // for nested paths recursively.

        // Collect all files including provenance
        let prov_hash = self.store_provenance(&snapshot.provenance).await?;
        let mut all_files: BTreeMap<String, BlobHash> = snapshot.files.clone();
        all_files.insert(".hick/provenance.json".to_string(), prov_hash);

        let tree_sha = self.build_tree(&all_files).await?;

        // Build commit
        let mut commit_args = vec!["commit-tree".to_string(), tree_sha];
        for parent in &snapshot.parents {
            commit_args.push("-p".to_string());
            commit_args.push(parent.0.clone());
        }

        let message = snapshot
            .message
            .clone()
            .unwrap_or_else(|| format!("snapshot at {}", snapshot.timestamp));
        commit_args.push("-m".to_string());
        commit_args.push(message);

        let args_refs: Vec<&str> = commit_args.iter().map(|s| s.as_str()).collect();
        let commit_sha = self.run_git(&args_refs).await?;

        debug!("Created git commit: {commit_sha}");
        Ok(SnapshotId(commit_sha))
    }

    async fn get_snapshot(&self, id: &SnapshotId) -> Result<Snapshot, StoreError> {
        // Parse commit to get parent(s) and tree
        let commit_info = self
            .run_git(&["cat-file", "-p", &id.0])
            .await
            .map_err(|_| StoreError::SnapshotNotFound(id.clone()))?;

        let mut parents = Vec::new();
        let mut tree_sha = String::new();

        for line in commit_info.lines() {
            if let Some(sha) = line.strip_prefix("tree ") {
                tree_sha = sha.trim().to_string();
            } else if let Some(sha) = line.strip_prefix("parent ") {
                parents.push(SnapshotId(sha.trim().to_string()));
            }
        }

        // Parse tree to get files
        let tree_info = self.run_git(&["ls-tree", "-r", &tree_sha]).await?;

        let mut files = BTreeMap::new();
        for line in tree_info.lines() {
            // Format: mode type hash\tpath
            let parts: Vec<&str> = line.splitn(4, [' ', '\t']).collect();
            if parts.len() >= 4 {
                let blob_hash = parts[2].to_string();
                let path = parts[3].to_string();
                if !path.starts_with(".hick/") {
                    files.insert(path, BlobHash(blob_hash));
                }
            }
        }

        // Read provenance
        let provenance = self.read_provenance(&id.0).await.unwrap_or_default();

        // Get commit timestamp
        let timestamp_str = self
            .run_git(&["log", "-1", "--format=%ct", &id.0])
            .await
            .unwrap_or_else(|_| "0".to_string());
        let timestamp = timestamp_str.parse::<u64>().unwrap_or(0);

        // Get commit message
        let message = self
            .run_git(&["log", "-1", "--format=%B", &id.0])
            .await
            .ok()
            .filter(|s| !s.is_empty());

        Ok(Snapshot {
            id: id.clone(),
            parents,
            files,
            provenance,
            timestamp,
            message,
        })
    }

    async fn common_ancestor(
        &self,
        a: &SnapshotId,
        b: &SnapshotId,
    ) -> Result<Option<SnapshotId>, StoreError> {
        if a == b {
            return Ok(Some(a.clone()));
        }

        match self.run_git(&["merge-base", &a.0, &b.0]).await {
            Ok(sha) => Ok(Some(SnapshotId(sha))),
            Err(_) => Ok(None), // No common ancestor
        }
    }

    async fn get_branch(&self, name: &str) -> Result<Option<SnapshotId>, StoreError> {
        let refname = format!("refs/heads/{name}");
        match self.run_git(&["rev-parse", &refname]).await {
            Ok(sha) => Ok(Some(SnapshotId(sha))),
            Err(_) => Ok(None),
        }
    }

    async fn set_branch(&self, name: &str, id: &SnapshotId) -> Result<(), StoreError> {
        let refname = format!("refs/heads/{name}");
        self.run_git(&["update-ref", &refname, &id.0]).await?;
        Ok(())
    }

    async fn list_branches(&self) -> Result<Vec<String>, StoreError> {
        let output = self
            .run_git(&["for-each-ref", "--format=%(refname:short)", "refs/heads/"])
            .await;

        match output {
            Ok(text) => Ok(text
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect()),
            Err(_) => Ok(Vec::new()), // No branches yet
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A store in a directory of its own, removed when the test ends.
    ///
    /// The name used to be a nanosecond timestamp, which is unique only if
    /// the clock has nanosecond resolution — macOS reports microseconds here,
    /// so two of these tests running in parallel got the SAME directory and
    /// the second `git init` failed with "cannot mkdir: File exists". It
    /// reproduced 18 times in 25 runs, and it looked like a flake in whichever
    /// test happened to lose.
    ///
    /// `TempDir` is unique by construction and cleans up on drop, including
    /// when a test panics — which the manual `remove_dir_all` at the end of
    /// each test did not.
    async fn temp_git_store() -> (GitVersionStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        // `git init` wants to create the leaf itself on some versions, and a
        // path inside the TempDir keeps that true while the TempDir still
        // owns the cleanup.
        let root = dir.path().join("store");
        let store = GitVersionStore::new(&root).await.unwrap();
        (store, dir)
    }

    #[tokio::test]
    async fn git_blob_roundtrip() {
        let (store, _dir) = temp_git_store().await;
        let hash = store.put_blob(b"hello git").await.unwrap();
        let data = store.get_blob(&hash).await.unwrap();
        assert_eq!(data, b"hello git");
    }

    #[tokio::test]
    async fn git_snapshot_roundtrip() {
        let (store, _dir) = temp_git_store().await;

        // Store blob first
        let blob_hash = store.put_blob(b"file content").await.unwrap();

        let snapshot = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![],
            files: BTreeMap::from([("test.txt".to_string(), blob_hash)]),
            provenance: BTreeMap::from([(
                "test.txt".to_string(),
                FileProvenance::HickFile {
                    source: "main.hick".to_string(),
                },
            )]),
            timestamp: 1000,
            message: Some("initial".to_string()),
        };

        let id = store.put_snapshot(&snapshot).await.unwrap();
        let retrieved = store.get_snapshot(&id).await.unwrap();

        assert_eq!(retrieved.files.len(), 1);
        assert!(retrieved.files.contains_key("test.txt"));
        assert_eq!(retrieved.parents.len(), 0);

    }

    #[tokio::test]
    async fn git_branch_operations() {
        let (store, _dir) = temp_git_store().await;

        // Create a snapshot to point the branch at
        let blob_hash = store.put_blob(b"data").await.unwrap();
        let snapshot = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![],
            files: BTreeMap::from([("file.txt".to_string(), blob_hash)]),
            provenance: BTreeMap::new(),
            timestamp: 1,
            message: Some("first".to_string()),
        };
        let id = store.put_snapshot(&snapshot).await.unwrap();

        // No branch initially
        assert!(store.get_branch("main").await.unwrap().is_none());

        // Set and get
        store.set_branch("main", &id).await.unwrap();
        let got = store.get_branch("main").await.unwrap();
        assert_eq!(got, Some(id));

        // List
        let branches = store.list_branches().await.unwrap();
        assert!(branches.contains(&"main".to_string()));

    }

    #[tokio::test]
    async fn git_common_ancestor() {
        let (store, _dir) = temp_git_store().await;

        // Create root
        let h1 = store.put_blob(b"root").await.unwrap();
        let root = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![],
            files: BTreeMap::from([("root.txt".to_string(), h1)]),
            provenance: BTreeMap::new(),
            timestamp: 1,
            message: Some("root".to_string()),
        };
        let root_id = store.put_snapshot(&root).await.unwrap();

        // Create two branches from root
        let h2a = store.put_blob(b"branch-a").await.unwrap();
        let branch_a = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![root_id.clone()],
            files: BTreeMap::from([("a.txt".to_string(), h2a)]),
            provenance: BTreeMap::new(),
            timestamp: 2,
            message: Some("branch-a".to_string()),
        };
        let id_a = store.put_snapshot(&branch_a).await.unwrap();

        let h2b = store.put_blob(b"branch-b").await.unwrap();
        let branch_b = Snapshot {
            id: SnapshotId(String::new()),
            parents: vec![root_id.clone()],
            files: BTreeMap::from([("b.txt".to_string(), h2b)]),
            provenance: BTreeMap::new(),
            timestamp: 3,
            message: Some("branch-b".to_string()),
        };
        let id_b = store.put_snapshot(&branch_b).await.unwrap();

        // Common ancestor should be root
        let ancestor = store.common_ancestor(&id_a, &id_b).await.unwrap();
        assert_eq!(ancestor, Some(root_id));

    }
}
