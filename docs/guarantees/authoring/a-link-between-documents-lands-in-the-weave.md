# A Link Between Documents Lands In The Weave

Given a `.hick` document containing an ordinary markdown link to another
document in the folder — `[the plan](plan.hick)` — when the document is woven,
then the woven markdown carries `[the plan](plan.md)`: the destination points
at the markdown that document weaves, not at the source the reader may not
have. When the caret is on that link in the editor, Mod-click opens the linked
document as a tab. Every other destination — a URL, an image, a `.csv`, a bare
`#fragment`, an absolute path — is passed through byte for byte.

## Why markdown and not a tag

A `.hick` document is markdown first (`bare-documents.md`), and a link is the
one piece of markup every reader of the woven `.md` already understands. A
`<hick:link>` would have to weave to `[…](…)` anyway and would be unreadable in
the source in the meantime. So a cross-document link is written the way anyone
would write it, and the weaver is what makes it land.

## The rules

1. **Only the destination changes, and only its extension.** The label, the
   title, the brackets, and every byte of prose around them are untouched.
   `plan.hick#risks` becomes `plan.md#risks`: the fragment goes with it,
   because a heading anchor is the same anchor in both files.
2. **A destination with a scheme is left alone.** `https:`, `mailto:`, `tel:`,
   and — deliberately — `C:` are all "somewhere else"; half-rewriting a Windows
   absolute path would be worse than not touching it.
3. **A bare `#fragment` is already correct.** It points inside the file that
   holds it, which is true in the source and true in the weave.
4. **A link inside a substituted value is left to the substitution.** Those
   bytes are already the weaver's and have no source span to keep intact, so
   the link pass runs only over passthrough text.
5. **A relative destination resolves against the LINKING document's own
   directory**, both in the weave and when the editor opens it. That is what a
   relative link means everywhere else, and it is what keeps a folder of notes
   movable.
6. **The editor and the weaver implement one rule.** `wovenTarget` in
   `apps/web/src/lib/mdLinks.ts` and `woven_target` in
   `crates/hick-literate/src/links.rs` are the same function in two languages,
   tested against the same cases. Two implementations is a real cost; the
   alternative is the editor asking the server where every link in the buffer
   points on every keystroke, which is worse.

## Boundary

Nothing verifies that the destination EXISTS. A link to a document that has
not been written yet is a normal thing to have in notes, and a weave that
failed on it would make the tool unusable for the way notes are actually
written. The editor's own answer to a missing target is that Mod-click opens
nothing and the folder tree is still the way to it.

A link that spans a newline, or whose parentheses never close, is a typo
rather than a link, and is passed through untouched by both sides.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-literate/src/links.rs` — `woven_target` (the rule),
  `has_scheme`, `destination_spans` (the bracket/paren scanner, with balanced
  parentheses and title exclusion), `split_one` (the segmenting, which is what
  keeps rule 6 of the lineage guarantee true), and
  `rewrite_document_links`. Composed into the weave's own segmenter in
  `crates/hick-literate/src/weave.rs`, after the substitution pass — the order
  is what makes rule 4 true.
  `apps/web/src/lib/mdLinks.ts` — `findLinks`, `wovenTarget`, `isLocalTarget`,
  `splitFragment`, `resolveTarget`.
  `apps/web/src/editor/mdLinks.ts` — the decoration, `linkAt`, `followLink`
  (a folder path goes out as the `hickory-open-path` event the shell already
  handles for File > Open File…; a URL goes to the browser), and the Mod-click
  handler.
- Test coverage: `crates/hick-literate/tests/document_links.rs` (4 tests) —
  the rewrite end to end through `run_pipeline`, a fragment and a URL
  surviving, and the two lineage assertions the sibling guarantee owns.
  `crates/hick-literate/src/links.rs` unit tests (10) — every `None` case, the
  title, parentheses inside a URL, the substituted-value case, and that the
  `pattern` carries what was REPLACED rather than what was produced (a slide
  there would misplace every span after the first link).
  `apps/web/src/lib/mdLinks.test.ts` (20) and
  `apps/web/src/editor/mdLinks.test.ts` (8) — the same cases on the editor
  side, plus the skip ranges, the decoration, and where a follow goes.
- Caveat requiring review: the two implementations of the rule are kept in
  step by the shared test cases and by nothing else — a divergence would show
  up as a link the editor opens and the weave points elsewhere. Worth a
  cross-language fixture if the rule ever grows a third case.
