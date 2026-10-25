<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Context protocol validates declared byte lineage before integration: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/agent/src/hickory_agent_worker/context_protocol.py](files/apps/agent/src/hickory_agent_worker/context_protocol.py.md)
- [apps/agent/src/hickory_agent_worker/context_types.py](files/apps/agent/src/hickory_agent_worker/context_types.py.md)
- [crates/hick-context-protocol/src/lib.rs](files/crates/hick-context-protocol/src/lib.rs.md)
- [crates/hick-context-protocol/src/types.rs](files/crates/hick-context-protocol/src/types.rs.md)
- [scripts/check-codegen.sh](../../../shared/files/scripts/check-codegen.sh.md)

## Tests and fixtures

- [apps/agent/tests/test_context_protocol.py](files/apps/agent/tests/test_context_protocol.py.md)
- [crates/hick-context-protocol/tests/conformance.rs](files/crates/hick-context-protocol/tests/conformance.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
