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

October 19 agent-first coach revision:

- The native workflow now uses nonmodal anchored popups over normal controls.
  Investigation/setup precedes the human criterion, reviewed correction, focused
  verification, explicit resolution and publication comparison. Profile work is
  optional. The homepage browser example retains its recorded-proposal labels.
- Reopening reconciles progress against host-retained evidence. A copied task,
  completed turn alone, arbitrary paused source or saved step cannot establish
  a gate. Skip preserves an explicitly unestablished step.
- Successful reviewed edits are correlated with host tool results and hash the
  effective human replacement bytes when the preview was replaced.
- Source-guarded run receipts distinguish requested and actually captured root
  hashes. A failure is not identified from source prose.
- Executed focused checks: four coach component tests, two scenario tests, five
  backend coach tests, the expected-source guard test, three recording-loader
  tests and two landing-page tests passed. These do not establish native visual
  accessibility, a paid provider, a current release or deployed playback.
- ACP UI dispatch is connected; the coach's provider-delivery gate currently
  requires Python's correlated delivery receipt. Universal UI parity and actual
  native/provider/released/deployed witnesses remain outstanding.
- The earlier nine-phase description above documents the previous implementation;
  the current capture procedure is docs/developers/engineer-tour-video.md.

### Coach disclosure follow-up — 2026-10-19

Closed guidance exposes only its launcher. Phase-specific controls retire outside
their actual visible phase, and closing retires guidance/popup commands. Explicit
bounded reads expose the current brief, instructions, skips and evidence gates;
opaque revisions do not disclose private practice paths or task bytes. The source
navigation result says navigation requested, not debugger started. Popup Locate
and Dismiss use shared handlers; hidden/absent targets return a recovery reason.
This adds agent access to the existing coach without sending a task, fabricating
human review, or satisfying any native/provider/released/deployed witness.

Executed focused checks: NativeWorkflowTour (6) and AnchoredCoach (2), all passed.
New coverage drives structured disclosure/read/dismiss, checks hidden commands
retire and checks discovery does not leak the practice path. Existing focus,
Escape, missing-target, retained-evidence and human-criterion checks remain in
those focused files. These are component witnesses, not visual/native footage.


October 22 native-layout correction: before a practice exists, the intro anchors
to its own launcher rather than a closed agent pane. The private coach's request
fields fit 825px and 390px browser viewports and its Locate target focuses the
actual action button without popup overlap of that button. Other form content
may be overlapped until dismissal. Fifteen focused component cases, one synthetic
metadata browser layout flow and targeted TypeScript passed. This is partial LLM
review, not human judgment or a new native package witness. The handoff records
exact source hashes; changing placement/styles invalidates this layout evidence.


## 2026-10-22 anchored native layout correction

The native tour content stays in its anchored popup's normal flow and scroll
bounds; long creation labels and returned paths wrap. Its header, Locate and
Dismiss remain operable instead of being covered by a legacy fixed-position
child. LLM review: partial. Two focused actual-component Chromium layout cases
passed at native/narrow widths; targeted TypeScript passed. The preceding
packaged native creation/End check found the defect and is not proof of the fix.
The issue-72 handoff records exact hashes, native process boundaries and the
synthetic browser responses. No human judgment, paid provider or launch-readiness
claim follows from these checks.
