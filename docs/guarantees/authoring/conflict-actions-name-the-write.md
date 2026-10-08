# Conflict actions name the write

Given a diverged file, when its choices are shown, then each button names
its consequence and the visible explanation says which bytes will be written
or discarded. Leaving disk unchanged dismisses a warning without claiming that
divergence is resolved. Regeneration is offered only when an output exists;
otherwise the surface explains how to obtain one.

---

Last LLM implementation review:
- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified: inspected implementation and executed checks support the
  contract; human usability and native OS behavior remain unverified.
- Originating requirement: implement all seven findings from the product
  coherence review, authorized in the 2026-10-08 conversation.
- Interpretation and assumptions: `docs/specs/freeform/coherent-working-surfaces.md`.
- Evidence: apps/web/src/components/DivergedBanner.tsx; apps/web/src/components/PlainFilePane.tsx; apps/web/src/shell/views.tsx.
- Test coverage: apps/web/src/components/DivergedBanner.test.tsx; apps/web/src/lib/plainFileSave.test.ts.
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
