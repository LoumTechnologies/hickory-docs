"""Refresh Simple Dev Apple from local Codex Stop events."""

import fcntl
import hashlib
import json
import os
from pathlib import Path
import shlex
import signal
import stat
import subprocess
import sys
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
STATE = Path.home() / "Library/Application Support/Hickory Docs/Codex Simple Dev"
QUEUE = STATE / "pending"
LOG = STATE / "preview.log"


def git(cwd, *args):
    return subprocess.check_output(["git", "-C", cwd, *args], stderr=subprocess.DEVNULL)


def git_path(cwd, option):
    path = git(cwd, "rev-parse", "--path-format=absolute", option).removesuffix(b"\n")
    return Path(os.fsdecode(path)).resolve()


def checkout(cwd):
    try:
        if git_path(cwd, "--git-common-dir") == git_path(ROOT, "--git-common-dir"):
            return git_path(cwd, "--show-toplevel")
    except (OSError, subprocess.CalledProcessError):
        pass
    return None


def publish(path, data, scratch=None):
    with tempfile.NamedTemporaryFile(dir=scratch or path.parent, delete=False) as handle:
        staged = Path(handle.name)
        try:
            handle.write(data)
            handle.close()
            staged.replace(path)
        finally:
            staged.unlink(missing_ok=True)


def start_worker():
    # Codex launched from the Dock may have a smaller PATH than the install shell.
    build_path = (STATE / "build-path").read_text()
    environment = {**os.environ, "PATH": build_path + os.pathsep + os.environ.get("PATH", "")}
    with LOG.open("ab") as log:
        subprocess.Popen(
            [sys.executable, str(Path(__file__).resolve()), "serve"],
            stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
            start_new_session=True,
            env=environment,
        )


def enqueue(event):
    if event.get("hook_event_name") != "Stop" or not isinstance(event.get("cwd"), str):
        return {}
    root = checkout(event["cwd"])
    if root is not None:
        QUEUE.mkdir(parents=True, exist_ok=True)
        # Stage outside the queue so the worker sees only complete requests.
        publish(QUEUE / uuid.uuid4().hex, os.fsencode(root), STATE)
        start_worker()
    return {}


def fingerprint(root):
    digest = hashlib.sha256(git(root, "rev-parse", "HEAD^{tree}"))
    digest.update(git(root, "diff", "--no-ext-diff", "--no-textconv", "--binary", "HEAD", "--"))
    untracked = git(root, "ls-files", "--others", "--exclude-standard", "-z")
    for name in sorted(filter(None, untracked.split(b"\0"))):
        path = root / os.fsdecode(name)
        try:
            mode = path.lstat().st_mode
            content = hashlib.sha256()
            if stat.S_ISLNK(mode):
                content.update(os.fsencode(os.readlink(path)))
            elif stat.S_ISREG(mode):
                with path.open("rb") as handle:
                    for chunk in iter(lambda: handle.read(128 * 1024), b""):
                        content.update(chunk)
            digest.update(name + b"\0" + str(mode).encode() + b"\0" + content.digest())
        except FileNotFoundError:
            # Another turn can remove a file while we read the snapshot.
            continue
    return digest.hexdigest()


def request_preview(root, supervisor):
    token = f"{uuid.uuid4().hex} {fingerprint(root)}"
    target = root / "target"
    target.mkdir(exist_ok=True)
    publish(target / "dev-desktop-request", token.encode())
    if supervisor is not None and supervisor.poll() is None:
        return supervisor
    return subprocess.Popen(
        [ROOT / "target/dev-supervisor", "--desktop-request",
         "cargo", "run", "--quiet", "--locked", "-p", "hickory-cli", "--example", "simple_dev", "--", "apple"],
        cwd=root,
        stdin=subprocess.DEVNULL,
    )


def run_requests(should_stop):
    supervisors = {}
    try:
        while not should_stop():
            supervisors = {root: proc for root, proc in supervisors.items() if proc.poll() is None}
            pending = {}
            for path in QUEUE.iterdir():
                root = Path(os.fsdecode(path.read_bytes()))
                pending.setdefault(root, []).append(path)
            if not supervisors and not pending:
                return
            for root, paths in pending.items():
                try:
                    if checkout(root) == root:
                        supervisors[root] = request_preview(root, supervisors.get(root))
                        print(f"Queued Apple preview for {root}", flush=True)
                except (OSError, subprocess.CalledProcessError) as error:
                    print(f"Could not queue Apple preview for {root}: {error}", file=sys.stderr, flush=True)
                finally:
                    for path in paths:
                        path.unlink(missing_ok=True)
            time.sleep(0.5)
    finally:
        for supervisor in supervisors.values():
            if supervisor.poll() is None:
                supervisor.terminate()
        for supervisor in supervisors.values():
            supervisor.wait()


def serve():
    stopping = False

    def stop(*_):
        nonlocal stopping
        stopping = True

    signal.signal(signal.SIGTERM, stop)
    with (STATE / "worker.lock").open("a") as lock:
        while not stopping:
            try:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                return
            try:
                run_requests(lambda: stopping)
            finally:
                fcntl.flock(lock, fcntl.LOCK_UN)
            # A hook may queue work while the previous worker is exiting.
            # Recheck after unlocking so either this worker or the new one sees it.
            if not any(QUEUE.iterdir()):
                return


def install():
    if sys.platform != "darwin":
        raise SystemExit("Codex Simple Dev installation requires macOS.")
    STATE.mkdir(parents=True, exist_ok=True)
    publish(STATE / "build-path", os.environ["PATH"].encode())
    hooks_path = Path(os.environ.get("CODEX_HOME", Path.home() / ".codex")) / "hooks.json"
    hooks_path.parent.mkdir(parents=True, exist_ok=True)
    original = hooks_path.read_bytes() if hooks_path.exists() else None
    config = json.loads(original) if original is not None else {}
    command = shlex.join([sys.executable, str(Path(__file__).resolve()), "enqueue"])
    entries = config.setdefault("hooks", {}).setdefault("Stop", [])
    if not any(hook.get("command") == command for entry in entries for hook in entry["hooks"]):
        entries.append({"hooks": [{"type": "command", "command": command, "timeout": 5}]})
        if original is not None:
            publish(hooks_path.with_name(f"hooks.json.before-hickory-{time.time_ns()}"), original)
        publish(hooks_path, (json.dumps(config, indent=2) + "\n").encode())
    print("Review the Hickory Stop hook in Codex Settings > Hooks or in the CLI with /hooks.")
    print(f"Preview log: {LOG}")


if __name__ == "__main__":
    action = sys.argv[1]
    if action == "enqueue":
        print(json.dumps(enqueue(json.load(sys.stdin))))
    elif action == "serve":
        serve()
    elif action == "install":
        install()
