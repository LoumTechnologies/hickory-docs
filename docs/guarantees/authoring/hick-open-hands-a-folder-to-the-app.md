# `hick open` Hands A Folder To The App And Returns, Or Says Why It Cannot

Given a folder of documents or a single `.hick` file, when `hick open [path]`
runs, then the desktop app is started on that path and the terminal comes
back immediately; and when the app is not on the machine, then the refusal
says the two are **separate downloads** rather than reporting a missing binary
nobody has heard of.

`code .` is the gesture being matched, and matching it means the terminal is
not held. A shell that blocks until the editor closes is `git commit`
behaviour, and that is not what was asked for.

Corollaries that are part of the guarantee:

- **The path handed over is absolute.** The app is launched detached, and one
  started from a dock or Finder inherits `/` as its working directory — so a
  relative path would resolve somewhere nobody meant.
- **A single document is passed through, not narrowed to its folder.** The app
  already handles a file target: it locks the parent directory and opens the
  document. Narrowing here would throw away which document was asked for.
- **A path that does not exist is refused before anything is launched.**
- **Resolution order is explicit variable, then beside `hick`, then the
  platform's conventional locations, then `PATH`.** Preferring `PATH` first
  would launch a different build from the `hick` being run, which is the kind
  of mismatch nobody thinks to check.
- **`HICKORY_DESKTOP` is honoured even when it is wrong**, and the refusal
  names the path it was given. Falling through to a different app than the one
  somebody named is how you debug the wrong binary for an hour.
- **The not-installed message names `hick up` as the pairing, not as a
  consolation.** Someone who installed "Hickory Docs" reasonably assumed that
  was one thing; the message leads with why it is two, and the CLI plus your
  own editor is the arrangement the product is built around.

What this does NOT do: bare `hick .` is not supported, deliberately. Making an
unrecognised first argument mean "open this path" costs clap's
did-you-mean — `hick tes` would try to open a folder called `tes` rather than
suggesting `test` — and `pre-launch.md` argues against carrying spellings that
are pure cost until somebody depends on them.

---

Last LLM verification:
- Date: 2026-08-24
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/open_app.rs` — `find` (the four-step order),
    `App::Exe` / `App::Bundle`, `open` (canonicalize, detached stdio, reaping
    only the macOS `open -a` helper), `not_installed`.
  - `crates/hickory-cli/src/main.rs` — `hick open [PATH]`, defaulting to `.`,
    and the does-not-exist refusal.
  - `apps/desktop/src-tauri/src/server.rs` — `named_dir`, which already took a
    first argument, and `start`, which already branched on `target.is_file()`.
    Nothing had to be taught to the app.
  - Tests: `crates/hickory-cli/src/open_app.rs` unit tests (5);
    `crates/hickory-cli/tests/open_app.rs` (4, driving the real binary against
    a stub app that records its argv).
- Caveat requiring LLM review: the integration tests use a stub rather than
  the real app, because **the desktop app is a separate cargo project**
  (`apps/desktop/src-tauri`) and is not built by this workspace's test run.
  What is verified is the contract — which path, in what form, and that the
  terminal returns — not that the app renders the folder.
- Second caveat: in a development checkout the "beside `hick`" step never
  matches, for the same reason: the two binaries build into different target
  directories. `HICKORY_DESKTOP` is the route there, and the installed layout
  is what the other steps are for.
