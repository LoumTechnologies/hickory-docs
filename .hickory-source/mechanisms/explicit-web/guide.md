<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Run, extend, and export the framework

For a C# developer building their first app with these libraries. The framework is intentionally small enough to read but supports an end-to-end persistent web application. The task app is a reference, not a required domain model. These documents distinguish supported behavior, tested claims, development-only measurements and production limitations.

## Readme

Lead with the path to a running app and the construction readers should open. Keep the framework’s scope and the evaluation limits next to the claims.

<web:copy id="ew-readme">
# Explicit Web

A C# web framework built from libraries and ordinary functions. Follow a request
from origin decision to authenticated, validated operation to prepared response.
There is no middleware registration API. Kestrel handles HTTP transport directly
through `IHttpApplication`; application policy stays in the application function.

The included task app has registration/login/logout, durable SQLite sessions,
private CRUD, server-rendered forms, a JSON API, optimistic edit conflicts,
CSRF/CORS decisions, encoded HTML, CSS assets, body limits and request logging.

From this directory:

```sh
just dev
```

Open the URL it prints. Requires just, .NET 10 SDK, Python 3, and the system SQLite
library. macOS and Linux normally provide SQLite; Windows needs sqlite3.dll on
the library search path. No third-party NuGet packages are required. Stop with
Ctrl+C. State remains in `.dev/`; the owned server stops with the launcher.

Read [the quickstart](docs/developers/quickstart.md), then
[build another app](docs/developers/another-app.md). The authoritative literate
construction in the parent repo starts at
`.hickory-source/mechanisms/explicit-web-experiment.md`. To get a standalone
literate project, use `python3 tools/export.py /absolute/new-directory`, then
`hick project materialize project.md --root /absolute/new-directory`. In that
standalone export, open `project.md`; its source lives under `.hickory-source/`.

Run focused checks:

```sh
python3 test_http.py
python3 evaluation/compiler_matrix.py --output /tmp/explicit-compiler.json
dotnet run --project Checks --no-launch-profile
python3 tests/browser.py
```

The browser witness needs Node and Playwright from the parent repository, or
EXPLICIT_PLAYWRIGHT_MODULE pointing to an installed Playwright package. Set
EXPLICIT_BROWSER_CHANNEL=chrome to use an installed Chrome instead of the
Playwright-managed Chromium. The other checks do not need Node or a browser.

See [the measured results](docs/developers/results.md), [the contract](docs/guarantees/basic-app.md) and
[the evaluation plan](docs/developers/evaluation.md). Compile-time witnesses
cover selected API mistakes. They do not prove arbitrary application security or
AI productivity. Reflection, deliberate null suppression, retained capabilities,
raw database access and wrong policies remain possible. This is experimental
software, with a working reference app rather than a production support promise.
</web:copy>

## Quickstart

Document prerequisites before commands and expose one typed environment schema. A new app author can run the demo without guessing a port or editing a source file.

<web:copy id="ew-quickstart">
# Run and debug the reference app

For C# developers evaluating the framework locally.

1. Install just, the .NET 10 SDK and Python 3. The app loads the system SQLite library:
   `/usr/lib/libsqlite3.dylib` on macOS, `libsqlite3.so.0` on Linux, or
   `sqlite3.dll` on Windows. Windows requires installing that native library.
2. From `experiments/explicit-web`, run `just dev` and open its URL.
3. Register an account with a 3–40 character username and 12–128 character password.
4. Create a task, select it, change its title/status, save, and delete it. Sign out
   and back in. Restart the app to check that tasks and sessions persist.
5. Debug `Server/Application.cs:Handle`, then the selected form/API endpoint,
   the security calls and `Tasks`. The host writes only a prepared response.

The launcher stores local data and a ready-file under `.dev/`. It binds a random
loopback port and allows that bound origin. It does not leave a server running
when it exits. It never deletes your database. Set environment variables before
running to override defaults. Direct `dotnet run --project Server` requires all
of the schema below; application code never reads raw environment values.

| Variable | Meaning | Local launcher default |
|---|---|---|
| EXPLICIT_LISTEN_URL | HTTP IP listener; port zero allocates a port | http://127.0.0.1:0 |
| EXPLICIT_ALLOWED_ORIGIN | Additional exact browser origin | https://reader.example |
| EXPLICIT_DATABASE | Absolute SQLite file path | .dev/app.sqlite, resolved absolute |
| EXPLICIT_READY_FILE | Absolute file publishing actual address | .dev/server.address, resolved absolute |
| EXPLICIT_SECURE_COOKIE | Secure cookie flag; required off loopback | false |
| EXPLICIT_MAX_BODY_BYTES | Buffered body limit, 1–16777216 | 65536 |
| EXPLICIT_REQUEST_TIMEOUT_SECONDS | Cooperative deadline, 1–300 | 10 |
| EXPLICIT_SESSION_SECONDS | Server-side expiry, 1–2592000 | 86400 |
| EXPLICIT_LOGIN_ATTEMPTS | Per-peer attempts per minute, 1–1000 | 20 |

`GET /api/session` returns a CSRF token and current username. For JSON mutations,
keep its cookie and send X-CSRF-Token. Forms carry a hidden csrf field. API routes:
GET/POST /api/tasks; GET/PUT/DELETE /api/tasks/{id}. Updates and deletes supply
`version`; updates also supply `title` and optional `done`. An old version returns
409. Creation returns 201 and Location; deletion returns 204.

A production deployment needs TLS termination, Secure cookies, a persistent
writable database volume, backups and a single application instance for the
process-local rate limit. The app does not trust forwarded IP/identity headers.
The sample has no password reset, email verification, MFA, multipart uploads,
WebSockets, background-job scheduler or distributed session/rate-limit service.
Those are outside the current basic-app contract, not implemented features.

Only the configured origin and the actual socket origin are allowed. HTTP parse
errors, disconnects and transport limits may occur before application code runs.
The host’s fallback origin policy is deliberately closed to cross-origin callers;
ordinary application errors pass through the application’s CORS decision.

## Focused automation

`python3 tools/check.py` enforces a 220-line budget for code/test files and
compiles with nullable warnings as errors. `python3 tools/ci.py` runs the focused
compiler, runtime and HTTP witnesses, including the conventional host. Browser
verification is separate: `python3 tests/browser.py` with an installed browser,
or `python3 tools/ci.py --browser` to explicitly install the locked Node/browser
dependencies first. No broad Hickory regression suite is invoked.

The GitHub workflow runs these checks on macOS, Linux and Windows and runs the
browser job on Linux. Its definitions are committed; this local implementation
has not executed remote CI or established all three platform results. Push builds
produce unstable NuGet package artifacts; they do not publish stable packages.
In a standalone Git clone, opt into the tracked hook with
`git config core.hooksPath .githooks`. It runs the same focused gate plus the
browser witness, and needs the same prerequisites. Parent repository hooks are
left unchanged. Monitor future GitHub runs with `gh run list` and `gh run view`.

`just --list` is the project command interface: devenv restores locked projects,
check compiles and checks size/syntax, test runs the focused non-browser contract,
e2e runs the browser witness, ci includes both, and build creates the four
unstable packages without publishing. The dev recipe delegates lifecycle control
to tools/dev.py; invoking that implementation directly is useful for diagnosis.
</web:copy>

## Another app

Show how to use the libraries without importing the demo. The minimal composition is the reusable API; each step explains which check a type carries and which runtime policy the application still owns.

<web:copy id="ew-another-app">
# Build another application

For C# developers reusing the framework rather than modifying the task app.

1. Create a net10.0 executable referencing Core, Data, Security and Hosting.
   Core has no dependency on ASP.NET. Hosting alone references the shared ASP.NET
   runtime. Your app does not need an HTTP context or a middleware container.
2. Construct Database with an absolute file path. Call Migrate with
   IdentitySchema.Migration followed by your own versioned schema. Append future
   migrations; do not change migrations used by an existing database.
3. Construct Sessions, Accounts, Cors and any repositories as concrete values.
   Resolve configuration once at startup and reject invalid values before binding.
4. Write `Task<Cors.Prepared> Handle(Request request)`. Inspect CORS first; return
   its Stop result when present. Load the session where your application needs it.
5. Define routes with Routes.At or Routes.Id. Match returns a checked route value;
   Link accepts that same parameter type. Parse body data with Form or Json, then
   validate domain values through a Result-returning constructor.
6. For protected reads, RequireUser returns Principal. For mutations, Verify the
   submitted CSRF token, then call Principal.Authorize with the VerifiedInput from
   that same session. Pass the resulting WritePermit and validated values to your
   repository. Include `permit.UserId` in every ownership predicate.
7. Render Html.Template with dynamic values in quoted text/attribute slots. A
   composed Html value is already encoded. Never interpolate into scripts, CSS,
   tag names or attribute names. Redirect requires LocalUrl.Parse. SQL Execute
   and Query accept FormattableString; interpolate values rather than formatting
   them to a string first. Keep table/column identifiers literal.
8. Return the response through the origin decision’s Apply method. Pass your
   handler to Server.Run. It implements IHttpApplication directly, bounds input,
   commits buffered bytes, logs exceptions privately, and handles shutdown.

Result.Match requires both a success and failure branch. That makes omissions
visible; it cannot prove that your failure branch is sensible. The compiler also
cannot check an arbitrary SQL ownership predicate. The sample repository’s
capability parameters make the required actor explicit, and its HTTP tests check
foreign-owned IDs. Add those tests for every new aggregate.

There are no global service locators or registration-order conventions. If a new
check is needed, call it in the request function and introduce a checked value
only where it makes an invalid call harder to express. Do not add a state type
for every incidental implementation line.
</web:copy>

## Basic app contract

State observable behavior and source-level restrictions separately. An HTTP suite can witness account isolation and persistence; it cannot witness a C# compiler rejection. Production assurance beyond the named checks remains an explicit limit.

<web:copy id="ew-basic-app-contract">
# Basic web application contract

For authors and evaluators deciding what this version supports.

Given a valid deployment configuration and writable SQLite storage:

- A visitor can register, sign in and sign out through HTML forms. Passwords are
  salted PBKDF2-SHA256 hashes; session tokens are random and only their hashes
  persist. Successful account entry rotates the session. Logout revokes it.
- Sessions expire on the server and survive a process restart while unexpired.
- An authenticated user can create, read, update, complete and delete their own
  tasks through forms and JSON. Another user sees no task contents and receives
  404 for foreign IDs. A stale update/delete receives 409 without changing data.
- Missing/wrong CSRF tokens cannot mutate accounts or tasks. Titles are validated
  before persistence. Invalid JSON and media types receive 400/415; invalid domain
  input receives 422. No successful mutation precedes these refusals.
- HTML interpolation encodes user data. Redirects use validated local paths. SQL
  template values are bound parameters; transaction failures roll back.
- The application explicitly refuses disallowed origins. Allowed origins receive
  CORS headers on application successes and errors; supported preflights return
  204. Credentialed CORS uses an exact origin. This is not authentication.
- CSS assets come from an explicit catalog with ETags. HEAD suppresses response
  bodies. Unknown routes and unsupported methods return 404 and 405 respectively.
- The host bounds bodies, buffers responses, logs unexpected exceptions without
  returning their stack traces, and stops its owned listener on shutdown.

Across ordinary assembly boundaries, supported APIs require Html for an HTML
page, LocalUrl for a redirect, Principal for a protected read, WritePermit for a
reference-repository write, Title for a task title, Guid for a task route link,
and a CORS-prepared result for the transport. Consumers cannot directly construct
Html, WritePermit or Prepared. These are construction constraints, not a claim
that arbitrary C# programs cannot contain web vulnerabilities.

---

Verification status: partially verified. The maintained results report records
macOS runtime, compiler and HTTP witnesses and source hashes. Browser execution,
other operating systems and production assurance remain unverified. The focused witnesses
are tests/test_app.py (live HTTP, persistence, ownership, concurrency and refusal
state), tests/browser.cjs (real-browser forms), Checks/Program.cs (transaction and
capability boundaries), and evaluation/compiler_matrix.py (selected compiler
constraints). Test comments refer to this contract.

Untestable by HTTP alone: source-level compiler constraints, password work factor,
and whether application authors confine raw database/transport capabilities.
Compiler witnesses and source review cover those paths to the stated extent.
C# null suppression/reflection and application bypass remain possible. Capabilities
are not single-use and do not automatically expire if retained by an app author.
Transport-generated errors may predate the app’s CORS policy. The native driver
and deployment behavior require platform witnesses; local macOS results do not
establish Linux/Windows support. Server CORS decisions are exercised over HTTP. Actual browser enforcement is
not established by those header assertions; browser execution status is recorded
separately in the maintained evidence report. No production security
review or general AI-efficiency claim is made.
</web:copy>

## Local launcher

The launcher resolves defaults once into the environment consumed by Settings, builds current source, publishes a dynamically assigned URL and owns the child lifetime. It preserves the database and never opens a fixed port. Request logs remain in a local file so ordinary dev startup stays readable.

<web:copy id="ew-local-launcher">
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
</web:copy>

## Export project

Export the same maintained construction into a new standalone directory. Rewrite only the output-root prefix, not code fragments. Refuse existing destinations. A standalone export materializes with the product’s normal project commands and has no dependency on the parent ownership maps. Reconstruction equality is checked before delivery.

<web:copy id="ew-export-project">
import argparse
from pathlib import Path
import re

HERE = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser(); parser.add_argument("destination", type=Path); args = parser.parse_args()
    source_root = HERE.parents[1] / ".hickory-source/mechanisms/explicit-web"
    if not source_root.is_dir(): raise SystemExit("Export from the parent Hickory checkout, where the maintained .hickory-source chapters exist.")
    destination = args.destination.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    source = destination / ".hickory-source"; source.mkdir()
    index = (source_root.parent / "explicit-web-experiment.md").read_text()
    pages = [source_root / name for name in re.findall(r'file="explicit-web/([^"]+\.md)"', index)]
    for page in pages:
        text = re.sub(r'(' + r'<' + r'web:file\s+path=")experiments/explicit-web/', r'\1', page.read_text())
        seen = set()
        def unique(match):
            path = match.group(1)
            if path in seen: return ""
            seen.add(path); return match.group(0)
        text = re.sub(r'<' + r'web:file path="([^"]+)"[^>]*>.*?<' + r'/web:file>', unique, text, flags=re.S)
        (source / page.name).write_text(text)
    includes = "".join('<book:include file=".hickory-source/' + page.name + '" />\n' for page in pages)
    (destination / "project.md").write_text('<book:doc xmlns:book="http://www.hickorydocs.com/1.0">\n# Build Explicit Web\n\n' + includes + '<book:project exclude=".dev/ target/ Core/bin/ Core/obj/ Data/bin/ Data/obj/ Security/bin/ Security/obj/ Hosting/bin/ Hosting/obj/ Server/bin/ Server/obj/ Baseline/bin/ Baseline/obj/ Checks/bin/ Checks/obj/ tests/node_modules/" />\n</book:doc>\n')
    print("Exported maintained source. Run: hick project materialize project.md --root " + str(destination))

if __name__ == "__main__": main()
</web:copy>

## Local state

Ignore reproducible compiler output and local credentials/data; keep dependency locks and literate source. These paths apply to the standalone project as well as the embedded experiment.

<web:copy id="ew-local-state">
**/bin/
**/obj/
**/__pycache__/
.dev/
target/
.env
.env.*
</web:copy>

## Developer audience

Make the intended reader concrete so future documentation stays task-oriented.

<web:copy id="ew-developer-audience">
# Developer audience

C# developers who can build a .NET app and want to inspect, extend or measure a
framework whose application policy is ordinary control flow. Start with the
quickstart; use another-app.md when reusing libraries and evaluation.md when
interpreting experiments. This audience is not promised a production support SLA.
</web:copy>

## Evaluation protocol

Predeclare what would count as useful evidence and keep three claims separate. A compiler-fixture success is not a productivity result; a host comparison shares the type library intentionally. Future AI trials retain failures and use independent acceptance rather than completion messages.

<web:copy id="ew-evaluation-protocol">
# Evaluate the idea

For researchers deciding whether explicit, typed composition deserves investment.

First run the advertised app contract against both hosts:

```sh
python3 test_http.py
EXPLICIT_TEST_HOST=Baseline python3 test_http.py
```

The second spelling is POSIX shell syntax; on Windows set EXPLICIT_TEST_HOST in
the shell before running the same Python command. Both arms must pass before
interpreting runtime numbers.

## Compiler feedback

```sh
python3 evaluation/compiler_matrix.py --output target/compiler.json
```

The matrix compiles one valid consumer, then 12 selected API mistakes against the
real built libraries, requiring each expected diagnostic. Corresponding untyped
control APIs compile the same categories of mistaken call. Two surviving controls
(null suppression and wrong CORS configuration) must also compile. The selected
cases are an executable catalog of constraints, not a random sample of real web
bugs, a comparison to all ASP.NET analyzers, or a general bug-prevention rate.

## Hosting overhead

```sh
python3 evaluation/host_benchmark.py --rounds 5 --requests 400 --concurrency 4 --output target/hosts.json
```

Both arms use the same typed application, SQLite driver, security, response
adapter, logs, data seed and runtime. One invokes IHttpApplication directly; the
other uses a normal ASP.NET terminal delegate. Release builds, warmup, randomized
arm order, source hashes, per-request latencies, status errors, concurrency and
sample counts are retained. Report medians per round and tail latency beside
throughput. Local load generation competes with the server for resources, and
short runs are noisy. This comparison isolates hosting overhead; it does not
compare type safety to conventional MVC, measure maximum capacity, or establish
AI productivity. A small/no runtime difference is a valid result.

## Matched AI implementation trials

Do not claim a token/time saving until this has actual independent observations.
Freeze both starting repositories, toolchain, model/version, prompts, time budget,
network access and acceptance checks. For each task, randomize arm order and run
fresh sessions without sharing the other arm’s solution. Compare this framework
to a separately implemented conventional ASP.NET app meeting the same behavior,
not the host-only Baseline executable. Both arms need the same documentation
quality or a declared documentation confound.

Use at least these task families: add a protected aggregate; add a validated
field; add a read-only export; diagnose a CORS refusal; add an owner-scoped API;
fix a stale edit; fix an HTML-encoding defect; add a session-dependent page.
Prewrite acceptance and adversarial checks; the implementing agent must not be
the only correctness judge. Repeat across tasks and seeds. A pilot of 8 tasks ×
3 runs × 2 arms gives 24 paired observations, not broad statistical certainty.

Retain task IDs, source hashes, arm/order/seed, model, wall time, compile attempts,
compiler diagnostics, tool calls, actual input/output token counters, test outcomes,
reviewer acceptance and failures/timeouts. Missing token counters stay unknown,
never zero. Compute paired time/token differences only for independently accepted
solutions, and report failure rate across all attempts alongside them. Include
framework/adapter/documentation construction and maintenance cost separately.
A useful initial decision rule is equal or better acceptance/safety with at least
20% lower median accepted-task time or tokens; retain uncertainty and negative
results. That threshold is a proposed investment criterion, not a measured result.

If testing Hickory’s literate interface itself, use its in-app human opt-in
measurement workflow. Do not attribute framework gains to literate presentation
or vice versa without controlling which factor changed. No extra agent trials
have been launched by this implementation.
</web:copy>

## Focused checks

Set a file-size budget at the start and compile the actual application, not a shadow model of the API. This check is fast and does not run the parent repository’s regressions.

<web:copy id="ew-focused-checks">
from pathlib import Path
import py_compile
import subprocess
import sys

HERE = Path(__file__).resolve().parents[1]

def main():
    for path in HERE.rglob("*"):
        if any(part in {"bin", "obj", "node_modules", ".dev", "target", "__pycache__"} for part in path.parts): continue
        if path.suffix in {".cs", ".py", ".cjs"} and path.is_file():
            count = len(path.read_text().splitlines())
            if count > 220: raise SystemExit(f"{path.relative_to(HERE)} has {count} lines; split at a responsibility boundary (limit 220).")
            if path.suffix == ".py": py_compile.compile(str(path), doraise=True)
    return subprocess.call(["dotnet", "build", "Checks/Checks.csproj", "--nologo", "-m:1", "-nr:false"], cwd=HERE)

if __name__ == "__main__": raise SystemExit(main())
</web:copy>

## Ci entry

CI and the opt-in local hook invoke this exact focused command. Browser provisioning is explicit and only runs in the browser job. The workflow does not turn local benchmark noise into a pass/fail performance gate.

<web:copy id="ew-ci-entry">
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve().parents[1]

def run(*args, env=None): subprocess.run(args, cwd=HERE, env=env, check=True)

def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--browser", action="store_true"); args = parser.parse_args()
    if args.browser:
        npm = shutil.which("npm")
        if npm is None: raise SystemExit("Install Node 22+ and npm for the browser witness.")
        run(npm, "ci", "--prefix", "tests", "--ignore-scripts", "--no-audit", "--no-fund")
        run("node", "tests/node_modules/playwright/cli.js", "install", "--with-deps", "chromium")
        env = dict(os.environ, EXPLICIT_PLAYWRIGHT_MODULE=str(HERE / "tests/node_modules/@playwright/test"))
        run(sys.executable, "tests/browser.py", env=env)
        return
    run(sys.executable, "tools/check.py")
    run("dotnet", "run", "--project", "Checks", "--no-build", "--no-launch-profile")
    run(sys.executable, "test_http.py")
    run(sys.executable, "test_http.py", env=dict(os.environ, EXPLICIT_TEST_HOST="Baseline"))
    run(sys.executable, "evaluation/compiler_matrix.py", "--output", "target/compiler.json")

if __name__ == "__main__": main()
</web:copy>

## Opt in hook

The standalone hook delegates to the same gate as CI. Installation is explicit; this change does not replace the parent repository’s existing hooks. Browser checks require the same Node/browser prerequisites as the CI browser job.

<web:copy id="ew-opt-in-hook">
#!/bin/sh
set -eu
cd "$(git rev-parse --show-toplevel)"
python3 tools/ci.py
python3 tests/browser.py
</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/README.md"><web:paste select="#ew-readme"/></web:file>
<web:file path="experiments/explicit-web/docs/developers/quickstart.md"><web:paste select="#ew-quickstart"/></web:file>
<web:file path="experiments/explicit-web/docs/developers/another-app.md"><web:paste select="#ew-another-app"/></web:file>
<web:file path="experiments/explicit-web/docs/guarantees/basic-app.md"><web:paste select="#ew-basic-app-contract"/></web:file>
<web:file path="experiments/explicit-web/tools/dev.py"><web:paste select="#ew-local-launcher"/></web:file>
<web:file path="experiments/explicit-web/tools/export.py"><web:paste select="#ew-export-project"/></web:file>
<web:file path="experiments/explicit-web/.gitignore"><web:paste select="#ew-local-state"/></web:file>
<web:file path="experiments/explicit-web/docs/developers/target-audience.md"><web:paste select="#ew-developer-audience"/></web:file>
<web:file path="experiments/explicit-web/docs/developers/evaluation.md"><web:paste select="#ew-evaluation-protocol"/></web:file>
<web:file path="experiments/explicit-web/tools/check.py"><web:paste select="#ew-focused-checks"/></web:file>
<web:file path="experiments/explicit-web/tools/ci.py"><web:paste select="#ew-ci-entry"/></web:file>
<web:file path="experiments/explicit-web/.githooks/pre-commit" executable="true"><web:paste select="#ew-opt-in-hook"/></web:file>

</web:doc>
