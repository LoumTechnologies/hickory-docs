<restart:doc xmlns:restart="http://www.hickorydocs.com/1.0">
# Restart development sessions when this worktree commits

For maintainers changing local launch behavior. This reading covers the shared supervisor and its harmless real-process witnesses, not app correctness or broad regression coverage. The [developer workflow](../../docs/developers/developer-environment.md) records eligible commands and mobile device ownership.

## Resolve the commit, rather than watching a Git file

Git resolves linked worktrees, detached HEAD and packed refs itself. An unborn branch is a valid initial revision. Signals only set a flag; shutdown stays in the foreground lifecycle.

<restart:copy id="signals-and-revisions">
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

</restart:copy>

## Stop the previous tree before starting its replacement

Each launcher starts in a new process group. npm and Tauri can create more groups, so remember descendant process identities while the launcher is alive. Refresh ownership using PID and birth time before signalling, so exited build processes cannot leave recycled process-group identifiers in the cleanup set. On termination, allow graceful cleanup, then kill remaining owned groups. Refuse replacement if any remain. Windows uses taskkill /T for the launched tree; host verification below covers Unix only.

<restart:copy id="owned-process-tree">
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
        .args(["-axo", "pid=,ppid=,pgid=,stat=,lstart="])
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
                birth: fields.collect::<Vec<_>>().join(" "),
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

</restart:copy>

## A commit during preparation must still be observed

The polling loop runs while the child prepares and builds. Check HEAD before examining child exit, then reread it after cleanup to coalesce changes. A worktree-local stop token also lets `dev-stop` and `dev-clean` end the supervisor without PID-file reuse risks. An OS lifetime lock makes cleanup wait until child teardown finishes and refuses overlapping desktop sessions; stale files cannot retain a lock after a crash. Successful window closure exits normally; a failed revision remains visible and waits for a new commit. Working files and environment are never reset.

<restart:copy id="restart-loop">
fn supervise(args: &[String], stop_file: Option<&str>) -> io::Result<()> {
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
    let mut seen = revision()?;
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
                eprintln!("Startup failed: {error}. Waiting for the next commit.");
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
                let now = revision()?;
                if now != seen {
                    return Ok(true);
                }
                if let Some(child) = &mut owned {
                    #[cfg(unix)]
                    child.remember_groups()?;
                    if let Some(status) = child.child.try_wait()? {
                        if status.success() {
                            return Ok(false);
                        }
                        eprintln!(
                            "Development command failed ({status}); waiting for the next commit."
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
        // Read again after shutdown: several commits can collapse into one launch.
        seen = revision()?;
        eprintln!("Commit changed; restarting with current working-tree files.");
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
    if !Command::new("./scripts/dev.sh")
        .arg(action)
        .status()?
        .success()
    {
        return Err(io::Error::other(
            "Desktop cleanup failed; see dev.sh output",
        ));
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = if args.first().is_some_and(|arg| arg == "--web-once") {
        web_once(&args[1..])
    } else if args.first().is_some_and(|arg| arg == "--desktop") {
        supervise(&args[1..], Some("target/dev-desktop-stop"))
    } else if args.first().is_some_and(|arg| arg == "--desktop-stop") {
        stop_desktop(&args[1..])
    } else {
        supervise(&args, None)
    };
    if let Err(error) = result {
        eprintln!("Development supervisor: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

</restart:copy>

## Prepare scoped servers without reinstalling unchanged dependencies

The helper runs inside the owned child tree. A byte-for-byte lockfile stamp inside node_modules prevents repeated installation, while a missing dependency tree or changed lockfile runs npm ci before Vite. No package upgrades occur.

<restart:copy id="frontend-preparation">
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
</restart:copy>

<restart:file path="scripts/dev-supervisor.rs"><restart:paste select="#signals-and-revisions"/><restart:paste select="#owned-process-tree"/><restart:paste select="#restart-loop"/><restart:paste select="#frontend-preparation"/></restart:file>

## Exercise commit changes without touching the product checkout

The witness creates disposable repositories and a long-running child with a grandchild in a separate session. Its event log checks save-only stability, first commit, amend, rapid commits during startup, failure recovery, argument/environment preservation and Ctrl-C cleanup. The linked-worktree case advances another checkout without restarting this one, then exercises packed refs, detached HEAD and reset. These are process witnesses on the executing host; GUI and Windows acceptance remain separate.

<restart:copy id="restart-witnesses">
"""Focused real-process witnesses; no app, GUI, downloads or target checkout commits."""
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[2]

class SupervisorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.build = tempfile.TemporaryDirectory(prefix='hickory-supervisor-build-')
        cls.binary = Path(cls.build.name) / 'supervisor'
        subprocess.run(['rustc', '--edition', '2024', str(ROOT / 'scripts/dev-supervisor.rs'), '-o', str(cls.binary)], check=True)
        witness = Path(cls.build.name) / 'ownership-test'
        subprocess.run(['rustc', '--edition', '2024', '--test', str(ROOT / 'scripts/dev-supervisor.rs'), '-o', str(witness)], check=True)
        subprocess.run([str(witness)], check=True)

    @classmethod
    def tearDownClass(cls):
        cls.build.cleanup()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='hickory-supervisor-test-')
        self.root = Path(self.temp.name)
        self.git('init', '-q')
        self.git('config', 'user.email', 'test@example.invalid')
        self.git('config', 'user.name', 'Supervisor test')
        self.git('config', 'core.hooksPath', '/dev/null')
        self.process = None
        self.log = (self.root / 'supervisor.log').open('w')
        self.child = self.root / 'child.py'
        self.child.write_text('''import os, pathlib, signal, subprocess, sys, time
root = pathlib.Path.cwd()
def log(text):
    with (root / 'events').open('a') as f: f.write(text + '\\n')
# Each replacement refuses to overlap with a live prior process tree.
if (root / 'grandchild.pid').exists():
    pid = int((root / 'grandchild.pid').read_text())
    try:
        os.kill(pid, 0)
        log('overlap')
        sys.exit(9)
    except ProcessLookupError: pass
log('start ' + str(os.getpid()) + ' ' + repr(sys.argv[1:]) + ' ' + os.environ.get('SUPERVISOR_TEST', ''))
if (root / 'fail').exists(): sys.exit(7)
grandchild = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(300)'], start_new_session=True)
(root / 'grandchild.pid').write_text(str(grandchild.pid))
def stop(*_):
    grandchild.terminate()
    grandchild.wait()
    log('stop ' + str(os.getpid()))
    sys.exit(0)
signal.signal(signal.SIGTERM, stop)
time.sleep(float((root / 'delay').read_text()) if (root / 'delay').exists() else 0)
log('ready')
while True: time.sleep(.05)
''')

    def tearDown(self):
        if self.process and self.process.poll() is None:
            self.process.send_signal(signal.SIGTERM)
            try: self.process.wait(timeout=25)
            except subprocess.TimeoutExpired:
                self.process.kill()
                raise
        self.log.close()
        self.temp.cleanup()

    def git(self, *args, cwd=None):
        return subprocess.run(['git', *args], cwd=cwd or self.root, check=True, capture_output=True, text=True).stdout.strip()

    def commit(self, amend=False):
        args = ['commit', '--allow-empty', '-qm', 'test']
        if amend: args += ['--amend']
        # Unique content guarantees amends made within the same second change ID.
        (self.root / 'tracked').write_text(str(time.time_ns()))
        self.git('add', 'tracked')
        self.git(*args)

    def events(self):
        path = self.root / 'events'
        return path.read_text().splitlines() if path.exists() else []

    def wait_for(self, predicate, timeout=12):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if predicate(): return
            if self.process and self.process.poll() is not None:
                self.fail('Supervisor exited: ' + (self.root / 'supervisor.log').read_text())
            time.sleep(.05)
        self.fail('Timed out: ' + repr(self.events()) + '\n' + (self.root / 'supervisor.log').read_text())

    def starts(self):
        return sum(line.startswith('start ') for line in self.events())

    def launch(self, cwd=None):
        self.process = subprocess.Popen([str(self.binary), os.sys.executable, str(self.child), 'space value', '$literal'], cwd=cwd or self.root, env={**os.environ, 'SUPERVISOR_TEST': 'kept'}, stdout=self.log, stderr=self.log)
        self.wait_for(lambda: self.starts() == 1)

    def test_commits_startup_failure_and_cleanup(self):
        self.launch()  # unborn HEAD starts immediately
        self.wait_for(lambda: 'ready' in self.events())
        (self.root / 'save').write_text('uncommitted')
        time.sleep(.8)
        self.assertEqual(self.starts(), 1)
        self.commit()  # first commit
        self.wait_for(lambda: self.starts() == 2)
        self.commit(amend=True)
        self.wait_for(lambda: self.starts() == 3)
        (self.root / 'delay').write_text('1.5')
        self.commit()
        self.wait_for(lambda: self.starts() == 4)
        self.commit()  # commit while startup is sleeping
        self.commit()
        self.wait_for(lambda: self.starts() >= 5)
        time.sleep(2)
        count = self.starts()
        self.assertGreaterEqual(count, 5)
        self.assertLessEqual(count, 6)
        (self.root / 'fail').touch()
        self.commit()
        self.wait_for(lambda: self.starts() == count + 1)
        time.sleep(.8)
        self.assertEqual(self.starts(), count + 1)  # failed revision never spins
        (self.root / 'fail').unlink()
        self.commit()
        self.wait_for(lambda: self.starts() == count + 2)
        self.wait_for(lambda: self.events()[-1] == 'ready')
        self.assertNotIn('overlap', self.events())
        self.assertTrue(all("['space value', '$literal'] kept" in line for line in self.events() if line.startswith('start ')))
        pid = int((self.root / 'grandchild.pid').read_text())
        self.process.send_signal(signal.SIGINT)
        self.assertEqual(self.process.wait(timeout=25), 0)
        with self.assertRaises(ProcessLookupError): os.kill(pid, 0)

    def test_worktree_detached_packed_refs_and_other_checkout(self):
        self.commit()
        other = self.root / 'linked'
        self.git('worktree', 'add', '-qb', 'linked', str(other))
        main = self.root
        self.root = other
        self.launch()
        self.git('commit', '--allow-empty', '-qm', 'other checkout', cwd=main)
        time.sleep(.8)
        self.assertEqual(self.starts(), 1)
        self.git('pack-refs', '--all', cwd=main)
        self.commit()
        self.wait_for(lambda: self.starts() == 2)
        self.commit(amend=True)
        self.wait_for(lambda: self.starts() == 3)
        self.git('checkout', '--detach', 'HEAD~1')
        self.wait_for(lambda: self.starts() == 4)
        self.git('reset', '--hard', self.git('rev-parse', 'linked'))
        self.wait_for(lambda: self.starts() == 5)
        self.assertNotIn('overlap', self.events())

    def test_scoped_frontend_prepares_missing_or_changed_dependencies_once(self):
        web = self.root / 'apps/web'
        web.mkdir(parents=True)
        lock = web / 'package-lock.json'
        lock.write_text('locked version one')
        tools = self.root / 'bin'
        tools.mkdir()
        npm = tools / 'npm'
        npm.write_text('#!/usr/bin/env python3\nimport pathlib, sys\nroot = pathlib.Path.cwd()\nwith (root / "npm.log").open("a") as f: f.write(repr(sys.argv[1:]) + "\\n")\n(root / "apps/web/node_modules").mkdir(exist_ok=True)\n')
        npm.chmod(0o755)
        environment = {**os.environ, 'PATH': str(tools) + os.pathsep + os.environ['PATH']}
        def run():
            subprocess.run([str(self.binary), '--web-once', 'dev:site'], cwd=self.root, env=environment, check=True)
        run()
        run()
        self.assertEqual((self.root / 'npm.log').read_text().count("'ci'"), 1)
        lock.write_text('locked version two')
        run()
        self.assertEqual((self.root / 'npm.log').read_text().count("'ci'"), 2)
        (web / 'node_modules/.hickory-dev-lock').unlink()
        run()
        self.assertEqual((self.root / 'npm.log').read_text().count("'ci'"), 3)

    def test_outside_git_fails_before_starting_child(self):
        with tempfile.TemporaryDirectory(prefix='hickory-outside-git-') as folder:
            result = subprocess.run([str(self.binary), 'missing-child'], cwd=folder, capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('inside its Git checkout', result.stderr)

    def test_explicit_desktop_stop_ends_supervision(self):
        (self.root / 'scripts').mkdir()
        script = self.root / 'scripts/dev.sh'
        script.write_text('#!/bin/sh\nexit 0\n')
        script.chmod(0o755)
        self.process = subprocess.Popen([str(self.binary), '--desktop', os.sys.executable, str(self.child)], cwd=self.root, stdout=self.log, stderr=self.log)
        self.wait_for(lambda: self.starts() == 1 and 'ready' in self.events())
        pid = int((self.root / 'grandchild.pid').read_text())
        subprocess.run([str(self.binary), '--desktop-stop', 'clean'], cwd=self.root, check=True)
        with self.assertRaises(ProcessLookupError): os.kill(pid, 0)
        self.assertEqual(self.process.wait(timeout=25), 0)

if __name__ == '__main__':
    unittest.main()
</restart:copy>
<restart:file path="scripts/tests/test_dev_supervisor.py"><restart:paste select="#restart-witnesses"/></restart:file>
</restart:doc>
