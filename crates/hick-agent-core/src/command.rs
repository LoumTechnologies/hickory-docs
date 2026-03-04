//! Sandboxed command execution abstraction.
//!
//! The [`CommandExecutor`] trait allows any agent to run shell commands in
//! isolated environments. The default implementation uses Hick WASM containers.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use hick::auth::AuthStack;
use hick::executor::ContainerExecutor;
use hick_token::ContainerCapabilities;

/// Structured output from a sandboxed command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Combined stdout/stderr output from the command.
    pub output: String,
    /// Exit code if available (parsed from shell `echo $?` wrapper).
    pub exit_code: Option<i32>,
}

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
    ) -> anyhow::Result<CommandOutput>;
}

/// Configuration for WASM sandbox execution.
pub struct WasmSandboxConfig {
    /// Directory containing `.wasm` container images.
    pub images_dir: PathBuf,
    /// Host directory to mount at `/workspace` in containers.
    pub workdir: PathBuf,
    /// Default container image when not specified.
    pub default_image: String,
    /// Capability restrictions for the sandbox container.
    pub capabilities: ContainerCapabilities,
    /// Optional auth injection stack for transparent credential management.
    pub auth: Option<Arc<AuthStack>>,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            images_dir: PathBuf::from("."),
            workdir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            default_image: "alpine".to_string(),
            capabilities: ContainerCapabilities::default(),
            auth: None,
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
            let mut caps_map = HashMap::new();
            caps_map.insert(container.to_string(), self.config.capabilities.clone());
            let mut exec = ContainerExecutor::new(self.config.images_dir.clone(), caps_map)?;
            exec.set_preopened_dirs(vec![(
                self.config.workdir.clone(),
                "/workspace".to_string(),
            )]);
            if let Some(ref auth) = self.config.auth {
                exec.set_auth(auth.clone());
            }
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
    ) -> anyhow::Result<CommandOutput> {
        self.ensure_ready(container, image).await?;

        let exec = self
            .executor
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("executor not initialized"))?;

        // Wrap the command to capture its exit code on a separate line.
        let inner = if let Some(secs) = timeout {
            format!(
                "cd /workspace && timeout {secs} sh -c {}",
                Self::shell_escape(command)
            )
        } else {
            format!("cd /workspace && sh -c {}", Self::shell_escape(command))
        };

        let wrapped = format!("{inner}\necho \"===EXIT_CODE===$?===\"");
        let raw = exec.execute(container, &wrapped).await?;

        // Parse exit code from the trailing marker.
        let (output, exit_code) = if let Some(idx) = raw.find("===EXIT_CODE===") {
            let before = raw[..idx].to_string();
            let after = &raw[idx + "===EXIT_CODE===".len()..];
            let code_str = after.trim_end_matches("===").trim();
            let code = code_str.parse::<i32>().ok();
            (before, code)
        } else {
            (raw, None)
        };

        Ok(CommandOutput { output, exit_code })
    }
}
