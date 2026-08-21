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
  `crates/hickory-cli/src/serve/formula.rs` — the route and the cell cap.
  `crates/hickory-cli/src/main.rs` — `hick formula list` / `install`.
  `apps/web/src/components/TablePanel.tsx` — the debounced evaluation, the
  value-vs-formula display, and the degradation when nothing can evaluate.
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
  `crates/hickory-cli/tests/serve_formula.rs` (5 tests) — the route computing
  a grid, only formula cells returned, the circle naming its cells, an
  unknown language, and the cell cap saying what to do instead.
  `apps/web/src/components/TablePanel.test.tsx` — six formula cases including
  "the file keeps the formula" and "still edits when nothing can evaluate".
- Caveat requiring review: the 30-second deadline is not exercised by a test —
  writing one means a deliberately looping formula and a thirty-second test,
  which is a bad trade for a path whose logic is four lines. The sandbox claim
  is a claim about where the boundary is, not about these backends; nothing
  here asserts that a formula cannot read a file, because it can, exactly as
  an exec cell can.
