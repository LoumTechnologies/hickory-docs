import contextlib
import http.client
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import urllib.parse

HERE = Path(__file__).resolve().parents[1]

def command(*args, timeout=120):
    if args[:2] == ("dotnet", "build"): args = (*args, "-m:1", "-nr:false")
    result = subprocess.run(args, cwd=HERE, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout)
    if result.returncode:
        raise AssertionError(result.stdout)
    return result

def build(project=None):
    project = project or os.environ.get("EXPLICIT_TEST_HOST", "Server")
    command("dotnet", "build", f"{project}/{project}.csproj", "--nologo")

@contextlib.contextmanager
def server(directory, project=None, configuration="Debug", arguments=(), **settings):
    project = project or os.environ.get("EXPLICIT_TEST_HOST", "Server")
    directory = Path(directory)
    ready = directory / (project + ".address")
    ready.unlink(missing_ok=True)
    env = dict(os.environ, EXPLICIT_LISTEN_URL="http://127.0.0.1:0", EXPLICIT_ALLOWED_ORIGIN="https://reader.example",
        EXPLICIT_DATABASE=str(directory / "app.sqlite"), EXPLICIT_READY_FILE=str(ready), EXPLICIT_SECURE_COOKIE="false",
        EXPLICIT_MAX_BODY_BYTES="65536", EXPLICIT_REQUEST_TIMEOUT_SECONDS="10", EXPLICIT_SESSION_SECONDS="3600", EXPLICIT_LOGIN_ATTEMPTS="100")
    env.update(settings)
    with tempfile.TemporaryFile(mode="w+") as log:
        process = subprocess.Popen(["dotnet", str(HERE / project / ("bin/" + configuration + "/net10.0") / (project + ".dll")), *arguments], cwd=HERE, env=env, stdout=log, stderr=log)
        try:
            for _ in range(300):
                if ready.exists(): break
                if process.poll() is not None:
                    log.seek(0); raise AssertionError(log.read())
                time.sleep(.025)
            if not ready.exists():
                log.seek(0); raise AssertionError("Server failed to become ready: " + log.read())
            yield ready.read_text()
        finally:
            process.terminate()
            try: process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill(); process.wait(timeout=5)
            ready.unlink(missing_ok=True)

class Client:
    def __init__(self, address):
        self.address = address
        self.cookie = None
        self.csrf = None
    def request(self, method, path, body=None, headers=None, form=None):
        headers = dict(headers or {})
        if self.cookie: headers.setdefault("Cookie", self.cookie)
        if form is not None:
            body = urllib.parse.urlencode(form).encode(); headers.setdefault("Content-Type", "application/x-www-form-urlencoded")
        elif isinstance(body, (dict, list)):
            body = json.dumps(body).encode(); headers.setdefault("Content-Type", "application/json")
        url = urllib.parse.urlsplit(self.address)
        connection = http.client.HTTPConnection(url.hostname, url.port, timeout=10)
        try:
            connection.request(method, path, body=body, headers=headers)
            response = connection.getresponse()
            data = response.read(); result_headers = dict(response.getheaders())
            if "Set-Cookie" in result_headers: self.cookie = result_headers["Set-Cookie"].split(';')[0]
            return response.status, result_headers, data
        finally: connection.close()
    def session(self):
        status, _, body = self.request("GET", "/api/session")
        assert status == 200
        data = json.loads(body); self.csrf = data["csrf"]; return data
    def register(self, name):
        self.session()
        status, _, _ = self.request("POST", "/register", form={"name": name, "password": "correct horse battery", "csrf": self.csrf})
        assert status == 303, status
        self.session()
    def write(self, method, path, body):
        return self.request(method, path, body, {"X-CSRF-Token": self.csrf})
