# Cell formatting keeps numbers numeric

Given a Markdown pipe table, a `<hick:table>`, or a CSV/TSV fence edited as a
Hickory grid, selecting cells offers General, Number, Percent, and money formats
for USD, EUR, GBP and JPY. Formatting changes the display of numeric cells and
computed formula results. Editing a cell and the formula bar show its underlying
value or expression. Formatting never rounds or replaces the stored number.
Text, blanks, unevaluated formulas and errors remain readable as themselves.

Entering or pasting an unambiguous monetary number with `$`, `€`, `£` or `¥`
stores its numeric text and assigns the corresponding money format. This includes
English thousands grouping, negative amounts and parenthesized negative amounts.
Thousands separators in a monetary clipboard cell do not create extra columns;
tab-separated monetary cells remain separate. `$` means USD and `¥` means JPY.
Decimal-comma amounts and malformed groupings remain text rather than being
silently misread. Currency formatting does not convert between currencies.

Python and JavaScript formulas receive the numeric value through the existing
formula evaluator; they need no currency-symbol stripping. Formula results are
formatted only for display, so dependent formulas use the original numeric
result. Arithmetic retains the chosen language's existing numeric semantics.

Formats belong to the document, saved separately from data with A1 addresses:

```md
<!-- hickory-table-formats: B2:USD B3:percent -->
| Item | Amount |
| --- | ---: |
| Cost | 1234.50 |
| Share | 0.125 |
```

For CSV-backed tables the equivalent is `formats="B2:USD B3:percent"` on
`<hick:table>`; a CSV/TSV fence carries the same attribute in its opening info
string. Promotion carries that attribute to the table tag. Named CSV output
contains only the data, never format metadata or display currency symbols.
Ordinary Markdown viewers show the raw numbers and ignore the format comment.
Hickory restores the formatting when the document reopens.

Data and inferred formatting are one undoable edit. Inserting or removing rows
and columns moves the formats with the cells; resizing discards formats outside
the new dimensions. Formats survive ordinary data edits. Raw standalone CSV
files have no document metadata and do not offer this format control.

---

Last LLM verification:

- Date: 2026-10-08
- Reviewer: Codex
- Result: verified for the stated editor and numeric persistence behavior.
- Originating requirement: user requested cell formats, especially money,
  automatic currency recognition on paste, numeric Python/JavaScript references,
  and a durable Markdown representation.
- Assumptions: `$` maps to USD and `¥` to JPY; parsing uses decimal points and
  comma thousands groups to avoid guessing a locale. Formats use explicit cell
  addresses rather than column defaults so mixed columns remain expressible.
- Evidence: `lib/tableFormats.ts` handles parsing, metadata, display and address
  movement; `lib/csv.ts` moves formats during structural edits;
  `components/TablePanel.tsx` passes raw rows to formula evaluation and applies
  display formats afterward. `components/MarkdownTable.tsx`,
  `editor/markdownTables.ts`, `editor/tableEdit.ts`, `editor/rendered.ts`,
  `elements/table/view.tsx` and `DocumentEditor.replaceFenceBody` persist formats
  with their table. `FenceTable.tableElementFor` preserves formats on promotion.
  `hick-blocks` recognizes the attribute; `csv_table::to_markdown_with_formats`
  carries it as a comment when a table is woven, shifting headerless addresses.
- Witnesses and challenges: `$1,234.50` becomes `1234.50`, `($1,234.50)` becomes
  `-1234.50`, and `€12,50` remains text. Displaying `1234.567` as `$1,234.57`
  leaves all original digits available for editing. Real Python and JavaScript
  compute `1234.50 * 2 + 0.25` as `2469.25`.
- Test coverage: `tableFormats.test.ts`, `TableFormatting.test.tsx`,
  `tableClipboard.test.ts`, `tableFormatting.test.tsx` (DocumentEditor),
  `FenceTable.test.tsx`, `e2e/tableFormatting.spec.ts`,
  `hick-formula/tests/backends.rs`, `hick-literate/src/csv_table.rs` tests and
  `hickory-cli/tests/tables.rs::number_formats_stay_in_the_document_and_out_of_the_csv_dataset`.
- Executed checks (2026-10-08): TypeScript typecheck passed; the selected ten
  frontend test files passed 249 tests, followed by 22 focused tests after the
  final format-control change. Both Chromium Playwright flows passed against
  the full editor with `VITE_MOCK=1`: money entry/precision and CSV fence
  format/promotion. `cargo test -p hick-literate csv_table --lib` passed 16
  tests; the hick-blocks attribute check passed; the hick-formula backend test
  passed with actual Python and Node interpreters. The current `.md` CLI
  dataset-export test passed. Source file-length and changed-file whitespace
  checks passed. The historical CLI table suite failed because its seven
  pre-existing cases create retired `.md` documents; forwarding those tests
  to the running engine also returned a permission error. The new CLI test
  invokes this binary directly using the existing engine-worker marker.
- Caveats: arbitrary number patterns and locale selection are not implemented.
  Plain Markdown tables retain their existing lack of a formula language;
  formulas use `<hick:table language="python|javascript">`. Clipboard copies
  contain raw values; number formats are not transported by ordinary copy/paste.
  Cross-browser UI behavior and live OS spreadsheet clipboard access require
  additional verification. Older CLI table tests still use retired `.md`
  documents and fail independently of this change.
- Dependencies (SHA-256 at final review):
  `apps/web/src/lib/tableFormats.ts`:
  `e33a54b20edd95698cb2f5ef6beeef6c5fab0c59e79fb8a49d948d9f87260f15`;
  `apps/web/src/editor/tableEdit.ts`:
  `c78f9b0d1e2d99934d0c886aee720142ab74b1c33fcfb3c5143c8a3974db179c`;
  `apps/web/src/components/TablePanel.tsx`:
  `99efef9ba56929087c2dca5b000445f1fff5ec9194c470080b9c3ce61a044efe`.
  Evidence also depends on the persistence adapters and tests in this change;
  changes to parsing, document offsets, persistence, or formula transport require
  revalidation.
- Human judgment: not recorded.

Subsequent revalidation (2026-10-08): the fill-handle change extracted the format
selector into `components/TableFormatControl.tsx` and added format copying during
fill. The original TablePanel hash above records the earlier verification.
`TableFormatting.test.tsx`, DocumentEditor's `tableFormatting.test.tsx`, and
`TablePanel.test.tsx` passed after extraction. The new guarantee
`a-fill-handle-extends-formulas.md` records current fill behavior and dependencies.
