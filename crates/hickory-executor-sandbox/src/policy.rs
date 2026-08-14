//! What a sandboxed cell may touch, and how each platform is asked to
//! enforce it.
//!
//! The policy is the same everywhere and deliberately small, because a policy
//! nobody can state is a policy nobody can check:
//!
//! * **Writes**: the container's own workdir, and a private `/tmp`. Nothing
//!   else — not `$HOME`, not the repository, not the document that spawned
//!   the cell.
//! * **Reads**: the system, so interpreters and libraries work. A document
//!   that runs `python3` needs Python; hiding it would only mean nothing runs.
//! * **Network**: denied unless the document declared it. `<hick:allow>`
//!   is the only thing that opens it, and silence is not consent
//!   (`hick_token::ContainerCapabilities::allows_network`).
//! * **Processes**: a fresh session, dying with the parent, so a cell cannot
//!   outlive the run that started it.
//!
//! Enforcement differs by platform and the difference is stated rather than
//! smoothed over — see [`Sandbox::describe`]. An executor that claimed
//! isolation it does not have would be worse than one that refuses.

use std::path::Path;

/// How this platform confines a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sandbox {
    /// `bwrap` — Linux namespaces. Read-only root, writable workdir, no
    /// network unless granted.
    Bubblewrap,
    /// `sandbox-exec` — the macOS Seatbelt profile compiler. Deprecated by
    /// Apple, present on every macOS, and the only thing available without
    /// asking the user to install a VM.
    Seatbelt,
    /// Nothing available. The executor refuses rather than pretending.
    None,
}

impl Sandbox {
    /// What is actually available on this machine, right now.
    pub fn detect() -> Self {
        if cfg!(target_os = "linux") && which("bwrap").is_some() {
            Sandbox::Bubblewrap
        } else if cfg!(target_os = "macos") && which("sandbox-exec").is_some() {
            Sandbox::Seatbelt
        } else {
            Sandbox::None
        }
    }

    /// One sentence a person can act on, for logs and for the refusal.
    pub fn describe(self) -> &'static str {
        match self {
            Sandbox::Bubblewrap => {
                "bubblewrap: read-only system, writable workdir and /tmp, no network unless the \
                 document allows it, own PID/IPC/UTS namespaces"
            }
            Sandbox::Seatbelt => {
                "macOS Seatbelt: writable workdir and /tmp, reads elsewhere permitted, network \
                 denied unless the document allows it. Coarser than bubblewrap — Seatbelt cannot \
                 give the cell its own process namespace"
            }
            Sandbox::None => "no sandbox available on this platform",
        }
    }

    /// How to install what is missing, named per platform.
    pub fn missing_hint() -> String {
        if cfg!(target_os = "linux") {
            "Install bubblewrap: `sudo apt install bubblewrap` (Debian/Ubuntu), \
             `sudo dnf install bubblewrap` (Fedora), `sudo pacman -S bubblewrap` (Arch)."
                .to_string()
        } else if cfg!(target_os = "macos") {
            "`sandbox-exec` ships with macOS; a PATH that hides /usr/bin would explain this."
                .to_string()
        } else {
            // Being blunt is the point: Windows has no equivalent this
            // executor can drive, and quietly running unconfined would be a
            // safety claim that is false.
            "Windows has no sandbox this executor can use. Run the cells under WSL2, use the \
             Docker executor (HICKORY_EXECUTOR=docker), or accept the local executor's stated \
             lack of isolation (HICKORY_EXECUTOR=local)."
                .to_string()
        }
    }
}

/// Build the argv that runs `command` under this sandbox.
///
/// Returns the program and its arguments, ready to spawn. The command itself
/// is always handed to `sh -c` inside the sandbox, so a cell's shell syntax
/// behaves exactly as it does unsandboxed.
pub fn wrap(
    sandbox: Sandbox,
    workdir: &Path,
    command: &str,
    allow_network: bool,
) -> Option<(String, Vec<String>)> {
    let dir = workdir.to_string_lossy().to_string();
    let dir_for_home = dir.clone();
    match sandbox {
        Sandbox::Bubblewrap => {
            // ORDER MATTERS: bwrap applies these in sequence, and a later
            // mount hides an earlier one underneath it. The workdir lives
            // under /tmp (LocalExecutor's tempdir), so a private /tmp mounted
            // *after* the workdir bind hides the one directory the cell needs
            // — it fails with "Can't chdir", which reads like a bug in the
            // executor rather than a mount ordering mistake.
            let mut args: Vec<String> = vec![
                // The whole system, read-only: interpreters and libraries
                // work, and nothing outside the workdir can be modified.
                "--ro-bind".into(),
                "/".into(),
                "/".into(),
                // A private /tmp first…
                "--tmpfs".into(),
                "/tmp".into(),
                // …then the workdir on top of it, which is the cell's whole
                // writable world.
                "--bind".into(),
                dir.clone(),
                dir.clone(),
                "--proc".into(),
                "/proc".into(),
                "--dev".into(),
                "/dev".into(),
                "--chdir".into(),
                dir,
                // Its own everything. `--unshare-all` includes the network;
                // `--share-net` below puts it back when the document says so.
                "--unshare-all".into(),
                "--new-session".into(),
                "--die-with-parent".into(),
            ];
            // An empty home. Read-only was not enough: a cell could still
            // LIST ~/.ssh and read whatever it found there, and "it cannot
            // exfiltrate because the network is off" is one mistake away from
            // false. Tools that want a home get an empty one, which is also
            // the more reproducible answer — a cell that behaves differently
            // because of somebody's dotfiles is a cell nobody can re-run.
            if let Some(home) = std::env::var_os("HOME") {
                let home = home.to_string_lossy().to_string();
                if home != "/" && !home.is_empty() && !dir_for_home.starts_with(&home) {
                    args.push("--tmpfs".into());
                    args.push(home);
                }
            }
            if allow_network {
                args.push("--share-net".into());
            }
            args.push("--".into());
            args.push("sh".into());
            args.push("-c".into());
            args.push(command.to_string());
            Some(("bwrap".into(), args))
        }
        Sandbox::Seatbelt => {
            let profile = seatbelt_profile(workdir, allow_network);
            Some((
                "sandbox-exec".into(),
                vec![
                    "-p".into(),
                    profile,
                    "sh".into(),
                    "-c".into(),
                    command.to_string(),
                ],
            ))
        }
        Sandbox::None => None,
    }
}

/// A Seatbelt profile: deny by default, then grant the minimum.
///
/// Written out rather than assembled from a template file so the policy and
/// the code that applies it cannot drift apart, and so a reader can see the
/// whole thing at once.
fn seatbelt_profile(workdir: &Path, allow_network: bool) -> String {
    let dir = workdir.to_string_lossy();
    let mut profile = String::from(
        "(version 1)\
         (deny default)\
         (allow process-exec)\
         (allow process-fork)\
         (allow sysctl-read)\
         (allow file-read*)\
         (allow file-write* (subpath \"/tmp\") (subpath \"/private/tmp\") (subpath \"/dev/null\"))",
    );
    profile.push_str(&format!("(allow file-write* (subpath \"{dir}\"))"));
    // Same reasoning as bubblewrap's empty home: a cell has no business
    // reading dotfiles. Seatbelt cannot mount an empty one, so it denies the
    // reads instead, with the workdir carved back out.
    if let Some(home) = std::env::var_os("HOME") {
        let home = home.to_string_lossy().to_string();
        if home != "/" && !home.is_empty() && !dir.starts_with(&home) {
            profile.push_str(&format!("(deny file-read* (subpath \"{home}\"))"));
        }
    }
    if allow_network {
        profile.push_str("(allow network*)");
    }
    profile
}

/// First match for `name` on `PATH`.
fn which(name: &str) -> Option<std::path::PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn args_for(allow_network: bool) -> Vec<String> {
        wrap(
            Sandbox::Bubblewrap,
            &PathBuf::from("/work/dir"),
            "echo hi",
            allow_network,
        )
        .unwrap()
        .1
    }

    #[test]
    fn the_workdir_is_the_only_writable_place() {
        let args = args_for(false);
        let joined = args.join(" ");
        assert!(joined.contains("--ro-bind / /"), "{joined}");
        assert!(joined.contains("--bind /work/dir /work/dir"), "{joined}");
        assert!(joined.contains("--tmpfs /tmp"), "{joined}");
    }

    #[test]
    fn the_private_tmp_is_mounted_before_the_workdir() {
        // The workdir lives under /tmp, and bwrap applies mounts in order —
        // so a tmpfs mounted afterwards hides it and nothing can run. This
        // ordering is load-bearing, not cosmetic.
        let args = args_for(false);
        let tmpfs = args.iter().position(|a| a == "--tmpfs").unwrap();
        let bind = args.iter().position(|a| a == "--bind").unwrap();
        assert!(tmpfs < bind, "tmpfs must come first: {args:?}");
    }

    #[test]
    fn the_network_is_denied_unless_the_document_granted_it() {
        // Silence is not consent, in the sandbox as in the capability model.
        assert!(!args_for(false).contains(&"--share-net".to_string()));
        assert!(args_for(true).contains(&"--share-net".to_string()));
        assert!(args_for(false).contains(&"--unshare-all".to_string()));
    }

    #[test]
    fn the_command_reaches_a_shell_unchanged() {
        // A cell's shell syntax must behave exactly as it does unsandboxed;
        // anything else makes the sandbox a second dialect to learn.
        let args = args_for(false);
        let tail: Vec<&String> = args.iter().rev().take(4).collect();
        assert_eq!(tail[0], "echo hi");
        assert_eq!(tail[1], "-c");
        assert_eq!(tail[2], "sh");
        assert_eq!(tail[3], "--");
    }

    #[test]
    fn a_cell_cannot_outlive_the_run_that_started_it() {
        assert!(args_for(false).contains(&"--die-with-parent".to_string()));
        assert!(args_for(false).contains(&"--new-session".to_string()));
    }

    #[test]
    fn the_seatbelt_profile_denies_by_default_and_grants_the_workdir() {
        let profile = seatbelt_profile(&PathBuf::from("/Users/x/work"), false);
        assert!(profile.contains("(deny default)"));
        assert!(profile.contains("(allow file-write* (subpath \"/Users/x/work\"))"));
        assert!(!profile.contains("(allow network*)"));
        assert!(seatbelt_profile(&PathBuf::from("/w"), true).contains("(allow network*)"));
    }

    #[test]
    fn no_sandbox_yields_no_command_rather_than_an_unconfined_one() {
        // The whole safety property: never silently run unsandboxed.
        assert!(wrap(Sandbox::None, &PathBuf::from("/w"), "echo hi", false).is_none());
    }
}
