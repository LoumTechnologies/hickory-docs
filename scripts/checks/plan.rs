use super::{git, Result};
use std::{path::Path, process::Command};

pub fn steps(root: &Path, head: &str) -> Result<Vec<(String, String)>> {
    // On master, merge-base(HEAD, master) is HEAD and misses the commit.
    // Use its parent there; on branches include the whole branch diff.
    let base = git(root, &["merge-base", head, "master"]).unwrap_or_else(|_| head.into());
    let base = if base == head {
        git(root, &["rev-parse", "HEAD^"]).unwrap_or_else(|_| {
            git(root, &["hash-object", "-t", "tree", "--stdin"]).expect("empty tree hash")
        })
    } else {
        base
    };
    let output = Command::new("bash")
        .arg("scripts/affected-checks.sh")
        .arg(&base)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    let selected = String::from_utf8(output.stdout)?;
    let has = |name| selected.lines().any(|line| line == name);
    let mut steps = vec![(
        "Guest scripts".into(),
        "scripts/check-guest-scripts.sh".into(),
    )];
    let mut add = |name: &str, command: &str| steps.push((name.into(), command.into()));
    if has("rust") {
        add("Rust formatting", "cargo fmt --all -- --check");
        add(
            "Rust Clippy",
            "cargo clippy --workspace --all-targets -- -D warnings",
        );
        add("Generated files", "scripts/check-codegen.sh");
    }
    if has("web") || has("desktop") {
        add(
            "Web dependencies",
            "npm --prefix apps/web ci --no-audit --no-fund",
        );
        add("Web build", "npm --prefix apps/web run build");
    }
    if has("web") {
        add("Web tests", "npm --prefix apps/web test -- --maxWorkers=2");
        add("Browser installation", "just install-test-browsers");
        add("Browser embedding", "just test-browser-embedding");
    }
    if has("rust") || has("web") {
        add("File lengths", "scripts/check-file-length.sh");
    }
    if has("desktop") {
        add(
            "Desktop formatting",
            "cd apps/desktop/src-tauri && cargo fmt --all -- --check",
        );
        add(
            "Desktop Clippy",
            "cd apps/desktop/src-tauri && cargo clippy --all-targets -- -D warnings",
        );
        add("Distribution scripts", "bash -n scripts/dist.sh && bash -n scripts/dist-desktop.sh && bash -n scripts/install.sh");
    }
    if has("dev") {
        add("Development seed", "scripts/check-dev-seed.sh");
    }
    if has("python-agent") {
        add("Python agent", "scripts/check-python-agent.sh");
    }
    if has("rust") || has("desktop") {
        let supported = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .filter(|out| out.status.success())
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|version| version.split('.').next()?.parse::<u32>().ok())
            .is_some_and(|major| major >= 26);
        if supported {
            add("FSKit build", "just build-fskit");
            add("Workspace filesystem", "just test-workspace-fs");
        }
    }
    Ok(steps)
}
