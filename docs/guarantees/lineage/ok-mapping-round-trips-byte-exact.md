# A Mappable Output Edit Round-Trips Byte-For-Byte; Everything Else Is Rejected With A Reason

Given a generated output file with provenance, when an edit at ANY character
position is submitted, then either `map_edits` returns source edits that —
applied to the source documents and re-woven — reproduce the edited output
byte-for-byte, or it rejects the edit with a provable reason: the range
touches synthetic bytes (separators, derived values, provenance gaps), or its
source bytes are woven into more than one place (a copy pasted twice, where
editing one occurrence cannot be reproduced exactly).

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `crates/hickory-lineage/src/lib.rs` — `map_edits` requires full
  editable coverage (`SyntheticOverlap` otherwise), contiguous single-document
  source spans, and rejects duplicated source spans via
  `reject_if_duplicated` (`Conflict`); boundary insertions attach to an
  editable neighbor. The sweep found and forced two fixes: silent
  double-application on duplicated pastes, and boundary-insertion rejection
  asymmetry.
- Test coverage: `crates/hickory-cli/tests/lineage_sweep.rs`
  (`every_position_edit_round_trips_or_is_provably_rejected`) — for every
  char boundary of a real woven output (multi-byte UTF-8, id paste, class
  paste with synthetic separator, a copy pasted twice), both an insertion and
  a one-char replacement are mapped, applied, re-woven, and compared
  byte-for-byte, with every rejection cross-checked against provenance.
  Plus `crates/hickory-lineage` unit tests and the server round-trip
  integration test (`output_edit_round_trips_byte_for_byte`).
