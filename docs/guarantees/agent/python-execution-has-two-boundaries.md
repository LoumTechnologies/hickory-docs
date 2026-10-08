# Python agent code crosses two explicit boundaries

Given the experimental standalone `hickory-agent` CLI with its installed,
trusted runtime, when a model supplies executable code, then it is Python
interpreted by Monty without host filesystem mounts. The trusted Python worker
also runs in Hickory-managed OS confinement. Provider keys and workspace writes
stay in the Rust supervisor. Unsupported platforms or unavailable confinement
refuse execution.

An exposed workspace edit needs an independently approved proposal. Detected
source changes while awaiting review invalidate that proposal. Requests,
assumptions, responses, proposals, decisions and failures remain in the `.md`
session. Rust-observed tool results and worker-reported observations are labeled
separately. A model completion is reported with checks unrun.

---

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
- Executed checks: `just test-python-agent` passed on macOS 26.6 with pinned
  CPython 3.14.7 and Monty 1.1.0. Linux execution is configured in CI but was not
  observed in this local evaluation. Record the final rerun in the change's git
  history; no live paid-provider call was required for these tests.
- Assumptions/exclusions: trusted installations and OS enforcement are outside
  the model-code boundary. Runtime directories must not contain credentials.
  macOS allows file metadata/root listing for startup. This is not a VM or a
  complete native-code security proof. Windows refuses; UI adoption, bundled
  releases and complete byte authorship are not implemented here.
- Challenges: a filesystem escape, a write without parent approval, an inherited
  key, stale approval applied, a shell/CPython fallback, or a model claim rendered
  as independently verified would falsify the relevant boundary. Arbitrary
  editors racing after the final freshness check require stronger shared writer
  coordination; this guarantee only covers detected review-time changes.
- Dependencies: the committed runtime sources and `apps/agent/uv.lock`; edits
  to either require renewed review and execution of the shared check script.
- Inspected SHA-256 dependencies (2026-10-08):
  - `crates/hickory-cli/src/python_agent.rs`: `5220ff3acd263830fb6248795eb44b7936eda7cf31c726efb2b7fa26ce0d39d1`
  - `crates/hickory-executor-sandbox/src/agent_process.rs`: `1e030edcb725350c25da649192cacbc35faa0e36a24a4119e8532caf5d0af5bc`
  - `apps/agent/src/hickory_agent_worker/__main__.py`: `f99164f232acf4bf85321fcfc646a7c50a0eed42f19e5a28914b260039289d84`
  - `apps/agent/uv.lock`: `f60af405b453a2701f1e91f710c5dd1e76c6d7ef888432845d0a1cb893539ead`
- Human judgment: not recorded; automated test reviewers are fixtures, not
  evidence of a human's semantic approval.
