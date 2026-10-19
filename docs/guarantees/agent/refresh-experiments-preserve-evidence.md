# Local refresh experiments preserve historical request evidence

Given an explicitly queued update, the Python refresh policy supports appended
replacement passages, exact prior-read projection with a change notice, and
notification followed by a bounded reread. Scheduling remains a separate input.
Projection assembles a copy; original request bundles and caller history remain
unchanged. It replaces only exact role-matching plain text messages and leaves
tool-call envelopes intact. Source updates remain user notices; role/scope is
reported rather than promoted. Notices invalidate earlier dependent plans and
summaries without claiming to have corrected their reasoning.

The offline harness pairs all encodings and manual/automatic schedules against
the same eight source-change scenarios, in fresh temporary directories, with at
least two repetitions. It records validated exact manifests, policy versions,
local assembly boundaries and measurement coverage. Missing provider metrics
remain null. No provider requests, paid inference or telemetry occur. A separate
observation comparison hook fixes refresh policy while changing representation.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: #53, #47 and live-agent-context.md.
- Implementation: `refresh.RefreshPolicy`, `refresh_experiment.replay`, `run`,
  `compare_observations`; MemoryContext validates request bundles.
- Assumptions: the deterministic witness measures exposure and invalidation;
  neither model reasoning nor actual user ease is inferred. Temporary files
  emulate saved source heads; unsaved buffers remain separate.
- Witnesses: test_refresh_experiment.py (three focused tests); existing six
  test_context_runtime.py safety tests; 144 local paired replay records generated.
- Challenges: deletion, changed requirements, changed instruction applicability,
  overlapping-read bytes, outside-read change, rename, unsaved and concurrent edits;
  exact role preservation, tool envelope preservation and receipt immutability.
- Coverage boundary: this baseline does not drive a real model, file-watcher or
  confinement adapter. Stale assumptions/edit attempts and real human effort are
  unavailable. The representation hook requires a task driver to measure real
  logs/debug/profile usability. No winner is established. Live providers need
  separately configured credentials, driver and explicit budget.
- Dependencies: refresh.py, refresh_experiment.py, context.py and their focused
  tests in this commit; changes invalidate this review.
- Executed checks: three experiment tests and six context-runtime tests passed;
  CLI generated 144 records. Broad regression checks not run.
- Human judgment: not recorded.
