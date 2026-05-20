//! Sandboxed command execution abstraction.
//!
//! The [`CommandExecutor`] trait allows any agent to run shell commands in
//! isolated environments. The default implementation uses Hick WASM containers.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use hick_literate::auth::AuthStack;
use hick_literate::executor::ContainerExecutor;
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

    /// Write raw bytes to `dest_path` inside the named container.
    ///
    /// Must be called before [`execute`] attempts to reference the file.
    /// The default implementation is a no-op (for direct/unsandboxed executors).
    async fn inject_file(
        &mut self,
        container: &str,
        image: &str,
        content: &[u8],
        dest_path: &str,
    ) -> anyhow::Result<()> {
        let _ = (container, image, content, dest_path);
        Ok(())
    }
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
    /// Enables the guest virtio network bootstrap (`--net=socket`). Default: `true`.
    ///
    /// When `true`, directory preopens are registered first (FD 3+), then the socket is placed
    /// after them via `--net=socket=listenfd=N` — both networking and `/workspace` work together.
    pub guest_network_enabled: bool,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            images_dir: PathBuf::from("."),
            workdir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            default_image: "alpine".to_string(),
            capabilities: ContainerCapabilities::default(),
            auth: None,
            guest_network_enabled: true,
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
    /// Containers that have had `/workspace` initialized (dirs created + SDK injected).
    initialized_containers: HashSet<String>,
}

impl WasmSandboxExecutor {
    /// Create a new executor with the given configuration.
    pub fn new(config: WasmSandboxConfig) -> Self {
        Self {
            config,
            executor: None,
            initialized_containers: HashSet::new(),
        }
    }

    /// Ensure the underlying executor is initialized and the container is booted.
    ///
    /// On first use of a container, creates the `/workspace` directory structure
    /// and injects the Hickory SDK so scripts can use `from hickory import hick`.
    async fn ensure_ready(&mut self, container: &str, image: &str) -> anyhow::Result<()> {
        if self.executor.is_none() {
            let mut caps_map = HashMap::new();
            caps_map.insert(container.to_string(), self.config.capabilities.clone());
            let mut exec = ContainerExecutor::new(self.config.images_dir.clone(), caps_map)?;
            exec.set_preopened_dirs(vec![(
                self.config.workdir.clone(),
                "/workspace".to_string(),
            )]);
            exec.set_guest_network_enabled(self.config.guest_network_enabled);
            if let Some(ref auth) = self.config.auth {
                exec.set_auth(auth.clone());
            }
            self.executor = Some(exec);
        }

        let exec = self.executor.as_mut().unwrap();
        exec.ensure_started(container, image).await?;

        if !self.initialized_containers.contains(container) {
            self.initialize_workspace(container).await?;
            self.initialized_containers.insert(container.to_string());
        }

        Ok(())
    }

    /// Creates `.hick/tmp` and `.hick/sdk` on the host project tree (visible at `/workspace/.hick/*`
    /// once the host mount is wired). No guest `mkdir` under `/workspace` — that can materialize
    /// an empty overlay and hide virtio-9p mounts on some kernels.
    async fn initialize_workspace(&mut self, _container: &str) -> anyhow::Result<()> {
        let hick_root = self.config.workdir.join(".hick");
        std::fs::create_dir_all(hick_root.join("tmp"))?;
        std::fs::create_dir_all(hick_root.join("sdk"))?;
        Ok(())
    }

    /// Write `content` to `dest_path` inside `container` using base64 chunking.
    ///
    /// Sends the data in 512-byte base64 chunks to stay within BusyBox ash's
    /// line-editor buffer limit (~1024 chars).
    async fn write_file_to_container(
        &mut self,
        container: &str,
        content: &[u8],
        dest_path: &str,
    ) -> anyhow::Result<()> {
        let encoded = hick_literate::volume_state::encode_base64(content);
        let tmp_b64 = "/tmp/_hick_inject.b64";
        let chunk_size = 512;

        let mut lines = vec![format!("> {tmp_b64}")];
        for chunk in encoded.as_bytes().chunks(chunk_size) {
            // Base64 alphabet contains no single-quote characters, so this is safe.
            let chunk_str = std::str::from_utf8(chunk).unwrap();
            lines.push(format!("echo -n '{chunk_str}' >> {tmp_b64}"));
        }
        lines.push(format!("base64 -d {tmp_b64} > {dest_path}"));
        lines.push(format!("rm -f {tmp_b64}"));

        let cmd = lines.join("\n");
        self.executor
            .as_mut()
            .unwrap()
            .execute(container, &cmd)
            .await?;
        Ok(())
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

    async fn inject_file(
        &mut self,
        container: &str,
        image: &str,
        content: &[u8],
        dest_path: &str,
    ) -> anyhow::Result<()> {
        self.ensure_ready(container, image).await?;

        if let Some(rest) = dest_path.strip_prefix("/workspace/") {
            // The host workdir is mounted at /workspace inside the container, so writing
            // to the host is sufficient — no shell interaction needed. Redundant container
            // writes can corrupt the marker protocol if a previous command timed out.
            let host_path = self.config.workdir.join(rest);
            if let Some(parent) = host_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(host_path, content)?;
            return Ok(());
        }

        if let Some(parent) = Path::new(dest_path).parent() {
            let mkdir_cmd = format!("mkdir -p {}", parent.display());
            self.executor
                .as_mut()
                .unwrap()
                .execute(container, &mkdir_cmd)
                .await?;
        }

        self.write_file_to_container(container, content, dest_path).await
    }
}
