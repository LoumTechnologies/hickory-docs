//! Sandboxed command execution abstraction.
//!
//! The [`CommandExecutor`] trait allows any agent to run shell commands in
//! isolated environments. [`WasmSandboxExecutor`] implements this trait using
//! a pluggable [`ContainerBackend`], allowing the underlying runtime to be
//! swapped without changing agent code.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use base64::Engine as _;

/// Structured output (stdout/stderr and exit code) from a sandboxed command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Combined stdout/stderr output from the command.
    pub output: String,
    /// Exit code if available (parsed from shell `echo $?` wrapper).
    pub exit_code: Option<i32>,
}

/// Trait for executing shell commands in a sandboxed environment.
///
/// Implementations may use WASM containers, VMs, or other isolation mechanisms.
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

/// Low-level container backend providing raw start and execute primitives.
///
/// Implement this trait to plug in a specific container runtime (WASM, Docker,
/// etc.) while reusing all of [`WasmSandboxExecutor`]'s workspace-init,
/// file-injection, and exit-code-parsing logic.
#[async_trait]
pub trait ContainerBackend: Send + Sync {
    /// Ensure the container is started and ready to accept commands.
    async fn ensure_started(&mut self, container: &str, image: &str) -> anyhow::Result<()>;

    /// Execute a raw command string in the container and return its combined output.
    async fn execute_raw(&mut self, container: &str, command: &str) -> anyhow::Result<String>;
}

/// Configuration for WASM sandbox execution.
pub struct WasmSandboxConfig {
    /// Host directory mounted at `/workspace` in containers.
    pub workdir: PathBuf,
    /// Default container image when not specified.
    pub default_image: String,
    /// Enables the guest virtio network bootstrap. Default: `true`.
    pub guest_network_enabled: bool,
}

impl Default for WasmSandboxConfig {
    fn default() -> Self {
        Self {
            workdir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            default_image: "alpine".to_string(),
            guest_network_enabled: true,
        }
    }
}

/// Executes commands in Hick WASM Linux containers with persistent state.
///
/// Delegates low-level container start/execute to a pluggable [`ContainerBackend`].
/// Handles workspace initialization, base64 file injection, and exit-code parsing.
pub struct WasmSandboxExecutor {
    config: WasmSandboxConfig,
    backend: Box<dyn ContainerBackend>,
    initialized_containers: HashSet<String>,
}

impl WasmSandboxExecutor {
    /// Create a new executor with the given configuration and backend.
    pub fn new(config: WasmSandboxConfig, backend: Box<dyn ContainerBackend>) -> Self {
        Self {
            config,
            backend,
            initialized_containers: HashSet::new(),
        }
    }

    async fn ensure_ready(&mut self, container: &str, image: &str) -> anyhow::Result<()> {
        self.backend.ensure_started(container, image).await?;

        if !self.initialized_containers.contains(container) {
            self.initialize_workspace().await?;
            self.initialized_containers.insert(container.to_string());
        }

        Ok(())
    }

    /// Creates `.hick/tmp` and `.hick/sdk` on the host project tree.
    async fn initialize_workspace(&mut self) -> anyhow::Result<()> {
        let hick_root = self.config.workdir.join(".hick");
        std::fs::create_dir_all(hick_root.join("tmp"))?;
        std::fs::create_dir_all(hick_root.join("sdk"))?;
        Ok(())
    }

    /// Write `content` to `dest_path` inside the container using base64 chunking.
    ///
    /// Sends the data in 512-byte base64 chunks to stay within BusyBox ash's
    /// line-editor buffer limit (~1024 chars).
    async fn write_file_to_container(
        &mut self,
        container: &str,
        content: &[u8],
        dest_path: &str,
    ) -> anyhow::Result<()> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(content);
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
        self.backend.execute_raw(container, &cmd).await?;
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

        let inner = if let Some(secs) = timeout {
            format!(
                "cd /workspace && timeout {secs} sh -c {}",
                Self::shell_escape(command)
            )
        } else {
            format!("cd /workspace && sh -c {}", Self::shell_escape(command))
        };

        let wrapped = format!("{inner}\necho \"===EXIT_CODE===$?===\"");
        let raw = self.backend.execute_raw(container, &wrapped).await?;

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
            let host_path = self.config.workdir.join(rest);
            if let Some(parent) = host_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(host_path, content)?;
        }

        if let Some(parent) = Path::new(dest_path).parent() {
            let mkdir_cmd = format!("mkdir -p {}", parent.display());
            self.backend.execute_raw(container, &mkdir_cmd).await?;
        }

        self.write_file_to_container(container, content, dest_path)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoBackend;

    #[async_trait]
    impl ContainerBackend for EchoBackend {
        async fn ensure_started(
            &mut self,
            _container: &str,
            _image: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn execute_raw(
            &mut self,
            _container: &str,
            command: &str,
        ) -> anyhow::Result<String> {
            if command.contains("EXIT_CODE") {
                Ok("hello\n===EXIT_CODE===0===".to_string())
            } else {
                Ok(String::new())
            }
        }
    }

    #[tokio::test]
    async fn executor_parses_exit_code() {
        let dir = tempfile::tempdir().unwrap();
        let config = WasmSandboxConfig {
            workdir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let mut exec = WasmSandboxExecutor::new(config, Box::new(EchoBackend));

        let out = exec
            .execute("ws", "echo hello", "alpine", None)
            .await
            .unwrap();
        assert_eq!(out.exit_code, Some(0));
        assert_eq!(out.output.trim(), "hello");
    }

    #[tokio::test]
    async fn executor_initializes_workspace_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let config = WasmSandboxConfig {
            workdir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let mut exec = WasmSandboxExecutor::new(config, Box::new(EchoBackend));

        exec.execute("ws", "echo hi", "alpine", None).await.unwrap();

        assert!(dir.path().join(".hick/tmp").exists());
        assert!(dir.path().join(".hick/sdk").exists());
    }

    #[test]
    fn shell_escape_wraps_in_quotes() {
        assert_eq!(WasmSandboxExecutor::shell_escape("echo hi"), "'echo hi'");
    }

    #[test]
    fn shell_escape_handles_single_quotes() {
        // Each ' in the input becomes '\'' (end-quote, escaped-quote, reopen-quote)
        // "echo 'hello'" → 'echo '\''hello'\'''
        let escaped = WasmSandboxExecutor::shell_escape("echo 'hello'");
        // format!("'{}'", ...) wraps the replaced string, so the final ' is the closing wrapper.
        assert_eq!(escaped, r#"'echo '\''hello'\'''"#);
    }
}
