# A Cell Knows Where Its Document Is

Given a document run by `hick` through the local or the sandboxed executor,
when any of its cells executes, then two environment variables are set in
that cell's process: `HICK_DOC_DIR`, the absolute, canonicalised directory
the document is in, and `HICK_DOC_PATH`, the absolute, canonicalised path of
the document file. The cell's working directory is unchanged — still its own
scratch directory — and a run with no document in hand (`hick up` serving a
folder, the MCP server) sets neither.

## Why

A cell runs in a scratch directory. That is not an accident to be worked
around: it is what makes a run reproducible
(`docs/guarantees/execution/a-cells-own-path-belongs-to-its-project.md`) and
what stops a document from quietly writing into the folder it lives in. But
it left a document with no way at all to name the project it is part of. A
document that wants to run its own repository's tests had to write an
absolute path into a cell — and an absolute path is the one thing a file
people send each other, and agents write, must not contain.

The variables are the smallest thing that closes that without giving the
scratch directory up. The document says where it is; the cell decides what
to do about it:

```xml
<hick:exec container="c" image="alpine">
cd "$HICK_DOC_DIR" && cargo test --workspace
</hick:exec>
```

## Canonicalised, and why that matters

Both values go through `std::fs::canonicalize`, so `d.hick`, `./d.hick` and
`/home/me/proj/d.hick` all produce the same bytes. This is the same
reasoning as the derived scratch-directory name beside it: a value that
differs per invocation appears in a cell's own output, and a document whose
claim is that it reproduces byte for byte cannot have `hick`'s own bookkeeping
be the thing that stops it. `Path::new("d.hick").parent()` is `Some("")`,
not `None` — the shape that has bitten this codebase before — and
canonicalising the DOCUMENT before taking its parent is what keeps that from
becoming an empty `HICK_DOC_DIR`.

## Under the sandbox

Cells run confined by default, and the document's directory is normally
**not readable** from inside that confinement. The variables are still set
and still true: they say where the document is, not that this cell may read
it. That is the honest posture — a variable that disappeared under
confinement would make a document behave differently depending on a flag,
which is worse than one that names a path the cell cannot open. A cell that
must actually reach the project is a cell for `HICKORY_EXECUTOR=local`, and
that is the same decision that trade always was.

Docker and canopy set nothing. A cell there runs on a machine where the
host's paths mean nothing at all, and a variable naming a directory that
does not exist would be worse than its absence.

---

Last LLM verification:
- Date: 2026-09-06
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `LocalExecutor::for_document` and `document_env` in
  `crates/hickory-executor/src/document.rs`; applied to every spawned cell
  in `run_command_as` (`crates/hickory-executor/src/lib.rs`), which is the
  single place both `execute_*` and the sandbox's confined `execute_argv_as`
  go through — so `SandboxedExecutor` inherits the variables rather than
  reimplementing them (it exposes its own `for_document` that forwards, and
  `bwrap` is invoked without `--clearenv`, so the child inherits). Wired in
  `ExecutorChoice::build_for_document`
  (`crates/hickory-cli/src/executor_choice.rs`), called from the run path in
  `crates/hickory-cli/src/lib.rs` and from `hick doc verify`
  (`crates/hickory-cli/src/doc_tools.rs`); the project used for the scratch
  key is still the document's parent, exactly as before.
- Test coverage: `crates/hickory-cli/tests/doc_dir_env.rs`, driving the real
  binary — a cell echoes both variables and its `pwd`, and the woven
  transcript is checked to contain the document's real directory and path
  AND to show a cwd that is not the document's directory; a second test runs
  `hick run report.hick` from inside the folder, the bare-name shape, and
  pins that `HICK_DOC_DIR` is still the absolute directory rather than
  empty.
- Caveat requiring LLM review: the sandbox claim above is verified by
  reading `policy.rs` (no `--clearenv` among the `bwrap` arguments) and by
  the fact that the sandbox spawns through the same `LocalExecutor` path,
  not by a test that runs a confined cell and inspects its environment —
  this machine's default executor is the sandboxed one, but the tests here
  pin `HICKORY_EXECUTOR=local` so they do not depend on what a given machine
  can confine. Docker and canopy are verified by reading
  `build_for_document`'s match arm, which routes them to the unchanged
  `build_for`.
