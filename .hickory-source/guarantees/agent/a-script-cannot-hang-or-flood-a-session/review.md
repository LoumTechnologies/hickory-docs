<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Model snippets cannot hang or flood an agent session indefinitely: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/files/apps/agent/src/hickory_agent_worker/core.py.md)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../support/files/apps/agent/src/hickory_agent_worker/hosted.py.md)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../support/files/apps/agent/src/hickory_agent_worker/runtime.py.md)
- [crates/hick-handlers/src/handlers/script.rs](../../../support/files/crates/hick-handlers/src/handlers/script.rs.md)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../shared/files/crates/hickory-cli/src/serve/python_agent.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve; no-existing-code-or-test-reference-resolved.

</book:doc>
