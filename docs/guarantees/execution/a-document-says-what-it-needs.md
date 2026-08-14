# A Document Says What It Needs

Given a `<hick:container>` carrying `<hick:needs bin="…" />` declarations,
when the document runs, then every declared program is looked for **before
any cell executes**, and a document missing one stops immediately — naming
every missing program at once, with the line each was declared on and the
reason its author gave.

Nothing is downloaded, nothing is installed, and no network is touched.
This is a declaration and a check, not a package manager.

```xml
<hick:container name="reporter" image="python:3.12">
  <hick:needs bin="duckdb" for="the queries in section 3" />
  <hick:needs bin="gnuplot" for="the chart" />
</hick:container>
```

## Why before, and why all at once

**Before**, because half a pipeline's side effects followed by
`duckdb: not found` is the worst available ordering: the document has already
changed things and still cannot finish. **All at once**, because three round
trips to learn about three missing programs is how a person decides a tool
hates them.

## The check asks the executor, not the machine

"Installed" and "visible to the cell" are different questions, and only the
second one decides whether the document works:

- A **sandboxed** cell has an empty `$HOME` with only the usual toolchain
  directories bound back, so a program in an unusual place exists on the
  machine and not in the cell.
- Under **Docker** it is the image's contents that matter and the host's not
  at all.

So the probe goes through the executor, and an executor that cannot answer
returns "present" rather than blocking a document over a check it never
performed.

The probe never appears in the transcript. A woven document listing
`command -v duckdb` beside the author's own commands would be a page about
our implementation in the middle of somebody else's work.

## What this deliberately does not do

It does not install anything, resolve versions, or understand package
managers. Resolving tools for people means owning the difference between what
we installed and what their own tooling installs — a permanent cost for a
problem their ecosystem already solves. Saying precisely what is missing is
the part that has to exist here.

---

Last LLM verification:
- Date: 2026-08-14
- Reviewer: Claude (Opus 5)
- Result: verified end to end on Linux
- Evidence:
  - `crates/hick-literate/src/needs.rs` — `needs_of` reads only `needs`
    children of a container (tested against `allow`, which lives in the same
    place); `probe_command` uses `command -v`, a POSIX shell builtin, so the
    check's own dependency cannot be the first thing missing, and quotes the
    program name so author-supplied text cannot become shell syntax;
    `Missing::report` groups by container and names lines and reasons.
  - `crates/hick-literate/src/lib.rs` — the preflight runs immediately after
    `declare_capabilities`, before the first cell.
  - `crates/hickory-executor/src/lib.rs` — `Executor::probe` defaults to
    `true`; `LocalExecutor::probe_command` spawns for exit status only, on a
    path that never touches the transcript (deliberately not
    `run_command_as` with a flag, which would be one refactor away from a
    probe appearing in somebody's document).
  - `crates/hickory-executor-sandbox/src/lib.rs` — probes through the same
    confinement cells get, so the answer is the cell's view.
  - `crates/hickory-cli/tests/needs_preflight.rs` drives the real binary: a
    missing program fails the run, names itself, its line and its reason, and
    **no cell runs** (asserted via a marker the command cannot contain — a
    transcript holds the command as well as its output, so a naive marker
    matches its own fixture); several missing programs are reported together;
    a satisfied document runs; the probe never appears in the woven file; a
    document declaring nothing is unaffected.
  - Observed by hand: a program installed in `~/Desktop/mytools` and on
    `PATH` was reported missing under the default confined executor and found
    under `HICKORY_EXECUTOR=local` — the same document, machine and `PATH`.
    That difference is the reason the probe goes through the executor.
- Caveats — what LLM review could NOT establish:
  - Presence only. There is no version check; a document needing `duckdb`
    ≥ 1.5 is satisfied by any `duckdb`. A `version=` attribute matching
    `--version` output would fit the same shape and does not exist.
  - `bin` only. A document needing a Python package, a system library or an
    environment variable has no way to say so.
  - Only the local and sandbox executors were exercised. Docker and canopy
    inherit the default `probe`, which answers "present" — so under those the
    declaration is documentation and not a check.
  - `hick run` writes its woven output even when a run fails, which predates
    this and is unchanged: a preflight failure leaves an `out.md` whose cells
    are marked `[never run]`, overwriting a previously good one.
- Test coverage: the unit tests in `needs.rs` and the five system tests in
  `needs_preflight.rs`.
