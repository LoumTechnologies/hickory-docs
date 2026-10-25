<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/editor-intelligence/actions-belong-to-their-subject.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `d2ae4d67ba9ec1bd5d1e3f5f7152036e6e6a20d751df7044cf112a6a583e46eb`.

<book:entry path="docs/guarantees/editor-intelligence/actions-belong-to-their-subject.md">
# Actions belong to their subject

Given a working surface, when controls and results are shown, then cell
actions stay on the action rail, tab actions and notices stay inside that tab,
and workspace notices stay at workspace scope. Document and file conversion,
review, baseline, and local-view management actions use More actions.
Document-wide Run and Test and file-wide Debug use the pane header’s action
menu; cell Run stays on its rail. Explicit literate Save remains directly
accessible. View identity and write destination use the header’s information
control rather than a separate document row.
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


Amendment (2026-10-13): the user authorized the minimalist editor audit’s
recommendations, then explicitly retained the visible ruler and downward action
rail collision handling. `PaneHeader` portals metadata and document/file action
controls into each spatial pane’s existing header without moving the editor.
`workspaceTabs.test.tsx` checks Run/Test menu dispatch and in-flight disabling.
Earlier verification above is historical; current evidence and coverage gaps
are in `authoring/the-editor-keeps-its-space.md`.
</book:entry>

</book:doc>
