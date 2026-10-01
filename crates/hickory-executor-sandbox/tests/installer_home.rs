#![cfg(unix)]

use hickory_executor_sandbox::policy::{self, Confinement, Profile, Sandbox};

#[test]
fn a_compound_installer_keeps_its_home_for_every_command() {
    let dir = tempfile::Builder::new()
        .prefix("hick install's ")
        .tempdir()
        .unwrap();
    let (program, args) = policy::wrap(
        Sandbox::Seatbelt,
        &Confinement {
            workdir: dir.path(),
            command: "sh -c 'printf \"%s\\n\" \"$HOME\"' && sh -c 'printf \"%s\\n\" \"$HOME\"'",
            allow_network: true,
            profile: Profile::Installer,
            tmpdir: None,
            peers: &[],
            tools: &[],
        },
    )
    .unwrap();
    // Execute the generated shell on every Unix platform; on macOS also
    // run it through Seatbelt to check the real confinement path.
    let output = std::process::Command::new("sh")
        .args(&args[3..])
        .output()
        .unwrap();
    assert!(output.status.success());
    let home = dir.path().join(".home");
    let expected = format!("{0}\n{0}\n", home.display());
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    if cfg!(target_os = "macos") {
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    }
}
