# A Paste Collects On Terms It Can Be Held To

Given `<hick:paste>`, three attributes decide what it collects, and each is
enforced rather than advisory.

**`select=".a.b"` requires BOTH classes.** CSS has always read a compound
class selector as "carries all of these", and hick's selectors are CSS-shaped,
so this is that form rather than a new grammar. A comma still means union
(`.a,.b` is "either"), which is also CSS. Together they replace what a
name/value tag system would do: matching on two properties at once is the
load-bearing half, and bare names carry it.

**`distinct` collapses matches whose text is identical, keeping the first.**
Dedup is the collector's policy, not the contributor's: several documents
independently asking for `bin/` is the normal shape of a shared file, and none
of them should have to know about the others. First wins, so the surviving
bytes belong to the earliest contributor and the ribbon points somewhere
stable. It spans the whole paste, not each comma part — two selectors matching
one fragment are exactly the overlap `distinct` was written to collapse.

**`min=` / `max=` fail the run.** They count what the paste will EMIT, so
`distinct` is applied before counting: a `min="2"` satisfied by the same line
contributed twice was never satisfied.

Checked against a **run**, never against a weave. A fragment a cell writes does
not exist until that cell has run, so enforcing this in `hick weave` or
`hick lineage` — which execute nothing on purpose — would report every such
document as broken, the same reason a cell with no transcript weaves as
`[never run]` instead of failing.

## Why the gate needed building twice

`min=` already existed and already returned an error. The caller that renders a
`<hick:file>` body logged that error and carried on, so the document wove an
**empty file** and `hick run` and `hick test` both exited **0** — the same
silence the comma-selector fix in `resolve_paste_node` was written to stop, one
layer up. A gate that exists to catch a caret collecting nothing must not be
the thing that says nothing.

---

Last LLM verification:
- Date: 2026-08-31
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `required_classes` / `has_all_classes` in `crates/hick-exec/src/state.rs`
    — the compound-class match, used by both the node and string paths and by
    `count_paste_matches`, so what is counted is what is emitted.
  - `distinct` read by `PasteHandler` via `hick_handlers::has_flag`; a bare
    valueless attribute parses because `hick-lang`'s attribute loop now
    accepts one (additive: that spelling was a syntax error before, so no
    document that parsed parses differently).
  - `record_paste_failure` / `paste_failures` / `clear_paste_failures` on
    `MultiDocumentState`, and `refuse_on_paste_failures` in
    `crates/hick-literate/src/lib.rs`, called only on the executing path.
    Failures are cleared before each pass so a later round can satisfy a gate
    an earlier one could not.
- Test coverage: `crates/hickory-cli/tests/paste_terms.rs`.
- Caveat requiring LLM review: `distinct` compares a node's settled text, and
  a node with no settled text yet is kept rather than compared. That is the
  safe direction (nothing is dropped on a guess) but means a streaming
  fragment identical to a literal one is not collapsed.
- Known wart, pinned by a test rather than fixed: `run_doc_cached` **stages**
  the woven files before executing, so a cell can run a file its own document
  assembles on the first run. That staging weave cannot fairly judge a gate —
  nothing has run yet — so a failing `hick run` leaves the empty output file
  it staged. The run fails loudly with exit 1, which is what the silence
  needed; cleaning up staged files after a failed run is a separate change,
  and `an_unmet_min_fails_the_run_and_writes_nothing` asserts the current
  behaviour so it cannot drift unnoticed.
