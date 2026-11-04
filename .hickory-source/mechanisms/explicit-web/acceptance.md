<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Drive the application through HTTP and a browser

For maintainers checking the advertised app, independent of its implementation stack. The HTTP client knows only URLs, JSON, forms and cookies. The launcher is the replaceable .NET adapter. Compiler witnesses protect source-level constraints that an HTTP test cannot observe. All tests refer to the contract in docs/guarantees/basic-app.md; broad Hickory regressions are not part of this focused suite.

## Test launcher

Use a temporary data directory and port zero. The launcher waits for an explicit ready file, captures logs without streaming them, and terminates the entire server before returning. The same client works against either hosting arm and preserves cookies across restarts.

<web:copy id="ew-test-launcher">
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
</web:copy>

## Http acceptance

Drive the complete signup-to-CRUD path, then probe malformed input, CSRF, stale edits, SQL/HTML injection, foreign ownership, CORS, assets, methods and persistence. A restart reuses both the database and the session cookie. Assertions on denied requests re-read persisted state, so a refusal that occurs after mutation fails the suite.

<web:copy id="ew-http-acceptance">
# Contract: docs/guarantees/basic-app.md
import concurrent.futures
import json
import tempfile
import unittest
from support import Client, build, server

class BasicApp(unittest.TestCase):
    @classmethod
    def setUpClass(cls): build()

    def test_complete_app_and_safety_boundaries(self):
        with tempfile.TemporaryDirectory() as directory:
            with server(directory) as address:
                alice, bob, guest = Client(address), Client(address), Client(address)
                status, headers, body = guest.request("GET", "/")
                self.assertEqual(status, 200); self.assertIn(b"Create account", body)
                self.assertIn("HttpOnly", headers["Set-Cookie"]); self.assertIn("SameSite=Lax", headers["Set-Cookie"])
                self.assertEqual(guest.request("GET", "/api/tasks")[0], 401)
                alice.register("alice"); bob.register("bob")
                old_cookie, old_csrf = alice.cookie, alice.csrf
                self.assertEqual(alice.write("POST", "/api/tasks", {"title": ""})[0], 422)
                self.assertEqual(alice.request("POST", "/api/tasks", {"title": "must not exist"})[0], 403)
                self.assertEqual(alice.write("POST", "/api/tasks", {"title": "x", "unexpected": 1})[0], 400)
                self.assertEqual(alice.write("POST", "/api/tasks", b"{bad")[0], 415)
                self.assertEqual(alice.request("POST", "/api/tasks", b"{bad", {"Content-Type": "application/json", "X-CSRF-Token": alice.csrf})[0], 400)
                malicious = '<img src=x onerror=alert(1)>' + chr(39) + " OR 1=1 --"
                status, headers, body = alice.write("POST", "/api/tasks", {"title": malicious})
                self.assertEqual(status, 201); task = json.loads(body); path = headers["Location"]
                self.assertEqual(task["version"], 1)
                status, _, page = alice.request("GET", "/tasks/" + task["id"])
                self.assertEqual(status, 200); self.assertNotIn(b"<img", page); self.assertIn(b"&lt;img", page)
                self.assertEqual(json.loads(bob.request("GET", "/api/tasks")[2]), [])
                self.assertEqual(bob.request("GET", path)[0], 404)
                self.assertEqual(bob.write("PUT", path, {"title": "stolen", "version": 1})[0], 404)
                self.assertEqual(bob.write("DELETE", path, {"version": 1})[0], 404)
                self.assertEqual(alice.write("PUT", path, {"title": "changed", "done": True, "version": 1})[0], 200)
                self.assertEqual(alice.write("PUT", path, {"title": "stale", "version": 1})[0], 409)
                self.assertEqual(alice.write("DELETE", path, {"version": 1})[0], 409)
                current = json.loads(alice.request("GET", path)[2]); self.assertEqual(current["title"], "changed")
                self.assertTrue(current["done"]); self.assertEqual(current["version"], 2)
                # Two simultaneous edits of one version must produce one winner.
                def edit(title): return alice.write("PUT", path, {"title": title, "version": 2})[0]
                with concurrent.futures.ThreadPoolExecutor(2) as workers:
                    self.assertEqual(sorted(workers.map(edit, ["winner a", "winner b"])), [200, 409])
                current = json.loads(alice.request("GET", path)[2]); self.assertEqual(current["version"], 3)
                self.assertEqual(alice.request("GET", "/api/tasks/not-a-guid")[0], 400)
                self.assertEqual(alice.request("PATCH", path)[0], 405)
                status, headers, body = alice.request("HEAD", "/")
                self.assertEqual(status, 200); self.assertEqual(body, b""); self.assertGreater(int(headers["Content-Length"]), 0)
                status, headers, css = guest.request("GET", "/site.css")
                self.assertEqual(status, 200); self.assertIn("text/css", headers["Content-Type"])
                self.assertEqual(guest.request("GET", "/site.css", headers={"If-None-Match": headers["ETag"]})[0], 304)
                self.assertEqual(guest.request("GET", "/../app.sqlite")[0], 404)
                for origin in ["https://evil.example", "null", "https://reader.example.evil"]:
                    status, headers, _ = alice.request("GET", path, headers={"Origin": origin})
                    self.assertEqual(status, 403); self.assertNotIn("Access-Control-Allow-Origin", headers)
                status, headers, _ = guest.request("GET", "/missing", headers={"Origin": "https://reader.example"})
                self.assertEqual(status, 404); self.assertEqual(headers["Access-Control-Allow-Origin"], "https://reader.example")
                preflight = {"Origin": "https://reader.example", "Access-Control-Request-Method": "PUT", "Access-Control-Request-Headers": "content-type,x-csrf-token"}
                status, headers, body = guest.request("OPTIONS", path, headers=preflight)
                self.assertEqual(status, 204); self.assertEqual(body, b""); self.assertEqual(headers["Access-Control-Allow-Credentials"], "true")
                preflight["Access-Control-Request-Headers"] = "x-unknown"
                self.assertEqual(guest.request("OPTIONS", path, headers=preflight)[0], 403)
                self.assertEqual(alice.request("POST", "/api/tasks", b"x" * 65537, {"X-CSRF-Token": alice.csrf, "Content-Type": "application/json"})[0], 413)
                self.assertEqual(len(json.loads(alice.request("GET", "/api/tasks")[2])), 1)
            with server(directory) as address:
                alice.address = address
                self.assertEqual(alice.session()["user"], "alice")
                self.assertEqual(json.loads(alice.request("GET", path)[2])["version"], 3)
                self.assertEqual(alice.write("DELETE", path, {"version": 3})[0], 204)
                self.assertEqual(alice.request("GET", path)[0], 404)
                self.assertEqual(alice.request("POST", "/logout", form={"csrf": alice.csrf})[0], 303)
                replay = Client(address); replay.cookie = old_cookie
                self.assertEqual(replay.request("GET", "/api/tasks")[0], 401)
                alice.session()
                self.assertEqual(alice.request("POST", "/login", form={"name": "alice", "password": "incorrect password", "csrf": alice.csrf})[0], 401)
                self.assertEqual(alice.request("POST", "/login", form={"name": "alice", "password": "correct horse battery", "csrf": alice.csrf})[0], 303)
                self.assertNotEqual(alice.cookie, old_cookie)
                alice.session()
                self.assertEqual(alice.request("POST", "/tasks", form={"csrf": alice.csrf, "title": "form task"})[0], 303)
                task = json.loads(alice.request("GET", "/api/tasks")[2])[0]
                form_path = "/tasks/" + task["id"]
                form = {"csrf": alice.csrf, "title": "form edited", "done": "true", "version": 1, "action": "save"}
                self.assertEqual(alice.request("POST", form_path, form=form)[0], 303)
                form.update(version=2, action="delete")
                self.assertEqual(alice.request("POST", form_path, form=form)[0], 303)
                self.assertEqual(json.loads(alice.request("GET", "/api/tasks")[2]), [])

    def test_session_expiry_and_login_throttle(self):
        import time
        with tempfile.TemporaryDirectory() as directory, server(directory, EXPLICIT_SESSION_SECONDS="2", EXPLICIT_LOGIN_ATTEMPTS="2") as address:
            client = Client(address); client.register("expiry")
            time.sleep(2.1)
            self.assertEqual(client.request("GET", "/api/tasks")[0], 401)
            client.session()
            fields = {"csrf": client.csrf, "name": "expiry", "password": "wrong password"}
            self.assertEqual(client.request("POST", "/login", form=fields)[0], 401)
            self.assertEqual(client.request("POST", "/login", form=fields)[0], 429)

if __name__ == "__main__": unittest.main(verbosity=2)
</web:copy>

## Test entry

Keep one short focused entry point for the app HTTP contract. Compiler and browser witnesses are separate commands so their costs and outcomes remain distinguishable.

<web:copy id="ew-test-entry">
import subprocess
import sys
from pathlib import Path

if __name__ == "__main__":
    raise SystemExit(subprocess.call([sys.executable, "-m", "unittest", "discover", "-s", "tests", "-p", "test_*.py", "-v"], cwd=Path(__file__).resolve().parent))
</web:copy>

## Browser flow

Drive actual form controls in Chromium with JavaScript disabled. Use role/label locators so the test follows the accessible UI, including the edit and delete forms. The launcher supplies a temporary server URL. Playwright is a development-only dependency installed in the parent repository.

<web:copy id="ew-browser-flow">
const { chromium } = require(process.env.EXPLICIT_PLAYWRIGHT_MODULE);
(async () => {
  const browser = await chromium.launch({headless: true, ...(process.env.EXPLICIT_BROWSER_EXECUTABLE ? {executablePath: process.env.EXPLICIT_BROWSER_EXECUTABLE} : {}), ...(process.env.EXPLICIT_BROWSER_CHANNEL ? {channel: process.env.EXPLICIT_BROWSER_CHANNEL} : {})});
  try {
    const context = await browser.newContext({javaScriptEnabled: false});
    const page = await context.newPage();
    await page.goto(process.env.EXPLICIT_BROWSER_URL);
    await page.getByRole('link', {name: 'Create account', exact: true}).click();
    await page.getByLabel('Username').fill('browser_user');
    await page.getByLabel('Password').fill('browser password twelve');
    await page.getByRole('button', {name: 'Create account', exact: true}).click();
    await page.getByLabel('New task').fill('<script>unsafe</script>');
    await page.getByRole('button', {name: 'Add task'}).click();
    await page.getByRole('link', {name: '<script>unsafe</script>', exact: true}).click();
    await page.getByLabel('Title').fill('Browser edited task');
    await page.getByLabel('Status').selectOption('true');
    await page.getByRole('button', {name: 'Save changes'}).click();
    if (await page.getByText('Complete', {exact: true}).count() !== 1) throw new Error('Completed state missing');
    await page.getByRole('link', {name: 'Browser edited task'}).click();
    await page.getByRole('button', {name: 'Delete task'}).click();
    if (await page.getByRole('link', {name: 'Browser edited task'}).count()) throw new Error('Task was not deleted');
    await page.getByRole('button', {name: 'Sign out'}).click();
    await page.getByRole('link', {name: 'Sign in', exact: true}).click();
    await page.getByLabel('Username').fill('browser_user');
    await page.getByLabel('Password').fill('browser password twelve');
    await page.getByRole('button', {name: 'Sign in', exact: true}).click();
    await page.getByRole('button', {name: 'Sign out'}).waitFor();
    console.log('PASS: real-browser registration, escaped task, edit, completion, deletion, logout and login');
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
</web:copy>

## Browser launcher

Keep process setup separate from the browser’s stack-independent user actions. A missing Playwright module is reported as a prerequisite, not counted as a pass.

<web:copy id="ew-browser-launcher">
import os
from pathlib import Path
import subprocess
import tempfile
from support import HERE, build, server

if __name__ == "__main__":
    build()
    module = Path(os.environ.get("EXPLICIT_PLAYWRIGHT_MODULE", str(HERE / "tests/node_modules/@playwright/test")))
    if not module.exists(): module = HERE.parents[1] / "apps/web/node_modules/@playwright/test"
    if not module.exists(): raise SystemExit("Install the parent repo's apps/web dependencies before running the browser witness.")
    with tempfile.TemporaryDirectory() as directory, server(directory) as address:
        env = dict(os.environ, EXPLICIT_BROWSER_URL=address, EXPLICIT_PLAYWRIGHT_MODULE=str(module))
        raise SystemExit(subprocess.call(["node", str(HERE / "tests/browser.cjs")], env=env))
</web:copy>

## Checks checks csproj

Keep assembly boundaries real: internal constructors cannot be called by consumers. Nullable warnings are errors; deterministic builds and dependency locks make evaluation reproducible.

<web:copy id="ew-Checks-Checks-csproj">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework><Nullable>enable</Nullable><ImplicitUsings>enable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors><UseAppHost>false</UseAppHost>
    <RestorePackagesWithLockFile>true</RestorePackagesWithLockFile><Deterministic>true</Deterministic>
  </PropertyGroup>
  <ItemGroup><ProjectReference Include="../Server/Server.csproj" /></ItemGroup>
</Project>
</web:copy>

## Runtime safety

Exercise boundaries below HTTP: SQL rollback and parameter bytes, retained rows after disposal, same-session capability checks, typed link round trips, and safe redirect parsing. An exception fails the executable. These checks cannot establish cross-platform native ABI behavior; CI runs the same witnesses on supported hosts.

<web:copy id="ew-runtime-safety">
using ExplicitWeb;
using ExplicitWeb.Data;
using ExplicitWeb.Security;
using TaskApp;

if (args.Contains("--serve-errors"))
{
    var settings = TaskApp.Settings.Load();
    using var stop = new CancellationTokenSource();
    using var signal = OperatingSystem.IsWindows() ? null : System.Runtime.InteropServices.PosixSignalRegistration.Create(System.Runtime.InteropServices.PosixSignal.SIGTERM, c => { c.Cancel = true; stop.Cancel(); });
    await ExplicitWeb.Hosting.Server.Run(settings.Server, async request =>
    {
        if (request.Path == "/throw") throw new InvalidOperationException("PRIVATE_EXCEPTION_MARKER");
        if (request.Path == "/timeout") await Task.Delay(Timeout.Infinite, request.Cancel);
        return new Cors([], [], []).Inspect(request).Apply(Response.Text("ok"));
    }, uri => File.WriteAllText(settings.ReadyFile, uri.GetLeftPart(UriPartial.Authority)), stop.Token);
    return;
}
void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
T Value<T>(Result<T> result) where T : notnull => result.Match(x => x, p => throw new Exception(p.Message));
var directory = Path.Combine(Path.GetTempPath(), "explicit-check-" + Guid.NewGuid()); Directory.CreateDirectory(directory);
try
{
    var database = new Database(Path.Combine(directory, "checks.sqlite")); Schema.Initialize(database); Schema.Initialize(database);
    database.Transaction(sql => { sql.Execute($"CREATE TABLE probes(text TEXT, bytes BLOB)"); return true; });
    var text = "Unicode ☃ ' ; DROP TABLE users; --" + (char)0 + "tail"; var bytes = new byte[] { 0, 127, 255 };
    database.Transaction(sql => { sql.Execute($"INSERT INTO probes VALUES({text},{bytes})"); return true; });
    Check(database.Transaction(sql => sql.Query($"SELECT {1.25}", r => r.GetDouble(0)).Single()) == 1.25, "Double round trip failed");
    Check(database.Transaction(sql => sql.Query($"SELECT {Array.Empty<byte>()}", r => !r.IsDBNull(0) && ((byte[])r[0]).Length == 0).Single()), "Empty BLOB became NULL");
    var row = database.Transaction(sql => sql.Query($"SELECT text,bytes FROM probes", r => r).Single());
    Check(row.GetString(0) == text && ((byte[])row[1]).SequenceEqual(bytes), "Parameter or row lifetime failure");
    try { database.Transaction<bool>(sql => { sql.Execute($"DELETE FROM probes"); throw new InvalidOperationException("rollback"); }); }
    catch (InvalidOperationException) { }
    Check(database.Transaction(sql => sql.Scalar($"SELECT COUNT(*) FROM probes")) == 1, "Transaction did not roll back");
    var sessions = new Sessions(database, TimeProvider.System, TimeSpan.FromHours(1), true);
    Request Guest() => new("GET", "/", "local", new Dictionary<string,string>(), []);
    var a = sessions.Load(Guest()); var b = sessions.Load(Guest());
    var write = new Request("POST", "/tasks", "local", new Dictionary<string,string>(), []);
    Check(a.Verify(write, b.Csrf).Match(_ => false, p => p.Status == 403), "Cross-session CSRF accepted");
    Check(a.Verify(Guest(), a.Csrf).Match(_ => false, p => p.Status == 405), "Safe request got mutation proof");
    var accounts = new Accounts(database, sessions);
    var signed = Value(accounts.Register(Value(a.Verify(write, a.Csrf)), Value(Credentials.Parse("runtime", "correct horse battery"))));
    Check(signed.NewCookie!.Contains("; Secure"), "Secure cookie flag missing");
    var principal = Value(signed.RequireUser());
    Check(principal.Authorize(Value(b.Verify(write, b.Csrf))).Match(_ => false, p => p.Status == 403), "Cross-session proof joined");
    var route = Routes.Id("GET", "/tasks/{id}"); var id = Guid.NewGuid();
    Check(Value(route.Match(new("GET", route.Link(id).Value, "local", new Dictionary<string,string>(), []))).Value == id, "Typed route round trip failed");
    foreach (var path in new[]{"//evil.example", "/%2fevil", "/bad\\path", "https://evil.example"})
        Check(LocalUrl.Parse(path).Match(_ => false, p => p.Status == 400), "Unsafe redirect accepted");
    Check(Html.Template($"<p>{"<script>"}</p>").Encoded == "<p>&lt;script&gt;</p>", "HTML interpolation was not encoded");
    Console.WriteLine("PASS: transaction rollback, parameter binding, row lifetime, sessions, capability binding, routes, redirects and HTML encoding");
}
finally { Directory.Delete(directory, true); }
</web:copy>

## Browser package

Pin the browser driver used in CI. Local verification may use the already installed parent package or a caller-selected module; the evidence report must name that choice.

<web:copy id="ew-browser-package">
{"name":"explicit-web-browser-checks","private":true,"devDependencies":{"@playwright/test":"1.63.0"}}
</web:copy>

## Browser lock

Retain exact registry digests for the three browser packages, drawn from the repository’s already installed lockfile. npm ci verifies these digests; no new dependency version is inferred during CI.

<web:copy id="ew-browser-lock">
{
  "name": "explicit-web-browser-checks",
  "lockfileVersion": 3,
  "requires": true,
  "packages": {
    "": {
      "name": "explicit-web-browser-checks",
      "devDependencies": {
        "@playwright/test": "1.63.0"
      }
    },
    "node_modules/@playwright/test": {
      "version": "1.63.0",
      "resolved": "https://registry.npmjs.org/@playwright/test/-/test-1.63.0.tgz",
      "integrity": "sha512-oxMK4vllB9RK5NQ2l1pq1IfOf2AvnEuj/vYGDj0H2nMtmtZpKtCwt/l00GEO6xjGfpBNAvjovvYdCm50dRQkpQ==",
      "dev": true,
      "license": "Apache-2.0",
      "dependencies": {
        "playwright": "1.63.0"
      },
      "bin": {
        "playwright": "cli.js"
      },
      "engines": {
        "node": ">=20"
      }
    },
    "node_modules/playwright": {
      "version": "1.63.0",
      "resolved": "https://registry.npmjs.org/playwright/-/playwright-1.63.0.tgz",
      "integrity": "sha512-+7ziBLidS4NaNCdt57SUDT+wYmmd5fmiQejUic/kb+YsYSCPyOOE9sebzMjNmQrsnNpDJqd4WHvV/8lfKfUDUg==",
      "dev": true,
      "license": "Apache-2.0",
      "dependencies": {
        "playwright-core": "1.63.0"
      },
      "bin": {
        "playwright": "cli.js"
      },
      "engines": {
        "node": ">=20"
      }
    },
    "node_modules/playwright-core": {
      "version": "1.63.0",
      "resolved": "https://registry.npmjs.org/playwright-core/-/playwright-core-1.63.0.tgz",
      "integrity": "sha512-rYCsBF/M5HjUch52bbtVONEFjv6Xu8sm8h72dNlR5bzIE1fvC/bxgspzkjSfU+MweEMmPM8KJebG6nnyxo5mCg==",
      "dev": true,
      "license": "Apache-2.0",
      "bin": {
        "playwright-core": "cli.js"
      },
      "engines": {
        "node": ">=20"
      }
    }
  }
}
</web:copy>

## Transport refusals

Use a separate fixture executable to throw inside a real request handler and to cooperate with cancellation. Verify that neither private exception details nor successful bodies escape. Check chunked input without a Content-Length so the explicit buffering bound is exercised too.

<web:copy id="ew-transport-refusals">
# Contract: docs/guarantees/basic-app.md
import http.client
import tempfile
import unittest
import urllib.parse
from support import Client, build, server

class Transport(unittest.TestCase):
    @classmethod
    def setUpClass(cls): build("Checks")
    def test_failures_and_chunked_limits(self):
        with tempfile.TemporaryDirectory() as directory, server(directory, "Checks", arguments=["--serve-errors"], EXPLICIT_REQUEST_TIMEOUT_SECONDS="1") as address:
            client = Client(address)
            status, _, body = client.request("GET", "/throw")
            self.assertEqual(status, 500); self.assertNotIn(b"PRIVATE_EXCEPTION_MARKER", body); self.assertIn(b"request ID", body)
            self.assertEqual(client.request("GET", "/timeout")[0], 408)
            self.assertEqual(client.request("GET", "/ok")[0], 200)
            url = urllib.parse.urlsplit(address)
            connection = http.client.HTTPConnection(url.hostname, url.port, timeout=5)
            try:
                connection.request("POST", "/ok", body=iter([b"x"*32768, b"y"*32768, b"z"]), encode_chunked=True)
                response = connection.getresponse(); response.read()
                self.assertEqual(response.status, 413)
            finally: connection.close()

if __name__ == "__main__": unittest.main(verbosity=2)
</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/tests/support.py"><web:paste select="#ew-test-launcher"/></web:file>
<web:file path="experiments/explicit-web/tests/test_app.py"><web:paste select="#ew-http-acceptance"/></web:file>
<web:file path="experiments/explicit-web/test_http.py"><web:paste select="#ew-test-entry"/></web:file>
<web:file path="experiments/explicit-web/tests/browser.cjs"><web:paste select="#ew-browser-flow"/></web:file>
<web:file path="experiments/explicit-web/tests/browser.py"><web:paste select="#ew-browser-launcher"/></web:file>
<web:file path="experiments/explicit-web/Checks/Checks.csproj"><web:paste select="#ew-Checks-Checks-csproj"/></web:file>
<web:file path="experiments/explicit-web/Checks/Program.cs"><web:paste select="#ew-runtime-safety"/></web:file>
<web:file path="experiments/explicit-web/tests/package.json"><web:paste select="#ew-browser-package"/></web:file>
<web:file path="experiments/explicit-web/tests/package-lock.json"><web:paste select="#ew-browser-lock"/></web:file>
<web:file path="experiments/explicit-web/tests/test_transport.py"><web:paste select="#ew-transport-refusals"/></web:file>

</web:doc>
