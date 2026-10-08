# Saved means the displayed bytes reached the file

Given an edited named document, ordinary file, generated-file buffer, or
literate view, when its save succeeds, then only the acknowledged text becomes
clean. An earlier save response cannot mark later typing saved. Rendering
failure does not invalidate acknowledged document persistence. A failed or
conflicted write stays dirty and closing waits for a successful Save or an
explicit discard/retention choice.

Recovery drafts do not count as file saves. Untitled stays unnamed until Save.
Literate and generated views participate in close prompts and Save All while
inactive; their prompts declare that they have no crash-recovery draft.

---

Last LLM implementation review:
- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified: inspected implementation and executed checks support the
  contract; human usability and native OS behavior remain unverified.
- Originating requirement: implement all seven findings from the product
  coherence review, authorized in the 2026-10-08 conversation.
- Interpretation and assumptions: `docs/specs/freeform/coherent-working-surfaces.md`.
- Evidence: apps/web/src/views/useDocumentPersistence.ts; apps/web/src/lib/plainFileSave.ts; apps/web/src/lib/outputSave.ts; apps/web/src/lib/viewBuffers.ts; apps/web/src/shell/views.tsx; apps/web/src/views/RepresentationPane.tsx; apps/web/src/views/documentSession.tsx; apps/web/src/views/useUnsavedLifecycle.ts.
- Test coverage: apps/web/src/views/useDocumentPersistence.test.tsx; apps/web/src/views/RepresentationPane.test.tsx; apps/web/src/views/useUnsavedLifecycle.test.tsx; apps/web/src/lib/plainFileSave.test.ts; apps/web/src/lib/outputSave.test.ts; apps/web/src/shell/views.test.tsx; apps/web/e2e/literate-editor.spec.ts.
- Witnesses and challenges: delayed replies, subsequent typing, rejected writes,
  inactive tabs, historical views, and stale evidence. Native OS usability is
  not established by automated checks.
- Dependencies: exact SHA-256 hashes are recorded in
  `docs/guarantees/audits/2026-10-08-coherent-working-surfaces.md`.
  Evidence becomes stale when those files change.
- Executed checks: `just test-web`: 197 files and 1969 tests passed;
  `just test-literate-editor`: 6 real-engine browser flows passed;
  `just check-web` and `just check-file-length`: passed. The final local-notice
  adjustment also passed all 8 `workspaceTabs.test.tsx` tests. See the audit
  for coverage gaps and earlier failed witnesses.
- Human judgment: not recorded.
