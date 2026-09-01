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

/// The failure returned when a command exceeded [`ExecOptions::timeout`] and
/// was killed.
///
/// A typed error rather than a string, because two callers have to tell "the
/// limit fired" apart from "the command failed", and one of them is not a
/// human: the agent turns a timeout into an observation that says the script
/// was killed, and matching on the wording of a message to do that is a bug
/// waiting for the wording to change. The [`Display`](std::fmt::Display) text
/// is the user-facing message and is what `anyhow` prints unchanged.
#[derive(Debug, Clone)]
pub struct ExecTimedOut {
    /// Container the command was running in.
    pub container: String,
    /// The limit that was exceeded.
    pub limit: Duration,
    /// The full user-facing message, next steps included.
    message: String,
}

impl std::fmt::Display for ExecTimedOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ExecTimedOut {}

// ---------------------------------------------------------------------------
// Script platform
// ---------------------------------------------------------------------------

/// The command language of the container a script will be written into and
/// run by.
///
/// This exists because "what shell is there" is a property of the **executor**,
/// not of the machine the process is running on. `LocalExecutor` runs a cell
/// through `cmd.exe /C` on Windows on purpose (see [`LocalExecutor::shell`]),
/// while the Docker and Canopy executors run it inside a Linux container even
/// when the host is Windows. Anything that composes a script — the agent, the
/// session replay — has to ask the executor rather than ask `cfg!(windows)`,
/// or it is right on one of those two and wrong on the other.
///
/// Every form below was measured against a real `cmd.exe` on Windows 11
/// (2026-08-20) rather than assumed; the specific traps are named at each
/// method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptPlatform {
    /// A POSIX shell (`sh`) and `python3`.
    Posix,
    /// Windows `cmd.exe` and `python`.
    WindowsCmd,
}

impl ScriptPlatform {
    /// The platform of the machine this process is running on.
    pub const fn host() -> Self {
        if cfg!(windows) {
            ScriptPlatform::WindowsCmd
        } else {
            ScriptPlatform::Posix
        }
    }

    /// Extension for a shell script file.
    ///
    /// `.cmd` rather than `.bat` on Windows: both are batch files, and `.cmd`
    /// is the one whose `errorlevel` handling has not been kept
    /// backwards-compatible with COMMAND.COM.
    pub fn shell_script_extension(&self) -> &'static str {
        match self {
            ScriptPlatform::Posix => "sh",
            ScriptPlatform::WindowsCmd => "cmd",
        }
    }

    /// Text prepended to a shell script before it is written.
    ///
    /// `@echo off` is not decoration. A batch file runs with command echo ON,
    /// so `cmd.exe` prints every line of the script to stdout before running
    /// it — the captured output of a five-line script would be the script
    /// itself interleaved with its results, and anything comparing that output
    /// to an expectation (or handing it to a model as an observation) reads
    /// our own file back. `sh` echoes nothing, so POSIX needs no preamble.
    pub fn shell_script_preamble(&self) -> &'static str {
        match self {
            ScriptPlatform::Posix => "",
            ScriptPlatform::WindowsCmd => "@echo off\r\n",
        }
    }

    /// The command that runs a shell script at `script_path`.
    ///
    /// On Windows the batch file is named on its own: the command is already
    /// being handed to `cmd.exe /C`, and a batch file is something cmd runs
    /// directly, propagating its last `errorlevel`. There is no `sh` to invoke
    /// and no reason to invent one.
    pub fn shell_script_command(&self, script_path: &str) -> String {
        match self {
            ScriptPlatform::Posix => format!("sh {script_path}"),
            ScriptPlatform::WindowsCmd => script_path.to_string(),
        }
    }

    /// The command that runs a Python script at `script_path`.
    ///
    /// `python3` does not exist on a stock Windows install — python.org's
    /// installer puts `python.exe` on PATH, and the name `python3` there is
    /// usually the Microsoft Store app-execution alias, which opens the Store
    /// instead of running anything.
    pub fn python_script_command(&self, script_path: &str) -> String {
        match self {
            ScriptPlatform::Posix => format!("python3 {script_path}"),
            ScriptPlatform::WindowsCmd => format!("python {script_path}"),
        }
    }

    /// Join a directory and a file name with this platform's separator.
    ///
    /// Backslashes on Windows, and not for tidiness: a forward slash works for
    /// redirection but `cmd` reads a leading `/` as the start of a switch, so
    /// `.hickory-agent/action-0.cmd` is not a path it will run.
    pub fn join_path(&self, dir: &str, file: &str) -> String {
        match self {
            ScriptPlatform::Posix => format!("{dir}/{file}"),
            ScriptPlatform::WindowsCmd => format!("{dir}\\{file}"),
        }
    }

    /// One line naming this shell, for a prompt or an error.
    pub fn describe(&self) -> &'static str {
        match self {
            ScriptPlatform::Posix => "POSIX sh",
            ScriptPlatform::WindowsCmd => "Windows cmd.exe",
        }
    }
}

/// The host's shell, ready to run `command`, as a plain `std` command.
///
/// One place knows how a command line reaches a shell on this machine, because
/// getting it wrong is invisible: on Windows `Command::arg` quotes what it is
/// given for `CommandLineToArgvW`, and `cmd.exe /C` wants the rest of the line
/// as text, so an argument that already carries quotes arrives re-escaped and
/// cmd reports a command nobody wrote. [`LocalExecutor`] builds its async
/// command from this, and so does anything outside the executor that has to
/// run a line of a user's shell (the token-economics harness's check command).
pub fn host_shell_command(command: &str) -> std::process::Command {
    let (shell, flag) = LocalExecutor::shell();
    let mut cmd = std::process::Command::new(shell);
    cmd.arg(flag);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        cmd.raw_arg(command);
    }
    #[cfg(not(windows))]
    {
        cmd.arg(command);
    }
    cmd
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

    /// The command language of this executor's containers.
    ///
    /// Default [`ScriptPlatform::Posix`]: a container is Linux unless the
    /// executor says otherwise, which is true of Docker and Canopy on every
    /// host. `LocalExecutor` — whose "container" is the host — overrides it.
    fn script_platform(&self) -> ScriptPlatform {
        ScriptPlatform::Posix
    }

    /// Write `contents` to `path` (relative to the container workdir),
    /// creating parent directories.
    ///
    /// This is on the trait because writing a file is not a shell operation
    /// and should never have been composed as one. The agent used to write
    /// its scripts with `mkdir -p … && cat > …` piped through
    /// [`Executor::execute_with_stdin`], which is POSIX text handed to
    /// whatever shell the executor resolves to — on Windows, `cmd.exe`, where
    /// `mkdir -p` creates a directory called `-p` and `cat` is not a command.
    /// An executor that can put bytes in a file needs no shell at all to do
    /// it, and one dialect fewer is one fewer to keep correct.
    ///
    /// The default implementation is the POSIX composition, for the executors
    /// whose containers really are Linux; it is the only place that text now
    /// exists. Any executor whose [`Executor::script_platform`] is not
    /// [`ScriptPlatform::Posix`] MUST override this.
    ///
    /// This is plumbing and does not belong in a woven document, so an
    /// override should record nothing — the default records only because
    /// piping through `execute_with_stdin` is the only tool it has.
    async fn write_file(&self, container: &str, path: &str, contents: &str) -> Result<()> {
        let dir = path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or(".");
        self.execute_with_stdin(
            container,
            &format!("mkdir -p {dir} && cat > {path}"),
            contents,
        )
        .await?;
        Ok(())
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

/// Where a run's container workdirs live.
///
/// Stable when this run could claim the name for its project, ephemeral when
/// it could not. See [`LocalExecutor::new`] for why the stable one matters.
enum ScratchRoot {
    /// A path derived from the project, held under a lock file and removed
    /// when the run ends.
    Stable { path: PathBuf, lock: PathBuf },
    /// A fresh random directory, removed on drop by `tempfile`.
    Ephemeral(tempfile::TempDir),
}

impl ScratchRoot {
    fn path(&self) -> &Path {
        match self {
            ScratchRoot::Stable { path, .. } => path,
            ScratchRoot::Ephemeral(dir) => dir.path(),
        }
    }
}

impl Drop for ScratchRoot {
    fn drop(&mut self) {
        if let ScratchRoot::Stable { path, lock } = self {
            let _ = std::fs::remove_dir_all(path);
            let _ = std::fs::remove_file(lock);
        }
    }
}

pub struct LocalExecutor {
    /// Root dir holding one workdir per container; cleaned on drop.
    root: ScratchRoot,
    state: Mutex<LocalState>,
}

impl LocalExecutor {
    /// Create an executor whose container workdirs live under a scratch
    /// directory, removed when the executor is dropped.
    ///
    /// **The name is derived, not random**, and that is the point. A cell's
    /// workdir path appears in its own output whenever the program it runs
    /// prints a path — `dotnet restore` names the project file it restored,
    /// compilers name their inputs, half of everything names its cwd on
    /// error. With a random name, that output differed on every run, so
    /// `hick test` reported drift forever on a document that had not changed.
    /// A product whose claim is that a document reproduces its outputs byte
    /// for byte cannot have its own temp directory be the thing that stops it.
    ///
    /// Derived from the working directory, which is the project for a CLI run
    /// and stable per-process for the app, so two runs of the same document
    /// in the same place agree.
    ///
    /// **Two runs at once do not share it.** A lock file names the process
    /// holding the directory; a second run that finds a live holder takes a
    /// random directory instead and says so. That run is not reproducible,
    /// which is the honest outcome — but it cannot quietly write into another
    /// run's workdirs, which would be worse than either.
    pub fn new() -> Result<Self> {
        Ok(Self {
            root: ScratchRoot::Ephemeral(
                tempfile::Builder::new()
                    .prefix("hickory-local-")
                    .tempdir()
                    .context("failed to create LocalExecutor temp root")?,
            ),
            state: Mutex::new(LocalState::default()),
        })
    }

    /// Like [`new`](Self::new), but with the derived scratch directory
    /// described above, named after `project`.
    ///
    /// **`project` is the directory the work belongs to** — for `hick run
    /// <path>` that is the document's own directory, not wherever the person
    /// was standing when they typed it. Those two are the same only when the
    /// command is run from the project, and keying on the second is a bug in
    /// both directions: two unrelated documents run from one shell share a
    /// name and fight over it, while one document run from two different
    /// shells gets two, which defeats the reproducibility this exists for.
    ///
    /// Passing `None` falls back to the process's working directory. It is
    /// for a caller that genuinely has no document in hand — `hick up`
    /// serving a folder, the MCP server — where the cwd IS the project.
    ///
    /// Opt-in rather than the default because the stable name is shared by
    /// everything running against one project, and only one holder can have
    /// it at a time. That is exactly right for `hick`, which builds one
    /// executor per process — and wrong for a test binary or an embedder that
    /// builds several at once, where each wants its own scratch and none
    /// wants a path a sibling might remove.
    pub fn new_stable_for(project: Option<&Path>) -> Result<Self> {
        Ok(Self {
            root: Self::scratch_root(project)?,
            state: Mutex::new(LocalState::default()),
        })
    }

    /// [`new_stable_for`](Self::new_stable_for) with no project, so the name
    /// comes from the working directory.
    pub fn new_stable() -> Result<Self> {
        Self::new_stable_for(None)
    }

    fn scratch_root(project: Option<&Path>) -> Result<ScratchRoot> {
        // Once per process, before this run takes a lock of its own: a run
        // that crashed leaves its directory behind, and nothing else ever
        // removes it.
        static SWEEP: std::sync::Once = std::sync::Once::new();
        SWEEP.call_once(|| sweep_stale_roots(&std::env::temp_dir()));

        let ephemeral = || -> Result<ScratchRoot> {
            Ok(ScratchRoot::Ephemeral(
                tempfile::Builder::new()
                    .prefix("hickory-local-")
                    .tempdir()
                    .context("failed to create LocalExecutor temp root")?,
            ))
        };
        // Canonicalised so `.`, `../proj` and an absolute path all name one
        // root. A path that cannot be canonicalised is used as given rather
        // than rejected: a worse name is still better than the wrong one.
        //
        // The EMPTY path is the exception, and it was a real bug rather than a
        // theoretical one. `Path::new("d.hick").parent()` is `Some("")`, not
        // `None`, so a caller writing `parent().unwrap_or(".")` handed an
        // empty path straight through — and `short_key("")` mixes in no bytes
        // and returns the FNV basis, the same twelve characters every time. So
        // every `hick run <bare-filename>` on the machine, in any project,
        // shared ONE scratch root: they raced for the lock, and a run that
        // took over a lock whose holder looked gone wiped a live run's
        // workdir, which surfaced as `failed to spawn sh -c: No such file or
        // directory`. Falling back to the working directory gives each project
        // its own key again.
        let key = match project.filter(|dir| !dir.as_os_str().is_empty()) {
            Some(dir) => std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf()),
            None => match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(_) => return ephemeral(),
            },
        };
        if key.as_os_str().is_empty() {
            return ephemeral();
        }
        let path = std::env::temp_dir().join(format!("hickory-local-{}", short_key(&key)));
        let lock = path.with_extension("lock");

        // Take the lock, or take over one whose process is gone: a run killed
        // with SIGKILL never removes its own, and a scratch directory nobody
        // can ever use again would be a worse bug than the one this fixes.
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(mut file) => {
                use std::io::Write as _;
                let _ = writeln!(file, "{}", std::process::id());
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if holder_is_alive(&lock) {
                    log::info!(
                        "another run holds {}; this one uses a temporary directory instead, \
                         so paths in its output will not match a previous run's",
                        path.display()
                    );
                    return ephemeral();
                }
                let _ = std::fs::write(&lock, format!("{}\n", std::process::id()));
            }
            Err(_) => return ephemeral(),
        }

        // A previous run's leftovers must not be visible to this one.
        let _ = std::fs::remove_dir_all(&path);
        if std::fs::create_dir_all(&path).is_err() {
            let _ = std::fs::remove_file(&lock);
            return ephemeral();
        }
        Ok(ScratchRoot::Stable { path, lock })
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

    /// The platform shell in `workdir`, ready to run `command`.
    ///
    /// The command line itself is composed by [`host_shell_command`], which is
    /// where the Windows quoting rule lives; this only adds the working
    /// directory and converts to the async command type.
    fn shell_command(command: &str, workdir: &std::path::Path) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::from(host_shell_command(command));
        cmd.current_dir(workdir);
        cmd
    }

    /// Kill a spawned command hard, and its children with it.
    ///
    /// On Unix the child was spawned as the leader of its own process group
    /// (`process_group(0)`), so `SIGKILL` to `-pid` takes down the shell AND
    /// everything it spawned — a cell's background children do not outlive
    /// the cell.
    ///
    /// Windows has no process groups to signal, and killing the direct child
    /// alone is not enough: `cmd.exe /C script.cmd` is a shell that spawns the
    /// programs in the script, so a timeout that kills only cmd leaves the
    /// `python` (or the `ping`, or the compiler) it started running with no
    /// parent, still holding the workdir the run is about to be judged on.
    /// `taskkill /T /F` walks the parent-pid tree from the child down and
    /// kills all of it, which is the same promise the process-group signal
    /// makes on Unix. It ships in `System32` on every Windows edition this
    /// product targets; if it is somehow missing, the direct child is still
    /// killed below and the cell still fails — one dead process short of the
    /// guarantee, not a hang.
    ///
    /// It has to run BEFORE `start_kill`: `taskkill` resolves the tree by
    /// asking the OS who the parents are, and once cmd.exe is dead its
    /// children have been reparented and are no longer reachable from this
    /// pid.
    ///
    /// `kill_on_drop(true)` on the spawn is the additional guarantee that a
    /// cancelled future never leaves the direct child running.
    async fn kill_hard(child: &mut tokio::process::Child, pid: Option<u32>) {
        #[cfg(unix)]
        if let Some(pid) = pid {
            // SAFETY: plain syscall; a stale pid is at worst an ESRCH error.
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
        #[cfg(windows)]
        if let Some(pid) = pid {
            let _ = tokio::process::Command::new("taskkill")
                .args(["/T", "/F", "/PID", &pid.to_string()])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
        }
        #[cfg(not(any(unix, windows)))]
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
            return Err(anyhow::Error::new(ExecTimedOut {
                container: container.to_string(),
                limit,
                message: format!(
                    "cell timed out in container '{container}' after {limit:?}: {}\n  \
                     The command exceeded the per-cell time limit and was killed. A cell \
                     that waits for input it will never get — reading stdin interactively, \
                     listening on a socket — hits this limit no matter how high it is set.\n  \
                     Next steps: raise the limit for this one cell with timeout=\"<seconds>\" \
                     on its <hick:exec> tag, or declare timeout=\"0\" to let it run \
                     unbounded; set HICKORY_CELL_TIMEOUT=<seconds> to change the default \
                     for every cell (default: 120 seconds).",
                    display.trim().lines().next().unwrap_or("?")
                ),
            }));
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

    /// The host's, because this executor's "container" is the host: the same
    /// answer [`LocalExecutor::shell`] gives, in the form a script writer
    /// needs.
    fn script_platform(&self) -> ScriptPlatform {
        ScriptPlatform::host()
    }

    /// A plain filesystem write, with no shell anywhere near it — the whole
    /// reason this method is on the trait.
    ///
    /// `path` is relative to the container workdir and may not climb out of
    /// it. A caller that hands over `../../etc/hosts` gets an error rather
    /// than a write: containers are the only boundary this executor has, and
    /// a helper that quietly writes outside one would remove it.
    async fn write_file(&self, container: &str, path: &str, contents: &str) -> Result<()> {
        let workdir = self.workdir_for(container)?;
        let relative = std::path::Path::new(path);
        // NOT `is_absolute()`. On Windows that is FALSE for `/etc/hosts` --
        // rooted, but naming no drive, which Windows calls relative -- so the
        // guard would let it through and `join` would resolve it to
        // `C:\etc\hosts`, outside the container entirely. The same mistake
        // let HICKORY_INBOX escape a notes folder (#22); scanning components
        // for a root or a drive prefix catches every spelling.
        if relative.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) {
            bail!(
                "refusing to write '{path}' in container '{container}': a script path must be \
                 relative to the container workdir and must not contain '..'"
            );
        }
        let target = workdir.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("creating {} in container '{container}'", parent.display())
            })?;
        }
        std::fs::write(&target, contents)
            .with_context(|| format!("writing {} in container '{container}'", target.display()))?;
        Ok(())
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

/// A short, stable key for a path: FNV-1a/64 folded to 12 hex characters.
///
/// Not a cryptographic hash and does not need to be — this names a temp
/// directory, and the only property required is that the same project gets
/// the same name on every run of the same machine.
fn short_key(path: &Path) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in path.as_os_str().as_encoded_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    // Masked to 48 bits so the result is EXACTLY 12 hex characters. `{:012x}`
    // pads a short value but does not truncate a long one, and a caller that
    // recognises our directories by their shape needs a fixed width.
    format!("{:012x}", hash & 0xffff_ffff_ffff)
}

/// Whether the process named in a lock file still exists.
///
/// Unix only in the strict sense; elsewhere the lock is treated as live,
/// which costs a run its stable path and never costs it correctness.
/// The project key in a derived scratch-root name, if the name is one of ours.
///
/// [`short_key`] makes exactly 12 hex characters so a derived root can be
/// recognised by shape. That matters because the ephemeral fallback asks
/// `tempfile` for the SAME `hickory-local-` prefix, so the prefix alone does
/// not tell a derived root from a random directory that a live run is using —
/// and anything that deletes by name has to know the difference.
pub fn derived_root_key(name: &str) -> Option<&str> {
    let key = name.strip_prefix("hickory-local-")?;
    (key.len() == 12 && key.chars().all(|c| c.is_ascii_hexdigit())).then_some(key)
}

/// Remove derived scratch roots whose owning run is gone.
///
/// These accumulate: a run that does not exit cleanly never drops its
/// `ScratchRoot`, so the directory and its lock stay behind. 42 were present
/// in the temp directory during one test run, and nothing had ever removed
/// one.
///
/// Only a root whose lock says its holder is gone is touched, which is the
/// same question the takeover path asks and is answered by the same function.
/// Two things are deliberately left alone:
///
/// * A directory with **no lock**, because the ephemeral fallback is a
///   `tempfile` with our own prefix and deleting one would pull the ground
///   out from under a live run.
/// * `hickory-home-*`, the persistent cell homes, which exist precisely to
///   outlive a run and are not leftovers.
fn sweep_stale_roots(temp: &Path) {
    let Ok(entries) = std::fs::read_dir(temp) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        // Walk the LOCKS, not the directories: a lock is the only evidence
        // that a directory was ever claimed, and a lockless one is either in
        // use or not ours to judge.
        let Some(stem) = name.strip_suffix(".lock") else {
            continue;
        };
        if derived_root_key(stem).is_none() {
            continue;
        }
        let lock = entry.path();
        if holder_is_alive(&lock) {
            continue;
        }
        let root = temp.join(stem);
        log::debug!(
            "sweeping scratch root {} of a run that is gone",
            root.display()
        );
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&lock);
    }
}

/// How long a lock with no readable pid is assumed to belong to whoever is
/// still writing it.
///
/// `create_new` makes the file and the pid lands a moment later, so the two
/// are not one operation and a reader can catch the gap.
const UNWRITTEN_LOCK_GRACE: std::time::Duration = std::time::Duration::from_secs(30);

/// Whether a lock is too young to be anything but a half-written one.
///
/// A lock that crashed between the two syscalls is indistinguishable from one
/// mid-write except by age, and guessing "held" forever would wedge a project's
/// scratch directory permanently. Age settles it: microseconds old is the gap,
/// half a minute old is a leftover.
fn lock_was_just_created(lock: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(lock) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        // No mtime to reason about — assume held, because deleting a live
        // run's workdirs is the worse of the two mistakes.
        return true;
    };
    modified
        .elapsed()
        .map(|age| age < UNWRITTEN_LOCK_GRACE)
        .unwrap_or(true)
}

fn holder_is_alive(lock: &Path) -> bool {
    let Ok(body) = std::fs::read_to_string(lock) else {
        return false;
    };
    let Ok(pid) = body.trim().parse::<u32>() else {
        // An EMPTY lock is the dangerous case, and it is the one that bit:
        // reading "no pid" as "nobody owns this" let a second run take over
        // and `remove_dir_all` a directory whose owner was a microsecond from
        // using it. That is exactly how a live run's workdir disappeared and
        // surfaced as `failed to spawn 'sh -c': No such file or directory`.
        return lock_was_just_created(lock);
    };
    if pid == std::process::id() {
        return true;
    }
    #[cfg(unix)]
    {
        Path::new(&format!("/proc/{pid}")).exists()
            || unsafe { libc::kill(pid as i32, 0) } == 0
            || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {

    /// Two projects, two scratch roots — even from one working directory.
    ///
    /// This is the regression for a flake that read as a caching bug. The
    /// root used to be named after the PROCESS's working directory, so every
    /// test binary that spawned `hick` without setting one inherited the
    /// harness's cwd, derived a single name, and contended for its one lock.
    /// The loser silently fell back to a random directory; a process that
    /// exited released the lock while its files were still there, so one
    /// run read another's inputs. Run serially it always passed, and in
    /// parallel `cache_inputs` failed 6 times in 6.
    #[test]
    fn two_projects_do_not_share_a_scratch_root() {
        let a = tempfile::tempdir().expect("a");
        let b = tempfile::tempdir().expect("b");
        let one = LocalExecutor::new_stable_for(Some(a.path())).expect("one");
        let two = LocalExecutor::new_stable_for(Some(b.path())).expect("two");
        assert_ne!(
            one.root.path(),
            two.root.path(),
            "two projects were given one scratch root, so their cells share a workdir"
        );
        // And both really got the derived root, not the ephemeral fallback —
        // otherwise this would pass while proving nothing.
        assert!(matches!(one.root, ScratchRoot::Stable { .. }));
        assert!(matches!(two.root, ScratchRoot::Stable { .. }));
    }

    /// The scratch root and lock a project would derive, without building one.
    fn derived_paths(project: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
        let key = std::fs::canonicalize(project).expect("canonical project");
        let path = std::env::temp_dir().join(format!("hickory-local-{}", super::short_key(&key)));
        let lock = path.with_extension("lock");
        (path, lock)
    }

    /// A pid that is genuinely gone: spawned, waited for, reaped.
    fn a_dead_pid() -> u32 {
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg("exit 0")
            .spawn()
            .expect("spawn a short-lived child");
        let pid = child.id();
        child.wait().expect("reap it");
        pid
    }

    /// A lock whose holder is gone is taken over, and its leftovers removed.
    ///
    /// The path the guarantee's caveat said had no test, because exercising it
    /// looked like it needed a SIGKILL. It does not: what the code actually
    /// reads is a pid in a file, so a reaped child's pid is the same evidence
    /// a killed run would leave.
    #[test]
    fn a_lock_whose_holder_is_gone_is_taken_over() {
        let project = tempfile::tempdir().expect("project");
        let (path, lock) = derived_paths(project.path());
        std::fs::create_dir_all(&path).expect("a previous run's root");
        std::fs::write(path.join("leftover.txt"), "from a run that is gone").expect("leftover");
        std::fs::write(
            &lock,
            format!(
                "{}
",
                a_dead_pid()
            ),
        )
        .expect("stale lock");

        let executor = LocalExecutor::new_stable_for(Some(project.path())).expect("takeover");

        assert!(
            matches!(executor.root, ScratchRoot::Stable { .. }),
            "a dead holder's lock was not taken over"
        );
        assert!(
            !path.join("leftover.txt").exists(),
            "a previous run's files survived the takeover and are visible to this one"
        );
        assert_eq!(
            std::fs::read_to_string(&lock)
                .expect("lock is readable")
                .trim(),
            std::process::id().to_string(),
            "the lock still names the process that is gone"
        );
    }

    /// A lock whose holder is alive is left alone, and so is its directory.
    ///
    /// The safety half. Taking over here is what deletes a running cell's
    /// workdir, so the fallback must be the random directory rather than the
    /// shared one.
    #[test]
    fn a_live_holders_directory_is_not_touched() {
        let project = tempfile::tempdir().expect("project");
        let (path, lock) = derived_paths(project.path());
        std::fs::create_dir_all(&path).expect("the holder's root");
        std::fs::write(path.join("in-use.txt"), "a cell is running here").expect("marker");
        // Our own pid: alive by definition, and `holder_is_alive` says so
        // without having to keep a second process around for the test.
        std::fs::write(
            &lock,
            format!(
                "{}
",
                std::process::id()
            ),
        )
        .expect("live lock");

        let executor = LocalExecutor::new_stable_for(Some(project.path())).expect("fallback");

        assert!(
            matches!(executor.root, ScratchRoot::Ephemeral(_)),
            "a live holder's lock was taken over"
        );
        assert!(
            path.join("in-use.txt").exists(),
            "a live run's scratch directory was deleted out from under it"
        );
        let _ = std::fs::remove_dir_all(&path);
        let _ = std::fs::remove_file(&lock);
    }

    /// A lock that exists but has no pid yet is held, not free.
    ///
    /// The race itself. `create_new` makes the file and the pid is written a
    /// moment later, so a reader can see it empty — and reading that as "nobody
    /// owns this" is what deleted a live run's workdirs. Measured 5 failures in
    /// 25 runs of `needs_preflight` before this, 0 in 30 after.
    #[test]
    fn a_lock_with_no_pid_yet_is_not_free_to_take() {
        let project = tempfile::tempdir().expect("project");
        let (path, lock) = derived_paths(project.path());
        std::fs::create_dir_all(&path).expect("the holder's root");
        std::fs::write(path.join("in-use.txt"), "a cell is about to run here").expect("marker");
        // Exactly what `create_new` leaves behind before `writeln!` lands.
        std::fs::write(&lock, "").expect("unwritten lock");

        let executor = LocalExecutor::new_stable_for(Some(project.path())).expect("fallback");

        assert!(
            matches!(executor.root, ScratchRoot::Ephemeral(_)),
            "a lock caught mid-creation read as unowned"
        );
        assert!(
            path.join("in-use.txt").exists(),
            "a directory whose owner was still writing its lock was deleted"
        );
        let _ = std::fs::remove_dir_all(&path);
        let _ = std::fs::remove_file(&lock);
    }

    /// An empty lock old enough to be a crash leftover IS free to take.
    ///
    /// The other side of the rule above: assuming "held" forever would wedge a
    /// project's scratch directory for good if a run died between creating its
    /// lock and writing its pid.
    #[test]
    fn an_old_lock_with_no_pid_is_a_leftover() {
        let project = tempfile::tempdir().expect("project");
        let (path, lock) = derived_paths(project.path());
        std::fs::create_dir_all(&path).expect("root");
        std::fs::write(path.join("leftover.txt"), "from a run that crashed").expect("leftover");
        std::fs::write(&lock, "").expect("unwritten lock");
        let old = std::time::SystemTime::now() - (super::UNWRITTEN_LOCK_GRACE * 2);
        std::fs::File::options()
            .write(true)
            .open(&lock)
            .expect("reopen lock")
            .set_times(std::fs::FileTimes::new().set_modified(old))
            .expect("age the lock");

        let executor = LocalExecutor::new_stable_for(Some(project.path())).expect("takeover");

        assert!(
            matches!(executor.root, ScratchRoot::Stable { .. }),
            "a crashed run's half-written lock wedged the directory forever"
        );
        assert!(!path.join("leftover.txt").exists());
    }

    /// A crashed run's scratch root is removed, and a live one is not.
    ///
    /// These accumulate — 42 were in the temp directory during one test run —
    /// and nothing had ever removed one. The sweep asks the same question the
    /// takeover path asks, so the two cannot disagree about who is gone.
    #[test]
    fn the_sweep_removes_only_roots_whose_run_is_gone() {
        let temp = tempfile::tempdir().expect("a temp directory of our own");
        let temp = temp.path();

        // A derived root whose owner is gone.
        let dead = temp.join("hickory-local-0123456789ab");
        std::fs::create_dir_all(&dead).expect("dead root");
        std::fs::write(dead.join("leftover.txt"), "x").expect("leftover");
        std::fs::write(
            temp.join("hickory-local-0123456789ab.lock"),
            format!("{}\n", a_dead_pid()),
        )
        .expect("dead lock");

        // A derived root whose owner is running.
        let live = temp.join("hickory-local-ffffffffffff");
        std::fs::create_dir_all(&live).expect("live root");
        std::fs::write(live.join("in-use.txt"), "x").expect("marker");
        std::fs::write(
            temp.join("hickory-local-ffffffffffff.lock"),
            format!("{}\n", std::process::id()),
        )
        .expect("live lock");

        // The ephemeral fallback: OUR prefix, a random suffix, and no lock.
        // Deleting one of these pulls the ground out from under a live run,
        // which is why the sweep walks locks rather than directories.
        let ephemeral = temp.join("hickory-local-Ab3Xy9");
        std::fs::create_dir_all(&ephemeral).expect("ephemeral root");
        std::fs::write(ephemeral.join("in-use.txt"), "x").expect("marker");

        // A persistent cell home, which exists to outlive a run.
        let home = temp.join("hickory-home-0123456789ab");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::write(home.join("kept.txt"), "x").expect("marker");

        // Somebody else's directory entirely.
        let stranger = temp.join("not-ours");
        std::fs::create_dir_all(&stranger).expect("stranger");

        super::sweep_stale_roots(temp);

        assert!(
            !dead.exists(),
            "a crashed run's scratch root was left behind"
        );
        assert!(
            !temp.join("hickory-local-0123456789ab.lock").exists(),
            "the lock outlived the root it named"
        );
        assert!(live.exists(), "a live run's scratch root was swept");
        assert!(
            temp.join("hickory-local-ffffffffffff.lock").exists(),
            "a live run's lock was removed, so the next run will take its root"
        );
        assert!(
            ephemeral.exists(),
            "an ephemeral fallback was swept — it shares our prefix and has no lock"
        );
        assert!(home.exists(), "a persistent cell home was swept");
        assert!(
            stranger.exists(),
            "the sweep reached outside its own naming"
        );
    }

    /// A lock whose root is already gone is removed too.
    ///
    /// `ScratchRoot::drop` removes the directory before the lock, so a crash
    /// between them leaves exactly this. Left alone it is harmless but
    /// permanent, and it makes the temp directory unreadable to a person.
    #[test]
    fn the_sweep_removes_an_orphaned_lock() {
        let temp = tempfile::tempdir().expect("temp");
        let temp = temp.path();
        let lock = temp.join("hickory-local-abcdefabcdef.lock");
        std::fs::write(&lock, format!("{}\n", a_dead_pid())).expect("orphan lock");

        super::sweep_stale_roots(temp);

        assert!(!lock.exists(), "an orphaned lock was left behind");
    }

    /// A lock caught mid-creation is not swept.
    ///
    /// The same gap that made the takeover destructive: an empty lock means a
    /// run is a microsecond from using that directory, and the sweep must read
    /// it the way the takeover now does.
    #[test]
    fn the_sweep_leaves_a_lock_that_has_no_pid_yet() {
        let temp = tempfile::tempdir().expect("temp");
        let temp = temp.path();
        let root = temp.join("hickory-local-000000000001");
        std::fs::create_dir_all(&root).expect("root");
        std::fs::write(temp.join("hickory-local-000000000001.lock"), "").expect("unwritten lock");

        super::sweep_stale_roots(temp);

        assert!(
            root.exists(),
            "a root whose owner was still writing its lock was swept"
        );
    }

    /// A document named without a directory is still a project of its own.
    ///
    /// The same class as the test above, through a different door and found
    /// the same way — a parallel run failing where a serial one passed.
    /// `Path::new("d.hick").parent()` is `Some("")`, not `None`, so a caller
    /// writing `parent().unwrap_or(".")` handed an empty path through; and
    /// `short_key("")` mixes in no bytes, so it returns the FNV basis — the
    /// same twelve characters for every caller. Every `hick run
    /// <bare-filename>` on the machine therefore contended for one scratch
    /// root, and a run that took over a lock whose holder looked gone deleted
    /// a live run's workdir mid-cell.
    #[test]
    fn an_empty_project_path_is_not_everybodys_scratch_root() {
        let shared = format!(
            "hickory-local-{}",
            super::short_key(std::path::Path::new(""))
        );
        let executor =
            LocalExecutor::new_stable_for(Some(std::path::Path::new(""))).expect("empty path");
        let name = executor
            .root
            .path()
            .file_name()
            .expect("a named root")
            .to_string_lossy()
            .to_string();
        assert_ne!(
            name, shared,
            "an empty project path fell through to the basis key every caller shares"
        );
    }

    /// The name follows the project, not the shell it was launched from.
    ///
    /// The reproducibility this exists for is a property of the document: a
    /// cell that prints its own path must print the same one whether the
    /// person typed `hick run doc.hick` from inside the project or
    /// `hick run ../proj/doc.hick` from outside it.
    #[test]
    fn one_project_keeps_its_name_from_anywhere() {
        let project = tempfile::tempdir().expect("project");
        let first = LocalExecutor::new_stable_for(Some(project.path()))
            .expect("first")
            .root
            .path()
            .to_path_buf();
        // Dropped, so the lock is free and the same name is available again.
        let second = LocalExecutor::new_stable_for(Some(project.path()))
            .expect("second")
            .root
            .path()
            .to_path_buf();
        assert_eq!(first, second);
    }

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
