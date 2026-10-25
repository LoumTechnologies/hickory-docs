<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Separate model snippets from trusted host operations: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/agent/src/hickory_agent_worker/__main__.py](files/apps/agent/src/hickory_agent_worker/__main__.py.md)
- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/files/apps/agent/src/hickory_agent_worker/core.py.md)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../support/files/apps/agent/src/hickory_agent_worker/hosted.py.md)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../support/files/apps/agent/src/hickory_agent_worker/runtime.py.md)
- [apps/web/src/components/AcpControls.tsx](files/apps/web/src/components/AcpControls.tsx.md)
- [apps/web/src/components/AgentChanges.tsx](../../../shared/files/apps/web/src/components/AgentChanges.tsx.md)
- [crates/hickory-agent/src/context.rs](../../../shared/files/crates/hickory-agent/src/context.rs.md)
- [crates/hickory-cli/src/python_agent.rs](../../../shared/files/crates/hickory-cli/src/python_agent.rs.md)
- [crates/hickory-cli/src/serve/acp/edits.rs](../../../shared/files/crates/hickory-cli/src/serve/acp/edits.rs.md)
- [crates/hickory-cli/src/serve/agent.rs](../../../shared/files/crates/hickory-cli/src/serve/agent.rs.md)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../shared/files/crates/hickory-cli/src/serve/python_agent.rs.md)
- [crates/hickory-executor-sandbox/src/agent_process.rs](files/crates/hickory-executor-sandbox/src/agent_process.rs.md)

## Tests and fixtures

- [apps/agent/tests/cli_cancel.py](files/apps/agent/tests/cli_cancel.py.md)
- [apps/agent/tests/test_worker.py](../../../shared/files/apps/agent/tests/test_worker.py.md)
- [apps/web/e2e/python-agent.spec.ts](../../../shared/files/apps/web/e2e/python-agent.spec.ts.md)
- [apps/web/src/components/AgentChanges.test.tsx](../../../shared/files/apps/web/src/components/AgentChanges.test.tsx.md)
- [apps/web/src/components/ChatDock.test.tsx](../../../shared/files/apps/web/src/components/ChatDock.test.tsx.md)
- [apps/web/src/views/AgentSettings.test.tsx](../../../shared/files/apps/web/src/views/AgentSettings.test.tsx.md)
- [crates/hickory-cli/tests/python_agent.rs](../../../shared/files/crates/hickory-cli/tests/python_agent.rs.md)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../shared/files/crates/hickory-cli/tests/serve_python_agent.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
