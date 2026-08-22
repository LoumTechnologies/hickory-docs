# A Rewritten Link Keeps Its Paragraph Attached

Given a paragraph of prose in a `.hick` document that a transform cuts into
pieces — a `hick:val` substitution, a rewritten cross-document link — when the
document is woven, then every piece that came through UNCHANGED still carries
the source bytes it came from, narrowed to exactly its own extent; only the
bytes a transform actually replaced are reported as synthetic. A ribbon is
drawn from the sentence in the source to the sentence in the weave, and an
edit made on the woven side still maps back.

## What this replaces

Before this, a passthrough segment inherited the WHOLE input node's origin
span. The lineage layer keeps an origin only when the span's length matches
the bytes it produced — correctly, because a 12-byte segment claiming a
200-byte region would send a reverse edit to the wrong place — so every piece
of a split paragraph degraded to `synthetic`. The effect was that **one
substitution anywhere in a paragraph erased the ribbons for the whole
paragraph**, and the cross-document link rewrite would have done the same to
every paragraph containing a link.

## The rules

1. **A passthrough segment's span is narrowed to the bytes it covers.**
   `ProvenanceTransformNode` tracks how far into the input text it has walked
   and cuts the origin span to match.
2. **A substituted segment advances the walk by the length of what it
   REPLACED, not what it produced.** That is why `TransformSegmentOrigin::
   Substituted` carries the pattern at all. Advancing by the output length
   would slide every span after the first substitution by the difference —
   silent, and off by exactly as much as the substitution changed.
3. **Narrowing happens only when the node's text IS its source span byte for
   byte** — the same condition lineage already checks before trusting an
   origin. When a dedent has changed the text, the whole-span origin stands
   and the old degrade-to-synthetic behaviour applies. A wrong mapping is
   worse than no mapping.
4. **A segment that would run past the end of its span is not narrowed.**
   Arithmetic that cannot be right is not guessed at.
5. **`start_line` / `start_col` stay the node's own.** They locate a region
   for a human reading an error; recomputing them would need source text the
   node does not have, and lineage maps with the byte offsets.
6. **Origins with no span — exec output, synthetic, a paste with no located
   source — come back unchanged.** There is nothing to narrow.

## Boundary

This narrows within ONE input node. A transform that reorders or duplicates
text would break the walk, and none does: every segmenter in the codebase
emits its segments in input order, covering the input exactly once. That is a
property of the segmenters, not something this node can check, and a new
segmenter that violated it would produce wrong spans rather than an error.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-flow/src/transform.rs` — `narrowed` (rules 1, 3, 4,
  5, 6) and the `consumed` walk in `ProvenanceTransformNode::get_stream`
  (rule 2). The condition in rule 3 mirrors
  `crates/hickory-lineage/src/lib.rs::from_provenance_map`, which is where a
  length mismatch becomes `synthetic`.
- Test coverage: `crates/hick-flow/src/transform.rs::narrowing_tests` (4) —
  three consecutive segments getting three disjoint spans, a text whose length
  disagrees with its span being left alone, an over-running segment being left
  alone, and span-less origins passing through.
  `crates/hick-literate/tests/document_links.rs` — the end-to-end pair:
  `the_prose_around_a_rewritten_link_keeps_its_ribbon` (the words before the
  link, the label, and the words after are all `Literal`; only the rewritten
  destination is `Synthetic`) and
  `a_span_after_a_rewritten_link_still_points_at_the_right_source_bytes`
  (rule 2, asserted by slicing the source with the span the map returns).
- Caveat requiring review: the substitution path — the original reason this
  node exists — gains the same fix and is covered only indirectly, through the
  link tests that share the mechanism. A `hick:val` inside a paragraph now
  keeps its ribbons too; that was checked by reading `apply_substitutions_
  segmented`, not by a test of its own.
