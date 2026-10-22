# Selected reading scope remains separate from complete edit state

Given a workspace conversation, when the user selects documents in **Next context
projection**, the next capture includes their expanded source text or explicitly
pinned full text, independently of scrolling, focus and visibility. Unselected
open files contribute identities and edit snapshots, not model-visible text.
Fold state is retained across editor unmount/remount for the same source bytes.
An unavailable or changed fold state omits text until the editor is reopened or pinned.
Collapsing changes future captures; it never erases historical model exposure.

Segments retain the SHA-256 of the complete captured unsaved source, original
UTF-8 byte offsets, 1-based source lines and explicit omitted ranges. Duplicate
passages remain separate coordinates. The preview shows the exact candidate JSON
and a labelled bytes/4 token heuristic. Send recaptures; a preview is neither a
receipt nor the entire request. Python's existing manifest records actual model
requests, including history, instructions and tools.

Complete buffers remain separate for concurrency and edit capture. Python strips
full buffer contents and duplicate observations when constructing model input;
ACP descriptions apply the same boundary. Session context ribbons link only
selected non-omitted source lines; complete edit snapshots do not imply exposure. Omitted text cannot be interpreted as
an edit, deletion or whole-file replacement. Legacy clients without a projection
retain their prior behavior.

Explicitly expanded or pinned observations use the UI's shared typed reading records,
including evidence, source revision, run/stop/frame/invocation, source precision,
payload truncation fields and unsupported coverage. Captured, historical and
changed states remain labelled. Capturing evidence alone does not select it.
Generated annotations are evidence, never authored source rows or action offsets.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Origin: issue #52 and parent #47; `live-agent-context.md`.
- Implementation: `WorkspaceChat.editorContext`, `contextProjection.projectDocument`,
  `core.reading_context`, `agent_context::model_projection`, existing
  `observations.observationContext`, `ObservationReading`, retained editor-state
  registrations and `session_receipts::editor_context`.
- Assumptions: selecting a document opts its expanded source into the next capture;
  pin means full text, not a frozen revision. No selection is the default. Scope
  is conversation-component state; reopening the application resets selection.
- Witnesses: `contextProjection.test.ts`, `WorkspaceChat.test.tsx`,
  `ObservationReading.test.tsx`, `test_reading_projection.py`, Rust
  `projection_keeps_edit_buffers_out_of_model_description`.
- Challenges: Unicode, duplicate passages, folded bodies, unavailable editor
  state, inactive documents, fresh unsaved edits, explicit empty scope and pins.
- Coverage boundary: no real provider or native desktop interaction observed.
  External adapter internals remain opaque. Full captures are ancestry inputs,
  not claims of full model exposure; projection JSON derivation currently has
  opaque correspondence in the existing lineage graph. No automatic refresh
  schedule is specified. Log/profile provider integration belongs to sibling
  issues; this serializer preserves their fields when supplied as shared records.
  Closed-document pins and persistence across application restart are unsupported.
- Dependencies: source files above in the implementing commit; changes invalidate
  this review. Existing edit tools still require a complete read before editing.
- Executed checks (2026-10-18, implementing worktree): 18 focused Vitest
  behavior/safety tests passed across four files; web TypeScript check passed;
  three locked Python projection/request tests passed; focused Rust model-input
  boundary and projected-receipt tests passed (one each). `git diff --check`
  passed. Broad regression checks and post-commit status monitoring are left to
  the user. See issue handoff for exact commands.
- Human judgment: not recorded.


## 2026-10-22 inline runtime disclosure review

LLM review: partial. `debug/cmRuntime.ts` now shares actual widget disclosure,
context selection and log-navigation controllers with installed language actions.
Its body reader requires current explicit expansion and pages at most 8192 UTF-8
bytes. Source replacement destroys selection and stale native/agent callbacks.
Replacement widgets begin collapsed; previous model exposure remains historical.
Seven focused inline/ObservationReading cases and changed-entrypoint TypeScript
passed. The issue-72 handoff records exact hashes and exclusions. No human
judgment or browser/native scrolling witness supplied.
