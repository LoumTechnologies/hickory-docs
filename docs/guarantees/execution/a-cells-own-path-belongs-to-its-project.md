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

## The same bug, through the empty path

Deriving the name from the document's directory instead of `current_dir()`
reintroduced the shared root it was meant to remove, in a way nothing about
the change looked like:

**`Path::new("d.hick").parent()` is `Some("")`, not `None`.** So
`doc_path.parent().unwrap_or(Path::new("."))` — the exact call this guarantee's
evidence names — never fired its fallback for a document named without a
directory, and handed an empty path through. `short_key("")` mixes in no bytes
and returns the FNV basis unchanged, so it is the same twelve characters for
everybody: **`hickory-local-9ce484222325`**, which is just the constant
`0xcbf29ce484222325` masked to 48 bits.

Every `hick run <bare-filename>` on the machine therefore contended for one
scratch root again, in *any* project — the first failure mode above, restored
by the fix for it. The lock mostly held: the losers fell back to a random
directory and said so. What it did not hold against is the takeover path this
guarantee's own caveat says has no test — a run that finds a lock whose holder
looks gone does `remove_dir_all` on the root, and that deletes a *live* run's
workdirs. It surfaced as `failed to spawn 'sh -c' in container 'lab': No such
file or directory`, which reads as a broken shell and is not one.

Measured at 5 failures in 25 runs of `needs_preflight`, and 0 in 30 after the
fix. Guarded at both ends now: `project_dir_of` in the CLI never returns an
empty path, and `scratch_root` treats an empty project path as no project at
all rather than hashing it.

The lesson is narrower than "check for empty strings". It is that
`Path::parent()` has a third case between "a directory" and "nothing", and
`unwrap_or` reads as though it does not.

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
    `run_doc_cached` with `project_dir_of(doc_path)`, which maps both `None`
    and `Some("")` to `.`; `doc_path.parent()` alone yields `Some("")` for a
    bare filename and was the recurrence above. `build()` remains for the
    callers whose cwd genuinely is the project (`hick up`, the MCP server).
  - Tests: `two_projects_do_not_share_a_scratch_root`,
    `one_project_keeps_its_name_from_anywhere` and
    `an_empty_project_path_is_not_everybodys_scratch_root` in
    `hickory-executor` — the second asserts both roots are `Stable` so it
    cannot pass by both falling back to a random directory, and the third
    asserts the derived name is not the basis constant. Plus
    `project_dir_tests::a_document_named_without_a_directory_lives_in_the_current_one`
    in `hickory-cli`.
  - `crates/hickory-cli/tests/cache_inputs.rs` — deliberately does NOT set a
    working directory, so it stays a live regression test for the contention.
    Measured 0 failures in 6 parallel runs after the change, against 6 in 6
    before it.
- Caveat: the lock's takeover path (a holder whose process is gone) is
  unchanged and still has no test; it is exercised only by killing a run with
  SIGKILL, which nothing automates. It is no longer only theoretical — it is
  what turned the empty-path collision above from a logged fallback into a
  deleted workdir, so the missing test is now known to cover a path that has
  bitten once. Stale `hickory-local-*` directories and their locks also
  accumulate in the temp directory when a run does not exit cleanly; nothing
  sweeps them, and a stale lock whose pid has been reused would be read as
  live.
