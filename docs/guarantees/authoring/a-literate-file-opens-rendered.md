# A Literate File Opens Rendered, And Its Source Is One Click Away

Given a `.hick` document, when it is opened in the app, then every exec cell
and every diagram shows its RESULT — the diagram drawn, the cell's commands
and last output — rather than its tags; and when the block's rail icon (or its
`source` control) is clicked, then that block shows its raw source, editable
as ordinary text, and clicking again returns it to the result.

A literate file is meant to be read. Opening one and finding
`<hick:diagram renderer="mermaid">` above nine lines of graph syntax is
reading the machinery instead of the document. But this is also an editor over
real source, and a reader-friendly view that cost you the ability to edit
would trade one failure for another. Both halves have to be true at once,
which is why this is a per-block toggle over one view rather than a reader
mode and an editor mode: there is only ever one document on screen, and any
part of it can be turned back into text where you stand.

Three properties hold it up:

1. **Rendering is a fold, not an overlay.** A rendered block is a block
   *replacement* decoration over the block's own lines. It stands in for them
   the way a collapsed region does, so it may be as tall as it likes without
   adding a row the gutters cannot account for — see
   [the-gutters-never-skip-a-number](the-gutters-never-skip-a-number.md). The
   text is folded, never altered: the document on disk is byte-identical
   whether a block is showing its result or its source.
2. **A rendered cell shows what the source would have shown, plus what it
   cannot.** The commands appear as a shell would echo them (`$ …`), because
   the source that carried them is folded away and a result with no visible
   command is a claim with no subject. Around them sit the container, the
   status, the transcript or expect-diff, and Run.
3. **Render-on-open happens once, on open.** A block written after the
   document was opened stays as source. Turning what you are typing into a
   picture under the caret is the opposite of helpful, and "I am reading this
   file" and "I am writing this block" are different moments.

Which blocks are rendered is remembered as document POSITIONS, mapped through
every change. Keying on "the third exec cell" would move the state onto the
wrong block the moment one was inserted above it; a mapped position follows
the block it was taken from and disappears with it.

## Boundary

A rendered block shows one line number beside a region that may be many rows
tall — the first line of the range it replaced. That is the accepted cost of
rendering in place. The alternative considered and rejected was a Read/Edit
mode pair, where Read hides the gutters entirely and the numbering question
dissolves; it was rejected because it splits one document into two views.

The rail icon is the way back from a rendered block, so it is never a no-op
and its label says which direction it goes. Because a cell's Run button now
lives on the rendered block itself, there is no popover for cells or diagrams
any more — the fence converter is the only thing the rail still opens as a
popover.

Rendering does not extend to the wrapper machinery: `hick:doc`, `hick:file`,
and copy/cut tags stay visible as dimmed chrome with inline chips. Hiding
those would make the view a preview of the woven output rather than a view of
the document, and the document is the thing being edited.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: implemented, unit-verified; NOT confirmed in a browser this round
  (see caveat)
- Evidence: `apps/web/src/editor/rendered.ts` — `renderedField` (positions
  mapped with `MapMode.TrackDel`), the `renderBlock` / `showBlockSource` /
  `setRenderedBlocks` effects, `buildRendered` (block replacement over whole
  lines only), `commandOf` (content minus a nested expect, via
  `codeRangesOf`). `apps/web/src/components/CellPanel.tsx` (`command` and
  `onShowSource`, `CommandLines`). `apps/web/src/editor/DocumentEditor.tsx`
  (`seedRendered` — once per open, dispatched out of the update via
  `queueMicrotask`; `toggleRenderedAt`; portals; the rendered slots added to
  the re-measure observer). `apps/web/src/editor/CardRail.tsx` (`renderedAt`,
  the direction-carrying label).
- Test coverage: `apps/web/src/editor/DocumentEditor.test.tsx` — "renders a
  cell on open, and the rail icon swaps it for the source" (result visible
  unasked, `$ hick --version` shown, document text byte-identical while
  folded, Run wired, toggle both ways, label says which way it goes); "keeps
  a block you are still typing as source"; "puts no unnumbered row in the
  document: every row is a line or a fold".
- Caveat requiring review: the browser check did not happen — the automation
  tooling failed to load any page during this change, so the rendered cell
  and diagram have been verified only in jsdom, which does no layout. What
  that leaves unverified is specifically visual: whether the rendered block's
  height settles correctly in CodeMirror's height map once a mermaid diagram
  finishes drawing (the ResizeObserver is wired for it, and the same wiring
  worked for the old block widgets), and whether the folded region's single
  gutter number reads acceptably beside a tall picture. Both want an eye
  before this is called done.
