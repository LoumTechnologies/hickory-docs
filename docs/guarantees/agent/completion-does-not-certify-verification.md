# Completion does not certify verification

Given a built-in turn that edits an executable source closure, when it claims
completion, then the harness requires a successful `verify` for the current
closure revision. Edits and scripts invalidate the result; changes during a
verification refuse a current pass. A model's prose is not an executed result.
An exhausted budget records **unfinished**, preserving the attempted work and
handoff. A non-executable or unverified answer records **not-run**.

Native and Hickory MCP `verify` results are recorded independently as passed or
failed, with their source revision and scope: execution and document expectations.
They certify no semantic truth or completeness. External adapter command reports
remain adapter reports; Hickory cannot enforce their final-answer policy.

---

Last LLM verification:

- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified
- Evidence: `react_loop.rs`, `tools/{mod,preview}.rs`, `doc_tools.rs`, `mcp.rs`.
- Witnesses: `model_failure_audit::claimed_tests_and_failed_verification_cannot_complete_edited_executable_work`,
  existing expectation, turn-budget and stop regressions.
- Challenges: counterfeit pass, failing expectations, mutation after checks,
  script writes, turn budget and opaque adapter commands.
- Exclusions: external data/runtime changes not represented in the source closure,
  expectation quality, semantic claims and opaque adapter tools.
- Human judgment: not recorded.

- Executed checks: see [implementation audit](../audits/2026-10-07-agent-misleading-output.md).
