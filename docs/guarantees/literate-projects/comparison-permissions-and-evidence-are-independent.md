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
