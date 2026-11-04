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
