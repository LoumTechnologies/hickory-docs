//! Exercise the shared hook/CI selector against real staged and unstaged diffs.
use std::{path::Path, process::Command};

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "master"]);
    git(dir.path(), &["config", "user.name", "Test"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    std::fs::create_dir(dir.path().join("scripts")).unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/affected-checks.sh"),
        dir.path().join("scripts/affected-checks.sh"),
    )
    .unwrap();
    git(dir.path(), &["add", "."]);
    git(
        dir.path(),
        &["-c", "core.hooksPath=/dev/null", "commit", "-qm", "base"],
    );
    dir
}

#[test]
fn changed_paths_select_checks_for_both_staged_and_unstaged_edits() {
    let repo = repo();
    let cases: &[(&str, &[&str])] = &[
        ("README.md", &[]),
        ("docs/a note.md", &["rust"]),
        ("docs/é.md", &["rust"]),
        ("crates/core/src/lib.rs", &["rust"]),
        ("apps/web/src/app.ts", &["web"]),
        ("apps/desktop/src-tauri/src/main.rs", &["desktop"]),
        ("scripts/install.sh", &["desktop"]),
        ("scripts/check-dev-seed.sh", &["dev", "rust", "web"]),
        ("scripts/check-codegen.sh", &["rust"]),
        ("scripts/file-length-baseline.txt", &["rust", "web"]),
        ("rust-toolchain.toml", &["rust"]),
        (".githooks/pre-commit", &["desktop", "dev", "rust", "web"]),
    ];
    for (path, expected) in cases {
        let file = repo.path().join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "initial\n").unwrap();
        git(repo.path(), &["add", path]);
        for staged in [true, false] {
            if !staged {
                git(
                    repo.path(),
                    &["-c", "core.hooksPath=/dev/null", "commit", "-qm", "fixture"],
                );
                std::fs::write(&file, "edited\n").unwrap();
            }
            let out = Command::new("bash")
                .arg(repo.path().join("scripts/affected-checks.sh"))
                .arg("HEAD")
                .output()
                .unwrap();
            assert!(out.status.success(), "{path}: {out:?}");
            let stdout = String::from_utf8(out.stdout).unwrap();
            let selected: Vec<_> = stdout.lines().collect();
            assert_eq!(&selected, expected, "{path}, staged={staged}");
        }
        git(repo.path(), &["reset", "--hard", "HEAD"]);
    }
}

#[test]
fn invalid_base_and_selector_failure_block_instead_of_passing_as_empty() {
    let repo = repo();
    let script = repo.path().join("scripts/affected-checks.sh");
    let out = Command::new("bash")
        .arg(&script)
        .arg("missing-ref")
        .output()
        .unwrap();
    assert!(!out.status.success(), "invalid base passed: {out:?}");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.githooks/pre-commit"),
        repo.path().join("pre-commit"),
    )
    .unwrap();
    std::fs::write(&script, "#!/usr/bin/env bash\nexit 42\n").unwrap();
    let out = Command::new("bash")
        .arg(repo.path().join("pre-commit"))
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(42),
        "selector failure was lost: {out:?}"
    );
}

#[cfg(unix)]
#[test]
fn selector_handles_newlines_without_losing_the_path_prefix() {
    let repo = repo();
    std::fs::create_dir(repo.path().join("docs")).unwrap();
    let name = "docs/a\nnote.md";
    std::fs::write(repo.path().join(name), "note").unwrap();
    git(repo.path(), &["add", name]);
    let out = Command::new("bash")
        .arg(repo.path().join("scripts/affected-checks.sh"))
        .arg("HEAD")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "rust\n");
}
