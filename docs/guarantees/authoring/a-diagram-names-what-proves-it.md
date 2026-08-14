# A Diagram Names What Proves It

Given a `<hick:diagram>` in a document, when the document is run or woven,
then the diagram weaves into a fenced block tagged with its `renderer` — so
the woven markdown renders wherever markdown renders — and its `asserts`
attribute is checked against the document: a reference to an id no tag carries
is a warning naming the missing id, and a diagram with no `asserts` at all is
a warning saying that nothing will fail when the picture stops being true.

The reason is that a diagram is the one claim in a repository that nothing
re-reads. Tests fail and types fail; a drawing just quietly stops describing
the system. `asserts` is what binds a picture to the cells that prove it, and
these warnings are what keep that binding from rotting silently — a renamed
cell is the ordinary way a proof gets unhooked from the thing it proved.

Both stay **warnings**, never errors. Whether a picture needs proof is the
author's call — a sketch of something outside this repository is a legitimate
thing to draw — and a tool that refused to weave an unproven diagram would only
teach people to draw somewhere it could not see.

In the notebook the same distinction is visible rather than merely logged: the
panel under a checked diagram names the cells that check it, and the panel
under an unchecked one says nothing checks it. Those must not look alike; a
drawing that appears authoritative and is not is the exact failure this
feature exists to prevent.

See `docs/specs/freeform/diagrams-that-fail-when-they-lie.md` for why this is
a tag rather than a markdown fence, and for the renderer staging
(mermaid now, d3 later, first-party SVG maybe never).

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - Weaving: `crates/hick-literate/src/weave.rs` — the `"diagram"` arm emits
    ```` ```<renderer> ```` around children processed by the same path
    `hick:file` uses, so `<hick:paste>` inside a diagram resolves and a
    picture can be derived rather than drawn. `renderer` defaults to
    `mermaid`.
  - Warnings: `crates/hickory-cli/src/lib.rs::diagram_assertion_warnings`,
    emitted from the same place as the escaping and mount warnings — before
    execution, so they are seen.
  - Run end to end on this machine against a document with three diagrams
    (one proved, one with no `asserts`, one naming a deleted id): both
    expected warnings printed, naming line numbers and the missing id, and
    `arch.md` came out with three ```mermaid fences.
  - Notebook: `apps/web/src/components/DiagramPanel.tsx` renders the picture
    and the assertion line; `apps/web/src/editor/wysiwyg.ts` mounts it above
    the block via the same slot/portal mechanism the cell panels use. Tests in
    `DiagramPanel.test.tsx` cover drawing, a parse failure leaving the source
    intact, the unchecked wording, the out-of-date wording, an unknown
    renderer, and a stale async answer being discarded.
- Caveats — what LLM review could NOT establish:
  - **The panel does not yet know whether an assertion passed.** It reports
    which cells check the diagram, always with state `unknown`, so the
    "out of date" wording is exercised only by its unit test. Wiring it to
    live run state is named in the spec as not-built.
  - **No real mermaid render has been observed.** The engine is mocked in
    tests (it wants a live browser to measure text), so what is proven here is
    the panel's behaviour around it, not that a given diagram draws.
  - Nothing checks that a diagram's `asserts` cell is one that could
    *possibly* fail — a diagram can name a cell that always prints `0`. That
    is unfalsifiable-by-construction and no different from a test that asserts
    nothing; it is a documentation problem, not a mechanism one.
- Test coverage: `crates/hickory-cli/tests/diagrams.rs` (8 tests) drives the
  real binary for the weave — fence emitted, `renderer` honoured, tag absent
  from the output — and covers every warning branch: correctly bound (silent),
  renamed cell, missing `asserts`, a selector that is not an `#id`, several
  assertions at once, and that an unchecked diagram still weaves.
  `apps/web/src/components/DiagramPanel.test.tsx` (6 tests) covers the panel.
