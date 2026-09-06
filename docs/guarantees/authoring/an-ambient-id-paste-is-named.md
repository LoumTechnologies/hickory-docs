# An Ambient `#id` Paste Is Named

Given `<hick:paste select="#x" />` in a document that declares no fragment
`#x` and names no `hick:upstream` or `hick:include` that declares one, when a
sibling `.hick` in the same folder declares `#x` and the paste therefore binds
to it, then the run says so, at run, test and weave time, through the same
warning channel the diagram and claim warnings use:

```
paste "#x" at notes/owner.hick:12 resolved from sibling other.hick, which this
document does not name (declare <hick:upstream file="other.hick" /> to mean it,
or a .class selector to collect)
```

A **warning, never an error**. The binding may well be what the author wants,
and refusing to weave would break every folder already relying on it —
the same reasoning the diagram assertions and the claim markings take. What it
must not be is silent.

## Why

Folder-wide discovery is the feature
(`many-documents-contribute-to-one-file.md`): many documents feed one shared
file and none of them names the others. The hazard is that exact mechanism
read the other way. When a document's OWN fragment for `#x` is renamed or
deleted and a sibling happens to declare `#x` too, the paste keeps resolving —
to somebody else's bytes — and the tangled file is quietly wrong. Nothing
fails, because everything resolved.

Found twice in one 760-document repository, both times on a generic id
(`#comment`, `#imports`) — which is the shape of the failure: the more
ordinary the name, the more likely some other document in the folder uses it.
Rule 3 of the contribution guarantee ("local wins, ambient adds") is what
makes the *silent* case possible: while the local declaration exists nothing
is wrong, and the day it goes away nothing says so.

## What is warned about, and what is not

- **Only `#id`.** A `.class` selector is multi-match by design; collecting a
  sibling's contribution is the whole point of writing one, and warning about
  it would make the feature unusable.
- **Only ambient.** A `hick:upstream` or `hick:include` the author wrote IS
  the document naming it. Those fragments are spliced before the sibling scan
  runs, so a declared edge never reaches it.
- **Only when the document does not answer it itself.** A local fragment, or
  one from a declared edge, resolves first and is not a sibling binding.
- **Only when a sibling really answers it.** A `#id` matching nothing anywhere
  is a different problem, reported where dangling selectors are reported.

## Boundary

The warning names the FIRST contributor that declares the id, which is the one
resolution binds to — contributors are visited in sorted order, and that order
is what makes a folder weave the same file on two machines. If two siblings
both declare `#x`, only the winner is named.

The check re-reads the folder rather than watching the splice, so it costs one
parse per sibling per run. That is the same read the splice itself does, and
it buys a warning that does not have to thread through the pipeline's rounds.

---

Last LLM verification:
- Date: 2026-09-06
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-literate/src/ambient.rs` — `ambient_id_paste_warnings`
  and `unresolved_own_id_pastes` (own pastes only, `#id` only, and only those
  `fragments_matching` cannot answer after `resolve_includes`), over the
  `contributors` list the splice itself uses;
  `crates/hickory-cli/src/warnings.rs` — `warn_before_run`, which is where
  `run_doc_subset` emits every pre-run warning, so run, test and weave all
  carry it.
- Test coverage: `crates/hickory-cli/tests/ambient_contribution.rs`, module
  `an_ambient_id_paste_is_named` — `an_id_that_resolved_from_a_sibling_is_named`
  (the message, and that the file is still written),
  `a_class_selector_is_never_warned_about`,
  `a_declared_upstream_is_the_document_naming_it`, and
  `a_documents_own_fragment_is_not_a_sibling_binding`.
- Caveat requiring LLM review: the app shows this because it shows the log
  channel; nothing draws it on the block that pasted. Whether an ambient
  binding deserves a mark in the editor is unanswered here.
