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

## The grid is a spreadsheet, because that is what people know

Everyone who will edit one of these has spent years in Excel, and every place
this behaves differently is a place they are simply wrong about what is going
to happen. So the grid follows the conventions rather than inventing better
ones: A1 labels across the top and down the side, a name box and a formula bar
above, one click to **select** and a second act to **edit**, arrows to move,
typing to replace, Delete to clear, Escape to abandon.

**Selection is a rectangle, not a cursor.** Dragging across cells sweeps one,
a column letter takes that whole column, a row number takes that whole row,
the corner box takes everything, shift reaches from where you were, and Delete
empties all of it. Anything less is a grid where removing four rows is four
trips to the toolbar, which is how people end up editing the CSV as text
instead. The rectangle keeps an **anchor** — the cell the drag began on —
because that is still the single cell the formula bar edits and the one a
keystroke replaces; a selection that normalised its corners would move the
edit somewhere nobody clicked.

Two things follow from that and are worth stating, because both are places a
plausible implementation is wrong:

- **What was clicked is remembered, not inferred from the shape.** A column
  selection and a swept rectangle that happens to cover a whole column are the
  same rectangle and are not the same thing. Removing every row is refused
  rather than performed: a table with no rows left is not something anybody
  asked for, and it is exactly what "select column B, press − Row" would
  otherwise do.
- **Selecting commits, it never discards.** The selection moves on
  *mousedown*, because that is where a drag begins — and the open cell's input
  unmounts the moment editing ends, so React sends it no blur. Every act that
  moves the selection therefore commits the edit itself. Relying on the blur
  loses a cell's worth of typing the first time somebody starts a drag
  elsewhere while a cell is open.

**Enter commits and drops a row** (shift-Enter goes up, Tab goes right), and
the cell arrived at is *selected* rather than opened: an editor that opened the
next cell would swallow the following keystroke as a replacement of whatever
was already there. At the bottom edge it stays where it is rather than falling
off the table.

**The clipboard is somebody else's spreadsheet.** Ctrl+C over a selection —
with no cell open — puts the rectangle on the clipboard twice: as
tab-separated text, which is what a spreadsheet reads (comma-separated text
pasted into Excel lands in a single column), and as a real `<table>`, which is
what Excel, Word and Sheets read when they want structure and what keeps a
cell containing a newline intact. Ctrl+V reads whichever of those arrived,
preferring the table, and falls back to parsing plain text as delimited data
with the delimiter guessed exactly as an opened file's is — so pasted CSV is
CSV and a single pasted word is the 1 × 1 rectangle it is. Ctrl+X is the copy
and then the clear.

Two decisions inside that:

- **A paste lands at the anchor and is not stretched to fill the selection.**
  A spreadsheet repeats a small block to fill a bigger one, which is a clever
  rule that silently writes cells nobody looked at. What lands is what was on
  the clipboard, and it grows the table rather than dropping the overflow.
- **What was written is what is then selected**, so the first thing on screen
  after a paste is exactly what changed, and one Delete puts it back.

The clipboard belongs to whatever has focus. An open cell and the formula bar
are both text fields, and copying the characters selected inside one is both
what the browser already does and what was meant — so the grid takes the event
only when it did not happen in a field.

**Nine rows, then a viewport.** A table in a document is a paragraph; a
hundred-row dataset that pushes the prose after it off the screen has stopped
being one, and the reader who wanted the fortieth row wanted it next to the
sentence about it. So past nine rows the grid keeps its height and scrolls,
and somebody who does want forty at once drags the bottom edge — a decision
about that table, remembered with the rest of its layout. Nothing is
truncated: every row is there, and the pinned height is a window onto them.

That number is arithmetic rather than a hope: the row height is DECLARED
(handed to the stylesheet as a custom property from the one constant that
states it), so nine rows is nine rows and entering a cell cannot change a
row's height any more than it can change a column's width.

**The size indicator is where the size is set.** Clicking "12 × 5" opens a
dialog to type both numbers, because nobody is going to press "+ Row" eleven
times — they will type the CSV by hand instead, which is the outcome the grid
exists to make unnecessary. Growing fills with empty cells; shrinking trims
from the end and says what it is about to take *while the number is still
being typed*, rather than as a confirm step, which trains people to click
through. It refuses a grid past 20,000 cells with the same sentence the
formula route uses, because that is a dataset rather than a spreadsheet.
Setting a size explicitly is also the one place a ragged row is squared up:
padding for display would claim the file said something it did not, but "make
this table four columns wide" is an instruction.

**Row numbers go down BOTH sides where the editor asks for it.** Beside the
first column is where a spreadsheet puts them and where a hand goes to grab a
row; the lane at the far right is there to line up with the editor's own
line-number rail, which cannot label a table's rows because a rendered table
is one fold with one number. Two jobs, two lanes — dropping either to do the
other was the wrong trade. Either lane selects the row it names; only one of
the two corner boxes carries the accessible name, because two controls called
"Select the whole table" is a screen reader reading the same thing twice.

**A card is a solid object in the text.** The dotted line marking where prose
wraps is absolutely positioned over the editor's content, so it paints above
every in-flow block — including a rendered card, whose opaque background is
not enough to stop it. A measure drawn down a table measures nothing and reads
as a column rule, so the card joins the positioned layer and occludes it.

The select/edit split is load-bearing and not decoration. A cell that opens on
a single click has no state in which a toolbar can act on it — pressing
"− Row" blurs the input, the blur clears the cursor, and the button disables
before the click lands, so the button does nothing at all. Selection that
survives losing focus is what makes a toolbar possible.

Column widths and row heights are **declared**, not measured, for a related
reason: a cell is a span until you enter it and an input while you are in it,
and under automatic table layout those two measure differently — so entering a
cell resized the whole table under the pointer. It is also what lets the panel
say "nine rows" in pixels and be right.

**Every grid line is a drag handle** — the right edge of a column and the
bottom edge of a row, in the header, in the row numbers, and in every cell. A
spreadsheet only puts them in its headers, which is fine while the headers are
on screen and irritating when the line you want to move is the one your
pointer is already beside.

**A double-click on a line fits what is behind it** — the column to the
widest thing in it, the row to the tallest — which is the gesture every
spreadsheet already has on exactly this target, on all four of them: a cell's
two edges, the line between two column letters, and the line between two row
numbers. It cuts both ways: a column dragged too wide comes back, because "the
smallest that still fits" is not "grow if needed".

This is the **one** place the grid measures rather than declares, and it has to
be: "what will still fit" is a question about rendered text in a font this
component cannot know. So it asks the cells — but **not** with `scrollWidth`,
which is the trap here and was wrong once. A cell fills its column and its row,
and a scroll size is the larger of the content and the box, so a cell whose
text is *smaller* than its column reports the column. A fit built on that can
only ever grow, and reads as "double-clicking anything expands it slightly" —
the slack being the only thing that changed.

The measurement is therefore taken with the constraint lifted: `max-content` on
the axis being asked about, the box read back, the inline style put back at
once. Same element, same font, same padding, only the size it was being held to
removed — so the answer is exact by construction rather than by a second copy
of the cell's styling, and no cell is left laid out unlike the ones beside it.
The answer is then written back as a declared number like every other, which is
what keeps the measuring contained: nothing downstream can tell a fitted column
from a dragged one.

The slack differs by axis, which is not fussiness. A width gets the cell's
border plus one pixel, because rounding a fractional measurement up can land on
the text and a column set to precisely its content clips the last letter into
an ellipsis. A height gets the border alone: there is no ellipsis on that axis,
and the extra pixel would nudge every row a pixel taller on every double-click
instead of leaving a row that already fits exactly where it was.

**A fit does not change what is selected.** Resizing is not a way of choosing
something. That takes a little arranging, because a double-click is two clicks
and the first of them cannot know the second is coming — its press has already
selected by the time "fit" turns out to be the answer. Delaying every press
behind a double-click timer would make an ordinary press on a line feel broken,
so instead the press *remembers* and the fit puts it back. The second press
does not select at all: `detail` is the click count, so a press of 2 is half a
double-click rather than a choice. Where no press selected anything — a
read-only table, whose cell handles have no tap — the fit leaves the selection
exactly as it is, because restoring something nobody took is the same bug the
other way round.

An empty column fits to the minimum, which is the honest answer to how much
room nothing needs, and one enormous cell is capped rather than allowed to make
a column nobody can scroll past.

That is affordable because **a press that never moves is a click**. The strip
is five pixels along the edge of a cell somebody also wants to select, so
below a three-pixel threshold it hands the press back to what is underneath —
the cell, the row number, the column letter. Without that, the bottom five
pixels of every row would be unselectable, which is a worse bug than the one
the handles fix.

**A cell fills its row.** Not a detail of appearance: the span used to be as
tall as its one line of text, so the space added by dragging a row taller
belonged to the `td` and to no cell at all. A click there hit nothing — the
selection did not move, focus left the grid, and the editor's ruler dropped
back from naming the table's columns to measuring prose, which is what the
symptom looks like from the outside. The row's height is declared on the cell,
so the percentage always has something to resolve against; `styles.test.ts`
asserts both halves, because jsdom lays nothing out and nothing else can.

**How big the table is, is not in the file.** Column widths, row heights and
the grid's height are presentation; the CSV is a dataset a script reads, and it has no
opinion about how much room it should take on somebody's screen. So a size is
remembered in the workspace's own state, keyed by the table's `path` where it
has one (`tableKey`), which means two people can want different amounts of room
for the same table and neither shows up in the other's `git status`.

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
and it is a separate thing with a separate contract — including the one place
that tells an author a formula begins with `=`, what a click on another cell
means while one is open, and what stepping through them shows.

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
  `apps/web/src/components/TablePanel.tsx` — `Resizer`, one handle for both
  axes, with the drag threshold that hands a press back to the cell under it
  and the double-click that fits; `fitColumn` / `fitRow`, the only measuring
  in the grid, over the cells that carry `data-row` and `data-column` for the
  purpose, through `intrinsic` — which lifts the constraint for the length of
  the measurement rather than reading a scroll size that can never be smaller
  than the box;
  `rowHeight` written onto the `<tr>` as a custom property, so a row cannot
  end up two heights at once, and summed rather than multiplied when the
  nine-row height is worked out. The grid; selection is separate
  from editing (which is what leaves the toolbar something to act on), the A1
  furniture is drawn from `columnLabel`, the keyboard chords are the
  spreadsheet ones, and `ColumnResizer`/`HeightResizer` report a `TableLayout`
  rather than touching the CSV. `stopEditing` is what every selecting act goes
  through, which is what stops a drag begun elsewhere from discarding an open
  edit; `takeRow`/`takeColumn`/`selectAll` are the header, gutter and corner.
  `apps/web/src/lib/tableSelection.ts` — the rectangle: anchor and focus kept
  apart, `kind` remembering what was clicked, `extendTo` refusing to select
  half a row or half a column, and `selectionLabel` (`B:B` rather than
  `B1:B7`, which would stop being true when a row is added).
  `apps/web/src/lib/csv.ts` — `clearCells` (which will not pad a ragged row to
  empty a cell that was never in the file), `removeRows` and `removeColumns`
  (bottom-up, so earlier removals do not shift later indices), `pasteBlock`
  (grows, and leaves untouched rows as ragged as they were), `resizeTable`
  (the one place a ragged row is squared up, and why), and `filledSize` (what
  a shrink would really take).
  `apps/web/src/lib/tableClipboard.ts` — `toTabSeparated` / `toHtmlTable`
  (both flavours, and the quoting rule Excel writes by), `parseHtmlTable`
  (through the wrapper Excel puts around it; `colspan` padded, `rowspan`
  declared out), and `parseClipboardTable` (the table preferred, the trailing
  newline not read as a row that would wipe cells).
  `apps/web/src/components/TableSizeDialog.tsx` — the two fields, the live
  "removes 3 rows and what is in them", and the 20,000-cell refusal that says
  what to do instead.
  `apps/web/src/styles.css` — the cell filling its row (and the input with
  it), `user-select: none` on an editable cell (so a
  sweep does not also drag a text selection through the prose), the declared
  `--table-row-height` that makes nine rows nine rows, the two gutter lanes
  kept apart (`--lane`), and `.cm-rendered` joining the positioned layer so
  the prose measure is not drawn down a card.
  `apps/web/src/lib/uiState.ts` — `tableKey`, `readTableLayout` (total, like
  the rest of that file, and clamping a row height to a paragraph's worth),
  and the `tables` record; threaded through
  `useWorkspaceUi`, `WorkspaceView`, `workspaceTabs` and `DocumentEditor`.
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
  `apps/web/src/components/TablePanel.test.tsx` (117 tests) — header vs data,
  ragged padding, the write-back and its quoting, no write when nothing
  changed, Enter/Tab/arrow navigation, row and column operations, and the
  read-only case saying where to make the change instead; plus the A1
  furniture, selecting from a column letter or a row number, "removes a row
  that was being EDITED, not merely selected" (the bug the split exists to
  stop), and a remembered size that never reaches the CSV. The rectangle:
  sweeping one with a drag, the anchor staying put when the drag goes
  backwards, the sweep ending on a mouseup anywhere, shift-click and
  shift-arrow, whole columns and whole rows from the furniture, Ctrl+A and the
  corner, Delete emptying all of it, "− 2 Rows" removing both, and the refusal
  to remove every row. Enter: commits, moves down, and leaves the cell
  selected rather than open. And "an edit that is interrupted by a click
  somewhere else is committed, not thrown away", in two shapes.
  Also: the row numbers down both sides and the single accessible name for
  the two corners; the nine-row viewport (a short table left alone, a long one
  pinned to 238px with all forty rows still present, a remembered height
  winning, and a drag starting from the pinned height rather than from zero);
  the size dialog in six shapes including the shrink warning, the silence when
  what it would drop is empty, the 20,000-cell refusal and Escape changing
  nothing; and the clipboard in twelve — both flavours written, a whole column
  copied, the input keeping the clipboard while a cell is open, cut, paste of
  TSV / HTML / CSV / one word, growth, the selection landing on what was
  written, an unusable clipboard ignored, a read-only table that copies
  but will not take a paste, and both text fields keeping their own clipboard.
  Double-clicking a grid line (14 tests) — the column fitted to its widest
  cell and the row to its tallest, from a cell's edge and from the furniture,
  a too-wide column and a too-tall row shrinking rather than only growing, a row
  that already fits staying exactly where it was, the cell left laid out as it
  was found, an empty column falling to the minimum, one enormous cell capped,
  the cell behind the line not opening for editing, and the selection surviving
  a fit in four shapes (a cell, a range, nothing selected at all, and an
  ordinary one-click press still selecting). Those go through the sequence a
  browser really sends — two presses, the second carrying `detail: 2`, then
  the dblclick — because `fireEvent.doubleClick` alone sends none of it and
  the presses are half the behaviour. jsdom reports zero for
  everything, so the test defines both numbers a browser would give —
  including `scrollWidth` being the LARGER of content and box, which is what
  makes the shrink cases fail if anything goes back to reading it.
  Dragging a grid line (9 tests) — the column moved from a line inside the
  table rather than only from the header, the row moved from its bottom edge
  and from its number, neither draggable away to nothing, a press that never
  moved selecting the cell instead, a two-pixel wobble counting as that press,
  and the nine visible rows counted at the heights they actually are.
  `apps/web/src/styles.test.ts` — the cell filling its row and the declared
  row height that percentage needs, as layout invariants with a real failure
  behind them.
  `apps/web/src/lib/tableSelection.test.ts` (15 tests),
  `apps/web/src/lib/tableClipboard.test.ts` (14 tests) and
  `apps/web/src/lib/csv.test.ts` (44 tests) — the arithmetic underneath,
  including the ragged-row refusal, the resize cases and the paste cases.
  `apps/web/src/lib/uiState.test.ts` (4 cases) — the key, the fallback key, a
  stored size coming back with nonsense dropped and an absurd height clamped.
  `apps/web/src/components/FenceTable.test.tsx` (6 tests) — the promotion's
  exact output, including the no-path and quoted-path cases.
- Caveat requiring review: `rowspan` in pasted HTML is not reconstructed —
  cells below a spanned one shift left, exactly as they do when the same table
  is pasted into a spreadsheet as values. Said rather than half-implemented.
  The clipboard is driven through synthetic `copy`/`cut`/`paste` events with a
  stand-in `clipboardData`, so what a real Excel copy puts on a real clipboard
  is covered only by the fixture in `tableClipboard.test.ts` being an actual
  Excel fragment; the round trip through a real OS clipboard is not asserted
  anywhere and cannot be in jsdom.
  The nine-row height is asserted in pixels against declared constants, which
  is the same arithmetic the component does — that the tenth row really is
  below the fold was checked in a browser, not measured.
  The inline grid is rendered through the same
  block-replacement machinery as diagrams, so its height is estimated before
  measurement (190px, raised with the formula bar and the letter row); a very tall table will reflow once on open. Not
  asserted — jsdom does no layout. The `<hick:table>` block model
  (`render.rs`) has no arm, so the server's `/render` endpoint does not
  describe tables; the editor reads them from the document structure directly,
  which is why nothing depends on that yet. No drag handle is exercised
  end to end — jsdom lays nothing out and has no `PointerEvent`, so every
  resizer uses mouse events and the tests state the geometry rather than
  measuring it; that the column widths actually stop moving when a cell is
  entered, and that five pixels is a comfortable target for a grid line, were
  checked in a browser rather than asserted.
  The auto-fit measurement was verified in Chrome against these same rules
  rather than reasoned about, after `scrollWidth` got it wrong: a cell holding
  `ab` in a 104px column reports a `scrollWidth` of 103 and an intrinsic width
  of 28, text reporting 229 has an intrinsic 230 (which is what the anti-clip
  pixel is for), one line of this font measures 23 against the 24px default
  row, and after fitting every column and row no cell reported itself clipped
  on either axis. What is NOT asserted anywhere is that a browser still
  answers that way — a layout-engine change would be caught by eye, not by
  this suite. The click sequence behind the `detail` guard was checked the
  same way rather than assumed: a real double-click sends down/up/click at
  `detail: 1`, then down/up/click at `detail: 2`, then `dblclick` — and the
  mouseup carrying it is the one on the WINDOW, which is where the handle
  listens.
