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
