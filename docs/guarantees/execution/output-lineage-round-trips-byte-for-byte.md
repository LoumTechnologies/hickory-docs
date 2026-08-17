# Output Lineage Edits Round-Trip Byte-for-Byte

Given a doc whose last successful run produced generated output files, when
an owner edits an output range whose provenance is byte-precise (literal
text or a pasted copy block) via `POST /api/docs/:id/outputs/edit`, then the
server maps the edit through the stored `Provenance[]` to source-document
edits, applies them (Postgres row + a git commit "lineage edit via
&lt;path&gt;"), and the next run reproduces the edited output byte-for-byte.
Edits overlapping `synthetic` ranges (separators, exec output, anything not
byte-identical to source bytes) are rejected with 422 carrying the
offending output range, and the source is left untouched.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Fable 5)
- Result: verified (local serve; the hosted server this file's guarantee text
  describes no longer exists)
- Evidence: `crates/hickory-lineage/src/lib.rs` (`from_provenance_map` only
  emits a source span when the span's byte length equals the output range —
  everything else degrades to `synthetic`; `map_edits` requires gap-free
  editable coverage, source contiguity, and non-overlapping source targets);
  `crates/hickory-cli/src/serve/api.rs::edit_outputs` weaves fresh from the
  files (and live rooms) in the same request, maps through
  `crate::output_lineage`, refuses synthetic overlaps with 422 carrying the
  offending range, and re-parses every edited document before writing —
  there is no stored run, so the hosted 409 staleness case cannot arise.
- Test coverage: `crates/hickory-cli/tests/serve_local.rs` —
  `the_ribbons_have_their_data_without_a_database` (a paste-block edit in a
  generated code file lands in the document byte-exactly) and
  `a_prose_edit_in_the_woven_markdown_lands_in_the_document` (the `weave=`
  output is listed, its prose carries non-synthetic provenance, an edit
  through POST `/outputs/edit` lands in the `.hick`, and a re-weave
  reproduces it byte-stably); `crates/hickory-lineage/src/lib.rs` unit tests;
  `crates/hick-literate/tests/pipeline_tests.rs::test_copy_paste_provenance_carries_source_spans`.
- Caveat requiring review: the guarantee text above still describes the
  hosted shape (Postgres row, git commit, "last successful run"); the local
  behavior is fresher (weave-of-this-request). Rewriting the text is a
  deliberate spec decision, not done as a side effect here.
