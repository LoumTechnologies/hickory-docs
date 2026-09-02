# A Ribbon Is Drawn Where It Is Asked For

Given a document and its generated file open beside each other, when the
overlay has connections to draw, then it **paints only the ones whose blocks
hold the caret** — the connection from the block the caret is in to the text
that block produced, and nothing else; moving the caret into another block
paints that one instead; and the old reading — every connection, all the
time — is one choice away in Settings → Appearance → **Lineage visibility**,
where **With the caret** is the default and **Always** restores it.

The reason: a document that pastes a dozen fragments into a file draws a
dozen braces over the text those fragments are about. Each one is true, and
together they are a picture nobody is reading — a wall of curves between two
panes of prose. Provenance answers "where did this come from", and that
question is asked at a place: the block someone is working in. Drawing the
answer there and nowhere else costs nothing, because every other connection
is still measured, still hoverable, and still one caret move away.

Three properties hold it up:

1. **Involvement is the shape's own record, not a new one.** A shape already
   carries `hl` — the character ranges it tints on each anchored editor,
   which are exactly the blocks it joins. `Shape.caret` is one predicate over
   those ranges (`caretTouches`, overlap rather than containment, so a
   selection across a block counts as much as a caret inside it).
2. **One caret, the focused editor's.** Both ends of a connection have a
   cursor in them at all times; only the editor that has focus is asked, and
   when focus leaves for a menu or the tree the *last* focused editor keeps
   answering — a ribbon that vanished because you reached for a button would
   be a ribbon you could not click.
3. **Hiding is drawing, not measuring.** The geometry pass is unchanged: the
   same shapes are built and compared, and visibility is read at render.
   Hover reveal, whitespace-only attribution, and chrome terminals behave
   exactly as before, so nothing the overlay knows becomes unreachable.

## Boundary

The caret moves the picture, so the picture moves while you type — that is
the point, and it is also why the setting exists: reading a whole document's
provenance at once is a real way to read it, and the only way to notice a
relationship you did not already suspect. A connection whose blocks are both
off-screen has no caret in it and is not drawn, which is the same as before.
The setting is per browser (localStorage) and read at workspace mount, like
the other Appearance choices.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude Opus 5
- Result: verified by reading the implementation and running the suite.
- Evidence: `apps/web/src/lib/ribbonVisibility.ts` (persistence, default
  `"caret"`, `caretTouches`), `apps/web/src/shell/Ribbons.tsx`
  (`Shape.caret`, the `lastFocused` ref, the `selectionchange` schedule, and
  the `shown` gate at render), `apps/web/src/views/SettingsView.tsx`
  ("Lineage visibility"), `apps/web/src/views/WorkspaceView.tsx` (read at
  mount, passed as `visibility`).
- Test coverage: `apps/web/src/lib/ribbonVisibility.test.ts` (persistence and
  the predicate) and `apps/web/src/views/SettingsView.test.tsx` (the row, its
  default, and that picking "Always" persists). The gate itself sits in the
  overlay, which measures DOM and is not unit-tested — LLM/manual review.
