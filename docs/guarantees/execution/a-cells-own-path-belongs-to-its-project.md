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
directory and said so. What it did not hold against was **an empty lock file**.

`create_new` makes the lock and the pid is written a moment later, so the two
are not one operation, and `holder_is_alive` parsed `""` as "no pid" and
returned false. A second run reading the lock in that gap concluded the holder
was gone, took over, and ran `remove_dir_all` on the root a microsecond before
its owner used it. It surfaced as `failed to spawn 'sh -c' in container 'lab':
No such file or directory`, which reads as a broken shell and is not one.

So an unwritten lock is now read as **held**, and the wedge that would
otherwise create — a run that died between the two syscalls owning the
directory forever — is settled by age: microseconds old is the gap, half a
minute old is a leftover.

Measured at 5 failures in 25 runs of `needs_preflight`, and 0 in 30 after the
fix. Guarded at both ends now: `project_dir_of` in the CLI never returns an
empty path, and `scratch_root` treats an empty project path as no project at
all rather than hashing it.

The lesson is narrower than "check for empty strings". It is that
`Path::parent()` has a third case between "a directory" and "nothing", and
`unwrap_or` reads as though it does not.

## Leftovers are swept, by lock and never by name

A run that does not exit cleanly never drops its `ScratchRoot`, so its
directory and lock stay in the temp directory forever. Nothing had ever removed
one; 42 were present during a single test run. A local executor now sweeps them
once per process, before taking a lock of its own.

What it may delete is decided by **the lock, never the directory name**, and
that is the whole safety argument. The ephemeral fallback asks `tempfile` for
the same `hickory-local-` prefix, so a directory with our prefix and no lock is
very likely a *live* run's — deleting by name would reintroduce the bug this
guarantee is about, from the other end. So the sweep walks `*.lock` files,
requires the derived shape (`hickory-local-` plus exactly twelve hex
characters, which is what `short_key` produces and what a random tempdir does
not), and asks `holder_is_alive` — the same question the takeover asks, so the
two cannot disagree about who is gone. `hickory-home-*`, the persistent cell
homes, exist to outlive a run and are not touched.

That shape rule now lives in one place, `derived_root_key`, because the
sandbox's persistent-home lookup applies the same test and two copies of a
safety rule is how they stop agreeing.

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
  - The takeover path is now tested, and did not need the SIGKILL its caveat
    assumed: what the code reads is a pid in a file, so a spawned-and-reaped
    child's pid is the same evidence a killed run would leave.
    `a_lock_whose_holder_is_gone_is_taken_over`,
    `a_live_holders_directory_is_not_touched`,
    `a_lock_with_no_pid_yet_is_not_free_to_take` (which fails against the old
    `holder_is_alive` and passes with it) and `an_old_lock_with_no_pid_is_a_leftover`.
  - `sweep_stale_roots`, called once per process from `scratch_root`, with
    `the_sweep_removes_only_roots_whose_run_is_gone` (which plants a dead
    root, a live one, an ephemeral fallback, a cell home and an unrelated
    directory, and asserts only the first goes), `the_sweep_removes_an_orphaned_lock`
    and `the_sweep_leaves_a_lock_that_has_no_pid_yet`. Verified end to end by
    planting three roots with a reaped pid: all three went on the next
    `hick run`, and a lockless one with our prefix stayed.
- Caveat: a stale lock whose pid has since been reused by an unrelated process
  reads as live, so neither the takeover nor the sweep will reclaim it — that
  project stays on the ephemeral fallback until the file is removed by hand.
  The age rule covers the empty lock, not this. Fixing it needs something
  stronger than a pid, such as the holder's start time.
- Caveat: `hick weave` and `hick lineage` never build an executor, so they
  never sweep. That is the right place for the work and not a gap, but it does
  mean a machine that only ever weaves keeps whatever it has.
