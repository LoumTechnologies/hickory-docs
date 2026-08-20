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

Two rules keep it:

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

The action rail down the editor's outer edge carries one icon per cell,
diagram, and convertible fence. An exec icon shows its cell's last result —
never run, running, passed, failed, or not yet known to the server — and
clicking it swaps that block between its result and its source.

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

Ribbons now terminate at the action rail's outer edge rather than the number
rail's, and the brace horn wraps both — otherwise a ribbon would run under the
icon column, which paints a background over it.

This says nothing about soft wrapping. A wrapped line is several screen rows
by design, and the right rail already marks continuation rows rather than
repeating the number (`editor/wrapGutter.ts`).

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change; confirmed in
  a browser against a document with fragments, a generated file, an exec cell
  and a container — gutters ran 1–40 unbroken)
- Evidence: `apps/web/src/editor/wysiwyg.ts` — no `block: true` decoration
  remains; `endOfLineAt` is where every annotation widget anchors, and the
  module header states the rule. `CellPanelWidget` and `DiagramWidget` are
  deleted along with `CellRegistry`/`DiagramRegistry`, and `wysiwyg()` no
  longer takes them.
  `apps/web/src/editor/cards.ts` (what the rail lists),
  `apps/web/src/editor/CardRail.tsx` (placement against the same height map
  the number rail reads; icons carry cell state),
  `apps/web/src/lib/cardRail.ts` (`stackIcons`, `iconVisible`, `popoverTop`),
  `apps/web/src/editor/DocumentEditor.tsx` (rail + popover, measured before
  paint), `apps/web/src/shell/Ribbons.tsx::paneEdges` (both rails).
- Test coverage: `apps/web/src/editor/DocumentEditor.test.tsx` — "puts no
  unnumbered row in the document: every row is a line or a fold" asserts
  every annotation sits inside a `.cm-line` AND that every `.cm-content`
  child is either a `.cm-line` or a `.cm-rendered` fold, over a document
  carrying one of every converted widget; "renders a cell on open, and the
  rail icon swaps it for the source" covers the toggle in both directions. `apps/web/src/lib/cardRail.test.ts` (12 tests:
  stacking, monotonicity, recovery, visibility band, popover clamping).
  `apps/web/src/editor/cards.test.ts` (what the rail lists, in order).
- By-eye fixture: `just dev-seed` writes `.dev/project/cards.hick` (every
  rail card, inline chip, banner, and both fold kinds in one document) and
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
