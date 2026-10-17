//! Dependency-free local check supervisor; compiled by checks.sh.
mod plan;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git").current_dir(root).args(args).output()?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned().into());
    }
    Ok(String::from_utf8(out.stdout)?.trim_end().to_owned())
}
struct Checkout(PathBuf);
impl Drop for Checkout {
    fn drop(&mut self) {
        let _ = Command::new("git")
            .args(["worktree", "remove", "--force"])
            .arg(&self.0)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
fn save(path: &Path, text: &str) -> Result<()> {
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, text)?;
    fs::rename(temporary, path)?;
    Ok(())
}
fn status(root: &Path) -> Result<bool> {
    let common = git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let head = git(root, &["rev-parse", "HEAD"])?;
    for (label, path) in [
        (
            "HEAD",
            PathBuf::from(&common)
                .join("checks/commits")
                .join(format!("{head}.txt")),
        ),
        (
            "Latest run",
            PathBuf::from(&common).join("checks/latest.txt"),
        ),
    ] {
        println!("{label}:");
        match fs::read_to_string(path) {
            Ok(mut report) => {
                if report.starts_with("RUNNING\n") {
                    let alive = report
                        .lines()
                        .find_map(|line| line.strip_prefix("PID: "))
                        .is_some_and(|pid| {
                            Command::new("kill")
                                .args(["-0", pid])
                                .stderr(Stdio::null())
                                .status()
                                .is_ok_and(|s| s.success())
                        });
                    if !alive {
                        report = report.replacen("RUNNING", "INTERRUPTED", 1);
                    }
                }
                print!("{report}");
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => println!("UNCHECKED"),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(true)
}

#[cfg(unix)]
fn inherit_lock(lock: &File) -> Result<()> {
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        fn fcntl(fd: i32, command: i32, ...) -> i32;
    }
    // F_SETFD = 2, clear FD_CLOEXEC. Keep the lock held by an active check
    // even if its supervisor is terminated. The kernel releases it on exit.
    if unsafe { fcntl(lock.as_raw_fd(), 2, 0) } == -1 {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

fn run() -> Result<bool> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args == ["--status"] {
        return status(Path::new("."));
    }
    if !args.is_empty() {
        return Err("just checks takes no arguments".into());
    }
    let root = PathBuf::from(git(Path::new("."), &["rev-parse", "--show-toplevel"])?);
    // One OS-managed advisory lock across repositories and worktrees. A killed
    // supervisor cannot leave a stale lock. Child commands inherit it on Unix.
    #[cfg(unix)]
    let lock_path = PathBuf::from("/tmp/hickory-local-checks.lock");
    #[cfg(not(unix))]
    let lock_path = env::temp_dir().join("hickory-local-checks.lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    if lock.try_lock().is_err() {
        println!("checks: waiting for another local check run…");
        lock.lock()?;
    }
    #[cfg(unix)]
    inherit_lock(&lock)?;
    let head = git(&root, &["rev-parse", "HEAD"])?;
    let clean = git(
        &root,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?
    .is_empty();
    let common = PathBuf::from(git(
        &root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?);
    let state = common.join("checks");
    let id = format!(
        "{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        process::id()
    );
    let logs = state.join("runs").join(&id);
    fs::create_dir_all(&logs)?;
    let result = logs.join("result.txt");
    let associated = if clean {
        Some(state.join("commits").join(format!("{head}.txt")))
    } else {
        None
    };
    fs::create_dir_all(state.join("commits"))?;
    let describe = if clean {
        format!("commit {head}")
    } else {
        "uncommitted files — result is not associated with a commit".into()
    };
    let mut report = format!(
        "RUNNING\n{describe}\nPID: {}\nLogs: {}\n",
        process::id(),
        logs.display()
    );
    let publish = |report: &str| -> Result<()> {
        save(&result, report)?;
        save(&state.join("latest.txt"), report)?;
        if let Some(path) = &associated {
            save(path, report)?;
        }
        Ok(())
    };
    publish(&report)?;
    println!("checks: {describe}");
    // Clean checks read immutable committed bytes, never the active worktree.
    let checkout = if clean {
        let path = state.join(format!("worktree-{id}"));
        git(
            &root,
            &[
                "-c",
                "core.hooksPath=/dev/null",
                "worktree",
                "add",
                "--detach",
                path.to_str().ok_or("non-UTF8 worktree path")?,
                &head,
            ],
        )?;
        Some(Checkout(path))
    } else {
        None
    };
    let work = checkout.as_ref().map_or(root.as_path(), |c| c.0.as_path());
    let steps = plan::steps(work, &head)?;
    let mut failed = 0;
    for (index, (name, command)) in steps.iter().enumerate() {
        let log_path = logs.join(format!("{:02}.log", index + 1));
        let log = File::create(&log_path)?;
        let start = Instant::now();
        print!("[{}/{}] {name} …", index + 1, steps.len());
        io::stdout().flush()?;
        let mut child = Command::new("bash")
            .args(["-c", command])
            .current_dir(work)
            .env("CARGO_TARGET_DIR", root.join("target"))
            .env("CARGO_TERM_COLOR", "never")
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log)
            .spawn()?;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            thread::sleep(Duration::from_secs(1));
            if io::stdout().is_terminal() {
                print!(
                    "\r[{}/{}] {name} … {}s",
                    index + 1,
                    steps.len(),
                    start.elapsed().as_secs()
                );
                io::stdout().flush()?;
            }
        };
        let verdict = if status.success() {
            "PASS"
        } else {
            failed += 1;
            "FAIL"
        };
        println!(" {verdict} ({}s)", start.elapsed().as_secs());
        report.push_str(&format!("{verdict} {name}: {}\n", log_path.display()));
        publish(&report)?;
        if !status.success() {
            let text = fs::read_to_string(&log_path).unwrap_or_default();
            let lines: Vec<_> = text.lines().collect();
            println!(
                "--- {name}: last 50 lines (full log: {}) ---",
                log_path.display()
            );
            for line in &lines[lines.len().saturating_sub(50)..] {
                println!("{line}");
            }
        }
    }
    report = report.replacen("RUNNING", if failed == 0 { "PASSED" } else { "FAILED" }, 1);
    publish(&report)?;
    println!(
        "checks: {} passed, {failed} failed. Result: {}",
        steps.len() - failed,
        result.display()
    );
    Ok(failed == 0)
}
fn main() {
    match run() {
        Ok(true) => (),
        Ok(false) => process::exit(1),
        Err(error) => {
            eprintln!("checks: {error}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests;
