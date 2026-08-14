# The Lineage Browser Shows One Column Per Stage

Given a project of documents connected into a pipeline, when the lineage
browser opens it, then there is one column per **stage** — a document plus the
files it generates — named for the stage, with one of the stage's files opened
in place inside the stage's own file tree; and every link drawn between blocks
is one the weaver computed, never one the view invented.

The shape answers a question two panes cannot: *why does this say what it
says?* The answer usually lives in another document, and reaching it must not
cost you the place you were reading.

Six rules hold, and each exists because the obvious alternative misleads:

- **A column is named for its stage, not for its open file.** Opening a
  generated file must not make the column look like it has become that file.
- **Links are typed and separable.** `paste` is computed from provenance;
  `asserted` is a claim a person made. Switching a kind off changes the graph
  traversal, not just the drawing, so turning off what nothing verifies
  shortens the chain to what is actually checked. The language cannot declare
  an influence yet, so today every link in the app is `paste`.
- **A link never simply disappears.** When an end is folded into a hole or
  sits in a file the column has closed, the link docks there and the hole or
  file row reports how many ended in it. Absence would read as "no such
  relationship", which is the one thing the picture must not say.
- **Both ends of a link can share a column**, because a fragment and the file
  it weaves are one stage — and that is the commonest link there is. Those
  draw as a bracket bowing out into the gap and back.
- **Selecting never closes the file you are reading.** Revealing what a block
  links to may open a file in another column, but never in a column already
  showing part of the same lineage: answering "what does this feed?" by
  destroying the question is not an answer.
- **The open file's row shows its relative path and stays pinned** while you
  scroll it. A name alone is not a location.

Holes work as they do in a pull request: drag the line-number gutter to fold,
and each hole has two edges that each move both ways — the arrow says which
direction that edge travels, and the control's position says which edge it
drives.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified in a real browser against the local API's data shapes
- Evidence:
  - Model and traversal: `apps/web/src/lineage/model.ts` with 11 unit tests
    (`model.test.ts`) covering range merging when regions touch, folding that
    splits a range, all four hole-edge moves, search folding, kind-filtered
    traversal, cycle termination, and tightest-node selection.
  - Links from real provenance: `apps/web/src/lineage/build.ts`, tested in
    `build.test.ts` by weaving a document with the app's own weaver and
    asserting the links point from the fragment to both files that paste it,
    and back again.
  - Behaviour: `LineageColumns.test.tsx` (9 tests) drives the component —
    stage naming, opening a generated file, gutter folding, the four hole
    controls, per-column and global search, selection marking, kind toggling,
    and that selection does not close the file being read.
  - Driven in Chromium against the mock API (the same shapes the local server
    returns): two stages rendered side by side, `▶ n` execution counts per
    file, four links docking into a closed generated file with a `4 ↦` badge,
    and the sticky path row measured flush against the first line at four
    scroll positions.
- Caveats — what LLM review could NOT establish:
  - **Nothing here edits.** The browser reads; the round trip that carries an
    edit from a generated file back into its fragment lives in the split view
    and in the landing demo's "Edit both ends" mode. A lineage column is not
    yet a place you can type.
  - **Not driven against the real local server**, only against the mock that
    implements the same client contract. The desktop app has no headless mode,
    so this is the closest check short of launching a window.
  - Cross-document links are unexercised by the fixtures available here: the
    mock project's documents each weave their own outputs, so every link
    observed was within one stage. The cross-column path has unit coverage
    but no browser run.
  - One request per generated file to fetch provenance. Fine for a handful;
    a project with hundreds of outputs will feel it.
- Test coverage: `apps/web/src/lineage/*.test.ts{,x}` (26 tests), plus
  `src/router.test.ts` for the project-scoped route and
  `src/landing/demos/demos.test.tsx` for the home page demo.
