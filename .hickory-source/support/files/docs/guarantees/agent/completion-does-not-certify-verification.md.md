<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/completion-does-not-certify-verification.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f7c20e3135231fe6149882a1577051879a35a7de3fb056d8484ccb2b29a5310b`.

<book:entry path="docs/guarantees/agent/completion-does-not-certify-verification.md">
# Completion does not certify runtime verification

Given a built-in Python turn, successful completion requires a current accepted
literate changeset checked by Hickory against actual captured bytes. A model's
summary, reasoning, snippet prints or claimed checks do not establish acceptance
or an executed result. Iteration exhaustion and rejected completion are incomplete.

The hosted tool surface withholds verify, shell, generic writes and debug
execution because they can bypass exact edit review. Completion is labeled
source-output equality only, with runtime checks unrun. Explicit manual/MCP
verification retains its independent receipts; external ACP reports remain
adapter declarations unless supported by separately observed evidence.

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
- Evidence: `react_loop.rs`, `tools/{mod,preview}.rs`, `doc_tools.rs`, `mcp.rs`.
- Witnesses: `model_failure_audit::claimed_tests_and_failed_verification_cannot_complete_edited_executable_work`,
  existing expectation, turn-budget and stop regressions.
- Challenges: counterfeit pass, failing expectations, mutation after checks,
  script writes, turn budget and opaque adapter commands.
- Exclusions: external data/runtime changes not represented in the source closure,
  expectation quality, semantic claims and opaque adapter tools.
- Human judgment: not recorded.

- Executed checks: see [implementation audit](../audits/2026-10-07-agent-misleading-output.md).
</book:entry>

</book:doc>
