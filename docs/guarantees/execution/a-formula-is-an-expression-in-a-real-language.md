# A Formula Is An Expression In A Real Language

Given a `<hick:table>` that names a `language`, when a cell begins with `=`,
then what follows is evaluated as an expression in that language, with the
cells it refers to bound as ordinary variables; and when it cannot be
evaluated, the cell reports what that language said.

## Why not a formula language

Every spreadsheet ships its own, and every one is a small, badly-specified
programming language that people then learn twice — once for the spreadsheet,
once for the language they actually work in. This product already runs Python,
JavaScript, Rust and shell in the same document. Adding a fifth, worse one for
the tables would be a strange thing to do.

So evaluating a formula is a request to a small program that speaks that
language: the same shape `hick-lsp` uses for intelligence and `hick-dap` uses
for debugging, for the same reason. The thing that knows Python is Python.

## The split, which is the whole design

The **host** owns everything language-independent: what a cell reference is,
which cells a formula depends on, what order they evaluate in, what a cycle is
and how it is reported. A **backend** owns exactly one thing: expression plus
resolved bindings in, value or error out.

That is not an implementation detail. It is what makes "any language"
affordable — a backend is an evaluator rather than a spreadsheet engine, so
adding a language is a small, obviously correct program instead of a second
chance to get topological sorting subtly wrong. It is also what makes the
order of evaluation identical whatever a document mixes, which per-backend
graphs could never promise.

## The rules

1. **Nothing is downloaded, ever.** A backend is a script this binary carries,
   written into `.hick-cache/formula/` on first use and run with an
   interpreter already on the machine. That is why installation is automatic
   here and asks permission for a language server: writing sixty lines into a
   cache has no network path, cannot fail halfway, and runs nobody's setup
   scripts.
2. **The file keeps the formula.** The value is a view, recomputed. What is
   written down is the expression — the thing worth reading in a diff, and the
   thing that still means something on a machine with no interpreter.
3. **Batches are by level, not by cell.** Every cell whose dependencies are
   known goes in one request, so round trips are the graph's *depth* rather
   than its size.
4. **A failure is the language's own message.** A `NameError` naming the
   missing thing is the only part the author can act on; `#VALUE!` throws it
   away. A cycle names every cell in the circle — "circular reference" without
   saying where is the single most useless error a spreadsheet produces.
5. **A broken cell fails alone.** Its dependants see it as *empty* and report
   their own trouble, rather than inheriting a string that happens to be
   somebody else's message.
6. **No language means no formulas.** A table that names none treats `=` as
   text, which is what it was before this existed and what a table of shell
   snippets still needs.
7. **A click or drag is how a reference gets written.** While a formula is open, a
   click on another cell writes that cell's A1 label, and a drag writes its
   rectangular A1 range (for example `B2:B4`), into what is being typed.
   — the feature that makes formulas usable without counting rows, and the
   reason somebody can produce `=B2+B3` having never learned A1 notation. The
   grid decides whether a click MEANS a reference by where the caret is: just
   after a reference this same edit inserted, it REPLACES it, so clicking
   around to find the right cell leaves one reference and not five; somewhere
   an operand could go (after `=`, an operator, an opener, a separator) it
   inserts; anywhere else — after `42`, after a word — the expression is not
   asking for an operand, so the click is a click and the edit ends. That last
   case is what keeps "leave this cell by clicking another one" working. The
   rule is lexical and language-independent for the same reason
   `references_in` is: the expression is in a language the panel does not
   parse. Single cells only, never a dragged range — `A1:B3` would be read by
   the host as two separate references and is a slice of nothing in Python.
8. **The grid says that `=` is the difference.** The `=` convention is
   universal among people who have used a spreadsheet and invisible to
   everybody else, and a table that silently keeps a formula as text is a
   table whose author never learns why. So the formula bar's placeholder says
   it, in the language the table names — `Text, or = for a python expression`
   — and says the other thing when the table names no language at all. Nothing
   else in the grid is in a position to: a cell that has been typed into shows
   what was typed, and a cell that has not is empty.

## Stepping through them

The order is the host's whole contribution, and it is the part a person cannot
see by looking at the grid: a cell showing `#NAME` says it broke, and only the
order says it broke because the cell above it was still empty when it ran. So
the table can be stepped through — cell by cell, in the order they actually
ran, each step naming what it read, what those references were WORTH at that
moment, which batch it went out in, and what it came to.

That is a debugger for the ORDER, not for the expression. Stepping inside an
expression is the language's own job (`hick-dap`, in an exec cell); doing it
here would mean a debugger per backend, which is the exact cost the backend
protocol exists to avoid.

Two rules keep it honest:

- **It is the same evaluation.** `trace_sheet` is the evaluator and
  `evaluate_sheet` is it with the steps dropped. A debugger walking its own
  copy of the order would eventually disagree with the grid about what
  happened, and a debugger that disagrees with the program is worse than none.
- **A circle has no steps at all.** Inventing an order to step through would
  be the debugger's first lie. The cells are marked with the circle, exactly
  as they are without the debugger open.

## Boundary

References are found **lexically** — a run of one or two letters followed by
digits, not attached to a word. The expression is in a language this crate
does not parse, so there is no other way that serves every language at once.
The two-letter cap is the mitigation: `ZZ` is 702 columns, far more than any
table edited by hand, and stopping there makes `myA1` and `sum1` identifiers
rather than references to columns nobody has. A variable genuinely called `A1`
would still be shadowed.

The backends are **not a security boundary** and do not pretend to be one.
`eval` with a restricted globals dict, and `new Function`, are accident
reduction — a formula reaching for `open` is far more often a typo than an
intention. The real boundary is the executor policy every cell in the document
already runs under.

There is a 30-second deadline per batch and a 20,000-cell limit per table.
Past the cell limit the answer says to compute it in an exec cell instead,
because a table that size is a dataset rather than a spreadsheet.

Formulas are not written back into the CSV, are not part of the weave, and do
not affect what `<hick:table path=…>` writes to disk. The file holds what the
author typed.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-formula/src/graph.rs` — `CellRef` (bijective base-26,
  two-letter cap and why), `references_in` (the lexical rule and its word
  boundaries), `evaluation_order` (an explicit stack, so a 5,000-link chain
  does not blow the stack) and `Cycle::message`.
  `crates/hick-formula/src/protocol.rs` — the wire types, LSP framing, and
  `Value::to_cell` (a whole number loses its `.0`).
  `crates/hick-formula/src/evaluate.rs` — `levels` (the batching rule) and
  `evaluate_sheet`, including a failed cell resolving to Empty for its
  dependants.
  `crates/hick-formula/src/session.rs` — the deadline, the crash-is-an-answer
  rule, and stderr captured rather than inherited.
  `crates/hick-formula/src/backend.rs` + `backends/*.py|.mjs` — the embedded
  scripts, `ensure` (the auto-install), and the interpreter check.
  `crates/hick-formula/src/evaluate.rs` — `trace_sheet` and `Step`: the
  evaluator that keeps each cell's turn, with `evaluate_sheet` delegating to
  it so the two can never disagree.
  `crates/hickory-cli/src/serve/formula.rs` — both routes, `sheet_of` and the
  cell cap they share, and `kind_of` (an `empty` cell is not an empty string).
  `crates/hickory-cli/src/main.rs` — `hick formula list` / `install`.
  `apps/web/src/components/TablePanel.tsx` — the debounced evaluation, the
  value-vs-formula display, the degradation when nothing can evaluate, the
  formula bar (which shows a selected cell's own text whatever the grid is
  showing, and whose placeholder is where the `=` rule is stated), and
  `showFormulas` — Ctrl+`, which swaps every formula for its text at once.
  `apps/web/src/lib/cellRef.ts` — `columnLabel`, drawn across the top of the
  grid, so the reference a formula needs is the thing already on screen, and
  `parseCellLabel`, which finds the cell a host answer names.
  `apps/web/src/lib/formulaPoint.ts` — `acceptsReference` and `pointAt`: the
  three-way decision a click makes while a formula is open, as a pure function
  over the text and the caret. Wired in `TablePanel.tsx` as `pointing` /
  `pointTo`, where the cell's mousedown is swallowed so the input keeps focus
  and the edit is not committed before the click can mean anything.
  `apps/web/src/components/FormulaDebugger.tsx` — the transport, what the cell
  read, and the marks it hands back to the grid (`--stepping`, `--read`).
- Test coverage: `crates/hick-formula` (35 unit tests) — A1 round trips, the
  identifier exclusions, the 5,000-link chain, cycles including
  self-reference, the level batching in four shapes, value formatting, and the
  installer writing nothing outside the cache.
  `crates/hick-formula/tests/backends.rs` (17 tests) — both backends actually
  spawned: bindings, aggregation, empty-as-None, their own error messages,
  batch matching by id, a chain evaluating in order, a total row, a broken
  cell failing alone without leaking its message downstream, and the backend
  installing itself on first use. Skipped loudly where the interpreter is
  absent.
  `crates/hick-formula/tests/backends.rs` also covers the trace (6 tests) —
  one step per formula in the order they ran, what a cell read at the moment
  it ran, a blank read as `empty` rather than as a blank string, levels
  matching the batches, a broken cell keeping its own step while its dependant
  reads empty, a circle having no steps, and "stepping and computing are the
  same evaluation" asserted cell by cell.
  `crates/hickory-cli/tests/serve_formula.rs` (8 tests) — the route computing
  a grid, only formula cells returned, the circle naming its cells, an
  unknown language, the cell cap saying what to do instead, and the trace
  route: the order, the levels, the bindings and their kinds, a circle with no
  steps, and the same cell cap.
  `apps/web/src/lib/formulaPoint.test.ts` (9 tests) — every branch of the
  operand rule, the replace-the-last-reference case, and the refusal that
  turns the click back into an ordinary click.
  `apps/web/src/components/FormulaDebugger.test.tsx` (13 tests) — starting on
  the first cell that RAN rather than the first cell in the grid, the batch
  number, what was read, the transport and its ends, `empty` named, the
  language's own message on a broken step, a circle with nothing to step
  through, the degradation when nothing can evaluate, the marks handed to the
  grid and taken away on close, and re-tracing when the table changes.
  `apps/web/src/components/TablePanel.test.tsx` — six formula cases including
  "the file keeps the formula" and "still edits when nothing can evaluate",
  plus the formula bar (it shows the cell's own text, not what it came to; it
  edits the cell it names; both placeholders) and the Ctrl+` swap in both
  directions; six pointing cases (the reference written in, the replacement,
  two references kept when an operator was typed between the clicks, the
  ordinary-click fallthrough, a non-formula cell, and a table naming no
  language); and four for stepping from the grid (offered only where there are
  formulas, never where `=` is text, the cell and its reads marked, the marks
  taken away on close).
- Caveat requiring review: the 30-second deadline is not exercised by a test —
  writing one means a deliberately looping formula and a thirty-second test,
  which is a bad trade for a path whose logic is four lines. The sandbox claim
  is a claim about where the boundary is, not about these backends; nothing
  here asserts that a formula cannot read a file, because it can, exactly as
  an exec cell can.
