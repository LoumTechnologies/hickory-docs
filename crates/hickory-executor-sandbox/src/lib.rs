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

pub mod policy;

use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::{Result, bail};
use async_trait::async_trait;
use hick_token::ContainerCapabilities;
use hickory_executor::{
    ContainerResourceStats, ExecTranscriptEntry, Executor, LocalExecutor, Transcripts,
};

pub use policy::Sandbox;

/// Runs cells through [`LocalExecutor`], each command confined by the
/// platform's sandbox.
pub struct SandboxedExecutor {
    inner: LocalExecutor,
    sandbox: Sandbox,
    /// Declared capabilities per container: the network is opened only for a
    /// container whose document asked for it.
    capabilities: Mutex<HashMap<String, ContainerCapabilities>>,
}

impl SandboxedExecutor {
    /// Build one, or explain why this machine cannot.
    pub fn new() -> Result<Self> {
        let sandbox = Sandbox::detect();
        if sandbox == Sandbox::None {
            bail!(
                "no sandbox is available, so `HICKORY_EXECUTOR=sandbox` cannot confine anything.\n\
                 {}\n\
                 Refusing rather than running your document's commands unconfined — that is what \
                 you asked this executor to prevent.",
                policy::Sandbox::missing_hint()
            );
        }
        log::info!("sandboxed executor: {}", sandbox.describe());
        Ok(Self {
            inner: LocalExecutor::new()?,
            sandbox,
            capabilities: Mutex::new(HashMap::new()),
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

    /// Rewrite a command so the shell that runs it is inside the sandbox.
    ///
    /// The wrapped form is a single `sh -c` string because that is what
    /// `LocalExecutor` spawns; quoting the inner command keeps a cell's own
    /// shell syntax intact.
    fn confine(&self, container: &str, command: &str) -> Result<String> {
        let workdir = self.inner.workdir_of(container)?;
        let Some((program, args)) = policy::wrap(
            self.sandbox,
            &workdir,
            command,
            self.allows_network(container),
        ) else {
            bail!("no sandbox available to confine container '{container}'");
        };
        let mut line = shell_quote(&program);
        for arg in args {
            line.push(' ');
            line.push_str(&shell_quote(&arg));
        }
        Ok(line)
    }
}

/// Single-quote for `sh`, the only quoting that has no escapes to get wrong.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
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

    async fn execute(&self, container: &str, command: &str) -> Result<String> {
        let confined = self.confine(container, command)?;
        // The transcript records the cell as written; only the spawn sees the
        // sandbox. A woven document full of `bwrap --ro-bind …` would be a
        // page about our implementation in the middle of somebody else's
        // work.
        self.inner
            .execute_as(container, command, &confined, None)
            .await
    }

    async fn execute_with_stdin(
        &self,
        container: &str,
        command: &str,
        stdin_data: &str,
    ) -> Result<String> {
        let confined = self.confine(container, command)?;
        self.inner
            .execute_as(container, command, &confined, Some(stdin_data))
            .await
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
mod tests {
    use super::*;

    #[test]
    fn quoting_survives_a_command_containing_quotes() {
        // A cell full of shell quoting must reach the shell unchanged.
        let quoted = shell_quote(r#"echo 'it'\''s fine'"#);
        assert!(quoted.starts_with('\''));
        assert!(quoted.ends_with('\''));
        assert!(quoted.contains(r"'\''"));
    }
}
