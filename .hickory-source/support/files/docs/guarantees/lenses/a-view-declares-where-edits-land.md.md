<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/lenses/a-view-declares-where-edits-land.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `864e29be0eeef68f8b3d0aee95080ba91a0063f0ed2e3831d370c9473183af67`.

<book:entry path="docs/guarantees/lenses/a-view-declares-where-edits-land.md">
# A view declares where edits land

Given a document, plain or generated file, literate view, change review,
commit reading, conversation history, worktree comparison, or history story,
when the working surface is shown, then its identity, editability, and write
destination appear together. Historical comparisons disable writes. Generated
regions retain their existing protection; declaring an editable surface does
not make every byte editable or grant a new engine capability.

---

Last LLM implementation review:
- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified: inspected implementation and executed checks support the
  contract; human usability and native OS behavior remain unverified.
- Originating requirement: implement all seven findings from the product
  coherence review, authorized in the 2026-10-08 conversation.
- Interpretation and assumptions: `docs/specs/freeform/coherent-working-surfaces.md`.
- Evidence: apps/web/src/components/SurfaceInfo.tsx; apps/web/src/views/workspaceTabs.tsx; apps/web/src/views/RepresentationPane.tsx; apps/web/src/views/DocumentReading.tsx; apps/web/src/views/CommitReading.tsx; apps/web/src/views/SessionLens.tsx; apps/web/src/views/MergedView.tsx; apps/web/src/views/HistoryLens.tsx.
- Test coverage: apps/web/src/views/RepresentationPane.test.tsx; apps/web/e2e/literate-editor.spec.ts.
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


## 2026-10-22 early pane status review

LLM review: partial. Load/error/restoration metadata remains explicitly readable
when a document/generated/literate editor cannot mount. Files-pane and changeset
rendering failures have bounded readers; source/recovery/project bytes are
excluded. Eighty-one focused cases and changed-entrypoint TypeScript passed,
with three selected final loading/failure checks. Exact dependencies and limits
are in the issue-72 handoff. No human judgment or current native/provider
completion witness supplied.
</book:entry>

</book:doc>
