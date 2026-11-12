<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Verify native app updates on commits

For maintainers checking real iOS simulator or Android device reloads. Select an already running target explicitly. This test starts the same shared supervisor and mobile session-child used by the development runner, but watches a disposable Git repository so it never commits or changes revisions in this checkout. It performs real builds, installations and launches after a first commit and an amend, checks a private app-data sentinel and changed process IDs, then cleans up its session and sentinel. It leaves the selected device and preexisting data intact. It does not verify editor rendering or physical iPhone signing.

<mobile:copy id="simple-mobile-reload-smoke">
#!/usr/bin/env python3
"""Real SDK reload acceptance on an explicitly selected, already running target."""
import argparse
import json
import os
from pathlib import Path
import re
import shlex
import signal
import subprocess
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT, **kwargs).strip()


class Device:
    def __init__(self, platform):
        self.platform = platform
        key = 'ANDROID_SERIAL' if platform == 'android' else 'HICKORY_IOS_DEVICE'
        self.target = os.environ.get(key)
        if not self.target:
            raise RuntimeError(f'Set {key} to an explicit, already running target for this test')
        self.package = 'com.hickorydocs.simple' + ('.mobile' if platform == 'ios' else '')
        self.name = 'hickory-reload-' + uuid.uuid4().hex
        self.token = uuid.uuid4().hex
        self.sentinel = None
        if platform == 'android':
            sdk = os.environ.get('ANDROID_HOME') or os.environ.get('ANDROID_SDK_ROOT')
            if not sdk:
                raise RuntimeError('Set ANDROID_HOME or ANDROID_SDK_ROOT to the installed SDK')
            self.adb = str(Path(sdk) / 'platform-tools' / ('adb.exe' if os.name == 'nt' else 'adb'))
            assert self.android('get-state') == 'device'
            self.boot_id = self.android('shell', 'cat', '/proc/sys/kernel/random/boot_id')
        else:
            listing = json.loads(self.ios('list', 'devices', 'available', '--json'))
            targets = [device for group in listing['devices'].values() for device in group]
            assert any(device['udid'] == self.target and device['state'] == 'Booted' for device in targets), 'HICKORY_IOS_DEVICE must be a booted simulator UDID'

    def android(self, *args):
        return run(self.adb, '-s', self.target, *args)

    def ios(self, *args):
        return run('xcrun', 'simctl', *args)

    def private(self, script):
        return self.android('shell', f'run-as {self.package} sh -c {shlex.quote(script)}')

    def pid(self, log):
        if self.platform == 'android':
            return self.android('shell', 'pidof', self.package)
        matches = re.findall(re.escape(self.package) + r': (\d+)', log)
        assert matches, 'simctl launch did not report the app PID'
        pid = matches[-1]
        os.kill(int(pid), 0)
        return pid

    def write_sentinel(self):
        if self.platform == 'android':
            self.private(f'mkdir -p files && printf %s {self.token} > files/{self.name}')
            self.sentinel = True
        else:
            container = Path(self.ios('get_app_container', self.target, self.package, 'data'))
            self.sentinel = container / 'Documents' / self.name
            self.sentinel.parent.mkdir(exist_ok=True)
            self.sentinel.write_text(self.token)

    def assert_preserved(self):
        if self.platform == 'android':
            assert self.private(f'cat files/{self.name}') == self.token, 'Android app data was lost'
            assert self.android('shell', 'cat', '/proc/sys/kernel/random/boot_id') == self.boot_id, 'Emulator restarted during app reload'
        else:
            container = Path(self.ios('get_app_container', self.target, self.package, 'data'))
            self.sentinel = container / 'Documents' / self.name
            assert self.sentinel.read_text() == self.token, 'iOS app data was lost'

    def cleanup(self):
        if self.sentinel is not None:
            if self.platform == 'android':
                self.private(f'rm -f files/{self.name}')
            else:
                self.sentinel.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('platform', choices=['ios', 'android'])
    parser.add_argument('--timeout', type=float, default=600)
    args = parser.parse_args()
    device = Device(args.platform)
    subprocess.run(['just', '_dev-supervisor'], cwd=ROOT, check=True)
    with tempfile.TemporaryDirectory(prefix='hickory-mobile-reload-') as folder:
        root = Path(folder)
        project = root / 'project'
        project.mkdir()
        (project / 'hello.md').write_text('# Mobile reload acceptance\n\nKeep this workspace and app data across revisions.\n')
        for command in [
            ['init', '-q'], ['config', 'core.hooksPath', '/dev/null'],
            ['config', 'user.name', 'Mobile reload acceptance'],
            ['config', 'user.email', 'test@example.invalid'],
            ['config', 'commit.gpgsign', 'false'],
        ]:
            run('git', *command, cwd=root)
        environment = {**os.environ, 'HICKORY_PROJECT_DIR': str(project)}
        command = [str(ROOT / 'target/dev-supervisor'), 'cargo', 'run', '--quiet', '--locked',
                   '--manifest-path', str(ROOT / 'Cargo.toml'), '-p', 'hickory-cli',
                   '--example', 'simple_dev', '--', args.platform, '--session-child']
        logpath = root / 'reload.log'
        marker = ('iOS' if args.platform == 'ios' else 'Android') + f' running on {device.target}.'
        with logpath.open('w') as log:
            process = subprocess.Popen(command, cwd=root, env=environment, stdout=log, stderr=log)
            def ready(count):
                deadline = time.monotonic() + args.timeout
                while time.monotonic() < deadline:
                    text = logpath.read_text()
                    if text.count(marker) >= count:
                        return device.pid(text)
                    if process.poll() is not None:
                        raise RuntimeError('Reload supervisor exited: ' + '\n'.join(text.splitlines()[-25:]))
                    if 'waiting for the next commit' in text.lower():
                        raise RuntimeError('Native update failed: ' + '\n'.join(text.splitlines()[-25:]))
                    time.sleep(.2)
                raise RuntimeError('Native startup timed out: ' + '\n'.join(logpath.read_text().splitlines()[-25:]))
            try:
                previous = ready(1)
                device.write_sentinel()
                for count in [2, 3]:
                    (root / 'revision').write_text(str(time.time_ns()))
                    run('git', 'add', 'revision', cwd=root)
                    commit = ['commit', '-qm', 'reload witness']
                    if count == 3:
                        commit.append('--amend')
                    run('git', *commit, cwd=root)
                    current = ready(count)
                    assert current != previous, 'App process did not restart'
                    device.assert_preserved()
                    previous = current
                process.send_signal(signal.SIGINT)
                assert process.wait(timeout=45) == 0, 'Ctrl-C cleanup failed'
                print(f'PASS: {args.platform} rebuilt, updated and relaunched after commit and amend on {device.target}; app data and device lifetime were preserved.')
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=45)
                device.cleanup()


if __name__ == '__main__':
    main()
</mobile:copy>
<mobile:file path="scripts/test-simple-mobile-reload.py"><mobile:paste select="#simple-mobile-reload-smoke"/></mobile:file>
</mobile:doc>
