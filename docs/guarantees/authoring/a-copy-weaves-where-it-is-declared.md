# A Copy Weaves Where It Is Declared

Given a `<hick:copy>` block, when the document is woven, then its content
appears in the woven markdown at the point the block stands — as a fenced
code block in the language its `language` attribute names, or as ordinary
prose when it names none — while a `<hick:cut>` block still contributes
nothing. Nested tags inside a copy (a `<hick:paste>`, a `<hick:val>`)
resolve, and every piece keeps the source span it came from, so a ribbon
still points at it and an edit made in the woven markdown still lands back
in the `.hick`.

## Why

`docs/hick-guide.hick` §5 has always said what the pair means:

> `copy` — stores content and **keeps it** in the output;
> `cut` — stores content but **removes it** from the output.

The weave did not. Both names sat in `SILENT_DECLARATION_TAGS`
(`crates/hick-literate/src/weave.rs`), so a copy contributed zero bytes and
the two tags were one tag with two names. The visible cost was in this
repository: `docs/todo-app/implementation.md` weaves `## Storage` and then
nothing at all, because the storage code lives in a `hick:copy` that the
prose introduces and a `hick:file` further down assembles.

It is the shape of a whole document that this blocks. Turning a large
codebase into a literate `.hick` puts every function in a `hick:copy` and
assembles the files from pastes — that is the point of literate
programming, and the reason the tangled file can stay byte-identical to the
original. Woven, such a document was prose about code nobody could read.

## The two renderings

Chosen by whether the author said what language the fragment is:

- **`language="rust"`** → a fenced block, exactly the way `hick:file`
  weaves one but with **no `### path` heading**, because a fragment has no
  path. It goes through the same `process_file_children_to_weave`, so the
  fence is clean by the same rules a file's is: the line break that ends
  the open tag's line is dropped (`strip_opening_break`), a leading BOM
  with it
  (`docs/guarantees/verification/a-woven-fenced-block-has-no-leading-bom.md`),
  the body is dedented by the tag's own source column, and each text piece
  is added as a `SpanNode` carrying its `SourceOrigin::Literal`.
- **no `language`** → the children weave as ordinary prose, through
  `process_weave_content`, the same path `hick:claim` uses for its own —
  so a copy can hold a paragraph the document quotes elsewhere without
  turning it into code.

`language` is declared on the element (`CopyElement` in
`crates/hick-literate/src/render.rs`), beside `id` and `class`, because an
attribute the weave reads and the registry does not name is the
string-matched vocabulary `hick-blocks` exists to end. The element renders
no block and descends into nothing, which is what it did before it was
declared: a copy's body is document text in the app, and descending would
draw it twice.

## What is deliberately unchanged

- **`hick:cut` still weaves nothing.** It stays in
  `SILENT_DECLARATION_TAGS`, and the blank-line collapsing around it is
  untouched (`a-silent-tag-does-not-double-a-blank-line.md`).
- **What a paste produces.** A `<hick:paste select="#x">` inlines the
  fragment's own bytes, break after the open tag included. Only the fence's
  *display* drops that break, exactly as a `hick:file` body's does. So a
  document that both shows a fragment and tangles it gets a clean fence and
  byte-exact output from the same source.
- **The block model the app draws.** `CopyElement::render` returns `None`.

---

Last LLM verification:
- Date: 2026-09-06
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: the `"copy"` arm of `process_weave_tag` in
  `crates/hick-literate/src/weave.rs`, and `SILENT_DECLARATION_TAGS` in the
  same file now six names rather than seven. `CopyElement` in
  `crates/hick-literate/src/render.rs` declares `id`, `class` and
  `language`; `the_registry_describes_the_vocabulary_it_draws` and
  `the_elements_route_lists_the_vocabulary_the_render_route_draws`
  (`crates/hickory-cli/tests/block_actions.rs`) both now list `copy` first.
  Every `.hick` document in this repository was re-woven and its `.md`
  committed in the same change; `docs/todo-app/implementation.md` now shows
  the Python under each heading, which is what surfaced the defect.
- Test coverage:
  `crates/hickory-cli/tests/copy_weaves_where_it_is_declared.rs`, driving
  the real binary — a copy with a language weaves a clean fence and no
  heading; a copy without one weaves prose and no fence; a cut still weaves
  nothing and still collapses its blank lines; a `hick:paste` inside a copy
  resolves in the fence; and a copy that is both shown and pasted into a
  `hick:file` appears twice in the markdown while the tangled file gets the
  fragment's bytes exactly once.
- Caveat requiring LLM review: the woven fence's leading-newline rule and
  the paste's are deliberately different (see "What is deliberately
  unchanged"). An author who wants the tangled bytes to have no leading
  newline must open the fragment on the same line as the tag; nothing warns
  when they do not, and no test can distinguish that from an intended blank
  first line.
