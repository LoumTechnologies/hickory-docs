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

    /// Add the sandbox's own explanation to a failure it caused.
    ///
    /// Only where the evidence points that way, and only when the container
    /// really had no network: appending "maybe the sandbox did this" to every
    /// failing cell would train people to ignore it.
    fn explain(&self, container: &str, error: anyhow::Error) -> anyhow::Error {
        let text = format!("{error:#}");
        if self.allows_network(container) || !looks_like_a_denied_network(&text) {
            return error;
        }
        error.context(format!(
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
        ))
    }

    /// Rewrite a command so the shell that runs it is inside the sandbox.
    ///
    /// The wrapped form is a single `sh -c` string because that is what
    /// `LocalExecutor` spawns; quoting the inner command keeps a cell's own
    /// shell syntax intact.
    fn confine(&self, container: &str, command: &str) -> Result<String> {
        let workdir = self.inner.workdir_of(container)?;
        // One /tmp per container, not per command — see `tmpdir_of`.
        let tmpdir = self.inner.tmpdir_of(container)?;
        // What to hide from this cell, asked at the moment it runs: a container
        // started later is not in the list, because it did not exist when the
        // policy was written.
        let peers = self.inner.peer_dirs(container);
        let Some((program, args)) = policy::wrap(
            self.sandbox,
            &workdir,
            command,
            self.allows_network(container),
            policy::Profile::Cell,
            Some(&tmpdir),
            &peers,
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
        let mut line = shell_quote_program(&program);
        for arg in args {
            line.push(' ');
            line.push_str(&shell_quote(&arg));
        }
        Ok(line)
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

/// Quote one argument for the shell this platform hands commands to.
///
/// Two shells, two rules, and getting it wrong is not a cosmetic bug: an
/// unquoted workdir with a space in it turns one argument into two, and the
/// sandbox confines the wrong directory.
/// Quote the program name.
///
/// On Windows this is deliberately NOT metacharacter-escaped: `cmd` resolves
/// the executable *before* it consumes carets, so `C:\Program Files ^(x86^)\`
/// is looked up literally and reported as "The system cannot find the path
/// specified" (measured on Windows 11, 2026-08-19).
fn shell_quote_program(value: &str) -> String {
    if cfg!(windows) {
        windows_quote(value, false)
    } else {
        format!("'{}'", value.replace('\'', r"'\''"))
    }
}

/// Quote an argument, hiding anything the outer shell would otherwise act on.
fn shell_quote(value: &str) -> String {
    if cfg!(windows) {
        windows_quote(value, true)
    } else {
        // Single quotes: the only sh quoting with no escapes to get wrong.
        format!("'{}'", value.replace('\'', r"'\''"))
    }
}

/// Quote for `cmd.exe`, which is two problems rather than one.
///
/// `CommandLineToArgvW` splits on unquoted spaces and treats backslashes as
/// escapes only when they precede a quote; `cmd.exe` *additionally* expands
/// `%VAR%` and treats `&|<>^` as syntax before any of that happens. So the
/// argument is double-quoted for the parser and the metacharacters that
/// survive quoting are caret-escaped for the shell.
fn windows_quote(value: &str, hide_metacharacters: bool) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    let mut backslashes = 0;
    for ch in value.chars() {
        match ch {
            '\\' => {
                backslashes += 1;
                out.push(ch);
            }
            '"' => {
                // Every backslash immediately before a quote is doubled, then
                // the quote itself is escaped.
                for _ in 0..=backslashes {
                    out.push('\\');
                }
                backslashes = 0;
                out.push('"');
            }
            // cmd.exe reads these as syntax *before* CommandLineToArgvW ever
            // splits the line, and — measured on Windows 11, not assumed — it
            // does so even inside double quotes. A cell's `a > b && c` is
            // otherwise executed by the OUTER shell: the redirect swallows the
            // argument and the second half runs unconfined. The caret is
            // consumed by cmd, so the confined process still receives the bare
            // character.
            _ => {
                backslashes = 0;
                if hide_metacharacters && matches!(ch, '&' | '<' | '>' | '^' | '|' | '(' | ')') {
                    out.push('^');
                }
                out.push(ch);
            }
        }
    }
    // Trailing backslashes would escape the closing quote.
    for _ in 0..backslashes {
        out.push('\\');
    }
    out.push('"');
    out
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

    async fn probe(&self, container: &str, command: &str) -> Result<bool> {
        // Confined, like everything else. "Is duckdb installed?" and "can
        // this cell see duckdb?" have different answers here — the cell's
        // $HOME is a tmpfs with only toolchain directories bound back — and
        // the second is the only one worth asking, since it is the one that
        // decides whether the cell works.
        let confined = self.confine(container, command)?;
        self.inner.probe_command(container, &confined).await
    }

    async fn execute(&self, container: &str, command: &str) -> Result<String> {
        let confined = self.confine(container, command)?;
        // The transcript records the cell as written; only the spawn sees the
        // sandbox. A woven document full of `bwrap --ro-bind …` would be a
        // page about our implementation in the middle of somebody else's
        // work.
        self.inner
            .execute_as(container, command, &confined, None, ExecOptions::default())
            .await
            .map_err(|error| self.explain(container, error))
    }

    async fn execute_with_stdin(
        &self,
        container: &str,
        command: &str,
        stdin_data: &str,
    ) -> Result<String> {
        let confined = self.confine(container, command)?;
        self.inner
            .execute_as(
                container,
                command,
                &confined,
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
        let confined = self.confine(container, command)?;
        self.inner
            .execute_as(container, command, &confined, stdin_data, options)
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
mod tests {
    use super::*;

    #[test]
    fn a_windows_path_with_a_space_stays_one_argument() {
        // The failure this prevents: `C:\Program Files\…` splits into two
        // arguments, the launcher confines a directory that does not exist,
        // and the sandbox protects nothing that matters.
        let quoted = windows_quote(r"C:\Program Files\thing", false);
        assert_eq!(quoted, r#""C:\Program Files\thing""#);
    }

    #[test]
    fn a_trailing_backslash_does_not_escape_the_closing_quote() {
        // A Windows directory path ends in a backslash often enough that this
        // is the realistic corruption: `"C:\dir\"` swallows the quote and
        // everything after it becomes part of the argument.
        assert_eq!(windows_quote(r"C:\dir\", false), r#""C:\dir\\""#);
    }

    #[test]
    fn an_embedded_quote_is_escaped_with_its_backslashes_doubled() {
        assert_eq!(windows_quote(r#"say "hi""#, false), r#""say \"hi\"""#);
    }

    /// The `sh` half of `shell_quote`, hence Unix-only.
    ///
    /// It calls the platform-dispatching function and asserts single quotes,
    /// which is the right shape on Unix and the wrong one on Windows — there
    /// `shell_quote` returns `windows_quote`'s double-quoted, caret-escaped
    /// form, and this failed as the very first Windows test anyone ran. The
    /// Windows side is covered directly by the three `windows_quote` tests
    /// above, which are not gated because that function compiles everywhere.
    #[cfg(unix)]
    #[test]
    fn quoting_survives_a_command_containing_quotes() {
        // A cell full of shell quoting must reach the shell unchanged.
        let quoted = shell_quote(r#"echo 'it'\''s fine'"#);
        assert!(quoted.starts_with('\''));
        assert!(quoted.ends_with('\''));
        assert!(quoted.contains(r"'\''"));
    }

    /// `cmd.exe`'s documented `/C` quote handling, so the rule this code
    /// depends on can be asserted from any machine.
    ///
    /// From `cmd /?`: quotes are preserved only when there is no `/S`, there
    /// are **exactly two** quote characters, no special characters between
    /// them, whitespace between them, and the text between them names an
    /// executable. Otherwise the leading quote and the **last** quote are
    /// removed. Verified against a real cmd.exe on Windows 11 (2026-08-19) —
    /// both the failing and the passing form.
    fn cmd_strips_quotes(line: &str) -> String {
        if line.matches('"').count() == 2 {
            return line.to_string();
        }
        let Some(rest) = line.strip_prefix('"') else {
            return line.to_string();
        };
        match rest.rfind('"') {
            Some(last) => format!("{}{}", &rest[..last], &rest[last + 1..]),
            None => rest.to_string(),
        }
    }

    /// The program `cmd.exe` would try to execute from a command line.
    fn program_of(line: &str) -> String {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix('"') {
            return rest.split('"').next().unwrap_or_default().to_string();
        }
        line.split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string()
    }

    /// Compose a confined line the way `confine` does, for a Windows target,
    /// regardless of the host this test runs on.
    fn confined_line(program: &str, args: &[&str]) -> String {
        let mut line = windows_quote(program, false);
        for arg in args {
            line.push(' ');
            line.push_str(&windows_quote(arg, true));
        }
        line
    }

    #[test]
    fn a_confined_line_survives_cmds_quote_stripping() {
        // Regression: the launcher path, the workdir, and the cell's command
        // are three quoted arguments — six quote characters — so cmd takes the
        // strip branch. Handed over bare, the program name kept a trailing
        // quote and every confined cell failed with ERROR_INVALID_NAME before
        // it ran. Measured, not guessed: see the doc on `cmd_strips_quotes`.
        let launcher = r"C:\Program Files\Hickory\hick.exe";
        let line = confined_line(
            launcher,
            &[
                "__sandbox-run",
                "--workdir",
                r"C:\Users\x\AppData\Local\Temp\c1",
                "--",
                "echo hello > note.txt && cat note.txt",
            ],
        );

        let bare = cmd_strips_quotes(&line);
        assert_ne!(
            program_of(&bare),
            launcher,
            "if cmd stopped mangling a bare line this guard is obsolete — check \
             a real cmd.exe before deleting it"
        );

        // What `shell_command` actually hands to `cmd /C`.
        let wrapped = cmd_strips_quotes(&format!("\"{line}\""));
        assert_eq!(
            program_of(&wrapped),
            launcher,
            "the launcher must survive cmd's quote stripping intact"
        );
    }

    #[test]
    fn the_cells_shell_syntax_is_hidden_from_the_outer_shell() {
        // Without the carets the outer cmd performs the redirect itself: the
        // argument is swallowed and `cat` runs unconfined. Confirmed on
        // Windows 11 — the confined process received no command at all.
        let quoted = windows_quote("echo hello > note.txt && cat note.txt", true);
        assert!(
            quoted.contains("^>"),
            "the redirect must be hidden: {quoted}"
        );
        assert!(
            quoted.contains("^&^&"),
            "the operator must be hidden: {quoted}"
        );
        assert!(
            !quoted.contains(" > ") && !quoted.contains(" && "),
            "no metacharacter may reach the outer shell unescaped: {quoted}"
        );
    }

    #[test]
    fn a_literal_caret_is_escaped_too() {
        assert_eq!(windows_quote("a^b", true), r#""a^^b""#);
    }

    #[test]
    fn the_program_path_is_never_metacharacter_escaped() {
        // `C:\Program Files (x86)\...` is the ordinary case, and cmd resolves
        // the executable before it consumes carets — escaping the parentheses
        // makes the launcher unfindable. Measured on Windows 11: "The system
        // cannot find the path specified".
        let path = r"C:\Program Files (x86)\Hickory\hick.exe";
        let quoted = windows_quote(path, false);
        assert!(
            !quoted.contains('^'),
            "the program must stay literal: {quoted}"
        );
        assert!(quoted.contains("(x86)"), "{quoted}");
    }

    #[test]
    fn a_caret_in_an_argument_is_doubled_so_it_survives() {
        // cmd consumes one caret from every argument it passes on, so a path
        // that genuinely contains one arrives short of it: `has^caret` reached
        // the confined process as `hascaret`.
        assert_eq!(
            windows_quote(r"C:\a\has^caret", true),
            r#""C:\a\has^^caret""#
        );
    }
}
