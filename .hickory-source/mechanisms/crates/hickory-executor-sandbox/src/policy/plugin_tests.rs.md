<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-executor-sandbox/src/policy/plugin_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-cf906c1fed5e"></a>
## `crates/hickory-executor-sandbox/src/policy/plugin_tests.rs`

<a id="f-cf906c1fed5e-runtime_defaults_do_not_change_the_command_or_grant_network_in_bubblewrap"></a>
### `runtime_defaults_do_not_change_the_command_or_grant_network_in_bubblewrap`

<storycode:copy id="f-cf906c1fed5e-runtime_defaults_do_not_change_the_command_or_grant_network_in_bubblewrap">
use super::*;

#[test]
fn runtime_defaults_do_not_change_the_command_or_grant_network_in_bubblewrap() {
    let workdir = Path::new("/work/plugin-test");
    let command = "dotnet new console -n blah";
    let (_, args) = wrap(
        Sandbox::Bubblewrap,
        &Confinement {
            workdir,
            command,
            allow_network: false,
            profile: Profile::Cell,
            tmpdir: None,
            peers: &[],
            tools: &[],
        },
    )
    .unwrap();
    assert_eq!(args.last().unwrap(), command);
    assert!(!args.iter().any(|arg| arg == "--share-net"));
    let cli = args
        .windows(3)
        .find(|parts| parts[0] == "--setenv" && parts[1] == "DOTNET_CLI_HOME")
        .unwrap();
    assert_eq!(
        cli[2],
        workdir
            .join(".hick-runtime/csharp/dotnet")
            .to_string_lossy()
    );
}

</storycode:copy>
<a id="f-cf906c1fed5e-seatbelt_runtime_paths_are_quoted_and_do_not_expand_shell_syntax"></a>
### `seatbelt_runtime_paths_are_quoted_and_do_not_expand_shell_syntax`

<storycode:copy id="f-cf906c1fed5e-seatbelt_runtime_paths_are_quoted_and_do_not_expand_shell_syntax">
#[cfg(unix)]
#[test]
fn seatbelt_runtime_paths_are_quoted_and_do_not_expand_shell_syntax() {
    let dir = tempfile::Builder::new()
        .prefix("Hickory ' $() ; ")
        .tempdir()
        .unwrap();
    let (_, args) = wrap(
        Sandbox::Seatbelt,
        &Confinement {
            workdir: dir.path(),
            command: "printf '%s' \"$DOTNET_CLI_HOME\"",
            allow_network: false,
            profile: Profile::Cell,
            tmpdir: None,
            peers: &[],
            tools: &[],
        },
    )
    .unwrap();
    // Execute only the shell environment setup; the real sandbox and
    // its filesystem grants are exercised by tests/plugin_runtime.rs.
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(args.last().unwrap())
        .output()
        .unwrap();
    assert!(output.status.success());
    let expected = dir.path().join(".hick-runtime/csharp/dotnet");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        expected.to_string_lossy()
    );
    assert!(!args[1].contains("(allow network*)"));
}
</storycode:copy>

### Assemble `crates/hickory-executor-sandbox/src/policy/plugin_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-sandbox/src/policy/plugin_tests.rs"><storycode:paste select="#f-cf906c1fed5e-runtime_defaults_do_not_change_the_command_or_grant_network_in_bubblewrap"/><storycode:paste select="#f-cf906c1fed5e-seatbelt_runtime_paths_are_quoted_and_do_not_expand_shell_syntax"/></storycode:file>

</storycode:doc>
