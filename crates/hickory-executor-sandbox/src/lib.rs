//! An executor that runs a document's cells confined.
//!
//! ## The problem it solves
//!
//! `LocalExecutor` runs a cell as you, with your files and your network —
//! its own documentation says so, and compares it to `make`. That is a fair
//! deal for a document you wrote. It is a bad deal for a document you were
//! sent, or one an agent wrote for you, and "read every cell before running
//! it" is advice nobody follows twice.
//!
//! This executor makes running an unfamiliar document a smaller decision:
//! the cell may write its own workdir and nothing else, and it reaches the
//! network only if the document declared it.
//!
//! ## What it is not
//!
//! It is not a virtual machine, and it does not make a cell *deterministic* —
//! the interpreter it runs is still whichever one your machine has. Isolation
//! and reproducibility are different problems: a sandbox restricts what
//! already exists, while a VM image brings a toolchain with it. For the
//! second, point `HICKORY_EXECUTOR=canopy` at a node you run
//! (`crates/hickory-executor-canopy`), which boots a microVM from an image
//! built the same way every time.
//!
//! ## How it works
//!
//! It wraps [`LocalExecutor`] rather than reimplementing it. Everything about
//! transcripts, volumes, forks and mounts is shared, so a document behaves
//! identically under both and there is no second execution path to keep in
//! step. The only difference is that the command is spawned inside a
//! sandbox — see [`policy`] for what that means per platform.
//!
//! ## When it cannot confine anything
//!
//! It refuses to run. Silently falling back to an unconfined execution would
//! be the worst outcome available: the user asked for isolation, believes
//! they have it, and does not. The error names the platform's options.

#[cfg(windows)]
pub mod appcontainer;
pub mod policy;

use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::{Result, bail};
use async_trait::async_trait;
use hick_token::ContainerCapabilities;
use hickory_executor::{
    ContainerResourceStats, ExecOptions, ExecTranscriptEntry, Executor, LocalExecutor, Transcripts,
};

pub use policy::Sandbox;

/// Directories of project-installed tools to bind read-only into a cell.
///
/// Resolved against the project, the same way the scratch root is — and, like
/// it, this used to be resolved against the process's working directory on the
/// assumption that the two are the same. They are the same only when the
/// command was run from inside the project, so `hick run ../proj/doc.hick`
/// looked for the project's installed tools next to whatever shell the person
/// happened to be in and quietly found none. `None` keeps the old behaviour
/// for a caller that genuinely has no document in hand.
///
/// A directory that does not exist is left out rather than bound empty,
/// because `bwrap` fails outright on a missing source.
///
/// The result is ABSOLUTE, and that is not cosmetic. These paths become
/// `bwrap --ro-bind-try <src> <dst>` and a `PATH` prefix inside the cell, both
/// of which need a real location: a relative one binds nothing and puts a
/// directory that does not exist from `/` onto the cell's `PATH`. The project
/// arrives relative in the ordinary case — `hick run doc.hick` gives
/// `Path::new("doc.hick").parent()`, which is `Some("")`, not `None` — so the
/// empty path is resolved the same way [`LocalExecutor::new_stable_for`]
/// resolves it, to the working directory.
fn project_tools(project: Option<&std::path::Path>) -> Vec<std::path::PathBuf> {
    let root = match project.filter(|dir| !dir.as_os_str().is_empty()) {
        Some(dir) => match std::fs::canonicalize(dir) {
            Ok(absolute) => absolute,
            // A path that cannot be canonicalised is no use here: an
            // un-resolvable relative path is exactly what this exists to
            // avoid handing to bwrap.
            Err(_) if dir.is_absolute() => dir.to_path_buf(),
            Err(_) => return Vec::new(),
        },
        None => match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(_) => return Vec::new(),
        },
    };
    [".hick-cache/models"]
        .iter()
        .map(|relative| root.join(relative))
        .filter(|path| path.is_dir())
        .collect()
}

/// Runs cells through [`LocalExecutor`], each command confined by the
/// platform's sandbox.
pub struct SandboxedExecutor {
    inner: LocalExecutor,
    sandbox: Sandbox,
    /// Declared capabilities per container: the network is opened only for a
    /// container whose document asked for it.
    capabilities: Mutex<HashMap<String, ContainerCapabilities>>,
    /// Project-installed tools a cell may run, bound read-only.
    ///
    /// Today this is the code model servers in `.hick-cache/models`. A cell
    /// cannot see the project — it is under `$HOME`, which is replaced by a
    /// tmpfs so your dotfiles and keys are not readable — and that correctly
    /// hides a tool the project installed for cells to use. Binding this one
    /// directory back is the same move `HOME_TOOL_DIRS` already makes for
    /// `~/.local/bin`: the cell can run what you installed and can read
    /// nothing else of yours.
    project_tools: Vec<std::path::PathBuf>,
}

impl SandboxedExecutor {
    /// Build one, or explain why this machine cannot.
    pub fn new() -> Result<Self> {
        Self::build(LocalExecutor::new()?, None)
    }

    /// Like [`new`](Self::new), with the derived scratch directory that makes
    /// a cell's own path stable across runs, named after `project`.
    ///
    /// Opt-in for the reason [`LocalExecutor::new_stable_for`] is: the
    /// derived name is shared by everything running against one project and
    /// only one holder can have it. Right for `hick`, which builds one
    /// executor per process; wrong for a test binary that builds several at
    /// once and would have them delete each other's workdirs.
    pub fn new_stable_for(project: Option<&std::path::Path>) -> Result<Self> {
        Self::build(LocalExecutor::new_stable_for(project)?, project)
    }

    /// [`new_stable_for`](Self::new_stable_for) with no project.
    pub fn new_stable() -> Result<Self> {
        Self::new_stable_for(None)
    }

    fn build(inner: LocalExecutor, project: Option<&std::path::Path>) -> Result<Self> {
        let sandbox = Sandbox::detect();
        if sandbox == Sandbox::None {
            bail!(
                "This machine cannot confine a cell, and hick runs cells confined by default.\n\
                 \n\
                 {}\n\
                 \n\
                 A document's cells are commands, and a document is a file people send each \n\
                 other and agents write. Running them unconfined is a decision, so it is not \n\
                 one made silently on your behalf — but it is yours to make:\n\
                 \n\
                     HICKORY_EXECUTOR=local hick run <document>\n\
                 \n\
                 That gives every cell your files, your keys and your network, exactly as \n\
                 `make` would. For a document you wrote, that is a fair trade.",
                policy::Sandbox::missing_hint()
            );
        }
        log::info!("sandboxed executor: {}", sandbox.describe());
        Ok(Self {
            inner,
            sandbox,
            capabilities: Mutex::new(HashMap::new()),
            project_tools: project_tools(project),
        })
    }

    /// The confinement in force, for reporting to a person.
    pub fn describe(&self) -> &'static str {
        self.sandbox.describe()
    }

    fn allows_network(&self, container: &str) -> bool {
        self.capabilities
            .lock()
            .unwrap()
            .get(container)
            .is_some_and(|caps| caps.allows_network())
    }

    /// Add the sandbox's own explanation to a failure it caused.
    ///
    /// Only where the evidence points that way, and only when the container
    /// really had no network: appending "maybe the sandbox did this" to every
    /// failing cell would train people to ignore it.
    fn explain(&self, container: &str, error: anyhow::Error) -> anyhow::Error {
        let text = format!("{error:#}");
        if !self.allows_network(container) && looks_like_a_denied_network(&text) {
            return error.context(format!(
                "This container has NO network: `HICKORY_EXECUTOR=sandbox` denies it unless the \n\
                 document asks. The error above is what the command says when it cannot reach \n\
                 anything, not a problem with your connection.\n\
                 \n\
                 To let this container out, declare what it may reach:\n\
                 \n\
                     <hick:allow container=\"{container}\" host=\"example.com\" port=\"443\" />\n\
                 \n\
                 A document that says which hosts it needs is one a reader can check. To run \n\
                 everything unconfined instead, set HICKORY_EXECUTOR=local — that gives the cell \n\
                 your whole machine, which is the trade this executor exists to avoid."
            ));
        }
        if looks_like_a_denied_mach_lookup(&text) {
            return error.context(
                "This sandbox denies Mach lookups by default — its Seatbelt profile grants no \n\
                 `(allow mach-lookup)` — so a program that reaches a macOS system service over \n\
                 Mach IPC fails with that service's own cryptic error instead of a hickory one. \n\
                 `dotnet` hits this on every run: its crypto/certificate stack calls \n\
                 `securityd`/`trustd`, and the denial surfaces as the line above, not as anything \n\
                 naming `dotnet` or hickory.\n\
                 \n\
                 This is not a network problem, and there is no per-document way to grant it: \n\
                 mach-lookup stays denied by design, the same as for every other cell. To run \n\
                 this cell unconfined instead, set HICKORY_EXECUTOR=local — that gives the cell \n\
                 your whole machine, which is the trade this executor exists to avoid."
            );
        }
        error
    }

    /// The program and arguments that run `command` confined.
    ///
    /// An argv, not a shell line. Every sandbox wrapper already carries the
    /// shell it wants inside its own arguments — `bwrap … sh -c …`,
    /// `sandbox-exec … sh -c …`, `hick __sandbox-run … -- …` — so composing a
    /// string for an outer shell to re-split adds a quoting layer that buys
    /// nothing and can only lose.
    fn confine(&self, container: &str, command: &str) -> Result<(String, Vec<String>)> {
        let workdir = self.inner.workdir_of(container)?;
        // One /tmp per container, not per command — see `tmpdir_of`.
        let tmpdir = self.inner.tmpdir_of(container)?;
        // What to hide from this cell, asked at the moment it runs: a container
        // started later is not in the list, because it did not exist when the
        // policy was written.
        let peers = self.inner.peer_dirs(container);
        let Some((program, args)) = policy::wrap(
            self.sandbox,
            &policy::Confinement {
                workdir: &workdir,
                command,
                allow_network: self.allows_network(container),
                profile: policy::Profile::Cell,
                tmpdir: Some(&tmpdir),
                peers: &peers,
                tools: &self.project_tools,
            },
        ) else {
            bail!(
                "cannot confine container '{container}': {}.\n\nOn Windows this \
                 is usually not a missing sandbox but a missing launcher — \
                 confinement re-invokes the `hick` binary, and the running \
                 program ({}) does not answer `{}`. Point {} at a `hick` \
                 executable, or run the cell through the `hick` CLI. To run \
                 without confinement instead, set HICKORY_EXECUTOR=local, which \
                 gives the cell your whole machine.",
                policy::Sandbox::missing_hint(),
                std::env::current_exe()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "this program".into()),
                policy::SANDBOX_RUN_SUBCOMMAND,
                policy::LAUNCHER_VAR,
            );
        };
        Ok((program, args))
    }
}

/// What a confined failure looks like from the outside, and why.
///
/// A cell denied the network does not fail with "the sandbox denied this".
/// It fails with whatever its tool says when the name does not resolve —
/// `Temporary failure in name resolution`, `Connect`, `dns error` — which
/// sends a reader to check their wifi. The sandbox is the only thing that
/// knows the real reason, so it is the only thing that can say so.
fn looks_like_a_denied_network(error: &str) -> bool {
    const SIGNS: &[&str] = &[
        "name resolution",
        "dns error",
        "Temporary failure in name",
        "Could not resolve host",
        "Name or service not known",
        "Network is unreachable",
        "Connection refused",
        "connect: network",
        "getaddrinfo",
        "ENOTFOUND",
        "EAI_AGAIN",
    ];
    SIGNS.iter().any(|sign| error.contains(sign))
}

/// What a denied Mach lookup looks like from the outside, and why.
///
/// The macOS Seatbelt `Profile::Cell` policy grants no `(allow mach-lookup)`,
/// so a program that talks to a system service over Mach IPC — `dotnet`
/// calling into `securityd`/`trustd` for `CSSM_ModuleLoad` is the confirmed
/// case — is refused at the kernel level. What reaches the user is that
/// service's own error, not a hickory string and not anything naming
/// `dotnet` or the sandbox, so it reads as an unrelated crash rather than a
/// missing grant. This list is one confirmed signature; extend it as more
/// are found rather than guessing ahead of evidence.
fn looks_like_a_denied_mach_lookup(error: &str) -> bool {
    const SIGNS: &[&str] = &["CSSM_ModuleLoad"];
    SIGNS.iter().any(|sign| error.contains(sign))
}

#[async_trait]
impl Executor for SandboxedExecutor {
    async fn declare_capabilities(
        &self,
        container: &str,
        capabilities: ContainerCapabilities,
    ) -> Result<()> {
        // Recorded here rather than passed through: this executor is the one
        // that can actually act on them.
        self.capabilities
            .lock()
            .unwrap()
            .insert(container.to_string(), capabilities.clone());
        self.inner
            .declare_capabilities(container, capabilities)
            .await
    }

    async fn ensure_started(&self, container: &str, image: &str) -> Result<()> {
        self.inner.ensure_started(container, image).await
    }

    fn script_platform(&self) -> hickory_executor::ScriptPlatform {
        // Whatever the shell inside the sandbox is, which is the inner
        // executor's answer: bubblewrap and Seatbelt wrap `sh`, and the
        // AppContainer launcher runs `cmd.exe`.
        self.inner.script_platform()
    }

    async fn write_file(&self, container: &str, path: &str, contents: &str) -> Result<()> {
        // Straight through, deliberately. The default on the trait composes
        // `mkdir -p … && cat > …` and runs it through a shell; the inner
        // executor writes the file. Confinement is about what a CELL can
        // reach, and this is the executor placing a file in a workdir it
        // already owns, not a cell reaching for one.
        self.inner.write_file(container, path, contents).await
    }

    async fn probe_program(&self, container: &str, bin: &str) -> Result<bool> {
        // Confined, like everything else. "Is duckdb installed?" and "can
        // this cell see duckdb?" have different answers here — under
        // bubblewrap and Seatbelt the cell's $HOME is a tmpfs with only
        // toolchain directories bound back, and under AppContainer the app
        // package's own ACL decides — and the second question is the only one
        // worth asking, since it is the one that decides whether the cell
        // works. So this asks INSIDE the sandbox rather than resolving the
        // name on the host the way `LocalExecutor` can.
        let Some(command) = policy::probe_command(self.sandbox, bin) else {
            // Nothing to ask with. Not blocking a document over a check that
            // was never performed.
            return Ok(true);
        };
        let (program, args) = self.confine(container, &command)?;
        self.inner.probe_argv(container, &program, &args).await
    }

    async fn execute(&self, container: &str, command: &str) -> Result<String> {
        let (program, args) = self.confine(container, command)?;
        // The transcript records the cell as written; only the spawn sees the
        // sandbox. A woven document full of `bwrap --ro-bind …` would be a
        // page about our implementation in the middle of somebody else's
        // work.
        self.inner
            .execute_argv_as(
                container,
                command,
                &program,
                &args,
                None,
                ExecOptions::default(),
            )
            .await
            .map_err(|error| self.explain(container, error))
    }

    async fn execute_with_stdin(
        &self,
        container: &str,
        command: &str,
        stdin_data: &str,
    ) -> Result<String> {
        let (program, args) = self.confine(container, command)?;
        self.inner
            .execute_argv_as(
                container,
                command,
                &program,
                &args,
                Some(stdin_data),
                ExecOptions::default(),
            )
            .await
            .map_err(|error| self.explain(container, error))
    }

    async fn execute_with_options(
        &self,
        container: &str,
        command: &str,
        stdin_data: Option<&str>,
        options: ExecOptions,
    ) -> Result<String> {
        // The timeout rides through unchanged: the sandbox wrapper (`bwrap`)
        // and the cell's shell are one process group under the inner
        // `LocalExecutor`, so a timed-out confined cell is killed group and
        // all, exactly like an unconfined one.
        // docs/guarantees/execution/a-cell-cannot-hang-a-run.md
        let (program, args) = self.confine(container, command)?;
        self.inner
            .execute_argv_as(container, command, &program, &args, stdin_data, options)
            .await
            .map_err(|error| self.explain(container, error))
    }

    async fn register_fork(
        &self,
        target: &str,
        from: &str,
        additional_caps: Option<ContainerCapabilities>,
    ) -> Result<()> {
        // A fork inherits its parent's grants, then attenuates: a branch can
        // never reach further than what it came from.
        let inherited = self.capabilities.lock().unwrap().get(from).cloned();
        if let Some(base) = inherited {
            let effective = match &additional_caps {
                Some(extra) => base.intersect(extra),
                None => base,
            };
            self.capabilities
                .lock()
                .unwrap()
                .insert(target.to_string(), effective);
        }
        self.inner
            .register_fork(target, from, additional_caps)
            .await
    }

    async fn create_mount_point(&self, container: &str, mount_path: &str) -> Result<()> {
        self.inner.create_mount_point(container, mount_path).await
    }

    async fn inject_volume(
        &self,
        container: &str,
        mount_path: &str,
        tar_data: &[u8],
    ) -> Result<()> {
        self.inner
            .inject_volume(container, mount_path, tar_data)
            .await
    }

    async fn extract_volume(&self, container: &str, mount_path: &str) -> Result<Vec<u8>> {
        self.inner.extract_volume(container, mount_path).await
    }

    fn transcripts(&self) -> Transcripts {
        self.inner.transcripts()
    }

    fn inject_transcript_entry(&self, container: &str, entry: ExecTranscriptEntry) {
        self.inner.inject_transcript_entry(container, entry);
    }

    fn resource_stats(&self) -> HashMap<String, ContainerResourceStats> {
        self.inner.resource_stats()
    }

    async fn shutdown(&self) -> Result<()> {
        self.inner.shutdown().await
    }
}

#[cfg(test)]
mod explain_tests {
    // Guarantee: docs/guarantees/execution/a-denied-mach-lookup-explains-itself.md
    use super::*;

    #[test]
    fn recognizes_the_confirmed_dotnet_signature() {
        assert!(looks_like_a_denied_mach_lookup(
            "CSSM_ModuleLoad(): One or more parameters passed to a function were not valid."
        ));
    }

    #[test]
    fn does_not_mistake_an_unrelated_failure_for_a_denied_mach_lookup() {
        assert!(!looks_like_a_denied_mach_lookup(
            "No such file or directory (os error 2)"
        ));
        assert!(!looks_like_a_denied_mach_lookup(
            "Temporary failure in name resolution"
        ));
    }

    #[test]
    fn still_recognizes_denied_network_signatures() {
        assert!(looks_like_a_denied_network(
            "Temporary failure in name resolution"
        ));
        assert!(!looks_like_a_denied_network("CSSM_ModuleLoad(): invalid"));
    }
}

#[cfg(test)]
mod project_tools_tests {
    //! A tool directory handed to `bwrap` has to be a real location.
    //! Guarantee: docs/guarantees/execution/a-cells-own-path-belongs-to-its-project.md
    use super::*;

    /// Both tests move the working directory, which is per-process.
    static CWD: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_models(dir: &std::path::Path) {
        std::fs::create_dir_all(dir.join(".hick-cache/models")).unwrap();
    }

    #[test]
    fn a_relative_project_resolves_to_an_absolute_tool_directory() {
        let project = tempfile::tempdir().unwrap();
        with_models(project.path());
        // A relative parent is the ordinary case, not an odd one: `hick run
        // doc.hick` and `hick run ../proj/doc.hick` both hand one over.
        // Binding it verbatim gave the cell a path that resolves from `/`.
        let _serial = CWD.lock().unwrap_or_else(|e| e.into_inner());
        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(project.path()).unwrap();
        let found = project_tools(Some(std::path::Path::new(".")));
        std::env::set_current_dir(previous).unwrap();

        assert_eq!(found.len(), 1, "the models directory was not found");
        assert!(
            found[0].is_absolute(),
            "a relative tool directory reaches bwrap as one: {}",
            found[0].display()
        );
    }

    #[test]
    fn an_empty_project_means_the_working_directory() {
        // `Path::new("doc.hick").parent()` is `Some("")`, not `None`, so this
        // is the ordinary `hick run doc.hick` case rather than an odd one.
        let project = tempfile::tempdir().unwrap();
        with_models(project.path());
        let _serial = CWD.lock().unwrap_or_else(|e| e.into_inner());
        let previous = std::env::current_dir().unwrap();
        std::env::set_current_dir(project.path()).unwrap();
        let found = project_tools(Some(std::path::Path::new("")));
        std::env::set_current_dir(previous).unwrap();

        assert_eq!(found.len(), 1, "an empty project found no models directory");
        assert!(found[0].is_absolute());
    }
}
