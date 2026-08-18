# Adoption Is Byte-Exact, Or It Does Not Happen

Given a plain text file in the open folder, when it is adopted into a
literate document — `hick adopt`, `POST /api/adopt`, or the plain-file
pane's "Make literate" button — then the new (or extended) document weaves
that file byte-for-byte, verified by an actual weave BEFORE anything lands
in the working tree; and when the round-trip cannot be byte-exact, then the
adoption is refused with the reason and nothing is changed.

Adoption transfers ownership, never content: after it, the file is a
generated file with full lineage, and `git status` shows one new `.hick`
file (or one extended one) and zero changes to the file itself. That is the
entire promise — it is what makes adoption a safe first step in a
repository that has never heard of literate programming.

Three properties hold it up:

1. **Verified, not assumed.** The wrapped document is woven and its output
   compared to the original bytes before the working tree changes. A new
   document is proven in a scratch directory the up-loop cannot see; a
   block appended to an existing document is proven in place with an
   explicit restore of both the document and the file on any failure.
2. **The no-escaping invariant is reported, not worked around.** File
   content goes into the `<hick:file>` block raw. Text the parser reads as
   structure (`hick:`-prefixed tags) makes byte-exactness impossible, and
   the refusal says so and names the first differing line — there is no
   escape mechanism, by language design.
3. **Appending widens the obligation.** `--into` must additionally leave
   every output the document already produced byte-identical, and refuses a
   path the document already generates. The document's **own woven markdown**
   is the single exemption, and is not a loophole: that file is the rendering
   of the document just appended to, so it must change — a document that grew
   a block and rendered identically would mean the block never took effect.
   The bytes this guarantee is about are the adopted file's.

## Boundary

Binary files and `.hick` documents are not adoptable — the first has no
text to wrap, the second already is a document. Reversal is git, or
deleting the document while keeping the woven file.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Fable 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-cli/src/adopt.rs` — `adopt_new` (scratch-dir
  verification via `woven_output` + `verify_bytes`, refusals for binaries,
  documents, and a taken `<stem>.hick` name), `adopt_into` (baseline weave,
  append before `</hick:doc>`, restore closure on every failure path, the
  already-produced-path refusal, the every-other-output-unchanged sweep).
  CLI: `cmd_adopt` in `crates/hickory-cli/src/main.rs`. Server:
  `crates/hickory-cli/src/serve/refactor.rs::adopt` (root-bounded paths,
  index registration, `output_path` in the response). UI:
  `apps/web/src/components/PlainFilePane.tsx` ("Make literate", disabled
  while a save is in flight or conflicted) and
  `apps/web/src/views/workspaceState.ts::adoptPlainFileTab` (in-place tab
  conversion to a generated tab keyed by the weave's output path).
- Test coverage: `crates/hickory-cli/src/adopt.rs::tests` — byte-identical
  round-trip, no-trailing-newline round-trip, `hick:`-tag refusal leaving
  no file behind, binary/document/taken-name refusals, append-preserves-
  other-outputs, restore-on-failure for both sides, already-produced-path
  refusal. `apps/web/src/views/workspaceState.test.ts` covers the tab
  surgery.
- Caveat requiring review: no test drives the app button end-to-end through
  the HTTP route; the route is a thin composition of the tested module plus
  the path bounds check.

Amended 2026-08-18 (bare documents): every document now weaves a markdown file
of its own name (`docs/specs/freeform/bare-documents.md`), which put a file
inside the every-other-output-unchanged sweep that adoption itself necessarily
changes.
`adopt_into` in `crates/hickory-cli/src/adopt.rs` now skips
`before.doc.weave_path` in that sweep, and `run_doc` in
`crates/hickory-cli/src/lib.rs` parses through `hick_lang::parse_from_path` so
a `DocRun`'s `doc.weave_path` agrees with the files it produced — they
disagreed, which is how the sweep came to trip on a file it had just written.
Covered by the existing `adopt_into_appends_and_leaves_every_other_output_alone`
test, which failed on exactly this before the fix.
