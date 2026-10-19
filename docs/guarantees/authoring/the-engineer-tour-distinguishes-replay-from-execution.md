# The engineer tour distinguishes replay from execution

Given the AI engineer tour on the homepage or in the application, the same
scenario identifies meeting evidence, a requirement, a recorded agent mistake,
a precise objection, a failing counterexample and a corrected implementation.
Agent proposals are labeled recorded examples. Browser execution uses the product
browser debugger. The counterexample step requires observed failure and the
correction step requires observed success before Continue is enabled.

The browser tour edits no project files. The native tour explicitly creates a
separate practice Git repository in application data and opens a new window when
native shell powers are available; it never implicitly edits the original project.
It can be closed, restarted and reopened;
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


October 19 integrated native workflow extension:

- `serve/engineer_tour.rs` seeds a separate Git repository with labelled VTT,
  deadline mistake, instruction source and real Python hotspot. Native-window
  failure returns a retained path and explicit recovery guidance.
- `NativeWorkflowTour` invokes/reuses the normal importer, shared debugger,
  runtime/profile controls, review/investigation records, context delivery receipts
  and published snapshot viewer. No synthetic model response or profiler result
  substitutes for an unavailable action. Actual Send and edit approval remain
  explicit human actions in the existing product controls.
- A specific 160/100 counterexample and bound paused marker gate execution.
  Structured observation IDs in host-confirmed request manifests gate selected
  debugger/profile delivery on the active conversation ancestry. This establishes
  supplied evidence bytes, not model understanding. Unconfirmed/opaque text does
  not satisfy that witness.
- Native verification receipts name the exact source hash. Changed-source or
  missing-cell runs cannot satisfy the current-source gate. Two completed profile
  runs must have distinct hotspot-source revisions and correctness output; no
  fixed speedup is required. Human resolution remains a separate real action.
- Homepage proposals remain recorded examples and execution remains live browser
  execution. The final page links to actual published results and describes the
  desktop-only path without simulating it.
- Scope limits: source IDs/hashes and receipts are not authenticated semantic
  truth. Full native-window/provider flow and the deployed current revision still
  require a human/agent browser witness before #63 can be called launch-ready.
  Audio capture is optional; no live microphone/model spending was performed.
- Executed checks: three seed/delivery-parser/confinement tests, four focused tour/scenario UI
  tests, native counterexample/correction check and exact native-source receipt
  check. Chromium's real tour and exact source-observation/context witnesses both
  passed; TypeScript compilation passed. These focused checks do not establish
  the entire native-window/provider/deployed flow.
- Video/recovery procedure: `docs/developers/engineer-tour-video.md`.
- Human judgment: no launch-readiness declaration recorded.
