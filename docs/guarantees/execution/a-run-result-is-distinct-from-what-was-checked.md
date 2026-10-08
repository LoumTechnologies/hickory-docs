# A run result is distinct from what was checked

Given a cell, diagram, or source-backed AI explanation, when status is
rendered, then successful execution does not alone claim verified output or
semantic correctness. Matching expected stdout is reported separately. Diagrams
report passed, failed, or not-yet-verified assertions; stale or absent execution
evidence cannot report passing assertions. Stale explanations identify source
change separately from execution verdicts. The action rail distinguishes stale
recordings from cells that have never run.

---

Last LLM implementation review:
- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified: inspected implementation and executed checks support the
  contract; human usability and native OS behavior remain unverified.
- Originating requirement: implement all seven findings from the product
  coherence review, authorized in the 2026-10-08 conversation.
- Interpretation and assumptions: `docs/specs/freeform/coherent-working-surfaces.md`.
- Evidence: apps/web/src/lib/executionEvidence.ts; apps/web/src/components/StatusChip.tsx; apps/web/src/components/CellPanel.tsx; apps/web/src/components/DiagramPanel.tsx; apps/web/src/editor/CardRail.tsx; apps/web/src/editor/DocumentEditor.tsx.
- Test coverage: apps/web/src/lib/executionEvidence.test.ts; apps/web/src/components/CellPanel.test.tsx; apps/web/src/components/DiagramPanel.test.tsx; apps/web/src/editor/DocumentEditor.test.tsx.
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
