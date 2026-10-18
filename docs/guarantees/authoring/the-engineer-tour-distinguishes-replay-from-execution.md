# The engineer tour distinguishes replay from execution

Given the AI engineer tour on the homepage or in the application, the same
scenario identifies meeting evidence, a requirement, a recorded agent mistake,
a precise objection, a failing counterexample and a corrected implementation.
Agent proposals are labeled recorded examples. Browser execution uses the product
browser debugger. The counterexample step requires observed failure and the
correction step requires observed success before Continue is enabled.

The tour edits no project files. It can be closed, restarted and reopened;
the step is remembered locally. The downloadable document executes the same
JavaScript through a `hick:exec` witness with an expectation and requires Node.js.
First materialize the generated file with `hick weave`, then verify with
`hick test`. Its deterministic budget-allocation check does not establish network timing.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: a shared in-product tour from an AI engineer perspective.
- Evidence: `components/WorkflowTour.tsx`, `tour/scenario.ts`,
  `embed/DebuggableDocument.tsx`, `views/LandingView.tsx`, `views/WorkspaceView.tsx`.
- Test coverage: `WorkflowTour.test.tsx` checks advancement gates; real browser
  coverage is in `e2e/browserEmbedding.spec.ts`.
- Assumptions and exclusions: browser sandbox execution is not native cell
  verification; the practice objection is not a persisted review thread.
  Reopening an execution step requires rerunning its witness after reload.
- Executed checks: advancement unit test passed. The real tour passed in
  Chromium, Firefox and WebKit. `cargo test -p hickory-cli --test engineer_tour`
  passed: the native engine verifies the corrected witness and rejects the
  original per-attempt budget bug. `tour/scenario.test.ts` checks that the
  downloaded document matches this native-tested fixture.
- Human judgment: not recorded.

- Dependency fingerprint: `apps/web/src/tour/scenario.ts` SHA-256 `97c64168f2cf70cfe4ce00c25b807ce1a2db33f66c82258d5cfb37741b3d2cd2`; changes to this implementation or its cited integration paths require review.
