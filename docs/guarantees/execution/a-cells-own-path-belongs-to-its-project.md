# A Cell's Own Path Belongs To Its Project, Not To Your Shell

Given a document run through a local executor (`LocalExecutor`, and the
`SandboxedExecutor` wrapping it), when its cells execute, then the scratch
directory holding their workdirs is named after **the project the document
lives in** — so two runs of one document agree about that path no matter which
directory the person was standing in, and two different projects never share
one.

Only one run may hold a project's derived directory at a time. A second run
that finds a live holder takes a random directory and says so in the log: that
run is not reproducible, which is the honest outcome, but it cannot write into
another run's workdirs.

## Why

The derived name exists so that a cell reproduces. A cell's workdir path
appears in its own output whenever the program it runs prints a path —
`dotnet restore` names the project file, compilers name their inputs, half of
everything names its cwd on error. With a random name that output differed on
every run and `hick test` reported drift forever on a document nobody had
touched. A product whose claim is that a document reproduces its outputs byte
for byte cannot have its own temp directory be the thing that breaks it.

The name was derived from **`current_dir()`**, on the assumption that the
working directory *is* the project for a CLI run. That is true only when the
command is run from inside the project, and `hick run ../other/doc.hick` is an
ordinary thing to type. The assumption was wrong in both directions: two
unrelated documents run from one shell shared a name and fought over it, and
one document run from two different shells got two names, which defeats the
reproducibility the mechanism exists for.

The visible cost was a test suite that failed differently on every run.
Integration tests spawn `hick` without setting a working directory, so they
all inherited the harness's and contended for a single lock. Whichever run
lost fell back to a random directory, and a process that exits releases the
lock while its files are still on disk — so one run read another's inputs.
`cache_inputs` failed 6 times in 6 when run in parallel and passed 6 times in 6
when run serially, reporting `total: 10` for a document that says `[1, 2, 3]`.
That reads as a caching bug and is not one.

## What this is not

It is **not** a claim that concurrent runs of the same project are
reproducible. They are not, and the second one says so rather than pretending.
Nor does it make the path *stable across machines*: it is derived from an
absolute path, so it differs between two checkouts of the same repository. What
it guarantees is stability across runs in one place, which is what drift
comparison needs.

---

Last LLM verification:
- Date: 2026-08-31
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `crates/hickory-executor/src/lib.rs` — `LocalExecutor::new_stable_for`
    and `scratch_root(Option<&Path>)`, which canonicalises the project path so
    `.`, `../proj` and an absolute path name one root, and falls back to the
    working directory only for a caller that has no document in hand.
  - `crates/hickory-executor-sandbox/src/lib.rs` —
    `SandboxedExecutor::new_stable_for`, passing it through unchanged.
  - `crates/hickory-cli/src/lib.rs` — `ExecutorChoice::build_for`, called from
    `run_doc_cached` with `doc_path.parent()`. `build()` remains for the
    callers whose cwd genuinely is the project (`hick up`, the MCP server).
  - Tests: `two_projects_do_not_share_a_scratch_root` and
    `one_project_keeps_its_name_from_anywhere` in `hickory-executor`; the
    second asserts both roots are `Stable` so it cannot pass by both falling
    back to a random directory.
  - `crates/hickory-cli/tests/cache_inputs.rs` — deliberately does NOT set a
    working directory, so it stays a live regression test for the contention.
    Measured 0 failures in 6 parallel runs after the change, against 6 in 6
    before it.
- Caveat: the lock's takeover path (a holder whose process is gone) is
  unchanged and still has no test; it is exercised only by killing a run with
  SIGKILL, which nothing automates.
