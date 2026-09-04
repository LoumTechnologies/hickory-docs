# An Edited AI Passage Stops Claiming The Model Wrote It

Given a `hick:transform` carrying `wrote=` — a fingerprint of the passage as
the model produced it — when the passage's bytes are no longer those bytes,
then the weave says *written by a model, and edited by hand since*, and
`hick test` reports it, naming the model.

It is **not a failure**. Editing an AI-written passage is the author's
business; `hick test` still exits `Verified`. What the document may not do is
go on claiming a model wrote words a person typed.

Whitespace around the passage is not an edit: the newlines surrounding it are
the element's formatting, and re-indenting a document must not read as
somebody rewriting the model.

A transform with **no** `wrote=` claims nothing either way, and is silent. It
was written before the attribute existed, it never said whose words these
are, and inventing an answer for it would be worse than saying nothing.

## Why

`from=` pins a transform's **inputs**: the bytes it was shown and the
instruction it followed. That is what makes an LLM-written passage checkable
offline, and it is the right claim — the document has never said the prose
reproduces.

It is also only half. On 2026-09-04 the receipts walkthrough was exercised by
trying to break it, and this is what broke: with the woven `.md` absent,
replacing *"All three faults were fixed"* with *"Nobody was affected and
nothing needed fixing"* left every fingerprint matching, because the inputs
had not moved. `hick test` answered `ok`. The passage went on naming its
model and its instruction, and read exactly as it had before.

In the usual workflow the committed weave catches it as drift, which is why
this was a gap rather than a hole. But drift is the wrong instrument: it says
"the output on disk is out of date", which is neither true nor useful here,
and it says nothing at all in a repository that does not commit its weave.

The answer is the one `sessions-you-run-again.md` already gives for sessions:
three kinds must never be mistaken for each other — **run** (harness-written,
evidence), **edited** (a declared layer over a frozen base, marked *in the
bytes*), and **staged**. A transform is the same three, and only two of them
were expressible.

---

Last LLM verification:
- Date: 2026-09-04
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hick-lang/src/lib.rs` — `passage_fingerprint`, which
  trims before hashing; `crates/hickory-cli/src/main.rs` — `hick refresh`
  stamps `wrote` beside `from`, and `cmd_test` reports `EDITED`;
  `crates/hickory-cli/src/lib.rs` — `CheckFailure::EditedTransform`, whose
  `outcome()` is `Verified`, and `edited_transforms`;
  `crates/hick-handlers/src/handlers/transform.rs` — the line the weave adds.
- Test coverage: `crates/hickory-cli/tests/cites.rs` (`edited_passages`) —
  "the model's own words are not marked", "a hand edit is marked and names
  the model" (including that the outcome stays `Verified`), "whitespace
  around the passage is not an edit", "a passage written before this existed
  claims nothing"; `hick-handlers`'
  `an_edited_passage_says_so_in_the_weave` and `no_fingerprint_means_no_claim`.
  Verified by hand end to end: stamped, silent; edited, marked in both the
  report and the woven markdown; restored, silent again.
- Caveat requiring LLM review: this says *that* a passage was edited, never
  *what* changed — the model's own words are not kept, so there is nothing to
  diff against. Keeping them would mean storing a second copy of every
  passage in the document, which is a real cost for a claim that "these are
  no longer the model's words" already carries. `hick refresh` replaces an
  edit with the model's own words and says so in the report, which is the
  only way back.
