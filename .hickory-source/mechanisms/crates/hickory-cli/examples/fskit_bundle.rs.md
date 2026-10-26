<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/examples/fskit_bundle.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fb890d5279a3"></a>
## `crates/hickory-cli/examples/fskit_bundle.rs`

Related contracts: [A protected workspace save is validated against the revision it read](../../../../guarantees/agent/the-mounted-workspace-keeps-source-and-output-together/contract.md).

<a id="f-fb890d5279a3-main"></a>
### `main`

Build the native frontend with Apple's SDK. Also used before Tauri bundling,
so the extension is signed and embedded BEFORE the containing app is signed.

<storycode:copy id="f-fb890d5279a3-main">
//! Build the native frontend with Apple's SDK. Also used before Tauri bundling,
//! so the extension is signed and embedded BEFORE the containing app is signed.
use anyhow::{Context, Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
fn main() -> Result<()> {
    ensure!(
        cfg!(target_os = "macos"),
        "FSKit builds require macOS and full Xcode"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let output = root.join(".dev/fskit/HickoryWorkspace.appex");
    let contents = output.join("Contents");
    std::fs::create_dir_all(contents.join("MacOS"))?;
    let developer = developer_dir()?;
    let sdk = output_of(
        Command::new("/usr/bin/xcrun")
            .env("DEVELOPER_DIR", &developer)
            .arg("--show-sdk-path"),
    )?;
    let arch = std::env::args()
        .nth(1)
        .unwrap_or_else(|| std::env::consts::ARCH.into());
    let arch = match arch.as_str() {
        "aarch64" | "aarch64-apple-darwin" | "arm64" => "arm64",
        "x86_64" | "x86_64-apple-darwin" => "x86_64",
        _ => anyhow::bail!("unsupported FSKit build target {arch}"),
    };
    let source = root.join("apps/desktop/fskit");
    run(Command::new("/usr/bin/xcrun")
        .env("DEVELOPER_DIR", developer)
        .arg("swiftc")
        .args([
            "-sdk",
            &sdk,
            "-target",
            &format!("{arch}-apple-macos26.0"),
            "-parse-as-library",
            "-application-extension",
            "-warnings-as-errors",
            "-O",
            "-o",
        ])
        .arg(contents.join("MacOS/HickoryWorkspace"))
        .arg(source.join("WorkspaceExtension.swift"))
        .arg(source.join("WorkspaceVolume.swift"))
        .args(["-framework", "FSKit", "-framework", "ExtensionFoundation"]))?;
    std::fs::copy(source.join("Info.plist"), contents.join("Info.plist"))?;
    if let Some(profile) = std::env::var_os("HICKORY_FSKIT_PROFILE") {
        std::fs::copy(profile, contents.join("embedded.provisionprofile"))?;
    }
    let identity = std::env::var("APPLE_SIGNING_IDENTITY").unwrap_or_else(|_| "-".into());
    if identity != "-" {
        ensure!(
            contents.join("embedded.provisionprofile").is_file(),
            "HICKORY_FSKIT_PROFILE must name a profile authorizing com.loumtechnologies.hickorydocs.workspace and the FSKit module entitlement"
        );
    }
    let mut sign = Command::new("/usr/bin/codesign");
    sign.args([
        "--force",
        "--sign",
        &identity,
        "--options",
        "runtime",
        "--entitlements",
    ])
    .arg(source.join("Workspace.entitlements"));
    if identity != "-" {
        sign.arg("--timestamp");
    }
    run(sign.arg(&output))?;
    println!("{}", output.display());
    if identity == "-" {
        eprintln!(
            "Development artifact only: FSKit activation requires an authorized signing identity and provisioning profile."
        );
    }
    Ok(())
}
</storycode:copy>
<a id="f-fb890d5279a3-developer_dir"></a>
### `developer_dir`

<storycode:copy id="f-fb890d5279a3-developer_dir">
fn developer_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("DEVELOPER_DIR") {
        return Ok(path.into());
    }
    let selected = output_of(Command::new("/usr/bin/xcode-select").arg("-p"))?;
    if selected.contains("Xcode") {
        return Ok(selected.into());
    }
    let mut apps: Vec<_> = std::fs::read_dir("/Applications")?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("Xcode"))
                && p.join("Contents/Developer").is_dir()
        })
        .collect();
    apps.sort();
    apps.pop()
        .map(|p| p.join("Contents/Developer"))
        .context("full Xcode is required; set DEVELOPER_DIR to its Contents/Developer directory")
}
</storycode:copy>
<a id="f-fb890d5279a3-run"></a>
### `run`

<storycode:copy id="f-fb890d5279a3-run">
fn run(command: &mut Command) -> Result<()> {
    ensure!(
        command.status()?.success(),
        "native build command failed: {}",
        command.get_program().to_string_lossy()
    );
    Ok(())
}
</storycode:copy>
<a id="f-fb890d5279a3-output_of"></a>
### `output_of`

<storycode:copy id="f-fb890d5279a3-output_of">
fn output_of(command: &mut Command) -> Result<String> {
    let out = command.output()?;
    ensure!(
        out.status.success(),
        "{} failed",
        command.get_program().to_string_lossy()
    );
    Ok(String::from_utf8(out.stdout)?.trim().into())
}
</storycode:copy>

### Assemble `crates/hickory-cli/examples/fskit_bundle.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/examples/fskit_bundle.rs"><storycode:paste select="#f-fb890d5279a3-main"/><storycode:paste select="#f-fb890d5279a3-developer_dir"/><storycode:paste select="#f-fb890d5279a3-run"/><storycode:paste select="#f-fb890d5279a3-output_of"/></storycode:file>

</storycode:doc>
