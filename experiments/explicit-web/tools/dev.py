import os
from pathlib import Path
import subprocess
import signal
import sys
import time

HERE = Path(__file__).resolve().parents[1]

def main():
    def interrupt(signum, frame): raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupt)
    state = HERE / ".dev"; state.mkdir(exist_ok=True)
    ready = state / "server.address"
    # A PID record prevents a second launcher from overwriting a live ready file.
    lock = state / "launcher.lock"
    try: fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    except FileExistsError: raise SystemExit("A launcher lock exists at " + str(lock) + ". Stop the other launcher first; if it crashed, remove this lock after checking that its server stopped.")
    os.close(fd)
    try:
        build = subprocess.run(["dotnet", "build", "Server/Server.csproj", "--nologo", "-m:1", "-nr:false"], cwd=HERE)
        if build.returncode: return build.returncode
        ready.unlink(missing_ok=True)
        defaults = dict(EXPLICIT_LISTEN_URL="http://127.0.0.1:0", EXPLICIT_ALLOWED_ORIGIN="https://reader.example",
            EXPLICIT_DATABASE=str(state / "app.sqlite"), EXPLICIT_READY_FILE=str(ready), EXPLICIT_SECURE_COOKIE="false",
            EXPLICIT_MAX_BODY_BYTES="65536", EXPLICIT_REQUEST_TIMEOUT_SECONDS="10", EXPLICIT_SESSION_SECONDS="86400", EXPLICIT_LOGIN_ATTEMPTS="20")
        env = dict(os.environ)
        for key, value in defaults.items(): env.setdefault(key, value)
        ready = Path(env["EXPLICIT_READY_FILE"])
        ready.unlink(missing_ok=True)
        with (state / "server.log").open("a") as log:
            child = subprocess.Popen(["dotnet", str(HERE / "Server/bin/Debug/net10.0/Server.dll")], cwd=HERE, env=env, stdout=log, stderr=log)
            try:
                for _ in range(300):
                    if ready.exists(): break
                    if child.poll() is not None: raise SystemExit("Server stopped during startup. Inspect " + str(state / "server.log"))
                    time.sleep(.025)
                if not ready.exists(): raise SystemExit("Server did not become ready. Inspect " + str(state / "server.log"))
                print("Open " + ready.read_text() + " — Ctrl+C stops the server.", flush=True)
                try: return child.wait()
                except KeyboardInterrupt: return 0
            finally:
                child.terminate()
                try: child.wait(timeout=15)
                except subprocess.TimeoutExpired: child.kill(); child.wait(timeout=5)
                ready.unlink(missing_ok=True)
    finally: lock.unlink(missing_ok=True)

if __name__ == "__main__": raise SystemExit(main())
