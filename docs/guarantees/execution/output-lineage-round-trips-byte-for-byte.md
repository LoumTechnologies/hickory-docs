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
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `crates/hickory-lineage/src/lib.rs` (`from_provenance_map` only
  emits a source span when the span's byte length equals the output range —
  everything else degrades to `synthetic`; `map_edits` requires gap-free
  editable coverage, source contiguity, and non-overlapping source targets);
  `apps/server/src/routes/outputs.rs` re-verifies before applying that every
  span-carrying provenance range still matches the current doc source bytes
  (409 on staleness) and that the edited doc still parses (422 otherwise);
  provenance is persisted per run in `run_outputs`
  (`apps/server/migrations/0002_run_outputs.sql`,
  `apps/server/src/runs.rs::store_run_outputs`) so reads never re-execute.
- Test coverage: `apps/server/tests/integration.rs` —
  `output_edit_round_trips_byte_for_byte` (edit → source update → git
  commit message → re-run → byte-for-byte equality),
  `outputs_listing_and_byte_precise_provenance` (gap-free coverage, spans
  map to exact source bytes, visibility),
  `output_edit_overlapping_synthetic_separator_is_422`;
  `crates/hickory-lineage/src/lib.rs` unit tests;
  `crates/hick-literate/tests/pipeline_tests.rs::test_copy_paste_provenance_carries_source_spans`.
