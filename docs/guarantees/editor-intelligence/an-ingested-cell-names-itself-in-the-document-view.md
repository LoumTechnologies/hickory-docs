# An Ingested Cell Names Itself In The Document View

Given an exec cell that owns a `<hick:ingested>` child, when its card renders
in the Document view, then the card's status bar names the run — file
count, and its fingerprint and date on hover — the same fact the woven
markdown's "Ingested from …" caption already states for free.

## Why

`rendered.ts`'s exec card deliberately stops rendering exactly where an
`<hick:ingested>` block begins ("a cell that OWNS ingested files stops
rendering where they begin"), so those bytes stay plain, editable text
instead of being swallowed by the card. That is the right call — but until
now nothing filled the gap it leaves: `CellPanel.tsx`'s bar showed the
container, a status chip, and (when verified) a "✓ output verified" chip,
and then nothing said the cell also owns real, ingested bytes. A reader
scrolling the live Document view got no signal at all that what follows the
card arrived from a real run rather than from the document's author — the
exact fact the WOVEN markdown communicates automatically, and the Document
view is supposed to be the same document, live.

## What changed, and what stayed a server-computed fact

`Block::Exec` (`crates/hick-literate/src/render.rs`) gained an optional
`ingested: Option<IngestedInfo>` field, populated in `exec_block` by finding
an `<hick:ingested>` child and reading the exact same attributes
(`from`, `at`, `sha256`, `files`, `skipped`) `weave_ingested_block` already
reads to write the woven caption — one server-side computation, two
surfaces. `CellPanel.tsx`'s `IngestedChip` renders it as a small badge next
to the verified chip, styled with the same accent
(`.cell-ingested`/`var(--accent-3, var(--accent-2))`) the Output pane's
`cm-prov-ingested` lineage highlight already uses, so the two views agree
about what color means "these bytes are foreign."

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hick-literate/src/render.rs`'s `IngestedInfo` and its
  population in `exec_block`. `apps/web/src/api/types.ts`'s `ExecBlock.ingested`.
  `apps/web/src/components/CellPanel.tsx`'s `IngestedChip`.
- Test coverage: `render::tests::an_exec_owning_an_ingested_block_carries_its_fingerprint`
  and `render::tests::an_ordinary_exec_carries_no_ingested_info` in
  `crates/hick-literate/src/render.rs` (the server-side block model).
  `CellPanel.test.tsx`'s `CellPanel ingested provenance` suite (3 tests: the
  chip names the file count and carries the date/fingerprint/skipped-count
  in its tooltip; a document with nothing skipped says nothing about
  skipping; an ordinary cell shows no chip at all). Confirmed load-bearing
  on the frontend by temporarily disabling the chip's render and observing
  two of the three tests fail.
- Caveat requiring LLM review: the chip reads the CURRENT document's
  `<hick:ingested>` attributes directly — it is not itself a live check that
  those attributes are still accurate (that is `hick lineage`'s job). A
  hand-edited `sha256=`/`files=` attribute would be reflected here exactly
  as written, faithfully rather than verified.
