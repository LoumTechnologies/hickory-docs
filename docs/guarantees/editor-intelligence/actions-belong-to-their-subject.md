# Actions belong to their subject

Given a working surface, when controls and results are shown, then cell
actions stay on the action rail, tab actions and notices stay inside that tab,
and workspace notices stay at workspace scope. Document and file conversion,
review, baseline, and local-view management actions use More actions; everyday
Run, Test, Debug, and explicit literate Save remain directly accessible.
Lineage settings use the same Ribbons and Braces names locally and in Settings.

---

Last LLM implementation review:
- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified: inspected implementation and executed checks support the
  contract; human usability and native OS behavior remain unverified.
- Originating requirement: implement all seven findings from the product
  coherence review, authorized in the 2026-10-08 conversation.
- Interpretation and assumptions: `docs/specs/freeform/coherent-working-surfaces.md`.
- Evidence: apps/web/src/lib/railActions.ts; apps/web/src/components/MoreActions.tsx; apps/web/src/views/workspaceTabs.tsx; apps/web/src/components/PlainFilePane.tsx; apps/web/src/views/RepresentationPane.tsx; apps/web/src/views/WorkspaceView.tsx.
- Test coverage: apps/web/src/views/workspaceTabs.test.tsx; apps/web/src/lib/railActions.test.ts; apps/web/e2e/literate-editor.spec.ts.
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
