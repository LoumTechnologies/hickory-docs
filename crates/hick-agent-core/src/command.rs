//! Sandboxed command execution abstraction.
//!
//! The [`CommandExecutor`] trait allows any agent to run shell commands in
//! isolated environments. The default implementation uses Hick WASM containers.

use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use hick::executor::ContainerExecutor;

/// Executes shell commands in a sandboxed environment.
///
/// Implementations may use WASM containers, VMs, or other isolation mechanisms.
/// This trait enables agents to run commands without depending on a specific
/// implementation.
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    /// Execute a shell command in the named container.
    ///
    /// - `container`: Logical container/session name (e.g. "workspace")
    /// - `command`: Shell command to run (e.g. "cargo test")
    /// - `image`: Container image name (e.g. "alpine", "ubuntu")
    /// - `timeout`: Optional timeout in seconds
    async fn execute(
        &mut self,
        container: &str,
        command: &str,
        image: &str,
        timeout: Option<u64>,
    ) -> anyhow::Result<String>;
}

/// Configuration for WASM sandbox execution.
#[derive(Clone)]
pub struct WasmSandboxConfig {
    /// Directory containing `.wasm` container images.
    pub images_dir: PathBuf,
    /// Host directory to mount at `/workspace` in containers.
    pub workdir: PathBuf,
    /// Default container image when not specified.
    pub default_image: String,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            images_dir: PathBuf::from("."),
            workdir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            default_image: "alpine".to_string(),
        }
    }
}

/// Executes commands in Hick WASM Linux containers.
///
/// Uses the same container runtime as the hick pipeline. Containers persist
/// across calls, preserving installed packages and environment.
pub struct WasmSandboxExecutor {
    config: WasmSandboxConfig,
    executor: Option<ContainerExecutor>,
}

impl WasmSandboxExecutor {
    /// Create a new executor with the given configuration.
    pub fn new(config: WasmSandboxConfig) -> Self {
        Self {
            config,
            executor: None,
        }
    }

    /// Ensure the underlying executor is initialized and the container is booted.
    async fn ensure_ready(&mut self, container: &str, image: &str) -> anyhow::Result<()> {
        if self.executor.is_none() {
            let executor = ContainerExecutor::new(
                self.config.images_dir.clone(),
                HashMap::new(),
            )?;
            let mut exec = executor;
            exec.set_preopened_dirs(vec![(
                self.config.workdir.clone(),
                "/workspace".to_string(),
            )]);
            self.executor = Some(exec);
        }

        let exec = self.executor.as_mut().unwrap();
        exec.ensure_started(container, image).await
    }

    fn shell_escape(cmd: &str) -> String {
        format!("'{}'", cmd.replace('\'', "'\\''"))
    }
}

#[async_trait]
impl CommandExecutor for WasmSandboxExecutor {
    async fn execute(
        &mut self,
        container: &str,
        command: &str,
        image: &str,
        timeout: Option<u64>,
    ) -> anyhow::Result<String> {
        self.ensure_ready(container, image).await?;

        let exec = self
            .executor
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("executor not initialized"))?;

        let cmd = if let Some(secs) = timeout {
            format!(
                "cd /workspace && timeout {secs} sh -c {}",
                Self::shell_escape(command)
            )
        } else {
            format!("cd /workspace && {command}")
        };

        exec.execute(container, &cmd).await
    }
}
