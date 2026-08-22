# Three Provenances Are Drawn Apart, And Any Subset Can Be Shown

Given a document on screen, when the overlay draws where its text came from,
then it draws up to three families — **lineage** (the weave's byte-exact
derivation), **context** (what was in front of the model when an agent wrote
these lines, from the session record), and **declared** (what the author says
it cites, `cites="…"`) — each in its own hue AND its own stroke (solid
palette / dotted amber / dashed grey), each with a title that says what it is
and what it cannot claim; and three toggles in the status bar show any
subset, remembered per browser.

The reason is the rule in `provenance-and-standing.md`: derived evidence and
declared assertion must never look alike — and now there are two kinds of
derived evidence that must not look alike either, because one says "these
bytes" and the other says "was present". A reader who took a dashed citation
for proof, or a context ribbon for derivation, would be misled by the very
feature that exists to keep them honest.

Three properties hold it up:

1. **Families are structural, not cosmetic.** `Shape.family` is set where the
   shape is built; lineage shapes come only from file provenance, context and
   declared shapes only from `links` the workspace derived from the two APIs.
   The CSS keys off the family; nothing draws a link in the lineage palette.
2. **Declared weaves as words.** A `cites=` on a claim or transform weaves
   `*cites: …*` after the prose — a selector list a reader can follow, never a
   checkmark, and the prose itself is untouched so its spans keep their
   lineage.
3. **Off means off.** A toggled-off family is not measured or drawn; an
   explicit empty set is remembered as a choice.

## Boundary

Context and declared far ends terminate on chrome (a tab, a tree row, a
port); clicking opens the target but does not yet select the exact lines.
Declared citations that resolve to nothing are reported by `hick cites`, not
drawn. A citation of a fragment in the same document draws to that
document's own tab.

---

Last LLM verification:
- Date: 2026-08-22
- Reviewer: Claude Fable 5
- Result: verified by driving the served UI: on
  `examples/receipts/claude-code/message.hick` the overlay listed declared
  shapes for every `cites=` (to the meeting's tab, analysis's tree row, fix's
  port) and on `examples/receipts/hick-agent/analysis.hick` context shapes for
  the CSV and for each session input; the three toggles render with their
  swatches. Screenshots: `examples/receipts/context-ribbon.png`,
  `examples/receipts/declared-ribbon.png`.
- Evidence: `RibbonFamily`, `RibbonLink`, `links`/`layers` in
  `apps/web/src/shell/Ribbons.tsx`; `apps/web/src/lib/provenanceLayers.ts`;
  `apps/web/src/shell/ProvenanceToggles.tsx`; `ribbonLinks`, the two fetches
  and the ports in `apps/web/src/views/WorkspaceView.tsx`; the
  `.ribbon-family-*` rules and `--ribbon-context`/`--ribbon-declared` tokens
  in `apps/web/src/styles.css`; `declared_cites` in
  `crates/hickory-cli/src/lib.rs`; the `cites` weave line in
  `crates/hick-literate/src/weave.rs` (claim) and
  `crates/hick-handlers/src/handlers/transform.rs`; `cited_ids` in
  `crates/hickory-cli/src/main.rs`.
- Tests: `apps/web/src/lib/provenanceLayers.test.ts`;
  `crates/hickory-cli/tests/cites.rs`; `restamp_tests::cited_ids_keeps_only_ids_the_input_labelled`.
  The multi-family overlay itself is not unit-tested (it measures DOM).
