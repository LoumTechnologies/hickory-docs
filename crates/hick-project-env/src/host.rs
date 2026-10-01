//! All automatic subprocesses are bounded and go through this host service.
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::process::Command;

pub struct Host {
    pub timeout: Duration,
    /// Explicit search path is also the deterministic test seam.
    pub path: std::ffi::OsString,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(8),
            path: crate::config::config().executable_path.clone(),
        }
    }
}

pub struct Output {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Host {
    pub fn executable(&self, name: &str) -> Option<PathBuf> {
        for dir in std::env::split_paths(&self.path).filter(|d| d.is_absolute()) {
            for suffix in if cfg!(windows) {
                &[".exe", ".cmd", ".bat", ""][..]
            } else {
                &[""][..]
            } {
                let candidate = dir.join(format!("{name}{suffix}"));
                if !candidate.is_file() {
                    continue;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if candidate.metadata().ok()?.permissions().mode() & 0o111 == 0 {
                        continue;
                    }
                }
                return Some(candidate);
            }
        }
        None
    }

    pub async fn probe(
        &self,
        executable: &Path,
        cwd: &Path,
        args: &[&str],
    ) -> Result<Output, String> {
        let cache = tempfile::tempdir().map_err(|e| e.to_string())?;
        let mut cmd = Command::new(executable);
        cmd.args(args)
            .current_dir(cwd)
            .env("PATH", &self.path)
            .env("UV_CACHE_DIR", cache.path())
            .env("UV_PYTHON_DOWNLOADS", "never")
            .env("UV_KEYRING_PROVIDER", "disabled")
            .env("UV_NO_PROGRESS", "1")
            .env("UV_OFFLINE", "1")
            .stdin(Stdio::null())
            .kill_on_drop(true);
        let output = tokio::time::timeout(self.timeout, cmd.output())
            .await
            .map_err(|_| {
                "Environment check timed out; no readiness result was established.".to_string()
            })?
            .map_err(|e| format!("Environment check could not run: {e}"))?;
        Ok(Output {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
