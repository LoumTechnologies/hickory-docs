use super::{Session, android_device, cmd, ios_device};
use anyhow::Result;
use std::path::PathBuf;
use tokio::process::Command;

pub(super) fn mobile_reload_command(platform: &str, target: &str) -> Command {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let supervisor = root.join(if cfg!(windows) {
        "target/dev-supervisor.exe"
    } else {
        "target/dev-supervisor"
    });
    let mut command = cmd(
        supervisor,
        &[
            "cargo",
            "run",
            "--quiet",
            "--locked",
            "-p",
            "hickory-cli",
            "--example",
            "simple_dev",
            "--",
            platform,
            "--session-child",
        ],
    );
    command.env(
        if platform == "android" {
            "ANDROID_SERIAL"
        } else {
            "HICKORY_IOS_DEVICE"
        },
        target,
    );
    command
}

pub(super) async fn supervise_mobile(platform: &str, hosts: &mut Session) -> Result<()> {
    let target = if platform == "android" {
        android_device(hosts).await?.1
    } else {
        ios_device(hosts).await?
    };
    println!(
        "Watching commits: {platform} app updates stay on {target}; Ctrl-C stops the session."
    );
    // The shared supervisor rebuilds this executable and its engine on every
    // commit. Only the host owns emulator/simulator lifetime across revisions.
    hosts
        .run(&mut mobile_reload_command(platform, &target))
        .await
}
