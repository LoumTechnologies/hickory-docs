# A failed action keeps its editor

Given a loaded document or generated file, when a Run, Test, render, or
save request fails, then its editable work remains reachable and a local notice
names the failure and offers the corresponding retry. Only a failed initial
load can replace the document editor. A failed template listing offers retry
without leaving New Project; unsupported SDK installation remains a link.

---

Last LLM implementation review:
- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified: inspected implementation and executed checks support the
  contract; human usability and native OS behavior remain unverified.
- Originating requirement: implement all seven findings from the product
  coherence review, authorized in the 2026-10-08 conversation.
- Interpretation and assumptions: `docs/specs/freeform/coherent-working-surfaces.md`.
- Evidence: apps/web/src/views/documentSession.tsx; apps/web/src/views/workspaceTabs.tsx; apps/web/src/components/ActionNotice.tsx; apps/web/src/shell/views.tsx; apps/web/src/components/NewProjectDialog.tsx.
- Test coverage: apps/web/src/views/workspaceTabs.test.tsx; apps/web/src/components/NewProjectDialog.test.tsx; apps/web/e2e/literate-editor.spec.ts.
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
