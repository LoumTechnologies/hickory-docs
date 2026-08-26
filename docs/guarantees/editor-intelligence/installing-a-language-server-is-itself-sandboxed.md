# Installing A Language Server Is Itself Sandboxed

Given `hick lsp install <language>`, when the server is fetched, then the
installer runs **inside the same sandbox a document's cells run in**: it may
write only `.hick-cache/servers/`, its `HOME` is redirected inside that
directory, and it is the one and only place in this product that is granted
network access. Where no sandbox is available, the install is **refused**.

The reason is what an install actually is. `npm install` executes arbitrary
`postinstall` scripts; `uv pip install` executes arbitrary build backends. So
"install a language server for the user" means *download code from the
internet and run it on their machine* — precisely what the sandbox exists to
confine. Sandboxing the document you were sent while running the installer
unconfined would be a strange trade.

## It is never automatic

Discovery runs first: a server already on the machine is always preferred, and
`hick init` reports what it found. Installing happens only when a person types
`hick lsp install`. Auto-installing on first open would reach the network and
run someone else's code as a side effect of opening a file, which this product
promises not to do.

## The installer profile

One difference from the cell profile, and it is forced rather than chosen: a
cell gets an **empty** `$HOME`, but an installer cannot, because the tool it
runs often lives in one — `uv` installs to `~/.local/bin`. Hiding the home
hides the installer, and the command fails with `uv: not found` on a machine
where `uv` is plainly installed. So for an install the real home stays visible
read-only, and `HOME` is pointed at a writable directory inside the prefix, so
the tool's caches land there instead of nowhere.

## What gets installed, and why that package

Python installs **basedpyright**, not pyright. Pyright implements no semantic
tokens, no inlay hints and no folding ranges — those are Pylance's, and
Pylance is licensed for Microsoft's own editors only. basedpyright is the
open-source fork that reimplements them, so it is what makes Python in a
document actually coloured.

## The C# entry, and what it took

`hick lsp install csharp` fetches **csharp-ls** (MIT), a Roslyn server
packaged as a `dotnet tool`. Three environment variables are what make it
installable under this sandbox at all, and none is decoration:

- `DOTNET_CLI_HOME={prefix}/dotnet` and `NUGET_PACKAGES={prefix}/dotnet/nuget`
  — `dotnet tool install` writes to `$HOME/.dotnet` and `$HOME/.nuget/packages`
  by default, and the installer sandbox makes only the prefix writable. Without
  the redirect the install fails on a permission error that reads like our bug.
- `DOTNET_CLI_TELEMETRY_OPTOUT=1` — this product says nothing to anyone, and a
  tool it spawns on the user's behalf must not be the exception. The .NET CLI
  phones home on first run unless told not to.

**Not OmniSharp**, though discovery has always looked for it and still prefers
it when it is on the machine: OmniSharp ships as a per-platform release
archive, so installing it means choosing a URL and a checksum, which is a
different installer shape than any entry here has. Both are MIT.

**Not Microsoft's Roslyn language server** (the one inside the C# extension):
its licence permits use only with Microsoft's editors, which is not something
to install on somebody's behalf.

Discovery had to learn one new thing for this: `dotnet tool install
--tool-path` puts the executable **straight into the directory it is given**,
with no `bin/` beneath it — unlike every other layout in the search list.

---

Last LLM verification:
- Date: 2026-08-14
- Reviewer: Claude (Opus 5)
- Result: verified end to end on Linux
- Evidence:
  - `crates/hickory-cli/src/lsp_install.rs` — `install` refuses an unknown
    language (naming the ones that work), refuses when the installer tool is
    absent (naming the tool), and refuses when `Sandbox::detect()` is `None`.
    `policy::wrap(…, allow_network = true, Profile::Installer)` is the only
    network grant in the product. Tests assert every installer command writes
    under `{prefix}` and none installs globally.
  - `crates/hickory-executor-sandbox/src/policy.rs` — `Profile::{Cell,
    Installer}`; bubblewrap redirects `HOME` with `--setenv`, Seatbelt (which
    cannot set an environment variable) prepends the assignment to the
    command, and Windows needs neither because an AppContainer's AppData is
    already redirected.
  - `crates/hick-lsp/src/discovery.rs` — `.hick-cache/servers/python/bin` and
    `…/node/node_modules/.bin` are searched **first**, ahead of the rest of
    the project. Test:
    `a_server_this_project_installed_is_found_without_configuration`.
  - **Run end to end on this machine.** In an empty directory,
    `hick lsp install python` fetched basedpyright 1.39.10 through
    bubblewrap in 19s. Afterwards the only thing in that directory was
    `.hick-cache/`; uv's cache had landed in the redirected `HOME` inside the
    prefix. Opening a document with an `app.py` block then discovered the
    server with `origin="project"` and **answered `semanticTokens/full`** —
    `[5, 4, 4, 12, 1, …]`, a `function` with the `declaration` modifier at
    line 5 character 4, which is `load` in `def load(path):` — and
    `inlayHint` with `-> str` at line 5 character 14. Neither had worked with
    stock pyright.
- Caveats — what LLM review could NOT establish:
  - Only the `uv`/Python installer was actually run. The two npm installers
    are the same code path with a different command, but unobserved.
  - Because `HOME` is redirected, a private registry configured in `~/.npmrc`
    or `~/.config/uv` is NOT picked up. A user behind one should install the
    server themselves; discovery prefers it anyway.
  - The install is confined but not *verified*: nothing checks a signature or
    a hash. What it protects is the rest of your machine, not the integrity
    of the package.
  - Never run on macOS or Windows.
- Evidence for the C# entry: the `csharp` `Installer` in
  `crates/hickory-cli/src/lsp_install.rs`; the `.hick-cache/servers/dotnet`
  entry in `project_dirs` and the `csharp-ls` candidate in `C_CSHARP`
  (`crates/hick-lsp/src/discovery.rs`). The "nothing to install" message in
  `crates/hickory-cli/src/main.rs` now derives its tool list from the
  catalogue (`installer_tools`) rather than naming `uv, npm` in a literal that
  would have gone stale the moment `dotnet` was added.
- Test coverage: `the_csharp_server_this_project_installed_is_found_without_configuration`
  (`crates/hick-lsp/src/discovery.rs`) — the flat `--tool-path` layout is found
  and invoked with no arguments. Verified by hand on 2026-08-26 with .NET
  10.0.111: `hick lsp install csharp` completed confined, put `csharp-ls`
  0.27.0 in `.hick-cache/servers/dotnet/`, and left no `csharp-ls` in the
  user's `~/.nuget/packages`.
  the unit tests above plus the discovery test. The end-to-end
  install is observed, not automated — it needs the network, which the test
  suite must never require.
