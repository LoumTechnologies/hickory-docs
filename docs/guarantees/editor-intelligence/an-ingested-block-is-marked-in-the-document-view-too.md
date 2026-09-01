# An Ingested Block Is Marked In The Document View Too

Given a `<hick:ingested>` block's content sitting in the Document view —
plain, editable text, since the owning exec cell's card deliberately stops
rendering right before it — when the view draws it, then it carries the
same `cm-prov-ingested` dashed-underline treatment the Output pane's
per-character lineage already gives ingested bytes there, not zero visual
marking.

## Why

Two views already agreed that ingested bytes deserve a distinct visual
language — `styles.css`'s `.cm-prov-ingested` (a dashed underline plus a
faint accent wash) is real, shipped CSS — but only ONE of them actually
applied it. The Output pane's per-character lineage highlighting
(`OutputEditorPane.tsx`) uses it correctly. The Document view — the primary
place a person actually reads a `.hick` document — never did: the exec
card's own rendering stops exactly at the ingested block's boundary (by
design, so the bytes stay editable), and nothing filled that gap with any
marking at all. The plain text after the card was visually indistinguishable
from prose the document's author typed by hand.

## What changed, and what the boundary still is

`buildEmbedMarks` (`apps/web/src/editor/wysiwyg.ts`) — the ViewPlugin
already responsible for embedded-language token coloring, the one
decoration family in this file that "genuinely cannot affect layout" — now
also marks any block named `ingested` with `tokMark("cm-prov-ingested")`
across its full span, reusing the SAME cached `Decoration.mark` helper and
the SAME CSS class the Output pane already ships. No new CSS, no new
extension registered — one ViewPlugin now serves two purposes for the same
mechanical reason (both are color-only marks over visible ranges).

The exec card's own cutoff point is unchanged: it still stops rendering
exactly where the `<hick:ingested>` block begins, for the reason
`rendered.ts` already states (the bytes must stay editable, not swallowed
by a card). This fix only changes what the text AFTER that cutoff looks
like — from unmarked to marked — never where the cutoff itself falls.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `apps/web/src/editor/wysiwyg.ts`'s `buildEmbedMarks`, the
  `block.name === "ingested"` branch added alongside the existing embedded-
  language loop, reusing `tokMark` and the `cm-prov-ingested` class already
  defined in `styles.css` for `OutputEditorPane.tsx`.
- Test coverage: `apps/web/src/editor/wysiwyg.test.ts` (new file, 2 tests) —
  constructs a real `EditorView` with the full `wysiwyg()` extension bundle
  over a document containing a `hick:exec` cell with a nested
  `hick:ingested` block, mounts it into `document.body` (matching
  `rendered.test.ts`'s established pattern for testing this codebase's
  CodeMirror extensions directly), and asserts the ingested text's DOM
  carries `.cm-prov-ingested` while an ordinary document with no ingested
  content carries none. Confirmed load-bearing by temporarily disabling the
  branch and observing the first test fail (0 marked elements found).
- Caveat requiring LLM review: like all `buildEmbedMarks` decorations, this
  only materializes for CodeMirror's current `view.visibleRanges` — an
  ingested block far outside the current scroll position is unmarked until
  scrolled into view, matching the existing, accepted behavior of every
  other decoration this ViewPlugin already produces (embedded-language
  colors included) rather than a new limitation this fix introduces.
