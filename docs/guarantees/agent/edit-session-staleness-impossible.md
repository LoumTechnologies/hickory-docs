# An Agent Edit Session Never Applies A Stale Edit

Given an agent edit session on a primary document, when a document or upstream
changes between read and edit, then the next edit refuses and directs another
read. Successful edits re-weave before returning fresh SHA-256 line anchors.
Ambiguous anchors require an occurrence; old short anchors do not resolve.

When a whole-surface `snapshot` is supplied, it must match exactly, including a
range's interior. Disk-backed in-process sessions compare current source and
upstreams before edits. Standalone invocations have no previous in-memory read;
callers must pass a snapshot for whole-surface freshness. Endpoint hashes alone
do not protect a changed interior or the caller's mistaken choice of range.

Review rechecks the complete preview before application. This is a refusal
boundary, not an atomic filesystem lock against arbitrary concurrent writers.

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

Verification update (2026-10-08): the original “impossible” wording overstated
16-bit anchors and concurrent-writer protection. SHA-256 anchors replace them;
`resolve_edit` refuses resynchronized surfaces and mismatched supplied snapshots.
`tools_edit_session` and `model_failure_audit` are regression witnesses. This
correction preserves the intended refusal behavior and records remaining limits.
