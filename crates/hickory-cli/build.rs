//! Stamp the build so two `hick` binaries can be told apart.
//!
//! `hick --version` on a release names the release, because `scripts/dist.sh`
//! sets `HICKORY_VERSION`. Every other build — a `cargo build`, a checkout
//! someone is hacking on, the copy on a developer's PATH from last week —
//! reported the workspace's `0.1.0` and nothing else, so all of them claimed
//! to be the same program.
//!
//! That matters here more than it would elsewhere. A document's woven output
//! depends on the version that wove it, and `hick test` compares those bytes:
//! two builds that render a code fence differently produce a drift failure
//! that reads as "your content changed" when what changed was the tool. The
//! failure is confusing exactly in proportion to how hard the versions are to
//! tell apart, and `0.1.0` == `0.1.0` is as hard as it gets.
//!
//! Absent git — a source tarball, a vendored build — this stamps nothing and
//! the version is the crate's, which is the same behaviour as before.

use std::process::Command;

fn main() {
    // Re-stamp when the checked-out commit moves. Not on every file change:
    // a dirty tree is reported as `+`, which does not need a rebuild to stay
    // truthful about the commit it started from.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-env-changed=HICKORY_VERSION");
    println!("cargo:rerun-if-env-changed=HICKORY_DEV_BUILD");
    if std::env::var("HICKORY_DEV_BUILD").as_deref() == Ok("1")
        && std::env::var_os("HICKORY_VERSION").is_none()
    {
        stamp_dev_build();
    }

    let Some(commit) = git(&["rev-parse", "--short=8", "HEAD"]) else {
        return;
    };
    let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|s| !s.trim().is_empty());
    // The whole string, composed here: `concat!` in the binary takes only
    // literals, so the version cannot be assembled on the other side.
    let crate_version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    println!(
        "cargo:rustc-env=HICKORY_BUILD_VERSION={crate_version} ({commit}{})",
        if dirty { "+" } else { "" }
    );
}


fn stamp_dev_build() {
    let Some(root) = std::env::current_dir().ok().and_then(|dir| {
        dir.join("../..").canonicalize().ok()
    }) else {
        return;
    };
    let git_at_root = |args: &[&str]| {
        let output = Command::new("git").arg("--no-optional-locks").arg("-C").arg(&root).args(args).output().ok()?;
        output.status.success().then_some(output.stdout)
    };
    let mut refs = vec!["HEAD".to_owned(), "index".to_owned(), "packed-refs".to_owned(), "logs/HEAD".to_owned()];
    if let Some(reference) = git_at_root(&["symbolic-ref", "-q", "HEAD"]) {
        refs.push(String::from_utf8_lossy(&reference).trim().to_owned());
    }
    for reference in refs {
        if let Some(path) = git_at_root(&["rev-parse", "--git-path", &reference]) {
            let path = root.join(String::from_utf8_lossy(&path).trim());
            if path.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    if let Some(files) = git_at_root(&["ls-files", "-z", "--cached", "--others", "--exclude-standard"]) {
        for file in files.split(|byte| *byte == 0).filter_map(|file| std::str::from_utf8(file).ok()) {
            let path = root.join(file);
            if !file.contains(['\n', '\r']) && path.is_file() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    let stamp = (|| {
        if !git_at_root(&["rev-parse", "--show-prefix"])?.iter().all(u8::is_ascii_whitespace) {
            return None;
        }
        let commit = git_at_root(&["rev-parse", "--verify", "HEAD"])?;
        let dirty = !git_at_root(&["status", "--porcelain=v1", "--untracked-files=normal"])?.is_empty();
        if git_at_root(&["rev-parse", "--verify", "HEAD"])? != commit {
            return None;
        }
        Some(format!("{}{}", String::from_utf8(commit).ok()?.trim(), if dirty { "+" } else { "" }))
    })();
    println!("cargo:rustc-env=HICKORY_DEV_BUILD_STAMP={}", stamp.as_deref().unwrap_or("unknown"));
}
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
