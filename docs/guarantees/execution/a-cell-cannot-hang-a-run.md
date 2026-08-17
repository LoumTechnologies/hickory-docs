# A Cell Cannot Hang A Run

Given a document executed through the local or sandbox executor, when a cell
runs longer than its time limit — the cell's own `timeout="<seconds>"`
attribute, else `HICKORY_CELL_TIMEOUT` (seconds), else 120 seconds — then the
cell's process is really killed (on Unix, its whole process group, so
background children die with it) and the cell fails with an error naming the
container, the command, the limit that was hit, and both next steps (raise
`timeout=` on the cell, or set `HICKORY_CELL_TIMEOUT`). `timeout="0"`
declares one cell unbounded — allowed, but only explicitly — and a malformed
`timeout=` or `HICKORY_CELL_TIMEOUT` value fails loudly instead of silently
removing the limit.

A cell that blocks on stdin nobody will feed, listens on a socket, or loops
forever used to hang `hick run`, `hick test`, `hick up --run`, the serve run
path, the pre-commit hook, and CI — forever, with no message. A limit that
only abandoned the await would be worse than none: the run would fail while
the process kept holding the workdir and the CPU. So the guarantee is about
the *process*, not the future: on timeout the child is killed and reaped, and
the transcript still records the attempt with an exit event.

## Boundary

The limit covers executing modes only — weave-only paths never execute, so
nothing there can time out. It is enforced where the process is spawned:
`LocalExecutor`, and therefore `SandboxedExecutor`, which wraps it. The
Docker and Canopy executors run cells **unbounded**, deliberately: killing
the `docker exec` client or abandoning the canopy HTTP await would not kill
the remote process, and pretending otherwise would be worse than saying so
(each carries a code comment to that effect). Group-kill is Unix-only; on
Windows the direct child (`cmd.exe`) is killed and grandchildren it spawned
may survive. Agent cells are bounded by `max-turns`, not wall-clock, and
agent scripts have their own in-container `timeout` (see
`docs/guarantees/agent/a-script-cannot-hang-or-flood-a-session.md`).

---

**Verification notes (2026-08-16).** The limit is carried as
`ExecInfo::timeout_secs` (parsed by `parse_timeout` in
`crates/hick-exec/src/dag.rs`, malformed values →
`DagValidationError::InvalidTimeout`), resolved against the run-wide default
by `CellTimeoutDefault::for_cell` in
`crates/hick-literate/src/cell_timeout.rs` (env resolution:
`CellTimeoutDefault::from_lookup`/`from_env`, called in `run_doc_cached` in
`crates/hickory-cli/src/lib.rs` — the entry point shared by `hick run`,
`hick test`, `hick up --run`, and serve), and handed to the executor as
`ExecOptions.timeout` via `Executor::execute_with_options` (the pipeline call
site is in `run_pipeline_live`'s exec loop in
`crates/hick-literate/src/lib.rs`). Enforcement is in
`LocalExecutor::run_command_as` in `crates/hickory-executor/src/lib.rs`:
`process_group(0)` + `kill_on_drop(true)` on spawn, `tokio::time::timeout`
around stdin/stdout/stderr/wait together, and `kill_hard` (SIGKILL to `-pid`,
then `start_kill` + reap) on expiry. Tests, each commented with this file's
path: `crates/hickory-executor/src/lib.rs` (timeout fails fast with next
steps, process-group leaves no survivor, unbounded still finishes, blocked
stdin is covered), `crates/hick-exec/src/dag.rs` (attribute parses, absent
means inherit, malformed rejected loudly),
`crates/hick-literate/src/cell_timeout.rs` (env parsing via injected lookup —
no process-global env mutation), `crates/hick-literate/tests/cell_timeout_tests.rs`
(default bounds a cell, attribute overrides upward, `timeout="0"` unbounded,
`Default` is 120s not unbounded),
`crates/hickory-executor-sandbox/tests/confinement.rs`
(`a_confined_cell_is_killed_at_its_timeout`), and
`crates/hickory-cli/tests/cell_timeout_env.rs` (`HICKORY_CELL_TIMEOUT`
through the shipped binary, malformed value fails before any cell runs,
attribute beats env; env is set only on the spawned process to avoid races).
Caveats: Docker/Canopy unbounded (comments at their `execute` impls);
Windows group-kill limitation is documented at `kill_hard` and untested in
CI; the trait's default `execute_with_options` ignores the timeout, so a new
executor must override it to be covered — the default's doc says so.
