<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/a-link-between-documents-lands-in-the-weave.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `a814dad31b6d9847d43e02b3e9a67e7ccd012837da2d4160143461fb64ae648c`.

<book:entry path="docs/guarantees/authoring/a-link-between-documents-lands-in-the-weave.md">
# A Link Between Documents Lands In The Weave

Given a `.md` document containing an ordinary markdown link to another
document in the folder — `[the plan](plan.md)` — when the document is woven,
then the woven markdown carries `[the plan](plan.md)` byte for byte. A
document is authored as `.md` and weaves the same `.md`, so a link to another
document already points at the file the reader has; there is nothing to
rewrite. When the caret is on that link in the editor, Mod-click opens the
linked document as a tab.

## Why markdown and not a tag

A `.md` document is markdown first (`bare-documents.md`), and a link is the
one piece of markup every reader already understands. A `<hick:link>` would
have to weave to `[…](…)` anyway and would be unreadable in the source in the
meantime. So a cross-document link is written the way anyone would write it,
and the weaver leaves it alone.

## The rules

1. **The destination is untouched.** The extension, the fragment, the title,
   the label, the brackets and every byte of prose around them are the
   author's and keep the author's origin — which is what draws a ribbon from
   the sentence in the source to the sentence in the weave.
2. **A relative destination resolves against the LINKING document's own
   directory**, both in the weave and when the editor opens it. That is what a
   relative link means everywhere else, and it is what keeps a folder of notes
   movable.
3. **The editor and the weaver agree.** The editor's `resolveTarget`
   (`apps/web/src/lib/mdLinks.ts`) and the weave's own path handling both
   treat the destination as a path beside the linking file.

## Boundary

Nothing verifies that the destination EXISTS. A link to a document that has
not been written yet is a normal thing to have in notes, and a weave that
failed on it would make the tool unusable for the way notes are actually
written. The editor's own answer to a missing target is that Mod-click opens
nothing and the folder tree is still the way to it.

A link that spans a newline, or whose parentheses never close, is a typo
rather than a link, and is passed through untouched.

---

Last LLM verification:
- Date: 2026-10-10
- Reviewer: Claude (Opus 5)
- Result: reviewed after the `.hick` → `.md` migration; the rewrite this
  guarantee used to describe was removed because it became an identity.
- Note: before the migration a `.hick` source wove a `.md` of its own name, so
  `woven_target` rewrote a `.hick` destination to `.md`. A document is now
  authored as `.md` and weaves the same `.md`, so
  `crates/hick-literate/src/links.rs::woven_target` returns the destination
  unchanged (and `split_one` skips the identity rather than marking unchanged
  bytes synthetic). `apps/web/src/lib/mdLinks.ts::wovenTarget` is likewise an
  identity. Rewriting a link the reader already has was the cost of a
  two-extension past; it is not paid any more.
- Test coverage: `crates/hick-literate/tests/document_links.rs` — the link,
  its fragment, a URL and the surrounding prose all surviving `run_pipeline`
  byte for byte with their source origin.
  `crates/hick-literate/src/links.rs` unit tests — the unchanged-destination
  cases. `apps/web/src/lib/mdLinks.test.ts` — the editor side.
</book:entry>

</book:doc>
