//! One plain git repository per project on disk, under `GIT_DATA_DIR`.
//!
//! Why git CLI plumbing instead of the vendored `hick-store` `GitVersionStore`:
//! that store is a content-addressed snapshot/blob store (its own `.hick/git`
//! plumbing, snapshot ids, no working tree). The product requirement here is
//! an *inspectable* per-project repository — `git log` shows one commit per
//! save, operators can clone it, and run checkouts seed from a normal working
//! tree. The git CLI gives exactly that with no impedance mismatch.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use tokio::process::Command;
use uuid::Uuid;

#[derive(Clone)]
pub struct GitStore {
    root: PathBuf,
}

impl GitStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)
            .with_context(|| format!("creating GIT_DATA_DIR {}", root.display()))?;
        Ok(GitStore { root })
    }

    pub fn project_dir(&self, project_id: Uuid) -> PathBuf {
        self.root.join(project_id.to_string())
    }

    async fn git(&self, dir: &Path, args: &[&str]) -> Result<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", "Hickory Docs")
            .env("GIT_AUTHOR_EMAIL", "server@hickorydocs.invalid")
            .env("GIT_COMMITTER_NAME", "Hickory Docs")
            .env("GIT_COMMITTER_EMAIL", "server@hickorydocs.invalid")
            .output()
            .await
            .context("spawning git")?;
        if !out.status.success() {
            bail!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// Initialize the repository for a new project.
    pub async fn init_project(&self, project_id: Uuid) -> Result<()> {
        let dir = self.project_dir(project_id);
        std::fs::create_dir_all(&dir)?;
        self.git(&dir, &["init", "--initial-branch=master", "-q"])
            .await?;
        Ok(())
    }

    /// Validate a doc path: relative, no traversal, no `.git` component.
    pub fn validate_path(path: &str) -> Result<()> {
        let p = Path::new(path);
        if p.is_absolute() || path.is_empty() {
            bail!("doc path must be relative and non-empty");
        }
        for comp in p.components() {
            match comp {
                std::path::Component::Normal(c) => {
                    if c == ".git" {
                        bail!("doc path may not contain .git");
                    }
                }
                _ => bail!("doc path may not contain '.' or '..' components"),
            }
        }
        Ok(())
    }

    /// Persist one file and commit (a commit per save; no-op when unchanged).
    pub async fn save_file(
        &self,
        project_id: Uuid,
        rel_path: &str,
        content: &str,
        message: &str,
    ) -> Result<()> {
        Self::validate_path(rel_path)?;
        let dir = self.project_dir(project_id);
        if !dir.join(".git").exists() {
            self.init_project(project_id).await?;
        }
        let full = dir.join(rel_path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full, content)?;
        self.git(&dir, &["add", "--", rel_path]).await?;
        // Commit only when the index actually changed.
        let status = self
            .git(&dir, &["status", "--porcelain", "--", rel_path])
            .await?;
        if !status.trim().is_empty() {
            self.git(&dir, &["commit", "-q", "-m", message]).await?;
        }
        Ok(())
    }

    /// Number of commits in the project repo (test/observability helper).
    pub async fn commit_count(&self, project_id: Uuid) -> Result<usize> {
        let dir = self.project_dir(project_id);
        let out = self.git(&dir, &["rev-list", "--count", "HEAD"]).await?;
        Ok(out.trim().parse().unwrap_or(0))
    }

    /// Copy a finished run's *declared outputs* back into the project repo and
    /// commit whatever changed — this is how woven markdown and generated
    /// files become the committed baseline that `check` verifies against.
    ///
    /// Only the paths the pipeline declared as outputs are copied, never the
    /// whole run tree. Execs routinely leave heavy incidental artifacts in
    /// their working directory (a duckdb database, a build cache); those are
    /// not part of any drift comparison — `hickory check` only diffs the
    /// files the pipeline produces — but committing them made the project
    /// repo grow without bound, and *every* render, run and LSP session pays
    /// for that by copying the tree again in `seed_checkout`. Declared
    /// outputs (including binary ones) are still committed byte-for-byte, so
    /// the drift baseline is unchanged.
    pub async fn commit_outputs(
        &self,
        project_id: Uuid,
        src: &Path,
        outputs: &[String],
        message: &str,
    ) -> Result<()> {
        let dir = self.project_dir(project_id);
        if !dir.join(".git").exists() {
            self.init_project(project_id).await?;
        }
        for rel in outputs {
            if Self::validate_path(rel).is_err() {
                log::warn!("skipping output with unsafe path: {rel}");
                continue;
            }
            let from = src.join(rel);
            if !from.is_file() {
                continue;
            }
            let to = dir.join(rel);
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&from, &to)
                .with_context(|| format!("copying run output {rel} into the project repo"))?;
        }
        self.git(&dir, &["add", "-A"]).await?;
        let status = self.git(&dir, &["status", "--porcelain"]).await?;
        if !status.trim().is_empty() {
            self.git(&dir, &["commit", "-q", "-m", message]).await?;
        }
        Ok(())
    }

    /// Copy the project working tree (minus `.git`) into `dest` — the
    /// per-run temp dir seeding.
    ///
    /// Synchronous, and proportional to the size of the project tree. Callers
    /// on an async runtime must use [`GitStore::seed_checkout_async`]; calling
    /// this directly from a request handler blocks a runtime worker thread.
    pub fn seed_checkout(&self, project_id: Uuid, dest: &Path) -> Result<()> {
        let src = self.project_dir(project_id);
        if src.is_dir() {
            copy_tree(&src, dest)?;
        }
        Ok(())
    }

    /// [`GitStore::seed_checkout`] on the blocking pool, for request paths.
    pub async fn seed_checkout_async(&self, project_id: Uuid, dest: &Path) -> Result<()> {
        let this = self.clone();
        let dest = dest.to_path_buf();
        tokio::task::spawn_blocking(move || this.seed_checkout(project_id, &dest)).await?
    }
}

fn copy_tree(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let from = entry.path();
        let to = dest.join(&name);
        if entry.file_type()?.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
