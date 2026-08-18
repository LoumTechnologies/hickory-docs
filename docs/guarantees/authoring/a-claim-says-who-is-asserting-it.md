# A Claim Says Who Is Asserting It, And Never Looks Verified

Given a `<hick:claim>` in a document, when the document is woven, then it
emits an attribution naming who made the claim, on what standing, and about
what scope — and the claim's own prose passes through unchanged. When the
marking is incomplete or uses a standing the document does not know, then a
**warning** names the problem, the value, and the vocabulary; never an error.

The reason is that a claim is the one thing in a note nothing can check. A
`hick:exec` output re-derives, a `hick:transform` passage carries a fingerprint
of the bytes it read, and `git blame` anchors authorship in a commit. Standing
has none of that: whether Sam knows about Postgres indexes is a judgment one
human makes about another, and no amount of machinery will ever verify it. So
the guarantee is not that a claim is true — it is that a claim never gets to
*look* verified, and that the reader can always see whose assertion it is.

Four properties hold it up:

1. **Attribution is mandatory in spirit, warned in practice.** A claim with no
   `by=` is indistinguishable from the document's own prose, which is what
   marking it was supposed to prevent — so it warns, naming that reason.
2. **The standing vocabulary is small and closed enough to compare.** `expert`,
   `judgment`, `report`, `assumption`. A document extends it with a
   `standings:` frontmatter list, so a team whose real distinctions do not fit
   is not stuck; an unknown standing warns and names both the value and the
   escape hatch, so the message teaches the vocabulary rather than just
   rejecting.
3. **Expertise without a scope is warned about.** Expertise is never global,
   and an expert speaking outside their scope is precisely the situation this
   marking exists to make visible. `standing="expert"` with no `scope=` is the
   unfalsifiable spelling of the very distinction being drawn.
4. **Everything here is a warning.** A tool that refused to weave an
   imperfectly marked claim would teach people to stop marking claims, and an
   unused marking system is worse than none — it makes the marked subset look
   complete.

## Boundary

**Unmarked prose is not warned about, ever.** Marking is sparse by design: you
mark the sentence somebody might later act on and regret, and leave the rest
alone. If ordinary prose produced warnings, every note would become a wall of
them and people would stop writing notes in the tool.

**`by=` is an assertion, not evidence.** Nothing verifies that Sam said it.
This is different from an ingested transcript, where the speaker is derived
from the source bytes, and different again from `git blame`, which anchors who
committed the span. The three must not render alike; see
`docs/specs/freeform/provenance-and-standing.md`.

**The woven prose is not blockquoted.** Prefixing every line would make the
woven bytes differ in length from the spans they came from, and the lineage
layer drops an origin whose span no longer matches — costing the claim's own
text its ribbons and its editability. The header carries the marking instead.

---

Last LLM verification:
- Date: 2026-08-18
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-literate/src/weave.rs` — the `"claim"` arm of
  `process_weave_tag` (attribution header, children through the ordinary
  content path so spans survive). `crates/hickory-cli/src/lib.rs` —
  `claim_warnings`, emitted from `run_doc` beside the escaping, mount, and
  diagram warnings, so they appear before execution rather than after.
  `crates/hick-lang/src/lib.rs` — `STANDINGS`, and `standings` as a reserved
  frontmatter key read through `Frontmatter::list`, which answers identically
  for the block and inline YAML list spellings.
- Test coverage: `crates/hickory-cli/tests/claims.rs` (8 tests) drives the real
  binary for the weave — attribution, standing, and scope present, body carried
  through, tag absent from the output — and covers every warning branch:
  missing `by`, missing `standing` (asserting the message names all four
  values), unknown standing (naming the value, the vocabulary, and the
  `standings:` escape hatch), the escape hatch working in both list spellings,
  expertise without scope, a well-formed claim being silent, and unmarked prose
  being silent. `crates/hick-lang/src/lib.rs::tests` covers the list spellings
  directly.
- Caveats — what LLM review could NOT establish:
  - **No app surface draws a claim.** The non-negotiable rendering property in
    the spec — that derived provenance and declared standing must never look
    alike — is currently enforced by nothing, because there is only one
    renderer and it is the markdown weaver. This is the most likely place the
    design turns out to be wrong.
  - **The woven attribution does not mark where a claim ENDS.** A reader of the
    markdown sees where the claim starts and has to infer its extent. Bracketing
    it would require rewriting the prose bytes, which property 4 of the boundary
    above rules out.
  - **Nothing checks that a claim is worth marking**, or that an unmarked
    sentence should have been marked. That is unfalsifiable by construction and
    is a documentation problem, not a mechanism one.
