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

mod capture;
mod program;

pub use capture::{CapturedStream, normalize_captured_newlines};
pub use program::{find_program, find_program_in};

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
    /// Captured stdout of the exec, byte-for-byte **after** the one rewrite
    /// every executor owes it: `\r\n` becomes `\n`, on every platform. See
    /// [`normalize_captured_newlines`] for why that is a property of capture
    /// rather than of comparison.
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
// Per-exec options
// ---------------------------------------------------------------------------

/// Per-exec options carried from a cell's attributes (and the run's
/// configuration) down to where the process is spawned.
///
/// `Default` is "no limit": the plain [`Executor::execute`] /
/// [`Executor::execute_with_stdin`] methods keep their historical unbounded
/// semantics, and the pipeline passes an explicit timeout through
/// [`Executor::execute_with_options`] instead.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExecOptions {
    /// Wall-clock limit for the command. `None` means unbounded. On timeout
    /// the process (and, where the platform allows, its whole process group)
    /// is killed and the exec fails.
    pub timeout: Option<Duration>,
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

    /// Ask a yes/no question of the container, without recording anything.
    ///
    /// Used by the `<hick:needs>` preflight to find out whether a program is
    /// visible to the thing that will run the cells — which is not the same
    /// question as whether it is on the host's PATH, since a sandboxed cell
    /// has a different view and a container has an entirely different
    /// filesystem.
    ///
    /// It must NOT appear in the transcript: a probe is our bookkeeping, and
    /// a woven document listing `command -v duckdb` next to the author's own
    /// commands would be a page about our implementation.
    ///
    /// The default is `true` — "assume it is present". An executor that
    /// cannot answer the question must not be able to block a document from
    /// running over a check it never performed.
    /// Is `bin` a program a cell in this container could run?
    ///
    /// A program NAME, not a shell command. Which shell a cell gets depends on
    /// the executor and the platform — `sh` here, `cmd.exe` on Windows, `sh`
    /// inside a Linux container on a Windows host — and the caller cannot know
    /// which, so a probe phrased as shell text is answerable by only one of
    /// them. `<hick:needs>` used to send `command -v '<bin>' > /dev/null 2>&1`,
    /// which reported every tool on Windows as missing.
    ///
    /// The default answers `true`: an executor that cannot check must not
    /// block a document over a check it did not perform.
    async fn probe_program(&self, container: &str, bin: &str) -> Result<bool> {
        let _ = (container, bin);
        Ok(true)
    }

    /// Run a command in the container, returning captured stdout.
    ///
    /// The command (and its output) is recorded as a transcript entry with
    /// timed events. A non-zero exit status is an error (the transcript
    /// entry is still recorded first).
    ///
    /// **Part of this trait's contract, not an implementation detail:** the
    /// returned stdout and everything recorded alongside it use `\n` line
    /// endings, whatever the host shell emitted. An implementation that
    /// decodes bytes from a process must pass them through
    /// [`normalize_captured_newlines`] (or [`CapturedStream`], if it decodes
    /// in chunks). Without it a document's `<hick:expect match="exact">` means
    /// something different per platform — see that function's docs and
    /// `docs/guarantees/verification/an-expectation-means-the-same-on-every-platform.md`.
    async fn execute(&self, container: &str, command: &str) -> Result<String>;

    /// Like [`Executor::execute`], but with `stdin_data` piped to the command.
    async fn execute_with_stdin(
        &self,
        container: &str,
        command: &str,
        stdin_data: &str,
    ) -> Result<String>;

    /// Run a command with per-exec [`ExecOptions`] — today, a wall-clock
    /// timeout. This is the method the pipeline calls for every cell.
    ///
    /// **The default implementation ignores `options.timeout`** and delegates
    /// to [`Executor::execute`] / [`Executor::execute_with_stdin`]: an
    /// executor that does not override this runs the cell unbounded. The
    /// process-spawning executors (`LocalExecutor`, `SandboxedExecutor`)
    /// override it and really kill the process on timeout; see
    /// `docs/guarantees/execution/a-cell-cannot-hang-a-run.md` for what is
    /// guaranteed where.
    async fn execute_with_options(
        &self,
        container: &str,
        command: &str,
        stdin_data: Option<&str>,
        options: ExecOptions,
    ) -> Result<String> {
        let _ = options;
        match stdin_data {
            Some(data) => self.execute_with_stdin(container, command, data).await,
            None => self.execute(container, command).await,
        }
    }

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
/// How a command reaches the operating system.
///
/// Two shapes, because there are two genuinely different things: a cell is
/// shell text an author wrote, and a confined cell is a program plus its
/// arguments that some sandbox composed. Collapsing the second into the first
/// means re-quoting an argv into a string for a shell to take apart again, and
/// every quoting rule that string has to survive is a place to be wrong.
#[derive(Clone, Copy)]
enum Launch<'a> {
    /// Shell text, handed to the platform shell.
    Shell(&'a str),
    /// A program and its arguments, spawned directly.
    Argv(&'a str, &'a [String]),
}

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

    /// Every OTHER container's workdir and private tmp, for a sandbox that has
    /// to be told what to hide.
    ///
    /// Bubblewrap gets this from namespaces: it binds one workdir into the
    /// cell's mount namespace and the rest simply do not exist. Seatbelt has no
    /// namespaces and allows reads globally, so it has to name what to deny —
    /// and naming the *peers* rather than the directory they all sit in
    /// matters, because that directory also holds things a cell is entitled to
    /// reach, such as a volume mounted into its own workdir.
    ///
    /// Only containers started so far are listed, which is what the caller
    /// builds a policy from at the moment it runs a command.
    pub fn peer_dirs(&self, except: &str) -> Vec<PathBuf> {
        let state = self.state.lock().unwrap();
        state
            .containers
            .iter()
            .filter(|(name, _)| name.as_str() != except)
            .flat_map(|(name, container)| {
                let workdir = container.workdir.clone();
                let tmp = workdir
                    .parent()
                    .map(|parent| parent.join(format!(".tmp-{name}")));
                std::iter::once(workdir).chain(tmp)
            })
            .collect()
    }

    /// A container's private `/tmp`, which lives as long as the container.
    ///
    /// A sandbox gives each spawned command a fresh `/tmp`, and a fresh one
    /// per COMMAND breaks what a container means here: state accumulates
    /// across execs in files, and `/tmp` is files. A document that writes a
    /// scratch file in one cell and reads it in the next works unsandboxed
    /// and mysteriously does not when confined — the cell is not wrong, the
    /// sandbox is.
    ///
    /// So it is a real directory beside the workdir rather than a tmpfs: one
    /// per container, shared by that container's cells, invisible to every
    /// other container, and removed with everything else when the run ends.
    pub fn tmpdir_of(&self, container: &str) -> Result<PathBuf> {
        let workdir = self.workdir_for(container)?;
        let parent = workdir
            .parent()
            .ok_or_else(|| anyhow::anyhow!("container '{container}' has no base directory"))?;
        // Beside the workdir, not inside it: a cell's `/tmp` scratch files
        // are not part of the files the cell produced, and putting them in
        // the workdir would mix the two.
        let tmp = parent.join(format!(".tmp-{container}"));
        std::fs::create_dir_all(&tmp)
            .with_context(|| format!("creating the tmp directory for container '{container}'"))?;
        Ok(tmp)
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
    /// Spawn a command purely for its exit status, recording nothing.
    ///
    /// Deliberately not `run_command_as` with a flag: every path through that
    /// function appends to the transcript, and a "sometimes" flag on it is
    /// one refactor away from a probe showing up in somebody's document.
    pub async fn probe_command(&self, container: &str, command: &str) -> Result<bool> {
        self.probe_launch(container, Launch::Shell(command)).await
    }

    /// [`probe_command`](Self::probe_command) for a program and its arguments.
    pub async fn probe_argv(
        &self,
        container: &str,
        program: &str,
        args: &[String],
    ) -> Result<bool> {
        self.probe_launch(container, Launch::Argv(program, args))
            .await
    }

    async fn probe_launch(&self, container: &str, launch: Launch<'_>) -> Result<bool> {
        let workdir = self.workdir_for(container)?;
        let status = Self::command_for(launch, &workdir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .with_context(|| format!("probing container '{container}'"))?;
        Ok(status.success())
    }

    pub async fn execute_as(
        &self,
        container: &str,
        display: &str,
        command: &str,
        stdin_data: Option<&str>,
        options: ExecOptions,
    ) -> Result<String> {
        self.run_command_as(
            container,
            display,
            Launch::Shell(command),
            stdin_data,
            options,
        )
        .await
    }

    /// Run a program directly, with no shell between us and it, while the
    /// transcript still shows `display` — the cell as its author wrote it.
    ///
    /// This is what a confined cell uses. A sandbox wrapper already carries
    /// the shell it wants inside its own argv (`bwrap … sh -c …`,
    /// `hick __sandbox-run … -- …`), so putting another shell in front of it
    /// only creates a quoting problem: three defects on Windows in one day
    /// came from that outer `cmd.exe`, including a cell whose `&&` was split
    /// by it and whose second half then ran UNCONFINED. An argv has no
    /// quoting to get wrong.
    pub async fn execute_argv_as(
        &self,
        container: &str,
        display: &str,
        program: &str,
        args: &[String],
        stdin_data: Option<&str>,
        options: ExecOptions,
    ) -> Result<String> {
        self.run_command_as(
            container,
            display,
            Launch::Argv(program, args),
            stdin_data,
            options,
        )
        .await
    }

    async fn run_command(
        &self,
        container: &str,
        command: &str,
        stdin_data: Option<&str>,
        options: ExecOptions,
    ) -> Result<String> {
        self.run_command_as(
            container,
            command,
            Launch::Shell(command),
            stdin_data,
            options,
        )
        .await
    }

    /// Build the process for a launch, in the workdir it runs in.
    fn command_for(launch: Launch<'_>, workdir: &std::path::Path) -> tokio::process::Command {
        match launch {
            Launch::Shell(command) => Self::shell_command(command, workdir),
            Launch::Argv(program, args) => {
                let mut cmd = tokio::process::Command::new(program);
                cmd.args(args);
                cmd.current_dir(workdir);
                cmd
            }
        }
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

    /// The platform shell, ready to run `command`.
    ///
    /// The Windows half is why this exists. `cmd.exe /C` takes the rest of the
    /// command line **as text**, but `Command::arg` quotes what it is given for
    /// `CommandLineToArgvW` — so a command that already carries quotes gets
    /// them escaped again, and `"C:\path\hick.exe" __sandbox-run …` reaches cmd
    /// as `\"C:\path\hick.exe\" …`, which it reports as
    /// `'\"C:\path\hick.exe\"' is not recognized as an internal or external
    /// command`. Every confined cell failed that way — the sandbox could not
    /// launch anything at all, which is worse than not confining, because the
    /// document simply does not run.
    ///
    /// `raw_arg` hands the string over untouched, which is what a shell taking
    /// a command line needs. `sh -c` already works that way, so only Windows
    /// changes.
    fn shell_command(command: &str, workdir: &std::path::Path) -> tokio::process::Command {
        let (shell, shell_flag) = Self::shell();
        let mut cmd = tokio::process::Command::new(shell);
        cmd.arg(shell_flag);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as _;
            // Handed over untouched. This once wrapped the line in another
            // pair of quotes, to survive cmd's documented habit of stripping
            // the leading quote and the last one — but that was for the
            // CONFINED line, which quoted a launcher, a workdir and a command
            // and so arrived with six. Confined commands are an argv now and
            // never come through here, so the only thing left is a cell's own
            // shell text, which the author wrote to be handed to a shell as
            // it stands.
            //
            // Wrapping it was actively wrong, measured on Windows 11: a cell
            // reading `echo one` produced NO output and no error, and a run
            // whose only cell was `exit 1` reported success. A tool that runs
            // nothing and says it worked is worse than one that fails.
            cmd.as_std_mut().raw_arg(command);
        }
        #[cfg(not(windows))]
        {
            cmd.arg(command);
        }
        cmd.current_dir(workdir);
        cmd
    }

    /// Kill a spawned command hard, group and all where the platform allows.
    ///
    /// On Unix the child was spawned as the leader of its own process group
    /// (`process_group(0)`), so `SIGKILL` to `-pid` takes down the shell AND
    /// everything it spawned — a cell's background children do not outlive
    /// the cell. On Windows only the direct child (`cmd.exe`) is killed;
    /// grandchildren it spawned may survive. `kill_on_drop(true)` on the
    /// spawn is the additional guarantee that a cancelled future never
    /// leaves the direct child running.
    async fn kill_hard(child: &mut tokio::process::Child, pid: Option<u32>) {
        #[cfg(unix)]
        if let Some(pid) = pid {
            // SAFETY: plain syscall; a stale pid is at worst an ESRCH error.
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
        #[cfg(not(unix))]
        let _ = pid;
        let _ = child.start_kill();
        // Reap, so the kill is observed and no zombie is left behind.
        let _ = child.wait().await;
    }

    async fn run_command_as(
        &self,
        container: &str,
        display: &str,
        launch: Launch<'_>,
        stdin_data: Option<&str>,
        options: ExecOptions,
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
        let mut cmd = Self::command_for(launch, &workdir);
        cmd.stdin(if stdin_data.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Belt and braces for the timeout path: if this future is ever
        // dropped instead of driven to the explicit kill, the direct
        // child still dies with it.
        .kill_on_drop(true);
        // Its own process group, so a timeout can kill the shell AND
        // whatever the shell spawned (see `kill_hard`).
        #[cfg(unix)]
        cmd.process_group(0);
        let mut child = cmd.spawn().with_context(|| match launch {
            Launch::Shell(_) => {
                let (shell, shell_flag) = Self::shell();
                format!("failed to spawn `{shell} {shell_flag}` in container '{container}'")
            }
            // Naming the program is the whole point here: this is the confined
            // path, where a failure to spawn means the sandbox launcher itself
            // could not be started.
            Launch::Argv(program, _) => {
                format!("failed to spawn `{program}` in container '{container}'")
            }
        })?;
        let child_pid = child.id();

        // The stdin write happens inside the timed section below: a command
        // that never reads its stdin leaves the pipe full and the write
        // blocked, which is exactly the hang the timeout exists to cut.
        let mut stdin_pipe = stdin_data.map(|data| {
            (
                child.stdin.take().expect("stdin was piped"),
                data.to_string(),
            )
        });

        let mut stdout = child.stdout.take().expect("stdout was piped");
        let mut stderr = child.stderr.take().expect("stderr was piped");

        // Stream both pipes, timestamping each chunk as it arrives.
        let epoch = {
            let mut state = self.state.lock().unwrap();
            let _ = Self::now_offset_ms(&mut state);
            state.epoch.expect("epoch set")
        };
        // Both streams are normalised as they are read (`CapturedStream`), so
        // a `\r\n` split across two 8 KiB reads still becomes one `\n` in the
        // events. The raw bytes are kept as well, because the aggregate is
        // decoded from the whole buffer rather than assembled from the
        // per-chunk decodes: a multi-byte character straddling a read would
        // survive there and be mangled here, and the aggregate is what an
        // expectation is compared against.
        let out_task = async {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            let mut evs = Vec::new();
            let mut stream = CapturedStream::new();
            loop {
                match stdout.read(&mut chunk).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let t = epoch.elapsed().as_millis() as u64;
                        let data = stream.push(&chunk[..n]);
                        if !data.is_empty() {
                            evs.push(TranscriptEvent::Out { t, data });
                        }
                        buf.extend_from_slice(&chunk[..n]);
                    }
                    Err(e) => return Err(anyhow::Error::from(e)),
                }
            }
            let tail = stream.finish();
            if !tail.is_empty() {
                let t = epoch.elapsed().as_millis() as u64;
                evs.push(TranscriptEvent::Out { t, data: tail });
            }
            Ok::<_, anyhow::Error>((buf, evs))
        };
        let err_task = async {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            let mut evs = Vec::new();
            let mut stream = CapturedStream::new();
            loop {
                match stderr.read(&mut chunk).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let t = epoch.elapsed().as_millis() as u64;
                        let data = stream.push(&chunk[..n]);
                        if !data.is_empty() {
                            evs.push(TranscriptEvent::Err { t, data });
                        }
                        buf.extend_from_slice(&chunk[..n]);
                    }
                    Err(e) => return Err(anyhow::Error::from(e)),
                }
            }
            let tail = stream.finish();
            if !tail.is_empty() {
                let t = epoch.elapsed().as_millis() as u64;
                evs.push(TranscriptEvent::Err { t, data: tail });
            }
            Ok::<_, anyhow::Error>((buf, evs))
        };
        let stdin_task = async {
            if let Some((mut stdin, data)) = stdin_pipe.take() {
                stdin
                    .write_all(data.as_bytes())
                    .await
                    .context("failed to write stdin")?;
                drop(stdin);
            }
            Ok::<_, anyhow::Error>(())
        };

        // Everything that can block on the child — stdin, both output pipes,
        // and the exit itself — inside one future, so a single timeout
        // covers every way a cell can hang.
        let run_to_completion = async {
            let (io, _) =
                tokio::try_join!(async { tokio::try_join!(out_task, err_task) }, stdin_task)?;
            let status = child.wait().await.context("failed to wait for command")?;
            Ok::<_, anyhow::Error>((io, status))
        };

        let waited = match options.timeout {
            Some(limit) => tokio::time::timeout(limit, run_to_completion).await.ok(),
            None => Some(run_to_completion.await),
        };
        let Some(completed) = waited else {
            // Timed out. Kill for real — the await being abandoned is not
            // the same thing as the process being dead — then record what
            // happened in the transcript and fail the cell.
            let limit = options.timeout.expect("timeout was set");
            Self::kill_hard(&mut child, child_pid).await;
            let t = epoch.elapsed().as_millis() as u64;
            events.push(TranscriptEvent::Err {
                t,
                data: format!("timed out after {limit:?}; process killed\n"),
            });
            events.push(TranscriptEvent::Exit { t, code: -1 });
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
                        output: String::new(),
                        events,
                        source_line: None,
                    });
            }
            // `display`, not `command`, for the same reason as the failure
            // path below: the reader wrote the cell, not the sandbox wrapper.
            bail!(
                "cell timed out in container '{container}' after {limit:?}: {}\n  \
                 The command exceeded the per-cell time limit and was killed. A cell \
                 that waits for input it will never get — reading stdin interactively, \
                 listening on a socket — hits this limit no matter how high it is set.\n  \
                 Next steps: raise the limit for this one cell with timeout=\"<seconds>\" \
                 on its <hick:exec> tag, or declare timeout=\"0\" to let it run \
                 unbounded; set HICKORY_CELL_TIMEOUT=<seconds> to change the default \
                 for every cell (default: 120 seconds).",
                display.trim().lines().next().unwrap_or("?")
            );
        };
        let (((out_buf, out_evs), (err_buf, err_evs)), status) = completed?;
        let code = status.code().unwrap_or(-1);
        let exit_t = epoch.elapsed().as_millis() as u64;

        // Merge stdout/stderr events by timestamp (stable within a stream).
        let mut merged: Vec<TranscriptEvent> = out_evs.into_iter().chain(err_evs).collect();
        merged.sort_by_key(|e| e.t_offset_ms());
        events.extend(merged);
        events.push(TranscriptEvent::Exit { t: exit_t, code });

        let output = normalize_captured_newlines(&String::from_utf8_lossy(&out_buf)).into_owned();
        let stderr_text =
            normalize_captured_newlines(&String::from_utf8_lossy(&err_buf)).into_owned();

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
        self.run_command(container, command, None, ExecOptions::default())
            .await
    }

    async fn probe_program(&self, container: &str, bin: &str) -> Result<bool> {
        // No subprocess: this is a filesystem question and the answer is on
        // the filesystem. The cell's own workdir is the cwd because on Windows
        // `cmd` resolves a bare name against it before PATH.
        let workdir = self.workdir_for(container)?;
        Ok(crate::find_program(bin, &workdir).is_some())
    }

    async fn execute_with_stdin(
        &self,
        container: &str,
        command: &str,
        stdin_data: &str,
    ) -> Result<String> {
        self.run_command(container, command, Some(stdin_data), ExecOptions::default())
            .await
    }

    async fn execute_with_options(
        &self,
        container: &str,
        command: &str,
        stdin_data: Option<&str>,
        options: ExecOptions,
    ) -> Result<String> {
        self.run_command(container, command, stdin_data, options)
            .await
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

    /// Protects
    /// `docs/guarantees/verification/an-expectation-means-the-same-on-every-platform.md`.
    ///
    /// Written in each platform's own shell on purpose. On Windows this is
    /// the real case — cmd's `echo` emits CRLF and cannot be told not to — and
    /// on Unix the `printf` spells out the same bytes, so the rule is checked
    /// where CI can see it rather than only on the one runner that has cmd.
    #[tokio::test(flavor = "multi_thread")]
    async fn captured_output_is_recorded_with_lf_line_endings() {
        let cell = if cfg!(windows) {
            "echo one& echo two"
        } else {
            "printf 'one\\r\\ntwo\\r\\n'"
        };
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c1", "alpine:3.20").await.unwrap();
        let out = ex.execute("c1", cell).await.unwrap();
        assert_eq!(out, "one\ntwo\n", "the returned stdout still carries CRLF");

        let ts = ex.transcripts();
        let entry = &ts["c1"][0];
        assert_eq!(
            entry.output, "one\ntwo\n",
            "the transcript entry still carries CRLF"
        );
        for event in &entry.events {
            if let TranscriptEvent::Out { data, .. } | TranscriptEvent::Err { data, .. } = event {
                assert!(
                    !data.contains("\r\n"),
                    "an event still carries CRLF: {data:?}"
                );
            }
        }
    }

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

    // The four tests below protect
    // docs/guarantees/execution/a-cell-cannot-hang-a-run.md — the
    // kill-on-timeout half of it.

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    async fn a_cell_that_outlives_its_timeout_fails_fast_with_next_steps() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c", "alpine").await.unwrap();
        let started = Instant::now();
        let err = ex
            .execute_with_options(
                "c",
                "sleep 30",
                None,
                ExecOptions {
                    timeout: Some(Duration::from_millis(300)),
                },
            )
            .await
            .unwrap_err();
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the failure must arrive near the limit, not after the command"
        );
        let msg = err.to_string();
        assert!(msg.contains("timed out"), "{msg}");
        assert!(msg.contains("'c'"), "must name the container: {msg}");
        assert!(msg.contains("sleep 30"), "must name the command: {msg}");
        assert!(msg.contains("timeout=\"<seconds>\""), "{msg}");
        assert!(msg.contains("HICKORY_CELL_TIMEOUT"), "{msg}");
        // The transcript still recorded the attempt, with an exit event.
        let ts = ex.transcripts();
        assert!(
            ts["c"][0]
                .events
                .iter()
                .any(|e| matches!(e, TranscriptEvent::Exit { code: -1, .. }))
        );
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    async fn a_timed_out_cell_leaves_no_process_behind() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c", "alpine").await.unwrap();
        // A background grandchild that would write a marker AFTER the
        // timeout fires. If the process GROUP dies, the marker never
        // appears; if only the shell dies, the orphan survives to write it.
        let err = ex
            .execute_with_options(
                "c",
                "( sleep 1; echo leaked > leaked.txt ) & sleep 30",
                None,
                ExecOptions {
                    timeout: Some(Duration::from_millis(300)),
                },
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("timed out"), "{err}");
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let workdir = ex.container_workdir("c").unwrap();
        assert!(
            !workdir.join("leaked.txt").exists(),
            "a background child of the cell survived the kill"
        );
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    async fn an_unbounded_cell_still_finishes() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c", "alpine").await.unwrap();
        // timeout: None is what `timeout="0"` resolves to — no limit at all.
        let out = ex
            .execute_with_options(
                "c",
                "sleep 0.2; echo done",
                None,
                ExecOptions { timeout: None },
            )
            .await
            .unwrap();
        assert_eq!(out, "done\n");
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    async fn a_blocked_stdin_write_is_covered_by_the_timeout() {
        let ex = LocalExecutor::new().unwrap();
        ex.ensure_started("c", "alpine").await.unwrap();
        // The command never reads stdin, so a large payload blocks the
        // writer once the pipe buffer fills — a hang that must ALSO be cut.
        let big = "x".repeat(4 * 1024 * 1024);
        let started = Instant::now();
        let err = ex
            .execute_with_options(
                "c",
                "sleep 30",
                Some(&big),
                ExecOptions {
                    timeout: Some(Duration::from_millis(300)),
                },
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("timed out"), "{err}");
        assert!(started.elapsed() < Duration::from_secs(5));
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
