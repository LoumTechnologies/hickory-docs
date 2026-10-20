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

## Continuation review (2026-10-12)

`apps/web/src/elements/patch/Editor.tsx` now calls WASM regeneration and reverse
editing for authored patch comparisons. Its source snapshot guard rejects a
stale editor before publication. Cross-document candidate edits use the native
source-publication guard. This supersedes the pending editor wiring statement
above; executable patch IDs remain required for this editing surface.

The focused comparison/WASM recipe and the portable early-edit/downstream-refusal
tests passed. Synthetic or ambiguous origins remain explicit refusals. Complete
visual automation of all patch publication gestures is not claimed. Current
executed evidence and dependency hashes are in `comparison-integration-evidence.md`;
human judgment is not recorded.

## Direct editing review (2026-10-12)

Patch code opens side by side, with Old read-only and New directly editable; after a typing pause, regeneration
checks the entire closure before changing the source. Old remains read-only by
default and immutable owning documents disable code edits. A rejected edit stays
in the mounted editor as an explicitly unsaved draft; correcting it retries the
check. Drafts are not a durable record and this review does not establish their
recovery after closing the document or restarting the app.

`apps/web/src/elements/patch/Editor.test.tsx` exercises direct editing, Unicode
publication, selection/editor preservation, downstream refusal and correction,
and read-only Old and owning-document policies. The comparison recipe passed
26 tests with type checking. Native visual behavior is not claimed by these
tests.

## Captured final drafts (2026-10-17)

`PatchEditor` now supplies the captured input tree to regeneration and candidate
evaluation instead of assuming an empty input. A captured draft callback receives
the successfully regenerated closure without dispatching any destination edit.
The owning accepted document stays read-only; only New code can be revised.
`ChangesetDocument.test.tsx` covers frozen-input regeneration, immutable original
source, and a later patch refusing an incompatible early edit. Unchecked edits
disable explicit application and lose native project context until construction
succeeds. See the final-revision guarantee for publication and recovery limits.

## Shared UI continuation — 2026-10-20

The additive shared-controller and captured-owner behavior, focused check results,
source fingerprints and remaining gaps are recorded in
[the scoped reading-surface contract](../agent/scoped-reading-surfaces-share-native-actions.md).
Earlier verification above is historical; it does not establish current universal
UI parity or native/provider/release acceptance.
