# An Upstream Refusal Names The Declaring Document

Given an edit to a generated file whose bytes were pasted from a fragment
declared in an **upstream** document, when `edit_output` refuses that edit as
unreproducible, then the refusal names the document where the fragment is
**declared** — not the document where it is pasted — and shows the fragment,
so the next call needs no search.

A refusal in this tool is routing rather than failure: it exists to say "not
here, there". That only works if "there" is right. `Origin::Paste` records
the location of the `<hick:paste>` tag, which inside one document sits close
enough to the fragment to be useful — and across a `hick:upstream` edge is a
different FILE. The old message sent the reader to the paste tag in the
downstream document, which is the one place where changing something cannot
possibly help; an agent that obeyed it would edit the tag that performs the
paste and call the job done.

Resolution follows the selector, not a guess: find the `<hick:paste>` at the
refused location, read its `select`, and look for the matching
`hick:copy`/`hick:cut` across the pipeline closure — primary document first,
then upstreams. A fragment found in the same document as the paste yields no
extra sentence, because the existing wording already points at the right file.
No match yields no extra sentence either: a better message must never become a
way to fail.

This is a message, not a capability. Editing a generated file whose bytes come
from an upstream fragment is still **refused**, and nothing is written to
either document. Making that edit land is issue #13.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `crates/hickory-agent/src/tools/mod.rs` — `EditSession::fragment_home`
    resolves the paste site to the declaring document; the `LineageError::
    Conflict` arm prefers it over the paste-site location.
  - The search walks NESTED tags (`all_tags`). `HickDocument::tags()` is
    top-level only, and neither tag this needs is top-level: a
    `<hick:paste>` lives inside the `<hick:file>` it fills. The first version
    of this fix silently found nothing for exactly that reason.
  - Reproduced by hand before and after. Before: *"use edit_doc on down.hick
    lines 3-5"* followed by the `hick:upstream`, `hick:file`, and
    `hick:paste` tags. After: *"Those bytes are pasted in from another
    document. Edit the fragment where it is DECLARED: use edit_doc on
    up.hick, from line 3"* followed by the `hick:copy id="greet"` block.
    Both documents were byte-identical afterwards in both runs.
- Caveats — what LLM review could NOT establish:
  - **Only the `Conflict` refusal routes this way.** A single-line edit inside
    a pasted block takes a different path and is refused with *"the mapped
    document edit would break the document (parse error …)"*, which names no
    location at all. That message is safe but unhelpful, and it is the more
    common one to hit.
  - The declaration is reported as a line, not a range: only the opening
    tag's span is known here, so the excerpt below it is context rather than
    a claim about where the block ends.
  - A selector matching fragments in more than one closure document takes the
    first hit, primary first. The parser already rejects duplicate ids across
    a chain, so this should be unreachable; nothing tests that it is.
- Test coverage:
  `crates/hickory-agent/tests/tools_edit_session.rs::a_refusal_across_an_upstream_edge_names_the_upstream_document`
  drives the real tool over a two-document chain and asserts the refusal names
  `up.hick` and shows the fragment.
