# Python agent code crosses two explicit boundaries

Given the built-in Python agent launched by Hickory, model snippets execute in
Monty without workspace mounts or CPython/shell fallback. The trusted Python
orchestrator runs under Seatbelt or Bubblewrap. Unsupported platforms and
unavailable confinement refuse execution. Standalone development is explicitly
outside hosted confinement and verification guarantees.

Python owns its graph, prompts, history, inference requests, actions, retries,
iteration limit and completion attempts. Hickory provides bounded inference
transport, allowlisted tools, exact edit review/application and recorded evidence.
Document edit tools require separate approval and freshness checks. Read/edit
weaving stays in memory. Direct shell/filesystem/verify/debug tools are absent
from Monty. Attached-window UI tools expose registered host process, filesystem,
setup, execution and debugger capabilities under existing local host policy.
Those host operations can have workspace effects beyond reviewed document edits.
Standalone CLI runs do not advertise window tools. Stop cancels inference/review
and kills the confined process group without approving a pending document edit;
cancelling a completed UI dispatch does not cancel an independently launched job.

A current independently accepted changeset is required for successful hosted
completion. Worker/model claims cannot establish verification. Equality proves
no runtime behavior, semantic correctness or authorship. Actual tool receipts
and approvals are recorded separately from worker-reported observations.

---

Last LLM implementation review (2026-10-18): partially verified. Requirement,
choices, limitations and executable witnesses are in
[the migration audit](../audits/2026-10-18-python-agent-ownership.md).
Evidence: Python `core.py`, `runtime.py`, `hosted.py`; Rust `python_agent.rs`,
`serve/python_agent.rs`, existing changeset verifier and confinement profile.
Executed checks: see the audit. Human correctness judgment: not recorded.

## Historical evidence for the retired implementation

The following notes predate the migration. Deleted Rust-runtime paths and old
check results do not verify the current implementation.


Last LLM verification:

- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified
- Originating requirement: the user selected a standalone Python agent with
  existing Hickory sandbox mechanisms and Monty for generated code; also required
  durable assumption audits and extra permission for human input.
- Implementation: `python_agent::run` and `Host::handle` own the conversation,
  budgets and receipts; `Host::tool` gates proposals through `Review`;
  `agent_process::command` clears environment and applies an allowlist;
  `hickory_agent_worker.execute` calls Monty with only the broker callback.
- Witnesses: `tests/python_agent.rs` exercises actual confined processes with a
  scripted model, declined/accepted edits, source mutation during approval,
  unsupported execution, workspace read/write denial and network denial.
  `terminal_review_can_be_cancelled_without_waiting_for_input` uses a real PTY
  and a local provider fixture (`apps/agent/tests/cli_cancel.py`) to challenge
  Ctrl-C while approval is pending: the process exits promptly, the session
  records cancellation and the proposed file is never written.
  `apps/agent/tests/test_worker.py` challenges filesystem access, subprocess
  imports, interpreted eval, large allocation, infinite loops, forged replies and
  exact printed text with stream labels kept apart.
  `agent_process::tests` challenges unsafe runtime paths and profile injection.
- UI implementation: `serve/python_agent.rs` probes configured installations,
  routes UI review separately from ACP/native policy, and bridges events through
  the existing run channel. `run_conversation` accepts turn identity and a Stop
  flag without registering a process-wide Ctrl-C handler. Raw model JSON remains
  inert context, while `SessionEvent::Assistant` carries the final labeled summary.
  `context::derive_from_session` recognizes Python request envelopes. New-file
  writes carry hash receipts without erasing their agent origin on approval.
- Failed browser witness and repair: an initial reload check sent its menu event
  before the workspace listener existed. The test now waits for the document
  editor before reopening the pane; the corrected flow passed. This is a test
  synchronization repair, not evidence that provider behavior is safe.
- UI witnesses: `tests/serve_python_agent.rs` exercises accepted/rejected writes,
  rejection reasons, a forbidden auto-accept request, live-room source changes,
  cancellation, token usage, exact branch context and engine restart. The real
  Chromium `e2e/python-agent.spec.ts` uses `python_agent_ui_fixture` only for the
  provider; it drives settings, selection, Accept, reload and Stop against a real
  supervisor and confined worker. React tests cover selection/model controls,
  mandatory review and failed setup feedback.
- Prior CLI review: commit `1146f87` retains the earlier implementation and
  verification record. This revision adds UI adoption and renews its evidence.
- Executed checks: `just test-python-agent` passed on macOS 26.6 with pinned
  CPython 3.14.7 and Monty 1.1.0. Linux execution is configured in CI but was not
  observed in this local evaluation. The shared script includes the real-browser flow. The full React suite
  passed 1,973 tests; native/ACP serve regressions and 102 agent/audit tests
  also passed. No live paid-provider call was required for these tests.
- Assumptions/exclusions: trusted installations and OS enforcement are outside
  the model-code boundary. Runtime directories must not contain credentials.
  macOS allows file metadata/root listing for startup. This is not a VM or a
  complete native-code security proof. Windows refuses; bundled releases and complete byte authorship are not
  implemented here. Browser-local static embeddings do not launch this runtime;
  it needs the local Rust engine. Unsaved buffer context is preserved, but the
  Python tool bridge edits saved documents.
- Challenges: a filesystem escape, a write without parent approval, an inherited
  key, stale approval applied, a shell/CPython fallback, or a model claim rendered
  as independently verified would falsify the relevant boundary. Arbitrary
  editors racing after the final freshness check require stronger shared writer
  coordination; this guarantee only covers detected review-time changes.
- Dependencies: the committed runtime sources and `apps/agent/uv.lock`; edits
  to either require renewed review and execution of the shared check script.
- Inspected SHA-256 dependencies (2026-10-08):
  - `crates/hickory-cli/src/python_agent.rs`: `8fb4fdc51658dfed4d3c7fff95ab95cf0bfa0a61878db79b2fb1099d28259c80`
  - `crates/hickory-executor-sandbox/src/agent_process.rs`: `1e030edcb725350c25da649192cacbc35faa0e36a24a4119e8532caf5d0af5bc`
  - `apps/agent/src/hickory_agent_worker/__main__.py`: `f99164f232acf4bf85321fcfc646a7c50a0eed42f19e5a28914b260039289d84`
  - `apps/agent/uv.lock`: `f60af405b453a2701f1e91f710c5dd1e76c6d7ef888432845d0a1cb893539ead`
  - `crates/hickory-cli/src/serve/python_agent.rs`: `e80e0c9fa379dbafa0ec708390197b6c63a0c095ad5128c1362ee0b03f847e39`
  - `crates/hickory-cli/src/serve/agent.rs`: `e5d48ebfe20d16ca1497cd9cd366bf297ff6fad8e59a8cb6d39fe0d57b62c66e`
  - `crates/hickory-cli/src/serve/acp/edits.rs`: `a1231a556edcc44ea94994c37157becbdb2d5130029a2b5403cac80dd34711bc`
  - `crates/hickory-agent/src/context.rs`: `ae2fdf0c100069fd6c08162292bfd4ed0ca132618f9feb5497c37b095340a3f7`
  - `apps/web/src/components/AcpControls.tsx`: `c767ef20a27f49db84898b64495e2e47e95182e17a7f14fc19c65010cec43ebc`
  - `apps/web/src/components/AgentChanges.tsx`: `048d236918096617519a4f72786e2c1fad16a6661d30de6d09d263c964e9f660`
  - `apps/web/e2e/python-agent.spec.ts`: `587b03eb6cc2e7372ca045dbfbf4d41a75250a861f6b1e085c10c6cd8e3e1a68`
  - `crates/hickory-cli/tests/serve_python_agent.rs`: `ab3340cd7af95fec8168cdce2ffbf151cb488a01287053401222b1a9cb277782`
- Human judgment: not recorded; automated test reviewers are fixtures, not
  evidence of a human's semantic approval.

Issue 72 boundary extension (2026-10-19):

- Origin: the explicit requirement to provide the built-in agent the same real
  run/debug/profile/setup and broader UI capabilities as a human. This supersedes
  the former blanket statement that all workspace effects require edit previews.
  The consequence is intentional host execution authority outside Monty, rather
  than an unconfined Python fallback. Exact document edit review remains.
- Consequential limitation: registered terminals and saved programs execute with
  existing host permissions and may write files. Instructions prohibit evading
  edit review, but this is not an adversarial same-user security boundary.
- The UI broker supplies actor/window identity and journals structured intent and
  results. Known terminal prompts and enabling automatic answers require human
  input; current recognized host prompts are rechecked before agent bytes. Prompt
  guessing cannot recognize every external protocol or future output.
- Executed focused evidence: actual confined Python/window routing with scripted
  provider/UI, registry/controller tests, source-guarded execution and real PTY
  prompt refusal tests. These do not establish a paid provider or native release.
- Review: partial; the issue-72 audit records coverage and remaining work. No
  human correctness or launch-readiness judgment was supplied.
