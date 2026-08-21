# A Table Is A Dataset And A Paragraph

Given a `<hick:table>` in a document, when the document is woven, then the
reader sees a markdown table; and when the tag carries a `path`, then those
same rows are written there as CSV, byte for byte, for anything that is not
this app to read.

Both halves, from one block. That is the whole feature, and each half fails
without the other:

- Weaving the CSV as a fenced block would show a reader the **delimiters**
  where they expected the data. A table in a document is prose that happens
  to be rectangular; nobody reads commas.
- Storing markdown and converting the other way would leave a file that only
  a markdown parser can read. A dataset earns the name by being loadable — by
  a script, a query, a spreadsheet — and CSV is what all three already speak.

## The rules

1. **The content is CSV, and the CSV is what is written.** No reformatting on
   the way to the file: the bytes between the tags are the bytes on disk. A
   dataset that had been "tidied" is a dataset nobody can diff.
2. **A table with no `path` writes nothing.** It is prose that happens to be
   tabular. Creating a file nobody asked for would litter the folder.
3. **The grid edits the document, not a copy.** There is no second
   representation of the table anywhere — the editor parses the CSV out of
   the document, and every edit writes CSV back into the same span. That is
   what keeps the file a file somebody reviews in a diff.
4. **Quote only what has to be quoted, and keep the line ending.** A one-cell
   edit is a one-line change. A parser that round-trips `a,b` as `"a","b"`, or
   turns a CRLF file into an LF one, makes every review a full re-read.
5. **A ragged row stays ragged.** A row with three fields where the header has
   four is a real thing that happens. The grid pads it for *display*; the file
   keeps what it had until one of those cells is actually edited, because
   writing four fields back would claim the file said something it did not.

## A fence is prose until it is promoted

A ```csv fence gets the grid too, and edits its body in place — but it does
not get the "make it a cell" converter, because running a CSV file as a shell
command is not something anybody means. What it gets instead is a promotion:
turning it into a `<hick:table path="…">` is the step from "a table I pasted
into my notes" to "the dataset this document owns", and it is worth being an
explicit act rather than something that happens to a fence when nobody was
looking. The CSV is carried across untouched, so the promotion is checkable.

## Boundary

The reader is forgiving on purpose, in both languages. A bare quote in the
middle of a field (`5" pipe`) is a literal quote, an unterminated quoted field
ends with the text, and a malformed file still opens. A weave that failed
because one cell was odd would be a weave nobody could rely on, and a table
editor that refuses to open a file is useless exactly when it is most needed.

The delimiter is guessed in the editor and declared in the element. `tab` is
spelled out because a tab character cannot be typed into an XML attribute in
any way a reader would recognise.

Formulas are **not** part of this. A cell holds text; there is no evaluation,
no dependency graph, and no recalculation. That is the formula protocol's job
and it is a separate thing with a separate contract.

Markdown has no quoting for `|`, so pipes and backslashes are escaped in the
woven table. A cell containing a pipe would otherwise silently become two
cells.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-literate/src/csv_table.rs` — the forgiving parser,
  `delimiter_of`, `header_of`, and `to_markdown` with its pipe escaping and
  ragged-row padding.
  `crates/hick-literate/src/weave.rs` — the `"table"` arm.
  `crates/hick-literate/src/lib.rs` — the file-output pass, which now accepts
  a `table` carrying a non-empty `path` alongside `file`.
  `apps/web/src/lib/csv.ts` — the editor's parser and writer: `guessDelimiter`
  (consistency per row, so a comma in prose is not a delimiter), `writeField`
  (quote only what must be), `writeCsv` (line ending and trailing newline
  preserved), the grid operations, and `isTabularFence`.
  `apps/web/src/components/TablePanel.tsx` — the grid; a cell is a span until
  entered, and the keyboard chords are the spreadsheet ones.
  `apps/web/src/components/FenceTable.tsx` — the fence's grid and
  `tableElementFor`, the promotion.
  `apps/web/src/editor/DocumentEditor.tsx` — `replaceBlockContent` (content
  offsets, so the opening tag's attributes survive) and `replaceFenceBody`.
  Card wiring: `cards.ts` (`"table"`), `railActions.ts` (`["source"]`),
  `CardRail.tsx` (`▦`), `rendered.ts` (the slot and its attributes).
  Insert menu, both halves: `insertCatalog.ts` and `INSERT_GROUPS`.
- Test coverage: `crates/hick-literate/src/csv_table.rs` (15 tests) — quoted
  delimiters, doubled quotes, embedded newlines, the bare mid-field quote,
  unterminated quotes, CRLF, ragged rows, the markdown output including pipe
  escaping and the headerless case.
  `crates/hickory-cli/tests/tables.rs` (7 tests) — weaves to markdown and not
  to a fence, writes CSV exactly (asserting no `|` reaches the dataset), does
  both at once, writes nothing without a path, honours a named delimiter,
  keeps a headerless first row as data, leaves the prose alone.
  `apps/web/src/lib/csv.test.ts` (28 tests) — including five byte-for-byte
  round trips and "one cell edited changes one line".
  `apps/web/src/components/TablePanel.test.tsx` (15 tests) — header vs data,
  ragged padding, the write-back and its quoting, no write when nothing
  changed, Enter/Tab/arrow navigation, row and column operations, and the
  read-only case saying where to make the change instead.
  `apps/web/src/components/FenceTable.test.tsx` (6 tests) — the promotion's
  exact output, including the no-path and quoted-path cases.
- Caveat requiring review: the inline grid is rendered through the same
  block-replacement machinery as diagrams, so its height is estimated before
  measurement (140px); a very tall table will reflow once on open. Not
  asserted — jsdom does no layout. The `<hick:table>` block model
  (`render.rs`) has no arm, so the server's `/render` endpoint does not
  describe tables; the editor reads them from the document structure directly,
  which is why nothing depends on that yet.
