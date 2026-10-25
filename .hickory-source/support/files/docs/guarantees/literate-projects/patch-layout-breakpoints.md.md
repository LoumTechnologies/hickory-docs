<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/literate-projects/patch-layout-breakpoints.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `e84399b59238cecf19bde3bfe3794ddde08a21cb847b36c09ff6dc2d24ae9518`.

<book:entry path="docs/guarantees/literate-projects/patch-layout-breakpoints.md">
# Patch layout exposes endpoint breakpoints

Given a comparison with a debug-capable captured project whose displayed file
matches its complete endpoint, when Patch layout is selected, then its code
rows offer breakpoint toggles through the same debug session as the other
layouts. Removed lines address the old endpoint, added lines address the new
endpoint, and context lines offer both. Hunk headers and metadata offer none.
Line numbers come from the unified diff's hunk coordinates. Held breakpoints
show a selected toggle that removes them when clicked.

---

Last LLM verification:

- Date: 2026-10-17
- Reviewer: Codex
- Result: verified by component tests and TypeScript checks.
- Evidence: `ComparisonEditor.tsx` connects `ComparisonTools.tsx` endpoint
  sessions to `DiffView.tsx` hunk coordinates and accessible toggles.
- Test: `ComparisonTools.test.tsx`, “sets and removes patch-layout breakpoints
  at the correct endpoint line”.
- Executed: `just test-literate-comparison-editor` (34 tests passed).
- Limits: native adapter execution and visual appearance were not exercised.
  A local intermediate file that differs from its complete endpoint retains
  the existing restriction on direct breakpoint placement.
</book:entry>

</book:doc>
