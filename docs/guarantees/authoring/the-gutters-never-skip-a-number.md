# The Gutters Never Skip A Number

Given a document open in the app, when it contains any element the editor
annotates — an exec cell, a generated file, a copy/cut fragment, a container,
a conditional, a feature, a session turn, a diagram — then every screen row in
the editor is either one document line or a rendered block standing in for a
contiguous run of lines, and no decoration ever adds a row that no line
accounts for.

Monospace is a promise that position means something. A person reading a
`.hick` file counts lines: to find what a stack trace named, to say where a
ribbon starts, to tell a collaborator where to look. CodeMirror's *block*
widget breaks that promise in the most confusing possible way — it is a real
screen row that no gutter can label, so the numbers step over it, and the
column that exists to be counted stops being countable.

Three rules keep it:

1. **An annotation rides the end of the line it describes.** File path chips,
   fragment badges, `when`/`feature` banners, session-turn chips, the
   environment note, and the empty-document hint are inline widgets anchored
   at `lineAt(block.from).to`. They render after the source they annotate, on
   a row that already has a number, and they never change that row's height.
2. **UI too tall for a line REPLACES lines rather than adding a row.** A
   cell's result and a rendered diagram are block *replacement* decorations
   over the block's own line range — a fold. The rows they occupy are rows
   they took away, so the numbers step over lines that are genuinely not
   being shown, which is what every editor's fold arrow already means. This
   is the one sanctioned way to occupy vertical space, and the distinction
   from a block widget is the whole guarantee: a widget's row exists and
   cannot be numbered; a fold's rows do not exist.
3. **A fold's own gap is padding, never margin.** CodeMirror takes a block
   widget's height from `getBoundingClientRect()`, which excludes margins, so
   a margin on a fold is height the height map never learns about — and the
   gutter, which is laid out from the height map, stops lining up with the
   text even though it skipped nothing. This is the *other* way to break the
   promise, and it is worse than a skipped number because the numbers all
   still look right: they are simply drawn beside the wrong line. Measured on
   `scaffolding.hick`, one exec card's `margin: 0.35rem 0` put every number
   below it 11.2px above its own line — half a row, for the whole rest of the
   document — and broke Home and End inside the file blocks underneath, since
   `moveToLineBoundary` resolves a boundary by asking `posAtCoords` about a
   real y that the height map then mapped to the next line down. So the gap
   lives on a wrapper the widget's `toDOM` returns (`.cm-rendered-frame`,
   `.cm-md-image`), where it is inside what CodeMirror measures.

The action rail down the editor's outer edge carries every verb a card has.
A cell contributes a column — run, source, and replay when an expect block is
standing in for its transcript; a diagram contributes source alone; a prose
fence contributes its converter. The run icon doubles as the cell's status —
never run, running, passed, failed, or not yet known to the server — because
it is the only place that status is visible.

A rendered card contains no controls at all. It shows what the cell is and
what it did, and every click lives in the one column that exists to be
clicked. That is a readability rule rather than a layout one, but it shares
the layout rule's reason: a control floating over a result competes with the
result, exactly as a block widget competes with the gutter.

## Boundary

The rail takes the pointer, unlike the number rail beside it: it is the route
between a block's result and its source, so its icons are real buttons with
accessible names, reachable by keyboard.

A rendered block does show one number beside a tall region — the first line of
the range it replaced. That is the accepted cost of rendering in place, chosen
over a reader/editor mode split; see
[a-literate-file-opens-rendered](a-literate-file-opens-rendered.md).

Two cards on nearby lines cannot both sit level with their line. The later
icon is pushed DOWN, never up, so rail order always matches document order and
an icon is never drawn above the line it belongs to. That means a rail icon is
level with its line except where cards crowd, and the popover it opens is
level with the ICON rather than with the line.

A card's several icons use that same stacker: each wants its card's line, so
the second and third are pushed into a column beneath the first. A card with
more icons than its neighbour therefore crowds the next card sooner, which is
accepted — the alternative is a rail wide enough to lay a card's actions out
sideways, which would take width from the text for the sake of the rare card
that has three.

Ribbons now terminate at the action rail's outer edge rather than the number
rail's, and the brace horn wraps both — otherwise a ribbon would run under the
icon column, which paints a background over it.

This says nothing about soft wrapping. A wrapped line is several screen rows
by design, and the right rail already marks continuation rows rather than
repeating the number (`editor/wrapGutter.ts`).

---

Last LLM verification:
- Date: 2026-08-27 (rule 3; rules 1 and 2 last verified 2026-08-17)
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change; confirmed in
  a browser against a document with fragments, a generated file, an exec cell
  and a container — gutters ran 1–40 unbroken)
- Evidence (rule 3): `apps/web/src/editor/rendered.ts::RenderedWidget.toDOM`
  returns a `.cm-rendered-frame` holding the card; the gap moved from
  `margin` on `.cm-rendered` to `padding` on the frame, and `.cm-md-image`
  (the other block widget, in `apps/web/src/editor/mdLinks.ts`) moved its own
  gap to padding for the same reason. Confirmed in the running app on
  `.dev/project/scaffolding.hick`: the worst gutter-to-line offset over the
  whole viewport went from 11.21px to 0.02px, and Home/End inside the
  `app/app.csproj` block land on the boundaries of their own line again.
- Evidence: `apps/web/src/editor/wysiwyg.ts` — no `block: true` decoration
  remains; `endOfLineAt` is where every annotation widget anchors, and the
  module header states the rule. `CellPanelWidget` and `DiagramWidget` are
  deleted along with `CellRegistry`/`DiagramRegistry`, and `wysiwyg()` no
  longer takes them.
  `apps/web/src/lib/railActions.ts` (which icons a card offers, and whether
  a cell has anything to replay — shared with `CellPanel` so the icon and the
  panel cannot disagree),
  `apps/web/src/editor/cards.ts` (what the rail lists),
  `apps/web/src/editor/CardRail.tsx` (placement against the same height map
  the number rail reads; icons carry cell state),
  `apps/web/src/lib/cardRail.ts` (`stackIcons`, `iconVisible`, `popoverTop`),
  `apps/web/src/editor/DocumentEditor.tsx` (rail + popover, measured before
  paint), `apps/web/src/shell/Ribbons.tsx::paneEdges` (both rails).
- Test coverage: `apps/web/src/styles.test.ts` — "block widget height
  invariants" reads the stylesheet and fails on any vertical margin on
  `.cm-rendered` or `.cm-md-image`, and requires the frame's padding. jsdom
  does no layout, so this is a stylesheet assertion standing in for a
  measurement; the measurement itself was taken by hand in the browser and is
  recorded above.
  `apps/web/src/editor/DocumentEditor.test.tsx` — "puts no
  unnumbered row in the document: every row is a line or a fold" asserts
  every annotation sits inside a `.cm-line` AND that every `.cm-content`
  child is either a `.cm-line` or a `.cm-rendered-frame` wrapping a fold,
  over a document
  carrying one of every converted widget; "renders a cell on open, and the
  rail icon swaps it for the source" covers the toggle in both directions. `apps/web/src/lib/cardRail.test.ts` (12 tests:
  stacking, monotonicity, recovery, visibility band, popover clamping).
  `apps/web/src/editor/cards.test.ts` (what the rail lists, in order).
- By-eye fixture: `just dev-seed` writes `.dev/project/cards.hick` (every
  rail card, inline chip, banner, and both fold kinds in one document — and,
  since 2026-08-21, a **bare** document, so the gutter is read on a file whose
  line 1 is its first heading rather than an XML declaration; see
  `docs/specs/freeform/bare-documents.md`) and
  `.dev/project/sessions/20260820-090000-every-turn-chip.hick` (the turn
  chips, which a `hick:doc` cannot carry because a session is its own root
  element). Open both and read the left gutter top to bottom; the numbers
  must run unbroken. That is what the jsdom assertion below cannot do.
- Caveat requiring review: the row-count assertion runs in jsdom, which does
  no layout — it proves the DOM structure has no extra row, not that nothing
  overflows its line visually. The inline chips are sized to sit on the
  baseline without changing line height, and that was checked by eye in a
  browser rather than by a test. A very long file path in a chip will
  overflow to the right of its line rather than wrap; that is deliberate
  (wrapping would change the row's height) but is not asserted anywhere.
