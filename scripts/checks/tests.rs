use super::*;
struct Repo(PathBuf);
impl Repo {
    fn new(script: &str) -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("checks-test-{}-{id}", process::id()));
        fs::create_dir_all(path.join("scripts")).unwrap();
        let repo = Self(path);
        repo.git(&["init", "-b", "master"]);
        repo.git(&["config", "user.name", "Checks test"]);
        repo.git(&["config", "user.email", "checks@example.invalid"]);
        repo.git(&["config", "core.hooksPath", "/dev/null"]);
        fs::write(
            repo.0.join("scripts/affected-checks.sh"),
            "#!/bin/bash\nexit 0\n",
        )
        .unwrap();
        fs::write(
            repo.0.join("scripts/check-guest-scripts.sh"),
            format!("#!/bin/bash\n{script}\n"),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                repo.0.join("scripts/check-guest-scripts.sh"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
        repo.git(&["add", "."]);
        repo.git(&["commit", "-m", "fixture"]);
        repo
    }
    fn git(&self, args: &[&str]) -> String {
        git(&self.0, args).unwrap()
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env::var("HICKORY_CHECKS_TEST_RUNNER").unwrap());
        command.current_dir(&self.0);
        command
    }
    fn record(&self) -> PathBuf {
        self.0
            .join(".git/checks/commits")
            .join(format!("{}.txt", self.git(&["rev-parse", "HEAD"])))
    }
}
impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn clean_run_uses_detached_bytes_and_records_head() {
    let repo = Repo::new("pwd; echo hidden-success-output");
    let output = repo.command().output().unwrap();
    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("hidden-success-output"));
    let report = fs::read_to_string(repo.record()).unwrap();
    assert!(report.starts_with("PASSED\n"));
    let log = report
        .lines()
        .find_map(|line| line.strip_prefix("PASS Guest scripts: "))
        .unwrap();
    let log = fs::read_to_string(log).unwrap();
    assert!(log.contains(".git/checks/worktree-"));
    assert!(!repo.git(&["worktree", "list"]).contains("worktree-"));
}
#[test]
fn tracked_staged_and_untracked_runs_never_update_commit_record() {
    let repo = Repo::new("true");
    assert!(repo.command().output().unwrap().status.success());
    let original = fs::read(repo.record()).unwrap();
    fs::write(repo.0.join("new-file"), "untracked").unwrap();
    for staged in [false, true] {
        if staged {
            repo.git(&["add", "."]);
        }
        let output = repo.command().output().unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("not associated"));
        assert_eq!(fs::read(repo.record()).unwrap(), original);
    }
    fs::write(
        repo.0.join("scripts/check-guest-scripts.sh"),
        "#!/bin/bash\ntrue\n# dirty\n",
    )
    .unwrap();
    assert!(repo.command().output().unwrap().status.success());
    assert_eq!(fs::read(repo.record()).unwrap(), original);
}
#[test]
fn failures_are_nonzero_and_show_bounded_diagnostics() {
    let repo = Repo::new("for n in {1..100}; do echo diagnostic-$n; done; exit 9");
    let output = repo.command().output().unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success());
    assert!(text.contains("diagnostic-100"));
    assert!(!text.contains("diagnostic-49\n"));
    assert!(fs::read_to_string(repo.record())
        .unwrap()
        .starts_with("FAILED\n"));
}
#[test]
fn machine_lock_serializes_different_repositories() {
    let first = Repo::new("sleep 2");
    let second = Repo::new("true");
    let mut a = first.command().stdout(Stdio::null()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !first.record().exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
    let b = second.command().output().unwrap();
    assert!(b.status.success());
    assert!(String::from_utf8_lossy(&b.stdout).contains("waiting for another"));
    assert!(a.wait().unwrap().success());
}
#[test]
fn arguments_are_rejected_and_status_does_not_run_checks() {
    let repo = Repo::new("exit 1");
    assert!(!repo
        .command()
        .arg("HEAD")
        .output()
        .unwrap()
        .status
        .success());
    let output = repo.command().arg("--status").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("UNCHECKED"));
    assert!(!repo.record().exists());
}
#[test]
fn post_commit_returns_while_checks_are_running() {
    let repo = Repo::new("true");
    let bin = repo.0.join("bin");
    fs::create_dir(&bin).unwrap();
    let marker = repo.0.join("hook-finished");
    fs::write(
        bin.join("just"),
        format!(
            "#!/bin/bash\n[ \"$1\" = checks ] || exit 1\nsleep 2\ntouch '{}'\n",
            marker.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(bin.join("just"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let hook = PathBuf::from(env!("HICKORY_CHECKS_SOURCE_ROOT")).join(".githooks/post-commit");
    repo.git(&[
        "config",
        "core.hooksPath",
        hook.parent().unwrap().to_str().unwrap(),
    ]);
    let start = Instant::now();
    let output = Command::new("git")
        .args(["commit", "--allow-empty", "-m", "hook test"])
        .current_dir(&repo.0)
        .env(
            "PATH",
            format!("{}:{}", bin.display(), env::var("PATH").unwrap()),
        )
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(!marker.exists());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !marker.exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn selection_includes_latest_master_commit_and_untracked_paths() {
    let repo = Repo::new("true");
    fs::write(
        repo.0.join("scripts/affected-checks.sh"),
        include_str!("../affected-checks.sh"),
    )
    .unwrap();
    repo.git(&["add", "."]);
    repo.git(&["commit", "-m", "real selector"]);
    fs::write(repo.0.join("Cargo.toml"), "# changed backend path\n").unwrap();
    repo.git(&["add", "."]);
    repo.git(&["commit", "-m", "backend change"]);
    let head = repo.git(&["rev-parse", "HEAD"]);
    assert!(plan::steps(&repo.0, &head)
        .unwrap()
        .iter()
        .any(|(name, _)| name == "Rust Clippy"));
    fs::write(repo.0.join("README"), "unrelated\n").unwrap();
    repo.git(&["add", "."]);
    repo.git(&["commit", "-m", "unrelated"]);
    fs::create_dir_all(repo.0.join("apps/web")).unwrap();
    fs::write(repo.0.join("apps/web/new.ts"), "// untracked\n").unwrap();
    let head = repo.git(&["rev-parse", "HEAD"]);
    let steps = plan::steps(&repo.0, &head).unwrap();
    assert!(steps.iter().any(|(name, _)| name == "Web tests"));
    assert!(!steps.iter().any(|(name, _)| name == "Rust Clippy"));
}

#[cfg(unix)]
#[test]
fn child_retains_lock_if_supervisor_dies() {
    let first = Repo::new("echo started > .started; sleep 3");
    // Dirty runs let the test observe when the child has started.
    fs::write(first.0.join("dirty"), "dirty").unwrap();
    let second = Repo::new("true");
    let mut a = first.command().stdout(Stdio::null()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !first.0.join(".started").exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
    a.kill().unwrap();
    a.wait().unwrap();
    let status = first.command().arg("--status").output().unwrap();
    assert!(String::from_utf8_lossy(&status.stdout).contains("INTERRUPTED"));
    let b = second.command().output().unwrap();
    assert!(b.status.success());
    assert!(String::from_utf8_lossy(&b.stdout).contains("waiting for another"));
}
