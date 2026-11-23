<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/build.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b8e7121aea56"></a>
## `crates/hickory-cli/build.rs`

<a id="f-b8e7121aea56-main"></a>
### `main`

Stamp the build so two `hick` binaries can be told apart.
`hick --version` on a release names the release, because `scripts/dist.sh`
sets `HICKORY_VERSION`. Every other build — a `cargo build`, a checkout
someone is hacking on, the copy on a developer's PATH from last week —
reported the workspace's `0.1.0` and nothing else, so all of them claimed
to be the same program.
That matters here more than it would elsewhere. A document's woven output
depends on the version that wove it, and `hick test` compares those bytes:
two builds that render a code fence differently produce a drift failure
that reads as "your content changed" when what changed was the tool. The
failure is confusing exactly in proportion to how hard the versions are to
tell apart, and `0.1.0` == `0.1.0` is as hard as it gets.
Absent git — a source tarball, a vendored build — this stamps nothing and
the version is the crate's, which is the same behaviour as before.

<storycode:copy id="f-b8e7121aea56-main">
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

</storycode:copy>
<a id="f-b8e7121aea56-git"></a>
### `git`

A supervised development build embeds its own full Git object ID and dirty bit.
The supervisor supplies that pair as a Cargo build-script input. When it stays
the same, Cargo can reuse the engine across edits outside its Rust dependency
graph. Compare the input with the build script's own sample after any build-lock
wait. If they disagree, record unknown. Direct Cargo callers with only the dev
flag still use Git's ref and source file watches, including in linked worktrees.

<storycode:copy id="dev-engine-build-stamp">

fn stamp_dev_build() {
    println!("cargo:rerun-if-env-changed=HICKORY_DEV_BUILD_INPUT");
    let input = std::env::var("HICKORY_DEV_BUILD_INPUT").ok();
    let Some(root) = std::env::current_dir().ok().and_then(|dir| {
        dir.join("../..").canonicalize().ok()
    }) else {
        return;
    };
    let git_at_root = |args: &[&str]| {
        let output = Command::new("git").arg("--no-optional-locks").arg("-C").arg(&root).args(args).output().ok()?;
        output.status.success().then_some(output.stdout)
    };
    if input.is_none() {
        watch_checkout(&root, &git_at_root);
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
        let stamp = format!("{}{}", String::from_utf8(commit).ok()?.trim(), if dirty { "+" } else { "" });
        if input.as_deref().is_some_and(|input| input != stamp) {
            return None;
        }
        Some(stamp)
    })();
    println!("cargo:rustc-env=HICKORY_DEV_BUILD_STAMP={}", stamp.as_deref().unwrap_or("unknown"));
}

fn watch_checkout(root: &std::path::Path, git_at_root: impl Fn(&[&str]) -> Option<Vec<u8>>) {
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
}
</storycode:copy>

<storycode:copy id="f-b8e7121aea56-git">
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").arg("--no-optional-locks").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
</storycode:copy>

### Assemble `crates/hickory-cli/build.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/build.rs"><storycode:paste select="#f-b8e7121aea56-main"/><storycode:paste select="#dev-engine-build-stamp"/><storycode:paste select="#f-b8e7121aea56-git"/></storycode:file>

</storycode:doc>
