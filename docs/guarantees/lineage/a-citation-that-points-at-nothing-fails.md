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

A `#id` selector resolves to **any element carrying that id** — `hick:exec`,
`hick:table`, `hick:file`, `hick:diagram` — not only the pasteable fragments
(`copy`, `cut`, `transcript`, `said`). A citation names what backs a
sentence, and what backs it is most often the cell whose expectations prove
it. `hick cites` lists the element's kind beside its id (`exec
#cited-tests`).

`hick:paste` is unchanged and deliberately narrower: it reproduces bytes, and
an exec cell is a computation. A paste whose `#id` names an element of
another kind is refused by kind, naming it — "paste selector '#cited-tests'
names a `<hick:exec>`, and that is not text to paste" — instead of resolving
to nothing.

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
- Date: 2026-09-06
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-lang/src/lib.rs` — `elements_matching` and the shared
  `collect_matching_where`, beside `fragments_matching`, which keeps
  `is_fragment_tag` and so keeps `hick:paste` and `hick:transform` narrow;
  `crates/hickory-cli/src/lib.rs` — `declared_cites` resolving each selector
  through `elements_matching`; `crates/hick-handlers/src/handlers/paste.rs` —
  `unpasteable_element` and `PASTEABLE`, recorded through
  `record_paste_failure` so the run refuses rather than weaving a hole;
  `crates/hick-exec/src/state.rs` — `element_kinds` / `register_element_kind`,
  populated by `register_element_kinds` in `crates/hick-literate/src/lib.rs`.
- Test coverage: `crates/hickory-cli/tests/cites.rs` —
  `a_citation_resolves_to_any_element_carrying_the_id` (exec, table and file
  all resolve and none is dangling) and `a_paste_of_an_exec_is_refused_by_its_kind`
  (the run fails and the message names `hick:exec` and the id).
- Caveat requiring LLM review: the refusal is checked against a RUN, like the
  other paste failures, so `hick weave` still resolves such a paste to
  nothing silently. That is the same trade `refuse_on_paste_failures`
  documents — a fragment a cell writes does not exist until the cell has run.

Previous verification:
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
