"""Focused black-box checks integration; pass adapter and real local-ci paths."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import re
import signal
import selectors
import urllib.request


def main():
    adapter, provider, fixture = map(lambda value: str(Path(value).resolve()), sys.argv[1:4])
    hook = Path(__file__).resolve().parents[1] / ".githooks/post-commit"
    with tempfile.TemporaryDirectory(prefix="hickory-checks-") as directory:
        root = Path(directory)
        work = root / "work"
        work.mkdir()
        env = dict(os.environ, LOCAL_CI_HOME=str(root / "service"), HICKORY_STATE_DIR=str(root / "hickory-state"))
        def run(argv, cwd=work, ok=True):
            result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, text=True, timeout=30)
            assert not ok or result.returncode == 0, (argv, result.stdout, result.stderr)
            return result
        origin = None
        def observe():
            if origin:
                with urllib.request.urlopen(origin + "/api/checks", timeout=20) as response:
                    return json.load(response)
            return json.loads(run([adapter, "status"]).stdout)
        # Contract: .hickory-source/mechanisms/optional-checks.md
        assert observe()["availability"] == "disabled"
        (work / "note.md").write_text("# Readable scratch note\n")
        server = subprocess.Popen([fixture, str(work)], env=env, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        selector = selectors.DefaultSelector()
        selector.register(server.stdout, selectors.EVENT_READ)
        assert selector.select(20), "Native API fixture did not start"
        origin = server.stdout.readline().strip()
        selector.close()
        assert origin.startswith("http://127.0.0.1:")
        try:
            assert observe()["availability"] == "disabled"
            run([adapter, "configure", provider, sys.executable, "-c", "print('argv with spaces')"])
            assert observe()["availability"] == "unavailable"  # reads never start service
            submitted = json.loads(run([adapter, "run"]).stdout)["id"]
            deadline = time.monotonic() + 15
            while True:
                status = observe()
                row = next((r for r in status["runs"] if r["id"] == submitted), None)
                if row and row["status"] == "passed":
                    break
                assert time.monotonic() < deadline, status
                time.sleep(0.1)
            assert row["revision"] is None
            run(["git", "init", "-b", "master"])
            run(["git", "config", "user.name", "Checks test"])
            run(["git", "config", "user.email", "checks@example.invalid"])
            # No global hooks or signatures enter the disposable fixture.
            run(["git", "config", "core.hooksPath", str(root / "empty-hooks")])
            run(["git", "config", "commit.gpgsign", "false"])
            (work / "file").write_text("committed input\n")
            run(["git", "add", "file"])
            run(["git", "commit", "-m", "fixture"])
            revision = run(["git", "rev-parse", "HEAD"]).stdout.strip()
            immutable = json.loads(run([adapter, "run", revision]).stdout)["id"]
            deadline = time.monotonic() + 15
            while True:
                row = next((r for r in observe()["runs"] if r["id"] == immutable), None)
                if row and row["status"] == "passed":
                    break
                assert time.monotonic() < deadline, row
                time.sleep(0.1)
            assert row["revision"] == revision
            # Bring up the ordinary web host against the native API for one focused flow.
            repo = Path(__file__).resolve().parents[1]
            with (root / "vite.log").open("w+") as log:
                web_env = dict(env, HICKORY_API_ORIGIN=origin)
                web = subprocess.Popen(["npm", "--prefix", "apps/web", "run", "dev"], cwd=repo, env=web_env, stdout=log, stderr=log, start_new_session=True)
                try:
                    deadline = time.monotonic() + 20
                    while True:
                        log.seek(0)
                        match = re.search(r"Local:.*?(http://127\.0\.0\.1:\d+/)", log.read())
                        if match:
                            break
                        assert web.poll() is None and time.monotonic() < deadline, "Web host did not start"
                        time.sleep(0.1)
                    result = subprocess.run(["node", "scripts/test-checks-ui.mjs", match[1].rstrip("/"), provider], cwd=repo, env=env, capture_output=True, text=True, timeout=45)
                    assert result.returncode == 0, (result.stdout, result.stderr)
                    print(result.stdout.strip())
                finally:
                    os.killpg(web.pid, signal.SIGTERM)
                    web.wait(timeout=10)
            assert run([adapter, "run", "HEAD"], ok=False).returncode != 0
            # The Git adapter dispatches an immutable ID and exits before the spy.
            bin_dir = root / "bin"
            bin_dir.mkdir()
            marker = root / "dispatched.json"
            gate = root / "release-worker"
            spy = bin_dir / "just"
            spy.write_text(f"#!{sys.executable}\nimport json,sys,time\nfrom pathlib import Path\nwhile not Path({str(gate)!r}).exists(): time.sleep(0.01)\ntarget = Path({str(marker)!r})\ntemporary = target.with_suffix('.tmp')\ntemporary.write_text(json.dumps(sys.argv[1:]))\ntemporary.replace(target)\n")
            spy.chmod(0o755)
            env["PATH"] = str(bin_dir) + os.pathsep + env["PATH"]
            hooks = root / "hooks"
            hooks.mkdir()
            installed = hooks / "post-commit"
            installed.write_bytes(hook.read_bytes())
            installed.chmod(0o755)
            run(["git", "config", "core.hooksPath", str(hooks)])
            for mode in ["local-ci", "local"]:
                marker.unlink(missing_ok=True)
                gate.unlink(missing_ok=True)
                run(["git", "config", "hickory.postCommitChecks", mode])
                run(["git", "commit", "--allow-empty", "-m", "hook " + mode])
                assert not marker.exists(), "Git returned only after its detached worker finished"
                committed = run(["git", "rev-parse", "HEAD"]).stdout.strip()
                expected = ["checks-submit", committed] if mode == "local-ci" else ["checks"]
                if mode == "local-ci":
                    # Advancing HEAD before the detached worker wakes cannot retarget it.
                    run(["git", "config", "hickory.postCommitChecks", "off"])
                    run(["git", "commit", "--allow-empty", "-m", "newer commit"])
                gate.touch()
                deadline = time.monotonic() + 5
                while not marker.exists():
                    assert time.monotonic() < deadline
                    time.sleep(0.05)
                assert json.loads(marker.read_text()) == expected
            marker.unlink()
            run(["git", "config", "hickory.postCommitChecks", "off"])
            run(["git", "commit", "--allow-empty", "-m", "disabled hook"])
            time.sleep(1.2)
            assert not marker.exists()
            run([adapter, "configure", "--off"])
            assert observe()["availability"] == "disabled"
        finally:
            run([provider, "service", "stop"], ok=False)
            server.terminate()
            server.wait(timeout=10)
    print("Optional checks: real provider, immutable source and detached hook witnesses passed")


if __name__ == "__main__":
    main()
