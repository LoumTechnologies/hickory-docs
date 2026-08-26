# A Generated Picture Shows As A Picture

Given a `<hick:file>` block whose `path` names something a browser can draw
(`.svg`, `.png`, `.jpg`/`.jpeg`, `.gif`, `.webp`, `.avif`), when the document
is opened in the app, then the block shows **the picture the file holds**, and
a `▨` icon on the action rail toggles it back to the source that draws it —
the plotting code, the cell, and the tags around them.

The block's source is forty lines of R or matplotlib; what the block *means*
is one chart. A reader who has to reconstruct the chart from the code is
reading the machinery instead of the document, which is the same argument
that already makes an exec cell and a `<hick:diagram>` render by default
([a-literate-file-opens-rendered](a-literate-file-opens-rendered.md)).

**And the woven markdown carries the picture too**: a `<hick:file>` writing
one weaves as `![path](path)` rather than as a heading and a code fence. The
fence exists to show a file's source, and a chart has none a reader wants —
fencing an SVG puts forty kilobytes of markup in the middle of a document,
which is why an author previously had to write `doc-hidden="true"` and then a
markdown image line by hand. **That workaround is what made the document name
the same picture twice**, and in the app it showed as the chart drawn twice:
once for the block, once for the line beneath it. A document written today
needs neither the attribute nor the line.

This is the second half of a door that already had one side: clicking a
generated picture in the woven prose jumps to the block that writes it. Now
the block itself shows the picture, and the rail is the way back to the code.
Both directions exist so neither reading — "what does this draw?" and "how is
this drawn?" — is a guess.

Four rules keep it:

1. **The picture is loaded from the file on disk**, through the same
   `/api/asset` URL the markdown `![…](chart.svg)` beneath it uses — never
   from the cell's transcript. The panel therefore shows exactly what a reader
   of the repository sees. A file that is missing, or that holds `[never
   run]`, LOOKS wrong here rather than looking fine while the disk disagrees;
   making that disagreement visible is the point, not a side effect (see
   [a-weave-without-a-recording-keeps-the-artifact](../verification/a-weave-without-a-recording-keeps-the-artifact.md)).
2. **There are exactly TWO states, and no third.** The picture, or the code
   that draws it — as editable text, the way you would fix it. A cell nested
   inside a picture block therefore never renders on its own: doing so would
   put a read-only rendering of its own command between those two, and a
   display of source you cannot type in is the worst of both — it looks like
   the editor and refuses to behave like one. (It also cannot coexist with
   the picture: two block replacements over the same rows is something
   CodeMirror refuses outright.)
3. **The picture card has one verb, and the cell it contains loses one.**
   Running belongs to the cell, which keeps its own `▶`; the source toggle
   belongs to the picture block, and the nested cell does not get a second
   one. An action offered twice is how a two-state thing grows a third state.
4. **It is a fold, not an overlay.** The picture is a block *replacement* over
   the block's own lines, so the gutters keep counting truthfully
   ([the-gutters-never-skip-a-number](the-gutters-never-skip-a-number.md)).

## Boundary

A `<hick:file>` writing text — a README, a `.py`, a `.csv` — is untouched in
both places: read as its source in the app, woven as a heading and a fence.
The extension list is the decision, and it is deliberately a list of what a
browser draws rather than of what a cell might produce.

`doc-hidden="true"` is unchanged and still wins: a document that wants a file
written but not shown keeps saying so, and then weaves nothing for it. A
document still carrying the old pattern — hidden block plus a hand-written
`![…]` — keeps working exactly as before and shows one picture, since the
hidden block contributes none. **Nothing was migrated for it**: dropping the
attribute and the line is an edit the author makes when they want to.

The extension list is duplicated, in `is_picture_path` (Rust) and
`isPicturePath` (TypeScript), and must stay in step — the app decides which
blocks render as a picture and the weave decides which weave as one, so a
disagreement shows a chart in one place and a code fence in the other. There
is no test holding the two lists together.

Nothing here re-runs anything or writes anything. A stale picture on disk is
shown as it is, and **the panel does not yet mark it stale** — it can say
"missing" and "no recording", because those it can see in the bytes, but
"out of date" needs the cell's cache key compared against the recording the
file came from, and no such comparison reaches the app today. `hick test` is
what says a file no longer matches what the document produces.

---

Last LLM verification:
- Date: 2026-08-26
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: the weave half is `is_picture_path` and the `"file"` arm of
  `process_weave_tag` in `crates/hick-literate/src/weave.rs`, which emits
  `![path](path)` and returns before the heading-and-fence path; the fence
  path shares `strip_opening_break` with the file output so the two agree
  about the file's first byte.
  The app half: `apps/web/src/editor/hickDoc.ts` — `isPicturePath` (the
  extension list) and `pictureBlocksOf`. Card and fold wiring:
  `apps/web/src/editor/cards.ts` (the `"picture"` card kind),
  `apps/web/src/lib/railActions.ts` (`["source"]`, sharing the arm with
  `table`/`math`/`diagram`), `apps/web/src/editor/CardRail.tsx` (the `▨`
  glyph), `apps/web/src/editor/rendered.ts` (`renderableBlocks`, the
  `picture` count, `slot.picture`, its own `estimatedHeight`, and the
  `insidePicture` guard that is rule 2),
  `apps/web/src/editor/DocumentEditor.tsx` (the picture slot's portal, whose
  `src` is `assetUrl(resolveTarget(path, …))`).
  The panel is `apps/web/src/components/PicturePanel.tsx`, which fetches the
  bytes before drawing so it can tell "no recording" from "not on disk";
  styles are `.rendered-picture*` in `apps/web/src/styles.css`.
- Test coverage: `apps/web/src/editor/cards.test.ts` — a picture file block
  gets its own icon alongside the cell's, the cell is marked `insidePicture`,
  a cell outside one is not, and a file block writing text gets no picture
  card. `apps/web/src/editor/rendered.test.ts` — the nesting guard (both
  blocks asked for, one slot rendered, kind `picture`), the two-state rule
  (picture off leaves NO rendered slot, so the source is the editable
  buffer), and a cell outside a picture still rendering.
  `apps/web/src/lib/railActions.test.ts` — the picture's single verb, and
  `source: false` taking the toggle off the cell that draws it.
  Weave: `crates/hick-literate/tests/picture_weave.rs` — the image line, no
  fence and no heading, all seven extensions, a text file keeping both, the
  fence agreeing with the file's first line, and `doc-hidden` still winning.
  End to end, `examples/bootstrap-ci.md` is committed with
  `![bootstrap-histogram.svg](…)` where it used to carry the whole SVG.
- Caveat requiring review: **nothing here has been seen drawn.** jsdom neither
  fetches nor lays out an `<img>`, so `PicturePanel`'s three states are
  covered only by the wiring around them, and this change has not yet been
  opened in the running app at all — that a chart appears, at a sensible size,
  in both themes, and that the `▨` icon lands beside the block it belongs to,
  are all unverified. The extension list is a judgement call too: a `.pdf` a
  cell produces is not offered as a picture, though a browser could draw one.
