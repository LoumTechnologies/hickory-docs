# An Element Is Drawn By Its View

Given a block the editor renders in place of its source — a cell, a
diagram, a formula, a table, a picture — when the Document view mounts
something in the block's place, then what it mounts is the one view
registered for that block's `kind` in `apps/web/src/elements`, and which
blocks are renderable at all is what those views say they draw. The
editor itself has no branch per element: it asks the registry which view
draws a block, folds the block's lines, and hands the view its slot and
one context object. A cell's Run goes through the element's `run` action
(`POST /api/docs/:id/blocks/:at/:action`) when the render has given the
cell a span to address, and through the older run route until it has.

Until this, `DocumentEditor.tsx` held a chain of `if (slot.kind === …)`
branches, each closing over a different handful of the editor's state, and
`rendered.ts` decided a block's kind with a nested ternary over tag names.
Adding an element meant editing both, and the editor's 53 imports were
mostly the panels those branches mounted.

Three properties hold it up:

1. **One folder per element, one record for all of them.**
   `elements/<kind>/view.tsx` exports an `ElementView` — its `kind`, a
   `draws(block)` predicate, and `render(slot, cx)` — and
   `elements/index.ts` is the `Record<SlotKind, ElementView>` keyed by the
   same `kind` string the server's blocks carry. `slotKindOf` and
   `renderableBlocks` are derived from the views, never listed twice.
2. **The context is one object.** `SlotContext` carries what the document
   around a block knows — the live view, the path, the server's blocks,
   the running cells, the replay set, table layouts, the write-back — so a
   view's signature never grows a parameter per element.
3. **The matching lives beside the matching.** Span-overlap-else-ordinal
   matching of editor blocks to server blocks, and a diagram's assertion
   states, are `lib/blockMatch.ts`, imported by the views and re-exported
   by the editor for the callers that always found them there.

## Boundary

The frontend registry has five kinds and the server's has four elements,
and they are not yet one list: `math` and `table` are drawn by the editor
from structure alone and the server renders them as prose; a `file` is a
`picture` only when its path says so. Uniting them is the point of the
next step, not this one. The Insert menu's catalogue is still hand-written
rather than read from `GET /api/elements`. The marketing demo's mock
server stays: the demo is kept on purpose, and it now parses with the real
parser, which was the requirement.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `apps/web/src/elements/{types.ts,index.ts}` and the five
  `*/view.tsx`; `apps/web/src/elements/index.test.ts` (one view per kind,
  `draws` decides, order); `apps/web/src/editor/rendered.ts` uses
  `slotKindOf` and re-exports `renderableBlocks`;
  `apps/web/src/editor/DocumentEditor.tsx` builds `slotContext` and maps
  `renderedSlots` through `elementViews[slot.kind].render`;
  `apps/web/src/lib/blockMatch.ts`; `apps/web/src/api/client.ts`
  `elements` and `blockAction`; `apps/web/src/views/documentSession.tsx`
  `runCell` chooses the action route when the block has a span.
