# An Element Is Declared Once

Given a tag the app draws — `exec`, `file`, `diagram`, the prose between
them, and every element added after this — when the block model is built
for a document, then the tag's name, the attributes it accepts, how it
renders to a block, which of its children the walk visits, and the actions
it answers are all read from one declaration: an implementation of
`hick_blocks::Element`, registered once in one `Registry`. The block on
the wire is `{kind, span, …props}` for every element alike, `kind` naming
the component that draws it, `span` the tag's byte span; and the registry
can describe its whole vocabulary as data.

Until this, "what is `<hick:exec>`" had about fifty answers: string
matches across the crates, a four-variant enum in the block model, six
card kinds in the editor, sixty-two entries in the Insert menu. Adding an
element meant finding them all.

Three properties hold it up:

1. **The registry walks; the element decides.** `Registry::blocks` visits
   the document's nodes in order, renders each tag through its element,
   and descends only as the element says (`Descend::All`, `None`, or
   `Named` — an `exec` shows the files it `ingested` and nothing else of
   its content). A tag no element claims is drawn as nothing, as before.
   Prose is a text renderer the registry's owner supplies.
2. **The registry is generic over the facts.** `Element<Cx>` renders from
   whatever context its registry's owner chose — `hick-literate` renders
   from a run's transcripts, expectations, files and staleness
   (`BlockModelInput`). The registry crate depends on the parser and
   nothing else, and runs nothing.
3. **A name registered twice is a panic.** Two declarations of one
   element is exactly the bug the crate exists to make impossible, so it
   is refused at construction rather than resolved by order.

## Boundary

An action is dispatched by the byte offset the tag starts at and answered
by the element; no route serves it yet — that is the generic server, step
3 of `the-minimal-core.md`. The pipeline's own per-tag processing
(`hick-handlers`, which decides what a tag *does* in a run) is a separate
concern and is not folded in here. The Insert menu's catalogue is still
hand-written; the registry's `describe` is what will replace it.

---

Last LLM verification:
- Date: 2026-09-05
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-blocks/src/lib.rs` — `Element`, `Registry`,
  `Block` (flat `Serialize`), `Descend`, `ActionError`, and its tests (walk,
  descent, action by span, describe, double registration);
  `crates/hick-literate/src/render.rs` — `registry()`, the four elements
  and the prose renderer, `build_block_model` delegating to the registry,
  and `a_block_serialises_to_the_v0_contract` pinning the wire shape;
  `crates/hickory-cli/src/lib.rs` `block_model` unchanged over the
  re-exported `Block`.
- Caveats: only the four elements the block model already drew are
  registered; the card rail's `table`, `math` and `picture` are still
  found by the editor from structure alone and join the registry with the
  frontend mirror (step 4).
