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
