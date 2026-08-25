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
- Date: 2026-08-25
- Reviewer: Claude (Fable 5)
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
    and the assertion line; `apps/web/src/editor/rendered.ts` folds the block
    behind it. **Live state is wired now**:
    `DocumentEditor.tsx::assertionStates` resolves each `asserts` id to the
    exec cell carrying it in the source (the server's exec ids are
    `container:line`, so the id attribute is matched in the structure parse,
    then span-matched to the server block) and maps its status — ok→passing,
    failed→failing, running/stale/never-run/missing→unknown.
  - Derived diagrams draw in the app: `crates/hick-literate/src/render.rs`
    emits a `diagram` block whose body has this document's `copy`/`cut`
    fragments inlined (`resolve_diagram_children`), and the editor prefers
    that body exactly when the raw source contains a `<hick:paste>`.
- Caveats — what LLM review could NOT establish:
  - **No real mermaid render has been observed.** The engine is mocked in
    tests (it wants a live browser to measure text), so what is proven here is
    the panel's behaviour around it, not that a given diagram draws.
  - The block model resolves pastes document-locally only; a paste whose
    fragment lives in another document draws nothing in the panel (the weave
    still resolves it fully).
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
  `apps/web/src/editor/DocumentEditor.test.tsx` ("live diagram assertions")
  covers the id→cell→state mapping, the in-document "out of date" line, and a
  derived diagram drawing from the resolved body.
  `crates/hick-literate/src/render.rs::tests` covers the block model's
  paste resolution and `#`-stripping.
