# Every Document Weaves A Markdown File Of Its Own Name

Given a `.hick` document loaded from a path, when it is run or woven, then it
writes a markdown file of its own name beside it — `notes/standup.hick` weaves
`notes/standup.md` — unless it names a different one with `weave=`, or opts out
entirely with `weave="none"`.

A document with no readable form is not a note. Before this, `weave=` was
opt-in, so the default outcome of writing a document was a document nobody
could read anywhere else; after it, the readable rendering is the thing you get
for free and silence is the unusual choice.

Three properties hold it up:

1. **The default is resolved where the path is known, and nowhere else.**
   `hick_lang::parse` leaves `weave_path` exactly as the source declared it;
   `hick_lang::parse_from_path` resolves it. Analyses, language-server buffers,
   and structural tools use the former and never invent an output file — only
   the loaders that are about to *write* outputs use the latter.
2. **The default is a file name, not a path.** `weave`, like every
   `<hick:file path=…>`, resolves relative to the document's own output
   directory, and an absolute path is refused as an escaping output path. So
   the default is `standup.md`, which lands beside `notes/standup.hick`.
3. **Every producer of a document agrees on its outputs.** The pipeline
   prologue, the re-prepare after an agent cell edits a document, and the CLI's
   `run_doc` all resolve through the same function. A `DocRun` whose
   `doc.weave_path` disagreed with the files it produced is not a cosmetic
   inconsistency: it is what made adoption's every-other-output-unchanged sweep
   trip on a file adoption had itself just written.

## Boundary

**This is a deliberate behaviour change, not a new feature.** A wrapped
document that omitted `weave=` produced no markdown before and produces one
now — a new file appearing in a repository, and a new file that `hick test`
then checks for drift. `pre-launch.md` permits it and asks that it be decided
rather than stumbled into; `docs/specs/freeform/bare-documents.md` records the
decision. `weave="none"` is the escape hatch for a document that is genuinely
only a generator of other files.

**A document whose markdown has never been generated reports drift.** `hick
test` compares committed outputs against freshly produced ones, and a `.md`
that does not exist yet differs from one that does. That is the same treatment
every generated file gets; the fix is `hick run`.

---

Last LLM verification:
- Date: 2026-08-18
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-lang/src/lib.rs` — `default_weave_path` (file stem plus
  `.md`), `WEAVE_NONE`, and `parse_from_path` (explicit wins, `none` clears,
  absent defaults). Loaders: `prepare_pipeline` and `reprepare_document` in
  `crates/hick-literate/src/lib.rs`, `run_doc` in
  `crates/hickory-cli/src/lib.rs`. Consumer:
  `crates/hick-literate/src/weave.rs::process_weave_output`.
- Test coverage: `crates/hick-lang/src/lib.rs::tests` —
  `a_document_weaves_a_markdown_file_of_its_own_name` (relative and absolute
  document paths), `an_explicit_weave_beats_the_default`,
  `weave_none_opts_out_of_the_default` (bare and wrapped),
  `parse_leaves_the_weave_path_unresolved`, `frontmatter_sets_reserved_keys`.
  `crates/hick-literate/tests/pipeline_tests.rs` —
  `test_weave_without_an_attribute_uses_the_document_name` (which replaced
  `test_weave_no_attribute_no_output`, the test that asserted the old
  contract), `test_no_file_outputs`, `test_multi_file_output`,
  `test_multi_stage_no_hick_output_single_stage`.
- Run end to end on this machine: `hick run notes/standup.hick` on a document
  declaring no `weave=` reported `wrote notes/standup.md`, and `hick test` then
  reported `ok`.
- Caveats — what LLM review could NOT establish:
  - **The blast radius on a real repository is untested.** Every existing
    document in `examples/` and in any user's tree that omits `weave=` will
    produce a new file on its next run. Nothing warns about this, and no
    migration path exists beyond running the tool.
  - **`hick equiv` now depends on both sides sharing a document name.**
    `check_equivalence` runs both sources as `document.hick` so their default
    weave paths coincide, which is semantically right — the refactor gate is
    about one file before and after — but it means the function is no longer
    meaningful for comparing documents that genuinely live at different paths.
  - **No decision has been made about whether a notes repository wants the
    pre-commit drift gate `hick init` installs.** A commit blocked because a
    meeting summary is stale is defensible for code and questionable for notes;
    named as an open edge in `notes-ide.md`.
