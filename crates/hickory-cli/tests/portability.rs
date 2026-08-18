//! The portable core stays portable.
//!
//! Protects docs/specs/freeform/shipping-mobile-and-desktop.md
//!
//! Notes sync, read, and capture on a phone, which means part of this engine
//! has to run where a process cannot be spawned: iOS does not permit a
//! sandboxed app to `fork`/`exec`, and Android will not have `git`, `python`,
//! or a language server on it either.
//!
//! The trap this exists for is that **the compiler cannot catch it**. A crate
//! that shells out cross-compiles for `aarch64-apple-ios` perfectly and dies
//! the first time it runs. `hick-store` is exactly that shape today, and it is
//! no longer hypothetical on either target: measured 2026-08-18, it compiles
//! clean for `aarch64-linux-android` *and* `aarch64-apple-ios` while
//! `git_backend.rs` calls `git init --bare` through `tokio::process::Command`.
//! That is why the portable set is a list which has to be defended rather than a
//! property that can be inferred.
//!
//! A simulator will not catch it either — a simulator app is a macOS process
//! wearing an iOS runtime and `posix_spawn` works there, as
//! `experiments/git-libraries/ios-push` demonstrates by spawning `/bin/echo`
//! successfully from inside an installed `.app`. Only a device settles it.

use std::path::{Path, PathBuf};

/// Crates a mobile shell may link.
///
/// Adding one is a promise that it runs where nothing can be spawned. Removing
/// the promise means removing the crate from this list, deliberately, rather
/// than discovering it on a device.
const PORTABLE_CRATES: [&str; 5] = [
    "hick-lang",
    "hick-transcript",
    "hick-flow",
    "hick-condition",
    "hick-case",
];

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/hickory-cli has a parent")
        .to_path_buf()
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Lines that would spawn a process, ignoring comments — a module doc that
/// *describes* the rule must not trip it.
fn spawn_sites(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("//") && !trimmed.starts_with("*")
        })
        .filter(|(_, line)| line.contains("Command::new") || line.contains("process::Command"))
        .map(|(index, line)| (index + 1, line.trim().to_string()))
        .collect()
}

#[test]
fn the_portable_crates_never_spawn_a_process() {
    let crates = crates_dir();
    let mut offences = Vec::new();

    for name in PORTABLE_CRATES {
        let src = crates.join(name).join("src");
        assert!(
            src.is_dir(),
            "{name} is in the portable set but {} does not exist — the list and \
             the tree disagree",
            src.display()
        );

        let mut files = Vec::new();
        rust_sources(&src, &mut files);
        assert!(
            !files.is_empty(),
            "no Rust sources found under {}",
            src.display()
        );

        for file in files {
            let text = std::fs::read_to_string(&file).expect("a readable source file");
            for (line, text) in spawn_sites(&text) {
                offences.push(format!("{}:{line}: {text}", file.display()));
            }
        }
    }

    assert!(
        offences.is_empty(),
        "a crate in the portable set spawns a process, which compiles for every \
         mobile target and runs on none of them:\n  {}\n\nEither keep the crate \
         portable, or take it out of PORTABLE_CRATES and out of the mobile \
         shell — see docs/specs/freeform/shipping-mobile-and-desktop.md.",
        offences.join("\n  ")
    );
}

#[test]
fn the_test_can_actually_see_a_spawn() {
    // A guard that cannot fail is not a guard. `hick-store` is the known
    // offender — it shells out to `git` — so it stands in as the fixture that
    // proves this test detects what it claims to.
    let git_backend = crates_dir()
        .join("hick-store")
        .join("src")
        .join("git_backend.rs");
    let text = std::fs::read_to_string(&git_backend).expect("hick-store has a git backend");
    assert!(
        !spawn_sites(&text).is_empty(),
        "hick-store no longer shells out — good, but this guard now proves \
         nothing. Point it at another known spawn site, or delete it."
    );
}

#[test]
fn a_comment_describing_a_spawn_is_not_a_spawn() {
    let source = "// Command::new is what this forbids\n/// process::Command, mentioned in a doc\nlet x = 1;\n";
    assert!(spawn_sites(source).is_empty());
}
