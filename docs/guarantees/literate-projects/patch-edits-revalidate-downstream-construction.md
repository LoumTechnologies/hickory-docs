# Patch regeneration revalidates downstream construction

Given an authored patch in an evaluated closure and an edited local New tree,
when patch regeneration is requested, then a candidate unified diff is produced
against that patch's exact local Old tree. The original source map remains
unchanged. The entire candidate closure is evaluated again; invalid downstream
context fails by name, and successful regeneration reproduces the requested
local New state. No model, fuzzy adaptation or filesystem write occurs.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified for the pure regeneration API.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, editable
  constructions, regeneration, invalidation and reporting early-edit failures.
- Evidence: `hick-lang/src/project/edit.rs::regenerate_patch` and
  `reconstruct.rs::unified_transition`, shared exact evaluator and patch replay.
- Witnesses and challenges: `patch_regeneration_preserves_order_and_reports_invalid_later_context`
  changes an early patch so the later hunk fails, then edits the last patch with
  a final-newline distinction and verifies the requested bytes.
- Assumptions and exclusions: requires an explicit authored patch ID and textual
  transitions. A no-op retains its dependency through an exact zero-count hunk;
  an empty-tree no-op or unrepresentable delimiter returns a named refusal.
  The inline comparison editor is not yet wired to this API.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: paths above; regeneration/evaluator edits invalidate this review.
- Human judgment: not recorded.
