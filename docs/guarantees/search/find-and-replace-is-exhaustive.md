# Find And Replace Is Exhaustive, And Refuses What It Would Undo

Given a pattern, when the folder is searched for it, then **every** match is
reported, in path order, or the answer says it stopped and why; and when those
matches are replaced, then exactly the matches that were shown are rewritten,
except in files a document generates, which are refused by name.

## Why this is not `/api/search`

The project search is **ranked** — BM25 plus embeddings, top-k. That is the
right shape for "where is the thing about invoices" and the wrong shape for
replace, because a ranked answer is a *sample*. Replacing across a sample
changes some of the occurrences and leaves the rest, which is the worst
outcome a rename can have: the code still compiles, and it is now wrong in
places nobody looked.

So there are two things, and they stay two things. The overlay (⌘⇧F) answers a
question and gets out of the way. This one is a list at the top of the tree
that you work through while the tree stays where it is.

## The rules

1. **One pattern, built once.** Literal-vs-regex, case, and whole-word are all
   changes to the *pattern* — `regex::escape`, a `(?i)` prefix, `\b…\b` —
   never branches in the matching loop. Find and replace therefore cannot
   match differently, which would be the worst possible bug here: a preview
   that does not describe the write.
2. **A generated file is shown and refused.** Its matches are real and worth
   reading, so they are listed. The *write* is refused, because the next weave
   would undo it — or the up-loop would fight it — and the refusal names the
   document, which is where the change survives *and* where it fixes every
   other copy at once. `generated_by` is on the find result too, so the UI can
   grey the row out before anyone presses the button.
3. **A `.hick` document is not refused.** It is source. A rename inside one is
   exactly the edit somebody means, and it is the edit that propagates.
4. **Limits are reported, never silent.** Files over 4MB are skipped, non-UTF-8
   files are skipped, and the match cap is 5,000 — past which `truncated` is
   true and the UI says "and more; narrow the pattern". A capped list that
   claimed to be complete would be a replace nobody could trust.
5. **Nothing is written until Replace is pressed**, and only to the paths the
   caller sent.

## Boundary

Which files a document generates is read from its **declarations** — the
`weave` target and each `hick:file` path — rather than by weaving it. Weaving
every document to answer a search would be far too slow. The honest cost is
that a path built from a variable is not recognised as generated, and such a
file is treated like any other; that is a narrowing of the refusal, not of the
search.

Zero-width matches (`^`, `\b` alone) are skipped rather than reported: they
would name every position on a line and replace nothing.

This says nothing about undo. A replace is a write to disk; the way back is
git, or the draft store if the file was open with unsaved changes. A
multi-file undo inside the app is not implemented.

The in-file find and replace is CodeMirror's own panel, opened at the top of
the editor in every pane. It is not this: it is scoped to one buffer, it
operates on the live text rather than on disk, and it is complete and
keyboard-correct already.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-cli/src/serve/find.rs` — `matcher()` (every option
  is a pattern change, shared by both routes), `line_hits()` (the zero-width
  skip and the match budget), `walk_text_files()` (the same walker
  configuration the tree and the search index use, so all three agree on
  which files exist), `find()` and `replace()` including the `generated`
  refusal with its `document` field.
  `crates/hickory-cli/src/serve/api.rs` — `generated_outputs()` /
  `outputs_of()` / `declared_outputs()`, shared with the folder tree so "is
  this generated" has one answer.
  `apps/web/src/api/client.ts` — `find` / `replaceAll`.
  `apps/web/src/shell/TreeFindReplace.tsx` — the panel; generated rows
  listed, badged, and excluded from the replace set.
  `apps/web/src/lib/revealLine.ts` — how a hit opens its file at its line
  when the pane that will answer does not exist yet.
- Test coverage: `crates/hickory-cli/tests/serve_find.rs` (9 tests) — path
  order and exhaustiveness, literal-is-not-regex, case default and override,
  the bad-regex message, replace counts and rewrites, the generated refusal
  (asserting the file is untouched AND that the document was rewritten
  instead), the find-time `generated_by` marking, `paths` narrowing, and the
  empty-pattern refusal.
  `apps/web/src/lib/revealLine.test.ts` (6 tests) — the remembered request,
  single consumption, newest-wins, clamping, and the listener.
- Caveat requiring review: the panel itself has no component test — the
  matching, the refusal, and the navigation channel are each covered, but
  "type a pattern, tick files, press Replace" was exercised by hand. The
  match cap and the 4MB file cap are asserted only by reading the code; no
  test builds a folder large enough to hit either.
