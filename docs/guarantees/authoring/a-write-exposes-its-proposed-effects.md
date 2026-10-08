# A write exposes its proposed effects

Given Insert, New Project, an environment command, a filesystem Dry Run,
an agent proposal, or unsaved literate-view edits, when the person inspects the
operation, then proposed bytes or effects are visible before committing the
write. A preview writes nothing. Literate previews compare the arrangement and
changed backing files; closed previews do not mount extra editors. The write
keeps its existing concurrency checks and reports the result locally.
Ordinary typing still autosaves without a confirmation for every edit.

---

Last LLM implementation review:
- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified: inspected implementation and executed checks support the
  contract; human usability and native OS behavior remain unverified.
- Originating requirement: implement all seven findings from the product
  coherence review, authorized in the 2026-10-08 conversation.
- Interpretation and assumptions: `docs/specs/freeform/coherent-working-surfaces.md`.
- Evidence: apps/web/src/components/ChangePreview.tsx; apps/web/src/components/InsertMenu.tsx; apps/web/src/components/NewProjectDialog.tsx; apps/web/src/environments/EnvironmentPanel.tsx; apps/web/src/shell/FilesystemTreeEditor.tsx; apps/web/src/views/DocumentReading.tsx; apps/web/src/views/RepresentationPane.tsx.
- Test coverage: apps/web/src/components/InsertMenu.test.tsx; apps/web/src/components/NewProjectDialog.test.tsx; apps/web/src/environments/EnvironmentPanel.test.tsx; apps/web/e2e/literate-editor.spec.ts.
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
