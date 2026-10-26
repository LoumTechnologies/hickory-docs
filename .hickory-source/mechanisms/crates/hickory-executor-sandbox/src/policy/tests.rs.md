<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-executor-sandbox/src/policy/tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-02e5e9c0f7b7"></a>
## `crates/hickory-executor-sandbox/src/policy/tests.rs`

<a id="f-02e5e9c0f7b7-args_for"></a>
### `args_for`

<storycode:copy id="f-02e5e9c0f7b7-args_for">
use super::*;
use std::path::PathBuf;

fn args_for(allow_network: bool) -> Vec<String> {
    wrap(
        Sandbox::Bubblewrap,
        &Confinement {
            workdir: &PathBuf::from("/work/dir"),
            command: "echo hi",
            allow_network,
            profile: Profile::Cell,
            tmpdir: None,
            peers: &[],
            tools: &[],
        },
    )
    .unwrap()
    .1
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_workdir_is_the_only_writable_place"></a>
### `the_workdir_is_the_only_writable_place`

<storycode:copy id="f-02e5e9c0f7b7-the_workdir_is_the_only_writable_place">
#[test]
fn the_workdir_is_the_only_writable_place() {
    let args = args_for(false);
    let joined = args.join(" ");
    assert!(joined.contains("--ro-bind / /"), "{joined}");
    assert!(joined.contains("--bind /work/dir /work/dir"), "{joined}");
    assert!(joined.contains("--tmpfs /tmp"), "{joined}");
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_private_tmp_is_mounted_before_the_workdir"></a>
### `the_private_tmp_is_mounted_before_the_workdir`

<storycode:copy id="f-02e5e9c0f7b7-the_private_tmp_is_mounted_before_the_workdir">
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

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_network_is_denied_unless_the_document_granted_it"></a>
### `the_network_is_denied_unless_the_document_granted_it`

<storycode:copy id="f-02e5e9c0f7b7-the_network_is_denied_unless_the_document_granted_it">
#[test]
fn the_network_is_denied_unless_the_document_granted_it() {
    // Silence is not consent, in the sandbox as in the capability model.
    assert!(!args_for(false).contains(&"--share-net".to_string()));
    assert!(args_for(true).contains(&"--share-net".to_string()));
    assert!(args_for(false).contains(&"--unshare-all".to_string()));
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_command_reaches_a_shell_unchanged"></a>
### `the_command_reaches_a_shell_unchanged`

<storycode:copy id="f-02e5e9c0f7b7-the_command_reaches_a_shell_unchanged">
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

</storycode:copy>
<a id="f-02e5e9c0f7b7-a_tool_name_cannot_become_shell_syntax"></a>
### `a_tool_name_cannot_become_shell_syntax`

<storycode:copy id="f-02e5e9c0f7b7-a_tool_name_cannot_become_shell_syntax">
#[test]
fn a_tool_name_cannot_become_shell_syntax() {
    // A tool name is author-supplied text and the probe runs in a shell.
    let command = probe_command(Sandbox::Bubblewrap, "evil'; rm -rf /; echo '")
        .expect("bubblewrap can be asked");
    assert!(command.starts_with("command -v '"), "{command}");
    assert!(!command.contains("; rm -rf /; echo ;"), "{command}");
    // The dangerous text survives INSIDE the quotes, which is the point:
    // it is looked up as a (very odd) program name, not executed.
    assert!(command.contains(r"'\''"), "{command}");
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-a_name_cmd_cannot_be_asked_about_safely_is_refused_rather_than_guessed"></a>
### `a_name_cmd_cannot_be_asked_about_safely_is_refused_rather_than_guessed`

<storycode:copy id="f-02e5e9c0f7b7-a_name_cmd_cannot_be_asked_about_safely_is_refused_rather_than_guessed">
#[test]
fn a_name_cmd_cannot_be_asked_about_safely_is_refused_rather_than_guessed() {
    // cmd has no quoting that survives an embedded quote the way sh's
    // does, so there is no safe spelling. `None` means "not asked", which
    // the caller reads as not blocking — never as missing.
    assert_eq!(probe_command(Sandbox::AppContainer, "ev\"il"), None);
    assert_eq!(
        probe_command(Sandbox::AppContainer, "python").as_deref(),
        Some("where \"python\"")
    );
    assert_eq!(probe_command(Sandbox::None, "python"), None);
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-a_cells_command_is_one_argv_element_on_every_sandbox"></a>
### `a_cells_command_is_one_argv_element_on_every_sandbox`

<storycode:copy id="f-02e5e9c0f7b7-a_cells_command_is_one_argv_element_on_every_sandbox">
#[test]
fn a_cells_command_is_one_argv_element_on_every_sandbox() {
    // The property that replaced the quoting: `wrap` hands back a program
    // and its arguments, and the cell's command is ONE of them, byte for
    // byte. Nothing downstream re-splits it, so there is no shell dialect
    // between the author and their command.
    //
    // This is the regression guard for a real escape. When the confined
    // argv was flattened into a line for an outer `cmd.exe`, a cell
    // running `echo hello > note.txt && cat note.txt` had the redirect
    // performed by that outer shell and its second half run UNCONFINED.
    const CELL: &str = "echo hello > note.txt && cat note.txt";

    let bwrap = wrap(
        Sandbox::Bubblewrap,
        &Confinement {
            workdir: &PathBuf::from("/work"),
            command: CELL,
            allow_network: false,
            profile: Profile::Cell,
            tmpdir: None,
            peers: &[],
            tools: &[],
        },
    )
    .expect("bubblewrap composes without touching the filesystem")
    .1;
    assert_eq!(
        bwrap.iter().filter(|a| a.as_str() == CELL).count(),
        1,
        "the cell appears once, unmangled: {bwrap:?}"
    );

    let appcontainer = appcontainer_args("C:\\work".to_string(), CELL, false);
    assert_eq!(appcontainer.last().map(String::as_str), Some(CELL));
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-a_cell_cannot_outlive_the_run_that_started_it"></a>
### `a_cell_cannot_outlive_the_run_that_started_it`

<storycode:copy id="f-02e5e9c0f7b7-a_cell_cannot_outlive_the_run_that_started_it">
#[test]
fn a_cell_cannot_outlive_the_run_that_started_it() {
    assert!(args_for(false).contains(&"--die-with-parent".to_string()));
    assert!(args_for(false).contains(&"--new-session".to_string()));
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_seatbelt_profile_denies_by_default_and_grants_the_workdir"></a>
### `the_seatbelt_profile_denies_by_default_and_grants_the_workdir`

<storycode:copy id="f-02e5e9c0f7b7-the_seatbelt_profile_denies_by_default_and_grants_the_workdir">
#[test]
fn the_seatbelt_profile_denies_by_default_and_grants_the_workdir() {
    let profile = seatbelt_profile(
        &PathBuf::from("/Users/x/work"),
        None,
        false,
        Profile::Cell,
        &[],
        &[],
    );
    assert!(profile.contains("(deny default)"));
    assert!(profile.contains("(allow file-write* (subpath \"/Users/x/work\"))"));
    assert!(!profile.contains("(allow network*)"));
    assert!(
        seatbelt_profile(&PathBuf::from("/w"), None, true, Profile::Cell, &[], &[])
            .contains("(allow network*)")
    );
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_seatbelt_profile_grants_the_resolved_workdir_not_the_symlinked_one"></a>
### `the_seatbelt_profile_grants_the_resolved_workdir_not_the_symlinked_one`

The bug that made a shipped example fail on macOS.
Seatbelt matches the path the kernel resolves to, and on macOS `/var`
is a symlink to `/private/var` — so a workdir handed in under `TMPDIR`
was granted under a spelling the kernel never sees, and the cell was
denied its own directory.

<storycode:copy id="f-02e5e9c0f7b7-the_seatbelt_profile_grants_the_resolved_workdir_not_the_symlinked_one">
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
    let profile = seatbelt_profile(given, None, false, Profile::Cell, &[], &[]);
    assert!(
        profile.contains(&format!(
            "(allow file-write* (subpath \"{}\"))",
            resolved.display()
        )),
        "the resolved path is not granted:\n{profile}"
    );
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_seatbelt_profile_hides_sibling_containers"></a>
### `the_seatbelt_profile_hides_sibling_containers`

One container may not read another's, which bubblewrap gets from
namespaces and Seatbelt has to be told.

<storycode:copy id="f-02e5e9c0f7b7-the_seatbelt_profile_hides_sibling_containers">
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
        &[],
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

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_seatbelt_profile_does_not_deny_the_directory_containers_share"></a>
### `the_seatbelt_profile_does_not_deny_the_directory_containers_share`

The regression that denying the shared parent caused: a volume mounted
into the cell's own workdir resolves through the directory every
container sits in, so denying that directory broke `cd project` in a
document that had done nothing wrong.

<storycode:copy id="f-02e5e9c0f7b7-the_seatbelt_profile_does_not_deny_the_directory_containers_share">
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
        &[],
    );
    assert!(
        !profile.contains("(deny file-read* (subpath \"/w/root\"))"),
        "the shared parent is denied, which takes volumes with it:\n{profile}"
    );
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-an_installer_can_see_the_tool_it_is_about_to_run"></a>
### `an_installer_can_see_the_tool_it_is_about_to_run`

<storycode:copy id="f-02e5e9c0f7b7-an_installer_can_see_the_tool_it_is_about_to_run">
#[test]
fn an_installer_can_see_the_tool_it_is_about_to_run() {
    // The bug this pins: the cell profile mounts a tmpfs over $HOME, and
    // `uv` lives in ~/.local/bin. Installing a language server therefore
    // failed with `uv: not found` on a machine where uv was plainly
    // installed — the sandbox had hidden the installer from itself.
    let args = wrap(
        Sandbox::Bubblewrap,
        &Confinement {
            workdir: &PathBuf::from("/work/dir"),
            command: "uv venv",
            allow_network: true,
            profile: Profile::Installer,
            tmpdir: None,
            peers: &[],
            tools: &[],
        },
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

</storycode:copy>
<a id="f-02e5e9c0f7b7-toolchain_stores_are_bound_at_their_root_not_their_bin"></a>
### `toolchain_stores_are_bound_at_their_root_not_their_bin`

<storycode:copy id="f-02e5e9c0f7b7-toolchain_stores_are_bound_at_their_root_not_their_bin">
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

</storycode:copy>
<a id="f-02e5e9c0f7b7-a_cell_still_gets_an_empty_home"></a>
### `a_cell_still_gets_an_empty_home`

<storycode:copy id="f-02e5e9c0f7b7-a_cell_still_gets_an_empty_home">
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

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_appcontainer_launcher_carries_the_policy_in_its_arguments"></a>
### `the_appcontainer_launcher_carries_the_policy_in_its_arguments`

<storycode:copy id="f-02e5e9c0f7b7-the_appcontainer_launcher_carries_the_policy_in_its_arguments">
#[test]
fn the_appcontainer_launcher_carries_the_policy_in_its_arguments() {
    // The Windows path re-invokes `hick` rather than exec'ing a wrapper,
    // so the arguments ARE the policy: workdir, network, then the command
    // after a `--` that stops a cell's own leading flag being read as
    // one of ours.
    let args = appcontainer_args(r"C:\work\dir".to_string(), "--version", false);
    assert_eq!(args[0], SANDBOX_RUN_SUBCOMMAND);
    assert_eq!(args[1], "--workdir");
    assert_eq!(args[2], r"C:\work\dir");
    let dashes = args.iter().position(|a| a == "--").expect("a -- separator");
    assert_eq!(args[dashes + 1], "--version", "the cell's command, unread");
    assert!(!args.contains(&"--allow-network".to_string()));
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_appcontainer_launcher_opens_the_network_only_when_granted"></a>
### `the_appcontainer_launcher_opens_the_network_only_when_granted`

<storycode:copy id="f-02e5e9c0f7b7-the_appcontainer_launcher_opens_the_network_only_when_granted">
#[test]
fn the_appcontainer_launcher_opens_the_network_only_when_granted() {
    let args = appcontainer_args(r"C:\work".to_string(), "curl example.com", true);
    let net = args
        .iter()
        .position(|a| a == "--allow-network")
        .expect("granted");
    let dashes = args.iter().position(|a| a == "--").unwrap();
    assert!(net < dashes, "the flag is ours, not the cell's: {args:?}");
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-detection_answers_the_same_thing_every_time"></a>
### `detection_answers_the_same_thing_every_time`

<storycode:copy id="f-02e5e9c0f7b7-detection_answers_the_same_thing_every_time">
#[test]
fn detection_answers_the_same_thing_every_time() {
    // Cached, because the answer cannot change within one run and the
    // check costs a process. A per-cell smoke test would be a real cost
    // for no new information.
    assert_eq!(Sandbox::detect(), Sandbox::detect());
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-a_missing_hint_distinguishes_absent_from_broken"></a>
### `a_missing_hint_distinguishes_absent_from_broken`

<storycode:copy id="f-02e5e9c0f7b7-a_missing_hint_distinguishes_absent_from_broken">
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

</storycode:copy>
<a id="f-02e5e9c0f7b7-no_sandbox_yields_no_command_rather_than_an_unconfined_one"></a>
### `no_sandbox_yields_no_command_rather_than_an_unconfined_one`

<storycode:copy id="f-02e5e9c0f7b7-no_sandbox_yields_no_command_rather_than_an_unconfined_one">
#[test]
fn no_sandbox_yields_no_command_rather_than_an_unconfined_one() {
    // The whole safety property: never silently run unsandboxed.
    assert!(
        wrap(
            Sandbox::None,
            &Confinement {
                workdir: &PathBuf::from("/w"),
                command: "echo hi",
                allow_network: false,
                profile: Profile::Cell,
                tmpdir: None,
                peers: &[],
                tools: &[],
            },
        )
        .is_none()
    );
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-launcher_exe"></a>
### `launcher_exe`

The launcher name on the platform the resolution is compiled for.

<storycode:copy id="f-02e5e9c0f7b7-launcher_exe">
/// The launcher name on the platform the resolution is compiled for.
fn launcher_exe() -> &'static str {
    if cfg!(windows) { "hick.exe" } else { "hick" }
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-the_launcher_is_ourselves_when_we_are_the_cli"></a>
### `the_launcher_is_ourselves_when_we_are_the_cli`

<storycode:copy id="f-02e5e9c0f7b7-the_launcher_is_ourselves_when_we_are_the_cli">
#[test]
fn the_launcher_is_ourselves_when_we_are_the_cli() {
    let me = std::path::PathBuf::from(format!("/opt/hickory/{}", launcher_exe()));
    assert_eq!(launcher_from(None, &me), Some(me));
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-a_program_that_is_not_the_cli_finds_the_cli_beside_it"></a>
### `a_program_that_is_not_the_cli_finds_the_cli_beside_it`

<storycode:copy id="f-02e5e9c0f7b7-a_program_that_is_not_the_cli_finds_the_cli_beside_it">
#[test]
fn a_program_that_is_not_the_cli_finds_the_cli_beside_it() {
    // The desktop app is the real case: `Hickory Docs.exe` cannot answer
    // `__sandbox-run`, so re-invoking it confines nothing.
    let dir = tempfile::tempdir().expect("a temp dir");
    let hick = dir.path().join(launcher_exe());
    std::fs::write(&hick, b"").expect("a file");
    let app = dir.path().join("Hickory Docs");
    assert_eq!(launcher_from(None, &app), Some(hick));
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-a_test_binary_finds_the_cli_one_directory_up"></a>
### `a_test_binary_finds_the_cli_one_directory_up`

<storycode:copy id="f-02e5e9c0f7b7-a_test_binary_finds_the_cli_one_directory_up">
#[test]
fn a_test_binary_finds_the_cli_one_directory_up() {
    // cargo builds integration tests into `target/debug/deps/` and the CLI
    // into `target/debug/`, so the launcher is never a sibling there.
    let dir = tempfile::tempdir().expect("a temp dir");
    let deps = dir.path().join("deps");
    std::fs::create_dir(&deps).expect("a deps dir");
    let hick = dir.path().join(launcher_exe());
    std::fs::write(&hick, b"").expect("a file");
    let test_binary = deps.join("confinement-9886057035e4c123");
    assert_eq!(launcher_from(None, &test_binary), Some(hick));
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-there_is_no_launcher_when_no_cli_is_anywhere_near"></a>
### `there_is_no_launcher_when_no_cli_is_anywhere_near`

<storycode:copy id="f-02e5e9c0f7b7-there_is_no_launcher_when_no_cli_is_anywhere_near">
#[test]
fn there_is_no_launcher_when_no_cli_is_anywhere_near() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let app = dir.path().join("Hickory Docs");
    assert_eq!(launcher_from(None, &app), None);
}

</storycode:copy>
<a id="f-02e5e9c0f7b7-an_explicit_launcher_wins_and_must_exist"></a>
### `an_explicit_launcher_wins_and_must_exist`

<storycode:copy id="f-02e5e9c0f7b7-an_explicit_launcher_wins_and_must_exist">
#[test]
fn an_explicit_launcher_wins_and_must_exist() {
    let dir = tempfile::tempdir().expect("a temp dir");
    let chosen = dir.path().join("somewhere-else");
    std::fs::write(&chosen, b"").expect("a file");
    let me = std::path::PathBuf::from(format!("/opt/hickory/{}", launcher_exe()));
    assert_eq!(
        launcher_from(Some(chosen.clone().into_os_string()), &me),
        Some(chosen),
    );
    // A path that names nothing is a mistake worth surfacing, not a
    // silent fall-back to a binary the user did not choose.
    assert_eq!(
        launcher_from(Some(dir.path().join("absent").into_os_string()), &me),
        None,
    );
}
</storycode:copy>

### Assemble `crates/hickory-executor-sandbox/src/policy/tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-sandbox/src/policy/tests.rs"><storycode:paste select="#f-02e5e9c0f7b7-args_for"/><storycode:paste select="#f-02e5e9c0f7b7-the_workdir_is_the_only_writable_place"/><storycode:paste select="#f-02e5e9c0f7b7-the_private_tmp_is_mounted_before_the_workdir"/><storycode:paste select="#f-02e5e9c0f7b7-the_network_is_denied_unless_the_document_granted_it"/><storycode:paste select="#f-02e5e9c0f7b7-the_command_reaches_a_shell_unchanged"/><storycode:paste select="#f-02e5e9c0f7b7-a_tool_name_cannot_become_shell_syntax"/><storycode:paste select="#f-02e5e9c0f7b7-a_name_cmd_cannot_be_asked_about_safely_is_refused_rather_than_guessed"/><storycode:paste select="#f-02e5e9c0f7b7-a_cells_command_is_one_argv_element_on_every_sandbox"/><storycode:paste select="#f-02e5e9c0f7b7-a_cell_cannot_outlive_the_run_that_started_it"/><storycode:paste select="#f-02e5e9c0f7b7-the_seatbelt_profile_denies_by_default_and_grants_the_workdir"/><storycode:paste select="#f-02e5e9c0f7b7-the_seatbelt_profile_grants_the_resolved_workdir_not_the_symlinked_one"/><storycode:paste select="#f-02e5e9c0f7b7-the_seatbelt_profile_hides_sibling_containers"/><storycode:paste select="#f-02e5e9c0f7b7-the_seatbelt_profile_does_not_deny_the_directory_containers_share"/><storycode:paste select="#f-02e5e9c0f7b7-an_installer_can_see_the_tool_it_is_about_to_run"/><storycode:paste select="#f-02e5e9c0f7b7-toolchain_stores_are_bound_at_their_root_not_their_bin"/><storycode:paste select="#f-02e5e9c0f7b7-a_cell_still_gets_an_empty_home"/><storycode:paste select="#f-02e5e9c0f7b7-the_appcontainer_launcher_carries_the_policy_in_its_arguments"/><storycode:paste select="#f-02e5e9c0f7b7-the_appcontainer_launcher_opens_the_network_only_when_granted"/><storycode:paste select="#f-02e5e9c0f7b7-detection_answers_the_same_thing_every_time"/><storycode:paste select="#f-02e5e9c0f7b7-a_missing_hint_distinguishes_absent_from_broken"/><storycode:paste select="#f-02e5e9c0f7b7-no_sandbox_yields_no_command_rather_than_an_unconfined_one"/><storycode:paste select="#f-02e5e9c0f7b7-launcher_exe"/><storycode:paste select="#f-02e5e9c0f7b7-the_launcher_is_ourselves_when_we_are_the_cli"/><storycode:paste select="#f-02e5e9c0f7b7-a_program_that_is_not_the_cli_finds_the_cli_beside_it"/><storycode:paste select="#f-02e5e9c0f7b7-a_test_binary_finds_the_cli_one_directory_up"/><storycode:paste select="#f-02e5e9c0f7b7-there_is_no_launcher_when_no_cli_is_anywhere_near"/><storycode:paste select="#f-02e5e9c0f7b7-an_explicit_launcher_wins_and_must_exist"/></storycode:file>

</storycode:doc>
