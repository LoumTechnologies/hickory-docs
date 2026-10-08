# A fill handle extends formulas

An editable rendered table shows one small plus handle at the bottom-right of
its selected cell or rectangular selection, including a selection made in
reverse. While editing a cell the handle is hidden. Dragging the handle up,
down, left or right previews the extended range; releasing fills that range
as one document edit and selects the result. Escape or losing window focus
cancels without changing data. A source or format edit during the drag invalidates
its frozen seed, so release cannot overwrite that newer edit.

The original selection stays intact. Its pattern repeats into the extension,
using each original cell as the source. In tables with a formula language,
relative A1 references shift by the destination's displacement from that source.
Both ends of a range shift; `$A1` locks the column, `A$1` locks the row, and
`$A$1` locks both. Quoted strings and language comments remain unchanged;
Python floor division is an operator, not a JavaScript comment. A reference
shifted before row 1, before column A or past the evaluator's column ZZ becomes
`#REF!` instead of wrapping to another cell. Existing language evaluation
reports invalid expressions through its usual error display.

Number formats, including money, repeat with the cells and remain metadata
separate from numeric values. A source with General formatting clears an
explicit destination format. Literal values repeat. In a table with no formula
language, an equals sign remains ordinary text. Dragging beyond the bottom or
right edge can add rows or columns within the existing 20,000-cell grid budget.
There is no row before 1 or column before A. A diagonal drag chooses the axis
with the larger distance outside the seed (vertical wins a tie).

The fill uses the existing document persistence path. Values and formats save
together in Markdown and one undo restores the original table and metadata.

---

Last LLM verification:

- Date: 2026-10-08
- Reviewer: Codex
- LLM implementation review: verified for the stated mouse-driven editor behavior.
- Originating requirement: the user requested a small plus icon at the lower
  right of selected cells to extrapolate formulas in all four directions.
- Interpretation and assumptions: extrapolation means spreadsheet-style formula
  copying with relative references and repeating a selected block. Numeric
  sequence inference (e.g. 1, 2 becomes 3, 4) is not part of this formula request.
  Coordinates remain zero-based internally and A1-based in source. Selection
  and preview are ephemeral; only release writes through the existing table
  adapter. A diagonal gesture has one axis rather than filling a whole rectangle.
- Evidence: `TablePanel.tsx` mounts `TableFillHandle` at normalized selection
  bounds, excludes editing/readonly states, and calls `apply(next, false)` once.
  `useTableFill.ts` freezes the seed, previews mouse movement, cancels on Escape
  and blur, checks the revision on release, and locates cells beyond grid edges.
  `tableFill.ts` tiles from immutable source rows and copies format metadata.
  `copyFormulaReferences` in `formulaReferences.ts` respects locks and skips
  strings and language comments. `TableFormatControl.tsx` extracts the existing
  format control to keep the oversized TablePanel within its shrinking ratchet.
- Independent witnesses: C3 containing `=B3*2`, filled down to C5, produces
  `=B5*2`; filled left to B3 it produces `=A3*2`. Filling before column A
  produces `#REF!`. A two-row seed `=A1*2`, `=A2*3` grows to rows 5 and 6 as
  `=A5*2`, `=A6*3`. A Python formula `=A1 // B1 # C1` filled down becomes
  `=A2 // B2 # C1`. Money metadata follows the filled destination without
  adding currency symbols to expressions or values.
- Challenges and falsifiers: reverse selection, multi-cell patterns, mixed locks,
  string contents, floor division, ragged rows, grid growth, stale seeds and
  cancellation are covered. Writes during preview, changing the seed during
  tiling, loss of money formats, or needing multiple undos would disprove this
  contract. Arbitrary JavaScript regular-expression literals and template-string
  interpolation are not parsed as language ASTs; reference rewriting is lexical,
  as in the existing structural-edit helpers. Touch dragging and exhaustive
  cross-browser/platform gesture behavior remain unverified.
- Test coverage: `lib/tableFill.test.ts`, `components/TableFill.test.tsx`,
  `editor/tableFill.test.tsx`, and `e2e/tableFill.spec.ts` name this guarantee.
  Integration checks exercise the real Markdown edit and CodeMirror undo;
  browser checks exercise physical mouse drags rather than invoking fill logic.
- Executed checks: 166 tests passed across seven fill, table-panel, formula-reference
  and number-formatting test files. TypeScript typecheck passed. Five Chromium
  Playwright flows passed against the full editor with `VITE_MOCK=1`: all four
  directions and a reversed multi-cell seed growing beyond the bottom edge.
  Formula evaluation in those browser flows is mocked; reference text and
  formats are independently checked through the formula bar and format selector.
  The initial browser command could not launch because Chrome is not installed;
  rerunning with the installed Chromium channel passed. Source-length and
  change-specific whitespace checks passed. Production web build passed; the
  final reference-parser revision passed 20 focused tests and typecheck.
- Dependencies (SHA-256 at review):
  `apps/web/src/lib/tableFill.ts`:
  `83d3df7e8217cc633912bb29025f9d4230ce3382dd92d7119efe35a025603975`;
  `apps/web/src/lib/formulaReferences.ts`:
  `85679d50d9c25d76a8a13f273f1fb64e821f87bb3c3a0ece102e60c75c729764`;
  `apps/web/src/components/useTableFill.ts`:
  `399cba8bc641555eebf431432b8b78be90beb2bb216c86e6038b65b3c5681e10`;
  `apps/web/src/components/TablePanel.tsx`:
  `53b3e83c9f0f1d0e9a0956960120254f7e6b9907e02a147d2443e053c214e7c8`.
  Persistence evidence also depends on the existing table adapters and
  CodeMirror transaction path. Changes to these dependencies require revalidation.
- Human judgment: not recorded.
