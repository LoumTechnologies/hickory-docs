<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/recording-failure-is-visible.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `ca3c59c0c1701aa6a6286326ece4db092a0473adfe7e791ebff0291d1d3aca41`.

<book:entry path="docs/guarantees/agent/recording-failure-is-visible.md">
# Recording failure is visible

Given a Hickory-handled agent action, when its intent cannot be recorded,
then that action is refused before execution. When recording an already executed
result fails, the caller receives an explicit incomplete-record error and must
inspect actual files; the system does not claim rollback.

A writer retains its first write failure. Before the next native action and at
completion, its health is checked; removed or replaced open session records are
not healthy. ACP recorder and MCP result failures propagate rather than becoming
success with only stderr evidence. Retained turn statuses distinguish completed,
failed and stopped work after restart; a missing terminal status is interrupted.

---

Last LLM verification:

- Date: 2026-10-08
- Reviewer: Codex
- Result: partially verified
- Evidence: `session.rs`, `react_loop.rs`, `doc_tools.rs`, `mcp.rs`,
  `serve/acp/record.rs`, `serve/agent.rs`.
- Witnesses: `session::tests::a_failed_writer_is_sticky_and_a_removed_record_is_not_healthy`,
  native turn/stop/recovery integration tests.
- Challenges: read-only writer, removed/replaced file, failure after effects.
- Exclusions: power-loss persistence, native adapter commands outside the bridge,
  hostile in-place file rewriting, deleted session recovery and rollback.
- Human judgment: not recorded.

- Executed checks: see [implementation audit](../audits/2026-10-07-agent-misleading-output.md).
</book:entry>

</book:doc>
