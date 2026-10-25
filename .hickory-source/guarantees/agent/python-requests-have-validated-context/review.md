<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Python model requests carry validated context manifests: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/agent/src/hickory_agent_worker/context.py](files/apps/agent/src/hickory_agent_worker/context.py.md)
- [apps/agent/src/hickory_agent_worker/core.py](../../verification/private-proposals-wait-for-review/files/apps/agent/src/hickory_agent_worker/core.py.md)
- [apps/agent/src/hickory_agent_worker/hosted.py](../../../support/files/apps/agent/src/hickory_agent_worker/hosted.py.md)
- [apps/agent/src/hickory_agent_worker/runtime.py](../../../support/files/apps/agent/src/hickory_agent_worker/runtime.py.md)
- [crates/hick-context-store/src/lib.rs](../../../shared/files/crates/hick-context-store/src/lib.rs.md)
- [crates/hickory-cli/src/python_agent.rs](../../../shared/files/crates/hickory-cli/src/python_agent.rs.md)

## Tests and fixtures

- [apps/agent/tests/test_context_runtime.py](files/apps/agent/tests/test_context_runtime.py.md)
- [crates/hick-context-store/tests/lineage.rs](../../../shared/files/crates/hick-context-store/tests/lineage.rs.md)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../shared/files/crates/hickory-cli/tests/serve_python_agent.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
