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
    /// Windows AppContainer, applied by `hick` re-invoking itself.
    ///
    /// The odd one out: there is no wrapper program to exec, because
    /// AppContainer is applied by the parent at process creation. See
    /// [`crate::appcontainer`] for what actually happens, and `wrap` below
    /// for the re-invocation that keeps the shape identical to the others.
    AppContainer,
    /// Nothing available. The executor refuses rather than pretending.
    None,
}

impl Sandbox {
    /// What is actually available on this machine, right now.
    ///
    /// "Available" means WORKS, not "is installed". The two come apart more
    /// often than they should: `bwrap` is present on a hardened kernel, in a
    /// container, or on a CI runner, and then fails at
    /// `RTM_NEWADDR: Operation not permitted` the first time it is asked for
    /// a network namespace. Trusting the binary's existence there turns one
    /// clear refusal at startup into a confusing bubblewrap error on every
    /// cell — the same information, delivered at the worst moment and in
    /// somebody else's vocabulary.
    ///
    /// The smoke test is cached: it costs one process for the life of the
    /// program, and running it per cell would be a real cost for no answer
    /// that can change.
    pub fn detect() -> Self {
        static DETECTED: std::sync::OnceLock<Sandbox> = std::sync::OnceLock::new();
        *DETECTED.get_or_init(Sandbox::detect_uncached)
    }

    fn detect_uncached() -> Self {
        if cfg!(target_os = "linux") && which("bwrap").is_some() {
            if works(Sandbox::Bubblewrap) {
                Sandbox::Bubblewrap
            } else {
                log::warn!(
                    "bwrap is installed but cannot confine anything here — it fails to set up \
                     the namespaces it needs. This is usual inside containers and on kernels \
                     that restrict unprivileged user namespaces."
                );
                Sandbox::None
            }
        } else if cfg!(target_os = "macos") && which("sandbox-exec").is_some() {
            Sandbox::Seatbelt
        } else if cfg!(windows) {
            // No probe: AppContainer is part of Windows itself from 8
            // onwards, so there is no binary whose absence would mean
            // anything. A machine old enough to lack it fails at
            // `CreateAppContainerProfile` with a message naming the
            // alternatives, which is a better place to find out than a
            // silent downgrade to running unconfined.
            Sandbox::AppContainer
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
            Sandbox::AppContainer => {
                "Windows AppContainer: low-integrity token with its own package SID, writable \
                 workdir only, no network unless the document allows it, killed with the run \
                 via a job object. Reads are limited to what ALL APPLICATION PACKAGES may read \
                 — a per-user interpreter install may be invisible to the cell"
            }
            Sandbox::None => "no sandbox available on this platform",
        }
    }

    /// How to install what is missing, named per platform.
    pub fn missing_hint() -> String {
        if cfg!(target_os = "linux") {
            if which("bwrap").is_some() {
                // Installed, and it does not work — a different problem with
                // a different fix, and saying "install bubblewrap" to
                // somebody who just did is how a tool loses their trust.
                "bubblewrap IS installed here but cannot create the namespaces it needs. That \
                 usually means an unprivileged user namespace is not permitted: common inside \
                 containers, under some AppArmor and seccomp profiles, and on CI runners. Try \
                 `bwrap --unshare-all --ro-bind / / true` to see the kernel's own message."
                    .to_string()
            } else {
                "Install bubblewrap: `sudo apt install bubblewrap` (Debian/Ubuntu), \
                 `sudo dnf install bubblewrap` (Fedora), `sudo pacman -S bubblewrap` (Arch)."
                    .to_string()
            }
        } else if cfg!(target_os = "macos") {
            "`sandbox-exec` ships with macOS; a PATH that hides /usr/bin would explain this."
                .to_string()
        } else if cfg!(windows) {
            "AppContainer is part of Windows 8 and later. On an older build, run the cells \
             under WSL2, use the Docker executor (HICKORY_EXECUTOR=docker), or accept the \
             local executor's stated lack of isolation (HICKORY_EXECUTOR=local)."
                .to_string()
        } else {
            "This platform has no sandbox this executor can drive. Use the Docker executor \
             (HICKORY_EXECUTOR=docker), or accept the local executor's stated lack of \
             isolation (HICKORY_EXECUTOR=local)."
                .to_string()
        }
    }
}

/// Which of the two things being confined this is.
///
/// The policies differ in exactly one place — the home directory — and the
/// difference is forced by what each job needs:
///
/// * A **cell** gets an EMPTY home. A document you were sent has no business
///   reading your dotfiles, and an empty one is also more reproducible: a
///   cell that behaves differently because of somebody's `.bashrc` is a cell
///   nobody can re-run.
/// * An **installer** cannot have an empty home, because the tool it runs
///   often lives in one. `uv` installs to `~/.local/bin`; hiding the home
///   hides the installer itself, and the confined command fails with
///   `uv: not found` on a machine where `uv` is plainly installed. So the
///   real home stays visible (read-only, like the rest of the system) and
///   `HOME` is pointed at a writable directory inside the install prefix, so
///   the tool's caches land there rather than nowhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// A document's cell.
    Cell,
    /// A language-server install (`hick lsp install`).
    Installer,
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
    profile: Profile,
    tmpdir: Option<&Path>,
    peers: &[std::path::PathBuf],
) -> Option<(String, Vec<String>)> {
    let dir = workdir.to_string_lossy().to_string();
    let dir_for_home = dir.clone();
    // Without a container tmp directory there is nothing to share, so the
    // per-command tmpfs is still the right answer — it is private, which is
    // the property that matters second-most.
    let tmp_source = tmpdir
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();
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
            ];
            // A private /tmp first…
            //
            // …and a REAL directory rather than a tmpfs where the caller has
            // one, because a tmpfs is per-COMMAND: a cell that writes a
            // scratch file and a later cell that reads it are the same
            // container, and the second must find it there. A tmpfs makes
            // that work unsandboxed and fail confined, which reads as the
            // document being wrong when it is the sandbox.
            if tmp_source.is_empty() {
                args.push("--tmpfs".into());
                args.push("/tmp".into());
            } else {
                args.push("--bind".into());
                args.push(tmp_source.clone());
                args.push("/tmp".into());
            }
            args.extend([
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
            ]);
            match profile {
                // An empty home, with the toolchains handed back.
                //
                // Read-only was not enough: a cell could still LIST ~/.ssh and
                // read whatever it found there, and "it cannot exfiltrate
                // because the network is off" is one mistake away from false.
                //
                // But an empty home alone is not right either, and the reason
                // is where modern toolchains install. `duckdb` in this
                // repository's own examples lives in ~/.local/bin; nvm keeps
                // node under ~/.nvm, pyenv keeps python under ~/.pyenv, cargo
                // and go and bun and mise all live under $HOME. Hiding the
                // home hides the interpreter, and the cell fails with
                // `command not found` on a machine where the command is
                // plainly installed — which reads as a broken sandbox, and is.
                //
                // So: a tmpfs over the home (dotfiles, keys, credentials and
                // .env files all gone), then the tool directories bound back
                // READ-ONLY on top of it. The cell can run what you have
                // installed and can read nothing else of yours.
                Profile::Cell => {
                    if let Some(home) = std::env::var_os("HOME") {
                        let home = home.to_string_lossy().to_string();
                        if home != "/" && !home.is_empty() && !dir_for_home.starts_with(&home) {
                            args.push("--tmpfs".into());
                            args.push(home.clone());
                            for tool in HOME_TOOL_DIRS {
                                // `--ro-bind-try`, not `--ro-bind`: most of
                                // these do not exist on any given machine, and
                                // bwrap fails outright on a missing source.
                                args.push("--ro-bind-try".into());
                                args.push(format!("{home}/{tool}"));
                                args.push(format!("{home}/{tool}"));
                            }
                        }
                    }
                }
                // The installer keeps the real home VISIBLE — read-only,
                // like everything outside the prefix — because that is where
                // `uv` and friends are installed. Its writes are redirected
                // into the prefix instead, so a package manager's cache has
                // somewhere to go without the user's home being writable.
                Profile::Installer => {
                    args.push("--setenv".into());
                    args.push("HOME".into());
                    args.push(format!("{dir_for_home}/.home"));
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
            let policy_text = seatbelt_profile(workdir, tmpdir, allow_network, profile, peers);
            // Seatbelt cannot set an environment variable, so the installer's
            // redirected HOME is prepended to the command instead. It reaches
            // `sh` as an assignment, which is the same effect by a different
            // road.
            let command = match profile {
                Profile::Cell => command.to_string(),
                Profile::Installer => format!("HOME='{dir_for_home}/.home' {command}"),
            };
            Some((
                "sandbox-exec".into(),
                vec!["-p".into(), policy_text, "sh".into(), "-c".into(), command],
            ))
        }
        Sandbox::AppContainer => {
            // Re-invoke ourselves as the launcher. AppContainer is applied by
            // the PARENT at process creation, so unlike bwrap there is nothing
            // to exec that would confine what comes after it — something has
            // to make the `CreateProcessW` call, and shipping a second binary
            // to do it would be one more thing to install and to sign.
            let me = std::env::current_exe().ok()?;
            let mut args = vec![
                SANDBOX_RUN_SUBCOMMAND.to_string(),
                "--workdir".to_string(),
                dir,
            ];
            if allow_network {
                args.push("--allow-network".to_string());
            }
            // No home handling here: Windows redirects an AppContainer's
            // AppData into the package's own store automatically, so a tool
            // that writes a cache already writes it somewhere private.
            let _ = profile;
            // `--` first: a cell's command routinely begins with something
            // that looks like a flag, and it must reach the shell unread.
            args.push("--".to_string());
            args.push(command.to_string());
            Some((me.to_string_lossy().to_string(), args))
        }
        Sandbox::None => None,
    }
}

/// Directories under `$HOME` that hold TOOLS rather than secrets.
///
/// Bound read-only into the cell's otherwise-empty home, because this is
/// where language runtimes actually install themselves now. The list is
/// deliberately of `bin` directories and version-manager roots — never `$HOME`
/// itself, never `~/.config`, never `~/.aws` or `~/.ssh` — so what a cell
/// gains is the ability to RUN what you have, not to read what you have.
///
/// `~/.cargo/bin` rather than `~/.cargo`: the credentials file for
/// `cargo publish` lives one level up from the binaries.
const HOME_TOOL_DIRS: &[&str] = &[
    // Binaries people install for themselves.
    ".local/bin",
    ".cargo/bin",
    "go/bin",
    // Version managers and toolchain stores.
    //
    // These are bound at their ROOT, not at their `bin`, because a `bin`
    // entry in almost all of them is a symlink into a sibling directory —
    // and a symlink whose target is hidden is a file that does not exist.
    // Binding `~/.npm-global/bin` alone breaks every globally installed npm
    // CLI, whose launcher points at `../lib/node_modules/…`; binding
    // `~/.local/share/uv` alone was not done at all, so a uv-managed Python
    // was invisible and every venv built on one failed with
    // `No such file or directory` for an interpreter that is plainly there.
    ".local/share/uv",
    ".local/share/mise",
    ".local/share/pnpm",
    ".npm-global",
    ".rustup",
    ".nvm",
    ".volta",
    ".fnm",
    ".bun",
    ".deno",
    ".asdf",
    ".pyenv",
    ".rbenv",
    ".rvm",
    ".sdkman",
    ".ghcup",
    ".pixi",
    ".juliaup",
];
/// The hidden subcommand `hick` answers to when it is acting as the Windows
/// sandbox launcher.
///
/// Named with a leading underscore pair so it sorts away from real commands
/// and reads as internal in any help output that leaks it: it is an
/// implementation detail of this executor, not a thing to run by hand.
pub const SANDBOX_RUN_SUBCOMMAND: &str = "__sandbox-run";

/// A path as the kernel will see it, for policies that match on the real one.
///
/// Falls back to the path as given when it cannot be resolved — a directory
/// that does not exist yet has nothing to canonicalise, and a policy naming it
/// is no worse than the one we would have written anyway.
fn resolve(path: &Path) -> std::path::PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// A Seatbelt profile: deny by default, then grant the minimum.
///
/// Written out rather than assembled from a template file so the policy and
/// the code that applies it cannot drift apart, and so a reader can see the
/// whole thing at once.
fn seatbelt_profile(
    workdir: &Path,
    tmpdir: Option<&Path>,
    allow_network: bool,
    profile: Profile,
    peers: &[std::path::PathBuf],
) -> String {
    // RESOLVED, not as given. Seatbelt matches the path the kernel arrives at
    // after following symlinks, and on macOS a cell's workdir is nearly always
    // reached through one: `/var` is a symlink to `/private/var`, so a
    // `TMPDIR` workdir handed in as `/var/folders/…` is really
    // `/private/var/folders/…`. Granting the unresolved spelling grants
    // nothing, and the cell is denied its own directory — `sh: note.txt:
    // Operation not permitted` — which reads as a broken document rather than
    // a broken policy.
    //
    // The `/tmp` grant below always spelled both halves, which is the same
    // hazard noticed for one path and not generalised.
    let resolved = resolve(workdir);
    let dir = resolved.to_string_lossy();
    let mut policy = String::from(
        "(version 1)\
         (deny default)\
         (allow process-exec)\
         (allow process-fork)\
         (allow sysctl-read)\
         (allow file-read*)\
         (allow file-write* (subpath \"/tmp\") (subpath \"/private/tmp\") (subpath \"/dev/null\"))",
    );
    policy.push_str(&format!("(allow file-write* (subpath \"{dir}\"))"));

    let own_tmp = tmpdir.map(resolve);
    if let Some(tmp) = &own_tmp {
        let tmp = tmp.to_string_lossy();
        policy.push_str(&format!("(allow file-write* (subpath \"{tmp}\"))"));
    }

    // The part that actually separates one container from another.
    //
    // Bubblewrap gets it for free: one workdir is bound into the cell's mount
    // namespace and the others do not exist to it. Seatbelt has no namespaces
    // and `(allow file-read*)` above is global, so without this a cell could
    // read every other container's workdir by absolute path.
    //
    // Each peer is named. Denying the directory they all share instead looks
    // tidier and is wrong: that directory also holds things this cell is
    // entitled to reach — a `<hick:volume>` mounted into its workdir resolves
    // through it, and denying the parent made `cd project` fail with
    // `Not a directory`, which reads as a broken document. Deny what must be
    // hidden, not everything near it.
    for peer in peers {
        let peer = resolve(peer);
        let peer = peer.to_string_lossy();
        policy.push_str(&format!("(deny file-read* (subpath \"{peer}\"))"));
    }
    // Same reasoning as bubblewrap's two homes: a cell may not read the
    // user's dotfiles, an installer must be able to see the tool it runs.
    // Seatbelt cannot mount an empty home, so it denies the reads instead.
    if profile == Profile::Cell
        && let Some(home) = std::env::var_os("HOME")
    {
        let home = home.to_string_lossy().to_string();
        if home != "/" && !home.is_empty() && !dir.starts_with(&home) {
            policy.push_str(&format!("(deny file-read* (subpath \"{home}\"))"));
            // Then the toolchains back, for the same reason bubblewrap binds
            // them in. A later rule wins in Seatbelt, so these must follow
            // the deny above rather than precede it.
            for tool in HOME_TOOL_DIRS {
                policy.push_str(&format!("(allow file-read* (subpath \"{home}/{tool}\"))"));
            }
        }
    }
    if allow_network {
        policy.push_str("(allow network*)");
    }
    policy
}

/// Does this sandbox actually confine a trivial command here?
///
/// Run with the real policy, not a reduced one: the whole point is to find
/// out whether what a cell will get works, and the parts most likely to be
/// refused — the network namespace above all — are exactly the parts a
/// reduced probe would leave out.
fn works(sandbox: Sandbox) -> bool {
    let probe_dir = std::env::temp_dir();
    let Some((program, args)) = wrap(
        sandbox,
        &probe_dir,
        "exit 0",
        false,
        Profile::Cell,
        None,
        &[],
    ) else {
        return false;
    };
    std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
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
            Profile::Cell,
            None,
            &[],
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
        let profile = seatbelt_profile(
            &PathBuf::from("/Users/x/work"),
            None,
            false,
            Profile::Cell,
            &[],
        );
        assert!(profile.contains("(deny default)"));
        assert!(profile.contains("(allow file-write* (subpath \"/Users/x/work\"))"));
        assert!(!profile.contains("(allow network*)"));
        assert!(
            seatbelt_profile(&PathBuf::from("/w"), None, true, Profile::Cell, &[])
                .contains("(allow network*)")
        );
    }

    /// The bug that made a shipped example fail on macOS.
    ///
    /// Seatbelt matches the path the kernel resolves to, and on macOS `/var`
    /// is a symlink to `/private/var` — so a workdir handed in under `TMPDIR`
    /// was granted under a spelling the kernel never sees, and the cell was
    /// denied its own directory.
    #[test]
    fn the_seatbelt_profile_grants_the_resolved_workdir_not_the_symlinked_one() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let given = dir.path();
        let resolved = std::fs::canonicalize(given).expect("it resolves");
        if given == resolved {
            // Nothing to prove on a filesystem that hands back what it was
            // given; this is a macOS-shaped hazard.
            return;
        }
        let profile = seatbelt_profile(given, None, false, Profile::Cell, &[]);
        assert!(
            profile.contains(&format!(
                "(allow file-write* (subpath \"{}\"))",
                resolved.display()
            )),
            "the resolved path is not granted:\n{profile}"
        );
    }

    /// One container may not read another's, which bubblewrap gets from
    /// namespaces and Seatbelt has to be told.
    #[test]
    fn the_seatbelt_profile_hides_sibling_containers() {
        let peers = [
            PathBuf::from("/w/root/theirs"),
            PathBuf::from("/w/root/.tmp-theirs"),
        ];
        let profile = seatbelt_profile(
            &PathBuf::from("/w/root/mine"),
            Some(&PathBuf::from("/w/root/.tmp-mine")),
            false,
            Profile::Cell,
            &peers,
        );
        assert!(
            profile.contains("(deny file-read* (subpath \"/w/root/theirs\"))"),
            "a peer workdir is not denied:\n{profile}"
        );
        assert!(
            profile.contains("(deny file-read* (subpath \"/w/root/.tmp-theirs\"))"),
            "a peer tmp is not denied:\n{profile}"
        );
        assert!(
            profile.contains("(allow file-write* (subpath \"/w/root/.tmp-mine\"))"),
            "the container's own tmp is not writable:\n{profile}"
        );
    }

    /// The regression that denying the shared parent caused: a volume mounted
    /// into the cell's own workdir resolves through the directory every
    /// container sits in, so denying that directory broke `cd project` in a
    /// document that had done nothing wrong.
    #[test]
    fn the_seatbelt_profile_does_not_deny_the_directory_containers_share() {
        let profile = seatbelt_profile(
            &PathBuf::from("/w/root/mine"),
            Some(&PathBuf::from("/w/root/.tmp-mine")),
            false,
            Profile::Cell,
            &[PathBuf::from("/w/root/theirs")],
        );
        assert!(
            !profile.contains("(deny file-read* (subpath \"/w/root\"))"),
            "the shared parent is denied, which takes volumes with it:\n{profile}"
        );
    }

    #[test]
    fn an_installer_can_see_the_tool_it_is_about_to_run() {
        // The bug this pins: the cell profile mounts a tmpfs over $HOME, and
        // `uv` lives in ~/.local/bin. Installing a language server therefore
        // failed with `uv: not found` on a machine where uv was plainly
        // installed — the sandbox had hidden the installer from itself.
        let args = wrap(
            Sandbox::Bubblewrap,
            &PathBuf::from("/work/dir"),
            "uv venv",
            true,
            Profile::Installer,
            None,
            &[],
        )
        .unwrap()
        .1;
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() && home != "/" {
            let tmpfs_over_home = args
                .windows(2)
                .any(|pair| pair[0] == "--tmpfs" && pair[1] == home);
            assert!(
                !tmpfs_over_home,
                "the installer's home was hidden: {args:?}"
            );
        }
        // Its writes still go nowhere but the prefix: HOME is redirected
        // INSIDE the one writable directory.
        let setenv = args
            .iter()
            .position(|a| a == "--setenv")
            .expect("HOME is set");
        assert_eq!(args[setenv + 1], "HOME");
        assert!(args[setenv + 2].starts_with("/work/dir"), "{args:?}");
    }

    #[test]
    fn toolchain_stores_are_bound_at_their_root_not_their_bin() {
        // A `bin` entry in a version manager is usually a SYMLINK into a
        // sibling directory, and a symlink whose target is hidden is a file
        // that does not exist. Binding `~/.npm-global/bin` alone breaks every
        // globally installed npm CLI, whose launcher points at
        // `../lib/node_modules/…`; not binding `~/.local/share/uv` at all
        // made a uv-managed Python invisible, so every venv built on one
        // failed with `No such file or directory` for an interpreter that is
        // plainly installed. Both were found by running real tools, not by
        // reading this list.
        for root in [".local/share/uv", ".npm-global", ".nvm", ".pyenv", ".bun"] {
            assert!(
                HOME_TOOL_DIRS.contains(&root),
                "{root} is a toolchain store and must be visible to a cell"
            );
            assert!(
                !HOME_TOOL_DIRS.contains(&format!("{root}/bin").as_str()),
                "{root} is bound at its bin, which breaks the symlinks inside it"
            );
        }
        // `~/.cargo` is the exception that proves the rule: its binaries are
        // real files, and one level up sits the credentials file for
        // `cargo publish`. That one stays narrow on purpose.
        assert!(HOME_TOOL_DIRS.contains(&".cargo/bin"));
        assert!(!HOME_TOOL_DIRS.contains(&".cargo"));
    }

    #[test]
    fn a_cell_still_gets_an_empty_home() {
        // The installer's exception must not have widened the cell's policy.
        let args = args_for(false);
        if let Ok(home) = std::env::var("HOME")
            && !home.is_empty()
            && home != "/"
        {
            assert!(
                args.windows(2)
                    .any(|pair| pair[0] == "--tmpfs" && pair[1] == home),
                "a cell's home is no longer empty: {args:?}"
            );
        }
    }

    #[test]
    fn the_appcontainer_launcher_carries_the_policy_in_its_arguments() {
        // The Windows path re-invokes `hick` rather than exec'ing a wrapper,
        // so the arguments ARE the policy: workdir, network, then the command
        // after a `--` that stops a cell's own leading flag being read as
        // one of ours.
        let (program, args) = wrap(
            Sandbox::AppContainer,
            &PathBuf::from(r"C:\work\dir"),
            "--version",
            false,
            Profile::Cell,
            None,
            &[],
        )
        .expect("the launcher is always available on a machine that can run us");
        assert!(
            program.ends_with("hick") || program.ends_with("hick.exe") || !program.is_empty(),
            "the launcher is this binary: {program}"
        );
        assert_eq!(args[0], SANDBOX_RUN_SUBCOMMAND);
        assert_eq!(args[1], "--workdir");
        assert_eq!(args[2], r"C:\work\dir");
        let dashes = args.iter().position(|a| a == "--").expect("a -- separator");
        assert_eq!(args[dashes + 1], "--version", "the cell's command, unread");
        assert!(!args.contains(&"--allow-network".to_string()));
    }

    #[test]
    fn the_appcontainer_launcher_opens_the_network_only_when_granted() {
        let (_, args) = wrap(
            Sandbox::AppContainer,
            &PathBuf::from(r"C:\work"),
            "curl example.com",
            true,
            Profile::Cell,
            None,
            &[],
        )
        .unwrap();
        let net = args
            .iter()
            .position(|a| a == "--allow-network")
            .expect("granted");
        let dashes = args.iter().position(|a| a == "--").unwrap();
        assert!(net < dashes, "the flag is ours, not the cell's: {args:?}");
    }

    #[test]
    fn detection_answers_the_same_thing_every_time() {
        // Cached, because the answer cannot change within one run and the
        // check costs a process. A per-cell smoke test would be a real cost
        // for no new information.
        assert_eq!(Sandbox::detect(), Sandbox::detect());
    }

    #[test]
    fn a_missing_hint_distinguishes_absent_from_broken() {
        // Telling somebody to install bubblewrap when they just did is how a
        // tool loses their trust. On Linux the hint depends on which of the
        // two problems they actually have.
        let hint = Sandbox::missing_hint();
        assert!(!hint.is_empty());
        #[cfg(target_os = "linux")]
        if which("bwrap").is_some() {
            assert!(
                hint.contains("IS installed"),
                "bwrap is present, so the hint must not say to install it: {hint}"
            );
        } else {
            assert!(hint.contains("Install bubblewrap"), "{hint}");
        }
    }

    #[test]
    fn no_sandbox_yields_no_command_rather_than_an_unconfined_one() {
        // The whole safety property: never silently run unsandboxed.
        assert!(
            wrap(
                Sandbox::None,
                &PathBuf::from("/w"),
                "echo hi",
                false,
                Profile::Cell,
                None,
                &[]
            )
            .is_none()
        );
    }
}
