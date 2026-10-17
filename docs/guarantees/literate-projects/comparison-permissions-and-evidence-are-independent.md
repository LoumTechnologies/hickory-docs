# Comparison edit permissions do not establish replay evidence

Given an N-source project comparison, when Old and New endpoints are selected,
then their identities, writable destinations and edit policy are separate from
replay verification. All four policies (neither, old, new, both) are supported
by the model. An immutable source without a writable fork cannot become editable.
An accepted endpoint edit clears equality evidence and leaves the opposite
endpoint unchanged. Destination trees must still equal the captured identity
before a host applies reviewed changes.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified for the portable comparison service, not a claim of full editor UI.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, shared
  comparison foundation, independent permissions and destination guards.
- Evidence: `hick-lang/src/project/comparison.rs::ProjectComparison`; evidence
  and source fields are private to prevent bypassing invalidation through mutation.
- Witnesses and challenges: `hick-lang/tests/project.rs` exercises all four
  policies, immutable Git refusal, replay evidence and changed destinations.
- Assumptions and exclusions: provider diff import alone is not replay. Existing
  review/commit views still default to immutable two-sided readings; fully shared
  editable N-target UI and per-target undo integration are pending.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: comparison and tests above; edits invalidate this review.
- Human judgment: not recorded.

## Continuation review (2026-10-12)

The common `ComparisonEditor` now presents inline/side-by-side/patch readings,
forks immutable readings, and invalidates evidence on typing through
`projectComparison.ts`. The model keeps all four policies; native immutable
identities do not become writable by receiving a label or target path.
`just test-literate-comparison-editor` passed all policy and integration witnesses.

Merged-view comparison membership and writable targets are separate. The current
`merged_write.rs` checks exact displayed bytes and reports each target. The
real-worktree tests passed shared/single-target writes and guarded undo. Neither
per-target success nor a rendered provider diff establishes replay equality.
Current evidence and hashes are in `comparison-integration-evidence.md`;
human judgment is not recorded.


## Default presentation (2026-10-16)

Comparisons open side by side. Both actual buffers remain separate: changed or
removed lines on Old are highlighted as deletions and changed or added lines on
New as insertions. Editing recomputes the highlights, without inserting historical
rows or changing saved text. Inline and patch layouts remain explicit options.

`ComparisonEditor.test.tsx` checks the default, both highlight colors' semantic
classes, unchanged-line exclusion, live update, layout switching and independent
edit permissions. Patch-editor tests continue to check that editing New preserves
the Old bytes, editor identity, selection and downstream construction checks.
