//! Container executor for running commands inside WASM containers.
//!
//! Manages the lifecycle of WASM containers: image loading, boot detection,
//! command execution with output capture, and shutdown. Adapts the
//! container-testbed pattern for automated (non-interactive) use.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use log::{info, warn};
use tokio::sync::{RwLock, mpsc};
use wasmtime::{Engine, Module};

use bridge::NetworkBridge;
use connection_filter::{AllowAll, CapabilityFilter, ConnectionFilter};
use container::{Container, ContainerConfig, create_async_engine};
use hick_token::ContainerCapabilities;
use net_config::NetworkConfig;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// BusyBox ash prompt pattern.
const PROMPT: &[u8] = b"/ # ";

/// Marker used to delimit captured command output.
const START_MARKER: &str = "===HICK_START===";
const END_MARKER: &str = "===HICK_END===";

/// How long to wait for the kernel to boot and show a prompt.
const BOOT_TIMEOUT: Duration = Duration::from_secs(120);

/// How long to wait for a command to complete.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(300);

// ---------------------------------------------------------------------------
// Prompt / echo helpers (adapted from container-testbed)
// ---------------------------------------------------------------------------

/// Streaming pattern matcher for the BusyBox ash prompt `/ # `.
struct PromptMatcher {
    matched: usize,
    buffer: Vec<u8>,
}

impl PromptMatcher {
    fn new() -> Self {
        Self {
            matched: 0,
            buffer: Vec::new(),
        }
    }

    /// Process a chunk of output. Returns `(passthrough_bytes, prompt_detected)`.
    fn process(&mut self, src: &[u8]) -> (Vec<u8>, bool) {
        let mut out = Vec::with_capacity(src.len());
        let mut detected = false;

        for &byte in src {
            if byte == PROMPT[self.matched] {
                self.buffer.push(byte);
                self.matched += 1;
                if self.matched == PROMPT.len() {
                    self.buffer.clear();
                    self.matched = 0;
                    detected = true;
                }
            } else {
                if !self.buffer.is_empty() {
                    out.extend_from_slice(&self.buffer);
                    self.buffer.clear();
                }
                self.matched = 0;
                if byte == PROMPT[0] {
                    self.buffer.push(byte);
                    self.matched = 1;
                } else {
                    out.push(byte);
                }
            }
        }

        (out, detected)
    }
}

/// Remove Device Status Report queries (ESC[6n) from container output.
fn strip_dsr(src: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        if i + 3 < src.len()
            && src[i] == 0x1b
            && src[i + 1] == b'['
            && src[i + 2] == b'6'
            && src[i + 3] == b'n'
        {
            i += 4;
        } else {
            out.push(src[i]);
            i += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Running container state
// ---------------------------------------------------------------------------

struct RunningContainer {
    container: Container,
    prompt_matcher: PromptMatcher,
    booted: bool,
    #[allow(dead_code)]
    network_config: NetworkConfig,
    /// Image name used to start this container (needed for forking).
    image_name: String,
    /// Commands executed in this container, in order (for replay-based forking).
    command_history: Vec<String>,
}

// ---------------------------------------------------------------------------
// Fork definition
// ---------------------------------------------------------------------------

/// Describes a pending fork: which source container to replay from and any
/// additional capability restrictions applied to the fork target.
struct ForkDef {
    from: String,
    /// Capability restrictions from fork's `<deny>`/`<allow>` children.
    /// The pipeline applies these to `container_caps` before executor creation,
    /// so the connection filter already uses the attenuated caps. Stored here
    /// for future use (e.g. runtime re-attenuation).
    #[allow(dead_code)]
    additional_caps: Option<ContainerCapabilities>,
}

// ---------------------------------------------------------------------------
// ContainerExecutor
// ---------------------------------------------------------------------------

/// A single exec's transcript entry: command lines and captured output.
#[derive(Debug, Clone)]
pub struct ExecTranscriptEntry {
    /// Individual command lines sent to the container.
    pub commands: Vec<String>,
    /// Captured output from the command execution.
    pub output: String,
}

/// Manages WASM container lifecycles for pipeline execution.
pub struct ContainerExecutor {
    engine: Engine,
    containers: HashMap<String, RunningContainer>,
    images_dir: PathBuf,
    bridge: Arc<RwLock<NetworkBridge>>,
    next_subnet: u8,
    container_caps: HashMap<String, ContainerCapabilities>,
    transcripts: HashMap<String, Vec<ExecTranscriptEntry>>,
    /// Pending fork definitions: target container name -> fork info.
    fork_defs: HashMap<String, ForkDef>,
}

impl ContainerExecutor {
    /// Create a new executor.
    ///
    /// `images_dir` is the directory containing pre-converted `.wasm` images.
    /// `container_caps` maps container names to their parsed capabilities.
    pub fn new(
        images_dir: PathBuf,
        container_caps: HashMap<String, ContainerCapabilities>,
    ) -> Result<Self> {
        let engine = create_async_engine()?;
        let bridge = Arc::new(RwLock::new(NetworkBridge::new()));
        Ok(Self {
            engine,
            containers: HashMap::new(),
            images_dir,
            bridge,
            next_subnet: 0,
            container_caps,
            transcripts: HashMap::new(),
            fork_defs: HashMap::new(),
        })
    }

    /// Register a fork definition. When `ensure_started` is called for `target`,
    /// the executor will start a fresh container from the source's image and
    /// replay the source's command history.
    pub fn register_fork(
        &mut self,
        target: &str,
        from: &str,
        additional_caps: Option<ContainerCapabilities>,
    ) {
        self.fork_defs.insert(
            target.to_string(),
            ForkDef {
                from: from.to_string(),
                additional_caps,
            },
        );
    }

    /// Ensure a container is started and booted. No-op if already running.
    ///
    /// If the container name is registered as a fork target, it will be started
    /// by replaying the source container's command history instead of starting
    /// fresh.
    pub async fn ensure_started(&mut self, name: &str, image: &str) -> Result<()> {
        if self.containers.contains_key(name) {
            return Ok(());
        }

        // Check if this is a fork target
        if let Some(fork_def) = self.fork_defs.remove(name) {
            return self.start_forked_container(name, &fork_def).await;
        }

        self.start_fresh_container(name, image).await
    }

    /// Start a brand-new container from a WASM image.
    async fn start_fresh_container(&mut self, name: &str, image: &str) -> Result<()> {
        let wasm_path = resolve_image_path(&self.images_dir, image);
        info!(
            "Loading image for container '{name}': {}",
            wasm_path.display()
        );

        let module = Module::from_file(&self.engine, &wasm_path)
            .with_context(|| format!("failed to load WASM image: {}", wasm_path.display()))?;

        let net_cfg = NetworkConfig::for_container(self.next_subnet);
        self.next_subnet = self.next_subnet.wrapping_add(1);

        // Create bridge channel
        let (bridge_tx, bridge_rx) = mpsc::channel::<Vec<u8>>(256);
        {
            let mut b = self.bridge.write().await;
            b.register(net_cfg.vm_ip_std(), bridge_tx, Some(name));
        }

        // Build connection filter from capabilities
        let filter: Arc<dyn ConnectionFilter> = if let Some(caps) = self.container_caps.get(name) {
            Arc::new(CapabilityFilter::new(caps.clone()))
        } else {
            Arc::new(AllowAll)
        };

        let mut config = ContainerConfig::new(self.engine.clone(), module);
        config.networking = true;
        config.connection_filter = Some(filter);
        config.network_config = Some(net_cfg.clone());
        config.bridge = Some(self.bridge.clone());
        config.bridge_rx = Some(bridge_rx);

        let container = Container::start(config)
            .await
            .with_context(|| format!("failed to start container '{name}'"))?;

        info!("Started container '{name}' (ip={})", net_cfg.vm_ip);

        self.containers.insert(
            name.to_string(),
            RunningContainer {
                container,
                prompt_matcher: PromptMatcher::new(),
                booted: false,
                network_config: net_cfg,
                image_name: image.to_string(),
                command_history: Vec::new(),
            },
        );

        // Wait for boot prompt
        self.wait_for_boot(name)
            .await
            .with_context(|| format!("container '{name}' failed to boot"))?;

        Ok(())
    }

    /// Start a forked container by replaying the source container's command history.
    ///
    /// 1. Looks up the source container's image and command history
    /// 2. Starts a fresh container from the same image
    /// 3. Replays each command from the source's history
    async fn start_forked_container(&mut self, name: &str, fork_def: &ForkDef) -> Result<()> {
        // Snapshot source state before starting the new container
        let (source_image, history) = {
            let source = self.containers.get(&fork_def.from).ok_or_else(|| {
                anyhow::anyhow!(
                    "fork source container '{}' not found (not started yet?)",
                    fork_def.from
                )
            })?;
            (source.image_name.clone(), source.command_history.clone())
        };

        info!(
            "Forking container '{}' -> '{name}': replaying {} commands from image '{source_image}'",
            fork_def.from,
            history.len()
        );

        // Start a fresh container from the same image
        self.start_fresh_container(name, &source_image).await?;

        // Replay command history (use replay_command to avoid recording to history)
        for cmd in &history {
            self.replay_command(name, cmd).await.with_context(|| {
                format!("failed to replay command in forked container '{name}': {cmd}")
            })?;
        }

        info!("Fork '{name}' ready: replayed {} commands", history.len());
        Ok(())
    }

    /// Execute a command in a container without recording it to the command history.
    /// Used for replaying commands during fork.
    async fn replay_command(&mut self, name: &str, command: &str) -> Result<String> {
        // This is the same as execute() but skips the history push
        let command = command.trim();
        if command.is_empty() {
            return Ok(String::new());
        }

        {
            let rc = self
                .containers
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("container '{name}' not found"))?;
            if !rc.booted {
                bail!("container '{name}' has not booted yet");
            }
        }

        info!("Replaying in '{name}': {command}");

        let cmd_lines: Vec<String> = command
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        {
            let rc = self.containers.get_mut(name).unwrap();
            rc.container
                .write_line(&format!("echo '{START_MARKER}'"))
                .await?;
            for line in &cmd_lines {
                rc.container.write_line(line).await?;
            }
            rc.container
                .write_line(&format!("echo '{END_MARKER}'"))
                .await?;
        }

        let deadline = tokio::time::Instant::now() + COMMAND_TIMEOUT;
        let mut buf = [0u8; 4096];
        let mut all_output = Vec::new();

        let (result, prompt_already_seen) = loop {
            let rc = self.containers.get_mut(name).unwrap();
            let read_result =
                tokio::time::timeout_at(deadline, rc.container.read_stdout(&mut buf)).await;

            match read_result {
                Err(_) => bail!("timed out waiting for command output"),
                Ok(Err(e)) => bail!("read error: {e}"),
                Ok(Ok(0)) => bail!("container exited unexpectedly"),
                Ok(Ok(n)) => {
                    let cleaned = strip_dsr(&buf[..n]);
                    all_output.extend_from_slice(&cleaned);

                    let text = String::from_utf8_lossy(&all_output);
                    let start_line = format!("\n{START_MARKER}");
                    let end_line = format!("\n{END_MARKER}");
                    if text.contains(&end_line) {
                        let start_pos = text
                            .find(&start_line)
                            .ok_or_else(|| anyhow::anyhow!("start marker not found on own line"))?;
                        let after_start = start_pos + start_line.len();
                        let end_pos = text.find(&end_line).unwrap();
                        let captured = &text[after_start..end_pos];

                        let result = clean_captured_output(captured, &cmd_lines);

                        let after_end = end_pos + end_line.len();
                        let leftover = text[after_end..].as_bytes();
                        let mut pm = PromptMatcher::new();
                        let (_, prompt_seen) = pm.process(leftover);

                        break (result, prompt_seen);
                    }
                }
            }
        };

        if !prompt_already_seen {
            self.wait_for_prompt(name, deadline).await?;
        }

        Ok(result)
    }

    /// Wait for the container's shell prompt after boot.
    async fn wait_for_boot(&mut self, name: &str) -> Result<()> {
        let rc = self
            .containers
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("container '{name}' not found"))?;

        let deadline = tokio::time::Instant::now() + BOOT_TIMEOUT;
        let mut buf = [0u8; 4096];

        loop {
            let read_result =
                tokio::time::timeout_at(deadline, rc.container.read_stdout(&mut buf)).await;

            match read_result {
                Err(_) => bail!("timed out waiting for boot prompt"),
                Ok(Err(e)) => bail!("read error during boot: {e}"),
                Ok(Ok(0)) => bail!("container exited during boot"),
                Ok(Ok(n)) => {
                    let cleaned = strip_dsr(&buf[..n]);
                    let (_passthrough, detected) = rc.prompt_matcher.process(&cleaned);
                    if detected {
                        rc.booted = true;
                        info!("Container '{name}' booted successfully");
                        return Ok(());
                    }
                }
            }
        }
    }

    /// Execute a command in a container and capture its output.
    ///
    /// Sends start marker, command lines, and end marker to the container,
    /// then reads all output until the end marker appears. The content
    /// between markers is extracted and cleaned of shell echoes, prompts,
    /// and ANSI escape sequences.
    ///
    /// The command is recorded in the container's history for replay-based
    /// forking.
    pub async fn execute(&mut self, name: &str, command: &str) -> Result<String> {
        let command = command.trim();
        if command.is_empty() {
            return Ok(String::new());
        }

        // Check container exists and is booted
        {
            let rc = self
                .containers
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("container '{name}' not found"))?;
            if !rc.booted {
                bail!("container '{name}' has not booted yet");
            }
        }

        info!("Executing in '{name}': {command}");

        // Record to history for potential future forks
        {
            let rc = self.containers.get_mut(name).unwrap();
            rc.command_history.push(command.to_string());
        }

        // Collect the command lines we're sending (for echo stripping later)
        let cmd_lines: Vec<String> = command
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        // Send start marker, all command lines, and end marker without
        // waiting for prompts in between. This ensures the command output
        // stays in the stdout stream for us to capture.
        {
            let rc = self.containers.get_mut(name).unwrap();
            rc.container
                .write_line(&format!("echo '{START_MARKER}'"))
                .await?;
            for line in &cmd_lines {
                rc.container.write_line(line).await?;
            }
            rc.container
                .write_line(&format!("echo '{END_MARKER}'"))
                .await?;
        }

        // Read all output until we see both markers.
        // Re-borrow `rc` each iteration so we don't hold a long-lived
        // mutable borrow on `self.containers`.
        let deadline = tokio::time::Instant::now() + COMMAND_TIMEOUT;
        let mut buf = [0u8; 4096];
        let mut all_output = Vec::new();

        let (result, prompt_already_seen) = loop {
            let rc = self.containers.get_mut(name).unwrap();
            let read_result =
                tokio::time::timeout_at(deadline, rc.container.read_stdout(&mut buf)).await;

            match read_result {
                Err(_) => bail!("timed out waiting for command output"),
                Ok(Err(e)) => bail!("read error: {e}"),
                Ok(Ok(0)) => bail!("container exited unexpectedly"),
                Ok(Ok(n)) => {
                    let cleaned = strip_dsr(&buf[..n]);
                    all_output.extend_from_slice(&cleaned);

                    let text = String::from_utf8_lossy(&all_output);
                    // Search for the marker on its own line (preceded by \n),
                    // not inside the echoed command (e.g. `/ # echo '===HICK_START==='`).
                    let start_line = format!("\n{START_MARKER}");
                    let end_line = format!("\n{END_MARKER}");
                    if text.contains(&end_line) {
                        let start_pos = text
                            .find(&start_line)
                            .ok_or_else(|| anyhow::anyhow!("start marker not found on own line"))?;
                        let after_start = start_pos + start_line.len();
                        let end_pos = text.find(&end_line).unwrap();
                        let captured = &text[after_start..end_pos];

                        let result = clean_captured_output(captured, &cmd_lines);
                        info!("Captured output from '{name}': {} bytes", result.len());

                        // Check if the prompt was already in the data we
                        // read past the end marker. Feed the leftover bytes
                        // through the prompt matcher so we don't wait for a
                        // prompt that's already been consumed.
                        let after_end = end_pos + end_line.len();
                        let leftover = text[after_end..].as_bytes();
                        let mut pm = PromptMatcher::new();
                        let (_, prompt_seen) = pm.process(leftover);

                        break (result, prompt_seen);
                    }
                }
            }
        };

        // Wait for the final prompt so the container is ready for the
        // next command — unless we already saw it in the buffered data.
        if !prompt_already_seen {
            self.wait_for_prompt(name, deadline).await?;
        }

        // Append structured transcript entry
        let entry = ExecTranscriptEntry {
            commands: cmd_lines,
            output: result.clone(),
        };
        self.transcripts
            .entry(name.to_string())
            .or_default()
            .push(entry);

        Ok(result)
    }

    /// Wait for a prompt, discarding output.
    async fn wait_for_prompt(&mut self, name: &str, deadline: tokio::time::Instant) -> Result<()> {
        let mut buf = [0u8; 4096];
        loop {
            let rc = self.containers.get_mut(name).unwrap();
            let read_result =
                tokio::time::timeout_at(deadline, rc.container.read_stdout(&mut buf)).await;

            match read_result {
                Err(_) => bail!("timed out waiting for prompt"),
                Ok(Err(e)) => bail!("read error: {e}"),
                Ok(Ok(0)) => bail!("container exited"),
                Ok(Ok(n)) => {
                    let cleaned = strip_dsr(&buf[..n]);
                    let (_passthrough, detected) = rc.prompt_matcher.process(&cleaned);
                    if detected {
                        return Ok(());
                    }
                }
            }
        }
    }

    /// Create a mount point directory inside a running container.
    pub async fn create_mount_point(&mut self, name: &str, mount_path: &str) -> Result<()> {
        let rc = self
            .containers
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("container '{name}' not found"))?;
        if !rc.booted {
            bail!("container '{name}' has not booted yet");
        }

        info!("Creating mount point in '{name}': {mount_path}");
        // Use execute to run mkdir, but don't record to history
        self.replay_command(name, &format!("mkdir -p {mount_path}"))
            .await
            .with_context(|| {
                format!("failed to create mount point '{mount_path}' in container '{name}'")
            })?;
        Ok(())
    }

    /// Inject volume data into a container by sending a base64-encoded tar
    /// and unpacking it at the mount path.
    pub async fn inject_volume(
        &mut self,
        name: &str,
        mount_path: &str,
        tar_data: &[u8],
    ) -> Result<()> {
        let rc = self
            .containers
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("container '{name}' not found"))?;
        if !rc.booted {
            bail!("container '{name}' has not booted yet");
        }

        let encoded = crate::volume_state::encode_base64(tar_data);
        info!(
            "Injecting volume into '{name}' at {mount_path}: {} bytes (base64: {} chars)",
            tar_data.len(),
            encoded.len()
        );

        // Create mount point, then decode and extract
        self.create_mount_point(name, mount_path).await?;

        // Use heredoc approach: echo the base64 data and pipe through base64 -d | tar xf -
        // Split into chunks to avoid shell line length limits
        let chunk_size = 4096;
        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(chunk_size)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect();

        let mut cmd = format!("cd {mount_path} && (");
        for chunk in &chunks {
            cmd.push_str(&format!("echo -n '{chunk}'; "));
        }
        cmd.push_str(") | base64 -d | tar xf -");

        self.replay_command(name, &cmd).await.with_context(|| {
            format!("failed to inject volume into container '{name}' at {mount_path}")
        })?;

        Ok(())
    }

    /// Extract volume data from a container by tarring the mount path
    /// and capturing the base64-encoded output.
    ///
    /// Uses inner markers (`===B64S===` / `===B64E===`) around the base64
    /// payload so that stray kernel or daemon messages don't corrupt the
    /// data.  The `-w 0` flag disables line-wrapping in the container's
    /// `base64` command.
    pub async fn extract_volume(
        &mut self,
        name: &str,
        mount_path: &str,
    ) -> Result<Vec<u8>> {
        let rc = self
            .containers
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("container '{name}' not found"))?;
        if !rc.booted {
            bail!("container '{name}' has not booted yet");
        }

        info!("Extracting volume from '{name}' at {mount_path}");

        let cmd = format!(
            "echo '===B64S==='; cd {mount_path} && tar cf - . | base64 -w 0; echo; echo '===B64E==='"
        );
        let output = self.replay_command(name, &cmd).await.with_context(|| {
            format!("failed to extract volume from container '{name}' at {mount_path}")
        })?;

        // Extract the base64 payload between the inner markers.
        let b64_data = extract_between_markers(&output, "===B64S===", "===B64E===")
            .with_context(|| {
                warn!(
                    "Base64 marker extraction failed. Cleaned output (first 500 chars): {:?}",
                    &output[..output.len().min(500)]
                );
                format!("failed to locate base64 markers in volume extraction output from '{name}'")
            })?;

        crate::volume_state::decode_base64(b64_data).with_context(|| {
            warn!(
                "Base64 decode failed. Extracted payload (first 200 chars): {:?}",
                &b64_data[..b64_data.len().min(200)]
            );
            "failed to decode base64".to_string()
        })
    }

    /// Per-container execution transcripts accumulated by [`execute`].
    ///
    /// Each entry maps a container name to a list of [`ExecTranscriptEntry`]
    /// values, each containing the command lines and output for one exec.
    pub fn transcripts(&self) -> &HashMap<String, Vec<ExecTranscriptEntry>> {
        &self.transcripts
    }

    /// Inject a transcript entry from cache (no actual execution).
    pub fn inject_transcript_entry(&mut self, container: &str, entry: ExecTranscriptEntry) {
        self.transcripts
            .entry(container.to_string())
            .or_default()
            .push(entry);
    }

    /// Kill all containers and clean up.
    pub async fn shutdown(&mut self) {
        let names: Vec<String> = self.containers.keys().cloned().collect();
        for name in &names {
            if let Some(rc) = self.containers.remove(name) {
                let vm_ip = rc.network_config.vm_ip_std();
                rc.container.kill();
                let mut b = self.bridge.write().await;
                b.unregister(vm_ip);
                info!("Killed container '{name}'");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Image resolution
// ---------------------------------------------------------------------------

/// Resolve an image name to a `.wasm` file path.
///
/// Search order:
/// 1. `{images_dir}/{name}.wasm` — exact file
/// 2. `{images_dir}/{name}/` — directory, pick latest `.wasm` by name
/// 3. Return as-is (wasmtime will produce a clear error)
pub fn resolve_image_path(images_dir: &Path, name: &str) -> PathBuf {
    // Strip tag if present (e.g. "alpine:3.19" → "alpine")
    let base_name = name.split(':').next().unwrap_or(name);

    // 1. Exact file
    let exact = images_dir.join(format!("{base_name}.wasm"));
    if exact.is_file() {
        return exact;
    }

    // 2. Directory with .wasm files
    let dir = images_dir.join(base_name);
    if dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&dir)
    {
        let mut wasm_files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "wasm"))
            .collect();
        wasm_files.sort();
        if let Some(latest) = wasm_files.last() {
            return latest.clone();
        }
    }

    // 3. Fallback
    warn!(
        "WASM image not found for '{name}', tried: {}",
        exact.display()
    );
    PathBuf::from(name)
}

// ---------------------------------------------------------------------------
// Output cleaning
// ---------------------------------------------------------------------------

/// Extract the text between two marker lines.
///
/// Markers must appear as the complete content of a line (after trimming)
/// to avoid matching markers embedded in shell command echoes.  Returns
/// the text between the two marker lines, trimmed of surrounding
/// whitespace.
fn extract_between_markers<'a>(
    text: &'a str,
    start_marker: &str,
    end_marker: &str,
) -> Option<&'a str> {
    let lines: Vec<&str> = text.lines().collect();
    let start_idx = lines.iter().position(|l| l.trim() == start_marker)?;
    let end_idx = lines[start_idx + 1..]
        .iter()
        .position(|l| l.trim() == end_marker)?
        + start_idx
        + 1;

    // Join the lines between the markers and trim.
    let start_byte = lines[..=start_idx]
        .iter()
        .map(|l| l.len() + 1) // +1 for the newline
        .sum::<usize>();
    let end_byte = start_byte
        + lines[start_idx + 1..end_idx]
            .iter()
            .map(|l| l.len() + 1)
            .sum::<usize>();
    // Avoid underflow if end_byte == start_byte (empty payload).
    let payload = if end_byte > start_byte {
        &text[start_byte..end_byte.min(text.len())]
    } else {
        ""
    };
    Some(payload.trim())
}

/// Strip ANSI escape sequences and carriage returns from a string.
fn strip_ansi(raw: &str) -> String {
    let mut result = String::new();
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Skip ANSI escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                // Read until alphabetic terminator or '~'
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() || next == '~' {
                        break;
                    }
                }
            }
        } else if ch == '\r' {
            // Skip carriage returns
        } else {
            result.push(ch);
        }
    }

    result
}

/// Clean the raw content captured between output markers.
///
/// The content between `===HICK_START===` and `===HICK_END===` includes:
/// - Shell prompt + command echo lines (e.g. `/ # cat /out/stamp.txt`)
/// - Actual command output
/// - Shell prompt + end marker echo (e.g. `/ # echo '===HICK_END==='`)
///
/// This function strips those shell artifacts, leaving only the real output.
fn clean_captured_output(raw: &str, sent_commands: &[String]) -> String {
    let cleaned = strip_ansi(raw);

    let mut output_lines = Vec::new();

    for line in cleaned.lines() {
        let trimmed = line.trim();

        // Skip empty lines at boundaries
        if trimmed.is_empty() {
            output_lines.push(String::new());
            continue;
        }

        // Skip lines that are a prompt followed by one of our sent commands
        // The prompt is typically "/ # " but may have other prefixes
        if is_prompt_echo(trimmed, sent_commands) {
            continue;
        }

        // Skip prompt + end marker echo
        if trimmed.contains(END_MARKER) || trimmed.contains(START_MARKER) {
            continue;
        }

        // Skip bare prompt lines (may include path or hostname:path prefix)
        if trimmed == "/ #"
            || trimmed == "/#"
            || (trimmed.ends_with('#')
                && !trimmed.contains(START_MARKER)
                && !trimmed.contains(END_MARKER)
                && looks_like_bare_prompt(trimmed))
        {
            continue;
        }

        output_lines.push(line.to_string());
    }

    // Trim leading/trailing empty lines
    while output_lines.first().is_some_and(|l| l.trim().is_empty()) {
        output_lines.remove(0);
    }
    while output_lines.last().is_some_and(|l| l.trim().is_empty()) {
        output_lines.pop();
    }

    output_lines.join("\n")
}

/// Check if a line is a shell prompt followed by one of the commands we sent.
///
/// Matches patterns like `/ # echo "hello"` where `echo "hello"` is in
/// `sent_commands`. Also handles prompt variants with ANSI codes stripped.
fn is_prompt_echo(line: &str, sent_commands: &[String]) -> bool {
    // Try stripping known prompt patterns
    let after_prompt = if let Some(rest) = line.strip_prefix("/ # ") {
        rest
    } else if let Some(rest) = line.strip_prefix("~ # ") {
        rest
    } else if let Some(idx) = line.find("# ") {
        // Fallback: find "# " anywhere (prompt might have path prefix or
        // hostname:path prefix like `localhost:/out # `).
        let rest = &line[idx + 2..];
        let before = line[..idx].trim();
        if before.starts_with('/')
            || before.is_empty()
            || before.contains(":/")
            || before.contains(":~")
        {
            rest
        } else {
            return false;
        }
    } else {
        return false;
    };

    let after_prompt = after_prompt.trim();

    // Check against sent commands
    for cmd in sent_commands {
        if after_prompt == cmd.trim() {
            return true;
        }
    }

    // Check against marker echo commands
    if after_prompt.contains(START_MARKER) || after_prompt.contains(END_MARKER) {
        return true;
    }

    false
}

/// Check whether a trimmed line looks like a bare shell prompt (no command
/// after the `#`).  Handles `/ #`, `/out #`, `localhost:/out #`, etc.
fn looks_like_bare_prompt(trimmed: &str) -> bool {
    // Must end with '#' (already checked by caller) and the part before '#'
    // should look like a path or hostname:path.
    let before = trimmed.trim_end_matches('#').trim();
    before.is_empty()
        || before.starts_with('/')
        || before.contains(":/")
        || before.contains(":~")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_image_path_exact_file() {
        let dir = std::env::temp_dir().join("hick-executor-test-resolve");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("alpine.wasm"), b"fake").unwrap();

        let result = resolve_image_path(&dir, "alpine");
        assert_eq!(result, dir.join("alpine.wasm"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_image_path_directory() {
        let dir = std::env::temp_dir().join("hick-executor-test-resolve-dir");
        let _ = std::fs::remove_dir_all(&dir);
        let sub = dir.join("ubuntu");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("22.04.wasm"), b"fake1").unwrap();
        std::fs::write(sub.join("24.04.wasm"), b"fake2").unwrap();

        let result = resolve_image_path(&dir, "ubuntu");
        assert_eq!(result, sub.join("24.04.wasm")); // latest by sort

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_image_path_strips_tag() {
        let dir = std::env::temp_dir().join("hick-executor-test-resolve-tag");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("python.wasm"), b"fake").unwrap();

        let result = resolve_image_path(&dir, "python:3.12");
        assert_eq!(result, dir.join("python.wasm"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_image_path_fallback() {
        let dir = std::env::temp_dir().join("hick-executor-test-resolve-miss");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let result = resolve_image_path(&dir, "nonexistent");
        assert_eq!(result, PathBuf::from("nonexistent"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_strip_ansi_codes() {
        let input = "hello\x1b[32m world\x1b[0m\r\n";
        assert_eq!(strip_ansi(input), "hello world\n");
    }

    #[test]
    fn test_strip_ansi_plain() {
        assert_eq!(strip_ansi("hello world"), "hello world");
    }

    #[test]
    fn test_clean_captured_output_strips_prompt_echo() {
        let raw = "/ # echo 'hello'\nhello\n/ # echo '===HICK_END==='\n";
        let cmds = vec!["echo 'hello'".to_string()];
        assert_eq!(clean_captured_output(raw, &cmds), "hello");
    }

    #[test]
    fn test_clean_captured_output_preserves_real_output() {
        let raw = "\n/ # cat /out/stamp.txt\nBuilt by hick\n";
        let cmds = vec!["cat /out/stamp.txt".to_string()];
        assert_eq!(clean_captured_output(raw, &cmds), "Built by hick");
    }

    #[test]
    fn test_prompt_matcher_detects_prompt() {
        let mut pm = PromptMatcher::new();
        let (_, detected) = pm.process(b"booting...\n/ # ");
        assert!(detected);
    }

    #[test]
    fn test_prompt_matcher_partial() {
        let mut pm = PromptMatcher::new();
        let (_, d1) = pm.process(b"/ ");
        assert!(!d1);
        let (_, d2) = pm.process(b"# ");
        assert!(d2);
    }

    #[test]
    fn test_strip_dsr() {
        let input = b"hello\x1b[6nworld";
        let result = strip_dsr(input);
        assert_eq!(result, b"helloworld");
    }

    #[test]
    fn test_is_prompt_echo_hostname_prefix() {
        let cmds = vec!["cd /out && tar cf - . | base64".to_string()];
        // Standard prompt
        assert!(is_prompt_echo(
            "/ # cd /out && tar cf - . | base64",
            &cmds
        ));
        // Path-based prompt after cd
        assert!(is_prompt_echo(
            "/out # cd /out && tar cf - . | base64",
            &cmds
        ));
        // Hostname:path prompt
        assert!(is_prompt_echo(
            "localhost:/ # cd /out && tar cf - . | base64",
            &cmds
        ));
        assert!(is_prompt_echo(
            "localhost:/out # cd /out && tar cf - . | base64",
            &cmds
        ));
    }

    #[test]
    fn test_clean_captured_output_hostname_prompt() {
        let raw = "\nlocalhost:/ # echo 'hello'\nhello\nlocalhost:/ # echo '===HICK_END==='\n";
        let cmds = vec!["echo 'hello'".to_string()];
        assert_eq!(clean_captured_output(raw, &cmds), "hello");
    }

    #[test]
    fn test_clean_captured_output_bare_prompt_with_path() {
        let raw = "\n/ # ls\nfile.txt\n/out #\n";
        let cmds = vec!["ls".to_string()];
        assert_eq!(clean_captured_output(raw, &cmds), "file.txt");
    }

    #[test]
    fn test_clean_captured_output_bare_prompt_hostname_path() {
        let raw = "\n/ # ls\nfile.txt\nlocalhost:/out #\n";
        let cmds = vec!["ls".to_string()];
        assert_eq!(clean_captured_output(raw, &cmds), "file.txt");
    }

    #[test]
    fn test_extract_between_markers() {
        let text = "junk\n===B64S===\nSGVsbG8=\n===B64E===\nmore junk";
        assert_eq!(
            extract_between_markers(text, "===B64S===", "===B64E==="),
            Some("SGVsbG8=")
        );
    }

    #[test]
    fn test_extract_between_markers_ignores_embedded() {
        // Markers embedded inside a longer line (command echo) should be
        // skipped; only standalone marker lines match.
        let text = "/ # echo '===B64S==='; tar | base64; echo '===B64E==='\n===B64S===\nDATA\n===B64E===\n";
        assert_eq!(
            extract_between_markers(text, "===B64S===", "===B64E==="),
            Some("DATA")
        );
    }

    #[test]
    fn test_extract_between_markers_missing() {
        assert_eq!(
            extract_between_markers("no markers here", "===B64S===", "===B64E==="),
            None
        );
    }
}
