# Stepping A Table Replays The Order The Host Chose

Given a table whose formulas can be evaluated, when the author steps through
them, then they see every formula cell's turn in the order it actually ran —
what the cell read, what those references were worth at that moment, which
batch it went out in, and what it came to — and those steps are the same
evaluation the grid is showing, not a second one.

## Why the ORDER is the thing being debugged

A formula is one expression in a real language, and the split that makes that
affordable puts everything language-independent on the host: what a cell
reference is, which cells a formula depends on, what order they evaluate in,
what a cycle is (`a-formula-is-an-expression-in-a-real-language.md`).

That order is also the part of a table nobody can see. A cell showing
`#NAME` tells you it broke. Only the order tells you it broke *because the
cell above it was still empty when it ran* — and that is the failure a
spreadsheet is most likely to produce and least likely to explain. So what is
worth stepping through is the host's contribution, made visible.

Stepping **inside** an expression is a different tool and deliberately not
this one. That is the language's own debugger — `hick-dap`, over a real debug
adapter, in an exec cell. Building it here would mean a debugger per backend,
which is the exact cost the backend protocol exists to avoid: a backend is
sixty lines because it is an evaluator, not a spreadsheet engine and not a
debug adapter.

## The rules

1. **One evaluation, two views.** `trace_sheet` is the evaluator;
   `evaluate_sheet` is `trace_sheet` with the steps dropped. A debugger that
   walked its own copy of the graph would eventually disagree with the grid
   about what happened, and a debugger that disagrees with the program is
   worse than none.
2. **A step is a formula cell's turn, and literals have none.** Nothing was
   evaluated in `42`; a step showing `42 → 42` would be furniture that makes
   the real steps harder to count.
3. **A binding is what the reference was worth WHEN THE CELL RAN**, not what
   that cell says now. This is the one fact the grid cannot show, and it is
   the whole reason to look.
4. **An empty cell is named, not drawn as nothing.** `sum` skips a blank and
   not an empty string, so a panel showing both as nothing would hide exactly
   the difference somebody opened it to find.
5. **A circle has no steps at all.** A circle has no order; inventing one to
   step through would be the debugger's first lie. The cells are marked with
   the circle as they already are, and the panel says there is nothing to
   step.
6. **The batch is reported, because the batch is the round trip.** Cells that
   share a level cannot depend on each other — that is what makes them one
   request — so their order among themselves means nothing and the panel says
   which batch rather than implying a sequence.
7. **A trace costs what an evaluation costs, and no more.** The same
   20,000-cell limit, the same automatic backend install, the same 30-second
   deadline per batch. Nothing is downloaded and nothing is sent anywhere.
8. **The marks are the debugger's, and they leave with it.** The stepped cell
   and the cells it read are marked in a colour that is not the selection's:
   what is selected and what is running are two different facts about one
   grid. Closing the panel takes both marks away.

## Boundary

There are no breakpoints and no watches, and a step cannot be *changed* — the
trace is a recording of an evaluation that already finished, not a paused
interpreter. Re-running is what an edit does: changing the table re-traces and
starts again, because a step describing a table that no longer exists is worse
than no step.

Stepping does not write anything into the CSV, does not appear in the weave,
and does not affect what `<hick:table path=…>` writes to disk. Like the values
themselves, it is a view.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-formula/src/evaluate.rs` — `Step`, `Trace`, and
  `trace_sheet`, with `evaluate_sheet` delegating to it (rule 1); the level
  loop carrying `level` into each step (rule 6); `reads` capturing the
  bindings at the moment they were resolved (rule 3); the cycle arm returning
  before any backend is started (rule 5); and the answer-by-cell lookup, so
  the steps are the order things happened in rather than the order a backend
  replied in.
  `crates/hickory-cli/src/serve/formula.rs` — `POST /api/formula/trace`,
  `sheet_of` shared with `evaluate` (rule 7), and `kind_of` (rule 4).
  `docs/specs/freeform/api.md` — the route, its shape, and why the two routes
  are one code path.
  `apps/web/src/api/client.ts` / `types.ts` — `traceFormulas`, `FormulaStep`,
  `FormulaBinding`.
  `apps/web/src/components/FormulaDebugger.tsx` — the transport, the reads
  table, the `empty` label, re-tracing on `revision`, and `onStep(null)` on
  unmount (rule 8).
  `apps/web/src/components/TablePanel.tsx` — the `Step` toggle (offered only
  where a language and formulas both exist), `steppingAt` / `steppingReads`
  via `parseCellLabel`, and the `--stepping` / `--read` marks.
  `apps/web/src/styles.css` — those two marks in the warning and second-accent
  colours rather than the selection's.
- Test coverage: `crates/hick-formula/tests/backends.rs` (6 trace tests) — one
  step per formula in the order they ran, what a cell read at the moment it
  ran, `empty` rather than a blank string, levels matching the batches, a
  broken cell keeping its own step while its dependant reads empty, a circle
  with no steps, and "stepping and computing are the same evaluation" asserted
  cell by cell against `evaluate_sheet`.
  `crates/hickory-cli/tests/serve_formula.rs` (3 trace tests) — the order, the
  levels, the bindings and their kinds through the real route; a circle
  answering no steps; and the cell cap.
  `apps/web/src/components/FormulaDebugger.test.tsx` (13 tests) — the panel in
  every state, including the degradation when no interpreter exists.
  `apps/web/src/components/TablePanel.test.tsx` — the toggle's conditions and
  the grid marks appearing and leaving.
- Caveat requiring review: the 30-second deadline and the automatic install
  are inherited from `evaluate_sheet` and are not re-asserted through the
  trace route; they are the same code. Nothing asserts the panel's appearance
  — that the stepped cell's colour is distinguishable from the selection's was
  checked in a browser, not measured, since jsdom does no layout.
