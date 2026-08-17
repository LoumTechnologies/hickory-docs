//! [`CanopyExecutor`]: the [`Executor`] implementation backed by a Cloud
//! Canopy node agent (gRPC `SpawnSandbox` / `AttachSandbox` /
//! `DestroySandbox`, guest pty protocol over the attach stream).
//!
//! # Semantics relative to `LocalExecutor`
//!
//! - **Images are honored.** Every `.hick` image ref must map through
//!   `CANOPY_IMAGE_MAP` to a Nix store image path in the tenant ledger.
//! - **One microVM per container.** Each `ensure_started` spawns a sandbox;
//!   `shutdown` destroys every sandbox this executor spawned.
//! - **Each exec is a fresh shell**, exactly like `LocalExecutor`: every
//!   `execute` opens a fresh attach stream, and the guest hands each
//!   connection a new shell. State accumulates in the guest filesystem. All
//!   commands run under a fixed workdir (`/hickory-work`), so relative paths
//!   behave like `LocalExecutor`'s per-container temp dir.
//! - **Output is a pty**: stdout and stderr arrive merged, so transcripts
//!   record `out` events only (no `err` events) and the captured output
//!   includes anything the command wrote to stderr.
//! - **Forks replay command history.** `register_fork(target, from)` makes
//!   the target's first start spawn a fresh sandbox and re-execute `from`'s
//!   recorded commands (transcripts hold them). Stdin payloads are not part
//!   of the transcript and are not replayed; a replayed command that fails
//!   is logged and skipped (the original run already surfaced its failure).
//! - **Volumes go over the attach stream as base64-chunked tar.** There is
//!   no file-transfer RPC, so tar bytes are shuttled through the guest pty
//!   in shell-safe base64 lines (~2 KiB of payload per round-trip — the pty
//!   line discipline caps line length at 4096 bytes). Correct and
//!   dependency-free, but slow for large volumes; a dedicated transfer
//!   channel is the upgrade path if volumes grow.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result, bail};
use async_trait::async_trait;
use base64::Engine as _;
use hick_token::ContainerCapabilities;
use hickory_executor::{
    ContainerResourceStats, ExecTranscriptEntry, Executor, TranscriptEvent, Transcripts,
};
use log::{debug, info, warn};
use tokio_stream::StreamExt as _;
use tonic::transport::{Channel, Endpoint};
// Only the unix-socket connector names the (placeholder) URI type.
#[cfg(unix)]
use tonic::transport::Uri;

use crate::config::CanopyConfig;
use crate::frame::{FrameItem, FrameParser, READY_BANNER, exec_line, shell_quote};
use crate::pb;
use crate::pb::canopy_agent_client::CanopyAgentClient;

/// Fixed working directory for every exec inside a sandbox. Mirrors
/// `LocalExecutor`'s per-container workdir so relative paths behave the same.
const GUEST_WORKDIR: &str = "/hickory-work";

/// Base64 payload bytes carried per attach round-trip when shuttling volume
/// data. The full line (workdir prefix + printf + payload, then re-encoded
/// once more by the exec framing) must stay under the pty's 4096-byte
/// canonical-mode line limit; 2048 leaves comfortable headroom.
const VOLUME_CHUNK: usize = 2048;

static SANDBOX_SEQ: AtomicU64 = AtomicU64::new(0);

struct SandboxInfo {
    sandbox_id: String,
    boot_duration: Duration,
    command_durations: Vec<Duration>,
}

#[derive(Default)]
struct CanopyState {
    containers: HashMap<String, SandboxInfo>,
    transcripts: Transcripts,
    /// target → (from, advisory caps)
    forks: HashMap<String, (String, Option<ContainerCapabilities>)>,
    epoch: Option<Instant>,
    /// Every sandbox id this executor ever spawned (destroyed on shutdown).
    spawned: Vec<String>,
    volseq: u64,
}

/// Executor backed by a Cloud Canopy node agent. Construct with
/// [`CanopyExecutor::from_env`] (deployment config) or
/// [`CanopyExecutor::new`] (explicit config — what the server and tests use).
///
/// Connection to the agent is lazy: construction only validates config, so
/// executor selection can happen before the agent is reachable.
pub struct CanopyExecutor {
    config: CanopyConfig,
    state: Mutex<CanopyState>,
    channel: tokio::sync::Mutex<Option<Channel>>,
}

/// Dial a canopy node agent listening on a unix domain socket.
///
/// Split out of [`CanopyExecutor::channel`] and `cfg`-gated because unix
/// domain sockets do not exist on Windows: `tokio::net::UnixStream` is not
/// compiled there at all, so the *only* thing that keeps this crate — and
/// therefore the whole `hick` binary — from building for
/// `x86_64-pc-windows-msvc` is this one connector. Gating it here keeps the
/// mesh (`host:port`) path, which is plain TCP, working identically on every
/// platform. See `docs/guarantees/release/a-download-runs-without-a-rust-toolchain.md`.
#[cfg(unix)]
async fn connect_unix_socket(target: &str) -> Result<Channel> {
    let path = target.to_string();
    // The URI is a placeholder; the connector ignores it.
    Endpoint::try_from("http://[::]:1")?
        .connect_with_connector(tower::service_fn(move |_: Uri| {
            let path = path.clone();
            async move {
                let stream = tokio::net::UnixStream::connect(&path).await?;
                Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(stream))
            }
        }))
        .await
        .with_context(|| {
            format!(
                "could not connect to canopy-agent at unix socket {target}. Is the \
                 agent running on this machine? For a remote node set CANOPY_AGENT \
                 to its mesh host:port instead"
            )
        })
}

/// The Windows stand-in for the connector above: everything else about the
/// canopy executor works here, so this fails only at the point where a unix
/// socket was actually asked for, and says what to set instead.
#[cfg(not(unix))]
async fn connect_unix_socket(target: &str) -> Result<Channel> {
    bail!(
        "CANOPY_AGENT={target} is a unix socket path, and this platform has no \
         unix domain sockets. A local canopy node agent can only be reached over \
         a unix socket, so it cannot be used from here. Point CANOPY_AGENT at the \
         agent's mesh address instead (e.g. CANOPY_AGENT=10.77.0.1:7433, with \
         CANOPY_TOKEN set to the capability token minted for that node), or run \
         documents locally with HICKORY_EXECUTOR=local"
    )
}

impl CanopyExecutor {
    /// Build from explicit configuration.
    pub fn new(config: CanopyConfig) -> Self {
        Self {
            config,
            state: Mutex::new(CanopyState::default()),
            channel: tokio::sync::Mutex::new(None),
        }
    }

    /// Build from the `CANOPY_*` environment
    /// (see `docs/specs/freeform/canopy-integration.md`).
    pub fn from_env() -> Result<Self> {
        Ok(Self::new(CanopyConfig::from_env()?))
    }

    /// The configuration in force (for health endpoints).
    pub fn config(&self) -> &CanopyConfig {
        &self.config
    }

    fn now_ms(&self) -> u64 {
        let mut state = self.state.lock().unwrap();
        let epoch = *state.epoch.get_or_insert_with(Instant::now);
        epoch.elapsed().as_millis() as u64
    }

    fn sandbox_id_of(&self, container: &str) -> Result<String> {
        let state = self.state.lock().unwrap();
        state
            .containers
            .get(container)
            .map(|c| c.sandbox_id.clone())
            .ok_or_else(|| anyhow::anyhow!("container '{container}' has not been started"))
    }

    /// Lazily connect to the agent: unix socket if `CANOPY_AGENT` is an
    /// absolute path, otherwise `host:port` over TCP (mesh address).
    async fn channel(&self) -> Result<Channel> {
        let mut guard = self.channel.lock().await;
        if let Some(c) = &*guard {
            return Ok(c.clone());
        }
        let target = self.config.agent.clone();
        let channel = if target.starts_with('/') {
            connect_unix_socket(&target).await?
        } else {
            Endpoint::try_from(format!("http://{target}"))?
                .connect()
                .await
                .with_context(|| {
                    format!(
                        "could not connect to canopy-agent at {target}. Check that the node \
                         is up, that this machine is on its WireGuard mesh (the agent only \
                         listens on the mesh address), and that CANOPY_TOKEN is set"
                    )
                })?
        };
        *guard = Some(channel.clone());
        Ok(channel)
    }

    async fn client(&self) -> Result<CanopyAgentClient<Channel>> {
        Ok(CanopyAgentClient::new(self.channel().await?))
    }

    /// Attach the capability token (raw biscuit bytes) as the binary
    /// metadata the agent expects. An absent token is left absent — over
    /// the local unix socket, reaching the socket is the trust.
    fn req<T>(&self, msg: T) -> tonic::Request<T> {
        let mut r = tonic::Request::new(msg);
        if let Some(bytes) = &self.config.token {
            r.metadata_mut().insert_bin(
                tonic::metadata::MetadataKey::from_static("x-canopy-capability-bin"),
                tonic::metadata::MetadataValue::from_bytes(bytes),
            );
        }
        r
    }

    /// Open an attach stream to `sandbox_id`, optionally writing `payload`
    /// in the first message. Returns the inbound chunk stream (the sender
    /// half is returned too so callers could keep writing; execs don't).
    async fn attach(
        &self,
        sandbox_id: &str,
        payload: Vec<u8>,
    ) -> Result<tonic::Streaming<pb::AttachSandboxChunk>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<pb::AttachSandboxRequest>(4);
        tx.send(pb::AttachSandboxRequest {
            sandbox_id: sandbox_id.to_string(),
            data: payload,
        })
        .await
        .expect("receiver alive");
        // The sender is dropped here: execs send everything up front (the
        // whole script is one line), and the agent keeps the guest→client
        // direction open regardless.
        let outbound = tokio_stream::wrappers::ReceiverStream::new(rx);
        let resp = self
            .client()
            .await?
            .attach_sandbox(self.req(outbound))
            .await
            .with_context(|| format!("AttachSandbox failed for sandbox '{sandbox_id}'"))?;
        Ok(resp.into_inner())
    }

    /// Run `script` in the guest and capture sentinel-framed output with
    /// per-line timed events. The exec framing (base64 + sentinels) mirrors
    /// cloud-canopy's reference client; see [`crate::frame`].
    async fn attach_exec(&self, sandbox_id: &str, script: &str) -> Result<AttachExecResult> {
        let line = exec_line(script);
        let mut inbound = self.attach(sandbox_id, line.into_bytes()).await?;

        let deadline = Instant::now() + self.config.exec_timeout;
        let mut parser = FrameParser::new();
        let mut output = String::new();
        let mut out_events: Vec<TranscriptEvent> = Vec::new();
        loop {
            let now = Instant::now();
            if now >= deadline {
                bail!(
                    "the guest did not finish within {}s (CANOPY_EXEC_TIMEOUT_SECS). \
                     Output so far:\n{output}",
                    self.config.exec_timeout.as_secs()
                );
            }
            let chunk = match tokio::time::timeout(deadline - now, inbound.next()).await {
                Err(_) => continue, // deadline check at loop top will fire
                Ok(None) => bail!(
                    "the guest channel ended before the command finished. Output so far:\n{output}"
                ),
                Ok(Some(item)) => {
                    item.context("the attach stream failed mid-command (agent restarted, or the sandbox hit its deadline)")?
                }
            };
            for item in parser.push(&chunk.data) {
                match item {
                    FrameItem::Line(l) => {
                        let t = self.now_ms();
                        out_events.push(TranscriptEvent::Out { t, data: l.clone() });
                        output.push_str(&l);
                    }
                    FrameItem::Done(code) => {
                        let exit_t = self.now_ms();
                        return Ok(AttachExecResult {
                            output,
                            exit_code: code,
                            out_events,
                            exit_t,
                        });
                    }
                }
            }
        }
    }

    /// Run `script` without recording anything (plumbing: mounts, volumes,
    /// fork replay). Non-zero exit is an error.
    async fn silent_exec(&self, container: &str, script: &str) -> Result<String> {
        let sandbox_id = self.sandbox_id_of(container)?;
        let full = format!("mkdir -p {GUEST_WORKDIR} && cd {GUEST_WORKDIR}\n{script}");
        let r = self.attach_exec(&sandbox_id, &full).await?;
        if r.exit_code != 0 {
            bail!(
                "internal command failed in container '{container}' (exit {}): {}\n{}",
                r.exit_code,
                script.lines().next().unwrap_or("?"),
                r.output.trim()
            );
        }
        Ok(r.output)
    }

    /// Wait for the guest agent's ready banner. The channel socket exists
    /// before the guest has booted, so spawning is not readiness — polling
    /// for the banner is the only honest signal.
    async fn wait_ready(&self, sandbox_id: &str) -> Result<()> {
        let deadline = Instant::now() + self.config.boot_timeout;
        let mut last = String::new();
        while Instant::now() < deadline {
            match self.probe_banner(sandbox_id).await {
                Ok(seen) => {
                    if seen.contains(READY_BANNER) {
                        return Ok(());
                    }
                    last = seen;
                }
                Err(e) => {
                    debug!("[canopy:{sandbox_id}] not attachable yet: {e:#}");
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        bail!(
            "the guest in sandbox '{sandbox_id}' never became ready within {}s \
             (CANOPY_BOOT_TIMEOUT_SECS). It may still be booting, or the image may \
             have failed to start — check the node's vmm.log. Last thing the \
             channel said: {:?}",
            self.config.boot_timeout.as_secs(),
            last.chars().take(200).collect::<String>()
        )
    }

    /// One banner probe: attach, read for up to 3s, report accumulated text.
    async fn probe_banner(&self, sandbox_id: &str) -> Result<String> {
        let mut inbound = self.attach(sandbox_id, Vec::new()).await?;
        let until = Instant::now() + Duration::from_secs(3);
        let mut seen = String::new();
        loop {
            let now = Instant::now();
            if now >= until || seen.contains(READY_BANNER) {
                return Ok(seen);
            }
            match tokio::time::timeout(until - now, inbound.next()).await {
                Err(_) | Ok(None) => return Ok(seen),
                Ok(Some(Ok(chunk))) => {
                    seen.push_str(&String::from_utf8_lossy(&chunk.data));
                }
                Ok(Some(Err(e))) => return Err(e.into()),
            }
        }
    }

    /// Spawn one sandbox for `container` and wait for its guest.
    async fn spawn_for(&self, container: &str, image: &str) -> Result<(String, Duration)> {
        let store_path = self.config.resolve_image(image)?;
        let seq = SANDBOX_SEQ.fetch_add(1, Ordering::Relaxed);
        let sandbox_id = format!(
            "hickory-{}-{}-{seq}",
            sanitize_id(container),
            std::process::id()
        );

        let started = Instant::now();
        let resp = self
            .client()
            .await?
            .spawn_sandbox(self.req(pb::SpawnSandboxRequest {
                sandbox_id: sandbox_id.clone(),
                image: store_path.clone(),
                vcpus: self.config.vcpus,
                mem_mib: self.config.mem_mib,
                lifetime_secs: self.config.lifetime_secs,
                egress_hosts: self.config.egress_hosts.clone(),
            }))
            .await
            .context("SpawnSandbox RPC failed (is the agent reachable, and does CANOPY_TOKEN permit spawn_sandbox?)")?
            .into_inner();
        if !resp.error.is_empty() {
            bail!(
                "canopy refused to spawn sandbox for container '{container}' \
                 (image '{image}' -> {store_path}): {}. Check that the image path is \
                 in the tenant ledger's allowlist and that CANOPY_VCPUS/CANOPY_MEM_MIB/\
                 CANOPY_LIFETIME_SECS fit the token's limits",
                resp.error
            );
        }
        {
            let mut state = self.state.lock().unwrap();
            state.spawned.push(sandbox_id.clone());
        }
        info!(
            "[canopy:{container}] spawned sandbox '{sandbox_id}' (deadline {})",
            resp.deadline
        );

        self.wait_ready(&sandbox_id).await?;
        Ok((sandbox_id, started.elapsed()))
    }

    /// Recorded command history of `from`, for fork replay.
    fn command_history(&self, from: &str) -> Vec<String> {
        let state = self.state.lock().unwrap();
        state
            .transcripts
            .get(from)
            .map(|entries| {
                entries
                    .iter()
                    .flat_map(|e| e.commands.iter().cloned())
                    .filter(|c| !c.trim().is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Guest-side directory for a mount path: `/data` → `/hickory-work/data`
    /// (mirrors `LocalExecutor::mount_dir`, so documents behave identically
    /// under both executors).
    fn mount_dir(mount_path: &str) -> String {
        let rel = mount_path.trim_start_matches('/');
        if rel.is_empty() {
            GUEST_WORKDIR.to_string()
        } else {
            format!("{GUEST_WORKDIR}/{rel}")
        }
    }
}

struct AttachExecResult {
    output: String,
    exit_code: i32,
    out_events: Vec<TranscriptEvent>,
    exit_t: u64,
}

#[async_trait]
impl Executor for CanopyExecutor {
    async fn ensure_started(&self, container: &str, image: &str) -> Result<()> {
        let fork_source = {
            let state = self.state.lock().unwrap();
            if state.containers.contains_key(container) {
                return Ok(());
            }
            state.forks.get(container).map(|(from, _)| from.clone())
        };

        let (sandbox_id, boot) = self.spawn_for(container, image).await?;
        {
            let mut state = self.state.lock().unwrap();
            let _ = state.epoch.get_or_insert_with(Instant::now);
            state.containers.insert(
                container.to_string(),
                SandboxInfo {
                    sandbox_id,
                    boot_duration: boot,
                    command_durations: Vec::new(),
                },
            );
        }

        // Fork: replay the source container's command history in the fresh
        // sandbox. Failures are logged, not fatal — a command that failed in
        // the source already surfaced its error there.
        if let Some(from) = fork_source {
            let history = self.command_history(&from);
            info!(
                "[canopy:{container}] forking from '{from}': replaying {} command(s)",
                history.len()
            );
            for cmd in history {
                if let Err(e) = self.silent_exec(container, &cmd).await {
                    warn!("[canopy:{container}] fork replay of a command failed: {e:#}");
                }
            }
        }
        Ok(())
    }

    // `execute_with_options` is NOT overridden here, deliberately: this
    // executor runs cells UNBOUNDED. The command executes on a remote node
    // the user runs, and abandoning the HTTP await here would not kill the
    // remote process — a real timeout needs the canopy API to carry and
    // enforce one. Until it does, saying "no timeout" is more honest than
    // pretending.
    // docs/guarantees/execution/a-cell-cannot-hang-a-run.md
    async fn execute(&self, container: &str, command: &str) -> Result<String> {
        let sandbox_id = self.sandbox_id_of(container)?;
        let cmd_lines: Vec<String> = vec![command.trim().to_string()];

        let start = Instant::now();
        let mut events = vec![TranscriptEvent::Cmd {
            t: self.now_ms(),
            data: command.trim().to_string(),
        }];

        info!("[canopy:{container}] executing: {}", command.trim());
        let script = format!("mkdir -p {GUEST_WORKDIR} && cd {GUEST_WORKDIR}\n{command}");
        let result = self.attach_exec(&sandbox_id, &script).await?;
        events.extend(result.out_events.clone());
        events.push(TranscriptEvent::Exit {
            t: result.exit_t,
            code: result.exit_code,
        });

        let duration = start.elapsed();
        {
            let mut state = self.state.lock().unwrap();
            if let Some(c) = state.containers.get_mut(container) {
                c.command_durations.push(duration);
            }
            state
                .transcripts
                .entry(container.to_string())
                .or_default()
                .push(ExecTranscriptEntry {
                    commands: cmd_lines,
                    output: result.output.clone(),
                    events,
                    source_line: None,
                });
        }

        if result.exit_code != 0 {
            bail!(
                "command failed in container '{container}' (exit {}): {}\n{}",
                result.exit_code,
                command.trim().lines().next().unwrap_or("?"),
                result.output.trim()
            );
        }
        debug!("[canopy:{container}] exit 0 in {duration:?}");
        Ok(result.output)
    }

    async fn execute_with_stdin(
        &self,
        container: &str,
        command: &str,
        stdin_data: &str,
    ) -> Result<String> {
        // The exec framing pipes the script itself into /bin/sh, so the
        // command cannot read our stdin directly. Ship the payload base64-
        // encoded inside the script and pipe it into the command instead.
        let b64 = base64::engine::general_purpose::STANDARD.encode(stdin_data);
        let wrapped = format!(
            "printf '%s' {b64} | base64 -d | /bin/sh -c {}",
            shell_quote(command)
        );
        self.execute(container, &wrapped).await
    }

    async fn register_fork(
        &self,
        target: &str,
        from: &str,
        additional_caps: Option<ContainerCapabilities>,
    ) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state
            .forks
            .insert(target.to_string(), (from.to_string(), additional_caps));
        Ok(())
    }

    async fn create_mount_point(&self, container: &str, mount_path: &str) -> Result<()> {
        let dir = Self::mount_dir(mount_path);
        self.silent_exec(container, &format!("mkdir -p {}", shell_quote(&dir)))
            .await?;
        Ok(())
    }

    async fn inject_volume(
        &self,
        container: &str,
        mount_path: &str,
        tar_data: &[u8],
    ) -> Result<()> {
        let dir = Self::mount_dir(mount_path);
        let staging = {
            let mut state = self.state.lock().unwrap();
            state.volseq += 1;
            format!("{GUEST_WORKDIR}/.hickory-vol-{}.b64", state.volseq)
        };
        let b64 = base64::engine::general_purpose::STANDARD.encode(tar_data);

        // Base64 tar over the pty, chunked to respect the 4096-byte line
        // limit. Slow for big volumes (one attach round-trip per ~2 KiB) —
        // see the crate docs for the tradeoff.
        self.silent_exec(container, &format!("rm -f {}", shell_quote(&staging)))
            .await?;
        for chunk in b64.as_bytes().chunks(VOLUME_CHUNK) {
            let part = std::str::from_utf8(chunk).expect("base64 is ascii");
            self.silent_exec(
                container,
                &format!("printf '%s' {part} >> {}", shell_quote(&staging)),
            )
            .await?;
        }
        self.silent_exec(
            container,
            &format!(
                "mkdir -p {dir_q} && base64 -d < {st_q} | tar -x -C {dir_q} && rm -f {st_q}",
                dir_q = shell_quote(&dir),
                st_q = shell_quote(&staging),
            ),
        )
        .await
        .with_context(|| format!("failed to unpack volume into {mount_path}"))?;
        Ok(())
    }

    async fn extract_volume(&self, container: &str, mount_path: &str) -> Result<Vec<u8>> {
        let dir = Self::mount_dir(mount_path);
        let out = self
            .silent_exec(
                container,
                &format!("tar -c -C {} . | base64", shell_quote(&dir)),
            )
            .await
            .with_context(|| format!("failed to tar volume at {mount_path}"))?;
        let compact: String = out.chars().filter(|c| !c.is_whitespace()).collect();
        base64::engine::general_purpose::STANDARD
            .decode(&compact)
            .context("guest returned invalid base64 for the volume tar")
    }

    fn transcripts(&self) -> Transcripts {
        self.state.lock().unwrap().transcripts.clone()
    }

    fn inject_transcript_entry(&self, container: &str, entry: ExecTranscriptEntry) {
        self.state
            .lock()
            .unwrap()
            .transcripts
            .entry(container.to_string())
            .or_default()
            .push(entry);
    }

    fn resource_stats(&self) -> HashMap<String, ContainerResourceStats> {
        let state = self.state.lock().unwrap();
        state
            .containers
            .iter()
            .map(|(name, c)| {
                let total = c.command_durations.iter().sum();
                (
                    name.clone(),
                    ContainerResourceStats {
                        boot_duration: c.boot_duration,
                        command_durations: c.command_durations.clone(),
                        total_exec_duration: total,
                    },
                )
            })
            .collect()
    }

    async fn shutdown(&self) -> Result<()> {
        let to_destroy: Vec<String> = {
            let mut state = self.state.lock().unwrap();
            std::mem::take(&mut state.spawned)
        };
        if to_destroy.is_empty() {
            return Ok(());
        }
        let mut client = self.client().await?;
        let mut failures = Vec::new();
        for sandbox_id in to_destroy {
            match client
                .destroy_sandbox(self.req(pb::DestroySandboxRequest {
                    sandbox_id: sandbox_id.clone(),
                }))
                .await
            {
                Ok(resp) => {
                    let resp = resp.into_inner();
                    if resp.destroyed || resp.error.is_empty() {
                        debug!("[canopy] destroyed sandbox '{sandbox_id}'");
                    } else {
                        failures.push(format!("{sandbox_id}: {}", resp.error));
                    }
                }
                Err(e) => failures.push(format!("{sandbox_id}: {e}")),
            }
        }
        if !failures.is_empty() {
            bail!(
                "failed to destroy {} sandbox(es) — they will die at their lifetime \
                 deadline, but until then they hold node resources: {}",
                failures.len(),
                failures.join("; ")
            );
        }
        Ok(())
    }
}

/// Make a container name safe inside a sandbox id (lowercase alnum and '-').
fn sanitize_id(name: &str) -> String {
    let mut out: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    out.truncate(32);
    if out.is_empty() {
        out.push('c');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_id_produces_safe_names() {
        assert_eq!(sanitize_id("My Container/1"), "my-container-1");
        assert_eq!(sanitize_id(""), "c");
    }

    #[test]
    fn mount_dir_mirrors_local_executor_semantics() {
        assert_eq!(CanopyExecutor::mount_dir("/data"), "/hickory-work/data");
        assert_eq!(CanopyExecutor::mount_dir("data"), "/hickory-work/data");
        assert_eq!(CanopyExecutor::mount_dir("/"), "/hickory-work");
    }
}
