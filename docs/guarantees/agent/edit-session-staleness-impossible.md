# An Agent Edit Session Never Applies A Stale Edit

Given an agent edit session on a primary document, when any `edit_output` or
`edit_doc` tool call is made, then the edit either applies against the
document's current on-disk content or fails with a structured error — a
stale-provenance (409-class) misapplication is impossible. Concretely: the
session is single-writer and re-weaves immediately after every successful
edit (refreshing hashes and provenance before the model sees another
anchor); edits anchor on 4-hex line *content* hashes, never positions, so an
anchor can only resolve against text that is actually present; and external
mutation of the document file is detected by content comparison before every
edit and absorbed by one automatic re-weave, after which anchors that no
longer resolve produce a structured error naming the external change.

---

Last LLM verification:
- Date: 2026-08-06
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `crates/hickory-agent/src/tools/mod.rs` — `EditSession` owns the
  source and weave; `sync_with_disk` runs at the top of `edit_output`,
  `edit_doc`, and `verify`; both edit paths re-weave (or roll back) before
  returning and write the document to disk themselves; `resolve_edit`
  reports unresolvable anchors after a re-sync as a structured error.
  `crates/hickory-agent/src/tools/hashline.rs` — anchors are content
  hashes with ambiguity surfaced as candidate lists.
- Test coverage: `crates/hickory-agent/tests/tools_edit_session.rs`
  (`external_mutation_is_absorbed_or_reported_never_misapplied`: an edit
  anchored on externally-changed content succeeds via the automatic
  re-weave; an edit anchored on content that no longer exists is a
  structured error and the file is untouched), plus
  `edit_output_maps_through_lineage_and_reweaves` (fresh hashes after every
  edit) and the hashline unit tests
  (`stale_hash_is_a_structured_miss`,
  `ambiguous_run_lists_candidates_and_occurrence_disambiguates`).
