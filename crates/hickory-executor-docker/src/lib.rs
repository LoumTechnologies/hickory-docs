//! Executor backend that runs each hick container as a real Docker container.
//!
//! This is the executor that makes `image=` mean something. `LocalExecutor`
//! records the image and runs against the host toolchain, so a document's
//! reproducibility claim only ever held on the machine that wrote it —
//! `hick test` proved the bytes reproduced with *your* Python, not with
//! `python:3.12`. Here the declared image is the environment, so two machines
//! that agree on the image agree on the result.
//!
//! It also makes isolation real rather than declared. Containers run with no
//! network unless the document's own `<hick:allow network="host:port">` asks
//! for one (see [`Executor::declare_capabilities`]), plus a memory cap and a
//! PID cap regardless; `hick:fork` is a genuine
//! `docker commit` of the source container, so a fork inherits filesystem
//! state exactly as the language says it does.
//!
//! ## Talking to Docker
//!
//! Through the `docker` CLI, not a client library. The surface used is four
//! verbs — `run`, `exec`, `cp`, `rm` — and `docker cp` reads and writes tar
//! streams directly, which is precisely the shape [`Executor::inject_volume`]
//! and [`Executor::extract_volume`] need. (The canopy executor, lacking that,
//! ships volumes as base64 over a pty in 2 KiB chunks.) `LocalExecutor`
//! likewise shells out to `sh`; the process boundary is the same one.
//!
//! ## Mount semantics match the other executors
//!
//! `mount="vol:/out"` resolves to `/out` UNDER the container workdir on every
//! backend — `LocalExecutor` to `<tmp>/<container>/out`, `CanopyExecutor` to
//! `/hickory-work/out`, and here to `/work/out`. A document that addresses a
//! mount absolutely is broken on all three, identically. There is a test
//! pinning the agreement.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result, bail};
use async_trait::async_trait;
use hick_token::ContainerCapabilities;
use hickory_executor::{
    ContainerResourceStats, ExecTranscriptEntry, Executor, TranscriptEvent, Transcripts,
    normalize_captured_newlines,
};
use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;

/// Working directory inside every container. Mount paths resolve under it.
const GUEST_WORKDIR: &str = "/work";

/// Docker container name prefix, so a stray container is identifiable and a
/// sweep is one `docker rm -f $(docker ps -aq --filter name=hickory-)`.
const NAME_PREFIX: &str = "hickory-";

/// One running container.
struct DockerContainer {
    /// The `--name` given to Docker (not the hick container name).
    docker_name: String,
    /// The `--network` it was actually started with. Kept so a later
    /// declaration for the same name can be checked against what it got,
    /// rather than silently disagreeing with reality.
    network: String,
    total_exec: Duration,
    command_durations: Vec<Duration>,
}

/// Resource limits applied to every container.
///
/// A hosted deployment runs documents written by strangers, and untrusted
/// code on free compute attracts miners within days. These are the floor,
/// not the whole answer — per-account execution-minute quotas live in the
/// server — but a container with no memory or PID cap can take down the host
/// before any quota notices.
#[derive(Debug, Clone)]
pub struct DockerLimits {
    /// `--memory` value (e.g. `512m`).
    pub memory: String,
    /// `--cpus` value (e.g. `1.0`).
    pub cpus: String,
    /// `--pids-limit`: the cheapest defence against a fork bomb.
    pub pids: u32,
    /// `--network` for a container that was **not** granted network access:
    /// `none` by default.
    ///
    /// Documents that genuinely need the network are the exception, and a
    /// default of "connected" is a default of "exfiltration is possible".
    pub network: String,
    /// `--network` for a container whose document grants it network access
    /// with `<hick:allow network="host:port">`: `bridge` by default.
    ///
    /// Two knobs rather than one because they answer different questions.
    /// `network` is the floor every undeclared container sits on;
    /// `allowed_network` is what "yes, this one may talk out" resolves to on
    /// this host — a deployment that routes egress through a filtering
    /// network names it here (`HICKORY_DOCKER_ALLOWED_NETWORK`).
    ///
    /// Docker's `--network` is on/off, so the host and port in the
    /// declaration are **not** enforced here: a container granted
    /// `github.com:443` can reach anything the chosen network can. Narrowing
    /// that needs an egress proxy and is deliberately out of scope.
    pub allowed_network: String,
}

impl Default for DockerLimits {
    fn default() -> Self {
        Self {
            memory: env_or("HICKORY_DOCKER_MEMORY", "512m"),
            cpus: env_or("HICKORY_DOCKER_CPUS", "1.0"),
            pids: env_or("HICKORY_DOCKER_PIDS", "256").parse().unwrap_or(256),
            network: env_or("HICKORY_DOCKER_NETWORK", "none"),
            allowed_network: env_or("HICKORY_DOCKER_ALLOWED_NETWORK", "bridge"),
        }
    }
}

/// The `--network` a container starts with, given what its document declared.
///
/// Pure so the decision is testable without a Docker daemon: it is the whole
/// of "a document declaring network access gets it; one that does not,
/// does not".
///
/// A container with no declaration at all falls to `limits.network` — the
/// executor-wide floor. That is the same answer as an explicit declaration
/// that grants nothing, and deliberately so: silence is not consent.
fn network_mode_for(limits: &DockerLimits, capabilities: Option<&ContainerCapabilities>) -> String {
    match capabilities {
        Some(caps) if caps.allows_network() => limits.allowed_network.clone(),
        _ => limits.network.clone(),
    }
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

/// Executor that runs hick containers as Docker containers.
pub struct DockerExecutor {
    /// Unique per executor instance, so concurrent runs on one host cannot
    /// collide on container names.
    run_id: String,
    limits: DockerLimits,
    containers: Mutex<HashMap<String, DockerContainer>>,
    transcripts: Mutex<Transcripts>,
    /// Images created by `register_fork`, removed on shutdown.
    fork_images: Mutex<Vec<String>>,
    /// Forks registered before their target started: target -> source.
    pending_forks: Mutex<HashMap<String, String>>,
    /// Capabilities the document declared, per container. Consulted at
    /// `docker run` time, which is why the pipeline declares them before
    /// anything starts.
    declared: Mutex<HashMap<String, ContainerCapabilities>>,
}

impl DockerExecutor {
    /// Create an executor with limits from the environment.
    ///
    /// Verifies the daemon is reachable now rather than at first exec, so a
    /// missing or stopped Docker surfaces as a clear startup error instead of
    /// a failed cell halfway through a document.
    pub async fn new() -> Result<Self> {
        Self::with_limits(DockerLimits::default()).await
    }

    /// Create an executor with explicit limits.
    pub async fn with_limits(limits: DockerLimits) -> Result<Self> {
        let probe = Command::new("docker")
            .args(["version", "--format", "{{.Server.Version}}"])
            .output()
            .await
            .context(
                "could not run `docker` — the docker executor needs the Docker CLI on PATH \
                 (set HICKORY_EXECUTOR=local to run against the host toolchain instead)",
            )?;
        if !probe.status.success() {
            bail!(
                "the Docker daemon is not reachable: {}",
                String::from_utf8_lossy(&probe.stderr).trim()
            );
        }
        Ok(Self {
            run_id: unique_suffix(),
            limits,
            containers: Mutex::new(HashMap::new()),
            transcripts: Mutex::new(HashMap::new()),
            fork_images: Mutex::new(Vec::new()),
            pending_forks: Mutex::new(HashMap::new()),
            declared: Mutex::new(HashMap::new()),
        })
    }

    /// Resolve a mount path to an absolute path under the container workdir.
    ///
    /// Leading `/` is stripped: `/data` becomes `/work/data`. This mirrors
    /// `LocalExecutor::mount_dir` and `CanopyExecutor::mount_dir` exactly.
    pub fn mount_dir(mount_path: &str) -> String {
        let rel = mount_path.trim_start_matches('/');
        if rel.is_empty() {
            GUEST_WORKDIR.to_string()
        } else {
            format!("{GUEST_WORKDIR}/{rel}")
        }
    }

    /// The configured `--network` value, for tests that assert the default.
    pub fn limits_network(&self) -> &str {
        &self.limits.network
    }

    /// The `--network` `container` has, or would get if started now.
    ///
    /// A running container answers with what it was actually started with,
    /// so this never describes a confinement the container does not have.
    pub fn network_mode(&self, container: &str) -> String {
        if let Some(running) = self.containers.lock().unwrap().get(container) {
            return running.network.clone();
        }
        network_mode_for(&self.limits, self.declared.lock().unwrap().get(container))
    }

    fn docker_name(&self, container: &str) -> String {
        format!("{NAME_PREFIX}{}-{}", self.run_id, sanitize(container))
    }

    fn name_of(&self, container: &str) -> Result<String> {
        self.containers
            .lock()
            .unwrap()
            .get(container)
            .map(|c| c.docker_name.clone())
            .ok_or_else(|| anyhow::anyhow!("container '{container}' has not been started"))
    }

    /// Run a command in `container` and record a transcript entry.
    async fn run_command(
        &self,
        container: &str,
        command: &str,
        stdin_data: Option<&str>,
    ) -> Result<String> {
        let name = self.name_of(container)?;
        let started = Instant::now();

        let mut cmd = Command::new("docker");
        cmd.args([
            "exec",
            "-i",
            "-w",
            GUEST_WORKDIR,
            &name,
            "sh",
            "-c",
            command,
        ])
        .stdin(if stdin_data.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .with_context(|| format!("spawning docker exec in '{container}'"))?;
        if let Some(data) = stdin_data
            && let Some(mut stdin) = child.stdin.take()
        {
            stdin.write_all(data.as_bytes()).await?;
            stdin.shutdown().await?;
        }
        let out = child.wait_with_output().await?;

        let elapsed = started.elapsed();
        // `\r\n` -> `\n`, the same rewrite `LocalExecutor` applies: a
        // Windows-container image, or a program inside a Linux one that emits
        // DOS line endings, must not make a document's expectations mean
        // something different from where they were written.
        // docs/guarantees/verification/an-expectation-means-the-same-on-every-platform.md
        let stdout =
            normalize_captured_newlines(&String::from_utf8_lossy(&out.stdout)).into_owned();
        let stderr =
            normalize_captured_newlines(&String::from_utf8_lossy(&out.stderr)).into_owned();
        let code = out.status.code().unwrap_or(-1);
        let t = elapsed.as_millis() as u64;

        // The transcript is recorded BEFORE the error is returned, exactly
        // like LocalExecutor: a failing cell is still an observation, and the
        // agent's script runner recovers the result from the transcript.
        let mut events = vec![TranscriptEvent::Cmd {
            t: 0,
            data: command.to_string(),
        }];
        if !stdout.is_empty() {
            events.push(TranscriptEvent::Out {
                t,
                data: stdout.clone(),
            });
        }
        if !stderr.is_empty() {
            events.push(TranscriptEvent::Err {
                t,
                data: stderr.clone(),
            });
        }
        events.push(TranscriptEvent::Exit { t, code });

        self.transcripts
            .lock()
            .unwrap()
            .entry(container.to_string())
            .or_default()
            .push(ExecTranscriptEntry {
                commands: command
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(str::to_string)
                    .collect(),
                output: stdout.clone(),
                events,
                source_line: None,
            });

        if let Some(c) = self.containers.lock().unwrap().get_mut(container) {
            c.total_exec += elapsed;
            c.command_durations.push(elapsed);
        }

        if code != 0 {
            bail!("command failed in container '{container}' (exit {code}): {command}\n{stderr}");
        }
        Ok(stdout)
    }

    /// Start a container from `image`, or from a forked image when
    /// `register_fork` named this container as a target.
    async fn start(&self, container: &str, image: &str) -> Result<()> {
        let source = self.pending_forks.lock().unwrap().remove(container);
        let image = match source {
            Some(from) => self.commit_fork(&from, container).await?,
            None => image.to_string(),
        };

        let name = self.docker_name(container);
        let network = self.network_mode(container);
        // `sleep infinity` is not portable (busybox ash lacks it); a loop is.
        let out = Command::new("docker")
            .args([
                "run",
                "-d",
                "--name",
                &name,
                "--workdir",
                GUEST_WORKDIR,
                "--memory",
                &self.limits.memory,
                "--cpus",
                &self.limits.cpus,
                "--pids-limit",
                &self.limits.pids.to_string(),
                "--network",
                &network,
                "--entrypoint",
                "/bin/sh",
                &image,
                "-c",
                "mkdir -p /work && while :; do sleep 3600; done",
            ])
            .output()
            .await
            .context("spawning docker run")?;
        if !out.status.success() {
            bail!(
                "could not start container '{container}' from image '{image}': {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }

        self.containers.lock().unwrap().insert(
            container.to_string(),
            DockerContainer {
                docker_name: name,
                network: network.clone(),
                total_exec: Duration::ZERO,
                command_durations: Vec::new(),
            },
        );
        log::info!("[docker:{container}] started from image '{image}' (network {network})");
        Ok(())
    }

    /// Commit the source container to a throwaway image so the fork inherits
    /// its filesystem — which is what `hick:fork` claims to do.
    async fn commit_fork(&self, from: &str, target: &str) -> Result<String> {
        let source_name = self.name_of(from).with_context(|| {
            format!("fork target '{target}' names source '{from}', which has not started")
        })?;
        let tag = format!("{NAME_PREFIX}fork-{}-{}", self.run_id, sanitize(target));
        let out = Command::new("docker")
            .args(["commit", &source_name, &tag])
            .output()
            .await
            .context("spawning docker commit")?;
        if !out.status.success() {
            bail!(
                "could not fork '{from}' into '{target}': {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        self.fork_images.lock().unwrap().push(tag.clone());
        Ok(tag)
    }
}

#[async_trait]
impl Executor for DockerExecutor {
    async fn declare_capabilities(
        &self,
        container: &str,
        capabilities: ContainerCapabilities,
    ) -> Result<()> {
        // One executor can serve several documents in a row (the agent runs
        // a document and its upstreams through the same one), so the same
        // container name can be declared again after it started. That is
        // fine while the declaration asks for the confinement the container
        // already has; it is not fine when it asks for a different one,
        // because `docker run` flags cannot be changed after the fact and
        // pretending otherwise would leave a container confined differently
        // from what its document says.
        let wanted = network_mode_for(&self.limits, Some(&capabilities));
        if let Some(running) = self.containers.lock().unwrap().get(container) {
            if running.network != wanted {
                bail!(
                    "container '{container}' is already running with `--network {}`, so a \
                     declaration asking for `--network {wanted}` cannot be applied.\n\
                     Next steps: run this document with a fresh executor, or give the \
                     container a distinct name — a container's capabilities must be \
                     declared before it starts.",
                    running.network
                );
            }
            return Ok(());
        }
        self.declared
            .lock()
            .unwrap()
            .insert(container.to_string(), capabilities);
        Ok(())
    }

    async fn ensure_started(&self, container: &str, image: &str) -> Result<()> {
        if self.containers.lock().unwrap().contains_key(container) {
            return Ok(());
        }
        self.start(container, image).await
    }

    // `execute_with_options` is NOT overridden here, deliberately: this
    // executor runs cells UNBOUNDED. Killing the `docker exec` client
    // process does not kill the command inside the container, so honouring
    // a timeout here means `docker kill`/exec-inspect plumbing that does
    // not exist yet. Until it does, saying "no timeout" is more honest than
    // killing the client and leaving the cell running.
    // docs/guarantees/execution/a-cell-cannot-hang-a-run.md
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
        _additional_caps: Option<ContainerCapabilities>,
    ) -> Result<()> {
        // Recorded, not acted on: the fork's image is committed when the
        // target actually starts, because committing earlier would capture a
        // filesystem the source has not finished writing.
        self.pending_forks
            .lock()
            .unwrap()
            .insert(target.to_string(), from.to_string());
        Ok(())
    }

    async fn create_mount_point(&self, container: &str, mount_path: &str) -> Result<()> {
        let dir = Self::mount_dir(mount_path);
        // Not run_command: creating a mount point is plumbing, and recording
        // it as a transcript entry would put `mkdir` in the woven document.
        let name = self.name_of(container)?;
        let out = Command::new("docker")
            .args(["exec", &name, "mkdir", "-p", &dir])
            .output()
            .await
            .context("spawning docker exec mkdir")?;
        if !out.status.success() {
            bail!(
                "could not create mount point {mount_path} in '{container}': {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(())
    }

    async fn inject_volume(
        &self,
        container: &str,
        mount_path: &str,
        tar_data: &[u8],
    ) -> Result<()> {
        let dir = Self::mount_dir(mount_path);
        self.create_mount_point(container, mount_path).await?;
        let name = self.name_of(container)?;

        // `docker cp -` reads a tar stream on stdin: exactly the bytes the
        // trait hands us, with no re-encoding.
        let mut child = Command::new("docker")
            .args(["cp", "-", &format!("{name}:{dir}")])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("spawning docker cp (inject)")?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(tar_data).await?;
            stdin.shutdown().await?;
        }
        let out = child.wait_with_output().await?;
        if !out.status.success() {
            bail!(
                "could not inject volume into {mount_path}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(())
    }

    async fn extract_volume(&self, container: &str, mount_path: &str) -> Result<Vec<u8>> {
        let dir = Self::mount_dir(mount_path);
        let name = self.name_of(container)?;
        // The `/.` suffix copies the directory CONTENTS, so the archive has
        // the same shape `inject_volume` expects to receive.
        let out = Command::new("docker")
            .args(["cp", &format!("{name}:{dir}/."), "-"])
            .output()
            .await
            .context("spawning docker cp (extract)")?;
        if !out.status.success() {
            bail!(
                "could not extract volume at {mount_path}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(out.stdout)
    }

    fn transcripts(&self) -> Transcripts {
        self.transcripts.lock().unwrap().clone()
    }

    fn inject_transcript_entry(&self, container: &str, entry: ExecTranscriptEntry) {
        self.transcripts
            .lock()
            .unwrap()
            .entry(container.to_string())
            .or_default()
            .push(entry);
    }

    fn resource_stats(&self) -> HashMap<String, ContainerResourceStats> {
        self.containers
            .lock()
            .unwrap()
            .iter()
            .map(|(name, c)| {
                (
                    name.clone(),
                    ContainerResourceStats {
                        boot_duration: Duration::ZERO,
                        command_durations: c.command_durations.clone(),
                        total_exec_duration: c.total_exec,
                    },
                )
            })
            .collect()
    }

    async fn shutdown(&self) -> Result<()> {
        let names: Vec<String> = self
            .containers
            .lock()
            .unwrap()
            .values()
            .map(|c| c.docker_name.clone())
            .collect();
        for name in names {
            // Best effort: a container that is already gone is not an error,
            // and one failure must not strand the rest.
            let _ = Command::new("docker")
                .args(["rm", "-f", &name])
                .output()
                .await;
        }
        self.containers.lock().unwrap().clear();

        let images: Vec<String> = std::mem::take(&mut *self.fork_images.lock().unwrap());
        for image in images {
            let _ = Command::new("docker")
                .args(["rmi", "-f", &image])
                .output()
                .await;
        }
        Ok(())
    }
}

/// Docker names allow `[a-zA-Z0-9][a-zA-Z0-9_.-]*`; hick container names are
/// arbitrary text.
fn sanitize(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if s.is_empty() { "c".to_string() } else { s }
}

/// A short unique-per-process suffix. Not random: `Instant` is monotonic and
/// the pid disambiguates concurrent processes, which is all that is needed to
/// keep container names from colliding.
fn unique_suffix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mount_dir_mirrors_the_other_executors() {
        // A document that addresses a mount absolutely must fail the same way
        // on every backend; divergence here is a document that works in dev
        // and breaks in production.
        assert_eq!(DockerExecutor::mount_dir("/data"), "/work/data");
        assert_eq!(DockerExecutor::mount_dir("data"), "/work/data");
        assert_eq!(DockerExecutor::mount_dir("/"), "/work");
    }

    /// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
    #[test]
    fn the_document_decides_whether_a_container_has_a_network() {
        let limits = DockerLimits {
            network: "none".to_string(),
            allowed_network: "bridge".to_string(),
            ..DockerLimits::default()
        };

        // Declared nothing, or declared and granted nothing: the floor.
        assert_eq!(network_mode_for(&limits, None), "none");
        assert_eq!(
            network_mode_for(&limits, Some(&ContainerCapabilities::new())),
            "none"
        );
        assert_eq!(
            network_mode_for(
                &limits,
                Some(&ContainerCapabilities::new().deny_all_network())
            ),
            "none"
        );

        // Declared network access: granted.
        let allowed = ContainerCapabilities::new().allow_network("github.com", "443");
        assert_eq!(network_mode_for(&limits, Some(&allowed)), "bridge");

        // `<hick:deny network="*">` alongside an allowlist is the language's
        // "default deny, except these" — the container still needs a network
        // to reach the exceptions, and a switch cannot express the rest.
        let allowlisted = ContainerCapabilities::new()
            .deny_all_network()
            .allow_network("github.com", "443");
        assert_eq!(network_mode_for(&limits, Some(&allowlisted)), "bridge");
    }

    #[test]
    fn container_names_are_docker_safe() {
        assert_eq!(sanitize("My Container/1"), "My-Container-1");
        assert_eq!(sanitize(""), "c");
        assert_eq!(sanitize("py-3.12_x"), "py-3.12_x");
    }
}
