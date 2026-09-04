# A Citation That Points At Nothing Fails

Given a document with `cites="…"`, when `hick test` runs, then every selector
in that list is resolved **one at a time**, and any selector matching nothing
fails the run — `EXPECTATION FAILED`, exit 3 — naming the document, the line,
and the selector. `hick cites` marks the same selector in its listing.

This checks the **pointer**, never the claim. Whether a citation's assertion
is *true* is not verifiable, and nothing here pretends otherwise: that is the
whole distinction between declared provenance and derived provenance
(`three-provenances-are-drawn-apart.md`). Whether the thing it points at
exists is entirely checkable, offline, and free.

A selector that resolves is not reported, and two selectors resolving to one
fragment list that place once.

## Why

`cites=` was resolved as a **set**: the comma-separated list went to
`fragments_matching` in one piece and came back as the union. So a citation
list where some selectors resolved and one did not produced a non-empty
result, and the one that matched nothing disappeared with no warning
anywhere.

Found on 2026-09-04 by taking `examples/receipts/hick-agent/message.hick` —
the walkthrough for exactly this feature — and replacing `#fix-summary` with
`#no-such-anchor` throughout. `hick cites` printed the four selectors that
resolved and said nothing about the fifth. `hick test`, after a re-weave,
answered **`ok`**. The weave rendered the dangling anchor's name five times,
beside the real ones, in the same words.

That is the worst available failure for this feature. A message whose whole
purpose is "here is what every sentence rests on" cited something that did
not exist, and every surface agreed it was fine. Somebody could write
`cites="#anything"` and it would read as sourced.

The existing test was called
`cites_resolve_across_the_chain_and_report_dangling_ones` and included a
`#missing` selector — and asserted only that the resolved count was 2, with a
comment noting the dangling one resolves to nothing. It documented the gap as
the behaviour. It now asserts the selector is named.

A dangling citation is **not drift**. There is nothing to regenerate and no
re-run fixes it, so it takes the same outcome a failed `hick:expect` takes:
the document states something that is not so, and a person has to decide
which end is wrong.

## What this does not close

A `hick:transform` pins its **inputs** with `from=`, not its output. Editing
the model's prose while leaving the facts alone does not make the transform
stale — the document's own words are "it claims the prose was written from
exactly these bytes under exactly this instruction", and that claim is still
true of the inputs. It is caught, in practice, because the woven `.md` is
committed and the weave-drift check sees the change; with no committed weave
it passes. An edited AI passage is therefore not currently *marked as
edited*, which the run / edited / staged distinction in
`sessions-you-run-again.md` says it should be. That is a real gap and is
recorded here rather than in a commit message so it is not lost.

---

Last LLM verification:
- Date: 2026-09-04
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/lib.rs` — `DeclaredCite::dangling`,
  `declared_cites` resolving one selector at a time and de-duplicating
  places, `CheckFailure::DanglingCitation`, its `outcome()` arm, and
  `dangling_citations`; `crates/hickory-cli/src/main.rs` — the failure's
  message in `cmd_test` and the marked line in `cmd_cites`.
- Test coverage: `crates/hickory-cli/tests/cites.rs` —
  `a_dangling_citation_fails_verification` (the failure, its line, its
  selector, and that the outcome is `ExpectationFailed`),
  `a_citation_that_resolves_is_not_reported` (including two selectors
  resolving to one place), and the strengthened
  `cites_resolve_across_the_chain_and_report_dangling_ones`. Verified by hand
  against `examples/receipts/hick-agent/message.hick`, which still passes
  untouched and fails with exit 3 when a selector is broken.
- Caveat requiring LLM review: only the selector's resolution is checked. A
  citation pointing at a real fragment that does not support the sentence is
  exactly as wrong to a reader and is not checkable by anything here — that
  is what "declared" means, and the weave says so in words.
