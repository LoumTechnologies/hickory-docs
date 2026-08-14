//! The execution boundary for Hickory Docs pipelines.
//!
//! This crate defines the [`Executor`] trait — the only interface the
//! pipeline (`hick-literate`) uses to run commands — plus [`LocalExecutor`],
//! the host-process implementation used for development, CI, and
//! self-hosters who accept its (lack of) isolation.
//!
//! # LocalExecutor semantics — read this before relying on it
//!
//! - **`image` is recorded but IGNORED.** A `<hick:container image="python:3.12">`
//!   declaration runs against the *host* toolchain: whatever `sh`, `python3`,
//!   `awk`, etc. exist on the machine. The image name is kept only for
//!   transcripts and display. `CanopyExecutor` (a later phase) is the
//!   implementation that honors images.
//! - **No sandboxing whatsoever.** Commands run as the invoking user with
//!   full host access, confined only by convention to a per-container
//!   temporary working directory. Never point `LocalExecutor` at untrusted
//!   documents.
//! - **Containers are directories.** Each container name maps to a persistent
//!   temp workdir for the lifetime of the pipeline run; every `execute` is
//!   `sh -c <command>` with that workdir as cwd, so state accumulates across
//!   execs in files (not shell variables — each exec is a fresh shell).
//! - **Forks copy the workdir.** `register_fork(target, from, …)` marks the
//!   target; when the fork target first starts, the source container's
//!   workdir is recursively copied. This approximates the original design's
//!   command-history replay: filesystem state carries over, in-memory state
//!   does not.
//! - **Volumes are tar archives** exchanged with the pipeline's in-memory
//!   `VolumeStore`: `inject_volume` unpacks a tar under the container workdir
//!   at the mount path, `extract_volume` re-tars that subdirectory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result, bail};
use async_trait::async_trait;
use hick_token::ContainerCapabilities;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ---------------------------------------------------------------------------
// Transcript types
// ---------------------------------------------------------------------------

/// One timed event in a container transcript.
///
/// Serializes to the block-model shape from `docs/specs/freeform/api.md`:
/// `{t, kind: "cmd"|"out"|"err", data}` or `{t, kind: "exit", code}`.
/// `t` is milliseconds since the container's transcript epoch (first start).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TranscriptEvent {
    Cmd { t: u64, data: String },
    Out { t: u64, data: String },
    Err { t: u64, data: String },
    Exit { t: u64, code: i32 },
}

impl TranscriptEvent {
    /// Millisecond offset of this event.
    pub fn t_offset_ms(&self) -> u64 {
        match self {
            TranscriptEvent::Cmd { t, .. }
            | TranscriptEvent::Out { t, .. }
            | TranscriptEvent::Err { t, .. }
            | TranscriptEvent::Exit { t, .. } => *t,
        }
    }
}

/// A single exec's transcript entry: command lines, captured stdout, and the
/// full timed event stream (for playback in the web UI).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecTranscriptEntry {
    /// Individual command lines (non-empty, trimmed) — for woven display.
    pub commands: Vec<String>,
    /// Captured stdout of the exec, byte-for-byte.
    pub output: String,
    /// Ordered timed events (cmd/out/err/exit) for this exec.
    pub events: Vec<TranscriptEvent>,
    /// Source line of the `<hick:exec>` tag that produced this entry, when
    /// known. Assigned by the pipeline (the executor does not parse docs).
    pub source_line: Option<usize>,
}

/// Container name → ordered transcript entries.
pub type Transcripts = HashMap<String, Vec<ExecTranscriptEntry>>;

/// Aggregated per-container resource statistics.
///
/// `LocalExecutor` fills in wall-clock durations; isolation-level stats
/// (I/O bytes, pause info) belong to sandboxed executors and default to zero.
#[derive(Debug, Clone, Default)]
pub struct ContainerResourceStats {
    /// Time from container start to readiness (≈0 for `LocalExecutor`).
    pub boot_duration: Duration,
    /// Per-command execution durations.
    pub command_durations: Vec<Duration>,
    /// Sum of `command_durations`.
    pub total_exec_duration: Duration,
}

// ---------------------------------------------------------------------------
// The Executor trait
// ---------------------------------------------------------------------------

/// The execution boundary: everything the pipeline needs from a runtime.
///
/// Implementations must be usable behind `Arc<dyn Executor>`: all methods
/// take `&self` and use interior mutability.
#[async_trait]
pub trait Executor: Send + Sync {
    /// Declare a container's document-derived capabilities.
    ///
    /// The pipeline calls this for every container it knows about *before*
    /// any of them starts, which is what lets `<hick:allow>` / `<hick:deny>`
    /// decide how the container is confined rather than merely describing it.
    /// `ensure_started` takes only a name and an image on purpose: a
    /// container's capabilities are a property of the document, not of the
    /// exec that happens to reach it first.
    ///
    /// Enforcement is per-executor and may be coarser than the declaration
    /// (see [`ContainerCapabilities::allows_network`]). An executor that
    /// cannot confine anything — `LocalExecutor` — records the declaration
    /// and imposes nothing; the default implementation drops it.
    async fn declare_capabilities(
        &self,
        container: &str,
        capabilities: ContainerCapabilities,
    ) -> Result<()> {
        let _ = (container, capabilities);
        Ok(())
    }

    /// Ensure a container exists and is ready. Idempotent.
    ///
    /// `image` is an OCI-style reference (`python:3.12`); whether it is
    /// honored is implementation-defined (see crate docs).
    async fn ensure_started(&self, container: &str, image: &str) -> Result<()>;

    /// Run a command in the container, returning captured stdout.
    ///
    /// The command (and its output) is recorded as a transcript entry with
    /// timed events. A non-zero exit status is an error (the transcript
    /// entry is still recorded first).
    async fn execute(&self, container: &str, command: &str) -> Result<String>;

    /// Like [`Executor::execute`], but with `stdin_data` piped to the command.
    async fn execute_with_stdin(
        &self,
        container: &str,
        command: &str,
        stdin_data: &str,
    ) -> Result<String>;

    /// Register `target` as a fork of `from`. Must be called before `target`
    /// starts. `additional_caps` further attenuates the fork's capabilities
    /// (advisory for executors that don't enforce capabilities).
    async fn register_fork(
        &self,
        target: &str,
        from: &str,
        additional_caps: Option<ContainerCapabilities>,
    ) -> Result<()>;

    /// Create an empty mount-point directory inside the container.
    async fn create_mount_point(&self, container: &str, mount_path: &str) -> Result<()>;

    /// Unpack `tar_data` (a tar archive) into the container at `mount_path`.
    async fn inject_volume(&self, container: &str, mount_path: &str, tar_data: &[u8])
    -> Result<()>;

    /// Tar up the container directory at `mount_path` and return the bytes.
    async fn extract_volume(&self, container: &str, mount_path: &str) -> Result<Vec<u8>>;

    /// Snapshot of all transcripts recorded so far.
    fn transcripts(&self) -> Transcripts;

    /// Append a transcript entry without executing (cache hits, stubs).
    fn inject_transcript_entry(&self, container: &str, entry: ExecTranscriptEntry);

    /// Per-container resource stats. May be a trivial default.
    fn resource_stats(&self) -> HashMap<String, ContainerResourceStats> {
        HashMap::new()
    }

    /// Tear down all containers. Idempotent.
    async fn shutdown(&self) -> Result<()>;
}

// ---------------------------------------------------------------------------
// LocalExecutor
// ---------------------------------------------------------------------------

struct LocalContainer {
    workdir: PathBuf,
    image: String,
    started_at: Instant,
    command_durations: Vec<Duration>,
}

#[derive(Default)]
struct LocalState {
    containers: HashMap<String, LocalContainer>,
    transcripts: Transcripts,
    /// target → (from, advisory caps)
    forks: HashMap<String, (String, Option<ContainerCapabilities>)>,
    /// container → declared capabilities. Recorded, never imposed: this
    /// executor has no sandbox to impose them with.
    declared: HashMap<String, ContainerCapabilities>,
    epoch: Option<Instant>,
}

/// Host-process executor. See the crate docs for its (loudly documented)
/// semantics: **no sandboxing, images ignored, containers are temp dirs.**
pub struct LocalExecutor {
    /// Root temp dir holding one workdir per container; cleaned on drop.
    root: tempfile::TempDir,
    state: Mutex<LocalState>,
}

impl LocalExecutor {
    /// Create an executor whose container workdirs live under a fresh
    /// temporary directory (removed when the executor is dropped).
    pub fn new() -> Result<Self> {
        let root = tempfile::Builder::new()
            .prefix("hickory-local-")
            .tempdir()
            .context("failed to create LocalExecutor temp root")?;
        Ok(Self {
            root,
            state: Mutex::new(LocalState::default()),
        })
    }

    /// The capabilities declared for a container, if any were.
    ///
    /// Present so a caller can *show* what a document asked for; this
    /// executor never acts on it.
    pub fn declared_capabilities(&self, container: &str) -> Option<ContainerCapabilities> {
        self.state.lock().unwrap().declared.get(container).cloned()
    }

    /// The workdir of a container, if started.
    pub fn container_workdir(&self, container: &str) -> Option<PathBuf> {
        let state = self.state.lock().unwrap();
        state.containers.get(container).map(|c| c.workdir.clone())
    }

    fn now_offset_ms(state: &mut LocalState) -> u64 {
        let epoch = *state.epoch.get_or_insert_with(Instant::now);
        epoch.elapsed().as_millis() as u64
    }

    /// A container's workdir on the host.
    ///
    /// Public so a wrapping executor can confine a command to it —
    /// `hickory-executor-sandbox` binds exactly this directory writable and
    /// nothing else. Without it, a sandbox would have to guess the one path
    /// a cell is allowed to touch.
    pub fn workdir_of(&self, container: &str) -> Result<PathBuf> {
        self.workdir_for(container)
    }

    fn workdir_for(&self, container: &str) -> Result<PathBuf> {
        let state = self.state.lock().unwrap();
        state
            .containers
            .get(container)
            .map(|c| c.workdir.clone())
            .ok_or_else(|| anyhow::anyhow!("container '{container}' has not been started"))
    }

    /// Resolve a mount path to a directory under the container workdir.
    /// Leading `/` is stripped: `/data` maps to `<workdir>/data`.
    fn mount_dir(workdir: &Path, mount_path: &str) -> PathBuf {
        let rel = mount_path.trim_start_matches('/');
        if rel.is_empty() {
            workdir.to_path_buf()
        } else {
            workdir.join(rel)
        }
    }

    /// Run `command`, but record `display` in the transcript.
    ///
    /// The two differ only for an executor that wraps the command in
    /// something the reader did not write — a sandbox, say. The transcript is
    /// part of the woven document, so it must show the cell as the author
    /// wrote it; a page of `bwrap --ro-bind …` in the middle of somebody's
    /// documentation is noise about our implementation, not about their work.
    pub async fn execute_as(
        &self,
        container: &str,
        display: &str,
        command: &str,
        stdin_data: Option<&str>,
    ) -> Result<String> {
        self.run_command_as(container, display, command, stdin_data)
            .await
    }

    async fn run_command(
        &self,
        container: &str,
        command: &str,
        stdin_data: Option<&str>,
    ) -> Result<String> {
        self.run_command_as(container, command, command, stdin_data)
            .await
    }

    /// The shell a cell's command is handed to.
    ///
    /// Every platform gets ITS shell rather than a lowest common
    /// denominator: a document's cells are shell commands, and a Windows
    /// user writing `dir` should not need MSYS installed for it to run.
    /// A document that must run everywhere is the author's concern, not
    /// something to enforce by making Windows worse.
    fn shell() -> (&'static str, &'static str) {
        if cfg!(windows) {
            ("cmd.exe", "/C")
        } else {
            ("sh", "-c")
        }
    }

    async fn run_command_as(
        &self,
        container: &str,
        display: &str,
        command: &str,
        stdin_data: Option<&str>,
    ) -> Result<String> {
        let workdir = self.workdir_for(container)?;
        // One entry: the whole command block, indentation preserved (weave
        // renders continuation lines under a single `$ ` prompt).
        let cmd_lines: Vec<String> = vec![display.trim().to_string()];

        let start = Instant::now();
        let mut events = Vec::new();
        {
            let mut state = self.state.lock().unwrap();
            let t = Self::now_offset_ms(&mut state);
            events.push(TranscriptEvent::Cmd {
                t,
                data: display.trim().to_string(),
            });
        }

        info!("[local:{container}] executing: {}", display.trim());
        let (shell, shell_flag) = Self::shell();
        let mut child = tokio::process::Command::new(shell)
            .arg(shell_flag)
            .arg(command)
            .current_dir(&workdir)
            .stdin(if stdin_data.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| {
                format!("failed to spawn `{shell} {shell_flag}` in container '{container}'")
            })?;

        if let Some(data) = stdin_data {
            let mut stdin = child.stdin.take().expect("stdin was piped");
            stdin
                .write_all(data.as_bytes())
                .await
                .context("failed to write stdin")?;
            drop(stdin);
        }

        let mut stdout = child.stdout.take().expect("stdout was piped");
        let mut stderr = child.stderr.take().expect("stderr was piped");

        // Stream both pipes, timestamping each chunk as it arrives.
        let epoch = {
            let mut state = self.state.lock().unwrap();
            let _ = Self::now_offset_ms(&mut state);
            state.epoch.expect("epoch set")
        };
        let out_task = async {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            let mut evs = Vec::new();
            loop {
                match stdout.read(&mut chunk).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let t = epoch.elapsed().as_millis() as u64;
                        evs.push(TranscriptEvent::Out {
                            t,
                            data: String::from_utf8_lossy(&chunk[..n]).into_owned(),
                        });
                        buf.extend_from_slice(&chunk[..n]);
                    }
                    Err(e) => return Err(anyhow::Error::from(e)),
                }
            }
            Ok::<_, anyhow::Error>((buf, evs))
        };
        let err_task = async {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            let mut evs = Vec::new();
            loop {
                match stderr.read(&mut chunk).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let t = epoch.elapsed().as_millis() as u64;
                        evs.push(TranscriptEvent::Err {
                            t,
                            data: String::from_utf8_lossy(&chunk[..n]).into_owned(),
                        });
                        buf.extend_from_slice(&chunk[..n]);
                    }
                    Err(e) => return Err(anyhow::Error::from(e)),
                }
            }
            Ok::<_, anyhow::Error>((buf, evs))
        };
        let ((out_buf, out_evs), (err_buf, err_evs)) = tokio::try_join!(out_task, err_task)?;

        let status = child.wait().await.context("failed to wait for command")?;
        let code = status.code().unwrap_or(-1);
        let exit_t = epoch.elapsed().as_millis() as u64;

        // Merge stdout/stderr events by timestamp (stable within a stream).
        let mut merged: Vec<TranscriptEvent> = out_evs.into_iter().chain(err_evs).collect();
        merged.sort_by_key(|e| e.t_offset_ms());
        events.extend(merged);
        events.push(TranscriptEvent::Exit { t: exit_t, code });

        let output = String::from_utf8_lossy(&out_buf).into_owned();
        let stderr_text = String::from_utf8_lossy(&err_buf).into_owned();

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
                    output: output.clone(),
                    events,
                    source_line: None,
                });
        }

        if !status.success() {
            // `display`, not `command`: under a sandbox the spawned command is
            // a hundred-argument `bwrap` invocation wrapped around the cell,
            // and printing that at someone whose Python script failed buries
            // the one line they need in our implementation. The transcript
            // already makes this distinction; the error must too.
            bail!(
                "command failed in container '{container}' (exit {code}): {}\n{}",
                display.trim().lines().next().unwrap_or("?"),
                stderr_text.trim()
            );
        }
        debug!("[local:{container}] exit {code} in {duration:?}");
        Ok(output)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else if ty.is_file() {
            std::fs::copy(entry.path(), &to)?;
        }
        // Symlinks and special files are skipped: container workdirs are
        // plain data directories created by this executor.
    }
    Ok(())
}

#[async_trait]
impl Executor for LocalExecutor {
    /// Recorded, not imposed. This executor runs commands as the invoking
    /// user with full host access; pretending a declaration confined them
    /// would be worse than admitting it did not. Sandboxed backends
    /// (`DockerExecutor`) are where a declaration changes what runs.
    async fn declare_capabilities(
        &self,
        container: &str,
        capabilities: ContainerCapabilities,
    ) -> Result<()> {
        debug!("[local:{container}] capabilities recorded (not enforced): {capabilities:?}");
        self.state
            .lock()
            .unwrap()
            .declared
            .insert(container.to_string(), capabilities);
        Ok(())
    }

    async fn ensure_started(&self, container: &str, image: &str) -> Result<()> {
        let fork_source = {
            let state = self.state.lock().unwrap();
            if state.containers.contains_key(container) {
                return Ok(());
            }
            state.forks.get(container).map(|(from, _)| from.clone())
        };

        let workdir = self.root.path().join(sanitize_name(container));
        std::fs::create_dir_all(&workdir)
            .with_context(|| format!("failed to create workdir for '{container}'"))?;

        // Fork: clone the source container's workdir (filesystem-state
        // approximation of command-history replay; see crate docs).
        if let Some(from) = fork_source {
            let src = self.workdir_for(&from).with_context(|| {
                format!("fork target '{container}' declared from unstarted container '{from}'")
            })?;
            info!("[local:{container}] forking from '{from}' (copying workdir)");
            copy_dir_recursive(&src, &workdir)?;
        }

        let mut state = self.state.lock().unwrap();
        let _ = Self::now_offset_ms(&mut state);
        info!("[local:{container}] started (image '{image}' recorded but ignored)");
        state.containers.insert(
            container.to_string(),
            LocalContainer {
                workdir,
                image: image.to_string(),
                started_at: Instant::now(),
                command_durations: Vec::new(),
            },
        );
        Ok(())
    }

    async fn execute(&self, container: &str, command: &str) -> Result<String> {
        self.run_command(container, command, None).await
    }

    async fn execute_with_stdin(
        &self,
        container: &str,
        command: &str,
        stdin_data: &str,
    ) -> Result<String> {
        self.run_command(container, command, Some(stdin_data)).await
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
        let workdir = self.workdir_for(container)?;
        let dir = Self::mount_dir(&workdir, mount_path);
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create mount point {mount_path}"))?;
        Ok(())
    }

    async fn inject_volume(
        &self,
        container: &str,
        mount_path: &str,
        tar_data: &[u8],
    ) -> Result<()> {
        let workdir = self.workdir_for(container)?;
        let dir = Self::mount_dir(&workdir, mount_path);
        std::fs::create_dir_all(&dir)?;
        let mut archive = tar::Archive::new(tar_data);
        archive
            .unpack(&dir)
            .with_context(|| format!("failed to unpack volume into {mount_path}"))?;
        Ok(())
    }

    async fn extract_volume(&self, container: &str, mount_path: &str) -> Result<Vec<u8>> {
        let workdir = self.workdir_for(container)?;
        let dir = Self::mount_dir(&workdir, mount_path);
        let mut builder = tar::Builder::new(Vec::new());
        builder
            .append_dir_all(".", &dir)
            .with_context(|| format!("failed to tar volume at {mount_path}"))?;
        Ok(builder.into_inner()?)
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
                        boot_duration: Duration::ZERO,
                        command_durations: c.command_durations.clone(),
                        total_exec_duration: total,
                    },
                )
            })
            .collect()
    }

    async fn shutdown(&self) -> Result<()> {
        // Nothing persistent to tear down: workdirs are removed when the
        // TempDir root drops. Keep container metadata for resource_stats().
        let state = self.state.lock().unwrap();
        for (name, c) in &state.containers {
            debug!(
                "[local:{name}] shutdown (image '{}', up {:?})",
                c.image,
                c.started_at.elapsed()
            );
        }
        Ok(())
    }
}

/// Make a container name safe as a directory name.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn execute_captures_stdout_and_records_events() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c1", "alpine:3.20").await.unwrap();
        let out = ex.execute("c1", "printf 'hello\\n'").await.unwrap();
        assert_eq!(out, "hello\n");

        let ts = ex.transcripts();
        let entries = &ts["c1"];
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].output, "hello\n");
        let kinds: Vec<&str> = entries[0]
            .events
            .iter()
            .map(|e| match e {
                TranscriptEvent::Cmd { .. } => "cmd",
                TranscriptEvent::Out { .. } => "out",
                TranscriptEvent::Err { .. } => "err",
                TranscriptEvent::Exit { .. } => "exit",
            })
            .collect();
        assert_eq!(kinds.first(), Some(&"cmd"));
        assert_eq!(kinds.last(), Some(&"exit"));
        assert!(kinds.contains(&"out"));
        // Timestamps are monotonically non-decreasing.
        let ts_ms: Vec<u64> = entries[0].events.iter().map(|e| e.t_offset_ms()).collect();
        assert!(ts_ms.windows(2).all(|w| w[0] <= w[1]));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn state_persists_across_execs_in_workdir() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c", "alpine").await.unwrap();
        ex.execute("c", "echo data > f.txt").await.unwrap();
        let out = ex.execute("c", "cat f.txt").await.unwrap();
        assert_eq!(out, "data\n");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn nonzero_exit_is_error_but_transcript_recorded() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c", "alpine").await.unwrap();
        let err = ex.execute("c", "echo oops >&2; exit 3").await.unwrap_err();
        assert!(err.to_string().contains("exit 3"), "err: {err}");
        assert!(err.to_string().contains("oops"), "err: {err}");
        let ts = ex.transcripts();
        assert!(
            ts["c"][0]
                .events
                .iter()
                .any(|e| matches!(e, TranscriptEvent::Exit { code: 3, .. }))
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn stdin_is_honored() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c", "alpine").await.unwrap();
        let out = ex
            .execute_with_stdin("c", "cat", "piped input")
            .await
            .unwrap();
        assert_eq!(out, "piped input");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fork_copies_source_workdir() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("base", "alpine").await.unwrap();
        ex.execute("base", "echo shared > seed.txt").await.unwrap();
        ex.register_fork("child", "base", None).await.unwrap();
        ex.ensure_started("child", "alpine").await.unwrap();
        let out = ex.execute("child", "cat seed.txt").await.unwrap();
        assert_eq!(out, "shared\n");
        // Divergence: writes in the fork do not affect the source.
        ex.execute("child", "echo forked > only.txt").await.unwrap();
        let err = ex.execute("base", "cat only.txt").await;
        assert!(err.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn volume_roundtrip() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("w", "alpine").await.unwrap();
        ex.create_mount_point("w", "/data").await.unwrap();
        ex.execute("w", "echo v1 > data/file.txt").await.unwrap();
        let tar_bytes = ex.extract_volume("w", "/data").await.unwrap();

        ex.ensure_started("r", "alpine").await.unwrap();
        ex.inject_volume("r", "/incoming", &tar_bytes)
            .await
            .unwrap();
        let out = ex.execute("r", "cat incoming/file.txt").await.unwrap();
        assert_eq!(out, "v1\n");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn image_is_ignored() {
        let ex = LocalExecutor::new().unwrap();
        // A nonsense image must not prevent execution on the host.
        ex.ensure_started("c", "no-such-image:99").await.unwrap();
        let out = ex.execute("c", "echo ok").await.unwrap();
        assert_eq!(out, "ok\n");
    }
}
