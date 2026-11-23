use std::{
    env, io,
    process::{Child, Command, ExitCode},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

static STOP: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
unsafe extern "C" {
    fn signal(sig: i32, handler: usize) -> usize;
    fn kill(pid: i32, sig: i32) -> i32;
}
#[cfg(unix)]
extern "C" fn interrupted(_: i32) {
    STOP.store(true, Ordering::SeqCst);
}
#[cfg(windows)]
#[link(name = "Kernel32")]
unsafe extern "system" {
    fn SetConsoleCtrlHandler(handler: extern "system" fn(u32) -> i32, add: i32) -> i32;
}
#[cfg(windows)]
extern "system" fn interrupted(_: u32) -> i32 {
    STOP.store(true, Ordering::SeqCst);
    1
}

fn revision() -> io::Result<Option<String>> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .output()?;
    if output.status.success() {
        return Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        ));
    }
    // An unborn branch is a valid state; any other Git failure must be visible.
    let unborn = Command::new("git")
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()?;
    if unborn.status.success() {
        Ok(None)
    } else {
        Err(io::Error::other("Cannot resolve this checkout's HEAD"))
    }
}

fn dev_build_input() -> String {
    let sample = || {
        let commit = revision().ok()??;
        let status = Command::new("git")
            .args([
                "--no-optional-locks",
                "status",
                "--porcelain=v1",
                "--untracked-files=normal",
            ])
            .output()
            .ok()?;
        if !status.status.success() || revision().ok()?.as_deref() != Some(commit.as_str()) {
            return None;
        }
        let dirty = if status.stdout.is_empty() { "" } else { "+" };
        Some(format!("{commit}{dirty}"))
    };
    sample().unwrap_or_else(|| "unknown".to_owned())
}

struct Owned {
    child: Child,
    stopped: bool,
    pid: u32,
    #[cfg(unix)]
    members: Vec<Process>,
    #[cfg(unix)]
    root_birth: Option<String>,
}
impl Owned {
    fn start(args: &[String]) -> io::Result<Self> {
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]);
        if env::var("HICKORY_DEV_BUILD").as_deref() == Ok("1") {
            command.env("HICKORY_DEV_BUILD_INPUT", dev_build_input());
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let child = command.spawn()?;
        let pid = child.id();
        Ok(Self {
            child,
            stopped: false,
            pid,
            #[cfg(unix)]
            members: Vec::new(),
            #[cfg(unix)]
            root_birth: None,
        })
    }
    fn stop(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            // Descendants can create their own groups (Tauri and npm do).
            // Snapshot those groups before the launcher can exit and orphan them.
            self.remember_groups()?;
            let groups = self.groups();
            for group in &groups {
                unsafe {
                    kill(-*group, 15);
                }
            }
            let deadline = Instant::now() + Duration::from_secs(15);
            loop {
                let _ = self.child.try_wait()?;
                self.remember_groups()?;
                if self.groups().is_empty() {
                    break;
                }
                if Instant::now() >= deadline {
                    for group in self.groups() {
                        unsafe {
                            kill(-group, 9);
                        }
                    }
                    let deadline = Instant::now() + Duration::from_secs(5);
                    loop {
                        self.remember_groups()?;
                        if self.groups().is_empty() {
                            break;
                        }
                        if Instant::now() >= deadline {
                            return Err(io::Error::other(
                                "Owned development processes did not stop; refusing replacement",
                            ));
                        }
                        let _ = self.child.try_wait()?;
                        thread::sleep(Duration::from_millis(50));
                    }
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
        #[cfg(windows)]
        {
            let status = Command::new("taskkill")
                .args(["/T", "/F", "/PID", &self.pid.to_string()])
                .status()?;
            if !status.success() && self.child.try_wait()?.is_none() {
                return Err(io::Error::other("taskkill failed; refusing replacement"));
            }
        }
        let _ = self.child.wait()?;
        self.stopped = true;
        Ok(())
    }
    #[cfg(unix)]
    fn remember_groups(&mut self) -> io::Result<()> {
        let rows = processes()?;
        // The unreaped Child reserves this PID until the first observation.
        if self.root_birth.is_none() {
            self.root_birth = rows
                .iter()
                .find(|row| row.pid == self.pid as i32)
                .map(|row| row.birth.clone());
        }
        self.members = owned_members(
            &rows,
            &self.members,
            self.pid as i32,
            self.root_birth.as_deref(),
        );
        Ok(())
    }
    #[cfg(unix)]
    fn groups(&self) -> Vec<i32> {
        let mut groups = Vec::new();
        for row in &self.members {
            if !row.zombie && row.group > 0 && !groups.contains(&row.group) {
                groups.push(row.group);
            }
        }
        groups
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        if !self.stopped {
            // An unexpected Git/ps I/O failure must not abandon the app.
            #[cfg(unix)]
            if self.remember_groups().is_ok() {
                for group in self.groups() {
                    unsafe {
                        kill(-group, 9);
                    }
                }
            }
            #[cfg(windows)]
            let _ = Command::new("taskkill")
                .args(["/T", "/F", "/PID", &self.pid.to_string()])
                .status();
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
#[cfg(unix)]
#[derive(Clone)]
struct Process {
    pid: i32,
    parent: i32,
    group: i32,
    zombie: bool,
    birth: String,
    executable: String,
}
#[cfg(unix)]
fn owned_members(
    rows: &[Process],
    previous: &[Process],
    root: i32,
    birth: Option<&str>,
) -> Vec<Process> {
    let mut members: Vec<Process> = rows
        .iter()
        .filter(|row| {
            (row.pid == root && Some(row.birth.as_str()) == birth)
                || previous.iter().any(|old| {
                    old.pid == row.pid && old.birth == row.birth && old.group == row.group
                })
        })
        .cloned()
        .collect();
    loop {
        let before = members.len();
        for row in rows {
            if !members.iter().any(|member| member.pid == row.pid)
                && members
                    .iter()
                    .any(|member| member.pid == row.parent || member.group == row.group)
            {
                members.push(row.clone());
            }
        }
        if members.len() == before {
            return members;
        }
    }
}
#[cfg(unix)]
fn processes() -> io::Result<Vec<Process>> {
    let output = Command::new("ps")
        .env("LC_ALL", "C")
        .env("TZ", "UTC")
        .args(["-axo", "pid=,ppid=,pgid=,stat=,lstart=,comm="])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(
            "ps is required to stop development process trees",
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            Some(Process {
                pid: fields.next()?.parse().ok()?,
                parent: fields.next()?.parse().ok()?,
                group: fields.next()?.parse().ok()?,
                zombie: fields.next()?.starts_with('Z'),
                birth: fields.by_ref().take(5).collect::<Vec<_>>().join(" "),
                executable: fields.collect::<Vec<_>>().join(" "),
            })
        })
        .collect())
}
#[cfg(all(test, unix))]
#[test]
fn ownership_rejects_recycled_ids_and_retains_orphans() {
    let process = |pid, parent, group, birth: &str| Process {
        pid,
        parent,
        group,
        zombie: false,
        birth: birth.into(),
        executable: String::new(),
    };
    let old = process(20, 10, 20, "old");
    assert!(
        owned_members(
            &[process(20, 1, 20, "new")],
            &[old.clone()],
            10,
            Some("root")
        )
        .is_empty()
    );
    let rows = [process(20, 1, 20, "old"), process(30, 20, 30, "child")];
    let members = owned_members(&rows, &[old], 10, Some("root"));
    assert_eq!(members.len(), 2);
}

const DESKTOP_STOP: &str = "target/dev-desktop-stop";
#[cfg(unix)]
const DESKTOP_REQUEST: &str = "target/dev-desktop-request";

enum RestartOn {
    Commit,
    #[cfg(unix)]
    AppleRequest,
}

impl RestartOn {
    fn token(&self) -> io::Result<Option<String>> {
        match self {
            Self::Commit => revision(),
            #[cfg(unix)]
            Self::AppleRequest => std::fs::read_to_string(DESKTOP_REQUEST).map(Some),
        }
    }

    fn can_restart(&self, _child: Option<&Owned>) -> bool {
        match self {
            Self::Commit => true,
            #[cfg(unix)]
            Self::AppleRequest => _child.is_none_or(|child| {
                child.members.iter().any(|process| {
                    !process.zombie
                        && std::path::Path::new(&process.executable)
                            .file_name()
                            .is_some_and(|name| name == "HickorySimple")
                })
            }),
        }
    }

    fn same_inputs(&self, before: Option<&str>, after: Option<&str>) -> bool {
        match self {
            Self::Commit => false,
            #[cfg(unix)]
            Self::AppleRequest => before
                .and_then(|token| token.split_once(' '))
                .zip(after.and_then(|token| token.split_once(' ')))
                .is_some_and(|((_, old), (_, new))| old == new),
        }
    }
}

fn supervise(args: &[String], stop_file: Option<&str>, restart: RestartOn) -> io::Result<()> {
    let _desktop_lock =
        if stop_file.is_some() {
            let file = desktop_lock()?;
            file.try_lock().map_err(|error| io::Error::other(format!(
            "Desktop session already running or unavailable ({error}); run just dev-stop first"
        )))?;
            Some(file)
        } else {
            None
        };
    let stop_token = stop_file.map(|path| std::fs::read(path).ok());
    if args.is_empty() {
        return Err(io::Error::other("Usage: dev-supervisor COMMAND [ARG ...]"));
    }
    if !Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?
        .status
        .success()
    {
        return Err(io::Error::other(
            "Run this development command inside its Git checkout",
        ));
    }
    #[cfg(unix)]
    unsafe {
        signal(2, interrupted as *const () as usize);
        signal(15, interrupted as *const () as usize);
    }
    #[cfg(windows)]
    unsafe {
        SetConsoleCtrlHandler(interrupted, 1);
    }
    let mut seen = restart.token()?;
    loop {
        if STOP.load(Ordering::SeqCst)
            || stop_file.map(|path| std::fs::read(path).ok()) != stop_token
        {
            return Ok(());
        }
        eprintln!(
            "Starting development session at {}",
            seen.as_deref().unwrap_or("unborn HEAD")
        );
        let mut owned = match Owned::start(args) {
            Ok(child) => Some(child),
            Err(error) => {
                eprintln!("Startup failed: {error}. Waiting for the next restart trigger.");
                None
            }
        };
        let outcome = (|| -> io::Result<bool> {
            loop {
                if STOP.load(Ordering::SeqCst) {
                    return Ok(false);
                }
                if stop_file.map(|path| std::fs::read(path).ok()) != stop_token {
                    return Ok(false);
                }
                let now = restart.token()?;
                if let Some(child) = &mut owned {
                    #[cfg(unix)]
                    child.remember_groups()?;
                }
                if now != seen && restart.can_restart(owned.as_ref()) {
                    if owned.is_some() && restart.same_inputs(seen.as_deref(), now.as_deref()) {
                        seen = now.clone();
                    } else {
                        return Ok(true);
                    }
                }
                if let Some(child) = &mut owned {
                    if let Some(status) = child.child.try_wait()? {
                        if status.success() {
                            return Ok(now != seen);
                        }
                        eprintln!(
                            "Development command failed ({status}); waiting for the next restart trigger."
                        );
                        child.stop()?;
                        owned = None;
                    }
                }
                thread::sleep(Duration::from_millis(300));
            }
        })();
        if let Some(child) = &mut owned {
            child.stop()?;
        }
        if !outcome? {
            return Ok(());
        }
        // Read again after shutdown so several changes can collapse into one launch.
        seen = restart.token()?;
        eprintln!("Restarting with current working-tree files.");
    }
}

fn desktop_lock() -> io::Result<std::fs::File> {
    std::fs::create_dir_all("target")?;
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open("target/dev-desktop.lock")
}

fn stop_desktop(args: &[String]) -> io::Result<()> {
    let action = args.first().map(String::as_str).unwrap_or("stop");
    if !["stop", "clean"].contains(&action) {
        return Err(io::Error::other("Expected stop or clean"));
    }
    std::fs::create_dir_all("target")?;
    // Outside .dev so clean cannot erase the stop request before it is observed.
    let token = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos()
        .to_string();
    std::fs::write("target/dev-desktop-stop", token)?;
    // Wait for the supervisor to release its lifetime lock after child teardown.
    // Stale lock files are harmless: the OS releases locks when a process dies.
    let lock = desktop_lock()?;
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        match lock.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(error) => {
                return Err(io::Error::other(format!(
                    "Desktop session did not stop ({error}); cleanup refused to protect its files"
                )));
            }
        }
    }
    if action == "clean" {
        match std::fs::remove_dir_all(".dev") {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("--web-once") => web_once(&args[1..]),
        Some("--desktop") => supervise(&args[1..], Some(DESKTOP_STOP), RestartOn::Commit),
        #[cfg(unix)]
        Some("--desktop-request") => {
            supervise(&args[1..], Some(DESKTOP_STOP), RestartOn::AppleRequest)
        }
        Some("--desktop-stop") => stop_desktop(&args[1..]),
        _ => supervise(&args, None, RestartOn::Commit),
    };
    if let Err(error) = result {
        eprintln!("Development supervisor: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn web_once(args: &[String]) -> io::Result<()> {
    let script = args
        .first()
        .ok_or_else(|| io::Error::other("Missing npm script"))?;
    let lock = std::fs::read("apps/web/package-lock.json")?;
    let stamp = "apps/web/node_modules/.hickory-dev-lock";
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    if std::fs::read(stamp).ok().as_ref() != Some(&lock) {
        let status = Command::new(npm)
            .args(["--prefix", "apps/web", "ci"])
            .status()?;
        if !status.success() {
            return Err(io::Error::other("npm ci failed"));
        }
        std::fs::write(stamp, lock)?;
    }
    let mut command = Command::new(npm);
    command.args(["--prefix", "apps/web", "run", script]);
    if let Some(origin) = args.get(1) {
        command.env("HICKORY_API_ORIGIN", origin);
    }
    if !command.status()?.success() {
        return Err(io::Error::other("Frontend development server failed"));
    }
    Ok(())
}
